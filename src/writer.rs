use crate::{
    line::HlsLine,
    tag::{
        known::TagInformation,
        value::{HlsPlaylistType, ParsedAttributeValue, ParsedTagValue},
    },
};
use std::io::{self, Write};

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
    pub fn write_line(&mut self, line: &HlsLine) -> io::Result<usize> {
        let mut count = 0usize;
        match line {
            HlsLine::Blank => (),
            HlsLine::Comment(c) => {
                count += self.write(b"#")?;
                count += self.write(c.as_bytes())?;
            }
            HlsLine::Uri(u) => count += self.write(u.as_bytes())?,
            HlsLine::UnknownTag(t) => count += self.write(t.as_str().as_bytes())?,
            HlsLine::KnownTag(t) => match t {
                crate::tag::known::Tag::Hls(tag) => {
                    count += self.write(tag.as_str().as_bytes())?;
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

fn string_from<T>(custom_tag: &T) -> String
where
    T: TagInformation,
{
    match custom_tag.value() {
        ParsedTagValue::Empty => format!("#EXT{}", custom_tag.name()),
        ParsedTagValue::TypeEnum(hls_playlist_type) => match hls_playlist_type {
            HlsPlaylistType::Event => format!("#EXT{}:EVENT", custom_tag.name()),
            HlsPlaylistType::Vod => format!("#EXT{}:VOD", custom_tag.name()),
        },
        ParsedTagValue::DecimalInteger(n) => {
            format!("#EXT{}:{}", custom_tag.name(), n)
        }
        ParsedTagValue::DecimalIntegerRange(n, r) => {
            format!("#EXT{}:{}@{}", custom_tag.name(), n, r)
        }
        ParsedTagValue::DecimalFloatingPointWithOptionalTitle(n, t) => {
            if t.is_empty() {
                format!("#EXT{}:{}", custom_tag.name(), n)
            } else {
                format!("#EXT{}:{},{}", custom_tag.name(), n, t)
            }
        }
        ParsedTagValue::DateTimeMsec(date_time) => {
            format!("#EXT{}:{}", custom_tag.name(), date_time)
        }
        ParsedTagValue::AttributeList(list) => {
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
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        date::{DateTime, DateTimeTimezoneOffset},
        tag::{
            hls::{
                self, inf::Inf, m3u::M3u, media_sequence::MediaSequence,
                target_duration::Targetduration, version::Version,
            },
            known,
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

        fn value(&self) -> ParsedTagValue {
            match self {
                TestTag::Empty => ParsedTagValue::Empty,
                TestTag::Type => ParsedTagValue::TypeEnum(HlsPlaylistType::Vod),
                TestTag::Int => ParsedTagValue::DecimalInteger(42),
                TestTag::Range => ParsedTagValue::DecimalIntegerRange(1024, 512),
                TestTag::Float { title } => {
                    ParsedTagValue::DecimalFloatingPointWithOptionalTitle(42.42, title)
                }
                TestTag::Date => ParsedTagValue::DateTimeMsec(DateTime {
                    date_fullyear: 2025,
                    date_month: 6,
                    date_mday: 17,
                    time_hour: 1,
                    time_minute: 37,
                    time_second: 15.129,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: -5,
                        time_minute: 0,
                    },
                }),
                TestTag::List => ParsedTagValue::AttributeList(HashMap::from([
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
        assert_eq!("#EXT-X-TEST-TAG", string_from(&test).as_str());
    }

    #[test]
    fn to_string_on_type_is_valid() {
        let test = TestTag::Type;
        assert_eq!("#EXT-X-TEST-TAG:VOD", string_from(&test).as_str());
    }

    #[test]
    fn to_string_on_int_is_valid() {
        let test = TestTag::Int;
        assert_eq!("#EXT-X-TEST-TAG:42", string_from(&test).as_str());
    }

    #[test]
    fn to_string_on_range_is_valid() {
        let test = TestTag::Range;
        assert_eq!("#EXT-X-TEST-TAG:1024@512", string_from(&test).as_str());
    }

    #[test]
    fn to_string_on_float_is_valid() {
        let test = TestTag::Float { title: "" };
        assert_eq!("#EXT-X-TEST-TAG:42.42", string_from(&test).as_str());
        let test = TestTag::Float {
            title: " A useful comment",
        };
        assert_eq!(
            "#EXT-X-TEST-TAG:42.42, A useful comment",
            string_from(&test).as_str()
        );
    }

    #[test]
    fn to_string_on_date_is_valid() {
        let test = TestTag::Date;
        assert_eq!(
            "#EXT-X-TEST-TAG:2025-06-17T01:37:15.129-05:00",
            string_from(&test).as_str()
        );
    }

    #[test]
    fn to_string_on_list_is_valid() {
        let test = TestTag::List;
        let mut found_int = false;
        let mut found_float = false;
        let mut found_quote = false;
        let mut found_enum = false;
        let tag_string = string_from(&test);
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
        writer
            .write_line(&HlsLine::KnownTag(known::Tag::Hls(hls::Tag::M3u(M3u))))
            .unwrap();
        writer
            .write_line(&HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Version(
                Version::new(3),
            ))))
            .unwrap();
        writer
            .write_line(&HlsLine::KnownTag(known::Tag::Hls(
                hls::Tag::Targetduration(Targetduration::new(8)),
            )))
            .unwrap();
        writer
            .write_line(&HlsLine::KnownTag(known::Tag::Hls(
                hls::Tag::MediaSequence(MediaSequence::new(2680)),
            )))
            .unwrap();
        writer.write_line(&HlsLine::Blank).unwrap();
        writer
            .write_line(&HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(
                Inf::new(7.975, ""),
            ))))
            .unwrap();
        writer
            .write_line(&HlsLine::Uri(
                "https://priv.example.com/fileSequence2680.ts",
            ))
            .unwrap();
        writer
            .write_line(&HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(
                Inf::new(7.941, ""),
            ))))
            .unwrap();
        writer
            .write_line(&HlsLine::Uri(
                "https://priv.example.com/fileSequence2681.ts",
            ))
            .unwrap();
        writer
            .write_line(&HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(
                Inf::new(7.975, ""),
            ))))
            .unwrap();
        writer
            .write_line(&HlsLine::Uri(
                "https://priv.example.com/fileSequence2682.ts",
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
            writer.write_line(&HlsLine::Comment(" A comment")).unwrap()
        );
        assert_eq!(
            13, // 12 (str) + 1 (\n) == 13
            writer.write_line(&HlsLine::Uri("example.m3u8")).unwrap()
        );
        assert_eq!(
            22, // 21 (#EXTINF:6.006,PTS:0.0) + 1 (\n) == 22
            writer
                .write_line(&HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(
                    Inf::new(6.006, "PTS:0.0")
                ))))
                .unwrap()
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
