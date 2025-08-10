//! Collection of methods and types used to extract meaning from the value component of a tag line.
//!
//! The value of a tag (when not empty) is everything after the `:` and before the new line break.
//! This module provides types of values and methods for parsing into these types from input data.

use crate::{
    date::{self, DateTime},
    error::{
        AttributeListParsingError, DateTimeSyntaxError, DecimalResolutionParseError,
        ParseDecimalFloatingPointWithTitleError, ParseDecimalIntegerRangeError, ParseFloatError,
        ParseNumberError, ParsePlaylistTypeError,
    },
    utils::parse_u64,
};
use memchr::{memchr, memchr_iter, memchr3_iter};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// A wrapper struct that provides many convenience methods for converting a tag value into a more
/// specialized type.
///
/// The `TagValue` is intended to wrap the bytes following the `:` and before the end of line (not
/// including the `\r` or `\n` characters). The constructor remains public (for convenience, as
/// described below) so bear this in mind if trying to use this struct directly. It is unlikely that
/// a user will need to construct this directly, and instead, should access this via
/// [`crate::tag::unknown::Tag::value`] (`Tag` is via [`crate::tag::unknown::parse`]). There may be
/// exceptions and so the library provides this flexibility.
///
/// For example, a (perhaps interesting) use case for using this struct directly can be to parse
/// information out of comment tags. For example, it has been noticed that the Unified Streaming
/// Packager seems to output a custom timestamp comment with its live playlists, that looks like a
/// tag; however, the library will not parse this as a tag because the syntax is
/// `#USP-X-TIMESTAMP-MAP:<attribute-list>`, so the lack of `#EXT` prefix means it is seen as a
/// comment only. Despite this, if we split on the `:`, we can use this struct to extract
/// information about the value.
/// ```
/// # use m3u8::{
/// #     HlsLine, Reader,
/// #     config::ParsingOptions,
/// #     date, date_time,
/// #     line::ParsedByteSlice,
/// #     tag::value::{TagValue, AttributeValue},
/// #     error::ValidationError,
/// # };
/// let pseudo_tag = "#USP-X-TIMESTAMP-MAP:MPEGTS=900000,LOCAL=1970-01-01T00:00:00Z";
/// let mut reader = Reader::from_str(pseudo_tag, ParsingOptions::default());
/// match reader.read_line() {
///     Ok(Some(HlsLine::Comment(tag))) => {
///         let mut tag_split = tag.splitn(2, ':');
///         if tag_split.next() != Some("USP-X-TIMESTAMP-MAP") {
///             return Err(format!("unexpected tag name").into());
///         }
///         let Some(value) = tag_split.next() else {
///             return Err(format!("unexpected no tag value").into());
///         };
///         let tag_value = TagValue(value.trim().as_bytes());
///         let list = tag_value.try_as_attribute_list()?;
///
///         // Prove that we can extract the value of MPEGTS
///         let mpegts = list
///             .get("MPEGTS")
///             .and_then(AttributeValue::unquoted)
///             .ok_or(ValidationError::MissingRequiredAttribute("MPEGTS"))?
///             .try_as_decimal_integer()?;
///         assert_eq!(900000, mpegts);
///
///         // Prove that we can extract the value of LOCAL
///         let local = list
///             .get("LOCAL")
///             .and_then(AttributeValue::unquoted)
///             .and_then(|v| date::parse_bytes(v.0).ok())
///             .ok_or(ValidationError::MissingRequiredAttribute("LOCAL"))?;
///         assert_eq!(date_time!(1970-01-01 T 00:00:00.000), local);
///     }
///     r => return Err(format!("unexpected result {r:?}").into()),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TagValue<'a>(pub &'a [u8]);
impl<'a> TagValue<'a> {
    /// Indicates whether the value is empty or not.
    ///
    /// This is only the case if the tag contained a `:` value separator but had no value content
    /// afterwards (before the new line). Under all known circumstances this is an error. If a tag
    /// value is empty then this is indicated via [`crate::tag::unknown::Tag::value`] providing
    /// `None`.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Attempt to convert the tag value bytes into a decimal integer.
    ///
    /// For example:
    /// ```
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:100")?.parsed;
    /// let value = tag.value().expect("should have value defined");
    /// assert_eq!(100, value.try_as_decimal_integer()?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_decimal_integer(&self) -> Result<u64, ParseNumberError> {
        parse_u64(self.0)
    }

    /// Attempt to convert the tag value bytes into a decimal integer range (`<n>[@<o>]`).
    ///
    /// For example:
    /// ```
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:1024@512")?.parsed;
    /// let value = tag.value().expect("should have value defined");
    /// assert_eq!((1024, Some(512)), value.try_as_decimal_integer_range()?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Attempt to convert the tag value bytes into a playlist type.
    ///
    /// For example:
    /// ```
    /// # use m3u8::tag::value::HlsPlaylistType;
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:VOD")?.parsed;
    /// let value = tag.value().expect("should have value defined");
    /// assert_eq!(HlsPlaylistType::Vod, value.try_as_playlist_type()?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_playlist_type(&self) -> Result<HlsPlaylistType, ParsePlaylistTypeError> {
        if self.0 == b"VOD" {
            Ok(HlsPlaylistType::Vod)
        } else if self.0 == b"EVENT" {
            Ok(HlsPlaylistType::Event)
        } else {
            Err(ParsePlaylistTypeError::InvalidValue)
        }
    }

    /// Attempt to convert the tag value bytes into a decimal floating point.
    ///
    /// For example:
    /// ```
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:3.14")?.parsed;
    /// let value = tag.value().expect("should have value defined");
    /// assert_eq!(3.14, value.try_as_decimal_floating_point()?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_decimal_floating_point(&self) -> Result<f64, ParseFloatError> {
        fast_float2::parse(self.0).map_err(|_| ParseFloatError)
    }

    /// Attempt to convert the tag value bytes into a decimal floating point with title.
    ///
    /// For example:
    /// ```
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:3.14,pi")?.parsed;
    /// let value = tag.value().expect("should have value defined");
    /// assert_eq!((3.14, "pi"), value.try_as_decimal_floating_point_with_title()?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_decimal_floating_point_with_title(
        &self,
    ) -> Result<(f64, &'a str), ParseDecimalFloatingPointWithTitleError> {
        match memchr(b',', self.0) {
            Some(n) => {
                let duration = fast_float2::parse(&self.0[..n])?;
                let title = std::str::from_utf8(&self.0[(n + 1)..])?;
                Ok((duration, title))
            }
            None => {
                let duration = fast_float2::parse(self.0)?;
                Ok((duration, ""))
            }
        }
    }

    /// Attempt to convert the tag value bytes into a date time.
    ///
    /// For example:
    /// ```
    /// # use m3u8::date_time;
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:2025-08-10T17:27:42.213-05:00")?.parsed;
    /// let value = tag.value().expect("should have value defined");
    /// assert_eq!(date_time!(2025-08-10 T 17:27:42.213 -05:00), value.try_as_date_time()?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_date_time(&self) -> Result<DateTime, DateTimeSyntaxError> {
        date::parse_bytes(self.0)
    }

    /// Attempt to convert the tag value bytes into an attribute list.
    ///
    /// For example:
    /// ```
    /// # use std::collections::HashMap;
    /// # use m3u8::tag::value::{AttributeValue, UnquotedAttributeValue};
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:TYPE=LIST,VALUE=\"example\"")?.parsed;
    /// let value = tag.value().expect("should have value defined");
    /// assert_eq!(
    ///     HashMap::from([
    ///         ("TYPE", AttributeValue::Unquoted(UnquotedAttributeValue(b"LIST"))),
    ///         ("VALUE", AttributeValue::Quoted("example"))
    ///     ]),
    ///     value.try_as_attribute_list()?
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_attribute_list(
        &self,
    ) -> Result<HashMap<&'a str, AttributeValue<'a>>, AttributeListParsingError> {
        let mut attribute_list = HashMap::new();
        let mut list_iter = memchr3_iter(b'=', b',', b'"', self.0);
        // Name in first position is special because we want to capture the whole value from the
        // previous_match_index (== 0), rather than in the rest of cases, where we want to capture
        // the value at the index after the previous match (which should be b','). Therefore, we use
        // the `next` method to step through the first match and handle it specially, then proceed
        // to loop through the iterator for all others.
        let Some(first_match_index) = list_iter.next() else {
            return Err(AttributeListParsingError::EndOfLineWhileReadingAttributeName);
        };
        if self.0[first_match_index] != b'=' {
            return Err(AttributeListParsingError::UnexpectedCharacterInAttributeName);
        }
        let mut previous_match_index = first_match_index;
        let mut state = AttributeListParsingState::ReadingValue {
            name: std::str::from_utf8(&self.0[..first_match_index])?,
        };
        for i in list_iter {
            let byte = self.0[i];
            match state {
                AttributeListParsingState::ReadingName => {
                    if byte == b'=' {
                        // end of name section
                        let name = std::str::from_utf8(&self.0[(previous_match_index + 1)..i])?;
                        if name.is_empty() {
                            return Err(AttributeListParsingError::EmptyAttributeName);
                        }
                        state = AttributeListParsingState::ReadingValue { name };
                    } else {
                        // b',' and b'"' are both unexpected
                        return Err(AttributeListParsingError::UnexpectedCharacterInAttributeName);
                    }
                    previous_match_index = i;
                }
                AttributeListParsingState::ReadingQuotedValue { name } => {
                    if byte == b'"' {
                        // only byte that ends the quoted value is b'"'
                        let value = std::str::from_utf8(&self.0[(previous_match_index + 1)..i])?;
                        state =
                            AttributeListParsingState::FinishedReadingQuotedValue { name, value };
                        previous_match_index = i;
                    }
                }
                AttributeListParsingState::ReadingValue { name } => {
                    if byte == b'"' {
                        // must check that this is the first character of the value
                        if previous_match_index != (i - 1) {
                            // finding b'"' mid-value is unexpected
                            return Err(
                                AttributeListParsingError::UnexpectedCharacterInAttributeValue,
                            );
                        }
                        state = AttributeListParsingState::ReadingQuotedValue { name };
                    } else if byte == b',' {
                        let value = UnquotedAttributeValue(&self.0[(previous_match_index + 1)..i]);
                        if value.0.is_empty() {
                            // an empty unquoted value is unexpected (only quoted may be empty)
                            return Err(AttributeListParsingError::EmptyUnquotedValue);
                        }
                        attribute_list.insert(name, AttributeValue::Unquoted(value));
                        state = AttributeListParsingState::ReadingName;
                    } else {
                        // b'=' is unexpected while reading value (only b',' or b'"' are expected)
                        return Err(AttributeListParsingError::UnexpectedCharacterInAttributeValue);
                    }
                    previous_match_index = i;
                }
                AttributeListParsingState::FinishedReadingQuotedValue { name, value } => {
                    if byte == b',' {
                        attribute_list.insert(name, AttributeValue::Quoted(value));
                        state = AttributeListParsingState::ReadingName;
                    } else {
                        // b',' (or end of line) must come after end of quote - all else is invalid
                        return Err(AttributeListParsingError::UnexpectedCharacterAfterQuoteEnd);
                    }
                    previous_match_index = i;
                }
            }
        }
        // Need to check state at end of line as this will likely not be a match in the above
        // iteration.
        match state {
            AttributeListParsingState::ReadingName => {
                return Err(AttributeListParsingError::EndOfLineWhileReadingAttributeName);
            }
            AttributeListParsingState::ReadingValue { name } => {
                let value = UnquotedAttributeValue(&self.0[(previous_match_index + 1)..]);
                if value.0.is_empty() {
                    // an empty unquoted value is unexpected (only quoted may be empty)
                    return Err(AttributeListParsingError::EmptyUnquotedValue);
                }
                attribute_list.insert(name, AttributeValue::Unquoted(value));
            }
            AttributeListParsingState::ReadingQuotedValue { name: _ } => {
                return Err(AttributeListParsingError::EndOfLineWhileReadingQuotedValue);
            }
            AttributeListParsingState::FinishedReadingQuotedValue { name, value } => {
                attribute_list.insert(name, AttributeValue::Quoted(value));
            }
        }
        Ok(attribute_list)
    }
}

