use crate::{
    date::DateTime,
    tag::{known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.6
#[derive(Debug, PartialEq)]
pub struct ProgramDateTime<'a> {
    program_date_time: DateTime,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for ProgramDateTime<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DateTimeMsec(date_time) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            program_date_time: date_time,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> ProgramDateTime<'a> {
    pub fn new(program_date_time: DateTime) -> Self {
        Self {
            program_date_time,
            output_line: Cow::Owned(calculate_line(program_date_time).into_bytes()),
        }
    }

    pub fn program_date_time(&self) -> DateTime {
        self.program_date_time
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

#[cfg(test)]
mod tests {
    use crate::date::DateTimeTimezoneOffset;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-PROGRAM-DATE-TIME:2025-06-16T21:52:08.010-05:00",
            ProgramDateTime::new(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 16,
                time_hour: 21,
                time_minute: 52,
                time_second: 8.01,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -5,
                    time_minute: 0
                }
            })
            .as_str()
        )
    }
}

fn calculate_line(date_time: DateTime) -> String {
    format!("#EXT-X-PROGRAM-DATE-TIME:{date_time}")
}
