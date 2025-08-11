//! All error types exposed by the library.
//!
//! The module offers a collection of many error types coming from various operations.

use crate::line::{ParsedByteSlice, ParsedLineSlice};
use std::{
    error::Error,
    fmt::{Display, Formatter},
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
    /// A required attribute was missing (the associated value should be the required attribute
    /// name).
    MissingRequiredAttribute(&'static str),
    /// Parsing for this tag is not implemented.
    NotImplemented,
    /// The expected value of the tag could not be obtained.
    ErrorExtractingTagValue(ParseTagValueError),
    /// An attribute value within an attribute list could not be parsed.
    ErrorExtractingAttributeListValue(ParseAttributeValueError),
    /// The enumerated string extracted from [`crate::tag::value::UnquotedAttributeValue`] was not a
    /// known value.
    InvalidEnumeratedString,
}
impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedTagName => write!(f, "unexpected tag name"),
            Self::MissingRequiredAttribute(a) => write!(f, "required attribute {a} is missing"),
            Self::NotImplemented => write!(f, "parsing into this tag is not implemented"),
            Self::ErrorExtractingTagValue(e) => write!(f, "tag value error - {e}"),
            Self::ErrorExtractingAttributeListValue(e) => {
                write!(f, "attribute list value error - {e}")
            }
            Self::InvalidEnumeratedString => write!(f, "invalid enumerated string in value"),
        }
    }
}
impl Error for ValidationError {}
impl From<ParseTagValueError> for ValidationError {
    fn from(value: ParseTagValueError) -> Self {
        Self::ErrorExtractingTagValue(value)
    }
}
impl From<ParseNumberError> for ValidationError {
    fn from(value: ParseNumberError) -> Self {
        Self::ErrorExtractingTagValue(From::from(value))
    }
}
impl From<ParseDecimalIntegerRangeError> for ValidationError {
    fn from(value: ParseDecimalIntegerRangeError) -> Self {
        Self::ErrorExtractingTagValue(From::from(value))
    }
}
impl From<ParsePlaylistTypeError> for ValidationError {
    fn from(value: ParsePlaylistTypeError) -> Self {
        Self::ErrorExtractingTagValue(From::from(value))
    }
}
impl From<ParseFloatError> for ValidationError {
    fn from(value: ParseFloatError) -> Self {
        Self::ErrorExtractingTagValue(From::from(value))
    }
}
impl From<ParseDecimalFloatingPointWithTitleError> for ValidationError {
    fn from(value: ParseDecimalFloatingPointWithTitleError) -> Self {
        Self::ErrorExtractingTagValue(From::from(value))
    }
}
impl From<DateTimeSyntaxError> for ValidationError {
    fn from(value: DateTimeSyntaxError) -> Self {
        Self::ErrorExtractingTagValue(From::from(value))
    }
}
impl From<AttributeListParsingError> for ValidationError {
    fn from(value: AttributeListParsingError) -> Self {
        Self::ErrorExtractingTagValue(From::from(value))
    }
}
impl From<ParseAttributeValueError> for ValidationError {
    fn from(value: ParseAttributeValueError) -> Self {
        Self::ErrorExtractingAttributeListValue(value)
    }
}

