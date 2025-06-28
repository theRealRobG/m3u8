use std::borrow::Cow;

use crate::{
    error::ValidationError,
    tag::{
        hls::{
            bitrate::Bitrate, byterange::Byterange, content_steering::ContentSteering,
            daterange::Daterange, define::Define, discontinuity::Discontinuity,
            discontinuity_sequence::DiscontinuitySequence, endlist::Endlist, gap::Gap,
            i_frame_stream_inf::IFrameStreamInf, i_frames_only::IFramesOnly,
            independent_segments::IndependentSegments, inf::Inf, key::Key, m3u::M3u, map::Map,
            media::Media, media_sequence::MediaSequence, part::Part, part_inf::PartInf,
            playlist_type::PlaylistType, preload_hint::PreloadHint,
            program_date_time::ProgramDateTime, rendition_report::RenditionReport,
            server_control::ServerControl, session_data::SessionData, session_key::SessionKey,
            skip::Skip, start::Start, stream_inf::StreamInf, targetduration::Targetduration,
            version::Version,
        },
        known::ParsedTag,
    },
    utils::split_by_first_lf,
};

pub mod bitrate;
pub mod byterange;
pub mod content_steering;
pub mod daterange;
pub mod define;
pub mod discontinuity;
pub mod discontinuity_sequence;
pub mod endlist;
pub mod gap;
pub mod i_frame_stream_inf;
pub mod i_frames_only;
pub mod independent_segments;
pub mod inf;
pub mod key;
pub mod m3u;
pub mod map;
pub mod media;
pub mod media_sequence;
pub mod part;
pub mod part_inf;
pub mod playlist_type;
pub mod preload_hint;
pub mod program_date_time;
pub mod rendition_report;
pub mod server_control;
pub mod session_data;
pub mod session_key;
pub mod skip;
pub mod start;
pub mod stream_inf;
pub mod targetduration;
pub mod version;

