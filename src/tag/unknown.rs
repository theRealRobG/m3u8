use crate::{
    error::{GenericSyntaxError, UnknownTagSyntaxError, ValidationError},
    line::ParsedLineSlice,
    utils::{split_by_first_lf, str_from},
};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Tag<'a> {
    pub(crate) name: &'a str,
    pub(crate) value: Option<&'a str>,
    pub(crate) original_input: &'a str,
    pub(crate) validation_error: Option<ValidationError>,
}

impl Tag<'_> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn value(&self) -> Option<&str> {
        self.value
    }

    pub fn validation_error(&self) -> Option<ValidationError> {
        self.validation_error
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(self.original_input).parsed
    }
}

pub fn parse(input: &str) -> Result<ParsedLineSlice<Tag>, UnknownTagSyntaxError> {
    let mut bytes = input.as_bytes().iter();
    let Some(b'#') = bytes.next() else {
        return Err(UnknownTagSyntaxError::InvalidTag);
    };
    let Some(b'E') = bytes.next() else {
        return Err(UnknownTagSyntaxError::InvalidTag);
    };
    let Some(b'X') = bytes.next() else {
        return Err(UnknownTagSyntaxError::InvalidTag);
    };
    let Some(b'T') = bytes.next() else {
        return Err(UnknownTagSyntaxError::InvalidTag);
    };
    parse_assuming_ext_taken(str_from(bytes.as_slice()), input)
}

pub(crate) fn parse_assuming_ext_taken<'a>(
    input: &'a str,
    original_input: &'a str,
) -> Result<ParsedLineSlice<'a, Tag<'a>>, UnknownTagSyntaxError> {
    if input.is_empty() {
        return Err(UnknownTagSyntaxError::UnexpectedEmptyInput);
    };
    let mut bytes = input.as_bytes().iter();
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        let Some(byte) = bytes.next() else {
            let name = &input[..(iterations - 1)];
            let value = None;
            let remaining = None;
            return Ok(ParsedLineSlice {
                parsed: Tag {
                    name,
                    value,
                    original_input,
                    validation_error: None,
                },
                remaining,
            });
        };
        match byte {
            b':' => {
                let name = &input[..(iterations - 1)];
                let ParsedLineSlice {
                    parsed: value,
                    remaining,
                } = split_by_first_lf(str_from(bytes.as_slice()));
                return Ok(ParsedLineSlice {
                    parsed: Tag {
                        name,
                        value: Some(value),
                        original_input,
                        validation_error: None,
                    },
                    remaining,
                });
            }
            b'\n' => {
                let name = &input[..(iterations - 1)];
                return Ok(ParsedLineSlice {
                    parsed: Tag {
                        name,
                        value: None,
                        original_input,
                        validation_error: None,
                    },
                    remaining: Some(str_from(bytes.as_slice())),
                });
            }
            b'\r' => {
                let Some(b'\n') = bytes.next() else {
                    return Err(GenericSyntaxError::CarriageReturnWithoutLineFeed)?;
                };
                let name = &input[..(iterations - 1)];
                return Ok(ParsedLineSlice {
                    parsed: Tag {
                        name,
                        value: None,
                        original_input,
                        validation_error: None,
                    },
                    remaining: Some(str_from(bytes.as_slice())),
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
            value: None,
            original_input: "#EXT-X-TEST",
            validation_error: None,
        };
        assert_eq!(None, tag.value());
        assert_eq!("#EXT-X-TEST", tag.as_str());
    }

    #[test]
    fn tag_value_empty_when_remaining_is_empty() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some(""),
            original_input: "#EXT-X-TEST:",
            validation_error: None,
        };
        assert_eq!(Some(""), tag.value());
        assert_eq!("#EXT-X-TEST:", tag.as_str());
    }

    #[test]
    fn tag_value_some_when_remaining_is_some() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some("42"),
            original_input: "#EXT-X-TEST:42",
            validation_error: None,
        };
        assert_eq!(Some("42"), tag.value());
        assert_eq!("#EXT-X-TEST:42", tag.as_str());
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_crlf() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some("42"),
            original_input: "#EXT-X-TEST:42\r\n#EXT-X-NEW-TEST\r\n",
            validation_error: None,
        };
        assert_eq!(Some("42"), tag.value());
        assert_eq!("#EXT-X-TEST:42", tag.as_str());
    }

    #[test]
    fn tag_value_remaining_is_some_when_split_by_lf() {
        let tag = Tag {
            name: "-X-TEST",
            value: Some("42"),
            original_input: "#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n",
            validation_error: None,
        };
        assert_eq!(Some("42"), tag.value());
        assert_eq!("#EXT-X-TEST:42", tag.as_str());
    }

    #[test]
    fn parses_tag_with_no_value() {
        assert_eq!(
            Ok(ParsedLineSlice {
                parsed: Tag {
                    name: "-TEST-TAG",
                    value: None,
                    original_input: "#EXT-TEST-TAG",
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
                    original_input: "#EXT-TEST-TAG\r\n",
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
                    original_input: "#EXT-TEST-TAG\n",
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
                    value: Some("42"),
                    original_input: "#EXT-TEST-TAG:42",
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
                    value: Some("42"),
                    original_input: "#EXT-TEST-TAG:42\r\n",
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
                    value: Some("42"),
                    original_input: "#EXT-TEST-TAG:42\n",
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
                    value: Some("42"),
                    original_input: "#EXT-X-TEST:42\r\n#EXT-X-NEW-TEST\r\n",
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
                    value: Some("42"),
                    original_input: "#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n",
                    validation_error: None,
                },
                remaining: Some("#EXT-X-NEW-TEST\n")
            }),
            parse("#EXT-X-TEST:42\n#EXT-X-NEW-TEST\n")
        );
    }
}
