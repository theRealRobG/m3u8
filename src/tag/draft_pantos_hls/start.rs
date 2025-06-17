use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2
#[derive(Debug, PartialEq)]
pub struct Start<'a> {
    time_offset: f64,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
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
            output_line: Cow::Owned(calculate_line(time_offset, precise).into_bytes()),
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

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const TIME_OFFSET: &'static str = "TIME-OFFSET";
const PRECISE: &'static str = "PRECISE";
const YES: &'static str = "YES";

fn calculate_line(time_offset: f64, precise: bool) -> String {
    if precise {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset},{PRECISE}={YES}")
    } else {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_without_precise_should_be_valid() {
        assert_eq!(
            "#EXT-X-START:TIME-OFFSET=-42",
            Start::new(-42.0, false).as_str()
        )
    }

    #[test]
    fn as_str_with_precise_should_be_valid() {
        assert_eq!(
            "#EXT-X-START:TIME-OFFSET=-42,PRECISE=YES",
            Start::new(-42.0, true).as_str()
        )
    }
}
