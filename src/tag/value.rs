use crate::date::{DateTime, DateTimeTimezoneOffset};
use std::{collections::HashMap, str::Chars};

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

pub fn parse(input: &str) -> Result<ParsedTagValue, &'static str> {
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
            validate_carriage_return(&mut input_chars)?;
            match input.parse::<u64>() {
                Ok(n) => return Ok(ParsedTagValue::DecimalInteger(n)),
                Err(_) => return Err("Could not parse decimal integer to u64 (perhaps too large)"),
            }
        }
        // End of line - should check if there was anything left and fail with error if so. Since
        // everything before this was a digit, this must be DecimalInteger.
        '\n' => {
            validate_new_line(&mut input_chars)?;
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
        ',' => return handle_float_with_title_on_comma(input, char_count),
        // This must be DecimalFloatingPointWithOptionalTitle(f64, &str)
        '.' => return handle_float_with_title_on_period(input, char_count, input_chars),
        _ => (),
    }
    // The types of value left to check (and what char could start it) are:
    //   - TypeEnum (`'V' | 'O' | 'D' | 'E' | 'N' | 'T'`)
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
            return handle_type_enum(input);
        } else if still_checking_float {
            // Add 1 to char_count so takes last char in line.
            return handle_float_with_title_on_end_of_line(input, char_count + 1);
        } else {
            return Err("Unexpected end of tag value");
        }
    };
    match second_break_char {
        // End of line
        '\r' => {
            validate_carriage_return(&mut input_chars)?;
            if still_checking_enum {
                handle_type_enum(input)
            } else if still_checking_float {
                handle_float_with_title_on_end_of_line(input, char_count)
            } else {
                Err("Unexpected end of tag value")
            }
        }
        '\n' => {
            validate_new_line(&mut input_chars)?;
            if still_checking_enum {
                handle_type_enum(input)
            } else if still_checking_float {
                handle_float_with_title_on_end_of_line(input, char_count)
            } else {
                return Err("Unexpected end of tag value");
            }
        }
        // DateTime separators
        't' | ':' => handle_date_time(input, char_count, input_chars),
        // Float with title break chars
        ',' => handle_float_with_title_on_comma(input, char_count),
        '.' => handle_float_with_title_on_period(input, char_count, input_chars),
        // Attribute list separator
        '=' => {
            let mut attribute_list = HashMap::new();
            let initial_attribute_name = &input[..(char_count - 1)];
            let (has_more, initial_attribute_value, new_char_count) =
                handle_attribute_value(input, char_count, &mut input_chars)?;
            char_count = new_char_count;
            attribute_list.insert(initial_attribute_name, initial_attribute_value);
            if has_more {
                'attribute_loop: loop {
                    let (attribute_name, new_char_count) =
                        handle_attribute_name(input, char_count, &mut input_chars)?;
                    char_count = new_char_count;
                    let (has_more, attribute_value, new_char_count) =
                        handle_attribute_value(input, char_count, &mut input_chars)?;
                    char_count = new_char_count;
                    attribute_list.insert(attribute_name, attribute_value);
                    if !has_more {
                        break 'attribute_loop;
                    }
                }
            }
            Ok(ParsedTagValue::AttributeList(attribute_list))
        }
        _ => Err("Unexpected char in tag value"),
    }
}

