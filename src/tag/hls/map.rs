use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq, Clone)]
pub struct MapAttributeList<'a> {
    pub uri: Cow<'a, str>,
    pub byterange: Option<MapByterange>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MapBuilder<'a> {
    uri: Cow<'a, str>,
    byterange: Option<MapByterange>,
}
impl<'a> MapBuilder<'a> {
    pub fn new(uri: impl Into<Cow<'a, str>>) -> Self {
        Self {
            uri: uri.into(),
            byterange: Default::default(),
        }
    }

    pub fn finish(self) -> Map<'a> {
        Map::new(MapAttributeList {
            uri: self.uri,
            byterange: self.byterange,
        })
    }

    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> Self {
        self.uri = uri.into();
        self
    }

    pub fn with_byterange(mut self, byterange: MapByterange) -> Self {
        self.byterange = Some(byterange);
        self
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5
#[derive(Debug, Clone)]
pub struct Map<'a> {
    uri: Cow<'a, str>,
    byterange: Option<MapByterange>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove(URI) else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
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
    pub fn new(attribute_list: MapAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let MapAttributeList { uri, byterange } = attribute_list;
        Self {
            uri,
            byterange,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn builder(uri: impl Into<Cow<'a, str>>) -> MapBuilder<'a> {
        MapBuilder::new(uri)
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
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
                    let mut parts = byterange_str.splitn(2, '@');
                    let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    let Some(Ok(offset)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    Some(MapByterange { length, offset })
                }
                _ => None,
            }
        }
    }

    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_byterange(&mut self, byterange: MapByterange) {
        self.attribute_list.remove(BYTERANGE);
        self.byterange = Some(byterange);
        self.output_line_is_dirty = true;
    }

    pub fn unset_byterange(&mut self) {
        self.attribute_list.remove(BYTERANGE);
        self.byterange = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&MapAttributeList {
            uri: self.uri().into(),
            byterange: self.byterange(),
        }));
        self.output_line_is_dirty = false;
    }
}

const URI: &str = "URI";
const BYTERANGE: &str = "BYTERANGE";

fn calculate_line(attribute_list: &MapAttributeList) -> Vec<u8> {
    let MapAttributeList { uri, byterange } = attribute_list;
    let mut line = format!("#EXT-X-MAP:{URI}=\"{uri}\"");
    if let Some(byterange) = byterange {
        line.push_str(format!(",{BYTERANGE}=\"{byterange}\"").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use crate::tag::hls::test_macro::mutation_tests;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_no_byterange_should_be_valid() {
        assert_eq!(
            b"#EXT-X-MAP:URI=\"example.mp4\"",
            Map::builder("example.mp4").finish().into_inner().value()
        );
    }

    #[test]
    fn as_str_with_byterange_should_be_valid() {
        assert_eq!(
            b"#EXT-X-MAP:URI=\"example.mp4\",BYTERANGE=\"1024@512\"",
            Map::builder("example.mp4")
                .with_byterange(MapByterange {
                    length: 1024,
                    offset: 512
                })
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        Map::builder("example.mp4")
            .with_byterange(MapByterange {
                length: 1024,
                offset: 512
            })
            .finish(),
        (uri, "init.mp4", @Attr="URI=\"init.mp4\""),
        (
            byterange,
            @Option MapByterange { length: 100, offset: 200 },
            @Attr="BYTERANGE=\"100@200\""
        )
    );
}
