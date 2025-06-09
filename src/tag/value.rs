use crate::date::{self, DateTime, DateTimeTimezoneOffset};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{self, take_till, take_while},
    combinator::{eof, map, map_res, opt},
    multi::many0,
    sequence::{delimited, terminated},
};
use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

// Not exactly the same as `tag-value`, because some of the types must be contextualized by the
// `tag-name`, but this list covers the possible raw values.
//
// Examples:
//   Empty                                 -> #EXTM3U
//   TypeEnum                              -> #EXT-X-PLAYLIST-TYPE:<type-enum>
//   DecimalInteger                        -> #EXT-X-VERSION:<n>
//   DecimalIntegerRange                   -> #EXT-X-BYTERANGE:<n>[@<o>]
//   DecimalFloatingPointWithOptionalTitle -> #EXTINF:<duration>,[<title>]
//   DateTimeMsec                          -> #EXT-X-PROGRAM-DATE-TIME:<date-time-msec>
//
#[derive(Debug, PartialEq)]
pub enum ParsedTagValue<'a> {
    Empty,
    TypeEnum(HlsPlaylistType),
    DecimalInteger(u64),
    DecimalIntegerRange(u64, u64),
    DecimalFloatingPointWithOptionalTitle(f64, &'a str),
    DateTimeMsec(DateTime),
    AttributeList(HashMap<&'a str, ParsedAttributeValue<'a>>),
}

#[derive(Debug, PartialEq)]
pub enum HlsPlaylistType {
    Event,
    Vod,
}

