use std::fmt::Display;

use crate::{tag::value::ParsedTagValue, utils::parse_date_time_bytes};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DateTime {
    pub date_fullyear: u32,
    pub date_month: u8,
    pub date_mday: u8,
    pub time_hour: u8,
    pub time_minute: u8,
    pub time_second: f64,
    pub timezone_offset: DateTimeTimezoneOffset,
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl From<DateTime> for String {
    fn from(value: DateTime) -> Self {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:06.3}{}",
            value.date_fullyear,
            value.date_month,
            value.date_mday,
            value.time_hour,
            value.time_minute,
            value.time_second,
            String::from(value.timezone_offset)
        )
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

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct DateTimeTimezoneOffset {
    pub time_hour: i8,
    pub time_minute: u8,
}

impl From<DateTimeTimezoneOffset> for String {
    fn from(value: DateTimeTimezoneOffset) -> Self {
        if value.time_hour == 0 && value.time_minute == 0 {
            Self::from("Z")
        } else {
            format!("{:+03}:{:02}", value.time_hour, value.time_minute)
        }
    }
}

pub fn parse(input: &str) -> Result<DateTime, &'static str> {
    let mut bytes = input.as_bytes().iter();
    let break_byte = loop {
        let Some(byte) = bytes.next() else {
            return Err("Unexpected end of line while parsing DateTime");
        };
        match byte {
            b't' | b':' => break byte,
            _ => (),
        }
    };
    let ParsedTagValue::DateTimeMsec(date_time) =
        parse_date_time_bytes(input, bytes, *break_byte)?.parsed
    else {
        return Err("Invalid DateTime");
    };
    Ok(date_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn no_timezone() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.148,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            },
            parse("2025-06-04T13:50:42.148Z").unwrap()
        );
    }

    #[test]
    fn plus_timezone() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.148,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 3,
                    time_minute: 0
                }
            },
            parse("2025-06-04T13:50:42.148+03:00").unwrap()
        );
    }

    #[test]
    fn negative_timezone() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.148,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -1,
                    time_minute: 30
                }
            },
            parse("2025-06-04T13:50:42.148-01:30").unwrap()
        );
    }

    #[test]
    fn no_fractional_seconds() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            },
            parse("2025-06-04T13:50:42Z").unwrap()
        );
    }

    #[test]
    fn string_from_single_digit_dates_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.123Z"),
            String::from(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.123,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            })
        )
    }

    #[test]
    fn string_from_no_fractional_seconds_should_still_be_3_decimals_precise() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.000Z"),
            String::from(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            })
        )
    }

    #[test]
    fn string_from_single_digit_times_should_be_valid() {
        assert_eq!(
            String::from("2025-12-25T04:00:02.000Z"),
            String::from(DateTime {
                date_fullyear: 2025,
                date_month: 12,
                date_mday: 25,
                time_hour: 4,
                time_minute: 0,
                time_second: 2.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            })
        )
    }

    #[test]
    fn string_from_negative_time_offset_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.123-05:00"),
            String::from(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.123,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -5,
                    time_minute: 0
                }
            })
        )
    }

    #[test]
    fn string_from_positive_offset_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.100+01:00"),
            String::from(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.1,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 1,
                    time_minute: 0
                }
            })
        )
    }

    #[test]
    fn string_from_positive_offset_non_zero_minutes_should_be_valid() {
        assert_eq!(
            String::from("2025-06-04T13:50:42.010+06:30"),
            String::from(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.01,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 6,
                    time_minute: 30
                }
            })
        )
    }
}