enum AttributeListParsingState<'a> {
    ReadingName,
    ReadingValue { name: &'a str },
    ReadingQuotedValue { name: &'a str },
    FinishedReadingQuotedValue { name: &'a str, value: &'a str },
}

/// An attribute value within an attribute list.
///
/// Values may be quoted or unquoted. In the case that they are unquoted they may be converted into
/// several other data types. This is done via use of convenience methods on
/// [`UnquotedAttributeValue`].
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AttributeValue<'a> {
    /// An unquoted value (e.g. `TYPE=AUDIO`, `BANDWIDTH=10000000`, `SCORE=1.5`,
    /// `RESOLUTION=1920x1080`, `SCTE35-OUT=0xABCD`, etc.).
    Unquoted(UnquotedAttributeValue<'a>),
    /// A quoted value (e.g. `CODECS="avc1.64002a,mp4a.40.2"`).
    Quoted(&'a str),
}
impl<'a> AttributeValue<'a> {
    /// A convenience method to get the value of the `Unquoted` case.
    ///
    /// This can be useful when chaining on optional values. For example:
    /// ```
    /// # use std::collections::HashMap;
    /// # use m3u8::tag::value::AttributeValue;
    /// fn get_bandwidth(list: &HashMap<&str, AttributeValue>) -> Option<u64> {
    ///     list
    ///         .get("BANDWIDTH")
    ///         .and_then(AttributeValue::unquoted)
    ///         .and_then(|v| v.try_as_decimal_integer().ok())
    /// }
    /// ```
    pub fn unquoted(&self) -> Option<UnquotedAttributeValue<'a>> {
        match self {
            AttributeValue::Unquoted(v) => Some(*v),
            AttributeValue::Quoted(_) => None,
        }
    }
    /// A convenience method to get the value of the `Quoted` case.
    ///
    /// This can be useful when chaining on optional values. For example:
    /// ```
    /// # use std::collections::HashMap;
    /// # use m3u8::tag::value::AttributeValue;
    /// fn get_codecs<'a>(list: &HashMap<&'a str, AttributeValue<'a>>) -> Option<&'a str> {
    ///     list
    ///         .get("CODECS")
    ///         .and_then(AttributeValue::quoted)
    /// }
    /// ```
    pub fn quoted(&self) -> Option<&'a str> {
        match self {
            AttributeValue::Unquoted(_) => None,
            AttributeValue::Quoted(s) => Some(*s),
        }
    }
}

