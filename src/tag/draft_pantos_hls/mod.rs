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
    known::{IsKnownName, ParsedTag},
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
    Start(Start),
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
    PartInf(PartInf),
    /// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.8
    ServerControl(ServerControl),
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

impl IsKnownName for Tag<'_> {
    fn is_known_name(name: &str) -> bool {
        TagName::try_from(name).is_ok()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
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
            Ok(Tag::Version(Version(9))),
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
            Ok(Tag::Start(Start {
                time_offset: 10.0,
                precise: false
            })),
            Tag::try_from(ParsedTag {
                name: "-X-START",
                value: ParsedTagValue::AttributeList(HashMap::from([(
                    "TIME-OFFSET",
                    ParsedAttributeValue::SignedDecimalFloatingPoint(10.0)
                )]))
            })
        );
        assert_eq!(
            Ok(Tag::Start(Start {
                time_offset: 10.0,
                precise: true
            })),
            Tag::try_from(ParsedTag {
                name: "-X-START",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("TIME-OFFSET", ParsedAttributeValue::DecimalInteger(10)),
                    ("PRECISE", ParsedAttributeValue::UnquotedString("YES"))
                ]))
            })
        );
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
            Ok(Tag::Targetduration(Targetduration(6))),
            Tag::try_from(ParsedTag {
                name: "-X-TARGETDURATION",
                value: ParsedTagValue::DecimalInteger(6)
            })
        );
    }

    #[test]
    fn media_sequence() {
        assert_eq!(
            Ok(Tag::MediaSequence(MediaSequence(100))),
            Tag::try_from(ParsedTag {
                name: "-X-MEDIA-SEQUENCE",
                value: ParsedTagValue::DecimalInteger(100)
            })
        );
    }

    #[test]
    fn discontinuity_sequencee() {
        assert_eq!(
            Ok(Tag::DiscontinuitySequence(DiscontinuitySequence(100))),
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
            Ok(Tag::PlaylistType(PlaylistType(HlsPlaylistType::Event))),
            Tag::try_from(ParsedTag {
                name: "-X-PLAYLIST-TYPE",
                value: ParsedTagValue::TypeEnum(HlsPlaylistType::Event)
            })
        );
        assert_eq!(
            Ok(Tag::PlaylistType(PlaylistType(HlsPlaylistType::Vod))),
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
            Ok(Tag::PartInf(PartInf { part_target: 0.5 })),
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
            Ok(Tag::ServerControl(ServerControl {
                can_skip_until: Some(36.0),
                can_skip_dateranges: true,
                hold_back: Some(12.0),
                part_hold_back: Some(1.5),
                can_block_reload: true,
            })),
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
            Ok(Tag::ServerControl(ServerControl {
                can_skip_until: None,
                can_skip_dateranges: false,
                hold_back: None,
                part_hold_back: None,
                can_block_reload: false,
            })),
            Tag::try_from(ParsedTag {
                name: "-X-SERVER-CONTROL",
                value: ParsedTagValue::AttributeList(HashMap::new())
            })
        );
    }

    #[test]
    fn inf() {
        assert_eq!(
            Ok(Tag::Inf(Inf {
                duration: 6.0,
                title: ""
            })),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalInteger(6)
            })
        );
        assert_eq!(
            Ok(Tag::Inf(Inf {
                duration: 6.006,
                title: ""
            })),
            Tag::try_from(ParsedTag {
                name: "INF",
                value: ParsedTagValue::DecimalFloatingPointWithOptionalTitle(6.006, "")
            })
        );
        assert_eq!(
            Ok(Tag::Inf(Inf {
                duration: 6.006,
                title: "A useful title"
            })),
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
            Ok(Tag::Byterange(Byterange {
                length: 1024,
                offset: Some(512)
            })),
            Tag::try_from(ParsedTag {
                name: "-X-BYTERANGE",
                value: ParsedTagValue::DecimalIntegerRange(1024, 512)
            })
        );
        assert_eq!(
            Ok(Tag::Byterange(Byterange {
                length: 1024,
                offset: None
            })),
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
            Ok(Tag::Key(Key {
                method: "SAMPLE-AES",
                uri: Some("skd://some-key-id"),
                iv: Some("0xABCD"),
                keyformat: "com.apple.streamingkeydelivery",
                keyformatversions: Some("1"),
            })),
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
            Ok(Tag::Key(Key {
                method: "NONE",
                uri: None,
                iv: None,
                keyformat: "identity",
                keyformatversions: None,
            })),
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
            Ok(Tag::Map(Map {
                uri: "init.mp4",
                byterange: Some(MapByterange {
                    length: 1024,
                    offset: 0
                })
            })),
            Tag::try_from(ParsedTag {
                name: "-X-MAP",
                value: ParsedTagValue::AttributeList(HashMap::from([
                    ("URI", ParsedAttributeValue::QuotedString("init.mp4")),
                    ("BYTERANGE", ParsedAttributeValue::QuotedString("1024@0")),
                ]))
            })
        );
        assert_eq!(
            Ok(Tag::Map(Map {
                uri: "init.mp4",
                byterange: None,
            })),
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
            Ok(Tag::ProgramDateTime(ProgramDateTime(date_time))),
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
            Ok(Tag::Bitrate(Bitrate(10000000))),
            Tag::try_from(ParsedTag {
                name: "-X-BITRATE",
                value: ParsedTagValue::DecimalInteger(10000000)
            })
        );
    }

    #[test]
    fn part() {
        assert_eq!(
            Ok(Tag::Part(Part {
                uri: "part.1.mp4",
                duration: 0.5,
                independent: true,
                byterange: Some(PartByterange {
                    length: 1024,
                    offset: Some(512)
                }),
                gap: true,
            })),
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
            Ok(Tag::Part(Part {
                uri: "part.1.mp4",
                duration: 0.5,
                independent: false,
                byterange: Some(PartByterange {
                    length: 1024,
                    offset: None
                }),
                gap: false,
            })),
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
            Ok(Tag::Part(Part {
                uri: "part.1.mp4",
                duration: 0.5,
                independent: false,
                byterange: None,
                gap: false,
            })),
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
            Ok(Tag::Daterange(Daterange {
                id: "test",
                class: Some("com.m3u8.test"),
                start_date: DateTime {
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
                cue: Some("ONCE"),
                end_date: Some(DateTime {
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
                duration: Some(120.0),
                planned_duration: Some(180.0),
                client_attributes: HashMap::from([(
                    "X-COM-M3U8-TEST",
                    ParsedAttributeValue::QuotedString("YES")
                )]),
                scte35_cmd: Some("0xABCD"),
                scte35_out: Some("0xABCD"),
                scte35_in: Some("0xABCD"),
                end_on_next: true,
            })),
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
            Ok(Tag::Daterange(Daterange {
                id: "test",
                class: None,
                start_date: DateTime {
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
                cue: None,
                end_date: None,
                duration: None,
                planned_duration: None,
                client_attributes: HashMap::new(),
                scte35_cmd: None,
                scte35_out: None,
                scte35_in: None,
                end_on_next: false,
            })),
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
            Ok(Tag::Skip(Skip {
                skipped_segments: 100,
                recently_removed_dateranges: Some("1234\tabcd"),
            })),
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
            Ok(Tag::Skip(Skip {
                skipped_segments: 100,
                recently_removed_dateranges: None,
            })),
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
            Ok(Tag::PreloadHint(PreloadHint {
                hint_type: "PART",
                uri: "part.2.mp4",
                byterange_start: 512,
                byterange_length: Some(1024),
            })),
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
            Ok(Tag::PreloadHint(PreloadHint {
                hint_type: "PART",
                uri: "part.2.mp4",
                byterange_start: 0,
                byterange_length: None,
            })),
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
            Ok(Tag::RenditionReport(RenditionReport {
                uri: "high.m3u8",
                last_msn: 1000,
                last_part: Some(2),
            })),
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
            Ok(Tag::RenditionReport(RenditionReport {
                uri: "high.m3u8",
                last_msn: 1000,
                last_part: None,
            })),
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
            Ok(Tag::Media(Media {
                media_type: "AUDIO",
                uri: Some("audio/en/stereo.m3u8"),
                group_id: "stereo",
                language: Some("en"),
                assoc_language: Some("en"),
                name: "English",
                stable_rendition_id: Some("1234"),
                default: true,
                autoselect: true,
                forced: true,
                instream_id: None,
                bit_depth: Some(8),
                sample_rate: Some(48000),
                characteristics: Some("public.accessibility.describes-video"),
                channels: Some("2"),
            })),
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
            Ok(Tag::Media(Media {
                media_type: "CLOSED-CAPTIONS",
                uri: None,
                group_id: "cc",
                language: None,
                assoc_language: None,
                name: "English",
                stable_rendition_id: None,
                default: false,
                autoselect: false,
                forced: false,
                instream_id: Some("CC1"),
                bit_depth: None,
                sample_rate: None,
                characteristics: None,
                channels: None,
            })),
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
            Ok(Tag::StreamInf(StreamInf {
                bandwidth: 10000000,
                average_bandwidth: Some(9000000),
                score: Some(2.0),
                codecs: Some("hvc1.2.4.L153.b0,ec-3"),
                supplemental_codecs: Some("dvh1.08.07/db4h"),
                resolution: Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                frame_rate: Some(23.976),
                hdcp_level: Some("TYPE-1"),
                allowed_cpc: Some("com.example.drm1:SMART-TV/PC"),
                video_range: Some("PQ"),
                req_video_layout: Some("CH-STEREO,CH-MONO"),
                stable_variant_id: Some("1234"),
                audio: Some("surround"),
                video: Some("alternate-view"),
                subtitles: Some("subs"),
                closed_captions: Some("cc"),
                pathway_id: Some("1234"),
            })),
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
            Ok(Tag::StreamInf(StreamInf {
                bandwidth: 10000000,
                average_bandwidth: None,
                score: None,
                codecs: None,
                supplemental_codecs: None,
                resolution: None,
                frame_rate: Some(25.0),
                hdcp_level: None,
                allowed_cpc: None,
                video_range: None,
                req_video_layout: None,
                stable_variant_id: None,
                audio: None,
                video: None,
                subtitles: None,
                closed_captions: None,
                pathway_id: None,
            })),
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
            Ok(Tag::StreamInf(StreamInf {
                bandwidth: 10000000,
                average_bandwidth: None,
                score: None,
                codecs: None,
                supplemental_codecs: None,
                resolution: None,
                frame_rate: None,
                hdcp_level: None,
                allowed_cpc: None,
                video_range: None,
                req_video_layout: None,
                stable_variant_id: None,
                audio: None,
                video: None,
                subtitles: None,
                closed_captions: None,
                pathway_id: None,
            })),
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
            Ok(Tag::IFrameStreamInf(IFrameStreamInf {
                uri: "iframe.high.m3u8",
                bandwidth: 10000000,
                average_bandwidth: Some(9000000),
                score: Some(2.0),
                codecs: Some("hvc1.2.4.L153.b0,ec-3"),
                supplemental_codecs: Some("dvh1.08.07/db4h"),
                resolution: Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                hdcp_level: Some("TYPE-1"),
                allowed_cpc: Some("com.example.drm1:SMART-TV/PC"),
                video_range: Some("PQ"),
                req_video_layout: Some("CH-STEREO,CH-MONO"),
                stable_variant_id: Some("1234"),
                video: Some("alternate-view"),
                pathway_id: Some("1234"),
            })),
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
            Ok(Tag::IFrameStreamInf(IFrameStreamInf {
                uri: "iframe.high.m3u8",
                bandwidth: 10000000,
                average_bandwidth: None,
                score: None,
                codecs: None,
                supplemental_codecs: None,
                resolution: None,
                hdcp_level: None,
                allowed_cpc: None,
                video_range: None,
                req_video_layout: None,
                stable_variant_id: None,
                video: None,
                pathway_id: None,
            })),
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
            Ok(Tag::SessionData(SessionData {
                data_id: "1234",
                value: Some("test"),
                uri: None,
                format: "JSON",
                language: Some("en"),
            })),
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
            Ok(Tag::SessionData(SessionData {
                data_id: "1234",
                value: None,
                uri: Some("test.bin"),
                format: "RAW",
                language: None,
            })),
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
            Ok(Tag::SessionKey(SessionKey {
                method: "SAMPLE-AES",
                uri: "skd://some-key-id",
                iv: Some("0xABCD"),
                keyformat: "com.apple.streamingkeydelivery",
                keyformatversions: Some("1"),
            })),
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
            Ok(Tag::SessionKey(SessionKey {
                method: "AES-128",
                uri: "skd://some-key-id",
                iv: None,
                keyformat: "identity",
                keyformatversions: None,
            })),
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
            Ok(Tag::ContentSteering(ContentSteering {
                server_uri: "content-steering.json",
                pathway_id: Some("1234"),
            })),
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
            Ok(Tag::ContentSteering(ContentSteering {
                server_uri: "content-steering.json",
                pathway_id: None,
            })),
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
