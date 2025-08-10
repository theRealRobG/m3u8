//! Methods for parsing unknown tag information
//!
//! This module also serves as a building block for the parsing of all known tags. Before a tag is
//! parsed as known, it is first parsed as unknown, and then we attempt to specialize it. Known tags
//! can also fall back to unknown tags if there is some issue in validating the strong type
//! requirements of the tag.

use crate::{
    error::{UnknownTagSyntaxError, ValidationError},
    line::{ParsedByteSlice, ParsedLineSlice},
    tag::value::TagValue,
    utils::{split_on_new_line, str_from},
};
use memchr::memchr2;
use std::fmt::Debug;

/// A tag that is unknown to the library found during parsing input data.
///
/// This may be because the tag is truly unknown (i.e., is not one of the 32 supported HLS defined
/// tags), or because the known tag has been ignored via [`crate::config::ParsingOptions`], or also
/// if there was an error in parsing the known tag. In the last case, the [`Self::validation_error`]
/// will provide details on the problem encountered.
///
/// For example:
/// ```
/// # use m3u8::{Reader, HlsLine, config::ParsingOptionsBuilder, error::ValidationError};
/// let lines = r#"#EXT-X-QUESTION:VALUE="Do you know who I am?"
/// #EXT-X-PROGRAM-DATE-TIME:2025-08-05T21:59:42.417-05:00
/// #EXT-X-STREAM-INF:AVERAGE-BANDWIDTH=10000000"#;
///
/// let mut reader = Reader::from_str(
///     lines,
///     ParsingOptionsBuilder::new()
///         .with_parsing_for_stream_inf()
///         .build()
/// );
///
/// // #EXT-X-QUESTION:VALUE="Do you know who I am?"
/// let Ok(Some(HlsLine::UnknownTag(tag))) = reader.read_line() else { panic!("unexpected tag") };
/// assert_eq!("-X-QUESTION", tag.name());
/// assert_eq!(Some(r#"VALUE="Do you know who I am?""#.as_bytes()), tag.value());
/// assert_eq!(None, tag.validation_error());
/// assert_eq!(r#"#EXT-X-QUESTION:VALUE="Do you know who I am?""#.as_bytes(), tag.as_bytes());
///
/// // #EXT-X-PROGRAM-DATE-TIME:2025-08-05T21:59:42.417-05:00
/// let Ok(Some(HlsLine::UnknownTag(tag))) = reader.read_line() else { panic!("unexpected tag") };
/// assert_eq!("-X-PROGRAM-DATE-TIME", tag.name());
/// assert_eq!(Some("2025-08-05T21:59:42.417-05:00".as_bytes()), tag.value());
/// assert_eq!(None, tag.validation_error());
/// assert_eq!(
///     "#EXT-X-PROGRAM-DATE-TIME:2025-08-05T21:59:42.417-05:00".as_bytes(),
///     tag.as_bytes()
/// );
///
/// // #EXT-X-STREAM-INF:AVERAGE-BANDWIDTH=10000000
/// let Ok(Some(HlsLine::UnknownTag(tag))) = reader.read_line() else { panic!("unexpected tag") };
/// assert_eq!("-X-STREAM-INF", tag.name());
/// assert_eq!(Some("AVERAGE-BANDWIDTH=10000000".as_bytes()), tag.value());
/// assert_eq!(
///     Some(ValidationError::MissingRequiredAttribute("BANDWIDTH")),
///     tag.validation_error()
/// );
/// assert_eq!(
///     "#EXT-X-STREAM-INF:AVERAGE-BANDWIDTH=10000000".as_bytes(),
///     tag.as_bytes()
/// );
/// ```
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Tag<'a> {
    pub(crate) name: &'a str,
    pub(crate) value: Option<TagValue<'a>>,
    pub(crate) original_input: &'a [u8],
    pub(crate) validation_error: Option<ValidationError>,
}

impl<'a> Tag<'a> {
    /// The name of the unknown tag.
    ///
    /// This includes everything after the `#EXT` prefix and before the `:` or new line. For
    /// example, `#EXTM3U` has name `M3U`, `#EXT-X-VERSION:3` has name `-X-VERSION`, etc.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// The value of the unknown tag.
    ///
    /// This will be the entire byte-slice after the first `:` in the line. If there is no `:` then
    /// this will be `None`.
    pub fn value(&self) -> Option<TagValue<'a>> {
        self.value
    }

    /// The error that led to this tag being unknown.
    ///
    /// This value is only `Some` if the tag is unknown as the result of a problem in parsing a
    /// known tag.
    pub fn validation_error(&self) -> Option<ValidationError> {
        self.validation_error
    }

    /// The raw bytes of the tag line for output.
    ///
    /// This is useful for when the tag needs to be writtern to an output.
    pub fn as_bytes(&self) -> &'a [u8] {
        split_on_new_line(self.original_input).parsed
    }
}

/// Try to parse some input into a tag.
///
/// The parsing will stop at the new line. Failures are described via [`UnknownTagSyntaxError`].
/// This method is at the root of parsing in this library and what other higher level types are
/// built on top of. It helps by splitting the input on a new line and providing a name and value
/// slice for the line we are parsing (assuming it is a tag line).
pub fn parse(input: &str) -> Result<ParsedLineSlice<Tag>, UnknownTagSyntaxError> {
    let input = input.as_bytes();
    if input.get(3) == Some(&b'T') && &input[..3] == b"#EX" {
        let ParsedByteSlice { parsed, remaining } = parse_assuming_ext_taken(&input[4..], input)?;
        Ok(ParsedLineSlice {
            parsed,
            remaining: remaining.map(str_from),
        })
    } else {
        Err(UnknownTagSyntaxError::InvalidTag)
    }
}

