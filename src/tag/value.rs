use crate::{
    date::DateTime,
    error::{GenericSyntaxError, TagValueSyntaxError},
    line::ParsedLineSlice,
    utils::{
        parse_date_time_bytes, str_from, take_until_end_of_bytes, validate_carriage_return_bytes,
    },
};
use std::{collections::HashMap, fmt::Display, slice::Iter};

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

// pub struct ExampleSemiParsedData<'a> {
//     pub name: &'a str,
//     pub value: ExampleSemiParsedDataValue<'a>,
//     pub(crate) original_source_data: &'a [u8],
// }

// pub enum ExampleSemiParsedDataValue<'a> {
//     ValueWeNeed(HashMap<&'a str, &'a str>),
//     OtherValue,
// }

// pub struct ExampleParsedData<'a> {
//     required_prop: &'a str,
//     source_value: HashMap<&'a str, &'a str>,
// }
// impl<'a> ExampleParsedData<'a> {
//     pub fn required_prop(&self) -> &str {
//         self.required_prop
//     }

//     pub fn optional_prop(&self) -> Option<&str> {
//         self.source_value.get("optional_prop").map(|v| *v)
//     }
// }

// pub struct ValidationError<'a> {
//     pub semi_parsed_data: ExampleSemiParsedData<'a>,
// }

// pub enum ValidationErrorKind {
//     UnexpectedValueType,
//     MissingRequiredAttribute(&'static str),
// }

// impl<'a> TryFrom<ExampleSemiParsedData<'a>> for ExampleParsedData<'a> {
//     type Error = &'static str;

//     fn try_from(value: ExampleSemiParsedData<'a>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HlsPlaylistType {
    Event,
    Vod,
}

// Not exactly the same as `attribute-value`, because some of the types must be contextualized by
// the `attribute-name`, but this list covers the possible raw values.
#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DecimalResolution {
    pub width: u64,
    pub height: u64,
}

impl Display for DecimalResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

