use crate::{
    date::DateTime,
    error::{ValidationError, ValidationErrorValueKind},
    tag::{hls::TagInner, known::ParsedTag, value::ParsedTagValue},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.6
#[derive(Debug, Clone)]
pub struct ProgramDateTime<'a> {
    program_date_time: DateTime,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for ProgramDateTime<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.program_date_time() == other.program_date_time()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for ProgramDateTime<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DateTimeMsec(date_time) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        Ok(Self {
            program_date_time: date_time,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> ProgramDateTime<'a> {
    pub fn new(program_date_time: DateTime) -> Self {
        Self {
            program_date_time,
            output_line: Cow::Owned(calculate_line(program_date_time)),
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

    pub fn program_date_time(&self) -> DateTime {
        self.program_date_time
    }

    pub fn set_program_date_time(&mut self, program_date_time: DateTime) {
        self.program_date_time = program_date_time;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.program_date_time()));
        self.output_line_is_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use crate::{date::DateTimeTimezoneOffset, date_time};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-PROGRAM-DATE-TIME:2025-06-16T21:52:08.010-05:00",
            ProgramDateTime::new(date_time!(2025-06-16 T 21:52:08.010 -05:00))
                .into_inner()
                .value()
        )
    }
}

fn calculate_line(date_time: DateTime) -> String {
    format!("#EXT-X-PROGRAM-DATE-TIME:{date_time}")
}
