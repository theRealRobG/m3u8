use crate::{
    error::ValidationError,
    line::HlsLine,
    tag::{
        known::{IsKnownName, ParsedTag, TagInformation},
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{
    borrow::Cow,
    fmt::Debug,
    io::{self, Write},
};

#[derive(Clone)]
pub struct Writer<W>
where
    W: Write,
{
    /// underlying writer
    writer: W,
}

impl<W> Writer<W>
where
    W: Write,
{
    /// Creates a `Writer` from a generic writer.
    pub const fn new(inner: W) -> Writer<W> {
        Writer { writer: inner }
    }

    /// Consumes this `Writer`, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Get a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Get a reference to the underlying writer.
    pub const fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Write the `HlsLine` to the underlying writer. Returns the number of bytes consumed during
    /// writing or an `io::Error` from the underlying writer.
    ///
    /// In this case the `CustomTag` generic is the default `NoCustomTag` struct.
    pub fn write_line(&mut self, line: HlsLine) -> io::Result<usize> {
        self.write_custom_line(line)
    }

    /// Example:
    /// ```
    /// # use m3u8::Writer;
    /// let mut writer = Writer::new(b"#EXTM3U\n".to_vec());
    /// writer.write_blank().unwrap();
    /// writer.write_comment(" Note blank line above.").unwrap();
    /// let expected = r#"#EXTM3U
    ///
    /// ## Note blank line above.
    /// "#;
    /// assert_eq!(expected.as_bytes(), writer.into_inner());
    /// ```
    pub fn write_blank(&mut self) -> io::Result<usize> {
        self.write_line(HlsLine::Blank)
    }

    /// Example:
    /// ```
    /// # use m3u8::Writer;
    /// let mut writer = Writer::new(Vec::new());
    /// writer.write_comment(" This is a comment.").unwrap();
    /// assert_eq!("# This is a comment.\n".as_bytes(), writer.into_inner());
    /// ```
    pub fn write_comment<'a>(&mut self, comment: impl Into<Cow<'a, str>>) -> io::Result<usize> {
        self.write_line(HlsLine::Comment(comment.into()))
    }

    /// Example:
    /// ```
    /// # use m3u8::Writer;
    /// let mut writer = Writer::new(Vec::new());
    /// writer.write_uri("example.m3u8").unwrap();
    /// assert_eq!("example.m3u8\n".as_bytes(), writer.into_inner());
    /// ```
    pub fn write_uri<'a>(&mut self, uri: impl Into<Cow<'a, str>>) -> io::Result<usize> {
        self.write_line(HlsLine::Uri(uri.into()))
    }

    /// Example:
    /// ```
    /// # use m3u8::Writer;
    /// # use m3u8::tag::hls::Tag;
    /// # use m3u8::tag::hls::bitrate::Bitrate;
    /// let mut writer = Writer::new(Vec::new());
    /// writer.write_hls_tag(Tag::Bitrate(Bitrate::new(10000000))).unwrap();
    /// assert_eq!(
    ///     "#EXT-X-BITRATE:10000000\n".as_bytes(),
    ///     writer.into_inner()
    /// );
    /// ```
    pub fn write_hls_tag(&mut self, tag: crate::tag::hls::Tag) -> io::Result<usize> {
        self.write_line(HlsLine::from(tag))
    }

    /// Example:
    /// ```
    /// # use m3u8::Writer;
    /// # use m3u8::tag::known::{ParsedTag, IsKnownName, TagInformation};
    /// # use m3u8::tag::value::{SemiParsedTagValue, UnparsedTagValue};
    /// # use m3u8::error::{ValidationError, ValidationErrorValueKind};
    /// #[derive(Debug, PartialEq, Clone)]
    /// struct ExampleCustomTag<'a> {
    ///     answer: u64,
    ///     original_value: &'a [u8],
    /// }
    /// impl<'a> TryFrom<ParsedTag<'a>> for ExampleCustomTag<'a> {
    ///     type Error = ValidationError;
    ///     fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
    ///         if tag.name != "-X-MEANING-OF-LIFE" {
    ///             return Err(ValidationError::UnexpectedTagName)
    ///         }
    ///         match tag.value {
    ///             SemiParsedTagValue::Unparsed(value) => {
    ///                 Ok(Self {
    ///                     answer: value.try_as_decimal_integer()?,
    ///                     original_value: value.0,
    ///                 })
    ///             }
    ///             _ => Err(ValidationError::UnexpectedValueType(
    ///                 ValidationErrorValueKind::from(&tag.value)
    ///             )),
    ///         }
    ///     }
    /// }
    /// impl IsKnownName for ExampleCustomTag<'_> {
    ///     fn is_known_name(name: &str) -> bool {
    ///         name == "-X-MEANING-OF-LIFE"
    ///     }
    /// }
    /// impl TagInformation for ExampleCustomTag<'_> {
    ///     fn name(&self) -> &str {
    ///         "-X-MEANING-OF-LIFE"
    ///     }
    ///
    ///     fn value(&self) -> SemiParsedTagValue {
    ///         SemiParsedTagValue::Unparsed(UnparsedTagValue(self.original_value))
    ///     }
    /// }
    ///
    /// let mut writer = Writer::new(Vec::new());
    /// let custom_tag = ExampleCustomTag { answer: 42, original_value: b"42" };
    /// writer.write_custom_tag(custom_tag).unwrap();
    /// assert_eq!(
    ///     "#EXT-X-MEANING-OF-LIFE:42\n".as_bytes(),
    ///     writer.into_inner()
    /// );
    /// ```
    pub fn write_custom_tag<'a, CustomTag>(&mut self, tag: CustomTag) -> io::Result<usize>
    where
        CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
            + IsKnownName
            + TagInformation
            + Debug
            + PartialEq,
    {
        self.write_custom_line(HlsLine::from(tag))
    }

    /// Write the `HlsLine` to the underlying writer. Returns the number of bytes consumed during
    /// writing or an `io::Error` from the underlying writer.
    pub fn write_custom_line<'a, CustomTag>(
        &mut self,
        line: HlsLine<'a, CustomTag>,
    ) -> io::Result<usize>
    where
        CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
            + IsKnownName
            + TagInformation
            + Debug
            + PartialEq,
    {
        let mut count = 0usize;
        match line {
            HlsLine::Blank => (),
            HlsLine::Comment(c) => {
                count += self.write(b"#")?;
                count += self.write(c.as_bytes())?;
            }
            HlsLine::Uri(u) => count += self.write(u.as_bytes())?,
            HlsLine::UnknownTag(t) => count += self.write(t.as_bytes())?,
            HlsLine::KnownTag(t) => match t {
                crate::tag::known::Tag::Hls(tag) => {
                    count += self.write(tag.into_inner().value())?;
                }
                crate::tag::known::Tag::Custom(tag) => {
                    count += self.write(string_from(tag).as_bytes())?;
                }
            },
        };
        count += self.write(b"\n")?;
        Ok(count)
    }

    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let mut count = 0usize;
        while !buf.is_empty() {
            match self.writer.write(buf) {
                Ok(0) => {
                    return Err(io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => {
                    count += n;
                    buf = &buf[n..];
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(count)
    }
}

fn string_from<T>(custom_tag: T) -> String
where
    T: TagInformation,
{
    match custom_tag.value() {
        SemiParsedTagValue::Empty => format!("#EXT{}", custom_tag.name()),
        SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(n, t) => {
            if t.is_empty() {
                format!("#EXT{}:{}", custom_tag.name(), n)
            } else {
                format!("#EXT{}:{},{}", custom_tag.name(), n, t)
            }
        }
        SemiParsedTagValue::AttributeList(list) => {
            let attrs = list
                .iter()
                .map(|(k, v)| match v {
                    ParsedAttributeValue::DecimalInteger(n) => format!("{k}={n}"),
                    ParsedAttributeValue::SignedDecimalFloatingPoint(n) => format!("{k}={n:?}"),
                    ParsedAttributeValue::QuotedString(s) => format!("{k}=\"{s}\""),
                    ParsedAttributeValue::UnquotedString(s) => format!("{k}={s}"),
                })
                .collect::<Vec<String>>();
            let value = attrs.join(",");
            format!("#EXT{}:{}", custom_tag.name(), value)
        }
        SemiParsedTagValue::Unparsed(bytes) => format!(
            "#EXT{}:{}",
            custom_tag.name(),
            String::from_utf8_lossy(bytes.0)
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::ParsingOptionsBuilder,
        tag::{
            hls::{
                self, inf::Inf, m3u::M3u, media_sequence::MediaSequence,
                targetduration::Targetduration, version::Version,
            },
            value::UnparsedTagValue,
        },
    };
    use std::collections::HashMap;

    use super::*;
    use pretty_assertions::assert_eq;

    enum TestTag {
        Empty,
        Type,
        Int,
        Range,
        Float { title: &'static str },
        Date,
        List,
    }

    impl TagInformation for TestTag {
        fn name(&self) -> &str {
            "-X-TEST-TAG"
        }

        fn value(&self) -> SemiParsedTagValue {
            match self {
                TestTag::Empty => SemiParsedTagValue::Empty,
                TestTag::Type => SemiParsedTagValue::Unparsed(UnparsedTagValue(b"VOD")),
                TestTag::Int => SemiParsedTagValue::Unparsed(UnparsedTagValue(b"42")),
                TestTag::Range => SemiParsedTagValue::Unparsed(UnparsedTagValue(b"1024@512")),
                TestTag::Float { title } => {
                    SemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(42.42, title)
                }
                TestTag::Date => {
                    SemiParsedTagValue::Unparsed(UnparsedTagValue(b"2025-06-17T01:37:15.129-05:00"))
                }
                TestTag::List => SemiParsedTagValue::AttributeList(HashMap::from([
                    ("TEST-INT", ParsedAttributeValue::DecimalInteger(42)),
                    (
                        "TEST-FLOAT",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(-42.42),
                    ),
                    (
                        "TEST-QUOTED-STRING",
                        ParsedAttributeValue::QuotedString("test"),
                    ),
                    (
                        "TEST-ENUMERATED-STRING",
                        ParsedAttributeValue::UnquotedString("test"),
                    ),
                ])),
            }
        }
    }

    #[test]
    fn to_string_on_empty_is_valid() {
        let test = TestTag::Empty;
        assert_eq!("#EXT-X-TEST-TAG", string_from(test).as_str());
    }

    #[test]
    fn to_string_on_type_is_valid() {
        let test = TestTag::Type;
        assert_eq!("#EXT-X-TEST-TAG:VOD", string_from(test).as_str());
    }

    #[test]
    fn to_string_on_int_is_valid() {
        let test = TestTag::Int;
        assert_eq!("#EXT-X-TEST-TAG:42", string_from(test).as_str());
    }

    #[test]
    fn to_string_on_range_is_valid() {
        let test = TestTag::Range;
        assert_eq!("#EXT-X-TEST-TAG:1024@512", string_from(test).as_str());
    }

    #[test]
    fn to_string_on_float_is_valid() {
        let test = TestTag::Float { title: "" };
        assert_eq!("#EXT-X-TEST-TAG:42.42", string_from(test).as_str());
        let test = TestTag::Float {
            title: " A useful comment",
        };
        assert_eq!(
            "#EXT-X-TEST-TAG:42.42, A useful comment",
            string_from(test).as_str()
        );
    }

    #[test]
    fn to_string_on_date_is_valid() {
        let test = TestTag::Date;
        assert_eq!(
            "#EXT-X-TEST-TAG:2025-06-17T01:37:15.129-05:00",
            string_from(test).as_str()
        );
    }

    #[test]
    fn to_string_on_list_is_valid() {
        let test = TestTag::List;
        let mut found_int = false;
        let mut found_float = false;
        let mut found_quote = false;
        let mut found_enum = false;
        let tag_string = string_from(test);
        let mut name_value_split = tag_string.split(':');
        assert_eq!("#EXT-X-TEST-TAG", name_value_split.next().unwrap());
        let attrs = name_value_split.next().unwrap().split(',').enumerate();
        for (index, attr) in attrs {
            match index {
                0..4 => match attr.split('=').next().unwrap() {
                    "TEST-INT" => {
                        if found_int {
                            panic!("Unexpected duplicated attribute {attr}");
                        }
                        found_int = true;
                        assert_eq!("TEST-INT=42", attr);
                    }
                    "TEST-FLOAT" => {
                        if found_float {
                            panic!("Unexpected duplicated attribute {attr}");
                        }
                        found_float = true;
                        assert_eq!("TEST-FLOAT=-42.42", attr);
                    }
                    "TEST-QUOTED-STRING" => {
                        if found_quote {
                            panic!("Unexpected duplicated attribute {attr}");
                        }
                        found_quote = true;
                        assert_eq!("TEST-QUOTED-STRING=\"test\"", attr);
                    }
                    "TEST-ENUMERATED-STRING" => {
                        if found_enum {
                            panic!("Unexpected duplicated attribute {attr}");
                        }
                        found_enum = true;
                        assert_eq!("TEST-ENUMERATED-STRING=test", attr);
                    }
                    x => panic!("Unexpected attribute {x}"),
                },
                _ => panic!("Unexpected index {index}"),
            }
        }
        assert!(found_int);
        assert!(found_float);
        assert!(found_quote);
        assert!(found_enum);
    }

    #[test]
    fn writer_should_output_expected() {
        let mut writer = Writer::new(Vec::new());
        writer.write_line(HlsLine::from(M3u)).unwrap();
        writer.write_line(HlsLine::from(Version::new(3))).unwrap();
        writer
            .write_line(HlsLine::from(Targetduration::new(8)))
            .unwrap();
        writer
            .write_line(HlsLine::from(MediaSequence::new(2680)))
            .unwrap();
        writer.write_line(HlsLine::Blank).unwrap();
        writer
            .write_line(HlsLine::from(Inf::new(7.975, "".to_string())))
            .unwrap();
        writer
            .write_line(HlsLine::Uri(
                "https://priv.example.com/fileSequence2680.ts".into(),
            ))
            .unwrap();
        writer
            .write_line(HlsLine::from(Inf::new(7.941, "".to_string())))
            .unwrap();
        writer
            .write_line(HlsLine::Uri(
                "https://priv.example.com/fileSequence2681.ts".into(),
            ))
            .unwrap();
        writer
            .write_line(HlsLine::from(Inf::new(7.975, "".to_string())))
            .unwrap();
        writer
            .write_line(HlsLine::Uri(
                "https://priv.example.com/fileSequence2682.ts".into(),
            ))
            .unwrap();
        assert_eq!(
            EXPECTED_WRITE_OUTPUT,
            std::str::from_utf8(&writer.into_inner()).unwrap()
        );
    }

    #[test]
    fn write_line_should_return_correct_byte_count() {
        let mut writer = Writer::new(Vec::new());
        assert_eq!(
            12, // 1 (#) + 10 (str) + 1 (\n) == 12
            writer
                .write_line(HlsLine::Comment(" A comment".into()))
                .unwrap()
        );
        assert_eq!(
            13, // 12 (str) + 1 (\n) == 13
            writer
                .write_line(HlsLine::Uri("example.m3u8".into()))
                .unwrap()
        );
        assert_eq!(
            22, // 21 (#EXTINF:6.006,PTS:0.0) + 1 (\n) == 22
            writer
                .write_line(HlsLine::from(hls::Tag::Inf(Inf::new(
                    6.006,
                    "PTS:0.0".to_string()
                ))))
                .unwrap()
        );
    }

    #[test]
    fn writing_with_no_manipulation_should_leave_output_unchaged_except_for_new_lines() {
        let mut writer = Writer::new(Vec::new());
        let options = ParsingOptionsBuilder::new()
            .with_parsing_for_m3u()
            .with_parsing_for_version()
            .build();
        let mut remaining = Some(EXPECTED_WRITE_OUTPUT);
        while let Some(line) = remaining {
            let slice = crate::line::parse(line, &options).unwrap();
            remaining = slice.remaining;
            writer.write_line(slice.parsed).unwrap();
        }
        let mut expected = EXPECTED_WRITE_OUTPUT.to_string();
        expected.push('\n');
        assert_eq!(
            expected.as_str(),
            std::str::from_utf8(&writer.into_inner()).unwrap()
        );
    }
}

#[cfg(test)]
const EXPECTED_WRITE_OUTPUT: &str = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:8
#EXT-X-MEDIA-SEQUENCE:2680

#EXTINF:7.975
https://priv.example.com/fileSequence2680.ts
#EXTINF:7.941
https://priv.example.com/fileSequence2681.ts
#EXTINF:7.975
https://priv.example.com/fileSequence2682.ts
"#;
