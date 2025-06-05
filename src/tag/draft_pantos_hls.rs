use crate::{
    date::DateTime,
    tag::{
        known::{IsKnownName, ParsedTag},
        value::{DecimalResolution, HlsPlaylistType, ParsedAttributeValue, ParsedTagValue},
    },
};
use std::collections::HashMap;

// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17
// 4.4.  Playlist Tags
//     4.4.1.  Basic Tags
//         4.4.1.1.  EXTM3U
#[derive(Debug, PartialEq)]
pub struct M3u;
//         4.4.1.2.  EXT-X-VERSION
#[derive(Debug, PartialEq)]
pub struct Version(u64);
//     4.4.2.  Media or Multivariant Playlist Tags
//         4.4.2.1.  EXT-X-INDEPENDENT-SEGMENTS
#[derive(Debug, PartialEq)]
pub struct IndependentSegments;
//         4.4.2.2.  EXT-X-START
#[derive(Debug, PartialEq)]
pub struct Start {
    pub time_offset: f64,
    pub precise: bool,
}
//         4.4.2.3.  EXT-X-DEFINE
#[derive(Debug, PartialEq)]
pub enum Define<'a> {
    Name { name: &'a str, value: &'a str },
    Import(&'a str),
    Queryparam(&'a str),
}
//     4.4.3.  Media Playlist Tags
//         4.4.3.1.  EXT-X-TARGETDURATION
#[derive(Debug, PartialEq)]
pub struct Targetduration(u64);
//         4.4.3.2.  EXT-X-MEDIA-SEQUENCE
#[derive(Debug, PartialEq)]
pub struct MediaSequence(u64);
//         4.4.3.3.  EXT-X-DISCONTINUITY-SEQUENCE
#[derive(Debug, PartialEq)]
pub struct DiscontinuitySequence(u64);
//         4.4.3.4.  EXT-X-ENDLIST
#[derive(Debug, PartialEq)]
pub struct Endlist;
//         4.4.3.5.  EXT-X-PLAYLIST-TYPE
#[derive(Debug, PartialEq)]
pub struct PlaylistType(HlsPlaylistType);
//         4.4.3.6.  EXT-X-I-FRAMES-ONLY
#[derive(Debug, PartialEq)]
pub struct IFramesOnly;
//         4.4.3.7.  EXT-X-PART-INF
#[derive(Debug, PartialEq)]
pub struct PartInf {
    pub part_target: f64,
}
//         4.4.3.8.  EXT-X-SERVER-CONTROL
#[derive(Debug, PartialEq)]
pub struct ServerControl {
    pub can_skip_until: Option<f64>,
    pub can_skip_dateranges: bool,
    pub hold_back: Option<f64>,
    pub part_hold_back: Option<f64>,
    pub can_block_reload: bool,
}
//     4.4.4.  Media Segment Tags
//         4.4.4.1.  EXTINF
#[derive(Debug, PartialEq)]
pub struct Inf<'a> {
    pub duration: f64,
    pub title: &'a str,
}
//         4.4.4.2.  EXT-X-BYTERANGE
#[derive(Debug, PartialEq)]
pub struct Byterange {
    pub length: u64,
    pub offset: Option<u64>,
}
//         4.4.4.3.  EXT-X-DISCONTINUITY
#[derive(Debug, PartialEq)]
pub struct Discontinuity;
//         4.4.4.4.  EXT-X-KEY
#[derive(Debug, PartialEq)]
pub struct Key<'a> {
    pub method: &'a str,
    pub uri: Option<&'a str>,
    pub iv: Option<&'a str>,
    pub keyformat: Option<&'a str>,
    pub keyformatversions: Option<&'a str>,
}
//         4.4.4.5.  EXT-X-MAP
#[derive(Debug, PartialEq)]
pub struct Map<'a> {
    pub uri: &'a str,
    pub byterange: Option<MapByterange>,
}
#[derive(Debug, PartialEq)]
pub struct MapByterange {
    pub length: u64,
    pub offset: u64,
}
//         4.4.4.6.  EXT-X-PROGRAM-DATE-TIME
#[derive(Debug, PartialEq)]
pub struct ProgramDateTime(DateTime);
//         4.4.4.7.  EXT-X-GAP
#[derive(Debug, PartialEq)]
pub struct Gap;
//         4.4.4.8.  EXT-X-BITRATE
#[derive(Debug, PartialEq)]
pub struct Bitrate(u64);
//         4.4.4.9.  EXT-X-PART
#[derive(Debug, PartialEq)]
pub struct Part<'a> {
    pub uri: &'a str,
    pub duration: f64,
    pub independent: bool,
    pub byterange: Option<PartByterange>,
    pub gap: bool,
}
#[derive(Debug, PartialEq)]
pub struct PartByterange {
    pub length: u64,
    pub offset: Option<u64>,
}
//     4.4.5.  Media Metadata Tags
//         4.4.5.1.  EXT-X-DATERANGE
#[derive(Debug, PartialEq)]
pub struct Daterange<'a> {
    pub id: &'a str,
    pub class: Option<&'a str>,
    pub start_date: DateTime,
    pub cue: Option<&'a str>,
    pub end_date: Option<DateTime>,
    pub duration: Option<f64>,
    pub planned_duration: Option<f64>,
    pub client_attributes: HashMap<&'a str, ParsedAttributeValue<'a>>,
    pub scte35_cmd: Option<&'a str>,
    pub scte35_out: Option<&'a str>,
    pub scte35_in: Option<&'a str>,
    pub end_on_next: bool,
}
//         4.4.5.2.  EXT-X-SKIP
#[derive(Debug, PartialEq)]
pub struct Skip<'a> {
    pub skipped_segments: u64,
    pub recently_removed_dateranges: Option<&'a str>,
}
//         4.4.5.3.  EXT-X-PRELOAD-HINT
#[derive(Debug, PartialEq)]
pub struct PreloadHint<'a> {
    pub hint_type: &'a str,
    pub uri: &'a str,
    pub byterange_start: u64,
    pub byterange_length: Option<u64>,
}
//         4.4.5.4.  EXT-X-RENDITION-REPORT
#[derive(Debug, PartialEq)]
pub struct RenditionReport<'a> {
    pub uri: &'a str,
    pub last_msn: u64,
    pub last_part: Option<u64>,
}
//     4.4.6.  Multivariant Playlist Tags
//         4.4.6.1.  EXT-X-MEDIA
#[derive(Debug, PartialEq)]
pub struct Media<'a> {
    pub media_type: &'a str,
    pub uri: Option<&'a str>,
    pub group_id: &'a str,
    pub language: Option<&'a str>,
    pub assoc_language: Option<&'a str>,
    pub name: &'a str,
    pub stable_rendition_id: Option<&'a str>,
    pub default: bool,
    pub autoselect: bool,
    pub forced: bool,
    pub instream_id: Option<&'a str>,
    pub bit_depth: Option<u64>,
    pub sample_rate: Option<u64>,
    pub characteristics: Option<&'a str>,
    pub channels: Option<&'a str>,
}
//         4.4.6.2.  EXT-X-STREAM-INF
#[derive(Debug, PartialEq)]
pub struct StreamInf<'a> {
    pub bandwidth: u64,
    pub average_bandwidth: Option<u64>,
    pub score: Option<f64>,
    pub codecs: Option<&'a str>,
    pub supplemental_codecs: Option<&'a str>,
    pub resolution: Option<DecimalResolution>,
    pub frame_rate: f64,
    pub hdcp_level: Option<&'a str>,
    pub allowed_cpc: Option<&'a str>,
    pub video_range: Option<&'a str>,
    pub req_video_layout: Option<&'a str>,
    pub stable_variant_id: Option<&'a str>,
    pub audio: Option<&'a str>,
    pub video: Option<&'a str>,
    pub subtitles: Option<&'a str>,
    pub closed_captions: Option<&'a str>,
    pub pathway_id: Option<&'a str>,
}
//         4.4.6.3.  EXT-X-I-FRAME-STREAM-INF
#[derive(Debug, PartialEq)]
pub struct IFrameStreamInf<'a> {
    pub uri: &'a str,
    pub bandwidth: u64,
    pub average_bandwidth: Option<u64>,
    pub score: Option<f64>,
    pub codecs: Option<&'a str>,
    pub supplemental_codecs: Option<&'a str>,
    pub resolution: Option<DecimalResolution>,
    pub hdcp_level: Option<&'a str>,
    pub allowed_cpc: Option<&'a str>,
    pub video_range: Option<&'a str>,
    pub req_video_layout: Option<&'a str>,
    pub stable_variant_id: Option<&'a str>,
    pub video: Option<&'a str>,
    pub pathway_id: Option<&'a str>,
}
//         4.4.6.4.  EXT-X-SESSION-DATA
#[derive(Debug, PartialEq)]
pub struct SessionData<'a> {
    pub data_id: &'a str,
    pub value: Option<&'a str>,
    pub uri: Option<&'a str>,
    pub format: &'a str,
    pub language: Option<&'a str>,
}
//         4.4.6.5.  EXT-X-SESSION-KEY
#[derive(Debug, PartialEq)]
pub struct SessionKey<'a> {
    pub method: &'a str,
    pub uri: Option<&'a str>,
    pub iv: Option<&'a str>,
    pub keyformat: Option<&'a str>,
    pub keyformatversions: Option<&'a str>,
}
//         4.4.6.6.  EXT-X-CONTENT-STEERING
#[derive(Debug, PartialEq)]
pub struct ContentSteering<'a> {
    pub server_uri: &'a str,
    pub pathway_id: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
