//! All error types exposed by the library.
//!
//! The module offers a collection of many error types coming from various operations.

use crate::{
    line::{ParsedByteSlice, ParsedLineSlice},
    tag::value::SemiParsedTagValue,
};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::Utf8Error,
};

/// Error in reading a line from a [`crate::Reader`] constructed with [`crate::Reader::from_str`].
#[derive(Debug, PartialEq, Clone)]
pub struct ReaderStrError<'a> {
    /// The original line that caused the error.
    ///
    /// The `Reader` exposes this to the user so that it can continue to the next line when
    /// [`crate::Reader::read_line`] is called again.
    pub errored_line: &'a str,
    /// The underlying error that was experienced.
    pub error: SyntaxError,
}

/// Error in reading a line from a [`crate::Reader`] constructed with [`crate::Reader::from_bytes`].
#[derive(Debug, PartialEq, Clone)]
pub struct ReaderBytesError<'a> {
    /// The original line that caused the error.
    ///
    /// The `Reader` exposes this to the user so that it can continue to the next line when
    /// [`crate::Reader::read_line`] is called again.
    pub errored_line: &'a [u8],
    /// The underlying error that was experienced.
    pub error: SyntaxError,
}

/// Error in reading a line from [`crate::line::parse`] (or [`crate::line::parse_with_custom`]).
#[derive(Debug, PartialEq, Clone)]
pub struct ParseLineStrError<'a> {
    /// The original line that caused the error along with the remaining slice after the line.
    pub errored_line_slice: ParsedLineSlice<'a, &'a str>,
    /// The underlying error that was experienced.
    pub error: SyntaxError,
}

/// Error in reading a line from [`crate::line::parse_bytes`] (or
/// [`crate::line::parse_bytes_with_custom`]).
#[derive(Debug, PartialEq, Clone)]
pub struct ParseLineBytesError<'a> {
    /// The original line that caused the error along with the remaining bytes after the line.
    pub errored_line_slice: ParsedByteSlice<'a, &'a [u8]>,
    /// The underlying error that was experienced.
    pub error: SyntaxError,
}

macro_rules! impl_error {
    ($type:ident) => {
        impl Display for $type<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.error.fmt(f)
            }
        }
        impl Error for $type<'_> {}
    };
}
impl_error!(ReaderStrError);
impl_error!(ReaderBytesError);
impl_error!(ParseLineStrError);
impl_error!(ParseLineBytesError);

/// Error experienced during parsing of a line.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SyntaxError {
    /// A generic syntax error that breaks parsing of the line.
    Generic(GenericSyntaxError),
    /// An error experienced while trying to parse [`crate::line::HlsLine::UnknownTag`].
    UnknownTag(UnknownTagSyntaxError),
    /// An error experienced while trying to parse [`crate::date::DateTime`].
    DateTime(DateTimeSyntaxError),
    /// An error experienced while trying to parse a tag value.
    TagValue(TagValueSyntaxError),
    /// Invalid UTF-8 was encountered.
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

/// A generic syntax error that breaks parsing of the line.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GenericSyntaxError {
    /// Carriage return (`U+000D`) was encountered and not followed by line feed (`U+000A`).
    CarriageReturnWithoutLineFeed,
    /// The line ended unexpectedly (e.g. within a quoted string in an attribute list).
    UnexpectedEndOfLine,
    /// Some part of the line could not be decoded as UTF-8.
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
            Self::InvalidUtf8(e) => write!(f, "invalid utf-8 due to {e}"),
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

