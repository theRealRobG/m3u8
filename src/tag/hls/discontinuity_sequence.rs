use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{hls::TagInner, known::ParsedTag, value::SemiParsedTagValue},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.3
#[derive(Debug, Clone)]
pub struct DiscontinuitySequence<'a> {
    discontinuity_sequence: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for DiscontinuitySequence<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.discontinuity_sequence() == other.discontinuity_sequence()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for DiscontinuitySequence<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::Unparsed(bytes) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let discontinuity_sequence = bytes.try_as_decimal_integer()?;
        Ok(Self {
            discontinuity_sequence,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> DiscontinuitySequence<'a> {
    pub fn new(discontinuity_sequence: u64) -> Self {
        let output_line = Cow::Owned(calculate_line(discontinuity_sequence));
        Self {
            discontinuity_sequence,
            output_line,
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

    pub fn discontinuity_sequence(&self) -> u64 {
        self.discontinuity_sequence
    }

    pub fn set_discontinuity_sequence(&mut self, discontinuity_sequence: u64) {
        self.discontinuity_sequence = discontinuity_sequence;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.discontinuity_sequence()));
        self.output_line_is_dirty = false;
    }
}

fn calculate_line(discontinuity_sequence: u64) -> Vec<u8> {
    format!("#EXT-X-DISCONTINUITY-SEQUENCE:{}", discontinuity_sequence).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid_tag() {
        assert_eq!(
            b"#EXT-X-DISCONTINUITY-SEQUENCE:42",
            DiscontinuitySequence::new(42).into_inner().value()
        )
    }
}
