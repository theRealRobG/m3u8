use crate::{
    config::ParsingOptions,
    error::{ReaderBytesError, ReaderStrError},
    line::{HlsLine, parse_bytes_with_custom, parse_with_custom},
    tag::known::{CustomTag, NoCustomTag},
};
use std::marker::PhantomData;

/// A reader that parses lines of input HLS playlist data.
///
/// The `Reader` is the primary intended structure provided by the library for parsing HLS playlist
/// data. The user has the flexibility to define which of the library provided HLS tags should be
/// parsed as well as define a custom tag type to be extracted during parsing.
///
/// ## Basic usage
///
/// A reader can take an input `&str` (or `&[u8]`) and sequentially parse information about HLS
/// lines. For example, you could use the `Reader` to build up a media playlist:
/// ```
/// # use m3u8::{HlsLine, Reader};
/// # use m3u8::config::ParsingOptions;
/// # use m3u8::tag::{
/// #     hls::{ self, DiscontinuitySequence, MediaSequence, Targetduration, Version, M3u },
/// #     known,
/// # };
/// # let playlist = r#"#EXTM3U
/// # #EXT-X-TARGETDURATION:4
/// # #EXT-X-MEDIA-SEQUENCE:541647
/// # #EXT-X-VERSION:6
/// # "#;
/// #[derive(Debug, PartialEq)]
/// struct MediaPlaylist<'a> {
///     version: u64,
///     targetduration: u64,
///     media_sequence: u64,
///     discontinuity_sequence: u64,
///     // etc.
///     lines: Vec<HlsLine<'a>>,
/// }
/// let mut reader = Reader::from_str(playlist, ParsingOptions::default());
///
/// let mut version = None;
/// let mut targetduration = None;
/// let mut media_sequence = 0;
/// let mut discontinuity_sequence = 0;
/// // etc.
/// let mut lines = Vec::new();
///
/// // Validate playlist header
/// match reader.read_line() {
///     Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::M3u(tag))))) => {
///         lines.push(HlsLine::from(tag))
///     }
///     _ => return Err(format!("missing playlist header").into()),
/// }
///
/// loop {
///     match reader.read_line() {
///         Ok(Some(line)) => match line {
///             HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Version(tag))) => {
///                 version = Some(tag.version());
///                 lines.push(HlsLine::from(tag));
///             }
///             HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Targetduration(tag))) => {
///                 targetduration = Some(tag.target_duration());
///                 lines.push(HlsLine::from(tag));
///             }
///             HlsLine::KnownTag(known::Tag::Hls(hls::Tag::MediaSequence(tag))) => {
///                 media_sequence = tag.media_sequence();
///                 lines.push(HlsLine::from(tag));
///             }
///             HlsLine::KnownTag(known::Tag::Hls(hls::Tag::DiscontinuitySequence(tag))) => {
///                 discontinuity_sequence = tag.discontinuity_sequence();
///                 lines.push(HlsLine::from(tag));
///             }
///             // etc.
///             _ => lines.push(line),
///         },
///         Ok(None) => break, // End of playlist
///         Err(e) => return Err(format!("problem reading line: {e}").into()),
///     }
/// }
///
/// let version = version.unwrap_or(1);
/// let Some(targetduration) = targetduration else {
///     return Err("missing required EXT-X-TARGETDURATION".into());
/// };
/// let media_playlist = MediaPlaylist {
///     version,
///     targetduration,
///     media_sequence,
///     discontinuity_sequence,
///     lines,
/// };
///
/// assert_eq!(
///     media_playlist,
///     MediaPlaylist {
///         version: 6,
///         targetduration: 4,
///         media_sequence: 541647,
///         discontinuity_sequence: 0,
///         lines: vec![
///             // --snip--
/// #            HlsLine::from(M3u),
/// #            HlsLine::from(Targetduration::new(4)),
/// #            HlsLine::from(MediaSequence::new(541647)),
/// #            HlsLine::from(Version::new(6)),
///         ],
///     }
/// );
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Configuring known tags
///
/// It is quite common that a user does not need to support parsing of all HLS tags for their use-
/// case. To support this better the `Reader` allows for configuration of what HLS tags are
/// considered "known" by the library. While it may sound strange to configure for less information
/// to be parsed, doing so can have significant performance benefits, and at no loss if the
/// information is not needed anyway. Unknown tags make no attempt to parse or validate the value
/// portion of the tag (the part after `:`) and just provide the name of the tag along with the line
/// up to (and not including) the new line characters. To provide some indication of the performance
/// difference, running locally (as of commit `6fcc38a67bf0eee0769b7e85f82599d1da6eb56d`), the
/// benchmarks show that on a very large media playlist parsing with all tags can be around 2x
/// slower than parsing with no tags (`2.3842 ms` vs `1.1364 ms`):
/// ```sh
/// Large playlist, all tags, using Reader::from_str, no writing
///                         time:   [2.3793 ms 2.3842 ms 2.3891 ms]
/// Large playlist, no tags, using Reader::from_str, no writing
///                         time:   [1.1357 ms 1.1364 ms 1.1372 ms]
/// ```
///
/// For example, let's say that we are updating a playlist to add in HLS interstitial daterange,
/// based on SCTE35-OUT information in an upstream playlist. The only tag we need to know about for
/// this is EXT-X-DATERANGE, so we can configure our reader to only consider this tag during parsing
/// which provides a benefit in terms of processing time.
/// ```
/// # use m3u8::{
/// # Reader, HlsLine, Writer,
/// # config::ParsingOptionsBuilder,
/// # tag::known,
/// # tag::hls::{self, Cue, Daterange, ExtensionAttributeValue},
/// # };
/// # use std::{borrow::Cow, error::Error, io::Write};
/// # fn advert_id_from_scte35_out(_: &str) -> Option<String> { None }
/// # fn advert_uri_from_id(_: &str) -> String { String::new() }
/// # fn duration_from_daterange(_: &Daterange) -> f64 { 0.0 }
/// # let output = Vec::new();
/// # let upstream_playlist = b"";
/// let mut reader = Reader::from_bytes(
///     upstream_playlist,
///     ParsingOptionsBuilder::new()
///         .with_parsing_for_daterange()
///         .build(),
/// );
/// let mut writer = Writer::new(output);
///
/// loop {
///     match reader.read_line() {
///         Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Daterange(tag))))) => {
///             if let Some(advert_id) = tag.scte35_out().and_then(advert_id_from_scte35_out) {
///                 let id = format!("ADVERT:{}", tag.id());
///                 let builder = Daterange::builder(id, tag.start_date())
///                     .with_class("com.apple.hls.interstitial")
///                     .with_cue(Cue::Once)
///                     .with_extension_attribute(
///                         "X-ASSET-URI",
///                         ExtensionAttributeValue::QuotedString(Cow::Owned(
///                             advert_uri_from_id(&advert_id),
///                         )),
///                     )
///                     .with_extension_attribute(
///                         "X-RESTRICT",
///                         ExtensionAttributeValue::QuotedString(Cow::Borrowed("SKIP,JUMP")),
///                     );
///                 let interstitial_daterange = if duration_from_daterange(&tag) == 0.0 {
///                     builder
///                         .with_extension_attribute(
///                             "X-RESUME-OFFSET",
///                             ExtensionAttributeValue::SignedDecimalFloatingPoint(0.0),
///                         )
///                         .finish()
///                 } else {
///                     builder.finish()
///                 };
///                 writer.write_line(HlsLine::from(interstitial_daterange))?;
///             } else {
///                 writer.write_line(HlsLine::from(tag))?;
///             }
///         }
///         Ok(Some(line)) => {
///             writer.write_line(line)?;
///         }
///         Ok(None) => break, // End of playlist
///         Err(e) => {
///             writer.get_mut().write_all(e.errored_line)?;
///         }
///     };
/// }
///
/// writer.into_inner().flush()?;
/// # Ok::<(), Box<dyn Error>>(())
/// ```
///
/// ## Custom tag reading
///
/// We can also configure the `Reader` to accept parsing of custom defined tags. Using the same idea
/// as above, we can imagine that instead of EXT-X-DATERANGE in the upstream playlist, we want to
/// depend on the EXT-X-SCTE35 tag that is defined within the SCTE35 specification. This tag is not
/// defined in the HLS specification; however, we can define it here, and use it when it comes to
/// parsing and utilizing that data. Below is a modified version of the above HLS interstitials
/// example that instead relies on a custom defined `Scte35Tag` (though I leave the details of
/// `TryFrom<ParsedTag>` unfilled for sake of simplicity in this example). Note, when defining a
/// that the reader should use a custom tag, utilize `std::marker::PhantomData` to specify what the
/// type of the custom tag is.
/// ```
/// # use m3u8::{
/// # Reader, HlsLine, Writer,
/// # config::ParsingOptionsBuilder,
/// # date::DateTime,
/// # tag::known::{self, ParsedTag, CustomTag, WritableCustomTag},
/// # tag::hls::{self, Cue, Daterange, ExtensionAttributeValue},
/// # error::ValidationError,
/// # };
/// # use std::{borrow::Cow, error::Error, io::Write, marker::PhantomData};
/// # fn advert_id_from_scte35_out(_: &str) -> Option<String> { None }
/// # fn advert_uri_from_id(_: &str) -> String { String::new() }
/// # fn generate_uuid() -> &'static str { "" }
/// # fn calculate_start_date_based_on_inf_durations() -> DateTime { todo!() }
/// # let output: Vec<u8> = Vec::new();
/// # let upstream_playlist = b"";
/// #[derive(Debug, PartialEq, Clone)]
/// struct Scte35Tag<'a> {
///     cue: &'a str,
///     duration: Option<f64>,
///     elapsed: Option<f64>,
///     id: Option<&'a str>,
///     time: Option<f64>,
///     type_id: Option<u64>,
///     upid: Option<&'a str>,
///     blackout: Option<BlackoutValue>,
///     cue_out: Option<CueOutValue>,
///     cue_in: bool,
///     segne: Option<(u64, u64)>,
/// }
/// #[derive(Debug, PartialEq, Clone)]
/// enum BlackoutValue {
///     Yes,
///     No,
///     Maybe,
/// }
/// #[derive(Debug, PartialEq, Clone)]
/// enum CueOutValue {
///     Yes,
///     No,
///     Cont,
/// }
/// impl<'a> TryFrom<ParsedTag<'a>> for Scte35Tag<'a> { // --snip--
/// #    type Error = ValidationError;
/// #    fn try_from(value: ParsedTag<'a>) -> Result<Self, Self::Error> {
/// #        todo!()
/// #    }
/// }
/// impl<'a> CustomTag<'a> for Scte35Tag<'a> {
///     fn is_known_name(name: &str) -> bool {
///         name == "-X-SCTE35"
///     }
/// }
/// impl<'a> WritableCustomTag<'a> for Scte35Tag<'a> { // --snip--
/// #    fn into_writable_tag(self) -> known::WritableTag<'a> {
/// #        todo!()
/// #    }
/// }
/// #
/// # let output: Vec<u8> = Vec::new();
/// # let upstream_playlist = b"";
///
/// let mut reader = Reader::with_custom_from_bytes(
///     upstream_playlist,
///     ParsingOptionsBuilder::new()
///         .with_parsing_for_daterange()
///         .build(),
///     PhantomData::<Scte35Tag>,
/// );
/// let mut writer = Writer::new(output);
///
/// loop {
///     match reader.read_line() {
///         Ok(Some(HlsLine::KnownTag(known::Tag::Custom(tag)))) => {
///             if let Some(advert_id) = advert_id_from_scte35_out(tag.as_ref().cue) {
///                 let tag_ref = tag.as_ref();
///                 let id = format!("ADVERT:{}", tag_ref.id.unwrap_or(generate_uuid()));
///                 let start_date = calculate_start_date_based_on_inf_durations();
///                 let builder = Daterange::builder(id, start_date)
///                     .with_class("com.apple.hls.interstitial")
///                     .with_cue(Cue::Once)
///                     .with_extension_attribute(
///                         "X-ASSET-URI",
///                         ExtensionAttributeValue::QuotedString(Cow::Owned(
///                             advert_uri_from_id(&advert_id),
///                         )),
///                     )
///                     .with_extension_attribute(
///                         "X-RESTRICT",
///                         ExtensionAttributeValue::QuotedString(Cow::Borrowed("SKIP,JUMP")),
///                     );
///                 let interstitial_daterange = if tag_ref.duration == Some(0.0) {
///                     builder
///                         .with_extension_attribute(
///                             "X-RESUME-OFFSET",
///                             ExtensionAttributeValue::SignedDecimalFloatingPoint(0.0),
///                         )
///                         .finish()
///                 } else {
///                     builder.finish()
///                 };
///                 writer.write_line(HlsLine::from(interstitial_daterange))?;
///             } else {
///                 writer.write_custom_line(HlsLine::from(tag))?;
///             }
///         }
///         Ok(Some(line)) => {
///             writer.write_custom_line(line)?;
///         }
///         Ok(None) => break, // End of playlist
///         Err(e) => {
///             writer.get_mut().write_all(e.errored_line)?;
///         }
///     };
/// }
///
/// writer.into_inner().flush()?;
///
/// # Ok::<(), Box<dyn Error>>(())
/// ```
pub struct Reader<R, Custom> {
    inner: R,
    options: ParsingOptions,
    _marker: PhantomData<Custom>,
}

