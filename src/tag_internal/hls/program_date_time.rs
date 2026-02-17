#[cfg(not(feature = "chrono"))]
use crate::date::DateTime;
use crate::{
    date::string_from,
    error::{ParseTagValueError, ValidationError},
    tag::{UnknownTag, hls::into_inner_tag},
};
use std::borrow::Cow;

/// Corresponds to the `#EXT-X-PROGRAM-DATE-TIME` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#section-4.4.4.6>
#[derive(Debug, Clone)]
pub struct ProgramDateTime<'a> {
    #[cfg(feature = "chrono")]
    program_date_time: chrono::DateTime<chrono::FixedOffset>,
    #[cfg(not(feature = "chrono"))]
    program_date_time: DateTime,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for ProgramDateTime<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.program_date_time() == other.program_date_time()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for ProgramDateTime<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let program_date_time = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_date_time()?;
        Ok(Self {
            program_date_time,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> ProgramDateTime<'a> {
    #[cfg(feature = "chrono")]
    /// Construct a new `ProgramDateTime` tag.
    ///
    /// Note, the library provides a convenience `date_time!` macro, in case you are setting the
    /// `DateTime` using literal values:
    /// ```
    /// use quick_m3u8::{
    ///     date_time,
    ///     tag::hls::ProgramDateTime,
    /// };
    ///
    /// let pdt = ProgramDateTime::new(date_time!(2025-08-03 T 18:26:34.439 -05:00));
    /// assert_eq!(
    ///     pdt.program_date_time(),
    ///     chrono::NaiveDate::from_ymd_opt(2025, 8, 3).unwrap()
    ///         .and_hms_milli_opt(18, 26, 34, 439).unwrap()
    ///         .and_local_timezone(chrono::FixedOffset::west_opt(5 * 3600).unwrap())
    ///         .earliest().unwrap()
    /// );
    /// ```
    pub fn new(program_date_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self {
            program_date_time,
            output_line: Cow::Owned(calculate_line(program_date_time)),
            output_line_is_dirty: false,
        }
    }
    #[cfg(not(feature = "chrono"))]
    /// Construct a new `ProgramDateTime` tag.
    ///
    /// Note, the library provides a convenience `date_time!` macro, in case you are setting the
    /// `DateTime` using literal values:
    /// ```
    /// use quick_m3u8::{
    ///     date_time,
    ///     tag::hls::ProgramDateTime,
    ///     date::{ DateTime, DateTimeTimezoneOffset}
    /// };
    ///
    /// let pdt = ProgramDateTime::new(date_time!(2025-08-03 T 18:26:34.439 -05:00));
    /// assert_eq!(
    ///     pdt.program_date_time(),
    ///     DateTime {
    ///         date_fullyear: 2025,
    ///         date_month: 8,
    ///         date_mday: 3,
    ///         time_hour: 18,
    ///         time_minute: 26,
    ///         time_second: 34.439,
    ///         timezone_offset: DateTimeTimezoneOffset {
    ///             time_hour: -5,
    ///             time_minute: 0,
    ///         },
    ///     },
    /// );
    /// ```
    pub fn new(program_date_time: DateTime) -> Self {
        Self {
            program_date_time,
            output_line: Cow::Owned(calculate_line(program_date_time)),
            output_line_is_dirty: false,
        }
    }

    #[cfg(feature = "chrono")]
    /// Corresponds to the value of the tag (`#EXT-X-PROGRAM-DATE-TIME:<date-time-msec>`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn program_date_time(&self) -> chrono::DateTime<chrono::FixedOffset> {
        self.program_date_time
    }
    #[cfg(not(feature = "chrono"))]
    /// Corresponds to the value of the tag (`#EXT-X-PROGRAM-DATE-TIME:<date-time-msec>`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn program_date_time(&self) -> DateTime {
        self.program_date_time
    }

    #[cfg(feature = "chrono")]
    /// Sets the value of the tag.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_program_date_time(
        &mut self,
        program_date_time: chrono::DateTime<chrono::FixedOffset>,
    ) {
        self.program_date_time = program_date_time;
        self.output_line_is_dirty = true;
    }
    #[cfg(not(feature = "chrono"))]
    /// Sets the value of the tag.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_program_date_time(&mut self, program_date_time: DateTime) {
        self.program_date_time = program_date_time;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.program_date_time()));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(ProgramDateTime);

#[cfg(feature = "chrono")]
fn calculate_line(date_time: chrono::DateTime<chrono::FixedOffset>) -> Vec<u8> {
    format!("#EXT-X-PROGRAM-DATE-TIME:{}", string_from(&date_time)).into_bytes()
}
#[cfg(not(feature = "chrono"))]
fn calculate_line(date_time: DateTime) -> Vec<u8> {
    format!("#EXT-X-PROGRAM-DATE-TIME:{}", string_from(&date_time)).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        date_time,
        tag::{IntoInnerTag, hls::test_macro::mutation_tests},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-PROGRAM-DATE-TIME:2025-06-16T21:52:08.011-05:00",
            str::from_utf8(
                ProgramDateTime::new(date_time!(2025-06-16 T 21:52:08.011 -05:00))
                    .into_inner()
                    .value()
            )
            .unwrap()
        )
    }

    mutation_tests!(
        ProgramDateTime::new(date_time!(2025-07-03 T 14:21:33.001 -05:00)),
        (program_date_time, default_date(), @Attr=":2000-01-01T00:00:00.123Z")
    );

    #[cfg(feature = "chrono")]
    fn default_date() -> chrono::DateTime<chrono::FixedOffset> {
        date_time!(2000-01-01 T 00:00:00.123)
    }
    #[cfg(not(feature = "chrono"))]
    fn default_date() -> DateTime {
        date_time!(2000-01-01 T 00:00:00.123)
    }
}
