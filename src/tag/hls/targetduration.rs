use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{hls::into_inner_tag, known::ParsedTag, value::SemiParsedTagValue},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.1
#[derive(Debug, Clone)]
pub struct Targetduration<'a> {
    target_duration: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
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
        let SemiParsedTagValue::Unparsed(bytes) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let target_duration = bytes.try_as_decimal_integer()?;
        Ok(Self {
            target_duration,
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

into_inner_tag!(Targetduration);

fn calculate_line(target_duration: u64) -> Vec<u8> {
    format!("#EXT-X-TARGETDURATION:{target_duration}").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            b"#EXT-X-TARGETDURATION:10",
            Targetduration::new(10).into_inner().value()
        );
    }

    mutation_tests!(Targetduration::new(10), (target_duration, 20, @Attr=":20"));
}
