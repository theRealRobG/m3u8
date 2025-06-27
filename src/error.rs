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
                "Carriage Return (U+000D) without following Line Feed (U+000A) is not supported"
            ),
            Self::UnexpectedEndOfLine => todo!(),
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
    UnexpectedEmptyInput,
    InvalidTag,
    Generic(GenericSyntaxError),
}
impl Display for UnknownTagSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEmptyInput => todo!(),
            Self::InvalidTag => todo!(),
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
impl Display for DateTimeSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidYear(parse_int_error) => todo!(),
            Self::UnexpectedYearToMonthSeparator(_) => todo!(),
            Self::InvalidMonth(parse_int_error) => todo!(),
            Self::UnexpectedMonthToDaySeparator(_) => todo!(),
            Self::InvalidDay(parse_int_error) => todo!(),
            Self::UnexpectedDayHourSeparator(_) => todo!(),
            Self::InvalidHour(parse_int_error) => todo!(),
            Self::UnexpectedHourMinuteSeparator(_) => todo!(),
            Self::InvalidMinute(parse_int_error) => todo!(),
            Self::UnexpectedMinuteSecondSeparator(_) => todo!(),
            Self::InvalidSecond(parse_float_error) => todo!(),
            Self::UnexpectedNoTimezone => todo!(),
            Self::UnexpectedCharactersAfterTimezone => todo!(),
            Self::InvalidTimezone => todo!(),
            Self::InvalidTimezoneHour(parse_int_error) => todo!(),
            Self::UnexpectedTimezoneHourMinuteSeparator(_) => todo!(),
            Self::InvalidTimezoneMinute(parse_int_error) => todo!(),
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
        todo!()
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
