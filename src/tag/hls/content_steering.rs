use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{TagInner, TagName},
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.6
#[derive(Debug, Clone)]
pub struct ContentSteering<'a> {
    server_uri: Cow<'a, str>,
    pathway_id: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for ContentSteering<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.server_uri() == other.server_uri() && self.pathway_id() == other.pathway_id()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for ContentSteering<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(server_uri)) = attribute_list.get(SERVER_URI)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(SERVER_URI));
        };
        Ok(Self {
            server_uri: Cow::Borrowed(server_uri),
            pathway_id: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> ContentSteering<'a> {
    pub fn new(server_uri: String, pathway_id: Option<String>) -> Self {
        let server_uri = Cow::Owned(server_uri);
        let pathway_id = pathway_id.map(Cow::Owned);
        let output_line = Cow::Owned(calculate_line(&server_uri, &pathway_id));
        Self {
            server_uri,
            pathway_id,
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

    pub fn server_uri(&self) -> &str {
        &self.server_uri
    }

    pub fn pathway_id(&self) -> Option<&str> {
        if let Some(pathway_id) = &self.pathway_id {
            Some(pathway_id)
        } else {
            match self.attribute_list.get(PATHWAY_ID) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn set_server_uri(&mut self, server_uri: String) {
        self.attribute_list.remove(SERVER_URI);
        self.server_uri = Cow::Owned(server_uri);
        self.output_line_is_dirty = true;
    }

    pub fn set_pathway_id(&mut self, pathway_id: Option<String>) {
        self.attribute_list.remove(PATHWAY_ID);
        self.pathway_id = pathway_id.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.server_uri(),
            &self.pathway_id().map(|x| x.into()),
        ));
        self.output_line_is_dirty = false;
    }
}

const SERVER_URI: &str = "SERVER-URI";
const PATHWAY_ID: &str = "PATHWAY-ID";

fn calculate_line<'a>(server_uri: &str, pathway_id: &Option<Cow<'a, str>>) -> String {
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
        let tag = ContentSteering::new("example.json".to_string(), None);
        assert_eq!(
            "#EXT-X-CONTENT-STEERING:SERVER-URI=\"example.json\"",
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_pathway_id_should_be_valid_line() {
        let tag = ContentSteering::new("example.json".to_string(), Some("1234".to_string()));
        assert_eq!(
            "#EXT-X-CONTENT-STEERING:SERVER-URI=\"example.json\",PATHWAY-ID=\"1234\"",
            tag.into_inner().value()
        );
    }
}