// Not exactly the same as `attribute-value`, because some of the types must be contextualized by
// the `attribute-name`, but this list covers the possible raw values.
#[derive(Debug, PartialEq)]
pub enum ParsedAttributeValue<'a> {
    DecimalInteger(u64),
    SignedDecimalFloatingPoint(f64),
    QuotedString(&'a str),
    UnquotedString(&'a str),
}

impl<'a> ParsedAttributeValue<'a> {
    /// Helper method to extract `DecimalInteger` value.
    /// ```
    /// use m3u8::tag::value::ParsedAttributeValue;
    ///
    /// assert_eq!(Some(42), ParsedAttributeValue::DecimalInteger(42).as_option_u64());
    /// assert_eq!(None, ParsedAttributeValue::QuotedString("42").as_option_u64());
    /// ```
    pub fn as_option_u64(&self) -> Option<u64> {
        if let Self::DecimalInteger(d) = self {
            Some(*d)
        } else {
            None
        }
    }

    /// Helper method to extract either `DecimalInteger` or `SignedDecimalFloatingPoint` as `f64`.
    ///
    /// We consider both enum cases because at time of parsing we do not yet know the context of the
    /// attribuet to understand whether the value MUST be a positive integer or whether it MAY be
    /// any decimal float. This therefore makes extraction of `f64` values easier.
    ///
    /// For example, consider if we had the tag `#EXT-X-START:TIME-OFFSET=6`. When parsing, we would
    /// consider the value of `TIME-OFFSET` to be `DecimalInteger(6)`; however, the EXT-X-START tag
    /// considers the value of `TIME-OFFSET` to be "a signed-decimal-floating-point number". So to
    /// extract the f64, if this method did not consider both `DecimalInteger` and
    /// `SignedDecimalFloatingPoint` cases, all users of the library would need to know that they
    /// should check both themselves. Therefore, it seems that the more normal usage pattern would
    /// be to take both enum cases into account.
    /// ```
    /// use m3u8::tag::value::ParsedAttributeValue;
    ///
    /// assert_eq!(
    ///     Some(42.0),
    ///     ParsedAttributeValue::SignedDecimalFloatingPoint(42.0).as_option_f64()
    /// );
    /// assert_eq!(Some(42.0), ParsedAttributeValue::DecimalInteger(42).as_option_f64());
    /// assert_eq!(None, ParsedAttributeValue::QuotedString("42").as_option_f64());
    /// ```
    pub fn as_option_f64(&self) -> Option<f64> {
        if let Self::SignedDecimalFloatingPoint(f) = self {
            Some(*f)
        } else if let Self::DecimalInteger(n) = self {
            Some(*n as f64)
        } else {
            None
        }
    }

    /// Helper method to extract `QuotedString` value.
    /// ```
    /// use m3u8::tag::value::ParsedAttributeValue;
    ///
    /// assert_eq!(
    ///     Some("Hello, World!"),
    ///     ParsedAttributeValue::QuotedString("Hello, World!").as_option_quoted_str()
    /// );
    /// assert_eq!(
    ///     None,
    ///     ParsedAttributeValue::UnquotedString("Hello, World!").as_option_quoted_str()
    /// );
    /// ```
    pub fn as_option_quoted_str(&self) -> Option<&str> {
        if let Self::QuotedString(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Helper method to extract `UnquotedString` value.
    /// ```
    /// use m3u8::tag::value::ParsedAttributeValue;
    ///
    /// assert_eq!(
    ///     Some("Hello, World!"),
    ///     ParsedAttributeValue::UnquotedString("Hello, World!").as_option_unquoted_str()
    /// );
    /// assert_eq!(
    ///     None,
    ///     ParsedAttributeValue::QuotedString("Hello, World!").as_option_unquoted_str()
    /// );
    /// ```
    pub fn as_option_unquoted_str(&self) -> Option<&str> {
        if let Self::UnquotedString(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DecimalResolution {
    pub width: u64,
    pub height: u64,
}

pub fn parse_chars(input: &str) -> Result<ParsedTagValue, &'static str> {
    if input.is_empty() {
        return Ok(ParsedTagValue::Empty);
    }
    let mut input_chars = input.chars();
    let mut char_count = 0usize;
    // At the start of all tag value types (except from type-enum) digits are allowed. Therefore,
    // start by seeing how far we get with digits.
    let initial_break_char = 'digits_loop: loop {
        let Some(char) = input_chars.next() else {
            break 'digits_loop None;
        };
        char_count += 1;
        if !char.is_ascii_digit() {
            break 'digits_loop Some(char);
        }
    };
    let Some(initial_break_char) = initial_break_char else {
        // If we reached the end of chars in the digits loop, that implies that the value was all
        // digits, and as such should be treated as a `DecimalInteger(u64)`.
        match input.parse::<u64>() {
            Ok(n) => return Ok(ParsedTagValue::DecimalInteger(n)),
            Err(_) => return Err("Could not parse decimal integer to u64 (perhaps too large)"),
        }
    };
    match initial_break_char {
        // Next char MUST be '\n' otherwise we will fail (for now don't support carriage return with
        // no line feed). Since everything before this was a digit, this must be DecimalInteger.
        '\r' => {
            let Some('\n') = input_chars.next() else {
                return Err("Unexpected carriage return without following line feed");
            };
            let None = input_chars.next() else {
                return Err("Unexpected char after line feed");
            };
            match input.parse::<u64>() {
                Ok(n) => return Ok(ParsedTagValue::DecimalInteger(n)),
                Err(_) => return Err("Could not parse decimal integer to u64 (perhaps too large)"),
            }
        }
        // End of line - should check if there was anything left and fail with error if so. Since
        // everything before this was a digit, this must be DecimalInteger.
        '\n' => {
            let None = input_chars.next() else {
                return Err("Unexpected carriage return without following line feed");
            };
            match input.parse::<u64>() {
                Ok(n) => return Ok(ParsedTagValue::DecimalInteger(n)),
                Err(_) => return Err("Could not parse decimal integer to u64 (perhaps too large)"),
            }
        }
        // This must be DecimalIntegerRange(u64, u64)
        '@' => {
            let length = match input[..(char_count - 1)].parse::<u64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err(
                        "Could not parse decimal length within range value (perhaps too large)",
                    );
                }
            };
            let offset = match input[char_count..].trim_end().parse::<u64>() {
                Ok(n) => n,
                Err(_) => return Err("Could not parse decimal offset within range value"),
            };
            return Ok(ParsedTagValue::DecimalIntegerRange(length, offset));
        }
        // This must be DecimalFloatingPointWithOptionalTitle(f64, &str)
        ',' => {
            let duration = match input[..(char_count - 1)].parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err("Could not parse decimal float (perhaps too large)");
                }
            };
            return Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                duration,
                &input[char_count..].trim_end(),
            ));
        }
        // This must be DecimalFloatingPointWithOptionalTitle(f64, &str)
        '.' => {
            let break_char = 'comma_loop: loop {
                let Some(char) = input_chars.next() else {
                    break 'comma_loop None;
                };
                char_count += 1;
                match char {
                    '0'..='9' => (),
                    ',' => break 'comma_loop Some(char),
                    '\r' => break 'comma_loop Some(char),
                    '\n' => break 'comma_loop Some(char),
                    _ => return Err("Invalid non-digit character in decimal floating point"),
                }
            };
            let title = match break_char {
                None => {
                    // In this sceanrio the digits run all the way through to the end so I add one
                    // for the duration str slice to capture the last character. This should
                    // probably be done more clearly with a dedicated enum.
                    char_count += 1;
                    ""
                }
                Some(',') => &input[char_count..].trim_end(),
                Some('\r') => {
                    let Some('\n') = input_chars.next() else {
                        return Err("Unexpected carriage return without following line feed");
                    };
                    let None = input_chars.next() else {
                        return Err("Unexpected char after line feed");
                    };
                    ""
                }
                Some('\n') => {
                    let None = input_chars.next() else {
                        return Err("Unexpected carriage return without following line feed");
                    };
                    ""
                }
                Some(_) => return Err("Invalid non-digit character in decimal floating point"),
            };
            let duration = match input[..(char_count - 1)].parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err("Could not parse decimal float (perhaps too large)");
                }
            };
            return Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                duration, title,
            ));
        }
        _ => (),
    }
    // The types of value left to check (and what char could start it) are:
    //   - TypeEnum (`'V' | 'O' | 'D' | 'E' | 'V' | 'N' | 'T'`)
    //   - DecimalFloatingPointWithOptionalTitle (`'-'`)
    //   - DateTimeMsec (`'-'`)
    //   - AttributeList (`'-' | '0'..='9' | 'A'..='Z'`)
    let mut still_checking_enum = true;
    let mut still_checking_float = true;
    let mut still_checking_date = true;
    match initial_break_char {
        'V' | 'O' | 'D' | 'E' | 'N' | 'T' => {
            still_checking_float = false;
            still_checking_date = false;
        }
        '-' => {
            still_checking_enum = false;
            match char_count {
                1 => still_checking_date = false,
                5 | 8 => still_checking_float = false,
                _ => {
                    still_checking_date = false;
                    still_checking_float = false;
                }
            }
        }
        'A'..='Z' => {
            still_checking_enum = false;
            still_checking_float = false;
            still_checking_date = false;
        }
        _ => return Err("Unexpected char in tag value"),
    }
    // Now we keep looping until the next control character for the above types.
    let second_break_char = 'value_name_loop: loop {
        let Some(char) = input_chars.next() else {
            break 'value_name_loop None;
        };
        char_count += 1;
        match char {
            // End of line
            '\r' | '\n' => break 'value_name_loop Some(char),
            // Attribute name separator
            '=' => break 'value_name_loop Some(char),
            // DateTime separators
            't' | ':' => {
                if still_checking_date {
                    break 'value_name_loop Some(char);
                } else {
                    return Err("Unexpected char in tag value");
                }
            }
            // Float chars
            ',' | '.' => {
                if still_checking_float {
                    break 'value_name_loop Some(char);
                } else {
                    return Err("Unexpected char in tag value");
                }
            }
            // Type enum
            'V' | 'O' | 'D' | 'E' | 'N' | 'T' => {
                still_checking_float = false;
                if char != 'T' {
                    still_checking_date = false;
                }
            }
            // Attribute name
            'A'..='Z' => {
                still_checking_enum = false;
                if !(char_count == 11 && char == 'T') {
                    still_checking_date = false;
                }
                still_checking_float = false;
            }
            '-' => {
                still_checking_enum = false;
                if !(char_count == 5 || char_count == 8) {
                    still_checking_date = false;
                }
                still_checking_float = false;
            }
            '0'..='9' => {
                still_checking_enum = false;
                if char_count == 5 || char_count == 8 {
                    still_checking_date = false;
                }
            }
            _ => return Err("Unexpected char in tag value"),
        }
    };
    let Some(second_break_char) = second_break_char else {
        if still_checking_enum {
            if input == "VOD" {
                return Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod));
            } else if input == "EVENT" {
                return Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event));
            } else {
                return Err("Unexpected end of tag value");
            }
        } else if still_checking_float {
            let duration = match input.parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err("Could not parse decimal float (perhaps too large)");
                }
            };
            return Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                duration, "",
            ));
        } else {
            return Err("Unexpected end of tag value");
        }
    };
    match second_break_char {
        // End of line
        '\r' => {
            let Some('\n') = input_chars.next() else {
                return Err("Unexpected carriage return without following line feed");
            };
            let None = input_chars.next() else {
                return Err("Unexpected char after line feed");
            };
            if still_checking_enum {
                if input == "VOD" {
                    return Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod));
                } else if input == "EVENT" {
                    return Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event));
                } else {
                    return Err("Unexpected end of tag value");
                }
            } else if still_checking_float {
                let duration = match input.parse::<f64>() {
                    Ok(n) => n,
                    Err(_) => {
                        return Err("Could not parse decimal float (perhaps too large)");
                    }
                };
                return Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    duration, "",
                ));
            } else {
                return Err("Unexpected end of tag value");
            }
        }
        '\n' => {
            let None = input_chars.next() else {
                return Err("Unexpected char after line feed");
            };
            if still_checking_enum {
                if input == "VOD" {
                    return Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod));
                } else if input == "EVENT" {
                    return Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event));
                } else {
                    return Err("Unexpected end of tag value");
                }
            } else if still_checking_float {
                let duration = match input.parse::<f64>() {
                    Ok(n) => n,
                    Err(_) => {
                        return Err("Could not parse decimal float (perhaps too large)");
                    }
                };
                return Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    duration, "",
                ));
            } else {
                return Err("Unexpected end of tag value");
            }
        }
        // DateTime separators
        't' | ':' => {
            // "2025-06-09T17:53:45" - 11 to t, 14 to :
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
                    'Z' | '+' | '-' => break 'time_offset_loop Some(char),
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
                'Z' => {
                    // Validate 'Z' is end of line.
                    match input_chars.next() {
                        None => (),
                        Some('\r') => match input_chars.next() {
                            Some('\n') => match input_chars.next() {
                                None => (),
                                _ => return Err("Invalid DateTimeMsec value"),
                            },
                            _ => return Err("Invalid DateTimeMsec value"),
                        },
                        Some('\n') => match input_chars.next() {
                            None => (),
                            _ => return Err("Invalid DateTimeMsec value"),
                        },
                        _ => return Err("Invalid DateTimeMsec value"),
                    }
                    return Ok(ParsedTagValue::DateTimeMsec(DateTime {
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
                    }));
                }
                _ => {
                    let multiplier = if time_offset_char == '-' { -1i8 } else { 1i8 };
                    input_chars.next();
                    input_chars.next();
                    let Some(':') = input_chars.next() else {
                        return Err("Invalid DateTimeMsec value");
                    };
                    let Ok(timeoffset_hour) = input[char_count..(char_count + 2)].parse::<i8>()
                    else {
                        return Err("Invalid time offset hour in DateTimeMsec value");
                    };
                    let timeoffset_hour = multiplier * timeoffset_hour;
                    input_chars.next();
                    input_chars.next();
                    // Validate is end of line.
                    match input_chars.next() {
                        None => (),
                        Some('\r') => match input_chars.next() {
                            Some('\n') => match input_chars.next() {
                                None => (),
                                _ => return Err("Invalid DateTimeMsec value"),
                            },
                            _ => return Err("Invalid DateTimeMsec value"),
                        },
                        Some('\n') => match input_chars.next() {
                            None => (),
                            _ => return Err("Invalid DateTimeMsec value"),
                        },
                        _ => return Err("Invalid DateTimeMsec value"),
                    }
                    let Ok(timeoffset_minute) = input[(char_count + 3)..].parse::<u8>() else {
                        return Err("Invalid time offset minute in DateTimeMsec value");
                    };
                    return Ok(ParsedTagValue::DateTimeMsec(DateTime {
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
                    }));
                }
            }
        }
        ',' => {
            let duration = match input[..(char_count - 1)].parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err("Could not parse decimal float (perhaps too large)");
                }
            };
            return Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                duration,
                &input[char_count..].trim_end(),
            ));
        }
        '.' => {
            let break_char = 'comma_loop: loop {
                let Some(char) = input_chars.next() else {
                    break 'comma_loop None;
                };
                char_count += 1;
                match char {
                    '0'..='9' => (),
                    ',' => break 'comma_loop Some(char),
                    '\r' => break 'comma_loop Some(char),
                    '\n' => break 'comma_loop Some(char),
                    _ => return Err("Invalid non-digit character in decimal floating point"),
                }
            };
            let title = match break_char {
                None => {
                    // In this sceanrio the digits run all the way through to the end so I add one
                    // for the duration str slice to capture the last character. This should
                    // probably be done more clearly with a dedicated enum.
                    char_count += 1;
                    ""
                }
                Some(',') => &input[char_count..].trim_end(),
                Some('\r') => {
                    let Some('\n') = input_chars.next() else {
                        return Err("Unexpected carriage return without following line feed");
                    };
                    let None = input_chars.next() else {
                        return Err("Unexpected char after line feed");
                    };
                    ""
                }
                Some('\n') => {
                    let None = input_chars.next() else {
                        return Err("Unexpected carriage return without following line feed");
                    };
                    ""
                }
                Some(_) => return Err("Invalid non-digit character in decimal floating point"),
            };
            let duration = match input[..(char_count - 1)].parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err("Could not parse decimal float (perhaps too large)");
                }
            };
            return Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                duration, title,
            ));
        }
        // Attribute list separator
        '=' => todo!(),
        _ => return Err("Unexpected char in tag value"),
    }
}

