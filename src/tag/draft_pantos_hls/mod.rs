use crate::tag::{
    draft_pantos_hls::{
        bitrate::Bitrate, byterange::Byterange, content_steering::ContentSteering,
        daterange::Daterange, define::Define, discontinuity::Discontinuity,
        discontinuity_sequence::DiscontinuitySequence, endlist::Endlist, gap::Gap,
        i_frame_stream_inf::IFrameStreamInf, i_frames_only::IFramesOnly,
        independent_segments::IndependentSegments, inf::Inf, key::Key, m3u::M3u, map::Map,
        media::Media, media_sequence::MediaSequence, part::Part, part_inf::PartInf,
        playlist_type::PlaylistType, preload_hint::PreloadHint, program_date_time::ProgramDateTime,
        rendition_report::RenditionReport, server_control::ServerControl,
        session_data::SessionData, session_key::SessionKey, skip::Skip, start::Start,
        stream_inf::StreamInf, target_duration::Targetduration, version::Version,
    },
    known::ParsedTag,
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
pub mod target_duration;
pub mod version;

#[derive(Debug, PartialEq)]
pub enum Tag<'a> {
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.1
    M3u(M3u),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.2
    Version(Version),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.1
    IndependentSegments(IndependentSegments),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2
    Start(Start<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.3
    Define(Define<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.1
    Targetduration(Targetduration),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.2
    MediaSequence(MediaSequence),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.3
    DiscontinuitySequence(DiscontinuitySequence),
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
    Byterange(Byterange),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.3
    Discontinuity(Discontinuity),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.4
    Key(Key<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5
    Map(Map<'a>),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.6
    ProgramDateTime(ProgramDateTime),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.7
    Gap(Gap),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.8
    Bitrate(Bitrate),
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
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let tag_name = TagName::try_from(tag.name)?;
        match tag_name {
            TagName::M3u => Ok(Self::M3u(M3u::try_from(tag.value)?)),
            TagName::Version => Ok(Self::Version(Version::try_from(tag.value)?)),
            TagName::IndependentSegments => Ok(Self::IndependentSegments(
                IndependentSegments::try_from(tag.value)?,
            )),
            TagName::Start => Ok(Self::Start(Start::try_from(tag.value)?)),
            TagName::Define => Ok(Self::Define(Define::try_from(tag.value)?)),
            TagName::Targetduration => {
                Ok(Self::Targetduration(Targetduration::try_from(tag.value)?))
            }
            TagName::MediaSequence => Ok(Self::MediaSequence(MediaSequence::try_from(tag.value)?)),
            TagName::DiscontinuitySequence => Ok(Self::DiscontinuitySequence(
                DiscontinuitySequence::try_from(tag.value)?,
            )),
            TagName::Endlist => Ok(Self::Endlist(Endlist::try_from(tag.value)?)),
            TagName::PlaylistType => Ok(Self::PlaylistType(PlaylistType::try_from(tag.value)?)),
            TagName::IFramesOnly => Ok(Self::IFramesOnly(IFramesOnly::try_from(tag.value)?)),
            TagName::PartInf => Ok(Self::PartInf(PartInf::try_from(tag.value)?)),
            TagName::ServerControl => Ok(Self::ServerControl(ServerControl::try_from(tag.value)?)),
            TagName::Inf => Ok(Self::Inf(Inf::try_from(tag.value)?)),
            TagName::Byterange => Ok(Self::Byterange(Byterange::try_from(tag.value)?)),
            TagName::Discontinuity => Ok(Self::Discontinuity(Discontinuity::try_from(tag.value)?)),
            TagName::Key => Ok(Self::Key(Key::try_from(tag.value)?)),
            TagName::Map => Ok(Self::Map(Map::try_from(tag.value)?)),
            TagName::ProgramDateTime => {
                Ok(Self::ProgramDateTime(ProgramDateTime::try_from(tag.value)?))
            }
            TagName::Gap => Ok(Self::Gap(Gap::try_from(tag.value)?)),
            TagName::Bitrate => Ok(Self::Bitrate(Bitrate::try_from(tag.value)?)),
            TagName::Part => Ok(Self::Part(Part::try_from(tag.value)?)),
            TagName::Daterange => Ok(Self::Daterange(Daterange::try_from(tag.value)?)),
            TagName::Skip => Ok(Self::Skip(Skip::try_from(tag.value)?)),
            TagName::PreloadHint => Ok(Self::PreloadHint(PreloadHint::try_from(tag.value)?)),
            TagName::RenditionReport => {
                Ok(Self::RenditionReport(RenditionReport::try_from(tag.value)?))
            }
            TagName::Media => Ok(Self::Media(Media::try_from(tag.value)?)),
            TagName::StreamInf => Ok(Self::StreamInf(StreamInf::try_from(tag.value)?)),
            TagName::IFrameStreamInf => {
                Ok(Self::IFrameStreamInf(IFrameStreamInf::try_from(tag.value)?))
            }
            TagName::SessionData => Ok(Self::SessionData(SessionData::try_from(tag.value)?)),
            TagName::SessionKey => Ok(Self::SessionKey(SessionKey::try_from(tag.value)?)),
            TagName::ContentSteering => {
                Ok(Self::ContentSteering(ContentSteering::try_from(tag.value)?))
            }
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
    type Error = &'static str;

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
            _ => Err("Unkown tag name."),
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

struct ValidationError;
impl ValidationError {
    fn unexpected_value_type() -> &'static str {
        "Unexpected parsed value type."
    }

    fn missing_required_attribute() -> &'static str {
        "Missing required attribute."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        date::{DateTime, DateTimeTimezoneOffset},
        tag::{
            draft_pantos_hls::{map::MapByterange, part::PartByterange},
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
                value: ParsedTagValue::Empty
            })
        )
    }

    #[test]
    fn version() {
        assert_eq!(
            Ok(Tag::Version(Version::new(9))),
            Tag::try_from(ParsedTag {
                name: "-X-VERSION",
                value: ParsedTagValue::DecimalInteger(9)
            })
        )
    }

    #[test]
    fn independent_segments() {
        assert_eq!(
            Ok(Tag::IndependentSegments(IndependentSegments)),
            Tag::try_from(ParsedTag {
                name: "-X-INDEPENDENT-SEGMENTS",
                value: ParsedTagValue::Empty
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
                )]))
            })
        );
        let expected = Tag::Start(Start::new(10.0, true));
        let actual = Tag::try_from(ParsedTag {
            name: "-X-START",
            value: ParsedTagValue::AttributeList(HashMap::from([
                ("TIME-OFFSET", ParsedAttributeValue::DecimalInteger(10)),
                ("PRECISE", ParsedAttributeValue::UnquotedString("YES")),
            ])),
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
            Ok(Tag::Define(Define::Name {
                name: "TEST",
                value: "GOOD"
            })),
            Tag::try_from(ParsedTag {
                name: "-X-DEFINE",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("NAME", ParsedAttributeValue::QuotedString("TEST")),
                    ("VALUE", ParsedAttributeValue::QuotedString("GOOD"))
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Define(Define::Import("TEST"))),
            Tag::try_from(ParsedTag {
                name: "-X-DEFINE",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "IMPORT",
                    ParsedAttributeValue::QuotedString("TEST")
                )]))
            })
        );
        assert_eq!(
            Ok(Tag::Define(Define::Queryparam("testQueryParam"))),
            Tag::try_from(ParsedTag {
                name: "-X-DEFINE",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "QUERYPARAM",
                    ParsedAttributeValue::QuotedString("testQueryParam")
                )]))
            })
        );
    }

    #[test]
    fn targetduration() {
        assert_eq!(
            Ok(Tag::Targetduration(Targetduration::new(6))),
            Tag::try_from(ParsedTag {
                name: "-X-TARGETDURATION",
                value: ParsedTagValue::DecimalInteger(6)
            })
        );
    }

    #[test]
    fn media_sequence() {
        assert_eq!(
            Ok(Tag::MediaSequence(MediaSequence::new(100))),
            Tag::try_from(ParsedTag {
                name: "-X-MEDIA-SEQUENCE",
                value: ParsedTagValue::DecimalInteger(100)
            })
        );
    }

    #[test]
    fn discontinuity_sequencee() {
        assert_eq!(
            Ok(Tag::DiscontinuitySequence(DiscontinuitySequence::new(100))),
            Tag::try_from(ParsedTag {
                name: "-X-DISCONTINUITY-SEQUENCE",
                value: ParsedTagValue::DecimalInteger(100)
            })
        );
    }

    #[test]
    fn endlist() {
        assert_eq!(
            Ok(Tag::Endlist(Endlist)),
            Tag::try_from(ParsedTag {
                name: "-X-ENDLIST",
                value: ParsedTagValue::Empty
            })
        )
    }

    #[test]
    fn playlist_type() {
        assert_eq!(
            Ok(Tag::PlaylistType(PlaylistType::new(HlsPlaylistType::Event))),
            Tag::try_from(ParsedTag {
                name: "-X-PLAYLIST-TYPE",
                value: ParsedTagValue::TypeEnum(HlsPlaylistType::Event)
            })
        );
        assert_eq!(
            Ok(Tag::PlaylistType(PlaylistType::new(HlsPlaylistType::Vod))),
            Tag::try_from(ParsedTag {
                name: "-X-PLAYLIST-TYPE",
                value: ParsedTagValue::TypeEnum(HlsPlaylistType::Vod)
            })
        );
    }

    #[test]
    fn i_frames_only() {
        assert_eq!(
            Ok(Tag::IFramesOnly(IFramesOnly)),
            Tag::try_from(ParsedTag {
                name: "-X-I-FRAMES-ONLY",
                value: ParsedTagValue::Empty
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
                )]))
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
                ]))
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
                value: ParsedTagValue::AttributeList(HashMap::new())
            })
        );
    }

    #[test]
    fn inf() {
        assert_eq!(
            Ok(Tag::Inf(Inf::new(6.0, ""))),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalInteger(6)
            })
        );
        assert_eq!(
            Ok(Tag::Inf(Inf::new(6.006, ""))),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(6.006, "")
            })
        );
        assert_eq!(
            Ok(Tag::Inf(Inf::new(6.006, "A useful title"))),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(
                    6.006,
                    "A useful title"
                )
            })
        );
    }

    #[test]
    fn byterange() {
        assert_eq!(
            Ok(Tag::Byterange(Byterange::new(1024, Some(512)))),
            Tag::try_from(ParsedTag {
                name: "-X-BYTERANGE",
                value: ParsedTagValue::DecimalIntegerRange(1024, 512)
            })
        );
        assert_eq!(
            Ok(Tag::Byterange(Byterange::new(1024, None))),
            Tag::try_from(ParsedTag {
                name: "-X-BYTERANGE",
                value: ParsedTagValue::DecimalInteger(1024)
            })
        );
    }

    #[test]
    fn discontinuity() {
        assert_eq!(
            Ok(Tag::Discontinuity(Discontinuity)),
            Tag::try_from(ParsedTag {
                name: "-X-DISCONTINUITY",
                value: ParsedTagValue::Empty
            })
        );
    }

    #[test]
    fn key() {
        assert_eq!(
            Ok(Tag::Key(Key::new(
                "SAMPLE-AES",
                Some("skd://some-key-id"),
                Some("0xABCD"),
                Some("com.apple.streamingkeydelivery"),
                Some("1"),
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Key(Key::new("NONE", None, None, None, None,))),
            Tag::try_from(ParsedTag {
                name: "-X-KEY",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "METHOD",
                    ParsedAttributeValue::UnquotedString("NONE")
                )]))
            })
        );
    }

    #[test]
    fn map() {
        assert_eq!(
            Ok(Tag::Map(Map::new(
                "init.mp4",
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Map(Map::new("init.mp4", None,))),
            Tag::try_from(ParsedTag {
                name: "-X-MAP",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "URI",
                    ParsedAttributeValue::QuotedString("init.mp4")
                ),]))
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
                value: ParsedTagValue::DateTimeMsec(date_time)
            })
        );
    }

    #[test]
    fn gap() {
        assert_eq!(
            Ok(Tag::Gap(Gap)),
            Tag::try_from(ParsedTag {
                name: "-X-GAP",
                value: ParsedTagValue::Empty
            })
        );
    }

    #[test]
    fn bitrate() {
        assert_eq!(
            Ok(Tag::Bitrate(Bitrate::new(10000000))),
            Tag::try_from(ParsedTag {
                name: "-X-BITRATE",
                value: ParsedTagValue::DecimalInteger(10000000)
            })
        );
    }

    #[test]
    fn part() {
        assert_eq!(
            Ok(Tag::Part(Part::new(
                "part.1.mp4",
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Part(Part::new(
                "part.1.mp4",
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Part(Part::new("part.1.mp4", 0.5, false, None, false,))),
            Tag::try_from(ParsedTag {
                name: "-X-PART",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("part.1.mp4")),
                    (
                        "DURATION",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(0.5)
                    ),
                ]))
            })
        );
    }

    #[test]
    fn daterange() {
        assert_eq!(
            Ok(Tag::Daterange(Daterange::new(
                "test",
                Some("com.m3u8.test"),
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
                Some("ONCE"),
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
                HashMap::from([("X-COM-M3U8-TEST", ParsedAttributeValue::QuotedString("YES"))]),
                Some("0xABCD"),
                Some("0xABCD"),
                Some("0xABCD"),
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Daterange(Daterange::new(
                "test",
                None,
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
                None,
                None,
                None,
                None,
                HashMap::new(),
                None,
                None,
                None,
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
                ]))
            })
        );
    }

    #[test]
    fn skip() {
        assert_eq!(
            Ok(Tag::Skip(Skip::new(100, Some("1234\tabcd"),))),
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Skip(Skip::new(100, None,))),
            Tag::try_from(ParsedTag {
                name: "-X-SKIP",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "SKIPPED-SEGMENTS",
                    ParsedAttributeValue::DecimalInteger(100)
                ),]))
            })
        );
    }

    #[test]
    fn preload_hint() {
        assert_eq!(
            Ok(Tag::PreloadHint(PreloadHint::new(
                "PART",
                "part.2.mp4",
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::PreloadHint(PreloadHint::new(
                "PART",
                "part.2.mp4",
                None,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-PRELOAD-HINT",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("TYPE", ParsedAttributeValue::UnquotedString("PART")),
                    ("URI", ParsedAttributeValue::QuotedString("part.2.mp4")),
                ]))
            })
        );
    }

    #[test]
    fn rendition_report() {
        assert_eq!(
            Ok(Tag::RenditionReport(RenditionReport::new(
                "high.m3u8",
                1000,
                Some(2),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-RENDITION-REPORT",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("high.m3u8")),
                    ("LAST-MSN", ParsedAttributeValue::DecimalInteger(1000)),
                    ("LAST-PART", ParsedAttributeValue::DecimalInteger(2)),
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::RenditionReport(RenditionReport::new(
                "high.m3u8",
                1000,
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-RENDITION-REPORT",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("high.m3u8")),
                    ("LAST-MSN", ParsedAttributeValue::DecimalInteger(1000)),
                ]))
            })
        );
    }

    #[test]
    fn media() {
        assert_eq!(
            Ok(Tag::Media(Media::new(
                "AUDIO",
                "English",
                "stereo",
                Some("audio/en/stereo.m3u8"),
                Some("en"),
                Some("en"),
                Some("1234"),
                true,
                true,
                true,
                None,
                Some(8),
                Some(48000),
                Some("public.accessibility.describes-video"),
                Some("2"),
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Media(Media::new(
                "CLOSED-CAPTIONS",
                "English",
                "cc",
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                Some("CC1"),
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
                ]))
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
                Some("hvc1.2.4.L153.b0,ec-3"),
                Some("dvh1.08.07/db4h"),
                Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                Some(23.976),
                Some("TYPE-1"),
                Some("com.example.drm1:SMART-TV/PC"),
                Some("PQ"),
                Some("CH-STEREO,CH-MONO"),
                Some("1234"),
                Some("surround"),
                Some("alternate-view"),
                Some("subs"),
                Some("cc"),
                Some("1234"),
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
                ]))
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
                ]))
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
                )]))
            })
        );
    }

    #[test]
    fn i_frame_stream_inf() {
        assert_eq!(
            Ok(Tag::IFrameStreamInf(IFrameStreamInf::new(
                "iframe.high.m3u8",
                10000000,
                Some(9000000),
                Some(2.0),
                Some("hvc1.2.4.L153.b0,ec-3"),
                Some("dvh1.08.07/db4h"),
                Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                Some("TYPE-1"),
                Some("com.example.drm1:SMART-TV/PC"),
                Some("PQ"),
                Some("CH-STEREO,CH-MONO"),
                Some("1234"),
                Some("alternate-view"),
                Some("1234"),
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::IFrameStreamInf(IFrameStreamInf::new(
                "iframe.high.m3u8",
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
                ]))
            })
        );
    }

    #[test]
    fn session_data() {
        assert_eq!(
            Ok(Tag::SessionData(SessionData::new(
                "1234",
                Some("test"),
                None,
                None,
                Some("en"),
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SESSION-DATA",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("DATA-ID", ParsedAttributeValue::QuotedString("1234")),
                    ("VALUE", ParsedAttributeValue::QuotedString("test")),
                    ("LANGUAGE", ParsedAttributeValue::QuotedString("en")),
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::SessionData(SessionData::new(
                "1234",
                None,
                Some("test.bin"),
                Some("RAW"),
                None,
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-SESSION-DATA",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("DATA-ID", ParsedAttributeValue::QuotedString("1234")),
                    ("URI", ParsedAttributeValue::QuotedString("test.bin")),
                    ("FORMAT", ParsedAttributeValue::UnquotedString("RAW")),
                ]))
            })
        );
    }

    #[test]
    fn session_key() {
        assert_eq!(
            Ok(Tag::SessionKey(SessionKey::new(
                "SAMPLE-AES",
                "skd://some-key-id",
                Some("0xABCD"),
                Some("com.apple.streamingkeydelivery"),
                Some("1"),
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
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::SessionKey(SessionKey::new(
                "AES-128",
                "skd://some-key-id",
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
                ]))
            })
        );
    }

    #[test]
    fn content_steering() {
        assert_eq!(
            Ok(Tag::ContentSteering(ContentSteering::new(
                "content-steering.json",
                Some("1234")
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-CONTENT-STEERING",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    (
                        "SERVER-URI",
                        ParsedAttributeValue::QuotedString("content-steering.json")
                    ),
                    ("PATHWAY-ID", ParsedAttributeValue::QuotedString("1234")),
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::ContentSteering(ContentSteering::new(
                "content-steering.json",
                None
            ))),
            Tag::try_from(ParsedTag {
                name: "-X-CONTENT-STEERING",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "SERVER-URI",
                    ParsedAttributeValue::QuotedString("content-steering.json")
                ),]))
            })
        );
    }
}
