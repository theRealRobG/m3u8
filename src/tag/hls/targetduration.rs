use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{hls::TagInner, known::ParsedTag, value::ParsedTagValue},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.1
#[derive(Debug)]
pub struct Targetduration<'a> {
    target_duration: u64,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Targetduration<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.target_duration() == other.target_duration()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Targetduration<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(d) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        Ok(Self {
            target_duration: d,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Targetduration<'a> {
    pub fn new(target_duration: u64) -> Self {
        Self {
            target_duration,
            output_line: Cow::Owned(calculate_line(target_duration)),
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

    pub fn target_duration(&self) -> u64 {
        self.target_duration
    }

    pub fn set_target_duration(&mut self, target_duration: u64) {
        self.target_duration = target_duration;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.target_duration()));
        self.output_line_is_dirty = false;
    }
}

fn calculate_line(target_duration: u64) -> String {
    format!("#EXT-X-TARGETDURATION:{target_duration}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-TARGETDURATION:10",
            Targetduration::new(10).into_inner().value()
        );
    }
}
