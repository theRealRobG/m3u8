use crate::{
    error::{UnrecognizedEnumerationError, ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{EnumeratedString, into_inner_tag},
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
    utils::AsStaticCow,
};
use std::{borrow::Cow, collections::HashMap, fmt::Display, marker::PhantomData};

/// Corresponds to the `#EXT-X-PRELOAD-HINT:TYPE` attribute.
///
/// See [`PreloadHint`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreloadHintType {
    /// The resource is a Partial Segment.
    Part,
    /// The resource is a Media Initialization Section.
    Map,
}
impl<'a> TryFrom<&'a str> for PreloadHintType {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            PART => Ok(Self::Part),
            MAP => Ok(Self::Map),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for PreloadHintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for PreloadHintType {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            PreloadHintType::Part => Cow::Borrowed(PART),
            PreloadHintType::Map => Cow::Borrowed(MAP),
        }
    }
}
impl From<PreloadHintType> for Cow<'_, str> {
    fn from(value: PreloadHintType) -> Self {
        value.as_cow()
    }
}
impl From<PreloadHintType> for EnumeratedString<'_, PreloadHintType> {
    fn from(value: PreloadHintType) -> Self {
        Self::Known(value)
    }
}
const PART: &str = "PART";
const MAP: &str = "MAP";

/// The attribute list for the tag (`#EXT-X-PRELOAD-HINT:<attribute-list>`).
///
/// See [`PreloadHint`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct PreloadHintAttributeList<'a> {
    /// Corresponds to the `TYPE` attribute.
    ///
    /// See [`PreloadHint`] for a link to the HLS documentation for this attribute.
    hint_type: Cow<'a, str>,
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`PreloadHint`] for a link to the HLS documentation for this attribute.
    uri: Cow<'a, str>,
    /// Corresponds to the `BYTERANGE-START` attribute.
    ///
    /// See [`PreloadHint`] for a link to the HLS documentation for this attribute.
    byterange_start: Option<u64>,
    /// Corresponds to the `BYTERANGE-LENGTH` attribute.
    ///
    /// See [`PreloadHint`] for a link to the HLS documentation for this attribute.
    byterange_length: Option<u64>,
}

/// Placeholder struct for [`PreloadHintBuilder`] indicating that `hint_type` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct PreloadHintTypeNeedsToBeSet;
/// Placeholder struct for [`PreloadHintBuilder`] indicating that `uri` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct PreloadHintUriNeedsToBeSet;
/// Placeholder struct for [`PreloadHintBuilder`] indicating that `hint_type` has been set.
#[derive(Debug, Clone, Copy)]
pub struct PreloadHintTypeHasBeenSet;
/// Placeholder struct for [`PreloadHintBuilder`] indicating that `uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct PreloadHintUriHasBeenSet;

