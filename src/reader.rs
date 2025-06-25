use std::{
    fmt::Debug,
    io::{self, BufRead},
    marker::PhantomData,
};

use crate::{
    config::ParsingOptions,
    line::{HlsLine, ParsedLineSlice, parse_with_custom},
    tag::known::{IsKnownName, NoCustomTag, ParsedTag, TagInformation},
};

pub struct Reader<R, CustomTag> {
    inner: R,
    options: ParsingOptions,
    _marker: PhantomData<CustomTag>,
}

impl<'a> Reader<&'a str, NoCustomTag> {
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
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str>
        + IsKnownName
        + TagInformation
        + Debug
        + PartialEq,
{
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

    pub fn read_line(&mut self) -> Result<Option<HlsLine<'a, CustomTag>>, &'static str> {
        if self.inner.is_empty() {
            return Ok(None);
        };
        let ParsedLineSlice { parsed, remaining } = parse_with_custom(self.inner, &self.options)?;
        std::mem::swap(&mut self.inner, &mut remaining.unwrap_or_default());
        Ok(Some(parsed))
    }
}

impl<R: BufRead> Reader<R, NoCustomTag> {
    pub fn from_reader(reader: R, options: ParsingOptions) -> Self {
        Self {
            inner: reader,
            options,
            _marker: PhantomData::<NoCustomTag>,
        }
    }

    pub fn read_line_into<'b>(
        &mut self,
        buf: &'b mut String,
    ) -> Result<Option<HlsLine<'b>>, &'static str> {
        self.read_line_into_custom::<NoCustomTag>(buf)
    }

    // Annoyingly, unlike when using &'a str, I can't completely infer the custom type on
    // construction. This is because if the custom tag has a generic lifetime, then when
    // constructing the Reader the lifetime is assumed to be that of the Reader; however, that is
    // incorrect per the read_line_into method, which expects the lifetime of the CustomTag within
    // the HlsLine to be that of the input String parameter. This means when looping in the below
    // tests, I cannot mutate the input String between loop iterations, because Rust expects that
    // the CustomTag could be valid as long as the Reader which would then be a problem with the
    // mutable borrow on String. There may be a better way of doing this but I don't know it yet.
    pub fn read_line_into_custom<'b, CustomTag>(
        &mut self,
        buf: &'b mut String,
    ) -> Result<Option<HlsLine<'b, CustomTag>>, &'static str>
    where
        CustomTag: TryFrom<ParsedTag<'b>, Error = &'static str>
            + IsKnownName
            + TagInformation
            + Debug
            + PartialEq,
    {
        let available = loop {
            match self.inner.fill_buf() {
                Ok(n) if n.is_empty() => return Ok(None),
                Ok(n) => break std::str::from_utf8(n).map_err(|_| "Not UTF-8")?,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => (),
                Err(_) => return Err("IO read error"),
            }
        };
        let total_len = available.len();
        buf.clear();
        buf.push_str(available);
        let ParsedLineSlice { parsed, remaining } = parse_with_custom(buf.as_str(), &self.options)?;
        let remaining_len = remaining.map_or(0, |s| s.len());
        self.inner.consume(total_len - remaining_len);
        Ok(Some(parsed))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::ParsingOptionsBuilder,
        tag::{
            hls::{
                endlist::Endlist, inf::Inf, m3u::M3u, targetduration::Targetduration,
                version::Version,
            },
            unknown,
            value::{ParsedAttributeValue, ParsedTagValue},
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
                value: Some("MEANING-OF-LIFE=42,QUESTION=\"UNKNOWN\""),
                original_input: &EXAMPLE_MANIFEST[50..],
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
        let mut string = String::new();
        reader_test!(
            reader,
            read_line_into,
            Some(HlsLine::from(unknown::Tag {
                name: "-X-EXAMPLE-TAG",
                value: Some("MEANING-OF-LIFE=42,QUESTION=\"UNKNOWN\""),
                original_input: &EXAMPLE_MANIFEST[50..],
            })),
            string
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
        let mut string = String::new();
        reader_test!(
            reader,
            read_line_into_custom,
            Some(HlsLine::from(ExampleTag::new(42, "UNKNOWN"))),
            string
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
        type Error = &'static str;
        fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
            let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
                return Err("Unexpected value");
            };
            let Some(ParsedAttributeValue::DecimalInteger(answer)) =
                attribute_list.get("MEANING-OF-LIFE")
            else {
                return Err("Missing MEANING-OF-LIFE attribute");
            };
            let Some(ParsedAttributeValue::QuotedString(question)) = attribute_list.get("QUESTION")
            else {
                return Err("Missing QUESTION attribute");
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

        fn value(&self) -> ParsedTagValue {
            ParsedTagValue::AttributeList(HashMap::from([
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
