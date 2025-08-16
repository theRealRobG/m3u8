use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{AttributeValue, UnknownTag, hls::into_inner_tag},
};
use std::{borrow::Cow, collections::HashMap, fmt::Display, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-MAP:<attribute-list>`).
///
/// See [`Map`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct MapAttributeList<'a> {
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Map`] for a link to the HLS documentation for this attribute.
    uri: Cow<'a, str>,
    /// Corresponds to the `BYTERANGE` attribute.
    ///
    /// See [`Map`] for a link to the HLS documentation for this attribute.
    byterange: Option<MapByterange>,
}

/// Placeholder struct for [`MapBuilder`] indicating that `uri` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct MapUriNeedsToBeSet;
/// Placeholder struct for [`MapBuilder`] indicating that `uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct MapUriHasBeenSet;

/// A builder for convenience in constructing a [`Map`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct MapBuilder<'a, UriStatus> {
    attribute_list: MapAttributeList<'a>,
    uri_status: PhantomData<UriStatus>,
}
impl<'a> MapBuilder<'a, MapUriNeedsToBeSet> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: MapAttributeList {
                uri: Cow::Borrowed(""),
                byterange: Default::default(),
            },
            uri_status: PhantomData,
        }
    }
}
impl<'a> MapBuilder<'a, MapUriHasBeenSet> {
    /// Finish building and construct the `Map`.
    pub fn finish(self) -> Map<'a> {
        Map::new(self.attribute_list)
    }
}
impl<'a, UriStatus> MapBuilder<'a, UriStatus> {
    /// Add the provided `uri` to the attributes built into `Map`.
    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> MapBuilder<'a, MapUriHasBeenSet> {
        self.attribute_list.uri = uri.into();
        MapBuilder {
            attribute_list: self.attribute_list,
            uri_status: PhantomData,
        }
    }

    /// Add the provided `byterange` to the attributes built into `Map`.
    pub fn with_byterange(mut self, byterange: MapByterange) -> Self {
        self.attribute_list.byterange = Some(byterange);
        self
    }
}
impl<'a> Default for MapBuilder<'a, MapUriNeedsToBeSet> {
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-MAP` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5>
#[derive(Debug, Clone)]
pub struct Map<'a> {
    uri: Cow<'a, str>,
    byterange: Option<MapByterange>,
    attribute_list: HashMap<&'a str, AttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                           // Used with Writer
    output_line_is_dirty: bool,                           // If should recalculate output_line
}

impl<'a> PartialEq for Map<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri && self.byterange() == other.byterange()
    }
}

/// Corresponds to the value of the `#EXT-X-MAP:BYTERANGE` attribute.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MapByterange {
    /// Corresponds to the length component in the value (`n` in `<n>@<o>`).
    ///
    /// See [`Map`] for a link to the HLS documentation for this attribute.
    pub length: u64,
    /// Corresponds to the offset component in the value (`o` in `<n>@<o>`).
    ///
    /// See [`Map`] for a link to the HLS documentation for this attribute.
    pub offset: u64,
}
impl Display for MapByterange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.length, self.offset)
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for Map<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let mut attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_attribute_list()?;
        let Some(AttributeValue::Quoted(uri)) = attribute_list.remove(URI) else {
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
    /// Constructs a new `Map` tag.
    fn new(attribute_list: MapAttributeList<'a>) -> Self {
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

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `Map` as such:
    /// ```
    /// # use quick_m3u8::tag::hls::{Map, MapByterange};
    /// let map = Map::builder()
    ///     .with_uri("uri")
    ///     .with_byterange(MapByterange { length: 1024, offset: 0 })
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `uri`. The following
    /// will fail to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::Map;
    /// let map = Map::builder().finish();
    /// ```
    pub fn builder() -> MapBuilder<'a, MapUriNeedsToBeSet> {
        MapBuilder::new()
    }

    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Corresponds to the `BYTERANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn byterange(&self) -> Option<MapByterange> {
        if let Some(byterange) = self.byterange {
            Some(byterange)
        } else {
            self.attribute_list
                .get(BYTERANGE)
                .and_then(AttributeValue::quoted)
                .and_then(|byterange_str| {
                    let mut parts = byterange_str.splitn(2, '@');
                    let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    let Some(Ok(offset)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    Some(MapByterange { length, offset })
                })
        }
    }

    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `BYTERANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_byterange(&mut self, byterange: MapByterange) {
        self.attribute_list.remove(BYTERANGE);
        self.byterange = Some(byterange);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `BYTERANGE` attribute (set it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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

into_inner_tag!(Map);

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
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_no_byterange_should_be_valid() {
        assert_eq!(
            b"#EXT-X-MAP:URI=\"example.mp4\"",
            Map::builder()
                .with_uri("example.mp4")
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_byterange_should_be_valid() {
        assert_eq!(
            b"#EXT-X-MAP:URI=\"example.mp4\",BYTERANGE=\"1024@512\"",
            Map::builder()
                .with_uri("example.mp4")
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
        Map::builder()
            .with_uri("example.mp4")
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