#[cfg(test)]
mod parse_chars_tests {
    use super::*;
    use crate::date::DateTimeTimezoneOffset;
    use pretty_assertions::assert_eq;

    #[ignore = "Not implemented yet"]
    #[test]
    fn type_enum() {
        assert_eq!(
            Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event)),
            parse_chars("EVENT")
        );
        assert_eq!(
            Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod)),
            parse_chars("VOD")
        );
    }

    #[test]
    fn decimal_integer() {
        assert_eq!(Ok(ParsedTagValue::DecimalInteger(42)), parse_chars("42"));
    }

    #[test]
    fn decimal_integer_range() {
        assert_eq!(
            Ok(ParsedTagValue::DecimalIntegerRange(42, 42)),
            parse_chars("42@42")
        );
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        // Positive tests
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0, ""
            )),
            parse_chars("42.0")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.42, ""
            )),
            parse_chars("42.42")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0, ""
            )),
            parse_chars("42,")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0,
                "=ATTRIBUTE-VALUE"
            )),
            parse_chars("42,=ATTRIBUTE-VALUE")
        );
        // Negative tests
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0, ""
            )),
            parse_chars("-42.0")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.42, ""
            )),
            parse_chars("-42.42")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0, ""
            )),
            parse_chars("-42,")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0,
                "=ATTRIBUTE-VALUE"
            )),
            parse_chars("-42,=ATTRIBUTE-VALUE")
        );
    }

    #[test]
    fn date_time_msec() {
        assert_eq!(
            Ok(ParsedTagValue::DateTimeMsec(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 3,
                time_hour: 17,
                time_minute: 56,
                time_second: 42.123,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0,
                }
            })),
            parse_chars("2025-06-03T17:56:42.123Z")
        );
        assert_eq!(
            Ok(ParsedTagValue::DateTimeMsec(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 3,
                time_hour: 17,
                time_minute: 56,
                time_second: 42.123,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 1,
                    time_minute: 0,
                }
            })),
            parse_chars("2025-06-03T17:56:42.123+01:00")
        );
        assert_eq!(
            Ok(ParsedTagValue::DateTimeMsec(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 3,
                time_hour: 17,
                time_minute: 56,
                time_second: 42.123,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -5,
                    time_minute: 0,
                }
            })),
            parse_chars("2025-06-03T17:56:42.123-05:00")
        );
    }
}

