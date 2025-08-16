use crate::{
    error::{ParseTagValueError, UnrecognizedEnumerationError, ValidationError},
    tag::{
        AttributeValue, UnknownTag,
        hls::{EnumeratedString, into_inner_tag},
    },
    utils::AsStaticCow,
};
use std::{borrow::Cow, collections::HashMap, fmt::Display, marker::PhantomData};

/// Corresponds to the `#EXT-X-SESSION-DATA:FORMAT` attribute.
///
/// See [`SessionData`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    /// The URI MUST reference a JSON (RFC8259) format file.
    Json,
    /// The URI SHALL be treated as a binary file.
    Raw,
}
impl<'a> TryFrom<&'a str> for Format {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            JSON => Ok(Self::Json),
            RAW => Ok(Self::Raw),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for Format {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Json => Cow::Borrowed(JSON),
            Self::Raw => Cow::Borrowed(RAW),
        }
    }
}
impl From<Format> for Cow<'_, str> {
    fn from(value: Format) -> Self {
        value.as_cow()
    }
}
impl From<Format> for EnumeratedString<'_, Format> {
    fn from(value: Format) -> Self {
        Self::Known(value)
    }
}
const JSON: &str = "JSON";
const RAW: &str = "RAW";

/// The attribute list for the tag (`#EXT-X-SESSION-DATA:<attribute-list>`).
///
/// See [`SessionData`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct SessionDataAttributeList<'a> {
    /// Corresponds to the `DATA-ID` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    data_id: Cow<'a, str>,
    /// Corresponds to the `VALUE` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    value: Option<Cow<'a, str>>,
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    uri: Option<Cow<'a, str>>,
    /// Corresponds to the `FORMAT` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    format: Option<Cow<'a, str>>,
    /// Corresponds to the `LANGUAGE` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    language: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`SessionDataBuilder`] indicating that `data_id` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct SessionDataDataIdNeedsToBeSet;
/// Placeholder struct for [`SessionDataBuilder`] indicating that `data_id` has been set.
#[derive(Debug, Clone, Copy)]
pub struct SessionDataDataIdHasBeenSet;
/// Placeholder struct for [`SessionDataBuilder`] indicating that `value` is not set.
#[derive(Debug, Clone, Copy)]
pub struct SessionDataValueIsNotSet;
/// Placeholder struct for [`SessionDataBuilder`] indicating that `value` has been set.
#[derive(Debug, Clone, Copy)]
pub struct SessionDataValueHasBeenSet;
/// Placeholder struct for [`SessionDataBuilder`] indicating that `uri` is not set.
#[derive(Debug, Clone, Copy)]
pub struct SessionDataUriIsNotSet;
/// Placeholder struct for [`SessionDataBuilder`] indicating that `uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct SessionDataUriHasBeenSet;

