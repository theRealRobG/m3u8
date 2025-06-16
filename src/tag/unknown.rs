use crate::{
    line::ParsedLineSlice,
    utils::{split_by_first_lf, str_from},
};
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    pub name: &'a str,
    pub(crate) remaining: Option<&'a str>,
    pub(crate) original_input: &'a str,
}

impl<'a> Tag<'a> {
    pub fn value(&self) -> ParsedLineSlice<'a, Option<&'a str>> {
        let Some(remaining) = self.remaining else {
            return ParsedLineSlice {
                parsed: None,
                remaining: None,
            };
        };
        let split = split_by_first_lf(remaining);
        ParsedLineSlice {
            parsed: Some(split.parsed),
            remaining: split.remaining,
        }
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(self.original_input).parsed
    }
}

pub fn parse(input: &str) -> Result<ParsedLineSlice<Tag>, &'static str> {
    let mut bytes = input.as_bytes().iter();
    let Some(b'#') = bytes.next() else {
        return Err("Not a tag");
    };
    let Some(b'E') = bytes.next() else {
        return Err("Not a tag");
    };
    let Some(b'X') = bytes.next() else {
        return Err("Not a tag");
    };
    let Some(b'T') = bytes.next() else {
        return Err("Not a tag");
    };
    parse_assuming_ext_taken(str_from(bytes.as_slice()), input)
}

pub(crate) fn parse_assuming_ext_taken<'a>(
    input: &'a str,
    original_input: &'a str,
) -> Result<ParsedLineSlice<'a, Tag<'a>>, &'static str> {
    if input.is_empty() {
        return Err("Unexpected empty input for parsing tag name");
    };
    let mut bytes = input.as_bytes().iter();
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        let Some(byte) = bytes.next() else {
            let name = &input[..(iterations - 1)];
            let remaining = None;
            return Ok(ParsedLineSlice {
                parsed: Tag {
                    name,
                    remaining,
                    original_input,
                },
                remaining,
            });
        };
        match byte {
            b':' | b'\n' => {
                let name = &input[..(iterations - 1)];
                let remaining = Some(str_from(bytes.as_slice()));
                return Ok(ParsedLineSlice {
                    parsed: Tag {
                        name,
                        remaining,
                        original_input,
                    },
                    remaining,
                });
            }
            b'\r' => {
                let Some(b'\n') = bytes.next() else {
                    return Err("Unsupported carriage return without line feed");
                };
                let name = &input[..(iterations - 1)];
                let remaining = Some(str_from(bytes.as_slice()));
                return Ok(ParsedLineSlice {
                    parsed: Tag {
                        name,
                        remaining,
                        original_input,
                    },
                    remaining,
                });
            }
            _ => (),
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
            remaining: None,
            original_input: "#EXT-X-TEST",
        };
        assert_eq!(
            ParsedLineSlice {
                parsed: None,
                remaining: None
            },
            tag.value()
        );
    }

    #[test]
    fn tag_value_empty_when_remaining_is_empty() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some(""),
            original_input: "#EXT-X-TEST:",
        };
        assert_eq!(
            ParsedLineSlice {
                parsed: Some(""),
                remaining: None
            },
            tag.value()
        );
    }

    #[test]
    fn tag_value_some_when_remaining_is_some() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42"),
            original_input: "#EXT-X-TEST:42",
        };
        assert_eq!(
            ParsedLineSlice {
                parsed: Some("42"),
                remaining: None
            },
            tag.value()
        );
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_crlf() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42\r\n#EXT-X-NEW-TEST\r\n"),
            original_input: "#EXT-X-TEST:42\r\n#EXT-X-NEW-TEST\r\n",
        };
        assert_eq!(
            ParsedLineSlice {
                parsed: Some("42"),
                remaining: Some("#EXT-X-NEW-TEST\r\n")
            },
            tag.value()
        );
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_lf() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42\n#EXT-X-NEW-TEST\n"),
            original_input: "#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n",
        };
        assert_eq!(
            ParsedLineSlice {
                parsed: Some("42"),
                remaining: Some("#EXT-X-NEW-TEST\n")
            },
            tag.value()
        );
    }

    #[test]
    fn parses_tag_with_no_value() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: None,
                    original_input: "#EXT-TEST-TAG",
                },
                remaining: None
            }),
            parse("#EXT-TEST-TAG")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some(""),
                    original_input: "#EXT-TEST-TAG\r\n",
                },
                remaining: Some("")
            }),
            parse("#EXT-TEST-TAG\r\n")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some(""),
                    original_input: "#EXT-TEST-TAG\n",
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
                    remaining: Some("42"),
                    original_input: "#EXT-TEST-TAG:42",
                },
                remaining: Some("42")
            }),
            parse("#EXT-TEST-TAG:42")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some("42\r\n"),
                    original_input: "#EXT-TEST-TAG:42\r\n",
                },
                remaining: Some("42\r\n")
            }),
            parse("#EXT-TEST-TAG:42\r\n")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some("42\n"),
                    original_input: "#EXT-TEST-TAG:42\n",
                },
                remaining: Some("42\n")
            }),
            parse("#EXT-TEST-TAG:42\n")
        );
    }
}
