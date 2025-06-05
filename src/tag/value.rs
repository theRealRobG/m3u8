use crate::date::{DateTime, parse_date_time};
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

    /// Helper method to extract `SignedDecimalFloatingPoint` value.
    /// ```
    /// use m3u8::tag::value::ParsedAttributeValue;
    ///
    /// assert_eq!(
    ///     Some(42.0),
    ///     ParsedAttributeValue::SignedDecimalFloatingPoint(42.0).as_option_f64()
    /// );
    /// assert_eq!(None, ParsedAttributeValue::DecimalInteger(42).as_option_f64());
    /// ```
    pub fn as_option_f64(&self) -> Option<f64> {
        if let Self::SignedDecimalFloatingPoint(f) = self {
            Some(*f)
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
    if let (input, Some(parsed_date)) = opt(parse_date_time).parse(input)? {
        return Ok((input, ParsedTagValue::DateTimeMsec(parsed_date)));
    }
    let (input, parsed) = take_till(|c| ",-=@".contains(c)).parse(input)?;
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