/// A wrapper struct that provides many convenience methods for converting an unquoted attribute
/// value into a specialized type.
///
/// It is very unlikely that this struct will need to be constructed directly. This is more normally
/// found when taking an attribute list tag value and accessing some of the internal attributes. For
/// example:
/// ```
/// # use std::collections::HashMap;
/// # use m3u8::tag::value::{AttributeValue, UnquotedAttributeValue};
/// let tag = m3u8::tag::unknown::parse("#EXT-X-EXAMPLE:TYPE=PI,NUMBER=3.14")?.parsed;
/// let value = tag.value().expect("should have value defined");
/// let list = value.try_as_attribute_list()?;
///
/// let type_value = list
///     .get("TYPE")
///     .and_then(AttributeValue::unquoted)
///     .expect("should be defined and unquoted");
/// assert_eq!(UnquotedAttributeValue(b"PI"), type_value);
/// assert_eq!(Ok("PI"), type_value.try_as_utf_8());
///
/// let number_value = list
///     .get("NUMBER")
///     .and_then(AttributeValue::unquoted)
///     .expect("should be defined and unquoted");
/// assert_eq!(UnquotedAttributeValue(b"3.14"), number_value);
/// assert_eq!(Ok(3.14), number_value.try_as_decimal_floating_point());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UnquotedAttributeValue<'a>(pub &'a [u8]);
impl<'a> UnquotedAttributeValue<'a> {
    /// Attempt to convert the attribute value bytes into a decimal integer.
    ///
    /// For example:
    /// ```
    /// # use m3u8::tag::value::AttributeValue;
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-TEST:EXAMPLE=42")?.parsed;
    /// let list = tag.value().expect("should have value defined").try_as_attribute_list()?;
    /// assert_eq!(
    ///     Some(42),
    ///     list
    ///         .get("EXAMPLE")
    ///         .and_then(AttributeValue::unquoted)
    ///         .and_then(|v| v.try_as_decimal_integer().ok())
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_decimal_integer(&self) -> Result<u64, ParseNumberError> {
        parse_u64(self.0)
    }

