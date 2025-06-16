use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.7
#[derive(Debug, PartialEq)]
pub struct PartInf<'a> {
    part_target: f64,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for PartInf<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
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
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
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
            output_line: Cow::Owned(calculate_line(part_target).into_bytes()),
        }
    }

    pub fn part_target(&self) -> f64 {
        self.part_target
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const PART_TARGET: &'static str = "PART-TARGET";

fn calculate_line(part_target: f64) -> String {
    format!("#EXT-X-PART-INF:{PART_TARGET}={part_target}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-PART-INF:PART-TARGET=0.5",
            PartInf::new(0.5).as_str()
        );
    }
}