pub enum Tag<'a> {
    M3u(M3u),
    Version(Version),
    IndependentSegments(IndependentSegments),
    Start(Start),
    Define(Define<'a>),
    Targetduration(Targetduration),
    MediaSequence(MediaSequence),
    DiscontinuitySequence(DiscontinuitySequence),
    Endlist(Endlist),
    PlaylistType(PlaylistType),
    IFramesOnly(IFramesOnly),
    PartInf(PartInf),
    ServerControl(ServerControl),
    Inf(Inf<'a>),
    Byterange(Byterange),
    Discontinuity(Discontinuity),
    Key(Key<'a>),
    Map(Map<'a>),
    ProgramDateTime(ProgramDateTime),
    Gap(Gap),
    Bitrate(Bitrate),
    Part(Part<'a>),
    Daterange(Daterange<'a>),
    Skip(Skip<'a>),
    PreloadHint(PreloadHint<'a>),
    RenditionReport(RenditionReport<'a>),
    Media(Media<'a>),
    StreamInf(StreamInf<'a>),
    IFrameStreamInf(IFrameStreamInf<'a>),
    SessionData(SessionData<'a>),
    SessionKey(SessionKey<'a>),
    ContentSteering(ContentSteering<'a>),
}

impl<'a> TryFrom<ParsedTag<'a>> for Tag<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        match tag.name {
            "M3U" => Ok(Self::M3u(M3u)),
            "-X-VERSION" => {
                let ParsedTagValue::DecimalInteger(version) = tag.value else {
                    return Self::unexpected_value_type();
                };
                Ok(Self::Version(Version(version)))
            }
            "-X-INDEPENDENT-SEGMENTS" => Ok(Self::IndependentSegments(IndependentSegments)),
            "-X-START" => {
                let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(Some(time_offset)) = attribute_list
                    .get("TIME-OFFSET")
                    .map(ParsedAttributeValue::as_option_f64)
                else {
                    return Self::missing_required_attribute();
                };
                let precise = attribute_list
                    .get("PRECISE")
                    .map(|v| v.as_option_unquoted_str() == Some("YES"))
                    .unwrap_or(false);
                Ok(Self::Start(Start {
                    time_offset,
                    precise,
                }))
            }
            "-X-DEFINE" => {
                let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                if let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.get("NAME") {
                    if let Some(ParsedAttributeValue::QuotedString(value)) =
                        attribute_list.get("VALUE")
                    {
                        Ok(Self::Define(Define::Name { name, value }))
                    } else {
                        Self::missing_required_attribute()
                    }
                } else if let Some(ParsedAttributeValue::QuotedString(import)) =
                    attribute_list.get("IMPORT")
                {
                    Ok(Self::Define(Define::Import(import)))
                } else if let Some(ParsedAttributeValue::QuotedString(queryparam)) =
                    attribute_list.get("QUERYPARAM")
                {
                    Ok(Self::Define(Define::Queryparam(queryparam)))
                } else {
                    Self::missing_required_attribute()
                }
            }
            "-X-TARGETDURATION" => {
                let ParsedTagValue::DecimalInteger(d) = tag.value else {
                    return Self::unexpected_value_type();
                };
                Ok(Tag::Targetduration(Targetduration(d)))
            }
            "-X-MEDIA-SEQUENCE" => {
                let ParsedTagValue::DecimalInteger(d) = tag.value else {
                    return Self::unexpected_value_type();
                };
                Ok(Tag::MediaSequence(MediaSequence(d)))
            }
            "-X-DISCONTINUITY-SEQUENCE" => {
                let ParsedTagValue::DecimalInteger(d) = tag.value else {
                    return Self::unexpected_value_type();
                };
                Ok(Tag::DiscontinuitySequence(DiscontinuitySequence(d)))
            }
            "-X-ENDLIST" => Ok(Tag::Endlist(Endlist)),
            "-X-PLAYLIST-TYPE" => {
                let ParsedTagValue::TypeEnum(t) = tag.value else {
                    return Self::unexpected_value_type();
                };
                Ok(Tag::PlaylistType(PlaylistType(t)))
            }
            "-X-I-FRAMES-ONLY" => Ok(Tag::IFramesOnly(IFramesOnly)),
            "-X-PART-INF" => {
                let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(Some(part_target)) = attribute_list
                    .get("PART-TARGET")
                    .map(ParsedAttributeValue::as_option_f64)
                else {
                    return Self::missing_required_attribute();
                };
                Ok(Tag::PartInf(PartInf { part_target }))
            }
            "-X-SERVER-CONTROL" => {
                let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let mut can_skip_until = None;
                let mut can_skip_dateranges = false;
                let mut hold_back = None;
                let mut part_hold_back = None;
                let mut can_block_reload = false;
                for (key, value) in attribute_list {
                    match key {
                        "CAN-SKIP-UNTIL" => can_skip_until = value.as_option_f64(),
                        "CAN-SKIP-DATERANGES" => {
                            can_skip_dateranges = value.as_option_unquoted_str() == Some("YES")
                        }
                        "HOLD-BACK" => hold_back = value.as_option_f64(),
                        "PART-HOLD-BACK" => part_hold_back = value.as_option_f64(),
                        "CAN-BLOCK-RELOAD" => {
                            can_block_reload = value.as_option_unquoted_str() == Some("YES")
                        }
                        _ => (),
                    }
                }
                Ok(Tag::ServerControl(ServerControl {
                    can_skip_until,
                    can_skip_dateranges,
                    hold_back,
                    part_hold_back,
                    can_block_reload,
                }))
            }
            "INF" => match tag.value {
                ParsedTagValue::DecimalInteger(d) => Ok(Tag::Inf(Inf {
                    duration: d as f64,
                    title: "",
                })),
                ParsedTagValue::DecimalFloatingPointWithOptionalTitle(duration, title) => {
                    Ok(Tag::Inf(Inf { duration, title }))
                }
                _ => Self::unexpected_value_type(),
            },
            "-X-BYTERANGE" => match tag.value {
                ParsedTagValue::DecimalInteger(length) => Ok(Tag::Byterange(Byterange {
                    length,
                    offset: None,
                })),
                ParsedTagValue::DecimalIntegerRange(length, offset) => {
                    Ok(Tag::Byterange(Byterange {
                        length,
                        offset: Some(offset),
                    }))
                }
                _ => Self::unexpected_value_type(),
            },
            "-X-DISCONTINUITY" => Ok(Tag::Discontinuity(Discontinuity)),
            "-X-KEY" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::UnquotedString(method)) =
                    attribute_list.remove("METHOD")
                else {
                    return Self::missing_required_attribute();
                };
                let uri = match attribute_list.remove("URI") {
                    Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
                    _ => None,
                };
                let iv = match attribute_list.remove("IV") {
                    Some(ParsedAttributeValue::UnquotedString(iv)) => Some(iv),
                    _ => None,
                };
                let keyformat = match attribute_list.remove("KEYFORMAT") {
                    Some(ParsedAttributeValue::QuotedString(keyformat)) => Some(keyformat),
                    _ => None,
                };
                let keyformatversions = match attribute_list.remove("KEYFORMATVERSIONS") {
                    Some(ParsedAttributeValue::QuotedString(keyformatversions)) => {
                        Some(keyformatversions)
                    }
                    _ => None,
                };
                Ok(Tag::Key(Key {
                    method,
                    uri,
                    iv,
                    keyformat,
                    keyformatversions,
                }))
            }
            "-X-MAP" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI")
                else {
                    return Self::missing_required_attribute();
                };
                let byterange = 'byterange_match: {
                    match attribute_list.remove("BYTERANGE") {
                        Some(ParsedAttributeValue::QuotedString(byterange_str)) => {
                            let mut parts = byterange_str.split('@');
                            let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                                break 'byterange_match None;
                            };
                            let Some(Ok(offset)) = parts.next().map(str::parse::<u64>) else {
                                break 'byterange_match None;
                            };
                            if parts.next().is_some() {
                                break 'byterange_match None;
                            }
                            Some(MapByterange { length, offset })
                        }
                        _ => None,
                    }
                };
                Ok(Tag::Map(Map { uri, byterange }))
            }
            "-X-PROGRAM-DATE-TIME" => {
                let ParsedTagValue::DateTimeMsec(date_time) = tag.value else {
                    return Self::unexpected_value_type();
                };
                Ok(Tag::ProgramDateTime(ProgramDateTime(date_time)))
            }
            "-X-GAP" => Ok(Tag::Gap(Gap)),
            "-X-BITRATE" => todo!(),
            "-X-PART" => todo!(),
            "-X-DATERANGE" => todo!(),
            "-X-SKIP" => todo!(),
            "-X-PRELOAD-HINT" => todo!(),
            "-X-RENDITION-REPORT" => todo!(),
            "-X-MEDIA" => todo!(),
            "-X-STREAM-INF" => todo!(),
            "-X-I-FRAME-STREAM-INF" => todo!(),
            "-X-SESSION-DATA" => todo!(),
            "-X-SESSION-KEY" => todo!(),
            "-X-CONTENT-STEERING" => todo!(),
            _ => Self::unknown_name(),
        }
    }
}

impl IsKnownName for Tag<'_> {
    fn is_known_name(name: &str) -> bool {
        [
            "M3U",
            "-X-VERSION",
            "-X-INDEPENDENT-SEGMENTS",
            "-X-START",
            "-X-DEFINE",
            "-X-TARGETDURATION",
            "-X-MEDIA-SEQUENCE",
            "-X-DISCONTINUITY-SEQUENCE",
            "-X-ENDLIST",
            "-X-PLAYLIST-TYPE",
            "-X-I-FRAMES-ONLY",
            "-X-PART-INF",
            "-X-SERVER-CONTROL",
            "INF",
            "-X-BYTERANGE",
            "-X-DISCONTINUITY",
            "-X-KEY",
            "-X-MAP",
            "-X-PROGRAM-DATE-TIME",
            "-X-GAP",
            "-X-BITRATE",
            "-X-PART",
            "-X-DATERANGE",
            "-X-SKIP",
            "-X-PRELOAD-HINT",
            "-X-RENDITION-REPORT",
            "-X-MEDIA",
            "-X-STREAM-INF",
            "-X-I-FRAME-STREAM-INF",
            "-X-SESSION-DATA",
            "-X-SESSION-KEY",
            "-X-CONTENT-STEERING",
        ]
        .contains(&name)
    }
}

impl Tag<'_> {
    fn unknown_name() -> Result<Self, &'static str> {
        Err("Unkown tag name.")
    }

    fn unexpected_value_type() -> Result<Self, &'static str> {
        Err("Unexpected parsed value type.")
    }

    fn missing_required_attribute() -> Result<Self, &'static str> {
        Err("Missing required attribute.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::DateTimeTimezoneOffset;
    use pretty_assertions::assert_eq;

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
                keyformat: Some("com.apple.streamingkeydelivery"),
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
                keyformat: None,
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
}

// TODO - parse the following:
// "-X-BITRATE",
// "-X-PART",
// "-X-DATERANGE",
// "-X-SKIP",
// "-X-PRELOAD-HINT",
// "-X-RENDITION-REPORT",
// "-X-MEDIA",
// "-X-STREAM-INF",
// "-X-I-FRAME-STREAM-INF",
// "-X-SESSION-DATA",
// "-X-SESSION-KEY",
// "-X-CONTENT-STEERING",
