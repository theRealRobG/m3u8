use crate::{
    date::{DateTime, DateTimeTimezoneOffset},
    error::{DateTimeSyntaxError, GenericSyntaxError},
    line::ParsedLineSlice,
};
use std::slice::Iter;

pub fn take_until_end_of_bytes<'a>(
    mut bytes: Iter<'a, u8>,
) -> Result<ParsedLineSlice<'a, &'a str>, GenericSyntaxError> {
    let input = bytes.as_slice();
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        match bytes.next() {
            None => {
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: None,
                });
            }
            Some(b'\r') => {
                validate_carriage_return_bytes(&mut bytes)?;
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: Some(str_from(bytes.as_slice())),
                });
            }
            Some(b'\n') => {
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: Some(str_from(bytes.as_slice())),
                });
            }
            _ => (),
        }
    }
}

pub fn validate_carriage_return_bytes(bytes: &mut Iter<'_, u8>) -> Result<(), GenericSyntaxError> {
    let Some(b'\n') = bytes.next() else {
        return Err(GenericSyntaxError::CarriageReturnWithoutLineFeed);
    };
    Ok(())
}

pub(crate) fn str_from(bytes: &[u8]) -> &str {
    unsafe {
        // SAFETY: The input for bytes is always &str in this project, and I only break on single
        // byte characters, so this is safe to do unchecked.
        std::str::from_utf8_unchecked(bytes)
    }
}

pub fn split_by_first_lf(str: &str) -> ParsedLineSlice<&str> {
    let mut cr_index = None;
    let next_new_line_index = str.as_bytes().iter().enumerate().find_map(|(i, b)| {
        if *b == b'\n' {
            Some(i)
        } else if *b == b'\r' {
            cr_index = Some(i);
            None
        } else {
            None
        }
    });
    match next_new_line_index {
        Some(index) => {
            if cr_index == Some(index - 1) {
                let parsed = &str[..cr_index.unwrap()];
                let remaining = Some(&str[(index + 1)..]);
                ParsedLineSlice { parsed, remaining }
            } else {
                let parsed = &str[..index];
                let remaining = Some(&str[(index + 1)..]);
                ParsedLineSlice { parsed, remaining }
            }
        }
        None => {
            let parsed = str;
            let remaining = None;
            ParsedLineSlice { parsed, remaining }
        }
    }
}

