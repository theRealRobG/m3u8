use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.3
#[derive(Debug, PartialEq)]
pub struct PreloadHint<'a> {
    hint_type: &'a str,
    uri: &'a str,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for PreloadHint<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(hint_type)) = attribute_list.get(TYPE) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            hint_type,
            uri,
            attribute_list,
        })
    }
}

impl<'a> PreloadHint<'a> {
    pub fn new(
        hint_type: &'a str,
        uri: &'a str,
        byterange_start: Option<u64>,
        byterange_length: Option<u64>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(TYPE, ParsedAttributeValue::UnquotedString(hint_type));
        attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        if let Some(byterange_start) = byterange_start {
            attribute_list.insert(
                BYTERANGE_START,
                ParsedAttributeValue::DecimalInteger(byterange_start),
            );
        }
        if let Some(byterange_length) = byterange_length {
            attribute_list.insert(
                BYTERANGE_LENGTH,
                ParsedAttributeValue::DecimalInteger(byterange_length),
            );
        }
        Self {
            hint_type,
            uri,
            attribute_list,
        }
    }

    pub fn hint_type(&self) -> &'a str {
        self.hint_type
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn byterange_start(&self) -> u64 {
        match self.attribute_list.get(BYTERANGE_START) {
            Some(ParsedAttributeValue::DecimalInteger(start)) => *start,
            _ => 0,
        }
    }

    pub fn byterange_length(&self) -> Option<u64> {
        match self.attribute_list.get(BYTERANGE_LENGTH) {
            Some(ParsedAttributeValue::DecimalInteger(length)) => Some(*length),
            _ => None,
        }
    }
}

const TYPE: &'static str = "TYPE";
const URI: &'static str = "URI";
const BYTERANGE_START: &'static str = "BYTERANGE-START";
const BYTERANGE_LENGTH: &'static str = "BYTERANGE-LENGTH";
