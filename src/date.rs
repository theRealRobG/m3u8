use std::str::Chars;

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
    let mut char_count = 0usize;
    let mut input_chars = input.chars();
    loop {
        let Some(char) = input_chars.next() else {
            return Err("Unexpected end of line while parsing DateTime");
        };
        char_count += 1;
        match char {
            't' | ':' => break,
            _ => (),
        }
    }
    handle_partially_parsed_date_time(input, char_count, input_chars)
}

/// Take the output of an existing parsing loop and complete the parsing of the DateTime value.
///
/// The `input_chars` are expected to have reached either the lowercase `t` separator or the first
/// `:` separator of the time (after the time_hour value).
pub fn handle_partially_parsed_date_time<'a>(
    input: &'a str,
    mut char_count: usize,
    mut input_chars: Chars<'_>,
) -> Result<DateTime, &'static str> {
    // "2025-06-09t17:53:45z" ==> 11 to t, 14 to :
    if !(char_count == 11 || char_count == 14) {
        return Err("Invalid DateTimeMsec value");
    }
    let Ok(date_fullyear) = input[..4].parse::<u32>() else {
        return Err("Invalid year in DateTimeMsec value");
    };
    let Some('-') = input.chars().nth(4) else {
        return Err("Invalid DateTimeMsec value");
    };
    let Ok(date_month) = input[5..7].parse::<u8>() else {
        return Err("Invalid month in DateTimeMsec value");
    };
    let Some('-') = input.chars().nth(7) else {
        return Err("Invalid DateTimeMsec value");
    };
    let Ok(date_mday) = input[8..10].parse::<u8>() else {
        return Err("Invalid day in DateTimeMsec value");
    };
    if char_count == 11 {
        let Some('t') = input.chars().nth(10) else {
            return Err("Invalid DateTimeMsec value");
        };
        let _ = input_chars.next();
        let _ = input_chars.next();
        let Some(':') = input_chars.next() else {
            return Err("Invalid DateTimeMsec value");
        };
    } else {
        let Some('T') = input.chars().nth(10) else {
            return Err("Invalid DateTimeMsec value");
        };
    }
    let Ok(time_hour) = input[11..13].parse::<u8>() else {
        return Err("Invalid hour in DateTimeMsec value");
    };
    input_chars.next();
    input_chars.next();
    let Some(':') = input_chars.next() else {
        return Err("Invalid DateTimeMsec value");
    };
    char_count = 17;
    let Ok(time_minute) = input[14..16].parse::<u8>() else {
        return Err("Invalid minute in DateTimeMsec value");
    };
    let time_offset_char = 'time_offset_loop: loop {
        let Some(char) = input_chars.next() else {
            break 'time_offset_loop None;
        };
        char_count += 1;
        match char {
            'Z' | 'z' | '+' | '-' => break 'time_offset_loop Some(char),
            '\r' | '\n' => return Err("Unexpected end of line in DateTimeMsec value"),
            '0'..='9' | '.' => (),
            _ => return Err("Invalid second in DateTimeMsec value"),
        }
    };
    let Some(time_offset_char) = time_offset_char else {
        return Err("Unexpected end of line in DateTimeMsec value");
    };
    let Ok(time_second) = input[17..(char_count - 1)].parse::<f64>() else {
        return Err("Invalid second in DateTimeMsec value");
    };
    match time_offset_char {
        'Z' | 'z' => {
            validate_end_of_line(&mut input_chars)?;
            Ok(DateTime {
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
            })
        }
        _ => {
            let multiplier = if time_offset_char == '-' { -1i8 } else { 1i8 };
            input_chars.next();
            input_chars.next();
            let Some(':') = input_chars.next() else {
                return Err("Invalid DateTimeMsec value");
            };
            let Ok(timeoffset_hour) = input[char_count..(char_count + 2)].parse::<i8>() else {
                return Err("Invalid time offset hour in DateTimeMsec value");
            };
            let timeoffset_hour = multiplier * timeoffset_hour;
            input_chars.next();
            input_chars.next();
            validate_end_of_line(&mut input_chars)?;
            let Ok(timeoffset_minute) = input[(char_count + 3)..].parse::<u8>() else {
                return Err("Invalid time offset minute in DateTimeMsec value");
            };
            Ok(DateTime {
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
            })
        }
    }
}

fn validate_end_of_line(input_chars: &mut Chars<'_>) -> Result<(), &'static str> {
    match input_chars.next() {
        None => Ok(()),
        Some('\r') => validate_carriage_return(input_chars),
        Some('\n') => validate_new_line(input_chars),
        _ => Err("Expected end of line"),
    }
}

fn validate_carriage_return(input_chars: &mut Chars<'_>) -> Result<(), &'static str> {
    let Some('\n') = input_chars.next() else {
        return Err("Unexpected carriage return without following line feed");
    };
    let None = input_chars.next() else {
        return Err("Unexpected char after line feed");
    };
    Ok(())
}

fn validate_new_line(input_chars: &mut Chars<'_>) -> Result<(), &'static str> {
    match input_chars.next() {
        Some(_) => Err("Unexpected carriage return without following line feed"),
        None => Ok(()),
    }
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
