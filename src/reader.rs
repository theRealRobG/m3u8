use crate::{
    config::ParsingOptions,
    error::{SyntaxError, ValidationError},
    line::{HlsLine, ParsedByteSlice, ParsedLineSlice, parse_bytes_with_custom, parse_with_custom},
    tag::known::{IsKnownName, NoCustomTag, ParsedTag, TagInformation},
};
use std::{
    error::Error,
    fmt::{Debug, Display},
    io::{self, BufRead},
    marker::PhantomData,
};

pub struct Reader<R, CustomTag> {
    inner: R,
    options: ParsingOptions,
    _marker: PhantomData<CustomTag>,
}

impl<'a> Reader<&'a str, NoCustomTag> {
    // Creates a reader.
    pub fn from_str(str: &'a str, options: ParsingOptions) -> Self {
        Self {
            inner: str,
            options,
            _marker: PhantomData::<NoCustomTag>,
        }
    }
}

impl<'a, CustomTag> Reader<&'a str, CustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + PartialEq,
{
    /// Creates a reader that supports custom tag parsing for the type specified by the
    /// `PhatomData`.
    pub fn from_str_with_custom_tag_parsing(
        str: &'a str,
        options: ParsingOptions,
        custom: PhantomData<CustomTag>,
    ) -> Self {
        Self {
            inner: str,
            options,
            _marker: custom,
        }
    }

    /// Reads a single HLS line from the reference str.
    pub fn read_line(&mut self) -> Result<Option<HlsLine<'a, CustomTag>>, SyntaxError> {
        if self.inner.is_empty() {
            return Ok(None);
        };
        let ParsedLineSlice { parsed, remaining } = parse_with_custom(self.inner, &self.options)?;
        std::mem::swap(&mut self.inner, &mut remaining.unwrap_or_default());
        Ok(Some(parsed))
    }

    /// Returns the inner data of the reader.
    pub fn into_inner(self) -> &'a str {
        self.inner
    }
}

pub struct BufReadHolder<R: BufRead> {
    reader: R,
    last_consume_count: usize,
}

#[derive(Debug)]
pub enum BufReadError {
    SyntaxError(SyntaxError),
    IoError(io::Error),
}
impl Display for BufReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError(e) => std::fmt::Display::fmt(&e, f),
            Self::IoError(e) => std::fmt::Display::fmt(&e, f),
        }
    }
}
impl Error for BufReadError {}
impl From<SyntaxError> for BufReadError {
    fn from(value: SyntaxError) -> Self {
        Self::SyntaxError(value)
    }
}
impl From<io::Error> for BufReadError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl<R: BufRead> Reader<BufReadHolder<R>, NoCustomTag> {
    /// Creates a Reader from a BufRead.
    pub fn from_reader(reader: R, options: ParsingOptions) -> Self {
        Self {
            inner: BufReadHolder {
                reader,
                last_consume_count: 0,
            },
            options,
            _marker: PhantomData::<NoCustomTag>,
        }
    }

    /// Reads a single HLS line from the buffer.
    ///
    /// This method assumes no custom tag parsing is desired. If the user wants to include custom
    /// tag parsing then use the `read_line_custom` method and indicate the custom tag type via
    /// specialization of the generic.
    pub fn read_line<'a>(&'a mut self) -> Result<Option<HlsLine<'a>>, BufReadError> {
        self.read_line_custom::<NoCustomTag>()
    }

    // Annoyingly, unlike when using &'a str, I can't completely infer the custom type on
    // construction. This is because if the custom tag has a generic lifetime, then when
    // constructing the Reader the lifetime is assumed to be that of the Reader; however, that is
    // incorrect per the read_line_into method, which expects the lifetime of the CustomTag within
    // the HlsLine to be that of the input String parameter. This means when looping in the below
    // tests, I cannot mutate the input String between loop iterations, because Rust expects that
    // the CustomTag could be valid as long as the Reader which would then be a problem with the
    // mutable borrow on String. There may be a better way of doing this but I don't know it yet.
    /// Reads a single HLS line from the buffer.
    pub fn read_line_custom<'a, CustomTag>(
        &'a mut self,
    ) -> Result<Option<HlsLine<'a, CustomTag>>, BufReadError>
    where
        CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
            + IsKnownName
            + TagInformation
            + Debug
            + PartialEq,
    {
        self.inner.reader.consume(self.inner.last_consume_count);
        let available = match self.inner.reader.fill_buf() {
            Ok([]) => return Ok(None),
            Ok(n) => n,
            Err(e) => return Err(BufReadError::from(e)),
        };
        let total_len = available.len();
        let ParsedByteSlice { parsed, remaining } =
            parse_bytes_with_custom::<CustomTag>(available, &self.options)?;
        let remaining_len = remaining.map_or(0, |s| s.len());
        let consumed = total_len - remaining_len;
        self.inner.last_consume_count = consumed;
        Ok(Some(parsed))
    }

    /// Returns the inner data of the reader.
    pub fn into_inner(mut self) -> R {
        self.inner.reader.consume(self.inner.last_consume_count);
        self.inner.reader
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::ParsingOptionsBuilder,
        error::ValidationErrorValueKind,
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
                        Some(HlsLine::Uri("http://media.example.com/first.ts")),
                        line
                    ),
                    6 => assert_eq!(Some(HlsLine::from(Inf::new(9.009, String::new()))), line),
                    7 => assert_eq!(
                        Some(HlsLine::Uri("http://media.example.com/second.ts")),
                        line
                    ),
                    8 => assert_eq!(Some(HlsLine::from(Inf::new(3.003, String::new()))), line),
                    9 => assert_eq!(
                        Some(HlsLine::Uri("http://media.example.com/third.ts")),
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
        let mut reader = Reader::from_reader(
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
        let mut reader = Reader::from_str_with_custom_tag_parsing(
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
        let mut reader = Reader::from_reader(
            inner,
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
        );
        reader_test!(
            reader,
            read_line_custom,
            Some(HlsLine::from(ExampleTag::new(42, "UNKNOWN")))
        );
    }

    // Example custom tag implementation for the tests above.
    #[derive(Debug, PartialEq)]
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
