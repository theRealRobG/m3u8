use crate::{
    date::{DateTime, DateTimeTimezoneOffset},
    line::ParsedLineSlice,
    utils::{str_from, take_until_end_of_bytes, validate_carriage_return_bytes},
};
use std::{collections::HashMap, slice::Iter};

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

pub fn new_parse(input: &str) -> Result<ParsedLineSlice<ParsedTagValue>, &'static str> {
    let mut bytes = input.as_bytes().iter();
    let mut count = 0usize;
    // ParsedTagValue {
    //     Empty                                 - b'/r' | b'/n'
    //     TypeEnum                              - b'V' | b'O' | b'D' | b'E' | b'N' | b'T'
    //     DecimalInteger                        - b'0'..=b'9'
    //     DecimalIntegerRange                   - b'0'..=b'9' | b'@'
    //     DecimalFloatingPointWithOptionalTitle - b'0'..=b'9' | b'-' | b'.' | b','
    //     DateTimeMsec                          - b'0'..=b'9' | b'-' | b'T' | b't' | b':'
    //     AttributeList                         - b'0'..=b'9' | b'-' | b'A'..=b'Z' | b'='
    // }
    let mut parsing_empty = true;
    let mut parsing_enum = true;
    let mut parsing_int = true;
    let mut parsing_range = true;
    let mut parsing_float = true;
    let mut parsing_date = true;
    let mut parsing_attr_name = true;
    let break_byte = loop {
        let Some(byte) = bytes.next() else {
            break None;
        };
        count += 1;
        match byte {
            b'\r' | b'\n' => break Some(byte),
            b'0'..=b'9' => {
                parsing_empty = false;
                parsing_enum = false;
            }
            b'.' => {
                if !parsing_float {
                    return Err("Unexpected character in tag value");
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_range = false;
                parsing_float = true;
                parsing_date = false;
                parsing_attr_name = false;
            }
            b',' => {
                if !parsing_float {
                    return Err("Unexpected character in tag value");
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_float = true;
                break Some(byte);
            }
            b'@' => {
                if !parsing_range {
                    return Err("Unexpected character in tag value");
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_float = false;
                break Some(byte);
            }
            b'-' => {
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_range = false;
            }
            b'T' => {
                parsing_empty = false;
                parsing_int = false;
                parsing_range = false;
                parsing_float = false;
            }
            b'V' | b'O' | b'D' | b'E' | b'N' => {
                parsing_empty = false;
                parsing_int = false;
                parsing_range = false;
                parsing_float = false;
                parsing_date = false;
            }
            b't' | b':' => {
                if !parsing_date {
                    return Err("Unexpected character in tag value");
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_float = false;
                break Some(byte);
            }
            b'A'..=b'Z' => {
                if !parsing_attr_name {
                    return Err("Unexpected character in tag value");
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_range = false;
                parsing_float = false;
                parsing_date = false;
                parsing_attr_name = true;
            }
            b'=' => {
                if !parsing_attr_name {
                    return Err("Unexpected character in tag value");
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_float = false;
                break Some(byte);
            }
            _ => return Err("Unexpected character in tag value"),
        }
    };

    let remaining = match break_byte {
        None => None,
        Some(b'\r') => {
            validate_carriage_return_bytes(&mut bytes)?;
            Some(bytes.as_slice())
        }
        Some(b'\n') => Some(bytes.as_slice()),
        Some(b',') => {
            let n = input[..(count - 1)]
                .parse::<f64>()
                .map_err(|_| "Could not parse decimal float in tag value")?;
            let title = take_until_end_of_bytes(bytes)?;
            return Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(n, title.parsed),
                remaining: title.remaining,
            });
        }
        Some(b'@') => {
            let rest = take_until_end_of_bytes(bytes)?;
            let length = input[..(count - 1)]
                .parse::<u64>()
                .map_err(|_| "Could not parse decimal integer in tag value")?;
            let offset = rest
                .parsed
                .parse::<u64>()
                .map_err(|_| "Could not parse decimal integer in tag value")?;
            return Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DecimalIntegerRange(length, offset),
                remaining: rest.remaining,
            });
        }
        Some(b't') | Some(b':') => {
            return parse_date_time_bytes(input, bytes, *break_byte.unwrap());
        }
        Some(b'=') => {
            let mut attribute_list = HashMap::new();
            let initial_attribute_name = &input[..(count - 1)];
            let (has_more, initial_attribute_value, new_count) =
                handle_attribute_value_bytes(input, count, &mut bytes)?;
            count = new_count;
            attribute_list.insert(initial_attribute_name, initial_attribute_value.parsed);
            let mut remaining = initial_attribute_value.remaining;
            if has_more {
                'attribute_loop: loop {
                    let (attribute_name, new_count) =
                        handle_attribute_name_bytes(input, count, &mut bytes)?;
                    count = new_count;
                    let (has_more, attribute_value, new_count) =
                        handle_attribute_value_bytes(input, count, &mut bytes)?;
                    count = new_count;
                    attribute_list.insert(attribute_name, attribute_value.parsed);
                    remaining = attribute_value.remaining;
                    if !has_more {
                        break 'attribute_loop;
                    }
                }
            }
            return Ok(ParsedLineSlice {
                parsed: ParsedTagValue::AttributeList(attribute_list),
                remaining,
            });
        }
        _ => return Err("Unexpected character in tag value"),
    }
    .map(str_from);
    let end_index = if remaining.is_some() {
        count - 1
    } else {
        count
    };

    if parsing_empty {
        Ok(ParsedLineSlice {
            parsed: ParsedTagValue::Empty,
            remaining,
        })
    } else if parsing_int {
        let n = input[..end_index]
            .parse::<u64>()
            .map_err(|_| "Could not parse decimal integer in tag value")?;
        Ok(ParsedLineSlice {
            parsed: ParsedTagValue::DecimalInteger(n),
            remaining,
        })
    } else if parsing_float {
        let n = input[..end_index]
            .parse::<f64>()
            .map_err(|_| "Could not parse decimal float in tag value")?;
        Ok(ParsedLineSlice {
            parsed: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(n, ""),
            remaining,
        })
    } else if parsing_enum {
        match count {
            3 => {
                if &input[..3] == "VOD" {
                    Ok(ParsedLineSlice {
                        parsed: ParsedTagValue::TypeEnum(HlsPlaylistType::Vod),
                        remaining,
                    })
                } else {
                    Err("Invalid tag value")
                }
            }
            5 => {
                if &input[..5] == "EVENT" {
                    Ok(ParsedLineSlice {
                        parsed: ParsedTagValue::TypeEnum(HlsPlaylistType::Event),
                        remaining,
                    })
                } else {
                    Err("Invalid tag value")
                }
            }
            _ => Err("Invalid tag value"),
        }
    } else {
        Err("Invalid tag value")
    }
}

