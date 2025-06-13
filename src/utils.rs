use crate::{
    date::{DateTime, DateTimeTimezoneOffset},
    line::ParsedLineSlice,
    tag::value::ParsedTagValue,
};
use std::slice::Iter;

pub fn take_until_end_of_bytes<'a>(
    mut bytes: Iter<'a, u8>,
) -> Result<ParsedLineSlice<'a, &'a str>, &'static str> {
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

pub fn validate_carriage_return_bytes(bytes: &mut Iter<'_, u8>) -> Result<(), &'static str> {
    let Some(b'\n') = bytes.next() else {
        return Err("Unexpected carriage return without following line feed");
    };
    Ok(())
}

pub fn str_from(bytes: &[u8]) -> &str {
    unsafe {
        // SAFETY: The input for bytes is always &str in this project, and I only break on single
        // byte characters, so this is safe to do unchecked.
        std::str::from_utf8_unchecked(bytes)
    }
}

/// Expectation is that bytes has already been iterated through until either `b't'` or `b':'`.
/// Calling this from the beginning of a DateTime will fail. Iterate partially through first and
/// then use this method for the rest.
pub fn parse_date_time_bytes<'a>(
    input: &'a str,
    mut bytes: Iter<'a, u8>,
    break_byte: u8,
) -> Result<ParsedLineSlice<'a, ParsedTagValue<'a>>, &'static str> {
    let date_bytes = input.as_bytes();
    let Ok(date_fullyear) = input[..4].parse::<u32>() else {
        return Err("Invalid year in DateTimeMsec value");
    };
    let Some(b'-') = date_bytes.get(4) else {
        return Err("Invalid DateTimeMsec value");
    };
    let Ok(date_month) = input[5..7].parse::<u8>() else {
        return Err("Invalid month in DateTimeMsec value");
    };
    let Some(b'-') = date_bytes.get(7) else {
        return Err("Invalid DateTimeMsec value");
    };
    let Ok(date_mday) = input[8..10].parse::<u8>() else {
        return Err("Invalid day in DateTimeMsec value");
    };
    if break_byte == b't' {
        let Some(b't') = date_bytes.get(10) else {
            return Err("Invalid DateTimeMsec value");
        };
        bytes.next();
        bytes.next();
        let Some(b':') = bytes.next() else {
            return Err("Invalid DateTimeMsec value");
        };
    } else {
        let Some(b'T') = date_bytes.get(10) else {
            return Err("Invalid DateTimeMsec value");
        };
    }
    let Ok(time_hour) = input[11..13].parse::<u8>() else {
        return Err("Invalid hour in DateTimeMsec value");
    };
    bytes.next();
    bytes.next();
    let Some(b':') = bytes.next() else {
        return Err("Invalid DateTimeMsec value");
    };
    let mut byte_count = 17;
    let Ok(time_minute) = input[14..16].parse::<u8>() else {
        return Err("Invalid minute in DateTimeMsec value");
    };
    let time_offset_byte = 'time_offset_loop: loop {
        let Some(&byte) = bytes.next() else {
            break 'time_offset_loop None;
        };
        byte_count += 1;
        match byte {
            b'Z' | b'z' | b'+' | b'-' => break 'time_offset_loop Some(byte),
            b'\r' | b'\n' => return Err("Unexpected end of line in DateTimeMsec value"),
            b'0'..=b'9' | b'.' => (),
            _ => return Err("Invalid second in DateTimeMsec value"),
        }
    };
    let Some(time_offset_byte) = time_offset_byte else {
        return Err("Unexpected end of line in DateTimeMsec value");
    };
    let Ok(time_second) = input[17..(byte_count - 1)].parse::<f64>() else {
        return Err("Invalid second in DateTimeMsec value");
    };
    match time_offset_byte {
        b'Z' | b'z' => {
            let remaining = take_until_end_of_bytes(bytes)?;
            if !remaining.parsed.is_empty() {
                return Err("Unexpected characteres after timezone in DateTimeMsec value");
            };
            let remaining = remaining.remaining;
            Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DateTimeMsec(DateTime {
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
                }),
                remaining,
            })
        }
        _ => {
            let multiplier = if time_offset_byte == b'-' { -1i8 } else { 1i8 };
            bytes.next();
            bytes.next();
            let Some(b':') = bytes.next() else {
                return Err("Invalid DateTimeMsec value");
            };
            let Ok(timeoffset_hour) = input[byte_count..(byte_count + 2)].parse::<i8>() else {
                return Err("Invalid time offset hour in DateTimeMsec value");
            };
            let timeoffset_hour = multiplier * timeoffset_hour;
            bytes.next();
            bytes.next();
            let remaining = take_until_end_of_bytes(bytes)?;
            if !remaining.parsed.is_empty() {
                return Err("Unexpected characteres after timezone in DateTimeMsec value");
            };
            let remaining = remaining.remaining;
            let Ok(timeoffset_minute) = input[(byte_count + 3)..(byte_count + 5)].parse::<u8>()
            else {
                return Err("Invalid time offset minute in DateTimeMsec value");
            };
            Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DateTimeMsec(DateTime {
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
                }),
                remaining,
            })
        }
    }
}
