use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.4
#[derive(Debug, PartialEq)]
pub struct Key<'a> {
    method: &'a str,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Key<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(method)) = attribute_list.get(METHOD) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            method,
            attribute_list,
        })
    }
}

impl<'a> Key<'a> {
    pub fn new(
        method: &'a str,
        uri: Option<&'a str>,
        iv: Option<&'a str>,
        keyformat: Option<&'a str>,
        keyformatversions: Option<&'a str>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(METHOD, ParsedAttributeValue::UnquotedString(method));
        if let Some(uri) = uri {
            attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        }
        if let Some(iv) = iv {
            attribute_list.insert(IV, ParsedAttributeValue::UnquotedString(iv));
        }
        if let Some(keyformat) = keyformat {
            attribute_list.insert(KEYFORMAT, ParsedAttributeValue::QuotedString(keyformat));
        }
        if let Some(keyformatversions) = keyformatversions {
            attribute_list.insert(
                KEYFORMATVERSIONS,
                ParsedAttributeValue::QuotedString(keyformatversions),
            );
        }
        Self {
            method,
            attribute_list,
        }
    }

    pub fn method(&self) -> &'a str {
        self.method
    }

    pub fn uri(&self) -> Option<&'a str> {
        match self.attribute_list.get(URI) {
            Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
            _ => None,
        }
    }

    pub fn iv(&self) -> Option<&'a str> {
        match self.attribute_list.get(IV) {
            Some(ParsedAttributeValue::UnquotedString(iv)) => Some(iv),
            _ => None,
        }
    }

    pub fn keyformat(&self) -> &'a str {
        match self.attribute_list.get(KEYFORMAT) {
            Some(ParsedAttributeValue::QuotedString(keyformat)) => keyformat,
            _ => "identity",
        }
    }

    pub fn keyformatversions(&self) -> Option<&'a str> {
        match self.attribute_list.get(KEYFORMATVERSIONS) {
            Some(ParsedAttributeValue::QuotedString(keyformatversions)) => Some(keyformatversions),
            _ => None,
        }
    }
}

const METHOD: &'static str = "METHOD";
const URI: &'static str = "URI";
const IV: &'static str = "IV";
const KEYFORMAT: &'static str = "KEYFORMAT";
const KEYFORMATVERSIONS: &'static str = "KEYFORMATVERSIONS";
