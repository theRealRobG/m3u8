use memchr::{memchr, memchr2, memchr3};

use crate::{
    date::{self, DateTime},
    error::{
        DateTimeSyntaxError, ParseDecimalIntegerRangeError, ParseFloatError, ParseNumberError,
        ParsePlaylistTypeError, TagValueSyntaxError,
    },
    line::ParsedByteSlice,
    utils::{f64_to_u64, parse_u64, split_on_new_line},
};
use std::{collections::HashMap, fmt::Display};

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
pub enum SemiParsedTagValue<'a> {
    Empty,
    DecimalFloatingPointWithOptionalTitle(f64, &'a str),
    AttributeList(HashMap<&'a str, ParsedAttributeValue<'a>>),
    Unparsed(UnparsedTagValue<'a>),
}
#[derive(Debug, PartialEq)]
pub struct UnparsedTagValue<'a>(pub &'a [u8]);
impl<'a> UnparsedTagValue<'a> {
    pub fn try_as_hls_playlist_type(&self) -> Result<HlsPlaylistType, ParsePlaylistTypeError> {
        if self.0 == b"VOD" {
            Ok(HlsPlaylistType::Vod)
        } else if self.0 == b"EVENT" {
            Ok(HlsPlaylistType::Event)
        } else {
            Err(ParsePlaylistTypeError::InvalidValue)
        }
    }

    pub fn try_as_decimal_integer(&self) -> Result<u64, ParseNumberError> {
        parse_u64(self.0)
    }

    pub fn try_as_decimal_integer_range(
        &self,
    ) -> Result<(u64, Option<u64>), ParseDecimalIntegerRangeError> {
        match memchr(b'@', self.0) {
            Some(n) => {
                let length = parse_u64(&self.0[..n])
                    .map_err(ParseDecimalIntegerRangeError::InvalidLength)?;
                let offset = parse_u64(&self.0[(n + 1)..])
                    .map_err(ParseDecimalIntegerRangeError::InvalidOffset)?;
                Ok((length, Some(offset)))
            }
            None => parse_u64(self.0)
                .map(|length| (length, None))
                .map_err(ParseDecimalIntegerRangeError::InvalidLength),
        }
    }

    pub fn try_as_float(&self) -> Result<f64, ParseFloatError> {
        fast_float2::parse(self.0).map_err(|_| ParseFloatError)
    }

    pub fn try_as_date_time(&self) -> Result<DateTime, DateTimeSyntaxError> {
        date::parse_bytes(self.0)
    }
}

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

pub fn new_parse(input: &[u8]) -> Result<ParsedByteSlice<SemiParsedTagValue>, TagValueSyntaxError> {
    match memchr3(b'\n', b',', b'=', input) {
        Some(n) => {
            let needle = input[n];
            if needle == b'=' {
                let mut attribute_list = HashMap::new();
                let name = std::str::from_utf8(&input[..n])?;
                let (
                    ParsedByteSlice {
                        parsed,
                        mut remaining,
                    },
                    mut more,
                ) = parse_attribute_value(&input[(n + 1)..])?;
                attribute_list.insert(name, parsed);
                while more {
                    let Some(input) = remaining else {
                        return Err(
                            TagValueSyntaxError::UnexpectedEndOfLineWhileReadingAttributeName,
                        );
                    };
                    match memchr2(b'=', b'\n', input) {
                        Some(n) => {
                            if input[n] == b'=' {
                                let name = std::str::from_utf8(&input[..n])?;
                                let (
                                    ParsedByteSlice {
                                        parsed,
                                        remaining: new_remaining,
                                    },
                                    new_more,
                                ) = parse_attribute_value(&input[(n + 1)..])?;
                                attribute_list.insert(name, parsed);
                                remaining = new_remaining;
                                more = new_more;
                            } else {
                                return Err(
                                    TagValueSyntaxError::UnexpectedEndOfLineWhileReadingAttributeName,
                                );
                            }
                        }
                        None => {
                            return Err(
                                TagValueSyntaxError::UnexpectedEndOfLineWhileReadingAttributeName,
                            );
                        }
                    }
                }
                Ok(ParsedByteSlice {
                    parsed: SemiParsedTagValue::AttributeList(attribute_list),
                    remaining: remaining,
                })
            } else if needle == b',' {
                let duration = fast_float2::parse(&input[..n])?;
                let rest = split_on_new_line(&input[(n + 1)..]);
                let title = std::str::from_utf8(rest.parsed)?;
                Ok(ParsedByteSlice {
                    parsed: SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                        duration, title,
                    ),
                    remaining: rest.remaining,
                })
            } else if n > 0 && input[n - 1] == b'\r' {
                Ok(ParsedByteSlice {
                    parsed: SemiParsedTagValue::Unparsed(UnparsedTagValue(&input[..(n - 1)])),
                    remaining: Some(&input[(n + 1)..]),
                })
            } else {
                Ok(ParsedByteSlice {
                    parsed: SemiParsedTagValue::Unparsed(UnparsedTagValue(&input[..n])),
                    remaining: Some(&input[(n + 1)..]),
                })
            }
        }
        None => Ok(ParsedByteSlice {
            parsed: SemiParsedTagValue::Unparsed(UnparsedTagValue(input)),
            remaining: None,
        }),
    }
}

