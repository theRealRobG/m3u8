use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.9
#[derive(Debug)]
pub struct Part<'a> {
    uri: Cow<'a, str>,
    duration: f64,
    independent: Option<bool>,
    byterange: Option<PartByterange>,
    gap: Option<bool>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
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
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        let Some(duration) = (match attribute_list.get(DURATION) {
            Some(a) => a.as_option_f64(),
            _ => None,
        }) else {
            return Err(super::ValidationError::MissingRequiredAttribute(DURATION));
        };
        Ok(Self {
            uri: Cow::Borrowed(uri),
            duration,
            independent: None,
            byterange: None,
            gap: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Part<'a> {
    pub fn new(
        uri: String,
        duration: f64,
        independent: bool,
        byterange: Option<PartByterange>,
        gap: bool,
    ) -> Self {
        let uri = Cow::Owned(uri);
        let output_line = Cow::Owned(calculate_line(&uri, duration, independent, byterange, gap));
        Self {
            uri,
            duration,
            independent: Some(independent),
            byterange,
            gap: Some(gap),
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub(crate) fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn independent(&self) -> bool {
        if let Some(independent) = self.independent {
            independent
        } else {
            matches!(
                self.attribute_list.get(INDEPENDENT),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }

    pub fn byterange(&self) -> Option<PartByterange> {
        if let Some(byterange) = self.byterange {
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
        if let Some(gap) = self.gap {
            gap
        } else {
            matches!(
                self.attribute_list.get(GAP),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }

    pub fn set_uri(&mut self, uri: String) {
        self.attribute_list.remove(URI);
        self.uri = Cow::Owned(uri);
        self.output_line_is_dirty = true;
    }
    pub fn set_duration(&mut self, duration: f64) {
        self.attribute_list.remove(DURATION);
        self.duration = duration;
        self.output_line_is_dirty = true;
    }
    pub fn set_independent(&mut self, independent: bool) {
        self.attribute_list.remove(INDEPENDENT);
        self.independent = Some(independent);
        self.output_line_is_dirty = true;
    }
    pub fn set_byterange(&mut self, byterange: Option<PartByterange>) {
        self.attribute_list.remove(BYTERANGE);
        self.byterange = byterange;
        self.output_line_is_dirty = true;
    }
    pub fn set_gap(&mut self, gap: bool) {
        self.attribute_list.remove(GAP);
        self.gap = Some(gap);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.uri(),
            self.duration(),
            self.independent(),
            self.byterange(),
            self.gap(),
        ));
        self.output_line_is_dirty = false;
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
            Part::new("part.1.0.mp4".to_string(), 0.5, false, None, false)
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_no_byterange_offset_should_be_valid() {
        assert_eq!(
            "#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5,INDEPENDENT=YES,BYTERANGE=1024,GAP=YES",
            Part::new(
                "part.1.0.mp4".to_string(),
                0.5,
                true,
                Some(PartByterange {
                    length: 1024,
                    offset: None
                }),
                true
            )
            .into_inner()
            .value()
        );
    }

    #[test]
    fn as_str_with_options_with_byterange_offset_should_be_valid() {
        assert_eq!(
            "#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5,INDEPENDENT=YES,BYTERANGE=1024@512,GAP=YES",
            Part::new(
                "part.1.0.mp4".to_string(),
                0.5,
                true,
                Some(PartByterange {
                    length: 1024,
                    offset: Some(512)
                }),
                true
            )
            .into_inner()
            .value()
        );
    }
}
