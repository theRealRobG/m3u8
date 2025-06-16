use crate::tag::{
    known::ParsedTag,
    value::{ParsedAttributeValue, ParsedTagValue},
};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.4
#[derive(Debug, PartialEq)]
pub struct RenditionReport<'a> {
    uri: &'a str,
    last_msn: u64,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTag<'a>> for RenditionReport<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::DecimalInteger(last_msn)) = attribute_list.get(LAST_MSN)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            uri,
            last_msn: *last_msn,
            attribute_list,
        })
    }
}

impl<'a> RenditionReport<'a> {
    pub fn new(uri: &'a str, last_msn: u64, last_part: Option<u64>) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        attribute_list.insert(LAST_MSN, ParsedAttributeValue::DecimalInteger(last_msn));
        if let Some(last_part) = last_part {
            attribute_list.insert(LAST_PART, ParsedAttributeValue::DecimalInteger(last_part));
        }
        Self {
            uri,
            last_msn,
            attribute_list,
        }
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn last_msn(&self) -> u64 {
        self.last_msn
    }

    pub fn last_part(&self) -> Option<u64> {
        match self.attribute_list.get(LAST_PART) {
            Some(ParsedAttributeValue::DecimalInteger(part)) => Some(*part),
            _ => None,
        }
    }
}

const URI: &'static str = "URI";
const LAST_MSN: &'static str = "LAST-MSN";
const LAST_PART: &'static str = "LAST-PART";