fn handle_attribute_name<'a>(
    input: &'a str,
    mut char_count: usize,
    input_chars: &mut Chars<'_>,
) -> Result<(&'a str, usize), &'static str> {
    let name_start_index = char_count;
    loop {
        let Some(char) = input_chars.next() else {
            return Err("Unexpected end of line while reading attribute name");
        };
        char_count += 1;
        match char {
            '=' => break,
            'A'..='Z' | '0'..='9' | '-' => (),
            _ => return Err("Unexpected char while reading attribute name"),
        }
    }
    Ok((&input[name_start_index..(char_count - 1)], char_count))
}

fn handle_attribute_value<'a>(
    input: &'a str,
    mut char_count: usize,
    input_chars: &mut Chars<'_>,
) -> Result<(bool, ParsedAttributeValue<'a>, usize), &'static str> {
    let value_start_index = char_count;
    let Some(initial_char) = input_chars.next() else {
        return Err("Unexpected empty attribute value");
    };
    char_count += 1;
    let (mut still_parsing_integer, mut still_parsing_float) = (true, true);
    match initial_char {
        '"' => {
            'double_quotes_loop: loop {
                let Some(char) = input_chars.next() else {
                    return Err("Unexpected end of line while reading attribute value");
                };
                char_count += 1;
                match char {
                    '"' => break 'double_quotes_loop,
                    '\r' => return Err("Unexpected carriage return while reading quoted string"),
                    '\n' => return Err("Unexpected line feed while reading quoted string"),
                    _ => (),
                }
            }
            let value = &input[(value_start_index + 1)..(char_count - 1)];
            char_count += 1;
            return match input_chars.next() {
                Some(',') => Ok((true, ParsedAttributeValue::QuotedString(value), char_count)),
                None => Ok((false, ParsedAttributeValue::QuotedString(value), char_count)),
                Some('\r') => {
                    validate_carriage_return(input_chars)?;
                    Ok((false, ParsedAttributeValue::QuotedString(value), char_count))
                }
                Some('\n') => {
                    validate_new_line(input_chars)?;
                    Ok((false, ParsedAttributeValue::QuotedString(value), char_count))
                }
                _ => Err("Unexpected char while reading value"),
            };
        }
        '-' => still_parsing_integer = false,
        c if c.is_ascii_whitespace() => return Err("Unexpected whitespace in attribute value"),
        ',' => return Err("Unexpected comma in attribute value"),
        c if !c.is_ascii_digit() => {
            still_parsing_integer = false;
            still_parsing_float = false;
        }
        _ => (),
    }
    let more_attributes_exist = loop {
        char_count += 1; // Before next to ensure if end of line we take whole value.
        let Some(char) = input_chars.next() else {
            break false;
        };
        match char {
            '.' => still_parsing_integer = false,
            ',' => break true,
            '0'..='9' => (),
            '\r' => {
                validate_carriage_return(input_chars)?;
                break false;
            }
            '\n' => {
                validate_new_line(input_chars)?;
                break false;
            }
            _ => {
                still_parsing_integer = false;
                still_parsing_float = false;
            }
        }
    };
    let number_value = if still_parsing_integer {
        input[value_start_index..(char_count - 1)]
            .parse::<u64>()
            .ok()
            .map(|n| ParsedAttributeValue::DecimalInteger(n))
    } else if still_parsing_float {
        input[value_start_index..(char_count - 1)]
            .parse::<f64>()
            .ok()
            .map(|n| ParsedAttributeValue::SignedDecimalFloatingPoint(n))
    } else {
        None
    };
    if let Some(number_value) = number_value {
        Ok((more_attributes_exist, number_value, char_count))
    } else {
        Ok((
            more_attributes_exist,
            ParsedAttributeValue::UnquotedString(&input[value_start_index..(char_count - 1)]),
            char_count,
        ))
    }
}

fn handle_date_time<'a>(
    input: &'a str,
    mut char_count: usize,
    mut input_chars: Chars<'_>,
) -> Result<ParsedTagValue<'a>, &'static str> {
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
            Ok(ParsedTagValue::DateTimeMsec(DateTime {
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
            }))
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
            Ok(ParsedTagValue::DateTimeMsec(DateTime {
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
            }))
        }
    }
}

fn handle_type_enum(input: &str) -> Result<ParsedTagValue, &'static str> {
    if &input[..=2] == "VOD" {
        Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod))
    } else if &input[..=4] == "EVENT" {
        Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event))
    } else {
        Err("Unexpected end of tag value")
    }
}

fn handle_float_with_title_on_end_of_line(
    input: &str,
    char_count: usize,
) -> Result<ParsedTagValue, &'static str> {
    let duration = match input[..char_count].parse::<f64>() {
        Ok(n) => n,
        Err(_) => {
            return Err("Could not parse decimal float (perhaps too large)");
        }
    };
    Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
        duration, "",
    ))
}

fn handle_float_with_title_on_comma(
    input: &str,
    char_count: usize,
) -> Result<ParsedTagValue, &'static str> {
    let duration = match input[..(char_count - 1)].parse::<f64>() {
        Ok(n) => n,
        Err(_) => {
            return Err("Could not parse decimal float (perhaps too large)");
        }
    };
    Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
        duration,
        &input[char_count..].trim_end(),
    ))
}