pub fn new_parse(input: &str) -> Result<ParsedLineSlice<ParsedTagValue>, TagValueSyntaxError> {
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
                    return Err(TagValueSyntaxError::UnexpectedCharacter(b'.'));
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
                    return Err(TagValueSyntaxError::UnexpectedCharacter(b','));
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_float = true;
                break Some(byte);
            }
            b'@' => {
                if !parsing_range {
                    return Err(TagValueSyntaxError::UnexpectedCharacter(b'@'));
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
                    return Err(TagValueSyntaxError::UnexpectedCharacter(*byte));
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_float = false;
                break Some(byte);
            }
            b'A'..=b'Z' => {
                if !parsing_attr_name {
                    return Err(TagValueSyntaxError::UnexpectedCharacter(*byte));
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
                    return Err(TagValueSyntaxError::UnexpectedCharacter(b'='));
                }
                parsing_empty = false;
                parsing_enum = false;
                parsing_int = false;
                parsing_float = false;
                break Some(byte);
            }
            _ => return Err(TagValueSyntaxError::UnexpectedCharacter(*byte)),
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
                .map_err(|e| TagValueSyntaxError::InvalidFloatForDecimalFloatingPointValue(e))?;
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
                .map_err(|e| TagValueSyntaxError::InvalidLengthForDecimalIntegerRange(e))?;
            let offset = rest
                .parsed
                .parse::<u64>()
                .map_err(|e| TagValueSyntaxError::InvalidOffsetForDecimalIntegerRange(e))?;
            return Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DecimalIntegerRange(length, offset),
                remaining: rest.remaining,
            });
        }
        Some(b't') | Some(b':') => {
            let ParsedLineSlice {
                parsed: date_time,
                remaining,
            } = parse_date_time_bytes(input, bytes, *break_byte.unwrap())?;
            return Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DateTimeMsec(date_time),
                remaining,
            });
        }
        Some(b'=') => {
            let mut attribute_list = HashMap::new();
            let first_name = &input[..(count - 1)];
            let (mut has_more, first_parsed_attribute_slice) = handle_attribute_value(&mut bytes)?;
            attribute_list.insert(first_name, first_parsed_attribute_slice.parsed);
            let mut remaining = first_parsed_attribute_slice.remaining;
            loop {
                if has_more {
                    let name = handle_attribute_name(&mut bytes)?;
                    let (more, value_slice) = handle_attribute_value(&mut bytes)?;
                    attribute_list.insert(name, value_slice.parsed);
                    remaining = value_slice.remaining;
                    has_more = more;
                } else {
                    return Ok(ParsedLineSlice {
                        parsed: ParsedTagValue::AttributeList(attribute_list),
                        remaining,
                    });
                }
            }
        }
        Some(break_byte) => return Err(TagValueSyntaxError::UnexpectedCharacter(*break_byte)),
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
            .map_err(|e| TagValueSyntaxError::InvalidDecimalInteger(e))?;
        Ok(ParsedLineSlice {
            parsed: ParsedTagValue::DecimalInteger(n),
            remaining,
        })
    } else if parsing_float {
        let n = input[..end_index]
            .parse::<f64>()
            .map_err(|e| TagValueSyntaxError::InvalidFloatForDecimalFloatingPointValue(e))?;
        Ok(ParsedLineSlice {
            parsed: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(n, ""),
            remaining,
        })
    } else if parsing_enum {
        match end_index {
            3 => {
                let value = &input[..3];
                if value == "VOD" {
                    Ok(ParsedLineSlice {
                        parsed: ParsedTagValue::TypeEnum(HlsPlaylistType::Vod),
                        remaining,
                    })
                } else {
                    Err(TagValueSyntaxError::InvalidTypeEnumValue(value.to_string()))
                }
            }
            5 => {
                let value = &input[..5];
                if value == "EVENT" {
                    Ok(ParsedLineSlice {
                        parsed: ParsedTagValue::TypeEnum(HlsPlaylistType::Event),
                        remaining,
                    })
                } else {
                    Err(TagValueSyntaxError::InvalidTypeEnumValue(value.to_string()))
                }
            }
            _ => Err(TagValueSyntaxError::InvalidTypeEnumValue(
                input[..end_index].to_string(),
            )),
        }
    } else {
        Err(GenericSyntaxError::UnexpectedEndOfLine)?
    }
}

fn handle_attribute_name<'a>(bytes: &mut Iter<'a, u8>) -> Result<&'a str, TagValueSyntaxError> {
    let input = str_from(bytes.as_slice());
    let mut index = 0usize;
    loop {
        let Some(char) = bytes.next() else {
            return Err(TagValueSyntaxError::UnexpectedEndOfLineWhileReadingAttributeName);
        };
        match char {
            b'=' => break,
            b'A'..=b'Z' | b'0'..=b'9' | b'-' => (),
            _ => {
                return Err(TagValueSyntaxError::UnexpectedCharacterInAttributeName(
                    *char,
                ));
            }
        }
        index += 1;
    }
    Ok(&input[..index])
}

fn handle_attribute_value<'a>(
    bytes: &mut Iter<'a, u8>,
) -> Result<(bool, ParsedLineSlice<'a, ParsedAttributeValue<'a>>), TagValueSyntaxError> {
    let input = str_from(bytes.as_slice());
    match bytes.next() {
        Some(b'"') => handle_quoted_string_attribute_value(input, bytes),
        None | Some(b'\n') | Some(b'\r') => Err(TagValueSyntaxError::UnexpectedEmptyAttributeValue),
        Some(byte) => handle_not_quoted_string_attribute_value(input, bytes, byte),
    }
}

