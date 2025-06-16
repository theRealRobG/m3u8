use crate::{
    tag::{
        draft_pantos_hls::TagName,
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.6
#[derive(Debug, PartialEq)]
pub struct ContentSteering<'a> {
    server_uri: &'a str,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for ContentSteering<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(server_uri)) = attribute_list.get(SERVER_URI)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            server_uri,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
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
            output_line: Cow::Owned(calculate_line(server_uri, pathway_id).into_bytes()),
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

    pub fn as_str(&self) -> &str {
        match self.output_line {
            Cow::Borrowed(bytes) => split_by_first_lf(str_from(bytes)).parsed,
            Cow::Owned(ref bytes) => str_from(bytes.as_slice()),
        }
    }
}

const SERVER_URI: &'static str = "SERVER-URI";
const PATHWAY_ID: &'static str = "PATHWAY-ID";

fn calculate_line(server_uri: &str, pathway_id: Option<&str>) -> String {
    let mut line = format!(
        "#EXT{}:{}=\"{}\"",
        TagName::ContentSteering.as_str(),
        SERVER_URI,
        server_uri
    );
    if let Some(pathway_id) = pathway_id {
        line.push_str(format!(",{}=\"{}\"", PATHWAY_ID, pathway_id).as_str());
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_without_pathway_id_should_be_valid_line() {
        let tag = ContentSteering::new("example.json", None);
        assert_eq!(
            "#EXT-X-CONTENT-STEERING:SERVER-URI=\"example.json\"",
            tag.as_str()
        );
    }

    #[test]
    fn new_with_pathway_id_should_be_valid_line() {
        let tag = ContentSteering::new("example.json", Some("1234"));
        assert_eq!(
            "#EXT-X-CONTENT-STEERING:SERVER-URI=\"example.json\",PATHWAY-ID=\"1234\"",
            tag.as_str()
        );
    }
}
