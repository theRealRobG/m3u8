//! Collection of methods and types used to extract meaning from the value component of a tag line.
//!
//! The value of a tag (when not empty) is everything after the `:` and before the new line break.
//! This module provides types of values and methods for parsing into these types from input data.

use crate::{
    date::{self, DateTime},
    error::{
        DateTimeSyntaxError, DecimalResolutionParseError, ParseDecimalIntegerRangeError,
        ParseFloatError, ParseNumberError, ParsePlaylistTypeError, TagValueSyntaxError,
    },
    line::ParsedByteSlice,
    utils::{f64_to_u64, parse_u64, split_on_new_line},
};
use memchr::{memchr, memchr2, memchr3};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

/// Describes what the library has been able to parse from the tag value.
///
/// The library makes a special attempt to parse decimal floats followed by a title (string slice),
/// which is the value that the `#EXTINF` tag uses, and also the attribute list type, which is used
/// by many of the HLS tags. Empty is trivially provided and everything else comes under the
/// `Unparsed` case, which itself provides helper methods for parsing. This design is a tradeoff
/// between functionality and performance. Internally, when walking through the value data, the
/// library only checks for a few tokens other than the end of line characters. If the end of line
/// is found instead of the searched for tokens (`,` and `=`) then the value is left as unparsed and
/// passed on to the next round of parsing. It is true that the next round will require to read the
/// value again; however, the [`UnparsedTagValue`] provides dedicated methods for parsing well
/// defined value types, which can be paired with the better context provided in the
/// `TryFrom<ParsedTag>` implementation of the tag to avoid needing to attempt several different
/// conversion strategies. This may change in the future, as per issue [#2].
///
/// [#2]: https://github.com/theRealRobG/m3u8/issues/2
#[derive(Debug, PartialEq)]
pub enum SemiParsedTagValue<'a> {
    /// The tag value was empty.
    ///
    /// For example, the `#EXTM3U` tag has an `Empty` value.
    Empty,
    /// The tag value was a float and a comma separated title.
    ///
    /// For example, the `#EXTINF:<duration>,[<title>]` tag has a
    /// `DecimalFloatingPointWithOptionalTitle` value (e.g. `#EXTINF:3.003,free-form text`).
    DecimalFloatingPointWithOptionalTitle(f64, &'a str),
    /// The tag value is an attribute list (that is, a comma separated list of key/value pairs, each
    /// separated by `=`).
    ///
    /// For example, the `#EXT-X-MAP:<attribute-list>` tag has an `AttributeList` value (e.g.
    /// `#EXT-X-MAP:URI="init.mp4"`).
    AttributeList(HashMap<&'a str, ParsedAttributeValue<'a>>),
    /// The tag value is unparsed; however, it is known not to be any of the other cases.
    ///
    /// The `UnparsedTagValue` provides methods for extracting known values out of the tag value.
    /// For example, the `#EXT-X-TARGETDURATION:<s>` tag has an `Unparsed` value (e.g.
    /// `#EXT-X-TARGETDURATION:10`, which could be accessed via the
    /// [`UnparsedTagValue::try_as_decimal_integer`] method).
    Unparsed(UnparsedTagValue<'a>),
}
impl<'a, T> From<(f64, T)> for SemiParsedTagValue<'a>
where
    T: Into<&'a str>,
{
    fn from(value: (f64, T)) -> Self {
        Self::DecimalFloatingPointWithOptionalTitle(value.0, value.1.into())
    }
}
impl<'a, K, V> From<HashMap<K, V>> for SemiParsedTagValue<'a>
where
    K: Into<&'a str>,
    V: Into<ParsedAttributeValue<'a>>,
{
    fn from(mut value: HashMap<K, V>) -> Self {
        let mut map = HashMap::new();
        for (key, value) in value.drain() {
            map.insert(key.into(), value.into());
        }
        Self::AttributeList(map)
    }
}
impl<'a, K, V, const N: usize> From<[(K, V); N]> for SemiParsedTagValue<'a>
where
    K: Into<&'a str>,
    V: Into<ParsedAttributeValue<'a>>,
{
    fn from(value: [(K, V); N]) -> Self {
        let mut map = HashMap::new();
        for (key, value) in value {
            map.insert(key.into(), value.into());
        }
        Self::AttributeList(map)
    }
}
impl<'a> From<&'a [u8]> for SemiParsedTagValue<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::Unparsed(UnparsedTagValue(value))
    }
}
/// A value that was not parsed in the initial round of parsing but does provide methods for
/// completing the parsing into a well defined type.
///
/// The documentation of [`SemiParsedTagValue`] provides more information on the reason that this
/// exists.
#[derive(Debug, PartialEq)]
pub struct UnparsedTagValue<'a>(pub &'a [u8]);
impl<'a> UnparsedTagValue<'a> {
    /// Attempts to read the inner data as a [`HlsPlaylistType`].
    pub fn try_as_hls_playlist_type(&self) -> Result<HlsPlaylistType, ParsePlaylistTypeError> {
        if self.0 == b"VOD" {
            Ok(HlsPlaylistType::Vod)
        } else if self.0 == b"EVENT" {
            Ok(HlsPlaylistType::Event)
        } else {
            Err(ParsePlaylistTypeError::InvalidValue)
        }
    }

