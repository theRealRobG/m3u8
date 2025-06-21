use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5
#[derive(Debug)]
pub struct Map<'a> {
    uri: &'a str,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
impl Display for MapByterange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.length, self.offset)
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Map<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            uri,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
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
            output_line: Cow::Owned(calculate_line(uri, byterange).into_bytes()),
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

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const URI: &str = "URI";
const BYTERANGE: &str = "BYTERANGE";

fn calculate_line(uri: &str, byterange: Option<MapByterange>) -> String {
    let mut line = format!("#EXT-X-MAP:{URI}=\"{uri}\"");
    if let Some(byterange) = byterange {
        line.push_str(format!(",{BYTERANGE}=\"{byterange}\"").as_str());
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_no_byterange_should_be_valid() {
        assert_eq!(
            "#EXT-X-MAP:URI=\"example.mp4\"",
            Map::new("example.mp4", None).as_str()
        );
    }

    #[test]
    fn as_str_with_byterange_should_be_valid() {
        assert_eq!(
            "#EXT-X-MAP:URI=\"example.mp4\",BYTERANGE=\"1024@512\"",
            Map::new(
                "example.mp4",
                Some(MapByterange {
                    length: 1024,
                    offset: 512
                })
            )
            .as_str()
        );
    }
}
