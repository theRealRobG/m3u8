use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.8
#[derive(Debug, PartialEq)]
pub struct ServerControl<'a> {
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for ServerControl<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> ServerControl<'a> {
    pub fn new(
        can_skip_until: Option<f64>,
        can_skip_dateranges: bool,
        hold_back: Option<f64>,
        part_hold_back: Option<f64>,
        can_block_reload: bool,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        if let Some(can_skip_until) = can_skip_until {
            attribute_list.insert(
                CAN_SKIP_UNTIL,
                ParsedAttributeValue::SignedDecimalFloatingPoint(can_skip_until),
            );
        }
        if can_skip_dateranges {
            attribute_list.insert(
                CAN_SKIP_DATERANGES,
                ParsedAttributeValue::UnquotedString(YES),
            );
        }
        if let Some(hold_back) = hold_back {
            attribute_list.insert(
                HOLD_BACK,
                ParsedAttributeValue::SignedDecimalFloatingPoint(hold_back),
            );
        }
        if let Some(part_hold_back) = part_hold_back {
            attribute_list.insert(
                PART_HOLD_BACK,
                ParsedAttributeValue::SignedDecimalFloatingPoint(part_hold_back),
            );
        }
        if can_block_reload {
            attribute_list.insert(CAN_BLOCK_RELOAD, ParsedAttributeValue::UnquotedString(YES));
        }
        Self {
            attribute_list,
            output_line: Cow::Owned(
                calculate_line(
                    can_skip_until,
                    can_skip_dateranges,
                    hold_back,
                    part_hold_back,
                    can_block_reload,
                )
                .into_bytes(),
            ),
        }
    }

    pub fn can_skip_until(&self) -> Option<f64> {
        match self.attribute_list.get(CAN_SKIP_UNTIL) {
            Some(ParsedAttributeValue::SignedDecimalFloatingPoint(can_skip_until)) => {
                Some(*can_skip_until)
            }
            _ => None,
        }
    }

    pub fn can_skip_dateranges(&self) -> bool {
        matches!(
            self.attribute_list.get(CAN_SKIP_DATERANGES),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }

    pub fn hold_back(&self) -> Option<f64> {
        match self.attribute_list.get(HOLD_BACK) {
            Some(ParsedAttributeValue::SignedDecimalFloatingPoint(hold_back)) => Some(*hold_back),
            _ => None,
        }
    }

    pub fn part_hold_back(&self) -> Option<f64> {
        match self.attribute_list.get(PART_HOLD_BACK) {
            Some(ParsedAttributeValue::SignedDecimalFloatingPoint(part_hold_back)) => {
                Some(*part_hold_back)
            }
            _ => None,
        }
    }

    pub fn can_block_reload(&self) -> bool {
        matches!(
            self.attribute_list.get(CAN_BLOCK_RELOAD),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const CAN_SKIP_UNTIL: &'static str = "CAN-SKIP-UNTIL";
const CAN_SKIP_DATERANGES: &'static str = "CAN-SKIP-DATERANGES";
const HOLD_BACK: &'static str = "HOLD-BACK";
const PART_HOLD_BACK: &'static str = "PART-HOLD-BACK";
const CAN_BLOCK_RELOAD: &'static str = "CAN-BLOCK-RELOAD";
const YES: &'static str = "YES";

fn calculate_line(
    can_skip_until: Option<f64>,
    can_skip_dateranges: bool,
    hold_back: Option<f64>,
    part_hold_back: Option<f64>,
    can_block_reload: bool,
) -> String {
    let mut line = String::from("#EXT-X-SERVER-CONTROL:");
    let mut separator = "";
    if let Some(can_skip_until) = can_skip_until {
        line.push_str(format!("{separator}{CAN_SKIP_UNTIL}={can_skip_until}").as_str());
        separator = ",";
    }
    if can_skip_dateranges {
        line.push_str(format!("{separator}{CAN_SKIP_DATERANGES}={YES}").as_str());
        separator = ",";
    }
    if let Some(hold_back) = hold_back {
        line.push_str(format!("{separator}{HOLD_BACK}={hold_back}").as_str());
        separator = ",";
    }
    if let Some(part_hold_back) = part_hold_back {
        line.push_str(format!("{separator}{PART_HOLD_BACK}={part_hold_back}").as_str());
        separator = ",";
    }
    if can_block_reload {
        line.push_str(format!("{separator}{CAN_BLOCK_RELOAD}={YES}").as_str());
        separator = ",";
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_one_value_should_be_valid() {
        assert_eq!(
            "#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=36",
            ServerControl::new(Some(36.0), false, None, None, false).as_str()
        );
    }

    #[test]
    fn as_str_with_bools_should_be_valid() {
        assert_eq!(
            "#EXT-X-SERVER-CONTROL:CAN-SKIP-DATERANGES=YES,CAN-BLOCK-RELOAD=YES",
            ServerControl::new(None, true, None, None, true).as_str()
        );
    }

    #[test]
    fn as_str_with_all_options_should_be_valid() {
        assert_eq!(
            concat!(
                "#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=36,CAN-SKIP-DATERANGES=YES,HOLD-BACK=18,",
                "PART-HOLD-BACK=1.5,CAN-BLOCK-RELOAD=YES",
            ),
            ServerControl::new(Some(36.0), true, Some(18.0), Some(1.5), true).as_str()
        );
    }
}
