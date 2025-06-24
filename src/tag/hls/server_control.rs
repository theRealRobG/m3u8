use crate::tag::{
    hls::TagInner,
    known::ParsedTag,
    value::{ParsedAttributeValue, ParsedTagValue},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.8
#[derive(Debug)]
pub struct ServerControl<'a> {
    can_skip_until: Option<f64>,
    can_skip_dateranges: Option<bool>,
    hold_back: Option<f64>,
    part_hold_back: Option<f64>,
    can_block_reload: Option<bool>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for ServerControl<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.can_skip_until() == other.can_skip_until()
            && self.can_skip_dateranges() == other.can_skip_dateranges()
            && self.hold_back() == other.hold_back()
            && self.part_hold_back() == other.part_hold_back()
            && self.can_block_reload() == other.can_block_reload()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for ServerControl<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            can_skip_until: None,
            can_skip_dateranges: None,
            hold_back: None,
            part_hold_back: None,
            can_block_reload: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
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
        let output_line = Cow::Owned(calculate_line(
            can_skip_until,
            can_skip_dateranges,
            hold_back,
            part_hold_back,
            can_block_reload,
        ));
        let can_skip_dateranges = Some(can_skip_dateranges);
        let can_block_reload = Some(can_block_reload);
        Self {
            can_skip_until,
            can_skip_dateranges,
            hold_back,
            part_hold_back,
            can_block_reload,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub(crate) fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn can_skip_until(&self) -> Option<f64> {
        if let Some(can_skip_until) = self.can_skip_until {
            Some(can_skip_until)
        } else {
            match self.attribute_list.get(CAN_SKIP_UNTIL) {
                Some(ParsedAttributeValue::SignedDecimalFloatingPoint(can_skip_until)) => {
                    Some(*can_skip_until)
                }
                _ => None,
            }
        }
    }

    pub fn can_skip_dateranges(&self) -> bool {
        if let Some(can_skip_dateranges) = self.can_skip_dateranges {
            can_skip_dateranges
        } else {
            matches!(
                self.attribute_list.get(CAN_SKIP_DATERANGES),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }

    pub fn hold_back(&self) -> Option<f64> {
        if let Some(hold_back) = self.hold_back {
            Some(hold_back)
        } else {
            match self.attribute_list.get(HOLD_BACK) {
                Some(ParsedAttributeValue::SignedDecimalFloatingPoint(hold_back)) => {
                    Some(*hold_back)
                }
                _ => None,
            }
        }
    }
    pub fn part_hold_back(&self) -> Option<f64> {
        if let Some(part_hold_back) = self.part_hold_back {
            Some(part_hold_back)
        } else {
            match self.attribute_list.get(PART_HOLD_BACK) {
                Some(ParsedAttributeValue::SignedDecimalFloatingPoint(part_hold_back)) => {
                    Some(*part_hold_back)
                }
                _ => None,
            }
        }
    }

    pub fn can_block_reload(&self) -> bool {
        if let Some(can_block_reload) = self.can_block_reload {
            can_block_reload
        } else {
            matches!(
                self.attribute_list.get(CAN_BLOCK_RELOAD),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }

    pub fn set_can_skip_until(&mut self, can_skip_until: Option<f64>) {
        self.attribute_list.remove(CAN_SKIP_UNTIL);
        self.can_skip_until = can_skip_until;
        self.output_line_is_dirty = true;
    }

    pub fn set_can_skip_dateranges(&mut self, can_skip_dateranges: bool) {
        self.attribute_list.remove(CAN_SKIP_DATERANGES);
        self.can_skip_dateranges = Some(can_skip_dateranges);
        self.output_line_is_dirty = true;
    }

    pub fn set_hold_back(&mut self, hold_back: Option<f64>) {
        self.attribute_list.remove(HOLD_BACK);
        self.hold_back = hold_back;
        self.output_line_is_dirty = true;
    }

    pub fn set_part_hold_back(&mut self, part_hold_back: Option<f64>) {
        self.attribute_list.remove(PART_HOLD_BACK);
        self.part_hold_back = part_hold_back;
        self.output_line_is_dirty = true;
    }

    pub fn set_can_block_reload(&mut self, can_block_reload: bool) {
        self.attribute_list.remove(CAN_BLOCK_RELOAD);
        self.can_block_reload = Some(can_block_reload);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.can_skip_until(),
            self.can_skip_dateranges(),
            self.hold_back(),
            self.part_hold_back(),
            self.can_block_reload(),
        ));
        self.output_line_is_dirty = false;
    }
}

const CAN_SKIP_UNTIL: &str = "CAN-SKIP-UNTIL";
const CAN_SKIP_DATERANGES: &str = "CAN-SKIP-DATERANGES";
const HOLD_BACK: &str = "HOLD-BACK";
const PART_HOLD_BACK: &str = "PART-HOLD-BACK";
const CAN_BLOCK_RELOAD: &str = "CAN-BLOCK-RELOAD";
const YES: &str = "YES";

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
        line.push_str(format!("{separator}{CAN_SKIP_UNTIL}={can_skip_until:?}").as_str());
        separator = ",";
    }
    if can_skip_dateranges {
        line.push_str(format!("{separator}{CAN_SKIP_DATERANGES}={YES}").as_str());
        separator = ",";
    }
    if let Some(hold_back) = hold_back {
        line.push_str(format!("{separator}{HOLD_BACK}={hold_back:?}").as_str());
        separator = ",";
    }
    if let Some(part_hold_back) = part_hold_back {
        line.push_str(format!("{separator}{PART_HOLD_BACK}={part_hold_back:?}").as_str());
        separator = ",";
    }
    if can_block_reload {
        line.push_str(format!("{separator}{CAN_BLOCK_RELOAD}={YES}").as_str());
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
            "#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=36.0",
            ServerControl::new(Some(36.0), false, None, None, false)
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_bools_should_be_valid() {
        assert_eq!(
            "#EXT-X-SERVER-CONTROL:CAN-SKIP-DATERANGES=YES,CAN-BLOCK-RELOAD=YES",
            ServerControl::new(None, true, None, None, true)
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_all_options_should_be_valid() {
        assert_eq!(
            concat!(
                "#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=36.0,CAN-SKIP-DATERANGES=YES,HOLD-BACK=18.0,",
                "PART-HOLD-BACK=1.5,CAN-BLOCK-RELOAD=YES",
            ),
            ServerControl::new(Some(36.0), true, Some(18.0), Some(1.5), true)
                .into_inner()
                .value()
        );
    }
}
