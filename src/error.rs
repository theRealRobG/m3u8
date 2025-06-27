use std::{
    error::Error,
    fmt::{Display, Formatter},
    num::{ParseFloatError, ParseIntError},
};

use crate::tag::value::ParsedTagValue;

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    Generic(GenericSyntaxError),
    UnknownTag(UnknownTagSyntaxError),
    DateTime(DateTimeSyntaxError),
    TagValue(TagValueSyntaxError),
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generic(e) => e.fmt(f),
            Self::UnknownTag(e) => e.fmt(f),
            Self::DateTime(e) => e.fmt(f),
            Self::TagValue(e) => e.fmt(f),
        }
    }
}
impl Error for SyntaxError {}

#[derive(Debug, PartialEq)]
pub enum GenericSyntaxError {
    CarriageReturnWithoutLineFeed,
    UnexpectedEndOfLine,
}
impl Display for GenericSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CarriageReturnWithoutLineFeed => write!(
                f,
                "carriage return (U+000D) without a following line feed (U+000A) is not supported"
            ),
            Self::UnexpectedEndOfLine => write!(f, "line ended unexpectedly during parsing"),
        }
    }
}
impl Error for GenericSyntaxError {}
impl From<GenericSyntaxError> for SyntaxError {
    fn from(value: GenericSyntaxError) -> Self {
        Self::Generic(value)
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum DateTimeSyntaxError {
    InvalidYear(ParseIntError),
    UnexpectedYearToMonthSeparator(Option<u8>),
    InvalidMonth(ParseIntError),
    UnexpectedMonthToDaySeparator(Option<u8>),
    InvalidDay(ParseIntError),
    UnexpectedDayHourSeparator(Option<u8>),
    InvalidHour(ParseIntError),
    UnexpectedHourMinuteSeparator(Option<u8>),
    InvalidMinute(ParseIntError),
    UnexpectedMinuteSecondSeparator(Option<u8>),
    InvalidSecond(ParseFloatError),
    UnexpectedNoTimezone,
    UnexpectedCharactersAfterTimezone,
    InvalidTimezone,
    InvalidTimezoneHour(ParseIntError),
    UnexpectedTimezoneHourMinuteSeparator(Option<u8>),
    InvalidTimezoneMinute(ParseIntError),
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
            Self::InvalidSecond(e) => write!(f, "invalid float for second in date due to {}", e),
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

#[derive(Debug, PartialEq)]
pub enum TagValueSyntaxError {
    UnexpectedCharacter(u8),
    InvalidFloatForDecimalFloatingPointValue(ParseFloatError),
    InvalidLengthForDecimalIntegerRange(ParseIntError),
    InvalidOffsetForDecimalIntegerRange(ParseIntError),
    InvalidDecimalInteger(ParseIntError),
    InvalidTypeEnumValue(String),
    UnexpectedEndOfLineWhileReadingAttributeName,
    UnexpectedCharacterInAttributeName(u8),
    UnexpectedEmptyAttributeValue,
    UnexpectedEndOfLineWithinQuotedString,
    UnexpectedCharacterAfterQuotedString(u8),
    UnexpectedWhitespaceInAttributeValue,
    InvalidIntegerInAttributeValue(ParseIntError),
    InvalidFloatInAttributeValue(ParseFloatError),
    Generic(GenericSyntaxError),
    DateTime(DateTimeSyntaxError),
}
impl Display for TagValueSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValueSyntaxError::UnexpectedCharacter(c) => {
                write!(f, "unexpected character '{}'", *c as char)
            }
            TagValueSyntaxError::InvalidFloatForDecimalFloatingPointValue(e) => {
                write!(f, "invalid float for decimal float value due to {}", e)
            }
            TagValueSyntaxError::InvalidLengthForDecimalIntegerRange(e) => {
                write!(f, "invalid length for decimal integer range due to {}", e)
            }
            TagValueSyntaxError::InvalidOffsetForDecimalIntegerRange(e) => {
                write!(f, "invalid offset for decimal integer range due to {}", e)
            }
            TagValueSyntaxError::InvalidDecimalInteger(e) => {
                write!(f, "invalid integer for decimal integer value due to {}", e)
            }
            TagValueSyntaxError::InvalidTypeEnumValue(v) => {
                write!(f, "invalid playlist type enum value '{}'", v)
            }
            TagValueSyntaxError::UnexpectedEndOfLineWhileReadingAttributeName => {
                write!(f, "unexpected end of line reading attribute name")
            }
            TagValueSyntaxError::UnexpectedCharacterInAttributeName(c) => {
                write!(f, "unexpected character '{}' in attribute name", *c as char)
            }
            TagValueSyntaxError::UnexpectedEmptyAttributeValue => {
                write!(f, "attribute name had no value")
            }
            TagValueSyntaxError::UnexpectedEndOfLineWithinQuotedString => write!(
                f,
                "unexpected end of line within quoted string attribute value"
            ),
            TagValueSyntaxError::UnexpectedCharacterAfterQuotedString(c) => write!(
                f,
                "unexpected character '{}' after end of quoted attribute value (only ',' is valid)",
                *c as char
            ),
            TagValueSyntaxError::UnexpectedWhitespaceInAttributeValue => {
                write!(f, "unexpected whitespace in attribute value")
            }
            TagValueSyntaxError::InvalidIntegerInAttributeValue(e) => {
                write!(f, "invalid integer for attribute value due to {}", e)
            }
            TagValueSyntaxError::InvalidFloatInAttributeValue(e) => {
                write!(f, "invalid float in attribute value due to {}", e)
            }
            TagValueSyntaxError::Generic(e) => e.fmt(f),
            TagValueSyntaxError::DateTime(e) => e.fmt(f),
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
impl From<DateTimeSyntaxError> for TagValueSyntaxError {
    fn from(value: DateTimeSyntaxError) -> Self {
        Self::DateTime(value)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValidationError {
    UnexpectedTagName,
    UnexpectedValueType(ValidationErrorValueKind),
    MissingRequiredAttribute(&'static str),
    NotImplemented,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValidationErrorValueKind {
    Empty,
    TypeEnum,
    DecimalInteger,
    DecimalIntegerRange,
    DecimalFloatingPointWithOptionalTitle,
    DateTimeMsec,
    AttributeList,
}
impl From<&ParsedTagValue<'_>> for ValidationErrorValueKind {
    fn from(value: &ParsedTagValue<'_>) -> Self {
        match value {
            ParsedTagValue::Empty => Self::Empty,
            ParsedTagValue::TypeEnum(_) => Self::TypeEnum,
            ParsedTagValue::DecimalInteger(_) => Self::DecimalInteger,
            ParsedTagValue::DecimalIntegerRange(_, _) => Self::DecimalIntegerRange,
            ParsedTagValue::DecimalFloatingPointWithOptionalTitle(_, _) => {
                Self::DecimalFloatingPointWithOptionalTitle
            }
            ParsedTagValue::DateTimeMsec(_) => Self::DateTimeMsec,
            ParsedTagValue::AttributeList(_) => Self::AttributeList,
        }
    }
}
