use crate::tag::value::SemiParsedTagValue;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    str::Utf8Error,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SyntaxError {
    Generic(GenericSyntaxError),
    UnknownTag(UnknownTagSyntaxError),
    DateTime(DateTimeSyntaxError),
    TagValue(TagValueSyntaxError),
    InvalidUtf8(Utf8Error),
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generic(e) => e.fmt(f),
            Self::UnknownTag(e) => e.fmt(f),
            Self::DateTime(e) => e.fmt(f),
            Self::TagValue(e) => e.fmt(f),
            Self::InvalidUtf8(e) => e.fmt(f),
        }
    }
}
impl Error for SyntaxError {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GenericSyntaxError {
    CarriageReturnWithoutLineFeed,
    UnexpectedEndOfLine,
    InvalidUtf8(Utf8Error),
}
impl Display for GenericSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CarriageReturnWithoutLineFeed => write!(
                f,
                "carriage return (U+000D) without a following line feed (U+000A) is not supported"
            ),
            Self::UnexpectedEndOfLine => write!(f, "line ended unexpectedly during parsing"),
            Self::InvalidUtf8(e) => write!(f, "invalid utf-8 due to {}", e),
        }
    }
}
impl Error for GenericSyntaxError {}
impl From<GenericSyntaxError> for SyntaxError {
    fn from(value: GenericSyntaxError) -> Self {
        Self::Generic(value)
    }
}
impl From<Utf8Error> for SyntaxError {
    fn from(value: Utf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnknownTagSyntaxError {
    UnexpectedNoTagName,
    InvalidTag,
    Generic(GenericSyntaxError),
}
impl Display for UnknownTagSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedNoTagName => write!(
                f,
                "tag (starting with '#EXT') had no name (no more characters until new line)"
            ),
            Self::InvalidTag => write!(
                f,
                "input did not start with '#EXT' and so is not a valid tag"
            ),
            Self::Generic(e) => e.fmt(f),
        }
    }
}
impl Error for UnknownTagSyntaxError {}
impl From<UnknownTagSyntaxError> for SyntaxError {
    fn from(value: UnknownTagSyntaxError) -> Self {
        Self::UnknownTag(value)
    }
}
impl From<GenericSyntaxError> for UnknownTagSyntaxError {
    fn from(value: GenericSyntaxError) -> Self {
        Self::Generic(value)
    }
}
impl From<Utf8Error> for UnknownTagSyntaxError {
    fn from(value: Utf8Error) -> Self {
        Self::Generic(GenericSyntaxError::InvalidUtf8(value))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DateTimeSyntaxError {
    InvalidYear(ParseNumberError),
    UnexpectedYearToMonthSeparator(Option<u8>),
    InvalidMonth(ParseNumberError),
    UnexpectedMonthToDaySeparator(Option<u8>),
    InvalidDay(ParseNumberError),
    UnexpectedDayHourSeparator(Option<u8>),
    InvalidHour(ParseNumberError),
    UnexpectedHourMinuteSeparator(Option<u8>),
    InvalidMinute(ParseNumberError),
    UnexpectedMinuteSecondSeparator(Option<u8>),
    InvalidSecond,
    UnexpectedNoTimezone,
    UnexpectedCharactersAfterTimezone,
    InvalidTimezone,
    InvalidTimezoneHour(ParseNumberError),
    UnexpectedTimezoneHourMinuteSeparator(Option<u8>),
    InvalidTimezoneMinute(ParseNumberError),
    Generic(GenericSyntaxError),
}
fn option_u8_to_string(u: &Option<u8>) -> String {
    u.map(|b| format!("{}", b as char))
        .unwrap_or("None".to_string())
}
impl Display for DateTimeSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidYear(e) => write!(f, "invalid integer for year in date due to {}", e),
            Self::UnexpectedYearToMonthSeparator(s) => write!(
                f,
                "expected '-' between year and month but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidMonth(e) => write!(f, "invalid integer for month in date due to {}", e),
            Self::UnexpectedMonthToDaySeparator(s) => write!(
                f,
                "expected '-' between month and day but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidDay(e) => write!(f, "invalid integer for day in date due to {}", e),
            Self::UnexpectedDayHourSeparator(s) => write!(
                f,
                "expected 'T' or 't' between day and hour but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidHour(e) => write!(f, "invalid integer for hour in date due to {}", e),
            Self::UnexpectedHourMinuteSeparator(s) => write!(
                f,
                "expected ':' between hour and minute but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidMinute(e) => write!(f, "invalid integer for minute in date due to {}", e),
            Self::UnexpectedMinuteSecondSeparator(s) => write!(
                f,
                "expected ':' between minute and second but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidSecond => write!(f, "invalid float for second in date"),
            Self::UnexpectedNoTimezone => write!(
                f,
                "no timezone in date (expect either 'Z' or full timezone)"
            ),
            Self::UnexpectedCharactersAfterTimezone => {
                write!(f, "unexpected characters after timezone in date")
            }
            Self::InvalidTimezone => write!(f, "timezone invalid in date"),
            Self::InvalidTimezoneHour(e) => {
                write!(f, "invalid integer for hour in timezone due to {}", e)
            }
            Self::UnexpectedTimezoneHourMinuteSeparator(s) => write!(
                f,
                "expected ':' between hour and minute in timezone but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidTimezoneMinute(e) => {
                write!(f, "invalid integer for minute in timezone due to {}", e)
            }
            Self::Generic(e) => e.fmt(f),
        }
    }
}
impl Error for DateTimeSyntaxError {}
impl From<DateTimeSyntaxError> for SyntaxError {
    fn from(value: DateTimeSyntaxError) -> Self {
        Self::DateTime(value)
    }
}
impl From<GenericSyntaxError> for DateTimeSyntaxError {
    fn from(value: GenericSyntaxError) -> Self {
        Self::Generic(value)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TagValueSyntaxError {
    UnexpectedCharacter(u8),
    InvalidFloatForDecimalFloatingPointValue,
    InvalidUtf8(Utf8Error),
    InvalidDecimalInteger(ParseNumberError),
    UnexpectedEndOfLineWhileReadingAttributeName,
    UnexpectedCharacterInAttributeName(u8),
    UnexpectedEmptyAttributeValue,
    UnexpectedEndOfLineWithinQuotedString,
    UnexpectedCharacterAfterQuotedString(u8),
    UnexpectedWhitespaceInAttributeValue,
    InvalidIntegerInAttributeValue(ParseNumberError),
    InvalidFloatInAttributeValue,
    Generic(GenericSyntaxError),
}
impl Display for TagValueSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCharacter(c) => {
                write!(f, "unexpected character '{}'", *c as char)
            }
            Self::InvalidFloatForDecimalFloatingPointValue => {
                write!(f, "invalid float for decimal float value")
            }
            Self::InvalidUtf8(e) => write!(f, "invalid utf-8 due to {}", e),
            Self::InvalidDecimalInteger(e) => {
                write!(f, "invalid integer for decimal integer value due to {}", e)
            }
            Self::UnexpectedEndOfLineWhileReadingAttributeName => {
                write!(f, "unexpected end of line reading attribute name")
            }
            Self::UnexpectedCharacterInAttributeName(c) => {
                write!(f, "unexpected character '{}' in attribute name", *c as char)
            }
            Self::UnexpectedEmptyAttributeValue => {
                write!(f, "attribute name had no value")
            }
            Self::UnexpectedEndOfLineWithinQuotedString => write!(
                f,
                "unexpected end of line within quoted string attribute value"
            ),
            Self::UnexpectedCharacterAfterQuotedString(c) => write!(
                f,
                "unexpected character '{}' after end of quoted attribute value (only ',' is valid)",
                *c as char
            ),
            Self::UnexpectedWhitespaceInAttributeValue => {
                write!(f, "unexpected whitespace in attribute value")
            }
            Self::InvalidIntegerInAttributeValue(e) => {
                write!(f, "invalid integer for attribute value due to {}", e)
            }
            Self::InvalidFloatInAttributeValue => {
                write!(f, "invalid float in attribute value")
            }
            Self::Generic(e) => e.fmt(f),
        }
    }
}
impl Error for TagValueSyntaxError {}
impl From<TagValueSyntaxError> for SyntaxError {
    fn from(value: TagValueSyntaxError) -> Self {
        Self::TagValue(value)
    }
}
impl From<GenericSyntaxError> for TagValueSyntaxError {
    fn from(value: GenericSyntaxError) -> Self {
        Self::Generic(value)
    }
}
impl From<fast_float2::Error> for TagValueSyntaxError {
    fn from(_: fast_float2::Error) -> Self {
        Self::InvalidFloatForDecimalFloatingPointValue
    }
}
impl From<ParseFloatError> for TagValueSyntaxError {
    fn from(_: ParseFloatError) -> Self {
        Self::InvalidFloatForDecimalFloatingPointValue
    }
}
impl From<Utf8Error> for TagValueSyntaxError {
    fn from(value: Utf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValidationError {
    UnexpectedTagName,
    UnexpectedValueType(ValidationErrorValueKind),
    MissingRequiredAttribute(&'static str),
    NotImplemented,
    InvalidPlaylistType(ParsePlaylistTypeError),
    InvalidDecimalInteger(ParseNumberError),
    InvalidDecimalIntegerRange(ParseDecimalIntegerRangeError),
    InvalidDateTime(DateTimeSyntaxError),
    InvalidFloat(ParseFloatError),
}
impl From<ParsePlaylistTypeError> for ValidationError {
    fn from(error: ParsePlaylistTypeError) -> Self {
        Self::InvalidPlaylistType(error)
    }
}
impl From<ParseNumberError> for ValidationError {
    fn from(error: ParseNumberError) -> Self {
        Self::InvalidDecimalInteger(error)
    }
}
impl From<ParseDecimalIntegerRangeError> for ValidationError {
    fn from(error: ParseDecimalIntegerRangeError) -> Self {
        Self::InvalidDecimalIntegerRange(error)
    }
}
impl From<DateTimeSyntaxError> for ValidationError {
    fn from(error: DateTimeSyntaxError) -> Self {
        Self::InvalidDateTime(error)
    }
}
impl From<ParseFloatError> for ValidationError {
    fn from(error: ParseFloatError) -> Self {
        Self::InvalidFloat(error)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValidationErrorValueKind {
    Empty,
    DecimalFloatingPointWithOptionalTitle,
    AttributeList,
    Unparsed,
}
impl From<&SemiParsedTagValue<'_>> for ValidationErrorValueKind {
    fn from(value: &SemiParsedTagValue<'_>) -> Self {
        match value {
            SemiParsedTagValue::Empty => Self::Empty,
            SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(_, _) => {
                Self::DecimalFloatingPointWithOptionalTitle
            }
            SemiParsedTagValue::AttributeList(_) => Self::AttributeList,
            SemiParsedTagValue::Unparsed(_) => Self::Unparsed,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseNumberError {
    InvalidDigit(u8),
    NumberTooBig,
    Empty,
}
impl Display for ParseNumberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDigit(got) => write!(f, "invalid digit {}", got),
            Self::NumberTooBig => write!(f, "number is too big"),
            Self::Empty => write!(f, "cannot parse number from empty slice"),
        }
    }
}
impl Error for ParseNumberError {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParsePlaylistTypeError {
    InvalidValue,
}
impl Display for ParsePlaylistTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidValue => write!(f, "expected 'EVENT' or 'VOD'"),
        }
    }
}
impl Error for ParsePlaylistTypeError {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseDecimalIntegerRangeError {
    InvalidLength(ParseNumberError),
    InvalidOffset(ParseNumberError),
}
impl Display for ParseDecimalIntegerRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength(e) => write!(f, "invalid length due to {}", e),
            Self::InvalidOffset(e) => write!(f, "invalid offset due to {}", e),
        }
    }
}
impl Error for ParseDecimalIntegerRangeError {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ParseFloatError;
impl Display for ParseFloatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid float")
    }
}
impl Error for ParseFloatError {}