#[cfg(test)]
mod bytes_tests {
    use super::*;
    use crate::date::DateTimeTimezoneOffset;
    use pretty_assertions::assert_eq;

    #[test]
    fn type_enum() {
        assert_eq!(
            Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event)),
            new_parse("EVENT").map(|p| p.parsed)
        );
        assert_eq!(
            Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod)),
            new_parse("VOD").map(|p| p.parsed)
        );
    }

    #[test]
    fn decimal_integer() {
        assert_eq!(
            Ok(ParsedTagValue::DecimalInteger(42)),
            new_parse("42").map(|p| p.parsed)
        );
    }

    #[test]
    fn decimal_integer_range() {
        assert_eq!(
            Ok(ParsedTagValue::DecimalIntegerRange(42, 42)),
            new_parse("42@42").map(|p| p.parsed)
        );
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        // Positive tests
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0, ""
            )),
            new_parse("42.0").map(|p| p.parsed)
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.42, ""
            )),
            new_parse("42.42").map(|p| p.parsed)
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0, ""
            )),
            new_parse("42,").map(|p| p.parsed)
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                42.0,
                "=ATTRIBUTE-VALUE"
            )),
            new_parse("42,=ATTRIBUTE-VALUE").map(|p| p.parsed)
        );
        // Negative tests
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0, ""
            )),
            new_parse("-42.0").map(|p| p.parsed)
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.42, ""
            )),
            new_parse("-42.42").map(|p| p.parsed)
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0, ""
            )),
            new_parse("-42,").map(|p| p.parsed)
        );
        assert_eq!(
            Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                -42.0,
                "=ATTRIBUTE-VALUE"
            )),
            new_parse("-42,=ATTRIBUTE-VALUE").map(|p| p.parsed)
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
            new_parse("2025-06-03T17:56:42.123Z").map(|p| p.parsed)
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
            new_parse("2025-06-03T17:56:42.123+01:00").map(|p| p.parsed)
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
            new_parse("2025-06-03T17:56:42.123-05:00").map(|p| p.parsed)
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
                    new_parse("NAME=123").map(|p| p.parsed)
                );
            }

            #[test]
            fn multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        ("NAME", ParsedAttributeValue::DecimalInteger(123)),
                        ("NEXT-NAME", ParsedAttributeValue::DecimalInteger(456))
                    ]))),
                    new_parse("NAME=123,NEXT-NAME=456").map(|p| p.parsed)
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
                    new_parse("NAME=42.42").map(|p| p.parsed)
                );
            }

            #[test]
            fn negative_integer_single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0)
                    )]))),
                    new_parse("NAME=-42").map(|p| p.parsed)
                );
            }

            #[test]
            fn negative_float_single_attribute() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([(
                        "NAME",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42)
                    )]))),
                    new_parse("NAME=-42.42").map(|p| p.parsed)
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
                    new_parse("NAME=42.42,NEXT-NAME=84.84").map(|p| p.parsed)
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
                    new_parse("NAME=-42,NEXT-NAME=-42").map(|p| p.parsed)
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
                    new_parse("NAME=-42.42,NEXT-NAME=-84.84").map(|p| p.parsed)
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
                    new_parse("NAME=\"Hello, World!\"").map(|p| p.parsed)
                );
            }

            #[test]
            fn multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        ("NAME", ParsedAttributeValue::QuotedString("Hello,")),
                        ("NEXT-NAME", ParsedAttributeValue::QuotedString("World!"))
                    ]))),
                    new_parse("NAME=\"Hello,\",NEXT-NAME=\"World!\"").map(|p| p.parsed)
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
                    new_parse("NAME=PQ").map(|p| p.parsed)
                );
            }

            #[test]
            fn multi_attributes() {
                assert_eq!(
                    Ok(ParsedTagValue::AttributeList(HashMap::from([
                        ("NAME", ParsedAttributeValue::UnquotedString("PQ")),
                        ("NEXT-NAME", ParsedAttributeValue::UnquotedString("HLG"))
                    ]))),
                    new_parse("NAME=PQ,NEXT-NAME=HLG").map(|p| p.parsed)
                );
            }
        }
    }
}

