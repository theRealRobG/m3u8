use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.4
#[derive(Debug, PartialEq)]
pub struct SessionData<'a> {
    data_id: &'a str,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for SessionData<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(data_id)) = attribute_list.get("DATA-ID")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            data_id,
            attribute_list,
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
}

const DATA_ID: &'static str = "DATA-ID";
const VALUE: &'static str = "VALUE";
const URI: &'static str = "URI";
const FORMAT: &'static str = "FORMAT";
const LANGUAGE: &'static str = "LANGUAGE";
