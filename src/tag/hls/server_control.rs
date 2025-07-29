use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::into_inner_tag,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, PartialEq, Clone)]
pub struct ServerControlAttributeList {
    pub can_skip_until: Option<f64>,
    pub can_skip_dateranges: bool,
    pub hold_back: Option<f64>,
    pub part_hold_back: Option<f64>,
    pub can_block_reload: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ServerControlBuilder {
    can_skip_until: Option<f64>,
    can_skip_dateranges: bool,
    hold_back: Option<f64>,
    part_hold_back: Option<f64>,
    can_block_reload: bool,
}
impl ServerControlBuilder {
    pub fn new() -> Self {
        Self {
            can_skip_until: Default::default(),
            can_skip_dateranges: Default::default(),
            hold_back: Default::default(),
            part_hold_back: Default::default(),
            can_block_reload: Default::default(),
        }
    }

    pub fn finish<'a>(self) -> ServerControl<'a> {
        ServerControl::new(ServerControlAttributeList {
            can_skip_until: self.can_skip_until,
            can_skip_dateranges: self.can_skip_dateranges,
            hold_back: self.hold_back,
            part_hold_back: self.part_hold_back,
            can_block_reload: self.can_block_reload,
        })
    }

    pub fn with_can_skip_until(mut self, can_skip_until: f64) -> Self {
        self.can_skip_until = Some(can_skip_until);
        self
    }
    pub fn with_can_skip_dateranges(mut self) -> Self {
        self.can_skip_dateranges = true;
        self
    }
    pub fn with_hold_back(mut self, hold_back: f64) -> Self {
        self.hold_back = Some(hold_back);
        self
    }
    pub fn with_part_hold_back(mut self, part_hold_back: f64) -> Self {
        self.part_hold_back = Some(part_hold_back);
        self
    }
    pub fn with_can_block_reload(mut self) -> Self {
        self.can_block_reload = true;
        self
    }
}
impl Default for ServerControlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.8
#[derive(Debug, Clone)]
pub struct ServerControl<'a> {
    can_skip_until: Option<f64>,
    can_skip_dateranges: Option<bool>,
    hold_back: Option<f64>,
    part_hold_back: Option<f64>,
    can_block_reload: Option<bool>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
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
    pub fn new(attribute_list: ServerControlAttributeList) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let ServerControlAttributeList {
            can_skip_until,
            can_skip_dateranges,
            hold_back,
            part_hold_back,
            can_block_reload,
        } = attribute_list;
        Self {
            can_skip_until,
            can_skip_dateranges: Some(can_skip_dateranges),
            hold_back,
            part_hold_back,
            can_block_reload: Some(can_block_reload),
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn builder() -> ServerControlBuilder {
        ServerControlBuilder::new()
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

    pub fn set_can_skip_until(&mut self, can_skip_until: f64) {
        self.attribute_list.remove(CAN_SKIP_UNTIL);
        self.can_skip_until = Some(can_skip_until);
        self.output_line_is_dirty = true;
    }

    pub fn unset_can_skip_until(&mut self) {
        self.attribute_list.remove(CAN_SKIP_UNTIL);
        self.can_skip_until = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_can_skip_dateranges(&mut self, can_skip_dateranges: bool) {
        self.attribute_list.remove(CAN_SKIP_DATERANGES);
        self.can_skip_dateranges = Some(can_skip_dateranges);
        self.output_line_is_dirty = true;
    }

    pub fn set_hold_back(&mut self, hold_back: f64) {
        self.attribute_list.remove(HOLD_BACK);
        self.hold_back = Some(hold_back);
        self.output_line_is_dirty = true;
    }

    pub fn unset_hold_back(&mut self) {
        self.attribute_list.remove(HOLD_BACK);
        self.hold_back = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_part_hold_back(&mut self, part_hold_back: f64) {
        self.attribute_list.remove(PART_HOLD_BACK);
        self.part_hold_back = Some(part_hold_back);
        self.output_line_is_dirty = true;
    }

    pub fn unset_part_hold_back(&mut self) {
        self.attribute_list.remove(PART_HOLD_BACK);
        self.part_hold_back = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_can_block_reload(&mut self, can_block_reload: bool) {
        self.attribute_list.remove(CAN_BLOCK_RELOAD);
        self.can_block_reload = Some(can_block_reload);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&ServerControlAttributeList {
            can_skip_until: self.can_skip_until(),
            can_skip_dateranges: self.can_skip_dateranges(),
            hold_back: self.hold_back(),
            part_hold_back: self.part_hold_back(),
            can_block_reload: self.can_block_reload(),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(ServerControl);

const CAN_SKIP_UNTIL: &str = "CAN-SKIP-UNTIL";
const CAN_SKIP_DATERANGES: &str = "CAN-SKIP-DATERANGES";
const HOLD_BACK: &str = "HOLD-BACK";
const PART_HOLD_BACK: &str = "PART-HOLD-BACK";
const CAN_BLOCK_RELOAD: &str = "CAN-BLOCK-RELOAD";
const YES: &str = "YES";

fn calculate_line(attribute_list: &ServerControlAttributeList) -> Vec<u8> {
    let ServerControlAttributeList {
        can_skip_until,
        can_skip_dateranges,
        hold_back,
        part_hold_back,
        can_block_reload,
    } = attribute_list;
    let mut line = String::from("#EXT-X-SERVER-CONTROL:");
    let mut separator = "";
    if let Some(can_skip_until) = can_skip_until {
        line.push_str(format!("{separator}{CAN_SKIP_UNTIL}={can_skip_until:?}").as_str());
        separator = ",";
    }
    if *can_skip_dateranges {
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
    if *can_block_reload {
        line.push_str(format!("{separator}{CAN_BLOCK_RELOAD}={YES}").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_one_value_should_be_valid() {
        assert_eq!(
            b"#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=36.0",
            ServerControl::builder()
                .with_can_skip_until(36.0)
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_bools_should_be_valid() {
        assert_eq!(
            b"#EXT-X-SERVER-CONTROL:CAN-SKIP-DATERANGES=YES,CAN-BLOCK-RELOAD=YES",
            ServerControl::builder()
                .with_can_block_reload()
                .with_can_skip_dateranges()
                .finish()
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
            )
            .as_bytes(),
            ServerControl::builder()
                .with_can_skip_until(36.0)
                .with_can_block_reload()
                .with_can_skip_dateranges()
                .with_hold_back(18.0)
                .with_part_hold_back(1.5)
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        ServerControl::builder()
            .with_can_skip_until(36.0)
            .with_can_skip_dateranges()
            .with_hold_back(18.0)
            .with_part_hold_back(1.5)
            .finish(),
        (can_skip_until, @Option 18.0, @Attr="CAN-SKIP-UNTIL=18"),
        (can_block_reload, true, @Attr="CAN-BLOCK-RELOAD=YES"),
        (can_skip_dateranges, true, @Attr="CAN-SKIP-DATERANGES=YES"),
        (hold_back, @Option 42.0, @Attr="HOLD-BACK=42"),
        (part_hold_back, @Option 3.0, @Attr="PART-HOLD-BACK=3")
    );
}