/// A builder for convenience in constructing a [`SessionData`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct SessionDataBuilder<'a, DataIdStatus, ValueStatus, UriStatus> {
    attribute_list: SessionDataAttributeList<'a>,
    data_id_status: PhantomData<DataIdStatus>,
    value_status: PhantomData<ValueStatus>,
    uri_status: PhantomData<UriStatus>,
}
impl<'a>
    SessionDataBuilder<
        'a,
        SessionDataDataIdNeedsToBeSet,
        SessionDataValueIsNotSet,
        SessionDataUriIsNotSet,
    >
{
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: SessionDataAttributeList {
                data_id: Cow::Borrowed(""),
                value: Default::default(),
                uri: Default::default(),
                format: Default::default(),
                language: Default::default(),
            },
            data_id_status: PhantomData,
            value_status: PhantomData,
            uri_status: PhantomData,
        }
    }
}
impl<'a>
    SessionDataBuilder<
        'a,
        SessionDataDataIdHasBeenSet,
        SessionDataValueHasBeenSet,
        SessionDataUriIsNotSet,
    >
{
    /// Finish building and construct the `SessionData`.
    ///
    /// Each EXT-X-SESSION-DATA tag MUST contain either a VALUE or URI attribute, but not both.
    pub fn finish(self) -> SessionData<'a> {
        SessionData::new(self.attribute_list)
    }
}
impl<'a>
    SessionDataBuilder<
        'a,
        SessionDataDataIdHasBeenSet,
        SessionDataValueIsNotSet,
        SessionDataUriHasBeenSet,
    >
{
    /// Finish building and construct the `SessionData`.
    ///
    /// Each EXT-X-SESSION-DATA tag MUST contain either a VALUE or URI attribute, but not both.
    pub fn finish(self) -> SessionData<'a> {
        SessionData::new(self.attribute_list)
    }
}
impl<'a, DataIdStatus, ValueStatus>
    SessionDataBuilder<'a, DataIdStatus, ValueStatus, SessionDataUriIsNotSet>
{
    /// Add the provided `value` to the attributes built into `SessionData`.
    pub fn with_value(
        mut self,
        value: impl Into<Cow<'a, str>>,
    ) -> SessionDataBuilder<'a, DataIdStatus, SessionDataValueHasBeenSet, SessionDataUriIsNotSet>
    {
        self.attribute_list.value = Some(value.into());
        SessionDataBuilder {
            attribute_list: self.attribute_list,
            data_id_status: PhantomData,
            value_status: PhantomData,
            uri_status: PhantomData,
        }
    }
}
impl<'a, DataIdStatus, UriStatus>
    SessionDataBuilder<'a, DataIdStatus, SessionDataValueIsNotSet, UriStatus>
{
    /// Add the provided `uri` to the attributes built into `SessionData`.
    pub fn with_uri(
        mut self,
        uri: impl Into<Cow<'a, str>>,
    ) -> SessionDataBuilder<'a, DataIdStatus, SessionDataValueIsNotSet, SessionDataUriHasBeenSet>
    {
        self.attribute_list.uri = Some(uri.into());
        SessionDataBuilder {
            attribute_list: self.attribute_list,
            data_id_status: PhantomData,
            value_status: PhantomData,
            uri_status: PhantomData,
        }
    }
}
impl<'a, DataIdStatus, ValueStatus, UriStatus>
    SessionDataBuilder<'a, DataIdStatus, ValueStatus, UriStatus>
{
    /// Add the provided `data_id` to the attributes built into `SessionData`.
    pub fn with_data_id(
        mut self,
        data_id: impl Into<Cow<'a, str>>,
    ) -> SessionDataBuilder<'a, SessionDataDataIdHasBeenSet, ValueStatus, UriStatus> {
        self.attribute_list.data_id = data_id.into();
        SessionDataBuilder {
            attribute_list: self.attribute_list,
            data_id_status: PhantomData,
            value_status: PhantomData,
            uri_status: PhantomData,
        }
    }
    /// Add the provided `format` to the attributes built into `SessionData`.
    pub fn with_format(mut self, format: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.format = Some(format.into());
        self
    }
    /// Add the provided `language` to the attributes built into `SessionData`.
    pub fn with_language(mut self, language: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.language = Some(language.into());
        self
    }
}
impl<'a> Default
    for SessionDataBuilder<
        'a,
        SessionDataDataIdNeedsToBeSet,
        SessionDataValueIsNotSet,
        SessionDataUriIsNotSet,
    >
{
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-SESSION-DATA` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.4>
#[derive(Debug, Clone)]
pub struct SessionData<'a> {
    data_id: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
    uri: Option<Cow<'a, str>>,
    format: Option<Cow<'a, str>>,
    language: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, AttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                           // Used with Writer
    output_line_is_dirty: bool,                           // If should recalculate output_line
}

impl<'a> PartialEq for SessionData<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data_id() == other.data_id()
            && self.value() == other.value()
            && self.uri() == other.uri()
            && self.format() == other.format()
            && self.language() == other.language()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for SessionData<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_attribute_list()?;
        let Some(data_id) = attribute_list.get(DATA_ID).and_then(AttributeValue::quoted) else {
            return Err(ValidationError::MissingRequiredAttribute(DATA_ID));
        };
        Ok(Self {
            data_id: Cow::Borrowed(data_id),
            value: None,
            uri: None,
            format: None,
            language: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> SessionData<'a> {
    /// Constructs a new `SessionData` tag.
    fn new(attribute_list: SessionDataAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let SessionDataAttributeList {
            data_id,
            value,
            uri,
            format,
            language,
        } = attribute_list;
        Self {
            data_id,
            value,
            uri,
            format,
            language,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `SessionData` as such:
    /// ```
    /// # use quick_m3u8::tag::hls::{SessionData, Format};
    /// let session_data = SessionData::builder()
    ///     .with_data_id("1234")
    ///     .with_uri("data.bin")
    ///     .with_format(Format::Raw)
    ///     .finish();
    /// ```
    /// Note that the HLS specification indicates:
    /// > Each EXT-X-SESSION-DATA tag MUST contain either a VALUE or URI attribute, but not both.
    ///
    /// This is enforced with the builder, meaning, the `finish` method is only available once the
    /// `data_id` attribute has been set and either the `value` or `uri` attribute. Further, the
    /// `uri` attribute can only be set when `value` has not been set, and similarly, `value` can
    /// only be set when `uri` has not been set. Each of the following fail to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::SessionData;
    /// let session_data = SessionData::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::SessionData;
    /// let session_data = SessionData::builder().with_data_id("1234").finish();
    /// ```
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::SessionData;
    /// let session_data_builder = SessionData::builder().with_value("test").with_uri("data.bin");
    /// ```
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::SessionData;
    /// let session_data_builder = SessionData::builder().with_uri("data.bin").with_value("test");
    /// ```
    pub fn builder() -> SessionDataBuilder<
        'a,
        SessionDataDataIdNeedsToBeSet,
        SessionDataValueIsNotSet,
        SessionDataUriIsNotSet,
    > {
        SessionDataBuilder::new()
    }

    /// Corresponds to the `DATA-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn data_id(&self) -> &str {
        &self.data_id
    }

    /// Corresponds to the `VALUE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn value(&self) -> Option<&str> {
        if let Some(value) = &self.value {
            Some(value)
        } else {
            self.attribute_list
                .get(VALUE)
                .and_then(AttributeValue::quoted)
        }
    }

    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> Option<&str> {
        if let Some(uri) = &self.uri {
            Some(uri)
        } else {
            self.attribute_list
                .get(URI)
                .and_then(AttributeValue::quoted)
        }
    }

    /// Corresponds to the `FORMAT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn format(&self) -> EnumeratedString<Format> {
        if let Some(format) = &self.format {
            EnumeratedString::from(format.as_ref())
        } else {
            self.attribute_list
                .get(FORMAT)
                .and_then(AttributeValue::unquoted)
                .and_then(|v| v.try_as_utf_8().ok())
                .map(EnumeratedString::from)
                .unwrap_or(EnumeratedString::Known(Format::Json))
        }
    }

    /// Corresponds to the `LANGUAGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn language(&self) -> Option<&str> {
        if let Some(language) = &self.language {
            Some(language)
        } else {
            self.attribute_list
                .get(LANGUAGE)
                .and_then(AttributeValue::quoted)
        }
    }

    /// Sets the `DATA-ID` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn set_data_id(&mut self, data_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(DATA_ID);
        self.data_id = data_id.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `VALUE` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn set_value(&mut self, value: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(VALUE);
        self.value = Some(value.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `VALUE` attribute (sets it to `None`).
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn unset_value(&mut self) {
        self.attribute_list.remove(VALUE);
        self.value = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `URI` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = Some(uri.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `URI` attribute (sets it to `None`).
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn unset_uri(&mut self) {
        self.attribute_list.remove(URI);
        self.uri = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `FORMAT` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn set_format(&mut self, format: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(FORMAT);
        self.format = Some(format.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `FORMAT` attribute (sets it to `None`).
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn unset_format(&mut self) {
        self.attribute_list.remove(FORMAT);
        self.format = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `LANGUAGE` attribute.
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn set_language(&mut self, language: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(LANGUAGE);
        self.language = Some(language.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `LANGUAGE` attribute (sets it to `None`).
    ///
    /// See [`SessionData`] for a link to the HLS documentation for this attribute.
    pub fn unset_language(&mut self) {
        self.attribute_list.remove(LANGUAGE);
        self.language = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        let format = self.format();
        let format = if format == EnumeratedString::Known(Format::Json) {
            None
        } else {
            Some(format)
        };
        self.output_line = Cow::Owned(calculate_line(&SessionDataAttributeList {
            data_id: self.data_id().into(),
            value: self.value().map(|x| x.into()),
            uri: self.uri().map(|x| x.into()),
            format: format.map(|x| x.into()),
            language: self.language().map(|x| x.into()),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(SessionData);

const DATA_ID: &str = "DATA-ID";
const VALUE: &str = "VALUE";
const URI: &str = "URI";
const FORMAT: &str = "FORMAT";
const LANGUAGE: &str = "LANGUAGE";

fn calculate_line(attribute_list: &SessionDataAttributeList) -> Vec<u8> {
    let SessionDataAttributeList {
        data_id,
        value,
        uri,
        format,
        language,
    } = attribute_list;
    let mut line = format!("#EXT-X-SESSION-DATA:{DATA_ID}=\"{data_id}\"");
    if let Some(value) = value {
        line.push_str(format!(",{VALUE}=\"{value}\"").as_str());
    }
    if let Some(uri) = uri {
        line.push_str(format!(",{URI}=\"{uri}\"").as_str());
    }
    if let Some(format) = format {
        line.push_str(format!(",{FORMAT}={format}").as_str());
    }
    if let Some(language) = language {
        line.push_str(format!(",{LANGUAGE}=\"{language}\"").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-SESSION-DATA:DATA-ID=\"1234\",VALUE=\"test\",LANGUAGE=\"en\"",
            SessionData::builder()
                .with_data_id("1234")
                .with_value("test")
                .with_language("en")
                .finish()
                .into_inner()
                .value()
        )
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-SESSION-DATA:DATA-ID=\"1234\",URI=\"test.bin\",FORMAT=RAW",
            SessionData::builder()
                .with_data_id("1234")
                .with_uri("test.bin")
                .with_format("RAW")
                .finish()
                .into_inner()
                .value()
        )
    }

    // Setting both URI and VALUE are not permitted via HLS spec and enforced here via special
    // Builder generic types that only make setting those properties available when the other is not
    // set.
    //
    // Therefore we run one set of tests with value and one with uri. Because the macro unwraps into
    // multiple methods we have to wrap the tests in their own modules.
    //
    // We don't restrict setting value or uri based on presence of the other on the main body of the
    // SessionData struct... Maybe we should?
    mod with_value_mutation {
        use super::*;
        use pretty_assertions::assert_eq;
        mutation_tests!(
            SessionData::builder()
                .with_data_id("1234")
                .with_value("test")
                .with_language("en")
                .with_format("RAW")
                .finish(),
            (data_id, "abcd", @Attr="DATA-ID=\"abcd\""),
            (language, @Option "es", @Attr="LANGUAGE=\"es\""),
            (uri, @Option "example.bin", @Attr="URI=\"example.bin\""),
            (
                format,
                EnumeratedString::<Format>::Unknown("INVALID");
                @Default=EnumeratedString::Known(Format::Json),
                @Attr="FORMAT=INVALID")
        );
    }

    // Setting both URI and VALUE are not permitted via HLS spec and enforced here via special
    // Builder generic types that only make setting those properties available when the other is not
    // set.
    //
    // Therefore we run one set of tests with value and one with uri. Because the macro unwraps into
    // multiple methods we have to wrap the tests in their own modules.
    //
    // We don't restrict setting value or uri based on presence of the other on the main body of the
    // SessionData struct... Maybe we should?
    mod with_uri_mutation {
        use super::*;
        use pretty_assertions::assert_eq;
        mutation_tests!(
            SessionData::builder()
                .with_data_id("1234")
                .with_language("en")
                .with_uri("test.bin")
                .with_format("RAW")
                .finish(),
            (data_id, "abcd", @Attr="DATA-ID=\"abcd\""),
            (language, @Option "es", @Attr="LANGUAGE=\"es\""),
            (uri, @Option "example.bin", @Attr="URI=\"example.bin\""),
            (
                format,
                EnumeratedString::<Format>::Unknown("INVALID");
                @Default=EnumeratedString::Known(Format::Json),
                @Attr="FORMAT=INVALID")
        );
    }
}
