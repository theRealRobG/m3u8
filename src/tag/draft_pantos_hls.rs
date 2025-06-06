use crate::{
    date::{self, DateTime},
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
    pub frame_rate: Option<f64>,
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
            "-X-BITRATE" => {
                let ParsedTagValue::DecimalInteger(rate) = tag.value else {
                    return Self::unexpected_value_type();
                };
                Ok(Tag::Bitrate(Bitrate(rate)))
            }
            "-X-PART" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI")
                else {
                    return Self::missing_required_attribute();
                };
                let Some(duration) = (match attribute_list.remove("DURATION") {
                    Some(a) => a.as_option_f64(),
                    _ => None,
                }) else {
                    return Self::missing_required_attribute();
                };
                let independent = match attribute_list.remove("INDEPENDENT") {
                    Some(ParsedAttributeValue::UnquotedString("YES")) => true,
                    _ => false,
                };
                let byterange = 'byterange_match: {
                    match attribute_list.remove("BYTERANGE") {
                        Some(ParsedAttributeValue::QuotedString(range)) => {
                            let mut parts = range.split('@');
                            let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                                break 'byterange_match None;
                            };
                            let offset = match parts.next().map(str::parse::<u64>) {
                                Some(Ok(d)) => Some(d),
                                None => None,
                                Some(Err(_)) => break 'byterange_match None,
                            };
                            if parts.next().is_some() {
                                break 'byterange_match None;
                            }
                            Some(PartByterange { length, offset })
                        }
                        _ => None,
                    }
                };
                let gap = match attribute_list.remove("GAP") {
                    Some(ParsedAttributeValue::UnquotedString("YES")) => true,
                    _ => false,
                };
                Ok(Tag::Part(Part {
                    uri,
                    duration,
                    independent,
                    byterange,
                    gap,
                }))
            }
            "-X-DATERANGE" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::QuotedString(id)) = attribute_list.remove("ID")
                else {
                    return Self::missing_required_attribute();
                };
                let class = match attribute_list.remove("CLASS") {
                    Some(ParsedAttributeValue::QuotedString(class)) => Some(class),
                    _ => None,
                };
                let Some(start_date) = (match attribute_list.remove("START-DATE") {
                    Some(ParsedAttributeValue::QuotedString(date_str)) => {
                        match date::parse(date_str) {
                            Ok((_, date_time)) => Some(date_time),
                            Err(_) => None,
                        }
                    }
                    _ => None,
                }) else {
                    return Self::missing_required_attribute();
                };
                let cue = match attribute_list.remove("CUE") {
                    Some(ParsedAttributeValue::QuotedString(cue)) => Some(cue),
                    _ => None,
                };
                let end_date = match attribute_list.remove("END-DATE") {
                    Some(ParsedAttributeValue::QuotedString(date_str)) => {
                        match date::parse(date_str) {
                            Ok((_, date_time)) => Some(date_time),
                            Err(_) => None,
                        }
                    }
                    _ => None,
                };
                let duration = match attribute_list.remove("DURATION") {
                    Some(d) => d.as_option_f64(),
                    _ => None,
                };
                let planned_duration = match attribute_list.remove("PLANNED-DURATION") {
                    Some(d) => d.as_option_f64(),
                    _ => None,
                };
                // The specification indicates that the SCTE35-(CMD|OUT|IN) attributes are
                // represented as hexadecimal sequences. This implies that they should be parsed as
                // UnquotedString (given that section "4.2. Attribute Lists" indicates that a
                // "hexadecimal-sequence [is] an unquoted string of characters"); however, in
                // practice, I've found that some packagers have put this information in quoted
                // strings (containing the hexadecimal sequence), so I'll allow this parser to be
                // lenient on that requirement and accept both.
                let scte35_cmd = match attribute_list.remove("SCTE35-CMD") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let scte35_out = match attribute_list.remove("SCTE35-OUT") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let scte35_in = match attribute_list.remove("SCTE35-IN") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let end_on_next = match attribute_list.remove("END-ON-NEXT") {
                    Some(ParsedAttributeValue::UnquotedString("YES")) => true,
                    _ => false,
                };

                // Deal with client attributes last as I will drain the rest of the HashMap.
                let mut client_attributes = HashMap::new();
                for (key, value) in attribute_list.drain() {
                    if key.starts_with("X-") {
                        client_attributes.insert(key, value);
                    }
                }
                Ok(Tag::Daterange(Daterange {
                    id,
                    class,
                    start_date,
                    cue,
                    end_date,
                    duration,
                    planned_duration,
                    client_attributes,
                    scte35_cmd,
                    scte35_out,
                    scte35_in,
                    end_on_next,
                }))
            }
            "-X-SKIP" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::DecimalInteger(skipped_segments)) =
                    attribute_list.remove("SKIPPED-SEGMENTS")
                else {
                    return Self::missing_required_attribute();
                };
                let recently_removed_dateranges =
                    match attribute_list.remove("RECENTLY-REMOVED-DATERANGES") {
                        Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                        _ => None,
                    };
                Ok(Tag::Skip(Skip {
                    skipped_segments,
                    recently_removed_dateranges,
                }))
            }
            "-X-PRELOAD-HINT" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::UnquotedString(hint_type)) =
                    attribute_list.remove("TYPE")
                else {
                    return Self::missing_required_attribute();
                };
                let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI")
                else {
                    return Self::missing_required_attribute();
                };
                let byterange_start = match attribute_list.remove("BYTERANGE-START") {
                    Some(ParsedAttributeValue::DecimalInteger(start)) => start,
                    _ => 0,
                };
                let byterange_length = match attribute_list.remove("BYTERANGE-LENGTH") {
                    Some(ParsedAttributeValue::DecimalInteger(length)) => Some(length),
                    _ => None,
                };
                Ok(Tag::PreloadHint(PreloadHint {
                    hint_type,
                    uri,
                    byterange_start,
                    byterange_length,
                }))
            }
            "-X-RENDITION-REPORT" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI")
                else {
                    return Self::missing_required_attribute();
                };
                let Some(ParsedAttributeValue::DecimalInteger(last_msn)) =
                    attribute_list.remove("LAST-MSN")
                else {
                    return Self::missing_required_attribute();
                };
                let last_part = match attribute_list.remove("LAST-PART") {
                    Some(ParsedAttributeValue::DecimalInteger(part)) => Some(part),
                    _ => None,
                };
                Ok(Tag::RenditionReport(RenditionReport {
                    uri,
                    last_msn,
                    last_part,
                }))
            }
            "-X-MEDIA" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::UnquotedString(media_type)) =
                    attribute_list.remove("TYPE")
                else {
                    return Self::missing_required_attribute();
                };
                let uri = match attribute_list.remove("URI") {
                    Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
                    _ => None,
                };
                let Some(ParsedAttributeValue::QuotedString(group_id)) =
                    attribute_list.remove("GROUP-ID")
                else {
                    return Self::missing_required_attribute();
                };
                let language = match attribute_list.remove("LANGUAGE") {
                    Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
                    _ => None,
                };
                let assoc_language = match attribute_list.remove("ASSOC-LANGUAGE") {
                    Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
                    _ => None,
                };
                let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.remove("NAME")
                else {
                    return Self::missing_required_attribute();
                };
                let stable_rendition_id = match attribute_list.remove("STABLE-RENDITION-ID") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let default = match attribute_list.remove("DEFAULT") {
                    Some(ParsedAttributeValue::UnquotedString("YES")) => true,
                    _ => false,
                };
                let autoselect = match attribute_list.remove("AUTOSELECT") {
                    Some(ParsedAttributeValue::UnquotedString("YES")) => true,
                    _ => false,
                };
                let forced = match attribute_list.remove("FORCED") {
                    Some(ParsedAttributeValue::UnquotedString("YES")) => true,
                    _ => false,
                };
                let instream_id = match attribute_list.remove("INSTREAM-ID") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let bit_depth = match attribute_list.remove("BIT-DEPTH") {
                    Some(ParsedAttributeValue::DecimalInteger(d)) => Some(d),
                    _ => None,
                };
                let sample_rate = match attribute_list.remove("SAMPLE-RATE") {
                    Some(ParsedAttributeValue::DecimalInteger(rate)) => Some(rate),
                    _ => None,
                };
                let characteristics = match attribute_list.remove("CHARACTERISTICS") {
                    Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
                    _ => None,
                };
                let channels = match attribute_list.remove("CHANNELS") {
                    Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
                    _ => None,
                };
                Ok(Tag::Media(Media {
                    media_type,
                    uri,
                    group_id,
                    language,
                    assoc_language,
                    name,
                    stable_rendition_id,
                    default,
                    autoselect,
                    forced,
                    instream_id,
                    bit_depth,
                    sample_rate,
                    characteristics,
                    channels,
                }))
            }
            "-X-STREAM-INF" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::DecimalInteger(bandwidth)) =
                    attribute_list.remove("BANDWIDTH")
                else {
                    return Self::missing_required_attribute();
                };
                let average_bandwidth = match attribute_list.remove("AVERAGE-BANDWIDTH") {
                    Some(ParsedAttributeValue::DecimalInteger(b)) => Some(b),
                    _ => None,
                };
                let score = match attribute_list.remove("SCORE") {
                    Some(value) => value.as_option_f64(),
                    _ => None,
                };
                let codecs = match attribute_list.remove("CODECS") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let supplemental_codecs = match attribute_list.remove("SUPPLEMENTAL-CODECS") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let resolution = 'resolution_match: {
                    match attribute_list.remove("RESOLUTION") {
                        Some(ParsedAttributeValue::UnquotedString(r)) => {
                            let mut split = r.split('x');
                            let Some(Ok(width)) = split.next().map(str::parse::<u64>) else {
                                break 'resolution_match None;
                            };
                            let Some(Ok(height)) = split.next().map(str::parse::<u64>) else {
                                break 'resolution_match None;
                            };
                            if split.next().is_some() {
                                break 'resolution_match None;
                            };
                            Some(DecimalResolution { width, height })
                        }
                        _ => None,
                    }
                };
                let frame_rate = match attribute_list.remove("FRAME-RATE") {
                    Some(v) => v.as_option_f64(),
                    _ => None,
                };
                let hdcp_level = match attribute_list.remove("HDCP-LEVEL") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                    _ => None,
                };
                let allowed_cpc = match attribute_list.remove("ALLOWED-CPC") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let video_range = match attribute_list.remove("VIDEO-RANGE") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                    _ => None,
                };
                let req_video_layout = match attribute_list.remove("REQ-VIDEO-LAYOUT") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let stable_variant_id = match attribute_list.remove("STABLE-VARIANT-ID") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let audio = match attribute_list.remove("AUDIO") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let video = match attribute_list.remove("VIDEO") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let subtitles = match attribute_list.remove("SUBTITLES") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let closed_captions = match attribute_list.remove("CLOSED-CAPTIONS") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let pathway_id = match attribute_list.remove("PATHWAY-ID") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                Ok(Tag::StreamInf(StreamInf {
                    bandwidth,
                    average_bandwidth,
                    score,
                    codecs,
                    supplemental_codecs,
                    resolution,
                    frame_rate,
                    hdcp_level,
                    allowed_cpc,
                    video_range,
                    req_video_layout,
                    stable_variant_id,
                    audio,
                    video,
                    subtitles,
                    closed_captions,
                    pathway_id,
                }))
            }
            "-X-I-FRAME-STREAM-INF" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI")
                else {
                    return Self::missing_required_attribute();
                };
                let Some(ParsedAttributeValue::DecimalInteger(bandwidth)) =
                    attribute_list.remove("BANDWIDTH")
                else {
                    return Self::missing_required_attribute();
                };
                let average_bandwidth = match attribute_list.remove("AVERAGE-BANDWIDTH") {
                    Some(ParsedAttributeValue::DecimalInteger(b)) => Some(b),
                    _ => None,
                };
                let score = match attribute_list.remove("SCORE") {
                    Some(value) => value.as_option_f64(),
                    _ => None,
                };
                let codecs = match attribute_list.remove("CODECS") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let supplemental_codecs = match attribute_list.remove("SUPPLEMENTAL-CODECS") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let resolution = 'resolution_match: {
                    match attribute_list.remove("RESOLUTION") {
                        Some(ParsedAttributeValue::UnquotedString(r)) => {
                            let mut split = r.split('x');
                            let Some(Ok(width)) = split.next().map(str::parse::<u64>) else {
                                break 'resolution_match None;
                            };
                            let Some(Ok(height)) = split.next().map(str::parse::<u64>) else {
                                break 'resolution_match None;
                            };
                            if split.next().is_some() {
                                break 'resolution_match None;
                            };
                            Some(DecimalResolution { width, height })
                        }
                        _ => None,
                    }
                };
                let hdcp_level = match attribute_list.remove("HDCP-LEVEL") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                    _ => None,
                };
                let allowed_cpc = match attribute_list.remove("ALLOWED-CPC") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let video_range = match attribute_list.remove("VIDEO-RANGE") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                    _ => None,
                };
                let req_video_layout = match attribute_list.remove("REQ-VIDEO-LAYOUT") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let stable_variant_id = match attribute_list.remove("STABLE-VARIANT-ID") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let video = match attribute_list.remove("VIDEO") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let pathway_id = match attribute_list.remove("PATHWAY-ID") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                Ok(Tag::IFrameStreamInf(IFrameStreamInf {
                    uri,
                    bandwidth,
                    average_bandwidth,
                    score,
                    codecs,
                    supplemental_codecs,
                    resolution,
                    hdcp_level,
                    allowed_cpc,
                    video_range,
                    req_video_layout,
                    stable_variant_id,
                    video,
                    pathway_id,
                }))
            }
            "-X-SESSION-DATA" => {
                let ParsedTagValue::AttributeList(mut attribute_list) = tag.value else {
                    return Self::unexpected_value_type();
                };
                let Some(ParsedAttributeValue::QuotedString(data_id)) =
                    attribute_list.remove("DATA-ID")
                else {
                    return Self::missing_required_attribute();
                };
                let value = match attribute_list.remove("VALUE") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let uri = match attribute_list.remove("URI") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                let format = match attribute_list.remove("FORMAT") {
                    Some(ParsedAttributeValue::UnquotedString(s)) => s,
                    _ => "JSON",
                };
                let language = match attribute_list.remove("LANGUAGE") {
                    Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                    _ => None,
                };
                Ok(Tag::SessionData(SessionData {
                    data_id,
                    value,
                    uri,
                    format,
                    language,
                }))
            }
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
}

// TODO - test the following:
// "-X-SESSION-KEY",
// "-X-CONTENT-STEERING",