pub fn parse(input: &str) -> IResult<&str, ParsedTagValue> {
    if input.is_empty() {
        return Ok((input, ParsedTagValue::Empty));
    }
    let (input, value) = opt(alt((complete::tag("EVENT"), complete::tag("VOD")))).parse(input)?;
    if let Some(playlist_type) = value {
        if playlist_type == "EVENT" {
            return Ok((input, ParsedTagValue::TypeEnum(HlsPlaylistType::Event)));
        } else {
            return Ok((input, ParsedTagValue::TypeEnum(HlsPlaylistType::Vod)));
        }
    }
    if let (input, Some(parsed_date)) = opt(date::parse).parse(input)? {
        return Ok((input, ParsedTagValue::DateTimeMsec(parsed_date)));
    }
    let (input, parsed) = take_till(|c| ",=@".contains(c)).parse(input)?;
    match input.chars().next() {
        // Can only be an AttributeList
        Some('=') => {
            let (input, attribute_value) = handle_tag_value_equals_sign(input)?;
            let mut attribute_list = HashMap::new();
            attribute_list.insert(parsed, attribute_value);
            let (input, list) = many0(attribute_list_name_and_value).parse(input)?;
            for (attribute_name, attribute_value) in list {
                attribute_list.insert(attribute_name, attribute_value);
            }
            Ok((input, ParsedTagValue::AttributeList(attribute_list)))
        }
        // Can only be a DecimalFloatingPointWithOptionalTitle
        Some(',') => handle_tag_value_comma(input, parsed),
        // Can only be a DecimalIntegerRange
        Some('@') => handle_tag_value_at_sign(input, parsed),
        // Could be DecimalInteger or DecimalFloatingPointWithOptionalTitle
        _ => handle_tag_value_end_of_line(parsed),
    }
}