    /// Attempt to convert the attribute value bytes into a decimal floating point.
    ///
    /// For example:
    /// ```
    /// # use m3u8::tag::value::AttributeValue;
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-TEST:EXAMPLE=3.14")?.parsed;
    /// let list = tag.value().expect("should have value defined").try_as_attribute_list()?;
    /// assert_eq!(
    ///     Some(3.14),
    ///     list
    ///         .get("EXAMPLE")
    ///         .and_then(AttributeValue::unquoted)
    ///         .and_then(|v| v.try_as_decimal_floating_point().ok())
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_decimal_floating_point(&self) -> Result<f64, ParseFloatError> {
        fast_float2::parse(self.0).map_err(|_| ParseFloatError)
    }

    /// Attempt to convert the attribute value bytes into a decimal resolution.
    ///
    /// For example:
    /// ```
    /// # use m3u8::tag::value::{AttributeValue, DecimalResolution};
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-TEST:EXAMPLE=1920x1080")?.parsed;
    /// let list = tag.value().expect("should have value defined").try_as_attribute_list()?;
    /// assert_eq!(
    ///     Some(DecimalResolution { width: 1920, height: 1080 }),
    ///     list
    ///         .get("EXAMPLE")
    ///         .and_then(AttributeValue::unquoted)
    ///         .and_then(|v| v.try_as_decimal_resolution().ok())
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_decimal_resolution(
        &self,
    ) -> Result<DecimalResolution, DecimalResolutionParseError> {
        let mut x_iter = memchr_iter(b'x', self.0);
        let Some(i) = x_iter.next() else {
            return Err(DecimalResolutionParseError::MissingHeight);
        };
        let width =
            parse_u64(&self.0[..i]).map_err(|_| DecimalResolutionParseError::InvalidWidth)?;
        let height = parse_u64(&self.0[(i + 1)..])
            .map_err(|_| DecimalResolutionParseError::InvalidHeight)?;
        Ok(DecimalResolution { width, height })
    }

