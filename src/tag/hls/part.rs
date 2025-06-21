use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.9
#[derive(Debug)]
pub struct Part<'a> {
    uri: &'a str,
    duration: f64,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    // This needs to exist because the user can construct a Part with `Part::new()`, but will pass a
    // `PartByteRange`, not a `&str`. I can't convert a `PartByteRange` to a `&str` and so need to
    // store it as is for later use.
    stored_byterange: Option<PartByterange>,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PartByterange {
    pub length: u64,
    pub offset: Option<u64>,
}
impl Display for PartByterange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(offset) = self.offset {
            write!(f, "{}@{}", self.length, offset)
        } else {
            write!(f, "{}", self.length)
        }
    }
}

impl<'a> PartialEq for Part<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri() == other.uri()
            && self.duration() == other.duration()
            && self.independent() == other.independent()
            && self.byterange() == other.byterange()
            && self.gap() == other.gap()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Part<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(duration) = (match attribute_list.get(DURATION) {
            Some(a) => a.as_option_f64(),
            _ => None,
        }) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            uri,
            duration,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            stored_byterange: None,
        })
    }
}

impl<'a> Part<'a> {
    pub fn new(
        uri: &'a str,
        duration: f64,
        independent: bool,
        byterange: Option<PartByterange>,
        gap: bool,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        attribute_list.insert(
            DURATION,
            ParsedAttributeValue::SignedDecimalFloatingPoint(duration),
        );
        if independent {
            attribute_list.insert(INDEPENDENT, ParsedAttributeValue::UnquotedString(YES));
        }
        if gap {
            attribute_list.insert(GAP, ParsedAttributeValue::UnquotedString(YES));
        }
        Self {
            uri,
            duration,
            attribute_list,
            output_line: Cow::Owned(
                calculate_line(uri, duration, independent, byterange, gap).into_bytes(),
            ),
            stored_byterange: byterange,
        }
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn independent(&self) -> bool {
        matches!(
            self.attribute_list.get(INDEPENDENT),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }

    pub fn byterange(&self) -> Option<PartByterange> {
        if let Some(byterange) = self.stored_byterange {
            Some(byterange)
        } else {
            match self.attribute_list.get(BYTERANGE) {
                Some(ParsedAttributeValue::QuotedString(range)) => {
                    let mut parts = range.split('@');
                    let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    let offset = match parts.next().map(str::parse::<u64>) {
                        Some(Ok(d)) => Some(d),
                        None => None,
                        Some(Err(_)) => return None,
                    };
                    if parts.next().is_some() {
                        return None;
                    }
                    Some(PartByterange { length, offset })
                }
                _ => None,
            }
        }
    }

    pub fn gap(&self) -> bool {
        matches!(
            self.attribute_list.get(GAP),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const URI: &str = "URI";
const DURATION: &str = "DURATION";
const INDEPENDENT: &str = "INDEPENDENT";
const BYTERANGE: &str = "BYTERANGE";
const GAP: &str = "GAP";
const YES: &str = "YES";

fn calculate_line(
    uri: &str,
    duration: f64,
    independent: bool,
    byterange: Option<PartByterange>,
    gap: bool,
) -> String {
    let mut line = format!("#EXT-X-PART:{URI}=\"{uri}\",{DURATION}={duration}");
    if independent {
        line.push_str(format!(",{INDEPENDENT}={YES}").as_str());
    }
    if let Some(byterange) = byterange {
        line.push_str(format!(",{BYTERANGE}={byterange}").as_str());
    }
    if gap {
        line.push_str(format!(",{GAP}={YES}").as_str());
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5",
            Part::new("part.1.0.mp4", 0.5, false, None, false).as_str()
        );
    }

    #[test]
    fn as_str_with_options_no_byterange_offset_should_be_valid() {
        assert_eq!(
            "#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5,INDEPENDENT=YES,BYTERANGE=1024,GAP=YES",
            Part::new(
                "part.1.0.mp4",
                0.5,
                true,
                Some(PartByterange {
                    length: 1024,
                    offset: None
                }),
                true
            )
            .as_str()
        );
    }

    #[test]
    fn as_str_with_options_with_byterange_offset_should_be_valid() {
        assert_eq!(
            "#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5,INDEPENDENT=YES,BYTERANGE=1024@512,GAP=YES",
            Part::new(
                "part.1.0.mp4",
                0.5,
                true,
                Some(PartByterange {
                    length: 1024,
                    offset: Some(512)
                }),
                true
            )
            .as_str()
        );
    }
}
