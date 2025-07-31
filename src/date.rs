//! Constructs to reason about date and time in HLS
//!
//! The structs offered here don't provide much functionality. The purpose is primarily
//! informational. These types can be used with another date/time library (such as [chrono]) for
//! more feature rich date/time comparisons and operations.
//!
//! [chrono]: https://crates.io/crates/chrono

use crate::{error::DateTimeSyntaxError, utils::parse_date_time_bytes};
use std::fmt::Display;

/// A macro to help constructing a [`DateTime`] struct.
///
/// Given that there are a lot of fields to the `DateTime` struct, for convenience this macro is
/// provided, so a date can be constructed more easily. The syntax is intended to mimic [RFC3339].
/// For example:
/// ```
/// # use m3u8::{date_time, date::{DateTime, DateTimeTimezoneOffset}};
/// assert_eq!(
///     date_time!(2025-07-30 T 22:44:38.718 -05:00),
///     DateTime {
///         date_fullyear: 2025,
///         date_month: 7,
///         date_mday: 30,
///         time_hour: 22,
///         time_minute: 44,
///         time_second: 38.718,
///         timezone_offset: DateTimeTimezoneOffset {
///             time_hour: -5,
///             time_minute: 0,
///         },
///     }
/// )
/// ```
///
/// [RFC3339]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
#[macro_export]
macro_rules! date_time {
    ($Y:literal-$M:literal-$D:literal T $h:literal:$m:literal:$s:literal) => {
        $crate::date::DateTime {
            date_fullyear: $Y,
            date_month: $M,
            date_mday: $D,
            time_hour: $h,
            time_minute: $m,
            time_second: $s,
            timezone_offset: Default::default(),
        }
    };
    ($Y:literal-$M:literal-$D:literal T $h:literal:$m:literal:$s:literal $x:literal:$y:literal) => {
        $crate::date::DateTime {
            date_fullyear: $Y,
            date_month: $M,
            date_mday: $D,
            time_hour: $h,
            time_minute: $m,
            time_second: $s,
            timezone_offset: $crate::date::DateTimeTimezoneOffset {
                time_hour: $x,
                time_minute: $y,
            },
        }
    };
}

/// A struct representing a date in the format of [RFC3339].
///
/// [RFC3339]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DateTime {
    /// The full year (must be `4DIGIT`).
    pub date_fullyear: u32,
    /// The month (`1-12`).
    pub date_month: u8,
    /// The day (`1-31`).
    pub date_mday: u8,
    /// The hour (`0-23`).
    pub time_hour: u8,
    /// The minute (`0-59`).
    pub time_minute: u8,
    /// The seconds, including millisconds (seconds are `0-59`, while the mantissa may be any
    /// length, though HLS recommends milliscond accuracy via the [EXT-X-PROGRAM-DATE-TIME]
    /// documentation).
    ///
    /// [EXT-X-PROGRAM-DATE-TIME]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.6
    pub time_second: f64,
    /// The timezone offset.
    pub timezone_offset: DateTimeTimezoneOffset,
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:06.3}{}",
            self.date_fullyear,
            self.date_month,
            self.date_mday,
            self.time_hour,
            self.time_minute,
            self.time_second,
            self.timezone_offset
        )
    }
}

impl From<DateTime> for String {
    fn from(value: DateTime) -> Self {
        format!("{value}")
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self {
            date_fullyear: 1970,
            date_month: 1,
            date_mday: 1,
            time_hour: 0,
            time_minute: 0,
            time_second: 0.0,
            timezone_offset: Default::default(),
        }
    }
}

/// The timezone offset.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct DateTimeTimezoneOffset {
    /// The hour offset (plus or minus `0-23`).
    pub time_hour: i8,
    /// The minute offset (`0-59`).
    pub time_minute: u8,
}

impl Display for DateTimeTimezoneOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.time_hour == 0 && self.time_minute == 0 {
            write!(f, "Z")
        } else {
            write!(f, "{:+03}:{:02}", self.time_hour, self.time_minute)
        }
    }
}

impl From<DateTimeTimezoneOffset> for String {
    fn from(value: DateTimeTimezoneOffset) -> Self {
        format!("{value}")
    }
}

/// Parses a string slice into a `DateTime`.
pub fn parse(input: &str) -> Result<DateTime, DateTimeSyntaxError> {
    parse_bytes(input.as_bytes())
}

/// Parses a byte slice into a `DateTime`.
pub fn parse_bytes(input: &[u8]) -> Result<DateTime, DateTimeSyntaxError> {
    Ok(parse_date_time_bytes(input)?.parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn no_timezone() {
        assert_eq!(
            date_time!(2025-06-04 T 13:50:42.148),
            parse("2025-06-04T13:50:42.148Z").unwrap()
        );
    }

    #[test]
    fn plus_timezone() {
        assert_eq!(
            date_time!(2025-06-04 T 13:50:42.148 03:00),
            parse("2025-06-04T13:50:42.148+03:00").unwrap()
        );
    }

    #[test]
    fn negative_timezone() {
        assert_eq!(
            date_time!(2025-06-04 T 13:50:42.148 -01:30),
            parse("2025-06-04T13:50:42.148-01:30").unwrap()
        );
    }

    #[test]
    fn no_fractional_seconds() {
        assert_eq!(
            date_time!(2025-06-04 T 13:50:42.0),
            parse("2025-06-04T13:50:42Z").unwrap()
        );
    }

    #[test]
    fn string_from_single_digit_dates_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.123Z"),
            String::from(date_time!(2025-06-04 T 13:50:42.123))
        )
    }

    #[test]
    fn string_from_no_fractional_seconds_should_still_be_3_decimals_precise() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.000Z"),
            String::from(date_time!(2025-06-04 T 13:50:42.0))
        )
    }

    #[test]
    fn string_from_single_digit_times_should_be_valid() {
        assert_eq!(
            String::from("2025-12-25T04:00:02.000Z"),
            String::from(date_time!(2025-12-25 T 04:00:02.000))
        )
    }

    #[test]
    fn string_from_negative_time_offset_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.123-05:00"),
            String::from(date_time!(2025-06-04 T 13:50:42.123 -05:00))
        )
    }

    #[test]
    fn string_from_positive_offset_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.100+01:00"),
            String::from(date_time!(2025-06-04 T 13:50:42.100 01:00))
        )
    }

    #[test]
    fn string_from_positive_offset_non_zero_minutes_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.010+06:30"),
            String::from(date_time!(2025-06-04 T 13:50:42.010 06:30))
        )
    }

    #[test]
    fn date_time_macro_should_work_with_no_offset() {
        assert_eq!(
            date_time!(2025-06-22 T 22:13:42.000),
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 22,
                time_hour: 22,
                time_minute: 13,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            }
        );
    }

    #[test]
    fn date_time_macro_should_work_with_positive_offset() {
        assert_eq!(
            date_time!(2025-06-22 T 22:13:42.000 01:00),
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 22,
                time_hour: 22,
                time_minute: 13,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 1,
                    time_minute: 0
                }
            }
        );
    }

    #[test]
    fn date_time_macro_should_work_with_negative_offset() {
        assert_eq!(
            date_time!(2025-06-22 T 22:13:42.000 -01:30),
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 22,
                time_hour: 22,
                time_minute: 13,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -1,
                    time_minute: 30
                }
            }
        );
    }
}