    /// Attempt to convert the attribute value bytes into a UTF-8 string.
    ///
    /// For example:
    /// ```
    /// # use m3u8::tag::value::AttributeValue;
    /// let tag = m3u8::tag::unknown::parse("#EXT-X-TEST:EXAMPLE=ENUMERATED-VALUE")?.parsed;
    /// let list = tag.value().expect("should have value defined").try_as_attribute_list()?;
    /// assert_eq!(
    ///     Some("ENUMERATED-VALUE"),
    ///     list
    ///         .get("EXAMPLE")
    ///         .and_then(AttributeValue::unquoted)
    ///         .and_then(|v| v.try_as_utf_8().ok())
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_as_utf_8(&self) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.0)
    }
}

/// The HLS playlist type, as defined in [`#EXT-X-PLAYLIST-TYPE`].
///
/// [`#EXT-X-PLAYLIST-TYPE`]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.5
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HlsPlaylistType {
    /// If the `EXT-X-PLAYLIST-TYPE` value is EVENT, Media Segments can only be added to the end of
    /// the Media Playlist.
    Event,
    /// If the `EXT-X-PLAYLIST-TYPE` value is Video On Demand (VOD), the Media Playlist cannot
    /// change.
    Vod,
}

/// Provides a mutable version of [`SemiParsedTagValue`].
///
/// This is provided so that custom tag implementations may provide an output that does not depend
/// on having parsed data to derive the write output from. This helps with mutability as well as
/// allowing for custom tags to be constructed from scratch (without being parsed from source data).
///
/// This mirrors the [`SemiParsedTagValue`] but provides data types that allow for owned data
/// (rather than just borrowed references from parsed input data).
#[derive(Debug, PartialEq)]
pub enum MutableSemiParsedTagValue<'a> {
    /// The value is empty.
    ///
    /// See [`SemiParsedTagValue::Empty`] for more information.
    Empty,
    /// The value is a float with a string title.
    ///
    /// See [`SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle`] for more information.
    DecimalFloatingPointWithOptionalTitle(f64, Cow<'a, str>),
    /// The value is an attribute list.
    ///
    /// See [`SemiParsedTagValue::AttributeList`] for more information.
    AttributeList(HashMap<Cow<'a, str>, MutableParsedAttributeValue<'a>>),
    /// The value is unparsed.
    ///
    /// See [`SemiParsedTagValue::Unparsed`] for more information.
    Unparsed(MutableUnparsedTagValue<'a>),
}
impl<'a, T> From<(f64, T)> for MutableSemiParsedTagValue<'a>
where
    T: Into<Cow<'a, str>>,
{
    fn from(value: (f64, T)) -> Self {
        Self::DecimalFloatingPointWithOptionalTitle(value.0, value.1.into())
    }
}
impl<'a, K, V> From<HashMap<K, V>> for MutableSemiParsedTagValue<'a>
where
    K: Into<Cow<'a, str>>,
    V: Into<MutableParsedAttributeValue<'a>>,
{
    fn from(mut value: HashMap<K, V>) -> Self {
        let mut map = HashMap::new();
        for (key, value) in value.drain() {
            map.insert(key.into(), value.into());
        }
        Self::AttributeList(map)
    }
}
impl<'a, K, V, const N: usize> From<[(K, V); N]> for MutableSemiParsedTagValue<'a>
where
    K: Into<Cow<'a, str>>,
    V: Into<MutableParsedAttributeValue<'a>>,
{
    fn from(value: [(K, V); N]) -> Self {
        let mut map = HashMap::new();
        for (key, value) in value {
            map.insert(key.into(), value.into());
        }
        Self::AttributeList(map)
    }
}
impl<'a> From<Cow<'a, [u8]>> for MutableSemiParsedTagValue<'a> {
    fn from(value: Cow<'a, [u8]>) -> Self {
        Self::Unparsed(MutableUnparsedTagValue(value))
    }
}