/// An error found trying to convert a tag value into a different data type needed for the tag.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseTagValueError {
    /// The tag value is not empty (`None`) but it should be.
    NotEmpty,
    /// The tag value is empty (`None`) when it should not be.
    UnexpectedEmpty,
    /// An issue found trying to convert into a decimal integer.
    DecimalInteger(ParseNumberError),
    /// An issue fouud trying to convert into a decimal integer range (`<n>[@<o>]`).
    DecimalIntegerRange(ParseDecimalIntegerRangeError),
    /// An issue found trying to convert into a playlist type enum (`EVENT` or `VOD`).
    PlaylistType(ParsePlaylistTypeError),
    /// An issue found trying to convert into a decimal floating point number.
    DecimalFloatingPoint(ParseFloatError),
    /// An issue found trying to convert into a decimal floating point number with a UTF-8 title.
    DecimalFloatingPointWithTitle(ParseDecimalFloatingPointWithTitleError),
    /// An issue found trying to convert into a date/time.
    DateTime(DateTimeSyntaxError),
    /// An issue found trying to convert into an attribute list.
    AttributeList(AttributeListParsingError),
}
impl Display for ParseTagValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotEmpty => write!(f, "tag value was unexpectedly not empty"),
            Self::UnexpectedEmpty => write!(f, "tag value was unexpectedly empty"),
            Self::DecimalInteger(e) => e.fmt(f),
            Self::DecimalIntegerRange(e) => e.fmt(f),
            Self::PlaylistType(e) => e.fmt(f),
            Self::DecimalFloatingPoint(e) => e.fmt(f),
            Self::DecimalFloatingPointWithTitle(e) => e.fmt(f),
            Self::DateTime(e) => e.fmt(f),
            Self::AttributeList(e) => e.fmt(f),
        }
    }
}
impl From<ParseNumberError> for ParseTagValueError {
    fn from(value: ParseNumberError) -> Self {
        Self::DecimalInteger(value)
    }
}
impl From<ParseDecimalIntegerRangeError> for ParseTagValueError {
    fn from(value: ParseDecimalIntegerRangeError) -> Self {
        Self::DecimalIntegerRange(value)
    }
}
impl From<ParsePlaylistTypeError> for ParseTagValueError {
    fn from(value: ParsePlaylistTypeError) -> Self {
        Self::PlaylistType(value)
    }
}
impl From<ParseFloatError> for ParseTagValueError {
    fn from(value: ParseFloatError) -> Self {
        Self::DecimalFloatingPoint(value)
    }
}
impl From<ParseDecimalFloatingPointWithTitleError> for ParseTagValueError {
    fn from(value: ParseDecimalFloatingPointWithTitleError) -> Self {
        Self::DecimalFloatingPointWithTitle(value)
    }
}
impl From<DateTimeSyntaxError> for ParseTagValueError {
    fn from(value: DateTimeSyntaxError) -> Self {
        Self::DateTime(value)
    }
}
impl From<AttributeListParsingError> for ParseTagValueError {
    fn from(value: AttributeListParsingError) -> Self {
        Self::AttributeList(value)
    }
}

/// An error in trying to convert into a decimal float with a title (used in the `EXTINF` tag).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseDecimalFloatingPointWithTitleError {
    /// The duration is not a valid number.
    InvalidDuration(ParseFloatError),
    /// The title is not valid UTF-8.
    InvalidTitle(Utf8Error),
}
impl Display for ParseDecimalFloatingPointWithTitleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDuration(e) => write!(f, "invalid duration due to {e}"),
            Self::InvalidTitle(e) => write!(f, "invalid title due to {e}"),
        }
    }
}
impl Error for ParseDecimalFloatingPointWithTitleError {}
impl From<ParseFloatError> for ParseDecimalFloatingPointWithTitleError {
    fn from(value: ParseFloatError) -> Self {
        Self::InvalidDuration(value)
    }
}
impl From<Utf8Error> for ParseDecimalFloatingPointWithTitleError {
    fn from(value: Utf8Error) -> Self {
        Self::InvalidTitle(value)
    }
}
impl From<fast_float2::Error> for ParseDecimalFloatingPointWithTitleError {
    fn from(_: fast_float2::Error) -> Self {
        Self::InvalidDuration(ParseFloatError)
    }
}