fn handle_tag_value_equals_sign(input: &str) -> IResult<&str, ParsedAttributeValue> {
    let (input, _) = complete::tag("=")(input)?;
    alt((
        // DecimalInteger(u64)
        map_res(
            terminated(
                take_while(|c: char| c.is_ascii_digit()),
                alt((eof, complete::tag(","))),
            ),
            |value: &str| -> Result<ParsedAttributeValue, ParseIntError> {
                let number = value.parse::<u64>()?;
                Ok(ParsedAttributeValue::DecimalInteger(number))
            },
        ),
        // SignedDecimalFloatingPoint(f64)
        map_res(
            terminated(
                take_while(|c: char| "-.".contains(c) || c.is_ascii_digit()),
                alt((eof, complete::tag(","))),
            ),
            |value: &str| -> Result<ParsedAttributeValue, ParseFloatError> {
                let number = value.parse::<f64>()?;
                Ok(ParsedAttributeValue::SignedDecimalFloatingPoint(number))
            },
        ),
        // QuotedString(&'a str)
        map(
            terminated(
                delimited(
                    complete::tag("\""),
                    take_while(|c: char| c != '"'),
                    complete::tag("\""),
                ),
                alt((eof, complete::tag(","))),
            ),
            ParsedAttributeValue::QuotedString,
        ),
        // UnquotedString(&'a str)
        map(
            terminated(
                take_while(|c: char| !"\", ".contains(c)),
                alt((eof, complete::tag(","))),
            ),
            ParsedAttributeValue::UnquotedString,
        ),
    ))
    .parse(input)
}