/// Provides a mutable version of [`UnparsedTagValue`].
///
/// This is provided so that custom tag implementations may provide an output that does not depend
/// on having parsed data to derive the write output from. This helps with mutability as well as
/// allowing for custom tags to be constructed from scratch (without being parsed from source data).
///
/// This mirrors the [`UnparsedTagValue`] but provides data types that allow for owned data (rather
/// than just borrowed references from parsed input data).
#[derive(Debug, PartialEq)]
pub struct MutableUnparsedTagValue<'a>(pub Cow<'a, [u8]>);

/// Provides a mutable version of [`ParsedAttributeValue`].
///
/// This is provided so that custom tag implementations may provide an output that does not depend
/// on having parsed data to derive the write output from. This helps with mutability as well as
/// allowing for custom tags to be constructed from scratch (without being parsed from source data).
///
/// This mirrors the [`ParsedAttributeValue`] but provides data types that allow for owned data
/// (rather than just borrowed references from parsed input data).
#[derive(Debug, PartialEq, Clone)]
pub enum MutableParsedAttributeValue<'a> {
    /// A decimal integer.
    ///
    /// See [ParsedAttributeValue::DecimalInteger] for more information.
    DecimalInteger(u64),
    /// A signed float.
    ///
    /// See [ParsedAttributeValue::SignedDecimalFloatingPoint] for more information.
    SignedDecimalFloatingPoint(f64),
    /// A quoted string.
    ///
    /// See [ParsedAttributeValue::QuotedString] for more information.
    QuotedString(Cow<'a, str>),
    /// An unquoted string.
    ///
    /// See [ParsedAttributeValue::UnquotedString] for more information.
    UnquotedString(Cow<'a, str>),
}
impl From<u64> for MutableParsedAttributeValue<'_> {
    fn from(value: u64) -> Self {
        Self::DecimalInteger(value)
    }
}
impl From<f64> for MutableParsedAttributeValue<'_> {
    fn from(value: f64) -> Self {
        Self::SignedDecimalFloatingPoint(value)
    }
}