/// An error experienced while trying to parse [`crate::line::HlsLine::UnknownTag`].
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnknownTagSyntaxError {
    /// The tag prefix `#EXT` existed but nothing more before the line ended or the `:` character
    /// was found.
    UnexpectedNoTagName,
    /// An `UnknownTag` was attempted to be parsed directly (via [`crate::tag::unknown::parse`]),
    /// but the line did not start with `#EXT`.
    InvalidTag,
    /// A generic syntax error that breaks parsing of the line.
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

/// An error experienced while trying to parse [`crate::date::DateTime`].
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DateTimeSyntaxError {
    /// The year component was not a valid number.
    InvalidYear(ParseNumberError),
    /// The separator between year and month was not `-`.
    UnexpectedYearToMonthSeparator(Option<u8>),
    /// The month component was not a valid number.
    InvalidMonth(ParseNumberError),
    /// The separator between month and day was not `-`.
    UnexpectedMonthToDaySeparator(Option<u8>),
    /// The day component was not a valid number.
    InvalidDay(ParseNumberError),
    /// The separator between day and hour was not `T` or `t`.
    UnexpectedDayHourSeparator(Option<u8>),
    /// The hour component was not a valid number.
    InvalidHour(ParseNumberError),
    /// The separator between hour and minute was not `:`.
    UnexpectedHourMinuteSeparator(Option<u8>),
    /// The minute component was not a valid number.
    InvalidMinute(ParseNumberError),
    /// The separator between minute and second was not `:`.
    UnexpectedMinuteSecondSeparator(Option<u8>),
    /// The second component was not a valid number.
    InvalidSecond,
    /// No timezone information was provided.
    UnexpectedNoTimezone,
    /// Characters existed after the timezone.
    UnexpectedCharactersAfterTimezone,
    /// The hour component of the timezone offset was not a valid number.
    InvalidTimezoneHour(ParseNumberError),
    /// The separator between hour and minute in the timezone was not `:`.
    UnexpectedTimezoneHourMinuteSeparator(Option<u8>),
    /// The minute component of the timezone offset was not a valid number.
    InvalidTimezoneMinute(ParseNumberError),
    /// A generic syntax error that breaks parsing of the line.
    Generic(GenericSyntaxError),
}
fn option_u8_to_string(u: &Option<u8>) -> String {
    u.map(|b| format!("{}", b as char))
        .unwrap_or("None".to_string())
}
impl Display for DateTimeSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidYear(e) => write!(f, "invalid integer for year in date due to {e}"),
            Self::UnexpectedYearToMonthSeparator(s) => write!(
                f,
                "expected '-' between year and month but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidMonth(e) => write!(f, "invalid integer for month in date due to {e}"),
            Self::UnexpectedMonthToDaySeparator(s) => write!(
                f,
                "expected '-' between month and day but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidDay(e) => write!(f, "invalid integer for day in date due to {e}"),
            Self::UnexpectedDayHourSeparator(s) => write!(
                f,
                "expected 'T' or 't' between day and hour but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidHour(e) => write!(f, "invalid integer for hour in date due to {e}"),
            Self::UnexpectedHourMinuteSeparator(s) => write!(
                f,
                "expected ':' between hour and minute but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidMinute(e) => write!(f, "invalid integer for minute in date due to {e}"),
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
            Self::InvalidTimezoneHour(e) => {
                write!(f, "invalid integer for hour in timezone due to {e}")
            }
            Self::UnexpectedTimezoneHourMinuteSeparator(s) => write!(
                f,
                "expected ':' between hour and minute in timezone but was {}",
                option_u8_to_string(s)
            ),
            Self::InvalidTimezoneMinute(e) => {
                write!(f, "invalid integer for minute in timezone due to {e}")
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

/// A syntax error found while trying to parse a tag value.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TagValueSyntaxError {
    /// Value was determined to be a decimal floating point but the data was not a valid float.
    InvalidFloatForDecimalFloatingPointValue,
    /// Some part of the value could not be decoded as UTF-8.
    InvalidUtf8(Utf8Error),
    /// Value was determined to be a decimal integer but the data was not a valid number.
    InvalidDecimalInteger(ParseNumberError),
    /// The line ended while reading an attribute name in an attribute list.
    UnexpectedEndOfLineWhileReadingAttributeName,
    /// No value existed for an associated attribute name in an attribute list.
    UnexpectedEmptyAttributeValue,
    /// The line ended while parsing a quoted string in an attribute list.
    UnexpectedEndOfLineWithinQuotedString,
    /// The quoted string ended and was not immediately followed by `,` or end of line.
    UnexpectedCharacterAfterQuotedString(u8),
    /// An attribute value contained whitespace unexpectedly.
    UnexpectedWhitespaceInAttributeValue,
    /// A value was determined to be a floating point but the data was not a valid float.
    InvalidFloatInAttributeValue,
    /// A generic syntax error that breaks parsing of the line.
    Generic(GenericSyntaxError),
}
impl Display for TagValueSyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFloatForDecimalFloatingPointValue => {
                write!(f, "invalid float for decimal float value")
            }
            Self::InvalidUtf8(e) => write!(f, "invalid utf-8 due to {e}"),
            Self::InvalidDecimalInteger(e) => {
                write!(f, "invalid integer for decimal integer value due to {e}")
            }
            Self::UnexpectedEndOfLineWhileReadingAttributeName => {
                write!(f, "unexpected end of line reading attribute name")
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

/// An error experienced while trying to convert into a known tag via `TryFrom<ParsedTag>`.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValidationError {
    /// The tag name did not match expectations for the tag.
    UnexpectedTagName,
    /// The value of the tag did not match expectations for the tag.
    UnexpectedValueType(ValidationErrorValueKind),
    /// A required attribute was missing (the associated value should be the required attribute
    /// name).
    MissingRequiredAttribute(&'static str),
    /// Parsing for this tag is not implemented.
    NotImplemented,
    /// A playlist type value was expected but could not be parsed.
    InvalidPlaylistType(ParsePlaylistTypeError),
    /// A decimal integer value was expected but could not be parsed.
    InvalidDecimalInteger(ParseNumberError),
    /// A decimal integer range value was expected but could not be parsed.
    InvalidDecimalIntegerRange(ParseDecimalIntegerRangeError),
    /// A [`crate::date::DateTime`] value was expected but could not be parsed.
    InvalidDateTime(DateTimeSyntaxError),
    /// A float value was expected but could not be parsed.
    InvalidFloat(ParseFloatError),
    /// The enumerated string extracted from
    /// [`crate::tag::value::ParsedAttributeValue::UnquotedString`] was not a known value.
    InvalidEnumeratedString,
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

/// The kind of value that was found unexpectedly in [`ValidationError::UnexpectedValueType`].
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValidationErrorValueKind {
    /// Corresponds to [`crate::tag::value::SemiParsedTagValue::Empty`].
    Empty,
    /// Corresponds to
    /// [`crate::tag::value::SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle`].
    DecimalFloatingPointWithOptionalTitle,
    /// Corresponds to [`crate::tag::value::SemiParsedTagValue::AttributeList`].
    AttributeList,
    /// Corresponds to [`crate::tag::value::SemiParsedTagValue::Unparsed`].
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

/// An error found while trying to parse a number.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseNumberError {
    /// An invalid digit was found.
    InvalidDigit(u8),
    /// The number was too big.
    NumberTooBig,
    /// Empty data was found instead of a number.
    Empty,
}
impl Display for ParseNumberError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDigit(got) => write!(f, "invalid digit {got}"),
            Self::NumberTooBig => write!(f, "number is too big"),
            Self::Empty => write!(f, "cannot parse number from empty slice"),
        }
    }
}
impl Error for ParseNumberError {}

/// An error when trying to parse the playlist type
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParsePlaylistTypeError {
    /// The value provided was not `VOD` or `EVENT`.
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

/// An error when trying to parse a decimal integer range (`<n>[@<o>]`).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseDecimalIntegerRangeError {
    /// The length component was not a valid number.
    InvalidLength(ParseNumberError),
    /// The offset component was not a valid number.
    InvalidOffset(ParseNumberError),
}
impl Display for ParseDecimalIntegerRangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength(e) => write!(f, "invalid length due to {e}"),
            Self::InvalidOffset(e) => write!(f, "invalid offset due to {e}"),
        }
    }
}
impl Error for ParseDecimalIntegerRangeError {}

/// An error found when trying to parse a float from a byte slice (`&[u8]`).
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ParseFloatError;
impl Display for ParseFloatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid float")
    }
}
impl Error for ParseFloatError {}