fn handle_quoted_string_attribute_value<'a>(
    input: &'a str,
    bytes: &mut Iter<'a, u8>,
) -> Result<(bool, ParsedLineSlice<'a, ParsedAttributeValue<'a>>), TagValueSyntaxError> {
    let mut index = 0usize;
    loop {
        index += 1;
        match bytes.next() {
            Some(b'"') => break,
            None | Some(b'\n') | Some(b'\r') => {
                return Err(TagValueSyntaxError::UnexpectedEndOfLineWithinQuotedString);
            }
            _ => (),
        }
    }
    let str = &input[1..index];
    let (has_more, remaining) = match bytes.next() {
        None => (false, None),
        Some(b',') => (true, Some(str_from(bytes.as_slice()))),
        Some(b'\n') => (false, Some(str_from(bytes.as_slice()))),
        Some(b'\r') => {
            validate_carriage_return_bytes(bytes)?;
            (false, Some(str_from(bytes.as_slice())))
        }
        Some(b) => {
            return Err(TagValueSyntaxError::UnexpectedCharacterAfterQuotedString(
                *b,
            ));
        }
    };
    Ok((
        has_more,
        ParsedLineSlice {
            parsed: ParsedAttributeValue::QuotedString(str),
            remaining,
        },
    ))
}