/// A decimal resolution (`<width>x<height>`).
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DecimalResolution {
    /// A horizontal pixel dimension (width).
    pub width: u64,
    /// A vertical pixel dimension (height).
    pub height: u64,
}
impl Display for DecimalResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}
impl TryFrom<&str> for DecimalResolution {
    type Error = DecimalResolutionParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut split = s.splitn(2, 'x');
        let Some(width_str) = split.next() else {
            return Err(DecimalResolutionParseError::MissingWidth);
        };
        let width = width_str
            .parse()
            .map_err(|_| DecimalResolutionParseError::InvalidWidth)?;
        let Some(height_str) = split.next() else {
            return Err(DecimalResolutionParseError::MissingHeight);
        };
        let height = height_str
            .parse()
            .map_err(|_| DecimalResolutionParseError::InvalidHeight)?;
        Ok(DecimalResolution { width, height })
    }
}

#[cfg(test)]
mod tests {
    use crate::date_time;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn type_enum() {
        let value = TagValue(b"EVENT");
        assert_eq!(Ok(HlsPlaylistType::Event), value.try_as_playlist_type());

        let value = TagValue(b"VOD");
        assert_eq!(Ok(HlsPlaylistType::Vod), value.try_as_playlist_type());
    }

    #[test]
    fn decimal_integer() {
        let value = TagValue(b"42");
        assert_eq!(Ok(42), value.try_as_decimal_integer());
    }

    #[test]
    fn decimal_integer_range() {
        let value = TagValue(b"42@42");
        assert_eq!(Ok((42, Some(42))), value.try_as_decimal_integer_range());
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        // Positive tests
        let value = TagValue(b"42.0");
        assert_eq!(
            Ok((42.0, "")),
            value.try_as_decimal_floating_point_with_title()
        );
        let value = TagValue(b"42.42");
        assert_eq!(
            Ok((42.42, "")),
            value.try_as_decimal_floating_point_with_title()
        );
        let value = TagValue(b"42,");
        assert_eq!(
            Ok((42.0, "")),
            value.try_as_decimal_floating_point_with_title()
        );
        let value = TagValue(b"42,=ATTRIBUTE-VALUE");
        assert_eq!(
            Ok((42.0, "=ATTRIBUTE-VALUE")),
            value.try_as_decimal_floating_point_with_title()
        );
        // Negative tests
        let value = TagValue(b"-42.0");
        assert_eq!(
            Ok((-42.0, "")),
            value.try_as_decimal_floating_point_with_title()
        );
        let value = TagValue(b"-42.42");
        assert_eq!(
            Ok((-42.42, "")),
            value.try_as_decimal_floating_point_with_title()
        );
        let value = TagValue(b"-42,");
        assert_eq!(
            Ok((-42.0, "")),
            value.try_as_decimal_floating_point_with_title()
        );
        let value = TagValue(b"-42,=ATTRIBUTE-VALUE");
        assert_eq!(
            Ok((-42.0, "=ATTRIBUTE-VALUE")),
            value.try_as_decimal_floating_point_with_title()
        );
    }

    #[test]
    fn date_time_msec() {
        let value = TagValue(b"2025-06-03T17:56:42.123Z");
        assert_eq!(
            Ok(date_time!(2025-06-03 T 17:56:42.123)),
            value.try_as_date_time(),
        );
        let value = TagValue(b"2025-06-03T17:56:42.123+01:00");
        assert_eq!(
            Ok(date_time!(2025-06-03 T 17:56:42.123 01:00)),
            value.try_as_date_time(),
        );
        let value = TagValue(b"2025-06-03T17:56:42.123-05:00");
        assert_eq!(
            Ok(date_time!(2025-06-03 T 17:56:42.123 -05:00)),
            value.try_as_date_time(),
        );
    }

    mod attribute_list {
        use super::*;