/// A builder for convenience in constructing a [`PreloadHint`].
#[derive(Debug, Clone)]
pub struct PreloadHintBuilder<'a, TypeStatus, UriStatus> {
    attribute_list: PreloadHintAttributeList<'a>,
    type_status: PhantomData<TypeStatus>,
    uri_status: PhantomData<UriStatus>,
}
impl<'a> PreloadHintBuilder<'a, PreloadHintTypeNeedsToBeSet, PreloadHintUriNeedsToBeSet> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: PreloadHintAttributeList {
                hint_type: Cow::Borrowed(""),
                uri: Cow::Borrowed(""),
                byterange_start: Default::default(),
                byterange_length: Default::default(),
            },
            type_status: PhantomData,
            uri_status: PhantomData,
        }
    }
}
impl<'a> PreloadHintBuilder<'a, PreloadHintTypeHasBeenSet, PreloadHintUriHasBeenSet> {
    /// Finish building and construct the `PreloadHint`.
    pub fn finish(self) -> PreloadHint<'a> {
        PreloadHint::new(self.attribute_list)
    }
}
impl<'a, TypeStatus, UriStatus> PreloadHintBuilder<'a, TypeStatus, UriStatus> {
    /// Add the provided `hint_type` to the attributes built into `PreloadHint`.
    pub fn with_hint_type(
        mut self,
        hint_type: impl Into<Cow<'a, str>>,
    ) -> PreloadHintBuilder<'a, PreloadHintTypeHasBeenSet, UriStatus> {
        self.attribute_list.hint_type = hint_type.into();
        PreloadHintBuilder {
            attribute_list: self.attribute_list,
            type_status: PhantomData,
            uri_status: PhantomData,
        }
    }

    /// Add the provided `uri` to the attributes built into `PreloadHint`.
    pub fn with_uri(
        mut self,
        uri: impl Into<Cow<'a, str>>,
    ) -> PreloadHintBuilder<'a, TypeStatus, PreloadHintUriHasBeenSet> {
        self.attribute_list.uri = uri.into();
        PreloadHintBuilder {
            attribute_list: self.attribute_list,
            type_status: PhantomData,
            uri_status: PhantomData,
        }
    }

    /// Add the provided `byterange_start` to the attributes built into `PreloadHint`.
    pub fn with_byterange_start(mut self, byterange_start: u64) -> Self {
        self.attribute_list.byterange_start = Some(byterange_start);
        self
    }

    /// Add the provided `byterange_length` to the attributes built into `PreloadHint`.
    pub fn with_byterange_length(mut self, byterange_length: u64) -> Self {
        self.attribute_list.byterange_length = Some(byterange_length);
        self
    }
}

