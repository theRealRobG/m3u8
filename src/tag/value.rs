use crate::date::{DateTime, parse_date_time};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{self, take_till},
    combinator::opt,
};
use std::collections::HashMap;

// Not exactly the same as `tag-value`, because some of the types must be contextualized by the
// `tag-name`, but this list covers the possible raw values.
//
// Examples:
//   TypeEnum                              -> #EXT-X-PLAYLIST-TYPE:<type-enum>
//   DecimalInteger                        -> #EXT-X-VERSION:<n>
//   DecimalIntegerRange                   -> #EXT-X-BYTERANGE:<n>[@<o>]
//   DecimalFloatingPointWithOptionalTitle -> #EXTINF:<duration>,[<title>]
//   DateTimeMsec                          -> #EXT-X-PROGRAM-DATE-TIME:<date-time-msec>
//
#[derive(Debug, PartialEq)]
pub enum ParsedTagValue<'a> {
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

pub fn parse(input: &str) -> IResult<&str, ParsedTagValue> {
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
    match input.chars().nth(0) {
        Some('=') => todo!(),
        // Can only be a DecimalFloatingPointWithOptionalTitle
        Some(',') => handle_tag_value_comma(input, parsed),
        // Can only be a DecimalIntegerRange
        Some('@') => handle_tag_value_at_sign(input, parsed),
        // Could be DecimalInteger or DecimalFloatingPointWithOptionalTitle
        _ => handle_tag_value_end_of_line(parsed),
    }
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
}