/// The Ok value is a tuple with `.0` being the parsed value and `.1` being whether there are more
/// attributes to parse or false if we have reached the end of the line.
fn parse_attribute_value(
    input: &[u8],
) -> Result<(ParsedByteSlice<ParsedAttributeValue>, bool), TagValueSyntaxError> {
    if input.is_empty() {
        return Err(TagValueSyntaxError::UnexpectedEmptyAttributeValue);
    }
    if input[0] == b'"' {
        let input = &input[1..];
        match memchr2(b'"', b'\n', input) {
            Some(n) => {
                if input[n] == b'"' {
                    let quoted_str = std::str::from_utf8(&input[..n])?;
                    match input.get(n + 1) {
                        Some(b',') => ok_quoted(input, quoted_str, Some(n + 2), true),
                        Some(b'\n') => ok_quoted(input, quoted_str, Some(n + 2), false),
                        Some(b'\r') => {
                            if input.get(n + 2) == Some(&b'\n') {
                                ok_quoted(input, quoted_str, Some(n + 3), false)
                            } else {
                                Err(TagValueSyntaxError::UnexpectedWhitespaceInAttributeValue)
                            }
                        }
                        None => ok_quoted(input, quoted_str, None, false),
                        Some(b) => Err(TagValueSyntaxError::UnexpectedCharacterAfterQuotedString(
                            *b,
                        )),
                    }
                } else {
                    Err(TagValueSyntaxError::UnexpectedEndOfLineWithinQuotedString)
                }
            }
            None => Err(TagValueSyntaxError::UnexpectedEndOfLineWithinQuotedString),
        }
    } else {
        match memchr3(b',', b'\n', b'.', input) {
            Some(n) => {
                if input[n] == b'.' {
                    match memchr2(b',', b'\n', &input[(n + 1)..]) {
                        Some(m) => {
                            if input[n + 1 + m] == b',' {
                                try_float(input, &input[..(n + 1 + m)], Some(n + 2 + m), true)
                            } else if input[n + m] == b'\r' {
                                try_float(input, &input[..(n + m)], Some(n + 2 + m), false)
                            } else {
                                try_float(input, &input[..(n + 1 + m)], Some(n + 2 + m), false)
                            }
                        }
                        None => try_float(input, input, None, false),
                    }
                } else if input[n] == b',' {
                    try_any(input, &input[..n], Some(n + 1), true)
                } else if n > 0 && input[n - 1] == b'\r' {
                    try_any(input, &input[..(n - 1)], Some(n + 1), false)
                } else {
                    try_any(input, &input[..n], Some(n + 1), false)
                }
            }
            None => try_any(input, input, None, false),
        }
    }
}

