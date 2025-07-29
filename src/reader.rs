use crate::{
    config::ParsingOptions,
    error::{ReaderBytesError, ReaderStrError, ValidationError},
    line::{HlsLine, parse_bytes_with_custom, parse_with_custom},
    tag::known::{IsKnownName, NoCustomTag, ParsedTag, TagInformation},
};
use std::{fmt::Debug, marker::PhantomData};

pub struct Reader<R, CustomTag> {
    inner: R,
    options: ParsingOptions,
    _marker: PhantomData<CustomTag>,
}

macro_rules! impl_reader {
    ($type:ty, $parse_fn:ident, $from_fn_ident:ident, $from_custom_fn_ident:ident, $error_type:ident) => {
        impl<'a> Reader<&'a $type, NoCustomTag> {
            // Creates a reader.
            pub fn $from_fn_ident(data: &'a $type, options: ParsingOptions) -> Self {
                Self {
                    inner: data,
                    options,
                    _marker: PhantomData::<NoCustomTag>,
                }
            }
        }
        impl<'a, CustomTag> Reader<&'a $type, CustomTag>
        where
            CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
                + IsKnownName
                + TagInformation
                + Debug
                + PartialEq,
        {
            /// Creates a reader that supports custom tag parsing for the type specified by the
            /// `PhatomData`.
            pub fn $from_custom_fn_ident(
                str: &'a $type,
                options: ParsingOptions,
                custom: PhantomData<CustomTag>,
            ) -> Self {
                Self {
                    inner: str,
                    options,
                    _marker: custom,
                }
            }

            /// Returns the inner data of the reader.
            pub fn into_inner(self) -> &'a $type {
                self.inner
            }

            /// Reads a single HLS line from the reference data.
            pub fn read_line(&mut self) -> Result<Option<HlsLine<'a, CustomTag>>, $error_type<'a>> {
                if self.inner.is_empty() {
                    return Ok(None);
                };
                match $parse_fn(self.inner, &self.options) {
                    Ok(slice) => {
                        let parsed = slice.parsed;
                        let remaining = slice.remaining;
                        std::mem::swap(&mut self.inner, &mut remaining.unwrap_or_default());
                        Ok(Some(parsed))
                    }
                    Err(error) => {
                        let remaining = error.errored_line_slice.remaining;
                        std::mem::swap(&mut self.inner, &mut remaining.unwrap_or_default());
                        Err($error_type {
                            errored_line: error.errored_line_slice.parsed,
                            error: error.error,
                        })
                    }
                }
            }
        }
    };
}

impl_reader!(
    str,
    parse_with_custom,
    from_str,
    with_custom_from_str,
    ReaderStrError
);
impl_reader!(
    [u8],
    parse_bytes_with_custom,
    from_bytes,
    with_custom_from_bytes,
    ReaderBytesError
);

#[cfg(test)]
mod tests {
    use crate::{
        config::ParsingOptionsBuilder,
        error::{SyntaxError, UnknownTagSyntaxError, ValidationErrorValueKind},
        tag::{
            hls::{
                endlist::Endlist, inf::Inf, m3u::M3u, targetduration::Targetduration,
                version::Version,
            },
            unknown,
            value::{ParsedAttributeValue, SemiParsedTagValue},
        },
    };
    use std::collections::HashMap;

    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! reader_test {
        ($reader:tt, $method:tt, $expectation:expr $(, $buf:ident)?) => {
            for i in 0..=11 {
                let line = $reader.$method($(&mut $buf)?).unwrap();
                match i {
                    0 => assert_eq!(Some(HlsLine::from(M3u)), line),
                    1 => assert_eq!(Some(HlsLine::from(Targetduration::new(10))), line),
                    2 => assert_eq!(Some(HlsLine::from(Version::new(3))), line),
                    3 => assert_eq!($expectation, line),
                    4 => assert_eq!(Some(HlsLine::from(Inf::new(9.009, String::new()))), line),
                    5 => assert_eq!(
                        Some(HlsLine::Uri("http://media.example.com/first.ts".into())),
                        line
                    ),
                    6 => assert_eq!(Some(HlsLine::from(Inf::new(9.009, String::new()))), line),
                    7 => assert_eq!(
                        Some(HlsLine::Uri("http://media.example.com/second.ts".into())),
                        line
                    ),
                    8 => assert_eq!(Some(HlsLine::from(Inf::new(3.003, String::new()))), line),
                    9 => assert_eq!(
                        Some(HlsLine::Uri("http://media.example.com/third.ts".into())),
                        line
                    ),
                    10 => assert_eq!(Some(HlsLine::from(Endlist)), line),
                    11 => assert_eq!(None, line),
                    _ => panic!(),
                }
            }
        };
    }

