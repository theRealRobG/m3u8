use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5
#[derive(Debug)]
pub struct Map<'a> {
    uri: &'a str,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
    // This needs to exist because the user can construct a Map with `Map::new()`, but will pass a
    // `MapByteRange`, not a `&str`. I can't convert a `MapByteRange` to a `&str` and so need to
    // store it as is for later use.
    stored_byterange: Option<MapByterange>,
}

impl<'a> PartialEq for Map<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri && self.byterange() == other.byterange()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MapByterange {
    pub length: u64,
    pub offset: u64,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Map<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            uri,
            attribute_list,
            stored_byterange: None,
        })
    }
}

impl<'a> Map<'a> {
    pub fn new(uri: &'a str, byterange: Option<MapByterange>) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        Self {
            uri,
            attribute_list,
            stored_byterange: byterange,
        }
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn byterange(&self) -> Option<MapByterange> {
        if let Some(byterange) = self.stored_byterange {
            Some(byterange)
        } else {
            match self.attribute_list.get(BYTERANGE) {
                Some(ParsedAttributeValue::QuotedString(byterange_str)) => {
                    let mut parts = byterange_str.split('@');
                    let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    let Some(Ok(offset)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    if parts.next().is_some() {
                        return None;
                    }
                    Some(MapByterange { length, offset })
                }
                _ => None,
            }
        }
    }
}

const URI: &'static str = "URI";
const BYTERANGE: &'static str = "BYTERANGE";
