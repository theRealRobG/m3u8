use crate::tag::{
    known::ParsedTag,
    value::{ParsedAttributeValue, ParsedTagValue},
};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2
#[derive(Debug, PartialEq)]
pub struct Start<'a> {
    time_offset: f64,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTag<'a>> for Start<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(Some(time_offset)) = attribute_list
            .get("TIME-OFFSET")
            .map(ParsedAttributeValue::as_option_f64)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            time_offset,
            attribute_list,
        })
    }
}

impl<'a> Start<'a> {
    pub fn new(time_offset: f64, precise: bool) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(
            TIME_OFFSET,
            ParsedAttributeValue::SignedDecimalFloatingPoint(time_offset),
        );
        if precise {
            attribute_list.insert(PRECISE, ParsedAttributeValue::UnquotedString(YES));
        }
        Self {
            time_offset,
            attribute_list,
        }
    }

    pub fn time_offset(&self) -> f64 {
        self.time_offset
    }

    pub fn precise(&self) -> bool {
        matches!(
            self.attribute_list.get(PRECISE),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }
}

const TIME_OFFSET: &'static str = "TIME-OFFSET";
const PRECISE: &'static str = "PRECISE";
const YES: &'static str = "YES";