fn parse_date_time_bytes<'a>(
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
            let Ok(timeoffset_minute) = input[(byte_count + 3)..].parse::<u8>() else {
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

fn handle_attribute_name_bytes<'a>(
    input: &'a str,
    mut char_count: usize,
    bytes: &mut Iter<'a, u8>,
) -> Result<(&'a str, usize), &'static str> {
    let name_start_index = char_count;
    loop {
        let Some(char) = bytes.next() else {
            return Err("Unexpected end of line while reading attribute name");
        };
        char_count += 1;
        match char {
            b'=' => break,
            b'A'..=b'Z' | b'0'..=b'9' | b'-' => (),
            _ => return Err("Unexpected char while reading attribute name"),
        }
    }
    Ok((&input[name_start_index..(char_count - 1)], char_count))
}

fn handle_attribute_value_bytes<'a>(
    input: &'a str,
    mut count: usize,
    bytes: &mut Iter<'a, u8>,
) -> Result<(bool, ParsedLineSlice<'a, ParsedAttributeValue<'a>>, usize), &'static str> {
    let value_start_index = count;
    let Some(initial_byte) = bytes.next() else {
        return Err("Unexpected empty attribute value");
    };
    count += 1;
    let (mut still_parsing_integer, mut still_parsing_float) = (true, true);
    match initial_byte {
        b'"' => {
            'double_quotes_loop: loop {
                let Some(char) = bytes.next() else {
                    return Err("Unexpected end of line while reading attribute value");
                };
                count += 1;
                match char {
                    b'"' => break 'double_quotes_loop,
                    b'\r' => return Err("Unexpected carriage return while reading quoted string"),
                    b'\n' => return Err("Unexpected line feed while reading quoted string"),
                    _ => (),
                }
            }
            let value = &input[(value_start_index + 1)..(count - 1)];
            count += 1;
            return match bytes.next() {
                Some(b',') => Ok((
                    true,
                    ParsedLineSlice {
                        parsed: ParsedAttributeValue::QuotedString(value),
                        remaining: Some(str_from(bytes.as_slice())),
                    },
                    count,
                )),
                Some(b'\n') => Ok((
                    false,
                    ParsedLineSlice {
                        parsed: ParsedAttributeValue::QuotedString(value),
                        remaining: Some(str_from(bytes.as_slice())),
                    },
                    count,
                )),
                None => Ok((
                    false,
                    ParsedLineSlice {
                        parsed: ParsedAttributeValue::QuotedString(value),
                        remaining: None,
                    },
                    count,
                )),
                Some(b'\r') => {
                    validate_carriage_return_bytes(bytes)?;
                    Ok((
                        false,
                        ParsedLineSlice {
                            parsed: ParsedAttributeValue::QuotedString(value),
                            remaining: Some(str_from(bytes.as_slice())),
                        },
                        count,
                    ))
                }
                _ => Err("Unexpected char while reading value"),
            };
        }
        b'-' => still_parsing_integer = false,
        c if c.is_ascii_whitespace() => return Err("Unexpected whitespace in attribute value"),
        b',' => return Err("Unexpected comma in attribute value"),
        c if !c.is_ascii_digit() => {
            still_parsing_integer = false;
            still_parsing_float = false;
        }
        _ => (),
    }
    let mut is_remaining_none = false;
    let more_attributes_exist = loop {
        count += 1; // Before next to ensure if end of line we take whole value.
        let Some(char) = bytes.next() else {
            is_remaining_none = true;
            break false;
        };
        match char {
            b'.' => still_parsing_integer = false,
            b',' => break true,
            b'0'..=b'9' => (),
            b'\r' => {
                validate_carriage_return_bytes(bytes)?;
                break false;
            }
            b'\n' => {
                break false;
            }
            _ => {
                still_parsing_integer = false;
                still_parsing_float = false;
            }
        }
    };
    let number_value = if still_parsing_integer {
        input[value_start_index..(count - 1)]
            .parse::<u64>()
            .ok()
            .map(ParsedAttributeValue::DecimalInteger)
    } else if still_parsing_float {
        input[value_start_index..(count - 1)]
            .parse::<f64>()
            .ok()
            .map(ParsedAttributeValue::SignedDecimalFloatingPoint)
    } else {
        None
    };
    if let Some(number_value) = number_value {
        Ok((
            more_attributes_exist,
            ParsedLineSlice {
                parsed: number_value,
                remaining: if is_remaining_none {
                    None
                } else {
                    Some(str_from(bytes.as_slice()))
                },
            },
            count,
        ))
    } else {
        Ok((
            more_attributes_exist,
            ParsedLineSlice {
                parsed: ParsedAttributeValue::UnquotedString(
                    &input[value_start_index..(count - 1)],
                ),
                remaining: if is_remaining_none {
                    None
                } else {
                    Some(str_from(bytes.as_slice()))
                },
            },
            count,
        ))
    }
}