fn handle_float_with_title_on_period<'a>(
    input: &'a str,
    mut char_count: usize,
    mut input_chars: Chars<'_>,
) -> Result<ParsedTagValue<'a>, &'static str> {
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
            validate_carriage_return(&mut input_chars)?;
            ""
        }
        Some('\n') => {
            validate_new_line(&mut input_chars)?;
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
    Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
        duration, title,
    ))
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
    use crate::date::DateTimeTimezoneOffset;
    use pretty_assertions::assert_eq;

    #[test]
    fn type_enum() {
        assert_eq!(
            Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event)),
            parse("EVENT")
        );
        assert_eq!(
            Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod)),
            parse("VOD")
        );
    }

    #[test]
    fn decimal_integer() {
        assert_eq!(Ok(ParsedTagValue::DecimalInteger(42)), parse("42"));
    }

    #[test]
    fn decimal_integer_range() {
        assert_eq!(
            Ok(ParsedTagValue::DecimalIntegerRange(42, 42)),
            parse("42@42")
        );
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        // Positive tests
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0, ""
            )),
            parse("42.0")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.42, ""
            )),
            parse("42.42")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0, ""
            )),
            parse("42,")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0,
                "=ATTRIBUTE-VALUE"
            )),
            parse("42,=ATTRIBUTE-VALUE")
        );
        // Negative tests
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0, ""
            )),
            parse("-42.0")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.42, ""
            )),
            parse("-42.42")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0, ""
            )),
            parse("-42,")
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0,
                "=ATTRIBUTE-VALUE"
            )),
            parse("-42,=ATTRIBUTE-VALUE")
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
            parse("2025-06-03T17:56:42.123Z")
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
            parse("2025-06-03T17:56:42.123+01:00")
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
            parse("2025-06-03T17:56:42.123-05:00")
        );
    }

    mod attribute_list {
        use super::*;

        mod decimal_integer {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::DecimalInteger(123)
                    )]))),
                    parse("NAME=123")
                );
            }

            #[test]
            fn multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        ("NAME", ParsedAttributeValue::DecimalInteger(123)),
                        ("NEXT-NAME", ParsedAttributeValue::DecimalInteger(456))
                    ]))),
                    parse("NAME=123,NEXT-NAME=456")
                );
            }
        }

        mod signed_decimal_floating_point {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn positive_float_single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(42.42)
                    )]))),
                    parse("NAME=42.42")
                );
            }

            #[test]
            fn negative_integer_single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0)
                    )]))),
                    parse("NAME=-42")
                );
            }

            #[test]
            fn negative_float_single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42)
                    )]))),
                    parse("NAME=-42.42")
                );
            }

            #[test]
            fn positive_float_multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        (
                            "NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(42.42)
                        ),
                        (
                            "NEXT-NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(84.84)
                        )
                    ]))),
                    parse("NAME=42.42,NEXT-NAME=84.84")
                );
            }

            #[test]
            fn negative_integer_multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        (
                            "NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0)
                        ),
                        (
                            "NEXT-NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0)
                        )
                    ]))),
                    parse("NAME=-42,NEXT-NAME=-42")
                );
            }

            #[test]
            fn negative_float_multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        (
                            "NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42)
                        ),
                        (
                            "NEXT-NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(-84.84)
                        )
                    ]))),
                    parse("NAME=-42.42,NEXT-NAME=-84.84")
                );
            }
        }

        mod quoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::QuotedString("Hello, World!")
                    )]))),
                    parse("NAME=\"Hello, World!\"")
                );
            }

            #[test]
            fn multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        ("NAME", ParsedAttributeValue::QuotedString("Hello,")),
                        ("NEXT-NAME", ParsedAttributeValue::QuotedString("World!"))
                    ]))),
                    parse("NAME=\"Hello,\",NEXT-NAME=\"World!\"")
                );
            }
        }

        mod unquoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::UnquotedString("PQ")
                    )]))),
                    parse("NAME=PQ")
                );
            }

            #[test]
            fn multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        ("NAME", ParsedAttributeValue::UnquotedString("PQ")),
                        ("NEXT-NAME", ParsedAttributeValue::UnquotedString("HLG"))
                    ]))),
                    parse("NAME=PQ,NEXT-NAME=HLG")
                );
            }
        }
    }
}
