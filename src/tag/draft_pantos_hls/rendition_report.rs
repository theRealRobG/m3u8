use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.4
#[derive(Debug, PartialEq)]
pub struct RenditionReport<'a> {
    uri: &'a str,
    last_msn: u64,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
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
            output_line: Cow::Owned(calculate_line(uri, last_msn, last_part).into_bytes()),
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

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const URI: &'static str = "URI";
const LAST_MSN: &'static str = "LAST-MSN";
const LAST_PART: &'static str = "LAST-PART";

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
            RenditionReport::new("low.m3u8", 100, None).as_str()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-RENDITION-REPORT:URI=\"low.m3u8\",LAST-MSN=100,LAST-PART=2",
            RenditionReport::new("low.m3u8", 100, Some(2)).as_str()
        );
    }
}