macro_rules! impl_reader {
    ($type:ty, $parse_fn:ident, $from_fn_ident:ident, $from_custom_fn_ident:ident, $error_type:ident) => {
        impl<'a> Reader<&'a $type, NoCustomTag> {
            /// Creates a reader without custom tag parsing support (in this case, the generic
            /// `Custom` type is [`NoCustomTag`]).
            pub fn $from_fn_ident(data: &'a $type, options: ParsingOptions) -> Self {
                Self {
                    inner: data,
                    options,
                    _marker: PhantomData::<NoCustomTag>,
                }
            }
        }
        impl<'a, Custom> Reader<&'a $type, Custom>
        where
            Custom: CustomTag<'a>,
        {
            /// Creates a reader that supports custom tag parsing for the type specified by the
            /// `PhatomData`.
            pub fn $from_custom_fn_ident(
                str: &'a $type,
                options: ParsingOptions,
                custom: PhantomData<Custom>,
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
            pub fn read_line(&mut self) -> Result<Option<HlsLine<'a, Custom>>, $error_type<'a>> {
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
    use super::*;
    use crate::{
        config::ParsingOptionsBuilder,
        error::{SyntaxError, UnknownTagSyntaxError, ValidationError, ValidationErrorValueKind},
        tag::{
            hls::{Endlist, Inf, M3u, Targetduration, Version},
            known::{CustomTagAccess, ParsedTag},
            unknown,
            value::{ParsedAttributeValue, SemiParsedTagValue},
        },
    };
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
            Some(HlsLine::from(CustomTagAccess {
                custom_tag: ExampleTag::new(42, "UNKNOWN"),
                is_dirty: false,
                original_input: EXAMPLE_MANIFEST[50..].as_bytes(),
            }))
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
            Some(HlsLine::from(CustomTagAccess {
                custom_tag: ExampleTag::new(42, "UNKNOWN"),
                is_dirty: false,
                original_input: EXAMPLE_MANIFEST[50..].as_bytes(),
            }))
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
            let SemiParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                return Err(ValidationError::UnexpectedValueType(
                    ValidationErrorValueKind::AttributeList,
                ));
            };
            let Some(ParsedAttributeValue::DecimalInteger(answer)) =
                attribute_list.remove("MEANING-OF-LIFE")
            else {
                return Err(ValidationError::MissingRequiredAttribute("MEANING-OF-LIFE"));
            };
            let Some(ParsedAttributeValue::QuotedString(question)) =
                attribute_list.remove("QUESTION")
            else {
                return Err(ValidationError::MissingRequiredAttribute("QUESTION"));
            };
            Ok(Self { answer, question })
        }
    }
    impl<'a> CustomTag<'a> for ExampleTag<'a> {
        fn is_known_name(name: &str) -> bool {
            name == "-X-EXAMPLE-TAG"
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