/// An error in trying to convert an attribute value into a different data type as defined for the
/// attribute in the HLS specification.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseAttributeValueError {
    /// The value is wrapped in quotes when it should not be.
    UnexpectedQuoted {
        /// The name of the attribute.
        attr_name: &'static str,
    },
    /// The value is not wrapped in quotes when it should be.
    UnexpectedUnquoted {
        /// The name of the attribute.
        attr_name: &'static str,
    },
    /// An issue found trying to convert into a decimal integer.
    DecimalInteger {
        /// The name of the attribute.
        attr_name: &'static str,
        /// The underlying error.
        error: ParseNumberError,
    },
    /// An issue found trying to convert into a decimal floating point.
    DecimalFloatingPoint {
        /// The name of the attribute.
        attr_name: &'static str,
        /// The underlying error.
        error: ParseFloatError,
    },
    /// An issue found trying to convert into a decimal resolution.
    DecimalResolution {
        /// The name of the attribute.
        attr_name: &'static str,
        /// The underlying error.
        error: DecimalResolutionParseError,
    },
    /// An issue found trying to convert into a UTF-8 string.
    Utf8 {
        /// The name of the attribute.
        attr_name: &'static str,
        /// The underlying error.
        error: Utf8Error,
    },
}
impl Display for ParseAttributeValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedQuoted { attr_name } => {
                write!(f, "{attr_name} expected to be unquoted but was quoted")
            }
            Self::UnexpectedUnquoted { attr_name } => {
                write!(f, "{attr_name} expected to be quoted but was unquoted")
            }
            Self::DecimalInteger { attr_name, error } => write!(
                f,
                "could not extract decimal integer for {attr_name} due to {error}"
            ),
            Self::DecimalFloatingPoint { attr_name, error } => write!(
                f,
                "could not extract decimal floating point for {attr_name} due to {error}"
            ),
            Self::DecimalResolution { attr_name, error } => write!(
                f,
                "could not extract decimal resolution for {attr_name} due to {error}"
            ),
            Self::Utf8 { attr_name, error } => write!(
                f,
                "could not extract utf-8 string for {attr_name} due to {error}"
            ),
        }
    }
}
impl Error for ParseAttributeValueError {}

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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DecimalResolutionParseError {
    /// The width component was missing.
    MissingWidth,
    /// The width component was not a valid integer.
    InvalidWidth,
    /// The height component was missing.
    MissingHeight,
    /// The height component was not a valid integer.
    InvalidHeight,
}
impl Display for DecimalResolutionParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingWidth => write!(f, "missing width"),
            Self::InvalidWidth => write!(f, "not a number for width"),
            Self::MissingHeight => write!(f, "missing height"),
            Self::InvalidHeight => write!(f, "not a number for height"),
        }
    }
}
impl Error for DecimalResolutionParseError {}

/// An error found while parsing an attribute list.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AttributeListParsingError {
    /// The line ended while reading an attribute name (before we found `=`).
    EndOfLineWhileReadingAttributeName,
    /// There was an unexpected character (`"` or `,`) in the attribute name.
    UnexpectedCharacterInAttributeName,
    /// The attribute name was empty (no characters between start and `=`).
    EmptyAttributeName,
    /// The unquoted attribute value was empty (no characters between start and `,` or end of line).
    EmptyUnquotedValue,
    /// Unexpected character in unquoted attribute value (`=` or `"` after initial index).
    UnexpectedCharacterInAttributeValue,
    /// Unexpected character occurring after quote end and before `,`.
    UnexpectedCharacterAfterQuoteEnd,
    /// The line ended while reading a quoted string value.
    EndOfLineWhileReadingQuotedValue,
    /// There was an error when trying to convert to UTF-8.
    InvalidUtf8(std::str::Utf8Error),
}
impl Display for AttributeListParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfLineWhileReadingAttributeName => {
                write!(f, "line ended while reading attribute name")
            }
            Self::UnexpectedCharacterInAttributeName => {
                write!(f, "unexpected character in attribute name")
            }
            Self::EmptyAttributeName => write!(f, "attribute name with no characters"),
            Self::EmptyUnquotedValue => write!(f, "unquoted value with no characters"),
            Self::UnexpectedCharacterInAttributeValue => {
                write!(f, "unexpected character in attribute value")
            }
            Self::UnexpectedCharacterAfterQuoteEnd => {
                write!(f, "unexpected character between quoted string end and ','")
            }
            Self::EndOfLineWhileReadingQuotedValue => {
                write!(f, "line ended while reading quoted string value")
            }
            Self::InvalidUtf8(e) => write!(f, "invalid utf-8 due to {e}"),
        }
    }
}
impl std::error::Error for AttributeListParsingError {}
impl From<std::str::Utf8Error> for AttributeListParsingError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}
