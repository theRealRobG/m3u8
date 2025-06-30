use crate::{
    error::{UnknownTagSyntaxError, ValidationError},
    line::{ParsedByteSlice, ParsedLineSlice},
    utils::{split_on_new_line, str_from},
};
use memchr::memchr2;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Tag<'a> {
    pub(crate) name: &'a str,
    pub(crate) value: Option<&'a [u8]>,
    pub(crate) original_input: &'a [u8],
    pub(crate) validation_error: Option<ValidationError>,
}

impl Tag<'_> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.value
    }

    pub fn validation_error(&self) -> Option<ValidationError> {
        self.validation_error
    }

    pub fn as_bytes(&self) -> &[u8] {
        split_on_new_line(self.original_input).parsed
    }
}

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
                    value: Some(parsed),
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
            value: Some(b""),
            original_input: b"#EXT-X-TEST:",
            validation_error: None,
        };
        assert_eq!(Some(b"" as &[u8]), tag.value());
        assert_eq!(b"#EXT-X-TEST:", tag.as_bytes());
    }

    #[test]
    fn tag_value_some_when_remaining_is_some() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(b"42"),
            original_input: b"#EXT-X-TEST:42",
            validation_error: None,
        };
        assert_eq!(Some(b"42" as &[u8]), tag.value());
        assert_eq!(b"#EXT-X-TEST:42", tag.as_bytes());
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_crlf() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(b"42"),
            original_input: b"#EXT-X-TEST:42\r\n#EXT-X-NEW-TEST\r\n",
            validation_error: None,
        };
        assert_eq!(Some(b"42" as &[u8]), tag.value());
        assert_eq!(b"#EXT-X-TEST:42", tag.as_bytes());
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_lf() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(b"42"),
            original_input: b"#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n",
            validation_error: None,
        };
        assert_eq!(Some(b"42" as &[u8]), tag.value());
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
                    value: Some(b"42"),
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
                    value: Some(b"42"),
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
                    value: Some(b"42"),
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
                    value: Some(b"42"),
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
                    value: Some(b"42"),
                    original_input: b"#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n",
                    validation_error: None,
                },
                remaining: Some("#EXT-X-NEW-TEST\n")
            }),
            parse("#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n")
        );
    }
}
