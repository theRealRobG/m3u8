//! Types and operations for working with lines of a HLS playlist.
//!
//! This module includes various types and functions for working with lines of a HLS playlist. The
//! main informational type of the library, [`HlsLine`], exists in this module (and re-exported at
//! the top level), along with parsing functions to extract `HlsLine` from input data.

use crate::{
    config::ParsingOptions,
    error::{ParseLineBytesError, ParseLineStrError, SyntaxError},
    tag::{
        self, hls,
        known::{self, CustomTag, CustomTagAccess, NoCustomTag, ParsedTag},
        unknown,
    },
    utils::{split_on_new_line, str_from},
};
use std::{borrow::Cow, cmp::PartialEq, fmt::Debug};

/// A parsed line from a HLS playlist.
///
/// The HLS specification, in [Section 4.1. Definition of a Playlist], defines lines in a playlist
/// as such:
/// > Each line is a URI, is blank, or starts with the character '#'. Lines that start with the
/// > character '#' are either comments or tags. Tags begin with #EXT.
///
/// This data structure follows that guidance but also adds [`HlsLine::UnknownTag`] and
/// [`crate::tag::known::Tag::Custom`]. These cases are described in more detail within their own
/// documentation, but in short, the first allows us to capture tags that are not yet known to the
/// library (providing at least a split between name and value), while the second allows a user of
/// the library to define their own custom tag specification that can be then parsed into a strongly
/// typed structure within a `HlsLine::KnownTag` by the library.
///
/// [Section 4.1. Definition of a Playlist]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.1
#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)] // See comment on crate::tag::known::Tag.
pub enum HlsLine<'a, Custom = NoCustomTag>
where
    Custom: CustomTag<'a>,
{
    /// A tag known to the library, either via the included definitions of HLS tags as specified in
    /// the `draft-pantos-hls` Internet-Draft, or via a custom tag registration provided by the user
    /// of the library.
    ///
    /// See [`crate::tag::known::Tag`] for more information.
    KnownTag(known::Tag<'a, Custom>),
    /// A tag, as defined by the `#EXT` prefix, but not one that is known to the library, or that is
    /// deliberately ignored via [`crate::config::ParsingOptions`].
    ///
    /// See [`crate::tag::unknown::Tag`] for more information.
    UnknownTag(unknown::Tag<'a>),
    /// A comment line. These are lines that begin with `#` and are followed by a string of UTF-8
    /// characters (though not BOM or UTF-8 control characters). The line is terminated by either a
    /// line feed (`\n`) or a carriage return followed by a line feed (`\r\n`).
    ///
    /// The associated value is a [`std::borrow::Cow`] to allow for both a user constructed value
    /// and also a copy-free reference to the original parsed data. It includes all characters after
    /// the `#` (including any whitespace) and does not include the line break characters. Below
    /// demonstrates this:
    /// ```
    /// # use m3u8::{config::ParsingOptions, line::{HlsLine, parse}, error::ParseLineStrError};
    /// # use std::borrow::Cow;
    /// # let options = ParsingOptions::default();
    /// let original = "# Comment line. Note the leading space.\r\n";
    /// let line = parse(original, &options)?.parsed;
    /// assert_eq!(
    ///     HlsLine::Comment(Cow::Borrowed(" Comment line. Note the leading space.")),
    ///     line,
    /// );
    /// # Ok::<(), ParseLineStrError>(())
    /// ```
    Comment(Cow<'a, str>),
    /// A URI line. These are lines that do not begin with `#` and are not empty. It is important to
    /// note that the library does not do any validation on the line being a valid URI. The only
    /// validation that happens is that line can be represented as a UTF-8 string (internally we use
    /// [`std::str::from_utf8`]). This means that the line may contain characters that are invalid
    /// in a URI, or may otherwise not make sense in the context of the parsed playlist. It is up to
    /// the user of the library to validate the URI, perhaps using a URL parsing library (such as
    /// [url]).
    ///
    /// The associated value is a [`std::borrow::Cow`] to allow for both a user constructed value
    /// and also a copy-free reference to the original parsed data. It includes all characters up
    /// until, but not including, the line break characters. The following demonstrates this:
    /// ```
    /// # use m3u8::{config::ParsingOptions, line::{HlsLine, parse}, error::ParseLineStrError};
    /// # use std::borrow::Cow;
    /// # let options = ParsingOptions::default();
    /// let expected = "hi.m3u8";
    /// // Demonstrating that new line characters are not included:
    /// assert_eq!(
    ///     HlsLine::Uri(Cow::Borrowed(expected)),
    ///     parse("hi.m3u8\n", &options)?.parsed,
    /// );
    /// assert_eq!(
    ///     HlsLine::Uri(Cow::Borrowed(expected)),
    ///     parse("hi.m3u8\r\n", &options)?.parsed,
    /// );
    /// assert_eq!(
    ///     HlsLine::Uri(Cow::Borrowed(expected)),
    ///     parse("hi.m3u8", &options)?.parsed,
    /// );
    /// # Ok::<(), ParseLineStrError>(())
    /// ```
    ///
    /// [url]: https://crates.io/crates/url
    Uri(Cow<'a, str>),
    /// A blank line. This line contained no characters other than a new line. Note that since the
    /// library does not validate characters in a URI line, a line comprised entirely of whitespace
    /// will still be parsed as a URI line, rather than a blank line. As mentioned, it is up to the
    /// user of the library to properly validate URI lines.
    /// ```
    /// # use m3u8::{config::ParsingOptions, line::{HlsLine, parse}, error::ParseLineStrError};
    /// # use std::borrow::Cow;
    /// # let options = ParsingOptions::default();
    /// // Demonstrating what is considered a blank line:
    /// assert_eq!(
    ///     HlsLine::Blank,
    ///     parse("", &options)?.parsed,
    /// );
    /// assert_eq!(
    ///     HlsLine::Blank,
    ///     parse("\n", &options)?.parsed,
    /// );
    /// assert_eq!(
    ///     HlsLine::Blank,
    ///     parse("\r\n", &options)?.parsed,
    /// );
    /// // Demonstrating that a whitespace only line is still parsed as a URI:
    /// assert_eq!(
    ///     HlsLine::Uri(Cow::Borrowed("    ")),
    ///     parse("    \n", &options)?.parsed,
    /// );
    /// # Ok::<(), ParseLineStrError>(())
    /// ```
    Blank,
}

impl<'a, Custom> From<hls::Tag<'a>> for HlsLine<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    fn from(tag: hls::Tag<'a>) -> Self {
        Self::KnownTag(known::Tag::Hls(tag))
    }
}

impl<'a, Custom> From<CustomTagAccess<'a, Custom>> for HlsLine<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    fn from(tag: CustomTagAccess<'a, Custom>) -> Self {
        Self::KnownTag(known::Tag::Custom(tag))
    }
}

impl<'a, Custom> From<unknown::Tag<'a>> for HlsLine<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    fn from(tag: unknown::Tag<'a>) -> Self {
        Self::UnknownTag(tag)
    }
}

impl<'a> HlsLine<'a> {
    /// Convenience constructor for [`HlsLine::Comment`]. This will construct the line with the
    /// generic `Custom` in [`HlsLine::KnownTag`] being [`crate::tag::known::NoCustomTag`].
    pub fn comment(comment: impl Into<Cow<'a, str>>) -> Self {
        Self::Comment(comment.into())
    }

    /// Convenience constructor for [`HlsLine::Uri`]. This will construct the line with the generic
    /// `Custom` in [`HlsLine::KnownTag`] being [`crate::tag::known::NoCustomTag`].
    pub fn uri(uri: impl Into<Cow<'a, str>>) -> Self {
        Self::Uri(uri.into())
    }

    /// Convenience constructor for [`HlsLine::Blank`]. This will construct the line with the
    /// generic `Custom` in [`HlsLine::KnownTag`] being [`crate::tag::known::NoCustomTag`].
    pub fn blank() -> Self {
        Self::Blank
    }
}

macro_rules! impl_line_from_tag {
    ($tag_mod_path:path, $tag_name:ident) => {
        impl<'a, Custom> From<$tag_mod_path> for HlsLine<'a, Custom>
        where
            Custom: CustomTag<'a>,
        {
            fn from(tag: $tag_mod_path) -> Self {
                Self::KnownTag($crate::tag::known::Tag::Hls(
                    $crate::tag::hls::Tag::$tag_name(tag),
                ))
            }
        }
    };
}

impl_line_from_tag!(hls::M3u, M3u);
impl_line_from_tag!(hls::Version<'a>, Version);
impl_line_from_tag!(hls::IndependentSegments, IndependentSegments);
impl_line_from_tag!(hls::Start<'a>, Start);
impl_line_from_tag!(hls::Define<'a>, Define);
impl_line_from_tag!(hls::Targetduration<'a>, Targetduration);
impl_line_from_tag!(hls::MediaSequence<'a>, MediaSequence);
impl_line_from_tag!(hls::DiscontinuitySequence<'a>, DiscontinuitySequence);
impl_line_from_tag!(hls::Endlist, Endlist);
impl_line_from_tag!(hls::PlaylistType, PlaylistType);
impl_line_from_tag!(hls::IFramesOnly, IFramesOnly);
impl_line_from_tag!(hls::PartInf<'a>, PartInf);
impl_line_from_tag!(hls::ServerControl<'a>, ServerControl);
impl_line_from_tag!(hls::Inf<'a>, Inf);
impl_line_from_tag!(hls::Byterange<'a>, Byterange);
impl_line_from_tag!(hls::Discontinuity, Discontinuity);
impl_line_from_tag!(hls::Key<'a>, Key);
impl_line_from_tag!(hls::Map<'a>, Map);
impl_line_from_tag!(hls::ProgramDateTime<'a>, ProgramDateTime);
impl_line_from_tag!(hls::Gap, Gap);
impl_line_from_tag!(hls::Bitrate<'a>, Bitrate);
impl_line_from_tag!(hls::Part<'a>, Part);
impl_line_from_tag!(hls::Daterange<'a>, Daterange);
impl_line_from_tag!(hls::Skip<'a>, Skip);
impl_line_from_tag!(hls::PreloadHint<'a>, PreloadHint);
impl_line_from_tag!(hls::RenditionReport<'a>, RenditionReport);
impl_line_from_tag!(hls::Media<'a>, Media);
impl_line_from_tag!(hls::StreamInf<'a>, StreamInf);
impl_line_from_tag!(hls::IFrameStreamInf<'a>, IFrameStreamInf);
impl_line_from_tag!(hls::SessionData<'a>, SessionData);
impl_line_from_tag!(hls::SessionKey<'a>, SessionKey);
impl_line_from_tag!(hls::ContentSteering<'a>, ContentSteering);

/// A slice of parsed line data from a HLS playlist.
///
/// This struct allows us to parse some way into a playlist, breaking on the new line, and providing
/// the remaining characters after the new line in the [`Self::remaining`] field. This is a building
/// block type that is used by the [`crate::Reader`] to work through an input playlist with each
/// call to [`crate::Reader::read_line`].
#[derive(Debug, PartialEq, Clone)]
pub struct ParsedLineSlice<'a, T>
where
    T: Debug + PartialEq,
{
    /// The parsed data from the slice of line data from the playlist.
    pub parsed: T,
    /// The remaining string slice (after new line characters) from the playlist after parsing. If
    /// the parsed line was the last in the input data then the `remaining` is `None`.
    pub remaining: Option<&'a str>,
}
/// A slice of parsed line data from a HLS playlist.
///
/// This struct allows us to parse some way into a playlist, breaking on the new line, and providing
/// the remaining characters after the new line in the [`Self::remaining`] field. This is a building
/// block type that is used by the [`crate::Reader`] to work through an input playlist with each
/// call to [`crate::Reader::read_line`].
#[derive(Debug, PartialEq, Clone)]
pub struct ParsedByteSlice<'a, T>
where
    T: Debug + PartialEq,
{
    /// The parsed data from the slice of line data from the playlist.
    pub parsed: T,
    /// The remaining byte slice (after new line characters) from the playlist after parsing. If
    /// the parsed line was the last in the input data then the `remaining` is `None`.
    pub remaining: Option<&'a [u8]>,
}

/// Parse an input string slice with the provided options.
///
/// This method is a lower level method than using [`crate::Reader`] directly. The `Reader` uses
/// this method internally. It allows the user to parse a single line of HLS data and provides the
/// remaining data after the new line. Custom reader implementations can be built on top of this
/// method.
///
/// ## Example
/// ```
/// # use m3u8::{
/// # config::ParsingOptions,
/// # line::{HlsLine, ParsedLineSlice, parse},
/// # error::ParseLineStrError,
/// # tag::hls::{M3u, Targetduration, Version},
/// # };
/// const PLAYLIST: &str = r#"#EXTM3U
/// #EXT-X-TARGETDURATION:10
/// #EXT-X-VERSION:3
/// "#;
/// let options = ParsingOptions::default();
///
/// let ParsedLineSlice { parsed, remaining } = parse(PLAYLIST, &options)?;
/// assert_eq!(parsed, HlsLine::from(M3u));
///
/// let Some(remaining) = remaining else { return Ok(()) };
/// let ParsedLineSlice { parsed, remaining } = parse(remaining, &options)?;
/// assert_eq!(parsed, HlsLine::from(Targetduration::new(10)));
///
/// let Some(remaining) = remaining else { return Ok(()) };
/// let ParsedLineSlice { parsed, remaining } = parse(remaining, &options)?;
/// assert_eq!(parsed, HlsLine::from(Version::new(3)));
///
/// let Some(remaining) = remaining else { return Ok(()) };
/// let ParsedLineSlice { parsed, remaining } = parse(remaining, &options)?;
/// assert_eq!(parsed, HlsLine::Blank);
/// assert_eq!(remaining, None);
/// # Ok::<(), ParseLineStrError>(())
/// ```
pub fn parse<'a>(
    input: &'a str,
    options: &ParsingOptions,
) -> Result<ParsedLineSlice<'a, HlsLine<'a>>, ParseLineStrError<'a>> {
    parse_with_custom::<NoCustomTag>(input, options)
}

/// Parse an input string slice with the provided options with support for the provided custom tag.
///
/// This method is a lower level method than using [`crate::Reader`] directly. The `Reader` uses
/// this method internally. It allows the user to parse a single line of HLS data and provides the
/// remaining data after the new line. Custom reader implementations can be built on top of this
/// method. This method differs from [`parse`] as it allows the user to provide their own custom tag
/// implementation for parsing.
///
/// ## Example
/// ```
/// # use m3u8::{
/// # config::ParsingOptions,
/// # line::{HlsLine, ParsedLineSlice, parse_with_custom},
/// # error::{ParseLineStrError, ValidationError, ValidationErrorValueKind},
/// # tag::known::{Tag, CustomTag, ParsedTag},
/// # tag::value::{ParsedAttributeValue, SemiParsedTagValue},
/// # tag::hls::{M3u, Targetduration, Version},
/// # };
/// #[derive(Debug, Clone, PartialEq)]
/// struct UserDefinedTag<'a> {
///     message: &'a str,
/// }
/// impl<'a> TryFrom<ParsedTag<'a>> for UserDefinedTag<'a> { // --snip--
/// #    type Error = ValidationError;
/// #    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
/// #        let SemiParsedTagValue::AttributeList(mut list) = tag.value else {
/// #            return Err(ValidationError::UnexpectedValueType(
/// #                ValidationErrorValueKind::from(&tag.value),
/// #            ));
/// #        };
/// #        let Some(ParsedAttributeValue::QuotedString(message)) = list.remove("MESSAGE") else {
/// #            return Err(ValidationError::MissingRequiredAttribute("MESSAGE"));
/// #        };
/// #        Ok(Self { message })
/// #    }
/// }
/// impl<'a> CustomTag<'a> for UserDefinedTag<'a> { // --snip--
/// #    fn is_known_name(name: &str) -> bool {
/// #        name == "-X-USER-DEFINED-TAG"
/// #    }
/// }
///
/// const PLAYLIST: &str = r#"#EXTM3U
/// #EXT-X-USER-DEFINED-TAG:MESSAGE="Hello, World!"
/// "#;
/// let options = ParsingOptions::default();
///
/// let ParsedLineSlice {
///     parsed,
///     remaining
/// } = parse_with_custom::<UserDefinedTag>(PLAYLIST, &options)?;
/// assert_eq!(parsed, HlsLine::from(M3u));
///
/// let Some(remaining) = remaining else { return Ok(()) };
/// let ParsedLineSlice {
///     parsed,
///     remaining
/// } = parse_with_custom::<UserDefinedTag>(remaining, &options)?;
/// let HlsLine::KnownTag(Tag::Custom(tag)) = parsed else { return Ok(()) };
/// assert_eq!(tag.as_ref(), &UserDefinedTag { message: "Hello, World!" });
///
/// let Some(remaining) = remaining else { return Ok(()) };
/// let ParsedLineSlice {
///     parsed,
///     remaining
/// } = parse_with_custom::<UserDefinedTag>(remaining, &options)?;
/// assert_eq!(parsed, HlsLine::Blank);
/// assert_eq!(remaining, None);
/// # Ok::<(), ParseLineStrError>(())
/// ```
pub fn parse_with_custom<'a, 'b, Custom>(
    input: &'a str,
    options: &'b ParsingOptions,
) -> Result<ParsedLineSlice<'a, HlsLine<'a, Custom>>, ParseLineStrError<'a>>
where
    Custom: CustomTag<'a>,
{
    parse_bytes_with_custom(input.as_bytes(), options)
        // These conversions from ParsedByteSlice to ParsedLineSlice are only safe here because we
        // know that these must represent valid UTF-8.
        .map(|r| ParsedLineSlice {
            parsed: r.parsed,
            remaining: r.remaining.map(str_from),
        })
        .map_err(|error| ParseLineStrError {
            errored_line_slice: ParsedLineSlice {
                parsed: str_from(error.errored_line_slice.parsed),
                remaining: error.errored_line_slice.remaining.map(str_from),
            },
            error: error.error,
        })
}

/// Parse an input byte slice with the provided options.
///
/// This method is equivalent to [`parse`] but using `&[u8]` instead of `&str`. Refer to
/// documentation of [`parse`] for more information.
pub fn parse_bytes<'a>(
    input: &'a [u8],
    options: &ParsingOptions,
) -> Result<ParsedByteSlice<'a, HlsLine<'a>>, ParseLineBytesError<'a>> {
    parse_bytes_with_custom::<NoCustomTag>(input, options)
}

/// Parse an input byte slice with the provided options with support for the provided custom tag.
///
/// This method is equivalent to [`parse_with_custom`] but using `&[u8]` instead of `&str`. Refer to
/// documentation of [`parse_with_custom`] for more information.
pub fn parse_bytes_with_custom<'a, 'b, Custom>(
    input: &'a [u8],
    options: &'b ParsingOptions,
) -> Result<ParsedByteSlice<'a, HlsLine<'a, Custom>>, ParseLineBytesError<'a>>
where
    Custom: CustomTag<'a>,
{
    if input.is_empty() {
        Ok(ParsedByteSlice {
            parsed: HlsLine::Blank,
            remaining: None,
        })
    } else if input[0] == b'#' {
        if input.get(3) == Some(&b'T') && &input[..3] == b"#EX" {
            let tag_rest = &input[4..];
            let mut tag = tag::unknown::parse_assuming_ext_taken(tag_rest, input)
                .map_err(|error| map_err_bytes(error, input))?;
            if options.is_known_name(tag.parsed.name) || Custom::is_known_name(tag.parsed.name) {
                let value_slice = match tag.parsed.value {
                    None => ParsedByteSlice {
                        parsed: tag::value::SemiParsedTagValue::Empty,
                        remaining: None,
                    },
                    Some(remaining) => tag::value::new_parse(remaining)
                        .map_err(|error| map_err_bytes(error, input))?,
                };
                let parsed_tag = ParsedTag {
                    name: tag.parsed.name,
                    value: value_slice.parsed,
                    original_input: input,
                };
                match known::Tag::try_from(parsed_tag) {
                    Ok(known_tag) => Ok(ParsedByteSlice {
                        parsed: HlsLine::KnownTag(known_tag),
                        remaining: tag.remaining,
                    }),
                    Err(e) => {
                        tag.parsed.validation_error = Some(e);
                        Ok(ParsedByteSlice {
                            parsed: HlsLine::UnknownTag(tag.parsed),
                            remaining: tag.remaining,
                        })
                    }
                }
            } else {
                Ok(ParsedByteSlice {
                    parsed: HlsLine::UnknownTag(tag.parsed),
                    remaining: tag.remaining,
                })
            }
        } else {
            let ParsedByteSlice { parsed, remaining } = split_on_new_line(&input[1..]);
            let comment =
                std::str::from_utf8(parsed).map_err(|error| map_err_bytes(error, input))?;
            Ok(ParsedByteSlice {
                parsed: HlsLine::Comment(Cow::Borrowed(comment)),
                remaining,
            })
        }
    } else {
        let ParsedByteSlice { parsed, remaining } = split_on_new_line(input);
        let uri = std::str::from_utf8(parsed).map_err(|error| map_err_bytes(error, input))?;
        if uri.is_empty() {
            Ok(ParsedByteSlice {
                parsed: HlsLine::Blank,
                remaining,
            })
        } else {
            Ok(ParsedByteSlice {
                parsed: HlsLine::Uri(Cow::Borrowed(uri)),
                remaining,
            })
        }
    }
}

fn map_err_bytes<E: Into<SyntaxError>>(error: E, input: &[u8]) -> ParseLineBytesError {
    let errored_line_slice = split_on_new_line(input);
    ParseLineBytesError {
        errored_line_slice,
        error: error.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::ParsingOptionsBuilder,
        error::{ValidationError, ValidationErrorValueKind},
        tag::hls::{self, M3u, Start},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn uri_line() {
        assert_eq!(
            Ok(HlsLine::Uri("hello/world.m3u8".into())),
            parse("hello/world.m3u8", &ParsingOptions::default()).map(|p| p.parsed)
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(
            Ok(HlsLine::Blank),
            parse("", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn comment() {
        assert_eq!(
            Ok(HlsLine::Comment("Comment".into())),
            parse("#Comment", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok(HlsLine::from(hls::Tag::M3u(M3u))),
            parse("#EXTM3U", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn custom_tag() {
        // Set up custom tag
        #[derive(Debug, PartialEq, Clone)]
        struct TestTag<'a> {
            greeting_type: &'a str,
            message: &'a str,
            times: u64,
            score: Option<f64>,
        }
        impl<'a> TryFrom<ParsedTag<'a>> for TestTag<'a> {
            type Error = ValidationError;

            fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
                match &tag.value {
                    tag::value::SemiParsedTagValue::AttributeList(list) => {
                        let Some(tag::value::ParsedAttributeValue::UnquotedString(greeting_type)) =
                            list.get("TYPE")
                        else {
                            return Err(ValidationError::MissingRequiredAttribute("TYPE"));
                        };
                        let Some(tag::value::ParsedAttributeValue::QuotedString(message)) =
                            list.get("MESSAGE")
                        else {
                            return Err(ValidationError::MissingRequiredAttribute("MESSAGE"));
                        };
                        let Some(tag::value::ParsedAttributeValue::DecimalInteger(times)) =
                            list.get("TIMES")
                        else {
                            return Err(ValidationError::MissingRequiredAttribute("TIMES"));
                        };
                        let score = list
                            .get("SCORE")
                            .map(tag::value::ParsedAttributeValue::as_option_f64)
                            .flatten();
                        Ok(Self {
                            greeting_type,
                            message,
                            times: *times,
                            score,
                        })
                    }
                    v => Err(ValidationError::UnexpectedValueType(
                        ValidationErrorValueKind::from(v),
                    )),
                }
            }
        }
        impl CustomTag<'static> for TestTag<'static> {
            fn is_known_name(name: &str) -> bool {
                name == "-X-TEST-TAG"
            }
        }
        // Test
        assert_eq!(
            Ok(HlsLine::from(CustomTagAccess {
                custom_tag: TestTag {
                    greeting_type: "GREETING".into(),
                    message: "Hello, World!".into(),
                    times: 42,
                    score: None,
                },
                is_dirty: false,
                original_input: b"#EXT-X-TEST-TAG:TYPE=GREETING,MESSAGE=\"Hello, World!\",TIMES=42"
            })),
            parse_with_custom::<TestTag>(
                "#EXT-X-TEST-TAG:TYPE=GREETING,MESSAGE=\"Hello, World!\",TIMES=42",
                &ParsingOptions::default()
            )
            .map(|p| p.parsed)
        );
    }

    #[test]
    fn avoiding_parsing_known_tag_when_configured_to_avoid_via_parsing_options() {
        assert_eq!(
            Ok(HlsLine::from(hls::Tag::Start(
                Start::builder(-18.0).finish()
            ))),
            parse("#EXT-X-START:TIME-OFFSET=-18", &ParsingOptions::default()).map(|p| p.parsed)
        );
        assert_eq!(
            Ok(HlsLine::UnknownTag(unknown::Tag {
                name: "-X-START",
                value: Some(b"TIME-OFFSET=-18"),
                original_input: b"#EXT-X-START:TIME-OFFSET=-18",
                validation_error: None,
            })),
            parse(
                "#EXT-X-START:TIME-OFFSET=-18",
                &ParsingOptionsBuilder::new()
                    .with_parsing_for_all_tags()
                    .without_parsing_for_start()
                    .build()
            )
            .map(|p| p.parsed)
        );
    }

    #[test]
    fn empty_line_before_new_line_break_should_be_parsed_as_blank() {
        let input = "\n#something else";
        assert_eq!(
            ParsedLineSlice {
                parsed: HlsLine::Blank,
                remaining: Some("#something else")
            },
            parse(input, &ParsingOptions::default()).unwrap()
        );
    }
}
