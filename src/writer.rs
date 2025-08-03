use crate::{
    line::HlsLine,
    tag::known::{IntoInnerTag, WritableCustomTag},
};
use std::{
    borrow::Cow,
    io::{self, Write},
};

/// A writer of HLS lines.
///
/// This structure wraps a [`Write`] with methods that make writing parsed (or user constructed) HLS
/// lines easier. The `Writer` handles inserting new lines where necessary and formatting for tags.
/// An important note to make, is that with every tag implementation within [`crate::tag::hls`], the
/// reference to the original input data is used directly when writing. This means that we avoid
/// unnecessary allocations unless the data has been mutated. The same is true of
/// [`crate::tag::known::Tag::Custom`] tags (described in [`crate::tag::known::CustomTagAccess`]).
/// Where necessary, the inner [`Write`] can be accessed in any type of ownership semantics (owned
/// via [`Self::into_inner`], mutable borrow via [`Self::get_mut`], borrow via [`Self::get_ref`]).
///
/// ## Mutate data as proxy
///
/// A common use case for using `Writer` is when implementing a proxy service for a HLS stream that
/// modifies the playlist. In that case, the [`crate::Reader`] is used to extract information from
/// the upstream bytes, the various tag types can be used to modify the data where necessary, and
/// the `Writer` is then used to write the result to data for the body of the HTTP response. Below
/// we provide a toy example of this (for a more interesting example, the repository includes an
/// implementation of a HLS delta update in `benches/delta_update_bench.rs`).
/// ```
/// # use m3u8::{ config::ParsingOptions, line::HlsLine, tag::{hls, known}, Reader, Writer };
/// # use std::io::{self, Write};
/// const INPUT: &str = r#"
/// #EXTINF:4
/// segment_100.mp4
/// #EXTINF:4
/// segment_101.mp4
/// "#;
///
/// let mut reader = Reader::from_str(INPUT, ParsingOptions::default());
/// let mut writer = Writer::new(Vec::new());
///
/// let mut added_hello = false;
/// loop {
///     match reader.read_line() {
///         // In this branch we match the #EXTINF tag and update the title property to add a
///         // message.
///         Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(mut tag))))) => {
///             if added_hello {
///                 tag.set_title("World!");
///             } else {
///                 tag.set_title("Hello,");
///                 added_hello = true;
///             }
///             writer.write_line(HlsLine::from(tag))?;
///         }
///         // For all other lines we just write out what we received as input.
///         Ok(Some(line)) => {
///             writer.write_line(line)?;
///         }
///         // When we encounter `Ok(None)` it indicates that we have reached the end of the
///         // playlist and so we break the loop.
///         Ok(None) => break,
///         // Even when encountering errors we can access the original problem line, then take a
///         // mutable borrow on the inner writer, and write out the bytes. In this way we can be a
///         // very unopinionated proxy. This is completely implementation specific, and other use
///         // cases may require an implementation that rejects the playlist, or we may also choose
///         // to implement tracing in such cases. We're just showing the possibility here.
///         Err(e) => writer.get_mut().write_all(e.errored_line.as_bytes())?,
///     };
/// }
///
/// const EXPECTED: &str = r#"
/// #EXTINF:4,Hello,
/// segment_100.mp4
/// #EXTINF:4,World!
/// segment_101.mp4
/// "#;
/// assert_eq!(EXPECTED, String::from_utf8_lossy(&writer.into_inner()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Construct a playlist output
///
/// It may also be the case that a user may want to write a complete playlist out without having to
/// parse any data. This is also possible (and may be made easier in the future if we implement a
/// playlist and playlist builder type). And of course, the user can mix and match, parsing some
/// input, mutating where necessary, introducing new lines as needed, and writing it all out. Below
/// is another toy example of how we may construct the [9.4. Multivariant Playlist] example provided
/// in the HLS specification.
///
/// ```
/// # use m3u8::{
/// #     HlsLine, Writer,
/// #     tag::hls::{M3u, StreamInf},
/// # };
/// # use std::error::Error;
/// const EXPECTED: &str = r#"#EXTM3U
/// #EXT-X-STREAM-INF:BANDWIDTH=1280000,AVERAGE-BANDWIDTH=1000000
/// http://example.com/low.m3u8
/// #EXT-X-STREAM-INF:BANDWIDTH=2560000,AVERAGE-BANDWIDTH=2000000
/// http://example.com/mid.m3u8
/// #EXT-X-STREAM-INF:BANDWIDTH=7680000,AVERAGE-BANDWIDTH=6000000
/// http://example.com/hi.m3u8
/// #EXT-X-STREAM-INF:BANDWIDTH=65000,CODECS="mp4a.40.5"
/// http://example.com/audio-only.m3u8
/// "#;
///
/// let mut writer = Writer::new(Vec::new());
/// writer.write_line(HlsLine::from(M3u))?;
/// writer.write_line(HlsLine::from(
///     StreamInf::builder(1280000)
///         .with_average_bandwidth(1000000)
///         .finish(),
/// ))?;
/// writer.write_uri("http://example.com/low.m3u8")?;
/// writer.write_line(HlsLine::from(
///     StreamInf::builder(2560000)
///         .with_average_bandwidth(2000000)
///         .finish(),
/// ))?;
/// writer.write_uri("http://example.com/mid.m3u8")?;
/// writer.write_line(HlsLine::from(
///     StreamInf::builder(7680000)
///         .with_average_bandwidth(6000000)
///         .finish(),
/// ))?;
/// writer.write_uri("http://example.com/hi.m3u8")?;
/// writer.write_line(HlsLine::from(
///     StreamInf::builder(65000).with_codecs("mp4a.40.5").finish(),
/// ))?;
/// writer.write_uri("http://example.com/audio-only.m3u8")?;
///
/// assert_eq!(EXPECTED, std::str::from_utf8(&writer.into_inner())?);
/// # Ok::<(), Box<dyn Error>>(())
/// ```
///
/// [9.4. Multivariant Playlist]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-9.4
#[derive(Debug, Clone)]
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
    /// In this case the `CustomTag` generic is the default `NoCustomTag` struct. See [`Self`] for
    /// more detailed documentation.
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

    /// Write a custom tag implementation to the inner writer.
    ///
    /// Note that if the custom tag is derived from parsed data (i.e. not user constructed), then
    /// this method should be avoided, as it will allocate data perhaps unnecessarily. In that case
    /// use [`Self::write_custom_line`] with [`crate::tag::known::CustomTagAccess`], as this will
    /// use the original parsed data if no mutation has occurred.
    ///
    /// Example:
    /// ```
    /// # use m3u8::Writer;
    /// # use m3u8::tag::known::{CustomTag, ParsedTag, WritableCustomTag, WritableTag};
    /// # use m3u8::tag::value::{SemiParsedTagValue, UnparsedTagValue};
    /// # use m3u8::error::{ValidationError, ValidationErrorValueKind};
    /// # use std::borrow::Cow;
    /// #[derive(Debug, PartialEq, Clone)]
    /// struct ExampleCustomTag {
    ///     answer: u64,
    /// }
    /// impl TryFrom<ParsedTag<'_>> for ExampleCustomTag {
    ///     type Error = ValidationError;
    ///     fn try_from(tag: ParsedTag) -> Result<Self, Self::Error> {
    ///         if tag.name != "-X-MEANING-OF-LIFE" {
    ///             return Err(ValidationError::UnexpectedTagName)
    ///         }
    ///         match tag.value {
    ///             SemiParsedTagValue::Unparsed(value) => {
    ///                 Ok(Self {
    ///                     answer: value.try_as_decimal_integer()?,
    ///                 })
    ///             }
    ///             _ => Err(ValidationError::UnexpectedValueType(
    ///                 ValidationErrorValueKind::from(&tag.value)
    ///             )),
    ///         }
    ///     }
    /// }
    /// impl CustomTag<'_> for ExampleCustomTag {
    ///     fn is_known_name(name: &str) -> bool {
    ///         name == "-X-MEANING-OF-LIFE"
    ///     }
    /// }
    /// impl WritableCustomTag<'_> for ExampleCustomTag {
    ///     fn into_writable_tag(self) -> WritableTag<'static> {
    ///         WritableTag::new(
    ///             "-X-MEANING-OF-LIFE",
    ///             Cow::Owned(format!("{}", self.answer).into_bytes()),
    ///         )
    ///     }
    /// }
    ///
    /// let mut writer = Writer::new(Vec::new());
    /// let custom_tag = ExampleCustomTag { answer: 42 };
    /// writer.write_custom_tag(custom_tag).unwrap();
    /// assert_eq!(
    ///     "#EXT-X-MEANING-OF-LIFE:42\n".as_bytes(),
    ///     writer.into_inner()
    /// );
    /// ```
    pub fn write_custom_tag<'a, Custom>(&mut self, tag: Custom) -> io::Result<usize>
    where
        Custom: WritableCustomTag<'a>,
    {
        let mut count = self.write(tag.into_inner().value())?;
        count += self.write(b"\n")?;
        Ok(count)
    }

    /// Write the `HlsLine` to the underlying writer. Returns the number of bytes consumed during
    /// writing or an `io::Error` from the underlying writer. Ultimately, all the other write
    /// methods are wrappers for this method.
    ///
    /// This method is necessary to use where the input lines carry a custom tag type (other than
    /// [`crate::tag::known::NoCustomTag`]). For example, say we are parsing some data using a
    /// reader that supports our own custom defined tag (`SomeCustomTag`).
    /// ```
    /// # use m3u8::{
    /// # Reader,
    /// # config::ParsingOptions,
    /// # tag::known::{ParsedTag, CustomTag, WritableCustomTag, WritableTag},
    /// # error::ValidationError
    /// # };
    /// # use std::marker::PhantomData;
    /// # #[derive(Debug, PartialEq, Clone)]
    /// # struct SomeCustomTag;
    /// # impl TryFrom<ParsedTag<'_>> for SomeCustomTag {
    /// #     type Error = ValidationError;
    /// #     fn try_from(_: ParsedTag) -> Result<Self, Self::Error> { todo!() }
    /// # }
    /// # impl CustomTag<'_> for SomeCustomTag {
    /// #     fn is_known_name(_: &str) -> bool { todo!() }
    /// # }
    /// # impl<'a> WritableCustomTag<'a> for SomeCustomTag {
    /// #     fn into_writable_tag(self) -> WritableTag<'a> { todo!() }
    /// # }
    /// # let input = "";
    /// # let options = ParsingOptions::default();
    /// let mut reader = Reader::with_custom_from_str(
    ///     input,
    ///     options,
    ///     PhantomData::<SomeCustomTag>
    /// );
    /// ```
    /// If we tried to use the [`Self::write_line`] method, it would fail to compile (as that method
    /// expects that the generic `Custom` type is [`crate::tag::known::NoCustomTag`], which is a
    /// struct provided by the library that never succeeds the
    /// [`crate::tag::known::CustomTag::is_known_name`] check so is never parsed). Therefore we must
    /// use the `write_custom_line` method in this case (even if we are not writing the custom tag
    /// itself):
    /// ```
    /// # use m3u8::{
    /// # Reader, Writer,
    /// # config::ParsingOptions,
    /// # tag::known::{ParsedTag, CustomTag, WritableCustomTag, WritableTag},
    /// # error::ValidationError
    /// # };
    /// # use std::{error::Error, marker::PhantomData};
    /// # #[derive(Debug, PartialEq, Clone)]
    /// # struct SomeCustomTag;
    /// # impl TryFrom<ParsedTag<'_>> for SomeCustomTag {
    /// #     type Error = ValidationError;
    /// #     fn try_from(_: ParsedTag) -> Result<Self, Self::Error> { todo!() }
    /// # }
    /// # impl CustomTag<'_> for SomeCustomTag {
    /// #     fn is_known_name(_: &str) -> bool { todo!() }
    /// # }
    /// # impl<'a> WritableCustomTag<'a> for SomeCustomTag {
    /// #     fn into_writable_tag(self) -> WritableTag<'a> { todo!() }
    /// # }
    /// # let input = "";
    /// # let options = ParsingOptions::default();
    /// # let mut reader = Reader::with_custom_from_str(
    /// #     input,
    /// #     options,
    /// #     PhantomData::<SomeCustomTag>
    /// # );
    /// let mut writer = Writer::new(Vec::new());
    /// loop {
    ///     match reader.read_line() {
    ///         // --snip--
    ///         Ok(Some(line)) => {
    ///             writer.write_custom_line(line)?;
    ///         }
    ///         // --snip--
    /// #        Ok(None) => break,
    /// #        _ => todo!(),
    ///     };
    /// }
    /// # Ok::<(), Box<dyn Error>>(())
    /// ```
    pub fn write_custom_line<'a, Custom>(&mut self, line: HlsLine<'a, Custom>) -> io::Result<usize>
    where
        Custom: WritableCustomTag<'a>,
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
            HlsLine::KnownTag(t) => count += self.write(t.into_inner().value())?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::ParsingOptionsBuilder,
        error::ValidationError,
        tag::{
            hls::{self, Inf, M3u, MediaSequence, Targetduration, Version},
            known::{CustomTag, ParsedTag, WritableTag},
            value::{MutableParsedAttributeValue, MutableSemiParsedTagValue},
        },
    };
    use pretty_assertions::assert_eq;

    #[derive(Debug, PartialEq, Clone)]
    enum TestTag {
        Empty,
        Type,
        Int,
        Range,
        Float { title: &'static str },
        Date,
        List,
    }

    impl TryFrom<ParsedTag<'_>> for TestTag {
        type Error = ValidationError;

        fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
            Err(ValidationError::NotImplemented)
        }
    }

    impl CustomTag<'_> for TestTag {
        fn is_known_name(_: &str) -> bool {
            true
        }
    }

    impl WritableCustomTag<'_> for TestTag {
        fn into_writable_tag(self) -> WritableTag<'static> {
            let value = match self {
                TestTag::Empty => MutableSemiParsedTagValue::Empty,
                TestTag::Type => MutableSemiParsedTagValue::from(Cow::Borrowed(b"VOD" as &[u8])),
                TestTag::Int => MutableSemiParsedTagValue::from(Cow::Borrowed(b"42" as &[u8])),
                TestTag::Range => {
                    MutableSemiParsedTagValue::from(Cow::Borrowed(b"1024@512" as &[u8]))
                }
                TestTag::Float { title } => MutableSemiParsedTagValue::from((42.42, title)),
                TestTag::Date => MutableSemiParsedTagValue::from(Cow::Borrowed(
                    b"2025-06-17T01:37:15.129-05:00" as &[u8],
                )),
                TestTag::List => MutableSemiParsedTagValue::from([
                    ("TEST-INT", MutableParsedAttributeValue::DecimalInteger(42)),
                    (
                        "TEST-FLOAT",
                        MutableParsedAttributeValue::SignedDecimalFloatingPoint(-42.42),
                    ),
                    (
                        "TEST-QUOTED-STRING",
                        MutableParsedAttributeValue::QuotedString("test".into()),
                    ),
                    (
                        "TEST-ENUMERATED-STRING",
                        MutableParsedAttributeValue::UnquotedString("test".into()),
                    ),
                ]),
            };
            WritableTag::new("-X-TEST-TAG", value)
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

    fn string_from(test_tag: TestTag) -> String {
        let mut writer = Writer::new(Vec::new());
        writer
            .write_custom_tag(test_tag)
            .expect("should not fail to write tag");
        String::from_utf8_lossy(&writer.into_inner())
            .trim_end()
            .to_string()
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
