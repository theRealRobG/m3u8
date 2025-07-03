use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.7
#[derive(Debug, Clone)]
pub struct PartInf<'a> {
    part_target: f64,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for PartInf<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.part_target() == other.part_target()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for PartInf<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(Some(part_target)) = attribute_list
            .get(PART_TARGET)
            .map(ParsedAttributeValue::as_option_f64)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(
                PART_TARGET,
            ));
        };
        Ok(Self {
            part_target,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> PartInf<'a> {
    pub fn new(part_target: f64) -> Self {
        Self {
            part_target,
            output_line: Cow::Owned(calculate_line(part_target)),
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

    pub fn part_target(&self) -> f64 {
        self.part_target
    }

    pub fn set_part_target(&mut self, part_target: f64) {
        self.part_target = part_target;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.part_target()));
        self.output_line_is_dirty = false;
    }
}

const PART_TARGET: &str = "PART-TARGET";

fn calculate_line(part_target: f64) -> Vec<u8> {
    format!("#EXT-X-PART-INF:{PART_TARGET}={part_target}").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::hls::test_macro::mutation_tests;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PART-INF:PART-TARGET=0.5",
            PartInf::new(0.5).into_inner().value()
        );
    }

    mutation_tests!(PartInf::new(0.5), (part_target, 1.0, @Attr="PART-TARGET=1"));
}