fn try_any<'a>(
    whole_input: &'a [u8],
    subrange: &'a [u8],
    remaining_start_index: Option<usize>,
    remaining: bool,
) -> Result<(ParsedByteSlice<'a, ParsedAttributeValue<'a>>, bool), TagValueSyntaxError> {
    if let Ok(number) = fast_float2::parse::<f64, &[u8]>(subrange) {
        if let Some(uint) = f64_to_u64(number) {
            ok_int(whole_input, uint, remaining_start_index, remaining)
        } else {
            ok_float(whole_input, number, remaining_start_index, remaining)
        }
    } else {
        let unquoted_str = std::str::from_utf8(subrange)?;
        ok_unquoted(whole_input, unquoted_str, remaining_start_index, remaining)
    }
}
fn try_float<'a>(
    whole_input: &'a [u8],
    float_input: &'a [u8],
    remaining_start_index: Option<usize>,
    remaining: bool,
) -> Result<(ParsedByteSlice<'a, ParsedAttributeValue<'a>>, bool), TagValueSyntaxError> {
    if let Ok(float) = fast_float2::parse(float_input) {
        ok_float(whole_input, float, remaining_start_index, remaining)
    } else {
        Err(TagValueSyntaxError::InvalidFloatInAttributeValue)
    }
}
fn ok_quoted<'a>(
    input: &'a [u8],
    quoted_str: &'a str,
    remaining_start_index: Option<usize>,
    remaining: bool,
) -> Result<(ParsedByteSlice<'a, ParsedAttributeValue<'a>>, bool), TagValueSyntaxError> {
    Ok((
        ParsedByteSlice {
            parsed: ParsedAttributeValue::QuotedString(quoted_str),
            remaining: remaining_start_index.map(|start| &input[start..]),
        },
        remaining,
    ))
}
fn ok_int<'a>(
    input: &'a [u8],
    int: u64,
    remaining_start_index: Option<usize>,
    remaining: bool,
) -> Result<(ParsedByteSlice<'a, ParsedAttributeValue<'a>>, bool), TagValueSyntaxError> {
    Ok((
        ParsedByteSlice {
            parsed: ParsedAttributeValue::DecimalInteger(int),
            remaining: remaining_start_index.map(|start| &input[start..]),
        },
        remaining,
    ))
}
fn ok_float<'a>(
    input: &'a [u8],
    float: f64,
    remaining_start_index: Option<usize>,
    remaining: bool,
) -> Result<(ParsedByteSlice<'a, ParsedAttributeValue<'a>>, bool), TagValueSyntaxError> {
    Ok((
        ParsedByteSlice {
            parsed: ParsedAttributeValue::SignedDecimalFloatingPoint(float),
            remaining: remaining_start_index.map(|start| &input[start..]),
        },
        remaining,
    ))
}
fn ok_unquoted<'a>(
    input: &'a [u8],
    unquoted_str: &'a str,
    remaining_start_index: Option<usize>,
    remaining: bool,
) -> Result<(ParsedByteSlice<'a, ParsedAttributeValue<'a>>, bool), TagValueSyntaxError> {
    Ok((
        ParsedByteSlice {
            parsed: ParsedAttributeValue::UnquotedString(unquoted_str),
            remaining: remaining_start_index.map(|start| &input[start..]),
        },
        remaining,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn type_enum() {
        test_str_and_with_crlf_and_with_lf("EVENT", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"EVENT"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("VOD", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"VOD"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_integer() {
        test_str_and_with_crlf_and_with_lf("42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_integer_range() {
        test_str_and_with_crlf_and_with_lf("42@42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42@42"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        // Positive tests
        test_str_and_with_crlf_and_with_lf("42.0", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42.0"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42.42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42.42"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42,", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.0, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42,=ATTRIBUTE-VALUE", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.0,
                    "=ATTRIBUTE-VALUE"
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        // Negative tests
        test_str_and_with_crlf_and_with_lf("-42.0", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"-42.0"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42.42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"-42.42"))),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42,", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    -42.0, ""
                )),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42,=ATTRIBUTE-VALUE", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
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
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(
                    b"2025-06-03T17:56:42.123Z"
                ))),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("2025-06-03T17:56:42.123+01:00", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(
                    b"2025-06-03T17:56:42.123+01:00"
                ))),
                new_parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("2025-06-03T17:56:42.123-05:00", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(
                    b"2025-06-03T17:56:42.123-05:00"
                ))),
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([(
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([(
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([(
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([(
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([(
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([(
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
                        Ok(SemiParsedTagValue::AttributeList(HashMap::from([
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
        F: Fn(&[u8]) -> (),
    {
        test(str.as_bytes());
        test(format!("{str}\r\n").as_bytes());
        test(format!("{str}\n").as_bytes());
    }
}