fn attribute_list_name_and_value(input: &str) -> IResult<&str, (&str, ParsedAttributeValue)> {
    let (input, attribute_name) = take_till(|c| c == '=').parse(input)?;
    let (input, attribute_value) = handle_tag_value_equals_sign(input)?;
    Ok((input, (attribute_name, attribute_value)))
}

fn handle_tag_value_comma<'a>(
    input: &'a str,
    parsed: &'a str,
) -> IResult<&'a str, ParsedTagValue<'a>> {
    let (input, _) = complete::tag(",")(input)?;
    let Ok(decimal_floating_point) = parsed.parse::<f64>() else {
        return Err(nom::Err::Failure(nom::error::Error::new(
            parsed,
            nom::error::ErrorKind::Digit,
        )));
    };
    Ok((
        "",
        ParsedTagValue::DecimalFloatingPointWithOptionalTitle(decimal_floating_point, input),
    ))
}

fn handle_tag_value_at_sign<'a>(
    input: &'a str,
    parsed: &'a str,
) -> IResult<&'a str, ParsedTagValue<'a>> {
    let (input, _) = complete::tag("@")(input)?;
    let Ok(first_int) = parsed.parse::<u64>() else {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Digit,
        )));
    };
    if input.is_empty() {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )));
    }
    let Ok(second_int) = input.parse::<u64>() else {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Digit,
        )));
    };
    Ok((
        "",
        ParsedTagValue::DecimalIntegerRange(first_int, second_int),
    ))
}

