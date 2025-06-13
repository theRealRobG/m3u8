use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.6
#[derive(Debug, PartialEq)]
pub struct ContentSteering<'a> {
    server_uri: &'a str,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for ContentSteering<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(server_uri)) = attribute_list.get(SERVER_URI)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            server_uri,
            attribute_list,
        })
    }
}

impl<'a> ContentSteering<'a> {
    pub fn new(server_uri: &'a str, pathway_id: Option<&'a str>) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(SERVER_URI, ParsedAttributeValue::QuotedString(server_uri));
        if let Some(pathway_id) = pathway_id {
            attribute_list.insert(PATHWAY_ID, ParsedAttributeValue::QuotedString(pathway_id));
        }
        Self {
            server_uri,
            attribute_list,
        }
    }

    pub fn server_uri(&self) -> &str {
        self.server_uri
    }

    pub fn pathway_id(&self) -> Option<&str> {
        match self.attribute_list.get(PATHWAY_ID) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }
}

const SERVER_URI: &'static str = "SERVER-URI";
const PATHWAY_ID: &'static str = "PATHWAY-ID";