#[derive(Debug, PartialEq)]
pub enum Tag<'a> {
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.1
    M3u(M3u),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.2
    Version(Version<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.1
    IndependentSegments(IndependentSegments),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2
    Start(Start<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.3
    Define(Define<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.1
    Targetduration(Targetduration<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.2
    MediaSequence(MediaSequence<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.3
    DiscontinuitySequence(DiscontinuitySequence<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.4
    Endlist(Endlist),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.5
    PlaylistType(PlaylistType),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.6
    IFramesOnly(IFramesOnly),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.7
    PartInf(PartInf<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.8
    ServerControl(ServerControl<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.1
    Inf(Inf<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.2
    Byterange(Byterange<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.3
    Discontinuity(Discontinuity),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.4
    Key(Key<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5
    Map(Map<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.6
    ProgramDateTime(ProgramDateTime<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.7
    Gap(Gap),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.8
    Bitrate(Bitrate<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.9
    Part(Part<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.1
    Daterange(Daterange<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.2
    Skip(Skip<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.3
    PreloadHint(PreloadHint<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.4
    RenditionReport(RenditionReport<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.1
    Media(Media<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.2
    StreamInf(StreamInf<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.3
    IFrameStreamInf(IFrameStreamInf<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.4
    SessionData(SessionData<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.5
    SessionKey(SessionKey<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.6
    ContentSteering(ContentSteering<'a>),
}

impl<'a> TryFrom<ParsedTag<'a>> for Tag<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let tag_name = TagName::try_from(tag.name)?;
        match tag_name {
            TagName::M3u => Ok(Self::M3u(M3u::try_from(tag)?)),
            TagName::Version => Ok(Self::Version(Version::try_from(tag)?)),
            TagName::IndependentSegments => Ok(Self::IndependentSegments(
                IndependentSegments::try_from(tag)?,
            )),
            TagName::Start => Ok(Self::Start(Start::try_from(tag)?)),
            TagName::Define => Ok(Self::Define(Define::try_from(tag)?)),
            TagName::Targetduration => Ok(Self::Targetduration(Targetduration::try_from(tag)?)),
            TagName::MediaSequence => Ok(Self::MediaSequence(MediaSequence::try_from(tag)?)),
            TagName::DiscontinuitySequence => Ok(Self::DiscontinuitySequence(
                DiscontinuitySequence::try_from(tag)?,
            )),
            TagName::Endlist => Ok(Self::Endlist(Endlist::try_from(tag)?)),
            TagName::PlaylistType => Ok(Self::PlaylistType(PlaylistType::try_from(tag)?)),
            TagName::IFramesOnly => Ok(Self::IFramesOnly(IFramesOnly::try_from(tag)?)),
            TagName::PartInf => Ok(Self::PartInf(PartInf::try_from(tag)?)),
            TagName::ServerControl => Ok(Self::ServerControl(ServerControl::try_from(tag)?)),
            TagName::Inf => Ok(Self::Inf(Inf::try_from(tag)?)),
            TagName::Byterange => Ok(Self::Byterange(Byterange::try_from(tag)?)),
            TagName::Discontinuity => Ok(Self::Discontinuity(Discontinuity::try_from(tag)?)),
            TagName::Key => Ok(Self::Key(Key::try_from(tag)?)),
            TagName::Map => Ok(Self::Map(Map::try_from(tag)?)),
            TagName::ProgramDateTime => Ok(Self::ProgramDateTime(ProgramDateTime::try_from(tag)?)),
            TagName::Gap => Ok(Self::Gap(Gap::try_from(tag)?)),
            TagName::Bitrate => Ok(Self::Bitrate(Bitrate::try_from(tag)?)),
            TagName::Part => Ok(Self::Part(Part::try_from(tag)?)),
            TagName::Daterange => Ok(Self::Daterange(Daterange::try_from(tag)?)),
            TagName::Skip => Ok(Self::Skip(Skip::try_from(tag)?)),
            TagName::PreloadHint => Ok(Self::PreloadHint(PreloadHint::try_from(tag)?)),
            TagName::RenditionReport => Ok(Self::RenditionReport(RenditionReport::try_from(tag)?)),
            TagName::Media => Ok(Self::Media(Media::try_from(tag)?)),
            TagName::StreamInf => Ok(Self::StreamInf(StreamInf::try_from(tag)?)),
            TagName::IFrameStreamInf => Ok(Self::IFrameStreamInf(IFrameStreamInf::try_from(tag)?)),
            TagName::SessionData => Ok(Self::SessionData(SessionData::try_from(tag)?)),
            TagName::SessionKey => Ok(Self::SessionKey(SessionKey::try_from(tag)?)),
            TagName::ContentSteering => Ok(Self::ContentSteering(ContentSteering::try_from(tag)?)),
        }
    }
}

pub struct TagInner<'a> {
    output_line: Cow<'a, str>,
}

impl<'a> TagInner<'a> {
    pub fn value(&self) -> &str {
        split_by_first_lf(&self.output_line).parsed
    }
}

impl<'a> Tag<'a> {
    pub fn into_inner(self) -> TagInner<'a> {
        match self {
            Tag::M3u(t) => t.into_inner(),
            Tag::Version(t) => t.into_inner(),
            Tag::IndependentSegments(t) => t.into_inner(),
            Tag::Start(t) => t.into_inner(),
            Tag::Define(t) => t.into_inner(),
            Tag::Targetduration(t) => t.into_inner(),
            Tag::MediaSequence(t) => t.into_inner(),
            Tag::DiscontinuitySequence(t) => t.into_inner(),
            Tag::Endlist(t) => t.into_inner(),
            Tag::PlaylistType(t) => t.into_inner(),
            Tag::IFramesOnly(t) => t.into_inner(),
            Tag::PartInf(t) => t.into_inner(),
            Tag::ServerControl(t) => t.into_inner(),
            Tag::Inf(t) => t.into_inner(),
            Tag::Byterange(t) => t.into_inner(),
            Tag::Discontinuity(t) => t.into_inner(),
            Tag::Key(t) => t.into_inner(),
            Tag::Map(t) => t.into_inner(),
            Tag::ProgramDateTime(t) => t.into_inner(),
            Tag::Gap(t) => t.into_inner(),
            Tag::Bitrate(t) => t.into_inner(),
            Tag::Part(t) => t.into_inner(),
            Tag::Daterange(t) => t.into_inner(),
            Tag::Skip(t) => t.into_inner(),
            Tag::PreloadHint(t) => t.into_inner(),
            Tag::RenditionReport(t) => t.into_inner(),
            Tag::Media(t) => t.into_inner(),
            Tag::StreamInf(t) => t.into_inner(),
            Tag::IFrameStreamInf(t) => t.into_inner(),
            Tag::SessionData(t) => t.into_inner(),
            Tag::SessionKey(t) => t.into_inner(),
            Tag::ContentSteering(t) => t.into_inner(),
        }
    }
}

impl Tag<'_> {
    pub fn name(&self) -> TagName {
        match self {
            Tag::M3u(_) => TagName::M3u,
            Tag::Version(_) => TagName::Version,
            Tag::IndependentSegments(_) => TagName::IndependentSegments,
            Tag::Start(_) => TagName::Start,
            Tag::Define(_) => TagName::Define,
            Tag::Targetduration(_) => TagName::Targetduration,
            Tag::MediaSequence(_) => TagName::MediaSequence,
            Tag::DiscontinuitySequence(_) => TagName::DiscontinuitySequence,
            Tag::Endlist(_) => TagName::Endlist,
            Tag::PlaylistType(_) => TagName::PlaylistType,
            Tag::IFramesOnly(_) => TagName::IFramesOnly,
            Tag::PartInf(_) => TagName::PartInf,
            Tag::ServerControl(_) => TagName::ServerControl,
            Tag::Inf(_) => TagName::Inf,
            Tag::Byterange(_) => TagName::Byterange,
            Tag::Discontinuity(_) => TagName::Discontinuity,
            Tag::Key(_) => TagName::Key,
            Tag::Map(_) => TagName::Map,
            Tag::ProgramDateTime(_) => TagName::ProgramDateTime,
            Tag::Gap(_) => TagName::Gap,
            Tag::Bitrate(_) => TagName::Bitrate,
            Tag::Part(_) => TagName::Part,
            Tag::Daterange(_) => TagName::Daterange,
            Tag::Skip(_) => TagName::Skip,
            Tag::PreloadHint(_) => TagName::PreloadHint,
            Tag::RenditionReport(_) => TagName::RenditionReport,
            Tag::Media(_) => TagName::Media,
            Tag::StreamInf(_) => TagName::StreamInf,
            Tag::IFrameStreamInf(_) => TagName::IFrameStreamInf,
            Tag::SessionData(_) => TagName::SessionData,
            Tag::SessionKey(_) => TagName::SessionKey,
            Tag::ContentSteering(_) => TagName::ContentSteering,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TagName {
    M3u,
    Version,
    IndependentSegments,
    Start,
    Define,
    Targetduration,
    MediaSequence,
    DiscontinuitySequence,
    Endlist,
    PlaylistType,
    IFramesOnly,
    PartInf,
    ServerControl,
    Inf,
    Byterange,
    Discontinuity,
    Key,
    Map,
    ProgramDateTime,
    Gap,
    Bitrate,
    Part,
    Daterange,
    Skip,
    PreloadHint,
    RenditionReport,
    Media,
    StreamInf,
    IFrameStreamInf,
    SessionData,
    SessionKey,
    ContentSteering,
}

impl TryFrom<&'_ str> for TagName {
    type Error = ValidationError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        match value {
            "M3U" => Ok(Self::M3u),
            "-X-VERSION" => Ok(Self::Version),
            "-X-INDEPENDENT-SEGMENTS" => Ok(Self::IndependentSegments),
            "-X-START" => Ok(Self::Start),
            "-X-DEFINE" => Ok(Self::Define),
            "-X-TARGETDURATION" => Ok(Self::Targetduration),
            "-X-MEDIA-SEQUENCE" => Ok(Self::MediaSequence),
            "-X-DISCONTINUITY-SEQUENCE" => Ok(Self::DiscontinuitySequence),
            "-X-ENDLIST" => Ok(Self::Endlist),
            "-X-PLAYLIST-TYPE" => Ok(Self::PlaylistType),
            "-X-I-FRAMES-ONLY" => Ok(Self::IFramesOnly),
            "-X-PART-INF" => Ok(Self::PartInf),
            "-X-SERVER-CONTROL" => Ok(Self::ServerControl),
            "INF" => Ok(Self::Inf),
            "-X-BYTERANGE" => Ok(Self::Byterange),
            "-X-DISCONTINUITY" => Ok(Self::Discontinuity),
            "-X-KEY" => Ok(Self::Key),
            "-X-MAP" => Ok(Self::Map),
            "-X-PROGRAM-DATE-TIME" => Ok(Self::ProgramDateTime),
            "-X-GAP" => Ok(Self::Gap),
            "-X-BITRATE" => Ok(Self::Bitrate),
            "-X-PART" => Ok(Self::Part),
            "-X-DATERANGE" => Ok(Self::Daterange),
            "-X-SKIP" => Ok(Self::Skip),
            "-X-PRELOAD-HINT" => Ok(Self::PreloadHint),
            "-X-RENDITION-REPORT" => Ok(Self::RenditionReport),
            "-X-MEDIA" => Ok(Self::Media),
            "-X-STREAM-INF" => Ok(Self::StreamInf),
            "-X-I-FRAME-STREAM-INF" => Ok(Self::IFrameStreamInf),
            "-X-SESSION-DATA" => Ok(Self::SessionData),
            "-X-SESSION-KEY" => Ok(Self::SessionKey),
            "-X-CONTENT-STEERING" => Ok(Self::ContentSteering),
            _ => Err(ValidationError::UnexpectedTagName),
        }
    }
}

impl TagName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::M3u => "M3U",
            Self::Version => "-X-VERSION",
            Self::IndependentSegments => "-X-INDEPENDENT-SEGMENTS",
            Self::Start => "-X-START",
            Self::Define => "-X-DEFINE",
            Self::Targetduration => "-X-TARGETDURATION",
            Self::MediaSequence => "-X-MEDIA-SEQUENCE",
            Self::DiscontinuitySequence => "-X-DISCONTINUITY-SEQUENCE",
            Self::Endlist => "-X-ENDLIST",
            Self::PlaylistType => "-X-PLAYLIST-TYPE",
            Self::IFramesOnly => "-X-I-FRAMES-ONLY",
            Self::PartInf => "-X-PART-INF",
            Self::ServerControl => "-X-SERVER-CONTROL",
            Self::Inf => "INF",
            Self::Byterange => "-X-BYTERANGE",
            Self::Discontinuity => "-X-DISCONTINUITY",
            Self::Key => "-X-KEY",
            Self::Map => "-X-MAP",
            Self::ProgramDateTime => "-X-PROGRAM-DATE-TIME",
            Self::Gap => "-X-GAP",
            Self::Bitrate => "-X-BITRATE",
            Self::Part => "-X-PART",
            Self::Daterange => "-X-DATERANGE",
            Self::Skip => "-X-SKIP",
            Self::PreloadHint => "-X-PRELOAD-HINT",
            Self::RenditionReport => "-X-RENDITION-REPORT",
            Self::Media => "-X-MEDIA",
            Self::StreamInf => "-X-STREAM-INF",
            Self::IFrameStreamInf => "-X-I-FRAME-STREAM-INF",
            Self::SessionData => "-X-SESSION-DATA",
            Self::SessionKey => "-X-SESSION-KEY",
            Self::ContentSteering => "-X-CONTENT-STEERING",
        }
    }

    pub fn tag_type(&self) -> TagType {
        match self {
            Self::M3u => TagType::Basic,
            Self::Version => TagType::Basic,
            Self::IndependentSegments => TagType::MediaOrMultivariantPlaylist,
            Self::Start => TagType::MediaOrMultivariantPlaylist,
            Self::Define => TagType::MediaOrMultivariantPlaylist,
            Self::Targetduration => TagType::MediaPlaylist,
            Self::MediaSequence => TagType::MediaPlaylist,
            Self::DiscontinuitySequence => TagType::MediaPlaylist,
            Self::Endlist => TagType::MediaPlaylist,
            Self::PlaylistType => TagType::MediaPlaylist,
            Self::IFramesOnly => TagType::MediaPlaylist,
            Self::PartInf => TagType::MediaPlaylist,
            Self::ServerControl => TagType::MediaPlaylist,
            Self::Inf => TagType::MediaSegment,
            Self::Byterange => TagType::MediaSegment,
            Self::Discontinuity => TagType::MediaSegment,
            Self::Key => TagType::MediaSegment,
            Self::Map => TagType::MediaSegment,
            Self::ProgramDateTime => TagType::MediaSegment,
            Self::Gap => TagType::MediaSegment,
            Self::Bitrate => TagType::MediaSegment,
            Self::Part => TagType::MediaSegment,
            Self::Daterange => TagType::MediaMetadata,
            Self::Skip => TagType::MediaMetadata,
            Self::PreloadHint => TagType::MediaMetadata,
            Self::RenditionReport => TagType::MediaMetadata,
            Self::Media => TagType::MultivariantPlaylist,
            Self::StreamInf => TagType::MultivariantPlaylist,
            Self::IFrameStreamInf => TagType::MultivariantPlaylist,
            Self::SessionData => TagType::MultivariantPlaylist,
            Self::SessionKey => TagType::MultivariantPlaylist,
            Self::ContentSteering => TagType::MultivariantPlaylist,
        }
    }
}

pub enum TagType {
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1
    Basic,
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2
    MediaOrMultivariantPlaylist,
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3
    MediaPlaylist,
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4
    MediaSegment,
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5
    MediaMetadata,
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6
    MultivariantPlaylist,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        date::{DateTime, DateTimeTimezoneOffset},
        tag::{
            hls::{
                daterange::OwnedExtensionAttributeValue, map::MapByterange, part::PartByterange,
            },
            value::{DecimalResolution, HlsPlaylistType, ParsedAttributeValue, ParsedTagValue},
        },
    };
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn m3u() {
        assert_eq!(
            Ok(Tag::M3u(M3u)),
            Tag::try_from(ParsedTag {
                name: "M3U",
                value: ParsedTagValue::Empty,
                original_input: "#EXTM3U"
            })
        )
    }

    #[test]
    fn version() {
        assert_eq!(
            Ok(Tag::Version(Version::new(9))),
            Tag::try_from(ParsedTag {
                name: "-X-VERSION",
                value: ParsedTagValue::DecimalInteger(9),
                original_input: "#EXT-X-VERSION:9"
            })
        )
    }

    #[test]
    fn independent_segments() {
        assert_eq!(
            Ok(Tag::IndependentSegments(IndependentSegments)),
            Tag::try_from(ParsedTag {
                name: "-X-INDEPENDENT-SEGMENTS",
                value: ParsedTagValue::Empty,
                original_input: "#EXT-X-INDEPENDENT-SEGMENTS"
            })
        )
    }

    #[test]
    fn start() {
        assert_eq!(
            Ok(Tag::Start(Start::new(10.5, false))),
            Tag::try_from(ParsedTag {
                name: "-X-START",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "TIME-OFFSET",
                    ParsedAttributeValue::SignedDecimalFloatingPoint(10.5)
                )])),
                original_input: "#EXT-X-START:TIME-OFFSET=10.5"
            })
        );
        let expected = Tag::Start(Start::new(10.0, true));
        let actual = Tag::try_from(ParsedTag {
            name: "-X-START",
            value: ParsedTagValue::AttributeList(HashMap::from([
                ("TIME-OFFSET", ParsedAttributeValue::DecimalInteger(10)),
                ("PRECISE", ParsedAttributeValue::UnquotedString("YES")),
            ])),
            original_input: "#EXT-X-START:TIME-OFFSET=10.5,PRECISE=YES",
        })
        .unwrap();
        match (expected, actual) {
            (Tag::Start(expected), Tag::Start(actual)) => {
                assert_eq!(expected.time_offset(), actual.time_offset());
                assert_eq!(expected.precise(), actual.precise());
            }
            _ => panic!("Unexpected tag type"),
        }
    }

    #[test]
    fn define() {
        assert_eq!(
            Ok(Tag::Define(Define::new_name(
                "TEST".to_string(),
                "GOOD".to_string()
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-DEFINE",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("NAME", ParsedAttributeValue::QuotedString("TEST")),
                    ("VALUE", ParsedAttributeValue::QuotedString("GOOD"))
                ])),
                original_input: "#EXT-X-DEFINE:NAME=\"TEST\",VALUE=\"GOOD\""
            })
        );
        assert_eq!(
            Ok(Tag::Define(Define::new_import("TEST".to_string()))),
            Tag::try_from(ParsedTag {
                name: "-X-DEFINE",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "IMPORT",
                    ParsedAttributeValue::QuotedString("TEST")
                )])),
                original_input: "#EXT-X-DEFINE:IMPORT=\"TEST\""
            })
        );
        assert_eq!(
            Ok(Tag::Define(Define::new_queryparam(
                "testQueryParam".to_string()
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-DEFINE",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "QUERYPARAM",
                    ParsedAttributeValue::QuotedString("testQueryParam")
                )])),
                original_input: "#EXT-X-DEFINE:QUERYPARAM=\"testQueryParam\""
            })
        );
    }

    #[test]
    fn targetduration() {
        assert_eq!(
            Ok(Tag::Targetduration(Targetduration::new(6))),
            Tag::try_from(ParsedTag {
                name: "-X-TARGETDURATION",
                value: ParsedTagValue::DecimalInteger(6),
                original_input: "#EXT-X-TARGETDURATION:6"
            }),
        );
    }

    #[test]
    fn media_sequence() {
        assert_eq!(
            Ok(Tag::MediaSequence(MediaSequence::new(100))),
            Tag::try_from(ParsedTag {
                name: "-X-MEDIA-SEQUENCE",
                value: ParsedTagValue::DecimalInteger(100),
                original_input: "#EXT-X-MEDIA-SEQUENCE:100"
            })
        );
    }

    #[test]
    fn discontinuity_sequencee() {
        assert_eq!(
            Ok(Tag::DiscontinuitySequence(DiscontinuitySequence::new(100))),
            Tag::try_from(ParsedTag {
                name: "-X-DISCONTINUITY-SEQUENCE",
                value: ParsedTagValue::DecimalInteger(100),
                original_input: "#EXT-X-DISCONTINUITY-SEQUENCE:100"
            })
        );
    }

    #[test]
    fn endlist() {
        assert_eq!(
            Ok(Tag::Endlist(Endlist)),
            Tag::try_from(ParsedTag {
                name: "-X-ENDLIST",
                value: ParsedTagValue::Empty,
                original_input: "#EXT-X-ENDLIST"
            })
        )
    }

    #[test]
    fn playlist_type() {
        assert_eq!(
            Ok(Tag::PlaylistType(PlaylistType::new(HlsPlaylistType::Event))),
            Tag::try_from(ParsedTag {
                name: "-X-PLAYLIST-TYPE",
                value: ParsedTagValue::TypeEnum(HlsPlaylistType::Event),
                original_input: "#EXT-X-PLAYLIST-TYPE:EVENT"
            })
        );
        assert_eq!(
            Ok(Tag::PlaylistType(PlaylistType::new(HlsPlaylistType::Vod))),
            Tag::try_from(ParsedTag {
                name: "-X-PLAYLIST-TYPE",
                value: ParsedTagValue::TypeEnum(HlsPlaylistType::Vod),
                original_input: "#EXT-X-PLAYLIST-TYPE:VOD"
            })
        );
    }

    #[test]
    fn i_frames_only() {
        assert_eq!(
            Ok(Tag::IFramesOnly(IFramesOnly)),
            Tag::try_from(ParsedTag {
                name: "-X-I-FRAMES-ONLY",
                value: ParsedTagValue::Empty,
                original_input: "#EXT-X-I-FRAMES-ONLY"
            })
        )
    }

    #[test]
    fn part_inf() {
        assert_eq!(
            Ok(Tag::PartInf(PartInf::new(0.5))),
            Tag::try_from(ParsedTag {
                name: "-X-PART-INF",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "PART-TARGET",
                    ParsedAttributeValue::SignedDecimalFloatingPoint(0.5)
                )])),
                original_input: "#EXT-X-PART-INF:PART-TARGET=0.5"
            })
        )
    }

    #[test]
    fn server_control() {
        assert_eq!(
            Ok(Tag::ServerControl(ServerControl::new(
                Some(36.0),
                true,
                Some(12.0),
                Some(1.5),
                true,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SERVER-CONTROL",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    (
                        "CAN-SKIP-UNTIL",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(36.0)
                    ),
                    (
                        "CAN-SKIP-DATERANGES",
                        ParsedAttributeValue::UnquotedString("YES")
                    ),
                    (
                        "HOLD-BACK",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(12.0)
                    ),
                    (
                        "PART-HOLD-BACK",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(1.5)
                    ),
                    (
                        "CAN-BLOCK-RELOAD",
                        ParsedAttributeValue::UnquotedString("YES")
                    ),
                ])),
                original_input: "#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=36,CAN-SKIP-DATERANGES=YES,HOLD-BACK=12,PART-HOLD-BACK=1.5,CAN-BLOCK-RELOAD=YES"
            })
        );
        // In reality this is not possible within regular parsing, as this would be considered empty
        // value case (rather than attribute list), but this is used to validate optionality of all
        // properties (which seems fair as part of a unit test).
        assert_eq!(
            Ok(Tag::ServerControl(ServerControl::new(
                None, false, None, None, false,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SERVER-CONTROL",
                value: ParsedTagValue::AttributeList(HashMap::new()),
                original_input: "#EXT-X-SERVER-CONTROL:"
            })
        );
    }

    #[test]
    fn inf() {
        assert_eq!(
            Ok(Tag::Inf(Inf::new(6.0, "".to_string()))),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalInteger(6),
                original_input: "#EXTINF:6"
            })
        );
        assert_eq!(
            Ok(Tag::Inf(Inf::new(6.006, "".to_string()))),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(6.006, ""),
                original_input: "#EXTINF:6.006,"
            })
        );
        assert_eq!(
            Ok(Tag::Inf(Inf::new(6.006, "A useful title".to_string()))),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    6.006,
                    "A useful title"
                ),
                original_input: "#EXTINF:6.006,A useful title"
            })
        );
    }

    #[test]
    fn byterange() {
        assert_eq!(
            Ok(Tag::Byterange(Byterange::new(1024, Some(512)))),
            Tag::try_from(ParsedTag {
                name: "-X-BYTERANGE",
                value: ParsedTagValue::DecimalIntegerRange(1024, 512),
                original_input: "#EXT-X-BYTERANGE:1024@512"
            })
        );
        assert_eq!(
            Ok(Tag::Byterange(Byterange::new(1024, None))),
            Tag::try_from(ParsedTag {
                name: "-X-BYTERANGE",
                value: ParsedTagValue::DecimalInteger(1024),
                original_input: "#EXT-X-BYTERANGE:1024"
            })
        );
    }

    #[test]
    fn discontinuity() {
        assert_eq!(
            Ok(Tag::Discontinuity(Discontinuity)),
            Tag::try_from(ParsedTag {
                name: "-X-DISCONTINUITY",
                value: ParsedTagValue::Empty,
                original_input: "#EXT-X-DISCONTINUITY"
            })
        );
    }

    #[test]
    fn key() {
        assert_eq!(
            Ok(Tag::Key(Key::new(
                "SAMPLE-AES".to_string(),
                Some("skd://some-key-id".to_string()),
                Some("0xABCD".to_string()),
                Some("com.apple.streamingkeydelivery".to_string()),
                Some("1".to_string()),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-KEY",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("METHOD", ParsedAttributeValue::UnquotedString("SAMPLE-AES")),
                    (
                        "URI",
                        ParsedAttributeValue::QuotedString("skd://some-key-id")
                    ),
                    ("IV", ParsedAttributeValue::UnquotedString("0xABCD")),
                    (
                        "KEYFORMAT",
                        ParsedAttributeValue::QuotedString("com.apple.streamingkeydelivery")
                    ),
                    ("KEYFORMATVERSIONS", ParsedAttributeValue::QuotedString("1")),
                ])),
                original_input: "#EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"skd://some-key-id\",IV=0xABCD,KEYFORMAT=\"com.apple.streamingkeydelivery\",KEYFORMATVERSIONS=\"1\""
            })
        );
        assert_eq!(
            Ok(Tag::Key(Key::new(
                "NONE".to_string(),
                None,
                None,
                None,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-KEY",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "METHOD",
                    ParsedAttributeValue::UnquotedString("NONE")
                )])),
                original_input: "#EXT-X-KEY:METHOD=NONE"
            })
        );
    }

    #[test]
    fn map() {
        assert_eq!(
            Ok(Tag::Map(Map::new(
                "init.mp4".to_string(),
                Some(MapByterange {
                    length: 1024,
                    offset: 0
                })
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-MAP",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("init.mp4")),
                    ("BYTERANGE", ParsedAttributeValue::QuotedString("1024@0")),
                ])),
                original_input: "#EXT-X-MAP:URI=\"init.mp4\",BYTERANGE=\"1024@0\""
            })
        );
        assert_eq!(
            Ok(Tag::Map(Map::new("init.mp4".to_string(), None,))),
            Tag::try_from(ParsedTag {
                name: "-X-MAP",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "URI",
                    ParsedAttributeValue::QuotedString("init.mp4")
                )])),
                original_input: "#EXT-X-MAP:URI=\"init.mp4\""
            })
        );
    }

    #[test]
    fn program_date_time() {
        let date_time = DateTime {
            date_fullyear: 2025,
            date_month: 6,
            date_mday: 5,
            time_hour: 16,
            time_minute: 46,
            time_second: 42.123,
            timezone_offset: DateTimeTimezoneOffset {
                time_hour: -5,
                time_minute: 0,
            },
        };
        assert_eq!(
            Ok(Tag::ProgramDateTime(ProgramDateTime::new(date_time))),
            Tag::try_from(ParsedTag {
                name: "-X-PROGRAM-DATE-TIME",
                value: ParsedTagValue::DateTimeMsec(date_time),
                original_input: "#EXT-X-PROGRAM-DATE-TIME:2025-06-05T16:46:42.123-05:00"
            })
        );
    }

    #[test]
    fn gap() {
        assert_eq!(
            Ok(Tag::Gap(Gap)),
            Tag::try_from(ParsedTag {
                name: "-X-GAP",
                value: ParsedTagValue::Empty,
                original_input: "#EXT-X-GAP"
            })
        );
    }

    #[test]
    fn bitrate() {
        assert_eq!(
            Ok(Tag::Bitrate(Bitrate::new(10000000))),
            Tag::try_from(ParsedTag {
                name: "-X-BITRATE",
                value: ParsedTagValue::DecimalInteger(10000000),
                original_input: "#EXT-X-BITRATE:10000000"
            })
        );
    }

    #[test]
    fn part() {
        assert_eq!(
            Ok(Tag::Part(Part::new(
                "part.1.mp4".to_string(),
                0.5,
                true,
                Some(PartByterange {
                    length: 1024,
                    offset: Some(512)
                }),
                true,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-PART",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("part.1.mp4")),
                    (
                        "DURATION",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(0.5)
                    ),
                    ("INDEPENDENT", ParsedAttributeValue::UnquotedString("YES")),
                    ("BYTERANGE", ParsedAttributeValue::QuotedString("1024@512")),
                    ("GAP", ParsedAttributeValue::UnquotedString("YES"))
                ])),
                original_input: "#EXT-X-PART:URI=\"part.1.mp4\",DURATION=0.5,INDEPENDENT=YES,BYTERANGE=1024@512,GAP=YES"
            })
        );
        assert_eq!(
            Ok(Tag::Part(Part::new(
                "part.1.mp4".to_string(),
                0.5,
                false,
                Some(PartByterange {
                    length: 1024,
                    offset: None
                }),
                false,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-PART",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("part.1.mp4")),
                    (
                        "DURATION",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(0.5)
                    ),
                    ("BYTERANGE", ParsedAttributeValue::QuotedString("1024")),
                ])),
                original_input: "#EXT-X-PART:URI=\"part.1.mp4\",DURATION=0.5,BYTERANGE=1024"
            })
        );
        assert_eq!(
            Ok(Tag::Part(Part::new(
                "part.1.mp4".to_string(),
                0.5,
                false,
                None,
                false,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-PART",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("part.1.mp4")),
                    (
                        "DURATION",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(0.5)
                    ),
                ])),
                original_input: "#EXT-X-PART:URI=\"part.1.mp4\",DURATION=0.5"
            })
        );
    }

    #[test]
    fn daterange() {
        assert_eq!(
            Ok(Tag::Daterange(Daterange::new(
                "test".to_string(),
                Some("com.m3u8.test".to_string()),
                DateTime {
                    date_fullyear: 2025,
                    date_month: 6,
                    date_mday: 5,
                    time_hour: 20,
                    time_minute: 38,
                    time_second: 42.149,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: -5,
                        time_minute: 0
                    }
                },
                Some("ONCE".to_string()),
                Some(DateTime {
                    date_fullyear: 2025,
                    date_month: 6,
                    date_mday: 5,
                    time_hour: 20,
                    time_minute: 40,
                    time_second: 42.149,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: -5,
                        time_minute: 0
                    }
                }),
                Some(120.0),
                Some(180.0),
                HashMap::from([(
                    "X-COM-M3U8-TEST".to_string(),
                    OwnedExtensionAttributeValue::QuotedString("YES".to_string())
                )]),
                Some("0xABCD".to_string()),
                Some("0xABCD".to_string()),
                Some("0xABCD".to_string()),
                true,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-DATERANGE",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("ID", ParsedAttributeValue::QuotedString("test")),
                    ("CLASS", ParsedAttributeValue::QuotedString("com.m3u8.test")),
                    (
                        "START-DATE",
                        ParsedAttributeValue::QuotedString("2025-06-05T20:38:42.149-05:00")
                    ),
                    ("CUE", ParsedAttributeValue::QuotedString("ONCE")),
                    (
                        "END-DATE",
                        ParsedAttributeValue::QuotedString("2025-06-05T20:40:42.149-05:00")
                    ),
                    (
                        "DURATION",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(120.0)
                    ),
                    (
                        "PLANNED-DURATION",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(180.0)
                    ),
                    ("X-COM-M3U8-TEST", ParsedAttributeValue::QuotedString("YES")),
                    ("SCTE35-CMD", ParsedAttributeValue::UnquotedString("0xABCD")),
                    ("SCTE35-OUT", ParsedAttributeValue::UnquotedString("0xABCD")),
                    ("SCTE35-IN", ParsedAttributeValue::UnquotedString("0xABCD")),
                    ("END-ON-NEXT", ParsedAttributeValue::UnquotedString("YES")),
                ])),
                original_input: concat!(
                    "#EXT-X-DATERANGE:",
                    "ID=\"test\",",
                    "CLASS=\"com.m3u8.test\",",
                    "START-DATE=\"2025-06-05T20:38:42.149-05:00\",",
                    "CUE=\"ONCE\",",
                    "END-DATE=\"2025-06-05T20:40:42.149-05:00\",",
                    "DURATION=120,",
                    "PLANNED-DURATION=180,",
                    "X-COM-M3U8-TEST=\"YES\",",
                    "SCTE35-CMD=0xABCD,",
                    "SCTE35-OUT=0xABCD,",
                    "SCTE35-IN=0xABCD,",
                    "END-ON-NEXT=YES",
                )
            })
        );
        assert_eq!(
            Ok(Tag::Daterange(Daterange::new(
                "test".to_string(),
                None as Option<String>,
                DateTime {
                    date_fullyear: 2025,
                    date_month: 6,
                    date_mday: 5,
                    time_hour: 20,
                    time_minute: 38,
                    time_second: 42.149,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: -5,
                        time_minute: 0
                    }
                },
                None as Option<String>,
                None,
                None,
                None,
                HashMap::new(),
                None as Option<String>,
                None as Option<String>,
                None as Option<String>,
                false,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-DATERANGE",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("ID", ParsedAttributeValue::QuotedString("test")),
                    (
                        "START-DATE",
                        ParsedAttributeValue::QuotedString("2025-06-05T20:38:42.149-05:00")
                    ),
                ])),
                original_input: "#EXT-X-DATERANGE:ID=\"test\",START-DATE=\"2025-06-05T20:38:42.149-05:00\""
            })
        );
    }

    #[test]
    fn skip() {
        assert_eq!(
            Ok(Tag::Skip(Skip::new(100, Some("1234\tabcd".to_string())))),
            Tag::try_from(ParsedTag {
                name: "-X-SKIP",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    (
                        "SKIPPED-SEGMENTS",
                        ParsedAttributeValue::DecimalInteger(100)
                    ),
                    (
                        "RECENTLY-REMOVED-DATERANGES",
                        ParsedAttributeValue::QuotedString("1234\tabcd")
                    ),
                ])),
                original_input: "#EXT-X-SKIP:SKIPPED-SEGMENTS=100,RECENTLY-REMOVED-DATERANGES=\"1234\tabcd\""
            })
        );
        assert_eq!(
            Ok(Tag::Skip(Skip::new(100, None,))),
            Tag::try_from(ParsedTag {
                name: "-X-SKIP",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "SKIPPED-SEGMENTS",
                    ParsedAttributeValue::DecimalInteger(100)
                )])),
                original_input: "#EXT-X-SKIP:SKIPPED-SEGMENTS=100"
            })
        );
    }

    #[test]
    fn preload_hint() {
        assert_eq!(
            Ok(Tag::PreloadHint(PreloadHint::new(
                "PART".to_string(),
                "part.2.mp4".to_string(),
                Some(512),
                Some(1024),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-PRELOAD-HINT",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("TYPE", ParsedAttributeValue::UnquotedString("PART")),
                    ("URI", ParsedAttributeValue::QuotedString("part.2.mp4")),
                    ("BYTERANGE-START", ParsedAttributeValue::DecimalInteger(512)),
                    (
                        "BYTERANGE-LENGTH",
                        ParsedAttributeValue::DecimalInteger(1024)
                    ),
                ])),
                original_input: "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\",BYTERANGE-START=512,BYTERANGE-LENGTH=1024"
            })
        );
        assert_eq!(
            Ok(Tag::PreloadHint(PreloadHint::new(
                "PART".to_string(),
                "part.2.mp4".to_string(),
                None,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-PRELOAD-HINT",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("TYPE", ParsedAttributeValue::UnquotedString("PART")),
                    ("URI", ParsedAttributeValue::QuotedString("part.2.mp4")),
                ])),
                original_input: "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\""
            })
        );
    }

    #[test]
    fn rendition_report() {
        assert_eq!(
            Ok(Tag::RenditionReport(RenditionReport::new(
                "high.m3u8".to_string(),
                1000,
                Some(2),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-RENDITION-REPORT",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("high.m3u8")),
                    ("LAST-MSN", ParsedAttributeValue::DecimalInteger(1000)),
                    ("LAST-PART", ParsedAttributeValue::DecimalInteger(2)),
                ])),
                original_input: "#EXT-X-RENDITION-REPORT:URI=\"high.m3u8\",LAST-MSN=1000,LAST-PART=2"
            })
        );
        assert_eq!(
            Ok(Tag::RenditionReport(RenditionReport::new(
                "high.m3u8".to_string(),
                1000,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-RENDITION-REPORT",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("high.m3u8")),
                    ("LAST-MSN", ParsedAttributeValue::DecimalInteger(1000)),
                ])),
                original_input: "#EXT-X-RENDITION-REPORT:URI=\"high.m3u8\",LAST-MSN=1000"
            })
        );
    }

    #[test]
    fn media() {
        assert_eq!(
            Ok(Tag::Media(Media::new(
                "AUDIO".to_string(),
                "English".to_string(),
                "stereo".to_string(),
                Some("audio/en/stereo.m3u8".to_string()),
                Some("en".to_string()),
                Some("en".to_string()),
                Some("1234".to_string()),
                true,
                true,
                true,
                None,
                Some(8),
                Some(48000),
                Some("public.accessibility.describes-video".to_string()),
                Some("2".to_string()),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-MEDIA",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("TYPE", ParsedAttributeValue::UnquotedString("AUDIO")),
                    (
                        "URI",
                        ParsedAttributeValue::QuotedString("audio/en/stereo.m3u8")
                    ),
                    ("GROUP-ID", ParsedAttributeValue::QuotedString("stereo")),
                    ("LANGUAGE", ParsedAttributeValue::QuotedString("en")),
                    ("ASSOC-LANGUAGE", ParsedAttributeValue::QuotedString("en")),
                    ("NAME", ParsedAttributeValue::QuotedString("English")),
                    (
                        "STABLE-RENDITION-ID",
                        ParsedAttributeValue::QuotedString("1234")
                    ),
                    ("DEFAULT", ParsedAttributeValue::UnquotedString("YES")),
                    ("AUTOSELECT", ParsedAttributeValue::UnquotedString("YES")),
                    ("FORCED", ParsedAttributeValue::UnquotedString("YES")),
                    ("BIT-DEPTH", ParsedAttributeValue::DecimalInteger(8)),
                    ("SAMPLE-RATE", ParsedAttributeValue::DecimalInteger(48000)),
                    (
                        "CHARACTERISTICS",
                        ParsedAttributeValue::QuotedString("public.accessibility.describes-video")
                    ),
                    ("CHANNELS", ParsedAttributeValue::QuotedString("2")),
                ])),
                original_input: concat!(
                    "#EXT-X-MEDIA:",
                    "TYPE=AUDIO,",
                    "NAME=\"English\",",
                    "GROUP-ID=\"stereo\",",
                    "URI=\"audio/en/stereo.m3u8\",",
                    "LANGUAGE=\"en\",",
                    "ASSOC-LANGUAGE=\"en\",",
                    "STABLE-RENDITION-ID=\"1234\",",
                    "DEFAULT=YES,",
                    "AUTOSELECT=YES,",
                    "FORCED=YES,",
                    "BIT-DEPTH=8,",
                    "SAMPLE-RATE=48000,",
                    "CHARACTERISTICS=\"public.accessibility.describes-video\",",
                    "CHANNELS=\"2\"",
                )
            })
        );
        assert_eq!(
            Ok(Tag::Media(Media::new(
                "CLOSED-CAPTIONS".to_string(),
                "English".to_string(),
                "cc".to_string(),
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                Some("CC1".to_string()),
                None,
                None,
                None,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-MEDIA",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    (
                        "TYPE",
                        ParsedAttributeValue::UnquotedString("CLOSED-CAPTIONS")
                    ),
                    ("GROUP-ID", ParsedAttributeValue::QuotedString("cc")),
                    ("NAME", ParsedAttributeValue::QuotedString("English")),
                    ("INSTREAM-ID", ParsedAttributeValue::QuotedString("CC1")),
                ])),
                original_input: concat!(
                    "#EXT-X-MEDIA:",
                    "TYPE=CLOSED-CAPTIONS,",
                    "NAME=\"English\",",
                    "GROUP-ID=\"cc\",",
                    "INSTREAM-ID=\"CC1\""
                )
            })
        );
    }

    #[test]
    fn stream_inf() {
        assert_eq!(
            Ok(Tag::StreamInf(StreamInf::new(
                10000000,
                Some(9000000),
                Some(2.0),
                Some("hvc1.2.4.L153.b0,ec-3".to_string()),
                Some("dvh1.08.07/db4h".to_string()),
                Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                Some(23.976),
                Some("TYPE-1".to_string()),
                Some("com.example.drm1:SMART-TV/PC".to_string()),
                Some("PQ".to_string()),
                Some("CH-STEREO,CH-MONO".to_string()),
                Some("1234".to_string()),
                Some("surround".to_string()),
                Some("alternate-view".to_string()),
                Some("subs".to_string()),
                Some("cc".to_string()),
                Some("1234".to_string()),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-STREAM-INF",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("BANDWIDTH", ParsedAttributeValue::DecimalInteger(10000000)),
                    (
                        "AVERAGE-BANDWIDTH",
                        ParsedAttributeValue::DecimalInteger(9000000)
                    ),
                    (
                        "SCORE",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(2.0)
                    ),
                    (
                        "CODECS",
                        ParsedAttributeValue::QuotedString("hvc1.2.4.L153.b0,ec-3")
                    ),
                    (
                        "SUPPLEMENTAL-CODECS",
                        ParsedAttributeValue::QuotedString("dvh1.08.07/db4h")
                    ),
                    (
                        "RESOLUTION",
                        ParsedAttributeValue::UnquotedString("3840x2160")
                    ),
                    (
                        "FRAME-RATE",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(23.976)
                    ),
                    ("HDCP-LEVEL", ParsedAttributeValue::UnquotedString("TYPE-1")),
                    (
                        "ALLOWED-CPC",
                        ParsedAttributeValue::QuotedString("com.example.drm1:SMART-TV/PC")
                    ),
                    ("VIDEO-RANGE", ParsedAttributeValue::UnquotedString("PQ")),
                    (
                        "REQ-VIDEO-LAYOUT",
                        ParsedAttributeValue::QuotedString("CH-STEREO,CH-MONO")
                    ),
                    (
                        "STABLE-VARIANT-ID",
                        ParsedAttributeValue::QuotedString("1234")
                    ),
                    ("AUDIO", ParsedAttributeValue::QuotedString("surround")),
                    (
                        "VIDEO",
                        ParsedAttributeValue::QuotedString("alternate-view")
                    ),
                    ("SUBTITLES", ParsedAttributeValue::QuotedString("subs")),
                    ("CLOSED-CAPTIONS", ParsedAttributeValue::QuotedString("cc")),
                    ("PATHWAY-ID", ParsedAttributeValue::QuotedString("1234")),
                ])),
                original_input: concat!(
                    "#EXT-X-STREAM-INF:",
                    "BANDWIDTH=10000000,",
                    "AVERAGE-BANDWIDTH=9000000,",
                    "SCORE=2.0,",
                    "CODECS=\"hvc1.2.4.L153.b0,ec-3\",",
                    "SUPPLEMENTAL-CODECS=\"dvh1.08.07/db4h\",",
                    "RESOLUTION=3840x2160,",
                    "FRAME-RATE=23.976,",
                    "HDCP-LEVEL=TYPE-1,",
                    "ALLOWED-CPC=\"com.example.drm1:SMART-TV/PC\",",
                    "VIDEO-RANGE=PQ,",
                    "REQ-VIDEO-LAYOUT=\"CH-STEREO,CH-MONO\",",
                    "STABLE-VARIANT-ID=\"1234\",",
                    "AUDIO=\"surround\",",
                    "VIDEO=\"alternate-view\",",
                    "SUBTITLES=\"subs\",",
                    "CLOSED-CAPTIONS=\"cc\",",
                    "PATHWAY-ID=\"1234\"",
                )
            })
        );
        // One more test to check that integer frame rate parses well
        assert_eq!(
            Ok(Tag::StreamInf(StreamInf::new(
                10000000,
                None,
                None,
                None,
                None,
                None,
                Some(25.0),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-STREAM-INF",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("BANDWIDTH", ParsedAttributeValue::DecimalInteger(10000000)),
                    ("FRAME-RATE", ParsedAttributeValue::DecimalInteger(25)),
                ])),
                original_input: "#EXT-X-STREAM-INF:BANDWIDTH=10000000,FRAME-RATE=25"
            })
        );
        // Final check with all options unset
        assert_eq!(
            Ok(Tag::StreamInf(StreamInf::new(
                10000000, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-STREAM-INF",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "BANDWIDTH",
                    ParsedAttributeValue::DecimalInteger(10000000)
                )])),
                original_input: "#EXT-X-STREAM-INF:BANDWIDTH=10000000"
            })
        );
    }

    #[test]
    fn i_frame_stream_inf() {
        assert_eq!(
            Ok(Tag::IFrameStreamInf(IFrameStreamInf::new(
                "iframe.high.m3u8".to_string(),
                10000000,
                Some(9000000),
                Some(2.0),
                Some("hvc1.2.4.L153.b0,ec-3".to_string()),
                Some("dvh1.08.07/db4h".to_string()),
                Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                Some("TYPE-1".to_string()),
                Some("com.example.drm1:SMART-TV/PC".to_string()),
                Some("PQ".to_string()),
                Some("CH-STEREO,CH-MONO".to_string()),
                Some("1234".to_string()),
                Some("alternate-view".to_string()),
                Some("1234".to_string()),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-I-FRAME-STREAM-INF",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    (
                        "URI",
                        ParsedAttributeValue::QuotedString("iframe.high.m3u8")
                    ),
                    ("BANDWIDTH", ParsedAttributeValue::DecimalInteger(10000000)),
                    (
                        "AVERAGE-BANDWIDTH",
                        ParsedAttributeValue::DecimalInteger(9000000)
                    ),
                    (
                        "SCORE",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(2.0)
                    ),
                    (
                        "CODECS",
                        ParsedAttributeValue::QuotedString("hvc1.2.4.L153.b0,ec-3")
                    ),
                    (
                        "SUPPLEMENTAL-CODECS",
                        ParsedAttributeValue::QuotedString("dvh1.08.07/db4h")
                    ),
                    (
                        "RESOLUTION",
                        ParsedAttributeValue::UnquotedString("3840x2160")
                    ),
                    ("HDCP-LEVEL", ParsedAttributeValue::UnquotedString("TYPE-1")),
                    (
                        "ALLOWED-CPC",
                        ParsedAttributeValue::QuotedString("com.example.drm1:SMART-TV/PC")
                    ),
                    ("VIDEO-RANGE", ParsedAttributeValue::UnquotedString("PQ")),
                    (
                        "REQ-VIDEO-LAYOUT",
                        ParsedAttributeValue::QuotedString("CH-STEREO,CH-MONO")
                    ),
                    (
                        "STABLE-VARIANT-ID",
                        ParsedAttributeValue::QuotedString("1234")
                    ),
                    (
                        "VIDEO",
                        ParsedAttributeValue::QuotedString("alternate-view")
                    ),
                    ("PATHWAY-ID", ParsedAttributeValue::QuotedString("1234")),
                ])),
                original_input: concat!(
                    "#EXT-X-I-FRAME-STREAM-INF:",
                    "URI=\"iframe.high.m3u8\",",
                    "BANDWIDTH=10000000,",
                    "AVERAGE-BANDWIDTH=9000000,",
                    "SCORE=2.0,",
                    "CODECS=\"hvc1.2.4.L153.b0,ec-3\",",
                    "SUPPLEMENTAL-CODECS=\"dvh1.08.07/db4h\",",
                    "RESOLUTION=3840x2160,",
                    "HDCP-LEVEL=TYPE-1",
                    "ALLOWED-CPC=\"com.example.drm1:SMART-TV/PC\",",
                    "VIDEO-RANGE=PQ,",
                    "REQ-VIDEO-LAYOUT=\"CH-STEREO,CH-MONO\",",
                    "STABLE-VARIANT-ID=\"1234\",",
                    "VIDEO=\"alternate-view\",",
                    "PATHWAY-ID=\"1234\"",
                )
            })
        );
        assert_eq!(
            Ok(Tag::IFrameStreamInf(IFrameStreamInf::new(
                "iframe.high.m3u8".to_string(),
                10000000,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-I-FRAME-STREAM-INF",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    (
                        "URI",
                        ParsedAttributeValue::QuotedString("iframe.high.m3u8")
                    ),
                    ("BANDWIDTH", ParsedAttributeValue::DecimalInteger(10000000))
                ])),
                original_input: concat!(
                    "#EXT-X-I-FRAME-STREAM-INF:",
                    "URI=\"iframe.high.m3u8\",",
                    "BANDWIDTH=10000000",
                )
            })
        );
    }

    #[test]
    fn session_data() {
        assert_eq!(
            Ok(Tag::SessionData(SessionData::new(
                "1234".to_string(),
                Some("test".to_string()),
                None,
                None,
                Some("en".to_string()),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SESSION-DATA",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("DATA-ID", ParsedAttributeValue::QuotedString("1234")),
                    ("VALUE", ParsedAttributeValue::QuotedString("test")),
                    ("LANGUAGE", ParsedAttributeValue::QuotedString("en")),
                ])),
                original_input: "#EXT-X-SESSION-DATA:DATA-ID=\"1234\",VALUE=\"test\",LANGUAGE=\"en\""
            })
        );
        assert_eq!(
            Ok(Tag::SessionData(SessionData::new(
                "1234".to_string(),
                None,
                Some("test.bin".to_string()),
                Some("RAW".to_string()),
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SESSION-DATA",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("DATA-ID", ParsedAttributeValue::QuotedString("1234")),
                    ("URI", ParsedAttributeValue::QuotedString("test.bin")),
                    ("FORMAT", ParsedAttributeValue::UnquotedString("RAW")),
                ])),
                original_input: "#EXT-X-SESSION-DATA:DATA-ID=\"1234\",URI=\"test.bin\",FORMAT=RAW"
            })
        );
    }

    #[test]
    fn session_key() {
        assert_eq!(
            Ok(Tag::SessionKey(SessionKey::new(
                "SAMPLE-AES".to_string(),
                "skd://some-key-id".to_string(),
                Some("0xABCD".to_string()),
                Some("com.apple.streamingkeydelivery".to_string()),
                Some("1".to_string()),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SESSION-KEY",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("METHOD", ParsedAttributeValue::UnquotedString("SAMPLE-AES")),
                    (
                        "URI",
                        ParsedAttributeValue::QuotedString("skd://some-key-id")
                    ),
                    ("IV", ParsedAttributeValue::UnquotedString("0xABCD")),
                    (
                        "KEYFORMAT",
                        ParsedAttributeValue::QuotedString("com.apple.streamingkeydelivery")
                    ),
                    ("KEYFORMATVERSIONS", ParsedAttributeValue::QuotedString("1")),
                ])),
                original_input: "#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"skd://some-key-id\",IV=0xABCD,KEYFORMAT=\"com.apple.streamingkeydelivery\",KEYFORMATVERSIONS=\"1\""
            })
        );
        assert_eq!(
            Ok(Tag::SessionKey(SessionKey::new(
                "AES-128".to_string(),
                "skd://some-key-id".to_string(),
                None,
                None,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SESSION-KEY",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("METHOD", ParsedAttributeValue::UnquotedString("AES-128")),
                    (
                        "URI",
                        ParsedAttributeValue::QuotedString("skd://some-key-id")
                    ),
                ])),
                original_input: "#EXT-X-SESSION-KEY:METHOD=AES-128,URI=\"skd://some-key-id\""
            })
        );
    }

    #[test]
    fn content_steering() {
        assert_eq!(
            Ok(Tag::ContentSteering(ContentSteering::new(
                "content-steering.json".to_string(),
                Some("1234".to_string())
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-CONTENT-STEERING",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    (
                        "SERVER-URI",
                        ParsedAttributeValue::QuotedString("content-steering.json")
                    ),
                    ("PATHWAY-ID", ParsedAttributeValue::QuotedString("1234")),
                ])),
                original_input: "#EXT-X-CONTENT-STEERING:SERVER-URI=\"content-steering.json\",PATHWAY-ID=\"1234\""
            })
        );
        assert_eq!(
            Ok(Tag::ContentSteering(ContentSteering::new(
                "content-steering.json".to_string(),
                None
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-CONTENT-STEERING",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "SERVER-URI",
                    ParsedAttributeValue::QuotedString("content-steering.json")
                )])),
                original_input: "#EXT-X-CONTENT-STEERING:SERVER-URI=\"content-steering.json\""
            })
        );
    }
}
