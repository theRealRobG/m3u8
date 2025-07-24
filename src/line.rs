use crate::{
    config::ParsingOptions,
    error::{ParseLineBytesError, ParseLineStrError, SyntaxError, ValidationError},
    tag::{
        self, hls,
        known::{self, IsKnownName, NoCustomTag, ParsedTag, TagInformation},
        unknown,
    },
    utils::{split_on_new_line, str_from},
};
use std::{cmp::PartialEq, fmt::Debug};

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)] // See comment on crate::tag::known::Tag.
pub enum HlsLine<'a, CustomTag = NoCustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + Clone
        + PartialEq,
{
    KnownTag(known::Tag<'a, CustomTag>),
    UnknownTag(unknown::Tag<'a>),
    Comment(&'a str),
    Uri(&'a str),
    Blank,
}

impl<'a, CustomTag> From<hls::Tag<'a>> for HlsLine<'a, CustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + Clone
        + PartialEq,
{
    fn from(tag: hls::Tag<'a>) -> Self {
        Self::KnownTag(known::Tag::Hls(tag))
    }
}

impl<'a, CustomTag> From<CustomTag> for HlsLine<'a, CustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + Clone
        + PartialEq,
{
    fn from(tag: CustomTag) -> Self {
        Self::KnownTag(known::Tag::Custom(tag))
    }
}

impl<'a, CustomTag> From<unknown::Tag<'a>> for HlsLine<'a, CustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + Clone
        + PartialEq,
{
    fn from(tag: unknown::Tag<'a>) -> Self {
        Self::UnknownTag(tag)
    }
}

impl<'a> HlsLine<'a> {
    pub fn new_comment(comment: &'a str) -> Self {
        Self::Comment(comment)
    }

    pub fn new_uri(uri: &'a str) -> Self {
        Self::Uri(uri)
    }

    pub fn new_blank() -> Self {
        Self::Blank
    }
}

macro_rules! impl_line_from_tag {
    ($tag_mod_path:path, $tag_name:ident) => {
        impl<'a, CustomTag> From<$tag_mod_path> for HlsLine<'a, CustomTag>
        where
            CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
                + IsKnownName
                + TagInformation
                + Debug
                + Clone
                + PartialEq,
        {
            fn from(tag: $tag_mod_path) -> Self {
                Self::KnownTag($crate::tag::known::Tag::Hls(
                    $crate::tag::hls::Tag::$tag_name(tag),
                ))
            }
        }
    };
}

impl_line_from_tag!(hls::m3u::M3u, M3u);
impl_line_from_tag!(hls::version::Version<'a>, Version);
impl_line_from_tag!(
    hls::independent_segments::IndependentSegments,
    IndependentSegments
);
impl_line_from_tag!(hls::start::Start<'a>, Start);
impl_line_from_tag!(hls::define::Define<'a>, Define);
impl_line_from_tag!(hls::targetduration::Targetduration<'a>, Targetduration);
impl_line_from_tag!(hls::media_sequence::MediaSequence<'a>, MediaSequence);
impl_line_from_tag!(
    hls::discontinuity_sequence::DiscontinuitySequence<'a>,
    DiscontinuitySequence
);
impl_line_from_tag!(hls::endlist::Endlist, Endlist);
impl_line_from_tag!(hls::playlist_type::PlaylistType, PlaylistType);
impl_line_from_tag!(hls::i_frames_only::IFramesOnly, IFramesOnly);
impl_line_from_tag!(hls::part_inf::PartInf<'a>, PartInf);
impl_line_from_tag!(hls::server_control::ServerControl<'a>, ServerControl);
impl_line_from_tag!(hls::inf::Inf<'a>, Inf);
impl_line_from_tag!(hls::byterange::Byterange<'a>, Byterange);
impl_line_from_tag!(hls::discontinuity::Discontinuity, Discontinuity);
impl_line_from_tag!(hls::key::Key<'a>, Key);
impl_line_from_tag!(hls::map::Map<'a>, Map);
impl_line_from_tag!(hls::program_date_time::ProgramDateTime<'a>, ProgramDateTime);
impl_line_from_tag!(hls::gap::Gap, Gap);
impl_line_from_tag!(hls::bitrate::Bitrate<'a>, Bitrate);
impl_line_from_tag!(hls::part::Part<'a>, Part);
impl_line_from_tag!(hls::daterange::Daterange<'a>, Daterange);
impl_line_from_tag!(hls::skip::Skip<'a>, Skip);
impl_line_from_tag!(hls::preload_hint::PreloadHint<'a>, PreloadHint);
impl_line_from_tag!(hls::rendition_report::RenditionReport<'a>, RenditionReport);
impl_line_from_tag!(hls::media::Media<'a>, Media);
impl_line_from_tag!(hls::stream_inf::StreamInf<'a>, StreamInf);
impl_line_from_tag!(
    hls::i_frame_stream_inf::IFrameStreamInf<'a>,
    IFrameStreamInf
);
impl_line_from_tag!(hls::session_data::SessionData<'a>, SessionData);
impl_line_from_tag!(hls::session_key::SessionKey<'a>, SessionKey);
impl_line_from_tag!(hls::content_steering::ContentSteering<'a>, ContentSteering);

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedLineSlice<'a, T>
where
    T: Debug + PartialEq,
{
    pub parsed: T,
    pub remaining: Option<&'a str>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct ParsedByteSlice<'a, T>
where
    T: Debug + PartialEq,
{
    pub parsed: T,
    pub remaining: Option<&'a [u8]>,
}

pub fn parse<'a>(
    input: &'a str,
    options: &ParsingOptions,
) -> Result<ParsedLineSlice<'a, HlsLine<'a>>, ParseLineStrError<'a>> {
    parse_with_custom::<NoCustomTag>(input, options)
}

pub fn parse_with_custom<'a, 'b, CustomTag>(
    input: &'a str,
    options: &'b ParsingOptions,
) -> Result<ParsedLineSlice<'a, HlsLine<'a, CustomTag>>, ParseLineStrError<'a>>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + Clone
        + PartialEq,
{
    parse_bytes_with_custom(input.as_bytes(), options)
        // These conversions from ParsedByteSlice to ParsedLineSlice are only safe here because we
        // know that these must represent valid UTF-8.
        .map(|r| ParsedLineSlice {
            parsed: r.parsed,
            remaining: r.remaining.map(str_from),
        })
        .map_err(|error| ParseLineStrError {
            errored_line_slice: ParsedLineSlice {
                parsed: str_from(error.errored_line_slice.parsed),
                remaining: error.errored_line_slice.remaining.map(str_from),
            },
            error: error.error,
        })
}

pub fn parse_bytes<'a>(
    input: &'a [u8],
    options: &ParsingOptions,
) -> Result<ParsedByteSlice<'a, HlsLine<'a>>, ParseLineBytesError<'a>> {
    parse_bytes_with_custom::<NoCustomTag>(input, options)
}

pub fn parse_bytes_with_custom<'a, 'b, CustomTag>(
    input: &'a [u8],
    options: &'b ParsingOptions,
) -> Result<ParsedByteSlice<'a, HlsLine<'a, CustomTag>>, ParseLineBytesError<'a>>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + Clone
        + PartialEq,
{
    if input.is_empty() {
        Ok(ParsedByteSlice {
            parsed: HlsLine::Blank,
            remaining: None,
        })
    } else if input[0] == b'#' {
        if input.get(3) == Some(&b'T') && &input[..3] == b"#EX" {
            let tag_rest = &input[4..];
            let mut tag = tag::unknown::parse_assuming_ext_taken(tag_rest, input)
                .map_err(|error| map_err_bytes(error, input))?;
            if options.is_known_name(tag.parsed.name) || CustomTag::is_known_name(tag.parsed.name) {
                let value_slice = match tag.parsed.value {
                    None => ParsedByteSlice {
                        parsed: tag::value::SemiParsedTagValue::Empty,
                        remaining: None,
                    },
                    Some(remaining) => tag::value::new_parse(remaining)
                        .map_err(|error| map_err_bytes(error, input))?,
                };
                let parsed_tag = ParsedTag {
                    name: tag.parsed.name,
                    value: value_slice.parsed,
                    original_input: input,
                };
                match known::Tag::try_from(parsed_tag) {
                    Ok(known_tag) => Ok(ParsedByteSlice {
                        parsed: HlsLine::KnownTag(known_tag),
                        remaining: tag.remaining,
                    }),
                    Err(e) => {
                        tag.parsed.validation_error = Some(e);
                        Ok(ParsedByteSlice {
                            parsed: HlsLine::UnknownTag(tag.parsed),
                            remaining: tag.remaining,
                        })
                    }
                }
            } else {
                Ok(ParsedByteSlice {
                    parsed: HlsLine::UnknownTag(tag.parsed),
                    remaining: tag.remaining,
                })
            }
        } else {
            let ParsedByteSlice { parsed, remaining } = split_on_new_line(&input[1..]);
            let comment =
                std::str::from_utf8(parsed).map_err(|error| map_err_bytes(error, input))?;
            Ok(ParsedByteSlice {
                parsed: HlsLine::Comment(comment),
                remaining,
            })
        }
    } else {
        let ParsedByteSlice { parsed, remaining } = split_on_new_line(input);
        let uri = std::str::from_utf8(parsed).map_err(|error| map_err_bytes(error, input))?;
        Ok(ParsedByteSlice {
            parsed: HlsLine::Uri(uri),
            remaining,
        })
    }
}

fn map_err_bytes<E: Into<SyntaxError>>(error: E, input: &[u8]) -> ParseLineBytesError {
    let errored_line_slice = split_on_new_line(input);
    ParseLineBytesError {
        errored_line_slice,
        error: error.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        config::ParsingOptionsBuilder,
        error::ValidationErrorValueKind,
        tag::{
            hls::{self, m3u::M3u, start::Start},
            value::{ParsedAttributeValue, SemiParsedTagValue},
        },
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn uri_line() {
        assert_eq!(
            Ok(HlsLine::Uri("hello/world.m3u8")),
            parse("hello/world.m3u8", &ParsingOptions::default()).map(|p| p.parsed)
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(
            Ok(HlsLine::Blank),
            parse("", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn comment() {
        assert_eq!(
            Ok(HlsLine::Comment("Comment")),
            parse("#Comment", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok(HlsLine::from(hls::Tag::M3u(M3u))),
            parse("#EXTM3U", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn custom_tag() {
        // Set up custom tag
        #[derive(Debug, PartialEq, Clone)]
        struct TestTag<'a> {
            greeting_type: &'a str,
            message: &'a str,
            times: u64,
            score: Option<f64>,
        }
        impl<'a> TryFrom<ParsedTag<'a>> for TestTag<'a> {
            type Error = ValidationError;

            fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
                match &tag.value {
                    tag::value::SemiParsedTagValue::AttributeList(list) => {
                        let Some(tag::value::ParsedAttributeValue::UnquotedString(greeting_type)) =
                            list.get("TYPE")
                        else {
                            return Err(ValidationError::MissingRequiredAttribute("TYPE"));
                        };
                        let Some(tag::value::ParsedAttributeValue::QuotedString(message)) =
                            list.get("MESSAGE")
                        else {
                            return Err(ValidationError::MissingRequiredAttribute("MESSAGE"));
                        };
                        let Some(tag::value::ParsedAttributeValue::DecimalInteger(times)) =
                            list.get("TIMES")
                        else {
                            return Err(ValidationError::MissingRequiredAttribute("TIMES"));
                        };
                        let score = list
                            .get("SCORE")
                            .map(tag::value::ParsedAttributeValue::as_option_f64)
                            .flatten();
                        Ok(Self {
                            greeting_type,
                            message,
                            times: *times,
                            score,
                        })
                    }
                    v => Err(ValidationError::UnexpectedValueType(
                        ValidationErrorValueKind::from(v),
                    )),
                }
            }
        }
        impl IsKnownName for TestTag<'_> {
            fn is_known_name(name: &str) -> bool {
                name == "-X-TEST-TAG"
            }
        }
        impl<'a> TagInformation for TestTag<'a> {
            fn name(&self) -> &str {
                "-X-TEST-TAG"
            }

            fn value(&self) -> tag::value::SemiParsedTagValue {
                let mut attribute_list = HashMap::new();
                attribute_list.insert(
                    "TYPE",
                    ParsedAttributeValue::UnquotedString(self.greeting_type),
                );
                attribute_list.insert("MESSAGE", ParsedAttributeValue::QuotedString(self.message));
                attribute_list.insert("TIMES", ParsedAttributeValue::DecimalInteger(self.times));
                if let Some(score) = self.score {
                    attribute_list.insert(
                        "SCORE",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(score),
                    );
                }
                SemiParsedTagValue::AttributeList(attribute_list)
            }
        }
        // Test
        assert_eq!(
            Ok(HlsLine::from(TestTag {
                greeting_type: "GREETING",
                message: "Hello, World!",
                times: 42,
                score: None,
            })),
            parse_with_custom::<TestTag>(
                "#EXT-X-TEST-TAG:TYPE=GREETING,MESSAGE=\"Hello, World!\",TIMES=42",
                &ParsingOptions::default()
            )
            .map(|p| p.parsed)
        );
    }

    #[test]
    fn avoiding_parsing_known_tag_when_configured_to_avoid_via_parsing_options() {
        assert_eq!(
            Ok(HlsLine::from(hls::Tag::Start(
                Start::builder(-18.0).finish()
            ))),
            parse("#EXT-X-START:TIME-OFFSET=-18", &ParsingOptions::default()).map(|p| p.parsed)
        );
        assert_eq!(
            Ok(HlsLine::UnknownTag(unknown::Tag {
                name: "-X-START",
                value: Some(b"TIME-OFFSET=-18"),
                original_input: b"#EXT-X-START:TIME-OFFSET=-18",
                validation_error: None,
            })),
            parse(
                "#EXT-X-START:TIME-OFFSET=-18",
                &ParsingOptionsBuilder::new()
                    .with_parsing_for_all_tags()
                    .without_parsing_for_start()
                    .build()
            )
            .map(|p| p.parsed)
        );
    }
}