pub(crate) fn parse_assuming_ext_taken<'a>(
    input: &'a [u8],
    original_input: &'a [u8],
) -> Result<ParsedByteSlice<'a, Tag<'a>>, UnknownTagSyntaxError> {
    if input.is_empty() || input[0] == b'\n' || input[0] == b'\r' {
        return Err(UnknownTagSyntaxError::UnexpectedNoTagName);
    };
    match memchr2(b':', b'\n', input) {
        Some(n) if input[n] == b':' => {
            let name = std::str::from_utf8(&input[..n])?;
            let ParsedByteSlice { parsed, remaining } = split_on_new_line(&input[(n + 1)..]);
            Ok(ParsedByteSlice {
                parsed: Tag {
                    name,
                    value: Some(TagValue(parsed)),
                    original_input,
                    validation_error: None,
                },
                remaining,
            })
        }
        Some(n) if input[n - 1] == b'\r' => {
            let name = std::str::from_utf8(&input[..(n - 1)])?;
            Ok(ParsedByteSlice {
                parsed: Tag {
                    name,
                    value: None,
                    original_input,
                    validation_error: None,
                },
                remaining: Some(&input[(n + 1)..]),
            })
        }
        Some(n) => {
            let name = std::str::from_utf8(&input[..n])?;
            Ok(ParsedByteSlice {
                parsed: Tag {
                    name,
                    value: None,
                    original_input,
                    validation_error: None,
                },
                remaining: Some(&input[(n + 1)..]),
            })
        }
        None => {
            let name = std::str::from_utf8(input)?;
            Ok(ParsedByteSlice {
                parsed: Tag {
                    name,
                    value: None,
                    original_input,
                    validation_error: None,
                },
                remaining: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tag_value_empty_when_remaining_none() {
        let tag = Tag {
            name: "-X-TEST",
            value: None,
            original_input: b"#EXT-X-TEST",
            validation_error: None,
        };
        assert_eq!(None, tag.value());
        assert_eq!(b"#EXT-X-TEST", tag.as_bytes());
    }

    #[test]
    fn tag_value_empty_when_remaining_is_empty() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(TagValue(b"")),
            original_input: b"#EXT-X-TEST:",
            validation_error: None,
        };
        assert_eq!(Some(TagValue(b"")), tag.value());
        assert_eq!(b"#EXT-X-TEST:", tag.as_bytes());
    }

    #[test]
    fn tag_value_some_when_remaining_is_some() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(TagValue(b"42")),
            original_input: b"#EXT-X-TEST:42",
            validation_error: None,
        };
        assert_eq!(Some(TagValue(b"42")), tag.value());
        assert_eq!(b"#EXT-X-TEST:42", tag.as_bytes());
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_crlf() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(TagValue(b"42")),
            original_input: b"#EXT-X-TEST:42\r\n#EXT-X-NEW-TEST\r\n",
            validation_error: None,
        };
        assert_eq!(Some(TagValue(b"42")), tag.value());
        assert_eq!(b"#EXT-X-TEST:42", tag.as_bytes());
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_lf() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(TagValue(b"42")),
            original_input: b"#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n",
            validation_error: None,
        };
        assert_eq!(Some(TagValue(b"42")), tag.value());
        assert_eq!(b"#EXT-X-TEST:42", tag.as_bytes());
    }

    #[test]
    fn parses_tag_with_no_value() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    value: None,
                    original_input: b"#EXT-TEST-TAG",
                    validation_error: None,
                },
                remaining: None
            }),
            parse("#EXT-TEST-TAG")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    value: None,
                    original_input: b"#EXT-TEST-TAG\r\n",
                    validation_error: None,
                },
                remaining: Some("")
            }),
            parse("#EXT-TEST-TAG\r\n")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    value: None,
                    original_input: b"#EXT-TEST-TAG\n",
                    validation_error: None,
                },
                remaining: Some("")
            }),
            parse("#EXT-TEST-TAG\n")
        );
    }

    #[test]
    fn parses_tag_with_value() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    value: Some(TagValue(b"42")),
                    original_input: b"#EXT-TEST-TAG:42",
                    validation_error: None,
                },
                remaining: None
            }),
            parse("#EXT-TEST-TAG:42")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    value: Some(TagValue(b"42")),
                    original_input: b"#EXT-TEST-TAG:42\r\n",
                    validation_error: None,
                },
                remaining: Some("")
            }),
            parse("#EXT-TEST-TAG:42\r\n")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    value: Some(TagValue(b"42")),
                    original_input: b"#EXT-TEST-TAG:42\n",
                    validation_error: None,
                },
                remaining: Some("")
            }),
            parse("#EXT-TEST-TAG:42\n")
        );
    }

    #[test]
    fn parse_remaining_is_some_when_split_by_crlf() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-X-TEST",
                    value: Some(TagValue(b"42")),
                    original_input: b"#EXT-X-TEST:42\r\n#EXT-X-NEW-TEST\r\n",
                    validation_error: None,
                },
                remaining: Some("#EXT-X-NEW-TEST\r\n")
            }),
            parse("#EXT-X-TEST:42\r\n#EXT-X-NEW-TEST\r\n")
        );
    }

    #[test]
    fn parse_remaining_is_some_when_split_by_lf() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-X-TEST",
                    value: Some(TagValue(b"42")),
                    original_input: b"#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n",
                    validation_error: None,
                },
                remaining: Some("#EXT-X-NEW-TEST\n")
            }),
            parse("#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n")
        );
    }
}
