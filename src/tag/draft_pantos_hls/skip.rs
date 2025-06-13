use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.2
#[derive(Debug, PartialEq)]
pub struct Skip<'a> {
    skipped_segments: u64,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Skip<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::DecimalInteger(skipped_segments)) =
            attribute_list.get(SKIPPED_SEGMENTS)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            skipped_segments: *skipped_segments,
            attribute_list,
        })
    }
}

impl<'a> Skip<'a> {
    pub fn new(skipped_segments: u64, recently_removed_dateranges: Option<&'a str>) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(
            SKIPPED_SEGMENTS,
            ParsedAttributeValue::DecimalInteger(skipped_segments),
        );
        if let Some(recently_removed_dateranges) = recently_removed_dateranges {
            attribute_list.insert(
                RECENTLY_REMOVED_DATERANGES,
                ParsedAttributeValue::QuotedString(recently_removed_dateranges),
            );
        }
        Self {
            skipped_segments,
            attribute_list,
        }
    }

    pub fn skipped_segments(&self) -> u64 {
        self.skipped_segments
    }

    pub fn recently_removed_dateranges(&self) -> Option<&'a str> {
        match self.attribute_list.get(RECENTLY_REMOVED_DATERANGES) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }
}

const SKIPPED_SEGMENTS: &'static str = "SKIPPED-SEGMENTS";
const RECENTLY_REMOVED_DATERANGES: &'static str = "RECENTLY-REMOVED-DATERANGES";
