use crate::tag::{
    hls::TagInner,
    known::ParsedTag,
    value::{ParsedAttributeValue, ParsedTagValue},
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5
#[derive(Debug)]
pub struct Map<'a> {
    uri: Cow<'a, str>,
    byterange: Option<MapByterange>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
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
            uri: Cow::Borrowed(uri),
            byterange: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Map<'a> {
    pub fn new(uri: String, byterange: Option<MapByterange>) -> Self {
        let uri = Cow::Owned(uri);
        let output_line = Cow::Owned(calculate_line(&uri, byterange));
        Self {
            uri,
            byterange,
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

    pub fn byterange(&self) -> Option<MapByterange> {
        if let Some(byterange) = self.byterange {
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

    pub fn set_uri(&mut self, uri: String) {
        self.attribute_list.remove(URI);
        self.uri = Cow::Owned(uri);
        self.output_line_is_dirty = true;
    }

    pub fn set_byterange(&mut self, byterange: Option<MapByterange>) {
        self.attribute_list.remove(BYTERANGE);
        self.byterange = byterange;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&self.uri().into(), self.byterange()));
        self.output_line_is_dirty = false;
    }
}

const URI: &str = "URI";
const BYTERANGE: &str = "BYTERANGE";

fn calculate_line<'a>(uri: &Cow<'a, str>, byterange: Option<MapByterange>) -> String {
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
            Map::new("example.mp4".to_string(), None)
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_byterange_should_be_valid() {
        assert_eq!(
            "#EXT-X-MAP:URI=\"example.mp4\",BYTERANGE=\"1024@512\"",
            Map::new(
                "example.mp4".to_string(),
                Some(MapByterange {
                    length: 1024,
                    offset: 512
                })
            )
            .into_inner()
            .value()
        );
    }
}
