use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.4
#[derive(Debug, PartialEq)]
pub struct SessionData<'a> {
    data_id: &'a str,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for SessionData<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(data_id)) = attribute_list.get("DATA-ID")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            data_id,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> SessionData<'a> {
    pub fn new(
        data_id: &'a str,
        value: Option<&'a str>,
        uri: Option<&'a str>,
        format: Option<&'a str>,
        language: Option<&'a str>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(DATA_ID, ParsedAttributeValue::QuotedString(data_id));
        if let Some(value) = value {
            attribute_list.insert(VALUE, ParsedAttributeValue::QuotedString(value));
        }
        if let Some(uri) = uri {
            attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        }
        if let Some(format) = format {
            attribute_list.insert(FORMAT, ParsedAttributeValue::UnquotedString(format));
        }
        if let Some(language) = language {
            attribute_list.insert(LANGUAGE, ParsedAttributeValue::QuotedString(language));
        }
        Self {
            data_id,
            attribute_list,
            output_line: Cow::Owned(
                calculate_line(data_id, value, uri, format, language).into_bytes(),
            ),
        }
    }

    pub fn data_id(&self) -> &'a str {
        self.data_id
    }

    pub fn value(&self) -> Option<&'a str> {
        match self.attribute_list.get(VALUE) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn uri(&self) -> Option<&'a str> {
        match self.attribute_list.get(URI) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn format(&self) -> &'a str {
        match self.attribute_list.get(FORMAT) {
            Some(ParsedAttributeValue::UnquotedString(s)) => s,
            _ => "JSON",
        }
    }

    pub fn language(&self) -> Option<&'a str> {
        match self.attribute_list.get(LANGUAGE) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const DATA_ID: &'static str = "DATA-ID";
const VALUE: &'static str = "VALUE";
const URI: &'static str = "URI";
const FORMAT: &'static str = "FORMAT";
const LANGUAGE: &'static str = "LANGUAGE";

fn calculate_line(
    data_id: &str,
    value: Option<&str>,
    uri: Option<&str>,
    format: Option<&str>,
    language: Option<&str>,
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
            SessionData::new("1234", Some("test"), None, None, Some("en"),).as_str()
        )
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-SESSION-DATA:DATA-ID=\"1234\",URI=\"test.bin\",FORMAT=RAW",
            SessionData::new("1234", None, Some("test.bin"), Some("RAW"), None,).as_str()
        )
    }
}
