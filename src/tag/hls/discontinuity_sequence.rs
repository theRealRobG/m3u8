use crate::{
    tag::{known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.3
#[derive(Debug, PartialEq)]
pub struct DiscontinuitySequence<'a> {
    discontinuity_sequence: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for DiscontinuitySequence<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(d) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            discontinuity_sequence: d,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> DiscontinuitySequence<'a> {
    pub fn new(discontinuity_sequence: u64) -> Self {
        let output_line = Cow::Owned(calculate_line(discontinuity_sequence).into_bytes());
        Self {
            discontinuity_sequence,
            output_line,
        }
    }

    pub fn discontinuity_sequence(&self) -> u64 {
        self.discontinuity_sequence
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

fn calculate_line(discontinuity_sequence: u64) -> String {
    format!("#EXT-X-DISCONTINUITY-SEQUENCE:{}", discontinuity_sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid_tag() {
        assert_eq!(
            "#EXT-X-DISCONTINUITY-SEQUENCE:42",
            DiscontinuitySequence::new(42).as_str()
        )
    }
}
