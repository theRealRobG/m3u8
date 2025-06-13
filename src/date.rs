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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DateTimeTimezoneOffset {
    pub time_hour: i8,
    pub time_minute: u8,
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
}
