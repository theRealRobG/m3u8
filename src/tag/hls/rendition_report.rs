use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.4
#[derive(Debug, Clone)]
pub struct RenditionReport<'a> {
    uri: Cow<'a, str>,
    last_msn: u64,
    last_part: Option<u64>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for RenditionReport<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri() == other.uri()
            && self.last_msn() == other.last_msn()
            && self.last_part() == other.last_part()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for RenditionReport<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        let Some(ParsedAttributeValue::DecimalInteger(last_msn)) = attribute_list.get(LAST_MSN)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(LAST_MSN));
        };
        Ok(Self {
            uri: Cow::Borrowed(uri),
            last_msn: *last_msn,
            last_part: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> RenditionReport<'a> {
    pub fn new(uri: String, last_msn: u64, last_part: Option<u64>) -> Self {
        let uri = Cow::Owned(uri);
        let output_line = Cow::Owned(calculate_line(&uri, last_msn, last_part));
        Self {
            uri,
            last_msn,
            last_part,
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

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn last_msn(&self) -> u64 {
        self.last_msn
    }

    pub fn last_part(&self) -> Option<u64> {
        if let Some(last_part) = self.last_part {
            Some(last_part)
        } else {
            match self.attribute_list.get(LAST_PART) {
                Some(ParsedAttributeValue::DecimalInteger(part)) => Some(*part),
                _ => None,
            }
        }
    }

    pub fn set_uri(&mut self, uri: String) {
        self.attribute_list.remove(URI);
        self.uri = Cow::Owned(uri);
        self.output_line_is_dirty = true;
    }

    pub fn set_last_msn(&mut self, last_msn: u64) {
        self.attribute_list.remove(LAST_MSN);
        self.last_msn = last_msn;
        self.output_line_is_dirty = true;
    }

    pub fn set_last_part(&mut self, last_part: Option<u64>) {
        self.attribute_list.remove(LAST_PART);
        self.last_part = last_part;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.uri(),
            self.last_msn(),
            self.last_part(),
        ));
        self.output_line_is_dirty = false;
    }
}

const URI: &str = "URI";
const LAST_MSN: &str = "LAST-MSN";
const LAST_PART: &str = "LAST-PART";

fn calculate_line(uri: &str, last_msn: u64, last_part: Option<u64>) -> String {
    let mut line = format!("#EXT-X-RENDITION-REPORT:{URI}=\"{uri}\",{LAST_MSN}={last_msn}");
    if let Some(last_part) = last_part {
        line.push_str(format!(",{LAST_PART}={last_part}").as_str());
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
            "#EXT-X-RENDITION-REPORT:URI=\"low.m3u8\",LAST-MSN=100",
            RenditionReport::new("low.m3u8".to_string(), 100, None)
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-RENDITION-REPORT:URI=\"low.m3u8\",LAST-MSN=100,LAST-PART=2",
            RenditionReport::new("low.m3u8".to_string(), 100, Some(2))
                .into_inner()
                .value()
        );
    }
}