    #[test]
    fn reader_from_str_should_read_as_expected() {
        let mut reader = Reader::from_str(
            EXAMPLE_MANIFEST,
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
        );
        reader_test!(
            reader,
            read_line,
            Some(HlsLine::from(unknown::Tag {
                name: "-X-EXAMPLE-TAG",
                value: Some(b"MEANING-OF-LIFE=42,QUESTION=\"UNKNOWN\""),
                original_input: &EXAMPLE_MANIFEST.as_bytes()[50..],
                validation_error: None,
            }))
        );
    }

    #[test]
    fn reader_from_buf_read_should_read_as_expected() {
        let inner = EXAMPLE_MANIFEST.as_bytes();
        let mut reader = Reader::from_bytes(
            inner,
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
        );
        reader_test!(
            reader,
            read_line,
            Some(HlsLine::from(unknown::Tag {
                name: "-X-EXAMPLE-TAG",
                value: Some(b"MEANING-OF-LIFE=42,QUESTION=\"UNKNOWN\""),
                original_input: &EXAMPLE_MANIFEST.as_bytes()[50..],
                validation_error: None,
            }))
        );
    }

    #[test]
    fn reader_from_str_with_custom_should_read_as_expected() {
        let mut reader = Reader::with_custom_from_str(
            EXAMPLE_MANIFEST,
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
            PhantomData::<ExampleTag>,
        );
        reader_test!(
            reader,
            read_line,
            Some(HlsLine::from(ExampleTag::new(42, "UNKNOWN")))
        );
    }

    #[test]
    fn reader_from_buf_with_custom_read_should_read_as_expected() {
        let inner = EXAMPLE_MANIFEST.as_bytes();
        let mut reader = Reader::with_custom_from_bytes(
            inner,
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
            PhantomData::<ExampleTag>,
        );
        reader_test!(
            reader,
            read_line,
            Some(HlsLine::from(ExampleTag::new(42, "UNKNOWN")))
        );
    }

    #[test]
    fn when_reader_fails_it_moves_to_next_line() {
        let input = concat!("#EXTM3U\n", "#EXT\n", "#Comment");
        let mut reader = Reader::from_bytes(
            input.as_bytes(),
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
        );
        assert_eq!(Ok(Some(HlsLine::from(M3u))), reader.read_line());
        assert_eq!(
            Err(ReaderBytesError {
                errored_line: b"#EXT",
                error: SyntaxError::from(UnknownTagSyntaxError::UnexpectedNoTagName)
            }),
            reader.read_line()
        );
        assert_eq!(
            Ok(Some(HlsLine::Comment("Comment".into()))),
            reader.read_line()
        );
    }

    // Example custom tag implementation for the tests above.
    #[derive(Debug, PartialEq, Clone)]
    struct ExampleTag<'a> {
        answer: u64,
        question: &'a str,
    }
    impl ExampleTag<'static> {
        fn new(answer: u64, question: &'static str) -> Self {
            Self { answer, question }
        }
    }
    impl<'a> TryFrom<ParsedTag<'a>> for ExampleTag<'a> {
        type Error = ValidationError;
        fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
            let SemiParsedTagValue::AttributeList(ref attribute_list) = tag.value else {
                return Err(ValidationError::UnexpectedValueType(
                    ValidationErrorValueKind::AttributeList,
                ));
            };
            let Some(ParsedAttributeValue::DecimalInteger(answer)) =
                attribute_list.get("MEANING-OF-LIFE")
            else {
                return Err(ValidationError::MissingRequiredAttribute("MEANING-OF-LIFE"));
            };
            let Some(ParsedAttributeValue::QuotedString(question)) = attribute_list.get("QUESTION")
            else {
                return Err(ValidationError::MissingRequiredAttribute("QUESTION"));
            };
            Ok(Self {
                answer: *answer,
                question,
            })
        }
    }
    impl<'a> IsKnownName for ExampleTag<'a> {
        fn is_known_name(name: &str) -> bool {
            name == "-X-EXAMPLE-TAG"
        }
    }
    impl<'a> TagInformation for ExampleTag<'a> {
        fn name(&self) -> &str {
            "-X-EXAMPLE-TAG"
        }

        fn value(&self) -> SemiParsedTagValue {
            SemiParsedTagValue::AttributeList(HashMap::from([
                (
                    "MEANING-OF-LIFE",
                    ParsedAttributeValue::DecimalInteger(self.answer),
                ),
                (
                    "QUESTION",
                    ParsedAttributeValue::QuotedString(self.question),
                ),
            ]))
        }
    }
}

#[cfg(test)]
// Example taken from HLS specification with one custom tag added.
// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-9.1
const EXAMPLE_MANIFEST: &str = r#"#EXTM3U
#EXT-X-TARGETDURATION:10
#EXT-X-VERSION:3
#EXT-X-EXAMPLE-TAG:MEANING-OF-LIFE=42,QUESTION="UNKNOWN"
#EXTINF:9.009,
http://media.example.com/first.ts
#EXTINF:9.009,
http://media.example.com/second.ts
#EXTINF:3.003,
http://media.example.com/third.ts
#EXT-X-ENDLIST
"#;