    /// Attempts to read the inner data as a decimal integer.
    pub fn try_as_decimal_integer(&self) -> Result<u64, ParseNumberError> {
        parse_u64(self.0)
    }

    /// Attempts to read the inner data as a decimal integer range (`<n>[@<o>]`).
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

    /// Attempts to read the inner data as a float.
    pub fn try_as_float(&self) -> Result<f64, ParseFloatError> {
        fast_float2::parse(self.0).map_err(|_| ParseFloatError)
    }

    /// Attempts to read the inner data as a [`DateTime`].
    pub fn try_as_date_time(&self) -> Result<DateTime, DateTimeSyntaxError> {
        date::parse_bytes(self.0)
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

/// A parsed attribute list value.
///
/// This represents the value of an attribute in an attribute list, as defined in [Section 4.2].
///
/// [Section 4.2]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParsedAttributeValue<'a> {
    /// The parsed value "looks like" a decimal integer.
    ///
    /// NOTE: the parser does not have the context to determine whether the tag that contains the
    /// value should interpret this value as a `SignedDecimalFloatingPoint` or if it should be
    /// interpreted as is (a `DecimalInteger`). If the value is found without any fractional parts
    /// (no manitissa) then it is parsed as an integer.
    ///
    /// The [`ParsedAttributeValue::as_option_f64`] helps resolve this problem by taking both cases
    /// into consideration when trying to provide a float value.
    ///
    /// From [Section 4.2], this represents:
    /// * decimal-integer
    ///
    /// [Section 4.2]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2
    DecimalInteger(u64),
    /// The parsed value is a signed floating point number.
    ///
    /// From [Section 4.2], this represents:
    /// * decimal-floating-point
    /// * signed-decimal-floating-point
    ///
    /// [Section 4.2]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2
    SignedDecimalFloatingPoint(f64),
    /// The parsed value is a quoted string.
    ///
    /// From [Section 4.2], this represents:
    /// * quoted-string
    /// * enumerated-string-list
    ///
    /// [Section 4.2]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2
    QuotedString(&'a str),
    /// The parsed value is an unquoted string.
    ///
    /// From [Section 4.2], this represents:
    /// * hexadecimal-sequence
    /// * enumerated-string
    /// * decimal-resolution
    ///
    /// [Section 4.2]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2
    UnquotedString(&'a str),
}
impl From<u64> for ParsedAttributeValue<'_> {
    fn from(value: u64) -> Self {
        Self::DecimalInteger(value)
    }
}
impl From<f64> for ParsedAttributeValue<'_> {
    fn from(value: f64) -> Self {
        Self::SignedDecimalFloatingPoint(value)
    }
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
    /// attribute to understand whether the value MUST be a positive integer or whether it MAY be
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
impl<'a> From<SemiParsedTagValue<'a>> for MutableSemiParsedTagValue<'a> {
    fn from(value: SemiParsedTagValue<'a>) -> Self {
        match value {
            SemiParsedTagValue::Empty => Self::Empty,
            SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(f, t) => {
                Self::DecimalFloatingPointWithOptionalTitle(f, t.into())
            }
            SemiParsedTagValue::Unparsed(u) => Self::Unparsed(u.into()),
            SemiParsedTagValue::AttributeList(mut m) => {
                let mut map = HashMap::new();
                for (key, value) in m.drain() {
                    map.insert(key.into(), value.into());
                }
                Self::AttributeList(map)
            }
        }
    }
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
impl<'a> From<UnparsedTagValue<'a>> for MutableUnparsedTagValue<'a> {
    fn from(value: UnparsedTagValue<'a>) -> Self {
        Self(value.0.into())
    }
}

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
impl<'a> From<ParsedAttributeValue<'a>> for MutableParsedAttributeValue<'a> {
    fn from(value: ParsedAttributeValue<'a>) -> Self {
        match value {
            ParsedAttributeValue::DecimalInteger(d) => Self::DecimalInteger(d),
            ParsedAttributeValue::SignedDecimalFloatingPoint(d) => {
                Self::SignedDecimalFloatingPoint(d)
            }
            ParsedAttributeValue::QuotedString(s) => Self::QuotedString(s.into()),
            ParsedAttributeValue::UnquotedString(s) => Self::UnquotedString(s.into()),
        }
    }
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
            .map_err(DecimalResolutionParseError::InvalidWidth)?;
        let Some(height_str) = split.next() else {
            return Err(DecimalResolutionParseError::MissingHeight);
        };
        let height = height_str
            .parse()
            .map_err(DecimalResolutionParseError::InvalidHeight)?;
        Ok(DecimalResolution { width, height })
    }
}