fn handle_not_quoted_string_attribute_value<'a>(
    input: &'a str,
    bytes: &mut Iter<'a, u8>,
    first_byte: &u8,
) -> Result<(bool, ParsedLineSlice<'a, ParsedAttributeValue<'a>>), TagValueSyntaxError> {
    // ParsedAttributeValue {
    //     DecimalInteger             - b'0'..=b'9'
    //     SignedDecimalFloatingPoint - b'0'..=b'9' | b'-' | b'.'
    //     UnquotedString             - b'0'..=b'9' | b'-' | b'.' | ...
    // }
    let mut index = 0usize;
    let (mut parsing_int, mut parsing_float) = match first_byte {
        b'0'..=b'9' => (true, true),
        b'-' => (false, true),
        _ => (false, false),
    };
    let break_byte = loop {
        let Some(byte) = bytes.next() else {
            break None;
        };
        index += 1;
        match byte {
            b'0'..=b'9' => (),
            b'-' | b'.' => parsing_int = false,
            b',' | b'\n' | b'\r' => break Some(byte),
            b if b.is_ascii_whitespace() => {
                return Err(TagValueSyntaxError::UnexpectedWhitespaceInAttributeValue);
            }
            _ => {
                parsing_int = false;
                parsing_float = false;
            }
        }
    };
    let (has_more, remaining, index) = match break_byte {
        None => (false, None, index + 1),
        Some(b',') => (true, Some(str_from(bytes.as_slice())), index),
        Some(b'\n') => (false, Some(str_from(bytes.as_slice())), index),
        Some(b'\r') => {
            validate_carriage_return_bytes(bytes)?;
            (false, Some(str_from(bytes.as_slice())), index)
        }
        _ => panic!("Should be impossible since we only break on None, CR, or LF"),
    };
    if parsing_int {
        let n = input[..index]
            .parse::<u64>()
            .map_err(|e| TagValueSyntaxError::InvalidIntegerInAttributeValue(e))?;
        Ok((
            has_more,
            ParsedLineSlice {
                parsed: ParsedAttributeValue::DecimalInteger(n),
                remaining,
            },
        ))
    } else if parsing_float {
        let n = input[..index]
            .parse::<f64>()
            .map_err(|e| TagValueSyntaxError::InvalidFloatInAttributeValue(e))?;
        Ok((
            has_more,
            ParsedLineSlice {
                parsed: ParsedAttributeValue::SignedDecimalFloatingPoint(n),
                remaining,
            },
        ))
    } else {
        let str = &input[..index];
        Ok((
            has_more,
            ParsedLineSlice {
                parsed: ParsedAttributeValue::UnquotedString(str),
                remaining,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::DateTimeTimezoneOffset;
    use pretty_assertions::assert_eq;

    #[test]
    fn type_enum() {
        test_str_and_with_crlf_and_with_lf("EVENT", |str| {
            assert_eq!(
                Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Event)),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("VOD", |str| {
            assert_eq!(
                Ok(ParsedTagValue::TypeEnum(HlsPlaylistType::Vod)),
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_integer() {
        test_str_and_with_crlf_and_with_lf("42", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalInteger(42)),
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_integer_range() {
        test_str_and_with_crlf_and_with_lf("42@42", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalIntegerRange(42, 42)),
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        // Positive tests
        test_str_and_with_crlf_and_with_lf("42.0", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.0, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42.42", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.42, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42,", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.0, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42,=ATTRIBUTE-VALUE", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.0,
                    "=ATTRIBUTE-VALUE"
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        // Negative tests
        test_str_and_with_crlf_and_with_lf("-42.0", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    -42.0, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42.42", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    -42.42, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42,", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    -42.0, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42,=ATTRIBUTE-VALUE", |str| {
            assert_eq!(
                Ok(ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    -42.0,
                    "=ATTRIBUTE-VALUE"
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn date_time_msec() {
        test_str_and_with_crlf_and_with_lf("2025-06-03T17:56:42.123Z", |str| {
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
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("2025-06-03T17:56:42.123+01:00", |str| {
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
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("2025-06-03T17:56:42.123-05:00", |str| {
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
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    mod attribute_list {
        use super::*;

        mod decimal_integer {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                test_str_and_with_crlf_and_with_lf("NAME=123", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([(
                            "NAME",
                            ParsedAttributeValue::DecimalInteger(123)
                        )]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn multi_attributes() {
                test_str_and_with_crlf_and_with_lf("NAME=123,NEXT-NAME=456", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([
                            ("NAME", ParsedAttributeValue::DecimalInteger(123)),
                            ("NEXT-NAME", ParsedAttributeValue::DecimalInteger(456))
                        ]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }
        }

        mod signed_decimal_floating_point {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn positive_float_single_attribute() {
                test_str_and_with_crlf_and_with_lf("NAME=42.42", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([(
                            "NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(42.42)
                        )]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn negative_integer_single_attribute() {
                test_str_and_with_crlf_and_with_lf("NAME=-42", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([(
                            "NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(-42.0)
                        )]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn negative_float_single_attribute() {
                test_str_and_with_crlf_and_with_lf("NAME=-42.42", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([(
                            "NAME",
                            ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42)
                        )]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn positive_float_multi_attributes() {
                test_str_and_with_crlf_and_with_lf("NAME=42.42,NEXT-NAME=84.84", |str| {
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
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn negative_integer_multi_attributes() {
                test_str_and_with_crlf_and_with_lf("NAME=-42,NEXT-NAME=-42", |str| {
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
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn negative_float_multi_attributes() {
                test_str_and_with_crlf_and_with_lf("NAME=-42.42,NEXT-NAME=-84.84", |str| {
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
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }
        }

        mod quoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                test_str_and_with_crlf_and_with_lf("NAME=\"Hello, World!\"", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([(
                            "NAME",
                            ParsedAttributeValue::QuotedString("Hello, World!")
                        )]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn multi_attributes() {
                test_str_and_with_crlf_and_with_lf("NAME=\"Hello,\",NEXT-NAME=\"World!\"", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([
                            ("NAME", ParsedAttributeValue::QuotedString("Hello,")),
                            ("NEXT-NAME", ParsedAttributeValue::QuotedString("World!"))
                        ]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }
        }

        mod unquoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                test_str_and_with_crlf_and_with_lf("NAME=PQ", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([(
                            "NAME",
                            ParsedAttributeValue::UnquotedString("PQ")
                        )]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }

            #[test]
            fn multi_attributes() {
                test_str_and_with_crlf_and_with_lf("NAME=PQ,NEXT-NAME=HLG", |str| {
                    assert_eq!(
                        Ok(ParsedTagValue::AttributeList(HashMap::from([
                            ("NAME", ParsedAttributeValue::UnquotedString("PQ")),
                            ("NEXT-NAME", ParsedAttributeValue::UnquotedString("HLG"))
                        ]))),
                        new_parse(str).map(|p| p.parsed)
                    );
                });
            }
        }
    }

    fn test_str_and_with_crlf_and_with_lf<F>(str: &'static str, test: F)
    where
        F: Fn(&str) -> (),
    {
        test(str);
        test(format!("{str}\r\n").as_str());
        test(format!("{str}\n").as_str());
    }
}