/// Corresponds to the `#EXT-X-PRELOAD-HINT` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.3>
#[derive(Debug, Clone)]
pub struct PreloadHint<'a> {
    hint_type: Cow<'a, str>,
    uri: Cow<'a, str>,
    byterange_start: Option<u64>,
    byterange_length: Option<u64>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for PreloadHint<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.hint_type() == other.hint_type()
            && self.uri() == other.uri()
            && self.byterange_start() == other.byterange_start()
            && self.byterange_length() == other.byterange_length()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for PreloadHint<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::UnquotedString(hint_type)) = attribute_list.get(TYPE) else {
            return Err(ValidationError::MissingRequiredAttribute(TYPE));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(ValidationError::MissingRequiredAttribute(URI));
        };
        Ok(Self {
            hint_type: Cow::Borrowed(hint_type),
            uri: Cow::Borrowed(uri),
            byterange_start: None,
            byterange_length: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> PreloadHint<'a> {
    /// Constructs a new `PreloadHint` tag.
    fn new(attribute_list: PreloadHintAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let PreloadHintAttributeList {
            hint_type,
            uri,
            byterange_start,
            byterange_length,
        } = attribute_list;
        Self {
            hint_type,
            uri,
            byterange_start,
            byterange_length,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `PreloadHint` as such:
    /// ```
    /// # use m3u8::tag::hls::{PreloadHint, PreloadHintType};
    /// let preload_hint = PreloadHint::builder()
    ///     .with_hint_type(PreloadHintType::Part)
    ///     .with_uri("part.100.2.mp4")
    ///     .with_byterange_start(1024)
    ///     .with_byterange_length(1024)
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `hint_type` AND `uri`.
    /// Each of the following fail to compile:
    /// ```compile_fail
    /// # use m3u8::tag::hls::PreloadHint;
    /// let preload_hint = PreloadHint::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::PreloadHint;
    /// let preload_hint = PreloadHint::builder().with_hint_type(PreloadHintType::Part).finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::PreloadHint;
    /// let preload_hint = PreloadHint::builder().with_uri("part.100.2.mp4").finish();
    /// ```
    pub fn builder()
    -> PreloadHintBuilder<'a, PreloadHintTypeNeedsToBeSet, PreloadHintUriNeedsToBeSet> {
        PreloadHintBuilder::new()
    }

    /// Corresponds to the `TYPE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn hint_type(&self) -> EnumeratedString<PreloadHintType> {
        EnumeratedString::from(self.hint_type.as_ref())
    }

    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Corresponds to the `BYTERANGE-START` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn byterange_start(&self) -> u64 {
        if let Some(byterange_start) = self.byterange_start {
            byterange_start
        } else {
            match self.attribute_list.get(BYTERANGE_START) {
                Some(ParsedAttributeValue::DecimalInteger(start)) => *start,
                _ => 0,
            }
        }
    }

    /// Corresponds to the `BYTERANGE-LENGTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn byterange_length(&self) -> Option<u64> {
        if let Some(byterange_length) = self.byterange_length {
            Some(byterange_length)
        } else {
            match self.attribute_list.get(BYTERANGE_LENGTH) {
                Some(ParsedAttributeValue::DecimalInteger(length)) => Some(*length),
                _ => None,
            }
        }
    }

    /// Sets the `TYPE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_hint_type(&mut self, hint_type: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(TYPE);
        self.hint_type = hint_type.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `BYTERANGE-START` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_byterange_start(&mut self, byterange_start: u64) {
        self.attribute_list.remove(BYTERANGE_START);
        self.byterange_start = Some(byterange_start);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `BYTERANGE-START` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_byterange_start(&mut self) {
        self.attribute_list.remove(BYTERANGE_START);
        self.byterange_start = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `BYTERANGE-LENGTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_byterange_length(&mut self, byterange_length: u64) {
        self.attribute_list.remove(BYTERANGE_LENGTH);
        self.byterange_length = Some(byterange_length);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `BYTERANGE-LENGTH` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_byterange_length(&mut self) {
        self.attribute_list.remove(BYTERANGE_LENGTH);
        self.byterange_length = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        let byterange_start = if self.byterange_start() == 0 {
            None
        } else {
            Some(self.byterange_start())
        };
        self.output_line = Cow::Owned(calculate_line(&PreloadHintAttributeList {
            hint_type: self.hint_type().into(),
            uri: self.uri().into(),
            byterange_start,
            byterange_length: self.byterange_length(),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(PreloadHint);

const TYPE: &str = "TYPE";
const URI: &str = "URI";
const BYTERANGE_START: &str = "BYTERANGE-START";
const BYTERANGE_LENGTH: &str = "BYTERANGE-LENGTH";

fn calculate_line(attribute_list: &PreloadHintAttributeList) -> Vec<u8> {
    let PreloadHintAttributeList {
        hint_type,
        uri,
        byterange_start,
        byterange_length,
    } = attribute_list;
    let mut line = format!("#EXT-X-PRELOAD-HINT:{TYPE}={hint_type},URI=\"{uri}\"");
    if let Some(byterange_start) = byterange_start {
        line.push_str(format!(",{BYTERANGE_START}={byterange_start}").as_str());
    }
    if let Some(byterange_length) = byterange_length {
        line.push_str(format!(",{BYTERANGE_LENGTH}={byterange_length}").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\"",
            PreloadHint::builder()
                .with_hint_type(PreloadHintType::Part)
                .with_uri("part.2.mp4")
                .finish()
                .into_inner()
                .value()
        )
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\",BYTERANGE-START=512,BYTERANGE-LENGTH=1024",
            PreloadHint::builder()
                .with_hint_type(PreloadHintType::Part)
                .with_uri("part.2.mp4")
                .with_byterange_start(512)
                .with_byterange_length(1024)
                .finish()
                .into_inner()
                .value()
        )
    }

    mutation_tests!(
        PreloadHint::builder()
            .with_hint_type(PreloadHintType::Map)
            .with_uri("init.mp4")
            .with_byterange_start(512)
            .with_byterange_length(1024)
            .finish(),
        (hint_type, EnumeratedString::Known(PreloadHintType::Part), @Attr="TYPE=PART"),
        (uri, "part.2.mp4", @Attr="URI=\"part.2.mp4\""),
        (byterange_start, 100; @Default=0, @Attr="BYTERANGE-START=100"),
        (byterange_length, @Option 200, @Attr="BYTERANGE-LENGTH=200")
    );
}