/// Parses the input data as a tag value and provides a [`SemiParsedTagValue`] when successful.
///
/// This method is the primary function used to extract tag data from lines. This is what is used
/// when the [`crate::tag::known::CustomTag::is_known_name`] method returns `true`, and is also used
/// by all tag implementations offered by the library.
///
/// This method is more low level than a user would normally need to have; however, it may have some
/// use cases. For example, while it is more preferable to define a `CustomTag` implementation, it
/// is possible to find an `UnknownTag` and then use this method to parse out the value.
///
/// Another (perhaps interesting) use case for using this method directly can be to parse
/// information out of comment tags. For example, it has been noticed that the Unified Streaming
/// Packager seems to output a custom timestamp comment with its live playlists, that looks like a
/// tag; however, the library will not parse this as a tag because the syntax is
/// `#USP-X-TIMESTAMP-MAP:<attribute-list>`, so the lack of `#EXT` prefix means it is seen as a
/// comment only. Despite this, if we split on the `:`, we can use this method to extract
/// information about the value.
/// ```
/// # use m3u8::{
/// #     HlsLine, Reader,
/// #     config::ParsingOptions,
/// #     date, date_time,
/// #     line::ParsedByteSlice,
/// #     tag::value::{ParsedAttributeValue, SemiParsedTagValue, parse},
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
///         let ParsedByteSlice {
///             parsed,
///             remaining: _,
///         } = parse(value.as_bytes()).map_err(|e| format!("value parsing failed: {e}"))?;
///         let SemiParsedTagValue::AttributeList(list) = parsed else {
///             return Err(format!("unexpected tag value type").into());
///         };
///
///         // Prove that we can extract the value of MPEGTS
///         let Some(mpegts) = (match list.get("MPEGTS") {
///             Some(v) => v.as_option_u64(),
///             None => None,
///         }) else {
///             return Err(format!("missing required MPEGTS").into());
///         };
///         assert_eq!(900000, mpegts);
///
///         // Prove that we can extract the value of LOCAL
///         let Some(local) = (match list.get("LOCAL") {
///             Some(ParsedAttributeValue::UnquotedString(s)) => date::parse(s).ok(),
///             _ => None,
///         }) else {
///             return Err(format!("missing required LOCAL").into());
///         };
///         assert_eq!(date_time!(1970-01-01 T 00:00:00.000), local);
///     }
///     r => return Err(format!("unexpected result {r:?}").into()),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse(input: &[u8]) -> Result<ParsedByteSlice<SemiParsedTagValue>, TagValueSyntaxError> {
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
                    remaining,
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
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("VOD", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"VOD"))),
                parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_integer() {
        test_str_and_with_crlf_and_with_lf("42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42"))),
                parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_integer_range() {
        test_str_and_with_crlf_and_with_lf("42@42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42@42"))),
                parse(str).map(|p| p.parsed)
            );
        });
    }

    #[test]
    fn decimal_floating_point_with_optional_title() {
        // Positive tests
        test_str_and_with_crlf_and_with_lf("42.0", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42.0"))),
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42.42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42.42"))),
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42,", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.0, ""
                )),
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("42,=ATTRIBUTE-VALUE", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    42.0,
                    "=ATTRIBUTE-VALUE"
                )),
                parse(str).map(|p| p.parsed)
            );
        });
        // Negative tests
        test_str_and_with_crlf_and_with_lf("-42.0", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"-42.0"))),
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42.42", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(b"-42.42"))),
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42,", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    -42.0, ""
                )),
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("-42,=ATTRIBUTE-VALUE", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    -42.0,
                    "=ATTRIBUTE-VALUE"
                )),
                parse(str).map(|p| p.parsed)
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
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("2025-06-03T17:56:42.123+01:00", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(
                    b"2025-06-03T17:56:42.123+01:00"
                ))),
                parse(str).map(|p| p.parsed)
            );
        });
        test_str_and_with_crlf_and_with_lf("2025-06-03T17:56:42.123-05:00", |str| {
            assert_eq!(
                Ok(SemiParsedTagValue::Unparsed(UnparsedTagValue(
                    b"2025-06-03T17:56:42.123-05:00"
                ))),
                parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
                        parse(str).map(|p| p.parsed)
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
