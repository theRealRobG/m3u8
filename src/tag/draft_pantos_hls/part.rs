use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.9
#[derive(Debug)]
pub struct Part<'a> {
    uri: &'a str,
    duration: f64,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
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

impl<'a> PartialEq for Part<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri() == other.uri()
            && self.duration() == other.duration()
            && self.independent() == other.independent()
            && self.byterange() == other.byterange()
            && self.gap() == other.gap()
    }
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Part<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
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
}

const URI: &'static str = "URI";
const DURATION: &'static str = "DURATION";
const INDEPENDENT: &'static str = "INDEPENDENT";
const BYTERANGE: &'static str = "BYTERANGE";
const GAP: &'static str = "GAP";
const YES: &'static str = "YES";