/// Expectation is that bytes has already been iterated through until either `b't'` or `b':'`.
/// Calling this from the beginning of a DateTime will fail. Iterate partially through first and
/// then use this method for the rest.
pub fn parse_date_time_bytes<'a>(
    input: &'a str,
    mut bytes: Iter<'a, u8>,
    break_byte: u8,
) -> Result<ParsedLineSlice<'a, DateTime>, DateTimeSyntaxError> {
    let date_bytes = input.as_bytes();
    let date_fullyear = input[..4]
        .parse::<u32>()
        .map_err(|e| DateTimeSyntaxError::InvalidYear(e))?;
    match date_bytes.get(4) {
        Some(b'-') => (),
        b => {
            return Err(DateTimeSyntaxError::UnexpectedYearToMonthSeparator(
                b.map(|b| *b),
            ));
        }
    };
    let date_month = input[5..7]
        .parse::<u8>()
        .map_err(|e| DateTimeSyntaxError::InvalidMonth(e))?;
    match date_bytes.get(7) {
        Some(b'-') => (),
        b => {
            return Err(DateTimeSyntaxError::UnexpectedMonthToDaySeparator(
                b.map(|b| *b),
            ));
        }
    };
    let date_mday = input[8..10]
        .parse::<u8>()
        .map_err(|e| DateTimeSyntaxError::InvalidDay(e))?;
    if break_byte == b't' {
        match date_bytes.get(10) {
            Some(b't') => (),
            b => {
                return Err(DateTimeSyntaxError::UnexpectedDayHourSeparator(
                    b.map(|b| *b),
                ));
            }
        };
        bytes.next();
        bytes.next();
        match bytes.next() {
            Some(b':') => (),
            b => {
                return Err(DateTimeSyntaxError::UnexpectedHourMinuteSeparator(
                    b.map(|b| *b),
                ));
            }
        };
    } else {
        match date_bytes.get(10) {
            Some(b'T') => (),
            b => {
                return Err(DateTimeSyntaxError::UnexpectedDayHourSeparator(
                    b.map(|b| *b),
                ));
            }
        };
    }
    let time_hour = input[11..13]
        .parse::<u8>()
        .map_err(|e| DateTimeSyntaxError::InvalidHour(e))?;
    bytes.next();
    bytes.next();
    match bytes.next() {
        Some(b':') => (),
        b => {
            return Err(DateTimeSyntaxError::UnexpectedHourMinuteSeparator(
                b.map(|b| *b),
            ));
        }
    };
    let mut byte_count = 17;
    let time_minute = input[14..16]
        .parse::<u8>()
        .map_err(|e| DateTimeSyntaxError::InvalidMinute(e))?;
    let time_offset_byte = 'time_offset_loop: loop {
        let Some(&byte) = bytes.next() else {
            break 'time_offset_loop None;
        };
        byte_count += 1;
        match byte {
            b'Z' | b'z' | b'+' | b'-' => break 'time_offset_loop Some(byte),
            b'\r' | b'\n' => return Err(GenericSyntaxError::UnexpectedEndOfLine)?,
            _ => (),
        }
    };
    let Some(time_offset_byte) = time_offset_byte else {
        return Err(GenericSyntaxError::UnexpectedEndOfLine)?;
    };
    let time_second = input[17..(byte_count - 1)]
        .parse::<f64>()
        .map_err(|e| DateTimeSyntaxError::InvalidSecond(e))?;
    match time_offset_byte {
        b'Z' | b'z' => {
            let remaining = take_until_end_of_bytes(bytes)?;
            if !remaining.parsed.is_empty() {
                return Err(DateTimeSyntaxError::UnexpectedCharactersAfterTimezone);
            };
            let remaining = remaining.remaining;
            Ok(ParsedLineSlice {
                parsed: DateTime {
                    date_fullyear,
                    date_month,
                    date_mday,
                    time_hour,
                    time_minute,
                    time_second,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: 0,
                        time_minute: 0,
                    },
                },
                remaining,
            })
        }
        _ => {
            let multiplier = if time_offset_byte == b'-' { -1i8 } else { 1i8 };
            bytes.next();
            bytes.next();
            match bytes.next() {
                Some(b':') => (),
                b => {
                    return Err(DateTimeSyntaxError::UnexpectedTimezoneHourMinuteSeparator(
                        b.map(|b| *b),
                    ));
                }
            };
            let timeoffset_hour = input[byte_count..(byte_count + 2)]
                .parse::<i8>()
                .map_err(|e| DateTimeSyntaxError::InvalidTimezoneHour(e))?;
            let timeoffset_hour = multiplier * timeoffset_hour;
            bytes.next();
            bytes.next();
            let remaining = take_until_end_of_bytes(bytes)?;
            if !remaining.parsed.is_empty() {
                return Err(DateTimeSyntaxError::UnexpectedCharactersAfterTimezone);
            };
            let remaining = remaining.remaining;
            let timeoffset_minute = input[(byte_count + 3)..(byte_count + 5)]
                .parse::<u8>()
                .map_err(|e| DateTimeSyntaxError::InvalidTimezoneMinute(e))?;
            Ok(ParsedLineSlice {
                parsed: DateTime {
                    date_fullyear,
                    date_month,
                    date_mday,
                    time_hour,
                    time_minute,
                    time_second,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: timeoffset_hour,
                        time_minute: timeoffset_minute,
                    },
                },
                remaining,
            })
        }
    }
}
