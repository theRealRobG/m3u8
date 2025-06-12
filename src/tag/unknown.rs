use crate::{line::ParsedLineSlice, utils::str_from};
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    pub name: &'a str,
    remaining: Option<&'a str>,
}

impl<'a> Tag<'a> {
    pub fn new(name: &'a str, remaining: Option<&'a str>) -> Self {
        Self { name, remaining }
    }

    pub fn value(&self) -> Result<ParsedLineSlice<'a, Option<&'a str>>, &'static str> {
        let Some(remaining) = self.remaining else {
            return Ok(ParsedLineSlice {
                parsed: None,
                remaining: None,
            });
        };
        let mut bytes = remaining.as_bytes().iter();
        let mut iterations = 0usize;
        loop {
            iterations += 1;
            let Some(char) = bytes.next() else {
                return Ok(ParsedLineSlice {
                    parsed: Some(&remaining[..(iterations - 1)]),
                    remaining: None,
                });
            };
            match char {
                b'\r' => {
                    let Some(b'\n') = bytes.next() else {
                        return Err("Unsupported carriage return without line feed");
                    };
                    return Ok(ParsedLineSlice {
                        parsed: Some(&remaining[..(iterations - 1)]),
                        remaining: Some(str_from(bytes.as_slice())),
                    });
                }
                b'\n' => {
                    return Ok(ParsedLineSlice {
                        parsed: Some(&remaining[..(iterations - 1)]),
                        remaining: Some(str_from(bytes.as_slice())),
                    });
                }
                _ => (),
            }
        }
    }
}

pub fn parse(input: &str) -> Result<ParsedLineSlice<Tag>, &'static str> {
    if input.is_empty() {
        return Err("Unexpected empty input for parsing tag name");
    };
    let mut chars = input.chars();
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        let Some(char) = chars.next() else {
            let name = &input[..(iterations - 1)];
            let remaining = None;
            return Ok(ParsedLineSlice {
                parsed: Tag { name, remaining },
                remaining,
            });
        };
        match char {
            ':' | '\n' => {
                let name = &input[..(iterations - 1)];
                let remaining = Some(chars.as_str());
                return Ok(ParsedLineSlice {
                    parsed: Tag { name, remaining },
                    remaining,
                });
            }
            '\r' => {
                let Some('\n') = chars.next() else {
                    return Err("Unsupported carriage return without line feed");
                };
                let name = &input[..(iterations - 1)];
                let remaining = Some(chars.as_str());
                return Ok(ParsedLineSlice {
                    parsed: Tag { name, remaining },
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
        };
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: None,
                remaining: None
            }),
            tag.value()
        );
    }

    #[test]
    fn tag_value_empty_when_remaining_is_empty() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some(""),
        };
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Some(""),
                remaining: None
            }),
            tag.value()
        );
    }

    #[test]
    fn tag_value_some_when_remaining_is_some() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42"),
        };
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Some("42"),
                remaining: None
            }),
            tag.value()
        );
    }

    #[test]
    fn tag_value_err_when_cr_with_no_lf() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42\r"),
        };
        assert_eq!(
            Err("Unsupported carriage return without line feed"),
            tag.value()
        );
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42\r#EXT-X-NEW-TEST"),
        };
        assert_eq!(
            Err("Unsupported carriage return without line feed"),
            tag.value()
        );
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_crlf() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42\r\n#EXT-X-NEW-TEST\r\n"),
        };
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Some("42"),
                remaining: Some("#EXT-X-NEW-TEST\r\n")
            }),
            tag.value()
        );
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_lf() {
        let tag = Tag {
            name: "-X-TEST",
            remaining: Some("42\n#EXT-X-NEW-TEST\n"),
        };
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Some("42"),
                remaining: Some("#EXT-X-NEW-TEST\n")
            }),
            tag.value()
        );
    }

    #[test]
    fn parses_tag_with_no_value() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: None
                },
                remaining: None
            }),
            parse("-TEST-TAG")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some("")
                },
                remaining: Some("")
            }),
            parse("-TEST-TAG\r\n")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some("")
                },
                remaining: Some("")
            }),
            parse("-TEST-TAG\n")
        );
    }

    #[test]
    fn parses_tag_with_value() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some("42")
                },
                remaining: Some("42")
            }),
            parse("-TEST-TAG:42")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some("42\r\n")
                },
                remaining: Some("42\r\n")
            }),
            parse("-TEST-TAG:42\r\n")
        );
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    remaining: Some("42\n")
                },
                remaining: Some("42\n")
            }),
            parse("-TEST-TAG:42\n")
        );
    }
}