/// An enumerated string provided to a parsed tag was not recognized.
#[derive(Debug, PartialEq, Clone)]
pub struct UnrecognizedEnumerationError<'a> {
    /// The unrecognized value that was found.
    pub value: &'a str,
}
impl<'a> UnrecognizedEnumerationError<'a> {
    /// Construct a new instance of the error using the unrecognized value that was found.
    pub fn new(value: &'a str) -> Self {
        Self { value }
    }
}
impl<'a> Display for UnrecognizedEnumerationError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is not a recognized enumeration", self.value)
    }
}
impl Error for UnrecognizedEnumerationError<'_> {}

/// An error found when trying to parse a decimal resolution (`<width>x<height>`).
#[derive(Debug, PartialEq, Clone)]
pub enum DecimalResolutionParseError {
    /// The width component was missing.
    MissingWidth,
    /// The width component was not a valid integer.
    InvalidWidth(ParseIntError),
    /// The height component was missing.
    MissingHeight,
    /// The height component was not a valid integer.
    InvalidHeight(ParseIntError),
}
impl Display for DecimalResolutionParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingWidth => write!(f, "missing width"),
            Self::InvalidWidth(e) => write!(f, "invalid width: {e}"),
            Self::MissingHeight => write!(f, "missing height"),
            Self::InvalidHeight(e) => write!(f, "invalid height: {e}"),
        }
    }
}
impl Error for DecimalResolutionParseError {}
