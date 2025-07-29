use crate::{
    error::{UnrecognizedEnumerationError, ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{EnumeratedString, into_inner_tag},
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
    utils::AsStaticCow,
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

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

#[derive(Debug, PartialEq, Clone)]
pub struct SessionDataAttributeList<'a> {
    pub data_id: Cow<'a, str>,
    pub value: Option<Cow<'a, str>>,
    pub uri: Option<Cow<'a, str>>,
    pub format: Option<Cow<'a, str>>,
    pub language: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SessionDataBuilder<'a> {
    data_id: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
    uri: Option<Cow<'a, str>>,
    format: Option<Cow<'a, str>>,
    language: Option<Cow<'a, str>>,
}
impl<'a> SessionDataBuilder<'a> {
    pub fn new(data_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            data_id: data_id.into(),
            value: Default::default(),
            uri: Default::default(),
            format: Default::default(),
            language: Default::default(),
        }
    }

    pub fn finish(self) -> SessionData<'a> {
        SessionData::new(SessionDataAttributeList {
            data_id: self.data_id,
            value: self.value,
            uri: self.uri,
            format: self.format,
            language: self.language,
        })
    }

    pub fn with_value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = Some(value.into());
        self
    }
    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> Self {
        self.uri = Some(uri.into());
        self
    }
    pub fn with_format(mut self, format: impl Into<Cow<'a, str>>) -> Self {
        self.format = Some(format.into());
        self
    }
    pub fn with_language(mut self, language: impl Into<Cow<'a, str>>) -> Self {
        self.language = Some(language.into());
        self
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.4
#[derive(Debug, Clone)]
pub struct SessionData<'a> {
    data_id: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
    uri: Option<Cow<'a, str>>,
    format: Option<Cow<'a, str>>,
    language: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
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

impl<'a> TryFrom<ParsedTag<'a>> for SessionData<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(data_id)) = attribute_list.get(DATA_ID) else {
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
    pub fn new(attribute_list: SessionDataAttributeList<'a>) -> Self {
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

    pub fn builder(data_id: impl Into<Cow<'a, str>>) -> SessionDataBuilder<'a> {
        SessionDataBuilder::new(data_id)
    }

    pub fn data_id(&self) -> &str {
        &self.data_id
    }

    pub fn value(&self) -> Option<&str> {
        if let Some(value) = &self.value {
            Some(value)
        } else {
            match self.attribute_list.get(VALUE) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn uri(&self) -> Option<&str> {
        if let Some(uri) = &self.uri {
            Some(uri)
        } else {
            match self.attribute_list.get(URI) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn format(&self) -> EnumeratedString<Format> {
        if let Some(format) = &self.format {
            EnumeratedString::from(format.as_ref())
        } else {
            match self.attribute_list.get(FORMAT) {
                Some(ParsedAttributeValue::UnquotedString(s)) => EnumeratedString::from(*s),
                _ => EnumeratedString::Known(Format::Json),
            }
        }
    }

    pub fn language(&self) -> Option<&str> {
        if let Some(language) = &self.language {
            Some(language)
        } else {
            match self.attribute_list.get(LANGUAGE) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn set_data_id(&mut self, data_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(DATA_ID);
        self.data_id = data_id.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_value(&mut self, value: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(VALUE);
        self.value = Some(value.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_value(&mut self) {
        self.attribute_list.remove(VALUE);
        self.value = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = Some(uri.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_uri(&mut self) {
        self.attribute_list.remove(URI);
        self.uri = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_format(&mut self, format: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(FORMAT);
        self.format = Some(format.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_format(&mut self) {
        self.attribute_list.remove(FORMAT);
        self.format = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_language(&mut self, language: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(LANGUAGE);
        self.language = Some(language.into());
        self.output_line_is_dirty = true;
    }

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
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-SESSION-DATA:DATA-ID=\"1234\",VALUE=\"test\",LANGUAGE=\"en\"",
            SessionData::builder("1234")
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
            SessionData::builder("1234")
                .with_uri("test.bin")
                .with_format("RAW")
                .finish()
                .into_inner()
                .value()
        )
    }

    mutation_tests!(
        SessionData::builder("1234")
            .with_value("test")
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
