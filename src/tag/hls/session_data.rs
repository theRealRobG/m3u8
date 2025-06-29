use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.4
#[derive(Debug, Clone)]
pub struct SessionData<'a> {
    data_id: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
    uri: Option<Cow<'a, str>>,
    format: Option<Cow<'a, str>>,
    language: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
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
    pub fn new(
        data_id: String,
        value: Option<String>,
        uri: Option<String>,
        format: Option<String>,
        language: Option<String>,
    ) -> Self {
        let data_id = Cow::Owned(data_id);
        let value = value.map(Cow::Owned);
        let uri = uri.map(Cow::Owned);
        let format = format.map(Cow::Owned);
        let language = language.map(Cow::Owned);
        let output_line = Cow::Owned(calculate_line(&data_id, &value, &uri, &format, &language));
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

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
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

    pub fn format(&self) -> &str {
        if let Some(format) = &self.format {
            format
        } else {
            match self.attribute_list.get(FORMAT) {
                Some(ParsedAttributeValue::UnquotedString(s)) => s,
                _ => "JSON",
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

    pub fn set_data_id(&mut self, data_id: String) {
        self.attribute_list.remove(DATA_ID);
        self.data_id = Cow::Owned(data_id);
        self.output_line_is_dirty = true;
    }

    pub fn set_value(&mut self, value: Option<String>) {
        self.attribute_list.remove(VALUE);
        self.value = value.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_uri(&mut self, uri: Option<String>) {
        self.attribute_list.remove(URI);
        self.uri = uri.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_format(&mut self, format: String) {
        self.attribute_list.remove(FORMAT);
        self.format = Some(Cow::Owned(format));
        self.output_line_is_dirty = true;
    }

    pub fn set_language(&mut self, language: Option<String>) {
        self.attribute_list.remove(LANGUAGE);
        self.language = language.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        let format = self.format();
        let format = if format == "JSON" { None } else { Some(format) };
        self.output_line = Cow::Owned(calculate_line(
            self.data_id(),
            &self.value().map(|x| x.into()),
            &self.uri().map(|x| x.into()),
            &format.map(|x| x.into()),
            &self.language().map(|x| x.into()),
        ));
        self.output_line_is_dirty = false;
    }
}

const DATA_ID: &str = "DATA-ID";
const VALUE: &str = "VALUE";
const URI: &str = "URI";
const FORMAT: &str = "FORMAT";
const LANGUAGE: &str = "LANGUAGE";

fn calculate_line<'a>(
    data_id: &str,
    value: &Option<Cow<'a, str>>,
    uri: &Option<Cow<'a, str>>,
    format: &Option<Cow<'a, str>>,
    language: &Option<Cow<'a, str>>,
) -> String {
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
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-SESSION-DATA:DATA-ID=\"1234\",VALUE=\"test\",LANGUAGE=\"en\"",
            SessionData::new(
                "1234".to_string(),
                Some("test".to_string()),
                None,
                None,
                Some("en".to_string()),
            )
            .into_inner()
            .value()
        )
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-SESSION-DATA:DATA-ID=\"1234\",URI=\"test.bin\",FORMAT=RAW",
            SessionData::new(
                "1234".to_string(),
                None,
                Some("test.bin".to_string()),
                Some("RAW".to_string()),
                None,
            )
            .into_inner()
            .value()
        )
    }
}