fn handle_tag_value_end_of_line(parsed: &str) -> IResult<&str, ParsedTagValue> {
    let Ok(decimal_integer) = parsed.parse::<u64>() else {
        let Ok(decimal_floating_point) = parsed.parse::<f64>() else {
            return Err(nom::Err::Failure(nom::error::Error::new(
                parsed,
                nom::error::ErrorKind::Digit,
            )));
        };
        return Ok((
            "",
            ParsedTagValue::DecimalFloatingPointWithOptionalTitle(decimal_floating_point, ""),
        ));
    };
    Ok(("", ParsedTagValue::DecimalInteger(decimal_integer)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::DateTimeTimezoneOffset;
    use pretty_assertions::assert_eq;

    #[test]
    fn type_enum() {
        assert_eq!(
            Ok(("", ParsedTagValue::TypeEnum(HlsPlaylistType::Event))),
            parse("EVENT")
        );
        assert_eq!(
            Ok(("", ParsedTagValue::TypeEnum(HlsPlaylistType::Vod))),
            parse("VOD")
        );
    }

    #[test]
    fn decimal_integer() {
        assert_eq!(Ok(("", ParsedTagValue::DecimalInteger(42))), parse("42"));
    }

    #[test]
    fn decimal_integer_range() {
        assert_eq!(
            Ok(("", ParsedTagValue::DecimalIntegerRange(42, 42))),
            parse("42@42")
        );
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        assert_eq!(
            Ok((
                "",
                ParsedTagValue::DecimalFloatingPointWithOptionalTitle(42.0, "")
            )),
            parse("42.0")
        );
        assert_eq!(
            Ok((
                "",
                ParsedTagValue::DecimalFloatingPointWithOptionalTitle(42.42, "")
            )),
            parse("42.42")
        );
        assert_eq!(
            Ok((
                "",
                ParsedTagValue::DecimalFloatingPointWithOptionalTitle(42.0, "")
            )),
            parse("42,")
        );
        assert_eq!(
            Ok((
                "",
                ParsedTagValue::DecimalFloatingPointWithOptionalTitle(42.0, "=ATTRIBUTE-VALUE")
            )),
            parse("42,=ATTRIBUTE-VALUE")
        );
    }

    #[test]
    fn date_time_msec() {
        assert_eq!(
            Ok((
                "",
                ParsedTagValue::DateTimeMsec(DateTime {
                    date_fullyear: 2025,
                    date_month: 6,
                    date_mday: 3,
                    time_hour: 17,
                    time_minute: 56,
                    time_second: 42.123,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: 0,
                        time_minute: 0,
                    }
                })
            )),
            parse("2025-06-03T17:56:42.123Z")
        );
    }

    mod attribute_list {
        use super::*;

        mod decimal_integer {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn eof_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::DecimalInteger(123))),
                    handle_tag_value_equals_sign("=123")
                );
            }

            #[test]
            fn comma_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::DecimalInteger(123))),
                    handle_tag_value_equals_sign("=123,")
                );
            }
        }

        mod signed_decimal_floating_point {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn positive_float_eof_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(42.42))),
                    handle_tag_value_equals_sign("=42.42")
                );
            }

            #[test]
            fn negative_integer_eof_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0))),
                    handle_tag_value_equals_sign("=-42")
                );
            }

            #[test]
            fn negative_float_eof_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42))),
                    handle_tag_value_equals_sign("=-42.42")
                );
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(42.42))),
                    handle_tag_value_equals_sign("=42.42,")
                );
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0))),
                    handle_tag_value_equals_sign("=-42,")
                );
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42))),
                    handle_tag_value_equals_sign("=-42.42,")
                );
            }

            #[test]
            fn positive_float_comma_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(42.42))),
                    handle_tag_value_equals_sign("=42.42,")
                );
            }

            #[test]
            fn negative_integer_comma_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0))),
                    handle_tag_value_equals_sign("=-42,")
                );
            }

            #[test]
            fn negative_float_comma_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42))),
                    handle_tag_value_equals_sign("=-42.42,")
                );
            }
        }

        mod quoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn eof_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::QuotedString("Hello, World!"))),
                    handle_tag_value_equals_sign("=\"Hello, World!\"")
                );
            }

            #[test]
            fn comma_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::QuotedString("Hello, World!"))),
                    handle_tag_value_equals_sign("=\"Hello, World!\",")
                );
            }
        }

        mod unquoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn eof_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::UnquotedString("PQ"))),
                    handle_tag_value_equals_sign("=PQ")
                );
            }

            #[test]
            fn comma_terminated() {
                assert_eq!(
                    Ok(("", ParsedAttributeValue::UnquotedString("PQ"))),
                    handle_tag_value_equals_sign("=PQ,")
                );
            }
        }
    }
}
