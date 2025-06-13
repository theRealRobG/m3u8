use std::collections::HashMap;

use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.7
#[derive(Debug, PartialEq)]
pub struct PartInf<'a> {
    part_target: f64,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for PartInf<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(Some(part_target)) = attribute_list
            .get(PART_TARGET)
            .map(ParsedAttributeValue::as_option_f64)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            part_target,
            attribute_list,
        })
    }
}

impl<'a> PartInf<'a> {
    pub fn new(part_target: f64) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(
            PART_TARGET,
            ParsedAttributeValue::SignedDecimalFloatingPoint(part_target),
        );
        Self {
            part_target,
            attribute_list,
        }
    }

    pub fn part_target(&self) -> f64 {
        self.part_target
    }
}

const PART_TARGET: &'static str = "PART-TARGET";