        macro_rules! unquoted_value_test {
            (TagValue is $tag_value:literal $($name_lit:literal=$val:literal expects $exp:literal from $method:ident)+) => {
                let value = TagValue($tag_value);
                let list = value.try_as_attribute_list().expect("should be valid list");
                assert_eq!(
                    list,
                    HashMap::from([
                        $(
                            ($name_lit, AttributeValue::Unquoted(UnquotedAttributeValue($val))),
                        )+
                    ])
                );
                $(
                    assert_eq!(Ok($exp), UnquotedAttributeValue($val).$method());
                )+
            };
        }

        macro_rules! quoted_value_test {
            (TagValue is $tag_value:literal $($name_lit:literal expects $exp:literal)+) => {
                let value = TagValue($tag_value);
                let list = value.try_as_attribute_list().expect("should be valid list");
                assert_eq!(
                    list,
                    HashMap::from([
                        $(
                            ($name_lit, AttributeValue::Quoted($exp)),
                        )+
                    ])
                );
            };
        }

        mod decimal_integer {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                unquoted_value_test!(
                    TagValue is b"NAME=123"
                    "NAME"=b"123" expects 123 from try_as_decimal_integer
                );
            }

            #[test]
            fn multi_attributes() {
                unquoted_value_test!(
                    TagValue is b"NAME=123,NEXT-NAME=456"
                    "NAME"=b"123" expects 123 from try_as_decimal_integer
                    "NEXT-NAME"=b"456" expects 456 from try_as_decimal_integer
                );
            }
        }

        mod signed_decimal_floating_point {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn positive_float_single_attribute() {
                unquoted_value_test!(
                    TagValue is b"NAME=42.42"
                    "NAME"=b"42.42" expects 42.42 from try_as_decimal_floating_point
                );
            }

            #[test]
            fn negative_integer_single_attribute() {
                unquoted_value_test!(
                    TagValue is b"NAME=-42"
                    "NAME"=b"-42" expects -42.0 from try_as_decimal_floating_point
                );
            }

            #[test]
            fn negative_float_single_attribute() {
                unquoted_value_test!(
                    TagValue is b"NAME=-42.42"
                    "NAME"=b"-42.42" expects -42.42 from try_as_decimal_floating_point
                );
            }

            #[test]
            fn positive_float_multi_attributes() {
                unquoted_value_test!(
                    TagValue is b"NAME=42.42,NEXT-NAME=84.84"
                    "NAME"=b"42.42" expects 42.42 from try_as_decimal_floating_point
                    "NEXT-NAME"=b"84.84" expects 84.84 from try_as_decimal_floating_point
                );
            }

            #[test]
            fn negative_integer_multi_attributes() {
                unquoted_value_test!(
                    TagValue is b"NAME=-42,NEXT-NAME=-84"
                    "NAME"=b"-42" expects -42.0 from try_as_decimal_floating_point
                    "NEXT-NAME"=b"-84" expects -84.0 from try_as_decimal_floating_point
                );
            }

            #[test]
            fn negative_float_multi_attributes() {
                unquoted_value_test!(
                    TagValue is b"NAME=-42.42,NEXT-NAME=-84.84"
                    "NAME"=b"-42.42" expects -42.42 from try_as_decimal_floating_point
                    "NEXT-NAME"=b"-84.84" expects -84.84 from try_as_decimal_floating_point
                );
            }
        }

        mod quoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                quoted_value_test!(
                    TagValue is b"NAME=\"Hello, World!\""
                    "NAME" expects "Hello, World!"
                );
            }

            #[test]
            fn multi_attributes() {
                quoted_value_test!(
                    TagValue is b"NAME=\"Hello,\",NEXT-NAME=\"World!\""
                    "NAME" expects "Hello,"
                    "NEXT-NAME" expects "World!"
                );
            }
        }

        mod unquoted_string {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn single_attribute() {
                unquoted_value_test!(
                    TagValue is b"NAME=PQ"
                    "NAME"=b"PQ" expects "PQ" from try_as_utf_8
                );
            }

            #[test]
            fn multi_attributes() {
                unquoted_value_test!(
                    TagValue is b"NAME=PQ,NEXT-NAME=HLG"
                    "NAME"=b"PQ" expects "PQ" from try_as_utf_8
                    "NEXT-NAME"=b"HLG" expects "HLG" from try_as_utf_8
                );
            }
        }
    }
}
