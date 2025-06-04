use crate::tag::known::{IsKnownName, ParsedTag};

// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17
// 4.4.  Playlist Tags
//     4.4.1.  Basic Tags
//         4.4.1.1.  EXTM3U
#[derive(Debug, PartialEq)]
pub struct M3u;
//         4.4.1.2.  EXT-X-VERSION
#[derive(Debug, PartialEq)]
pub struct Version;
//     4.4.2.  Media or Multivariant Playlist Tags
//         4.4.2.1.  EXT-X-INDEPENDENT-SEGMENTS
#[derive(Debug, PartialEq)]
pub struct IndependentSegments;
//         4.4.2.2.  EXT-X-START
#[derive(Debug, PartialEq)]
pub struct Start;
//         4.4.2.3.  EXT-X-DEFINE
#[derive(Debug, PartialEq)]
pub struct Define;
//     4.4.3.  Media Playlist Tags
//         4.4.3.1.  EXT-X-TARGETDURATION
#[derive(Debug, PartialEq)]
pub struct Targetduration;
//         4.4.3.2.  EXT-X-MEDIA-SEQUENCE
#[derive(Debug, PartialEq)]
pub struct MediaSequence;
//         4.4.3.3.  EXT-X-DISCONTINUITY-SEQUENCE
#[derive(Debug, PartialEq)]
pub struct DiscontinuitySequence;
//         4.4.3.4.  EXT-X-ENDLIST
#[derive(Debug, PartialEq)]
pub struct Endlist;
//         4.4.3.5.  EXT-X-PLAYLIST-TYPE
#[derive(Debug, PartialEq)]
pub struct PlaylistType;
//         4.4.3.6.  EXT-X-I-FRAMES-ONLY
#[derive(Debug, PartialEq)]
pub struct IFramesOnly;
//         4.4.3.7.  EXT-X-PART-INF
#[derive(Debug, PartialEq)]
pub struct PartInf;
//         4.4.3.8.  EXT-X-SERVER-CONTROL
#[derive(Debug, PartialEq)]
pub struct ServerControl;
//     4.4.4.  Media Segment Tags
//         4.4.4.1.  EXTINF
#[derive(Debug, PartialEq)]
pub struct Inf<'a> {
    pub duration: f64,
    pub title: &'a str,
}
//         4.4.4.2.  EXT-X-BYTERANGE
#[derive(Debug, PartialEq)]
pub struct Byterange;
//         4.4.4.3.  EXT-X-DISCONTINUITY
#[derive(Debug, PartialEq)]
pub struct Discontinuity;
//         4.4.4.4.  EXT-X-KEY
#[derive(Debug, PartialEq)]
pub struct Key;
//         4.4.4.5.  EXT-X-MAP
#[derive(Debug, PartialEq)]
pub struct Map;
//         4.4.4.6.  EXT-X-PROGRAM-DATE-TIME
#[derive(Debug, PartialEq)]
pub struct ProgramDateTime;
//         4.4.4.7.  EXT-X-GAP
#[derive(Debug, PartialEq)]
pub struct Gap;
//         4.4.4.8.  EXT-X-BITRATE
#[derive(Debug, PartialEq)]
pub struct Bitrate;
//         4.4.4.9.  EXT-X-PART
#[derive(Debug, PartialEq)]
pub struct Part;
//     4.4.5.  Media Metadata Tags
//         4.4.5.1.  EXT-X-DATERANGE
#[derive(Debug, PartialEq)]
pub struct Daterange;
//         4.4.5.2.  EXT-X-SKIP
#[derive(Debug, PartialEq)]
pub struct Skip;
//         4.4.5.3.  EXT-X-PRELOAD-HINT
#[derive(Debug, PartialEq)]
pub struct PreloadHint;
//         4.4.5.4.  EXT-X-RENDITION-REPORT
#[derive(Debug, PartialEq)]
pub struct RenditionReport;
//     4.4.6.  Multivariant Playlist Tags
//         4.4.6.1.  EXT-X-MEDIA
#[derive(Debug, PartialEq)]
pub struct Media;
//         4.4.6.2.  EXT-X-STREAM-INF
#[derive(Debug, PartialEq)]
pub struct StreamInf;
//         4.4.6.3.  EXT-X-I-FRAME-STREAM-INF
#[derive(Debug, PartialEq)]
pub struct IFrameStreamInf;
//         4.4.6.4.  EXT-X-SESSION-DATA
#[derive(Debug, PartialEq)]
pub struct SessionData;
//         4.4.6.5.  EXT-X-SESSION-KEY
#[derive(Debug, PartialEq)]
pub struct SessionKey;
//         4.4.6.6.  EXT-X-CONTENT-STEERING
#[derive(Debug, PartialEq)]
pub struct ContentSteering;

#[derive(Debug, PartialEq)]
pub enum Tag<'a> {
    M3u(M3u),
    Version(Version),
    IndependentSegments(IndependentSegments),
    Start(Start),
    Define(Define),
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
    Key(Key),
    Map(Map),
    ProgramDateTime(ProgramDateTime),
    Gap(Gap),
    Bitrate(Bitrate),
    Part(Part),
    Daterange(Daterange),
    Skip(Skip),
    PreloadHint(PreloadHint),
    RenditionReport(RenditionReport),
    Media(Media),
    StreamInf(StreamInf),
    IFrameStreamInf(IFrameStreamInf),
    SessionData(SessionData),
    SessionKey(SessionKey),
    ContentSteering(ContentSteering),
}

impl<'a> TryFrom<ParsedTag<'a>> for Tag<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        match tag.name {
            "M3U" => Ok(Self::M3u(M3u)),
            "-X-VERSION" => todo!(),
            "-X-INDEPENDENT-SEGMENTS" => todo!(),
            "-X-START" => todo!(),
            "-X-DEFINE" => todo!(),
            "-X-TARGETDURATION" => todo!(),
            "-X-MEDIA-SEQUENCE" => todo!(),
            "-X-DISCONTINUITY-SEQUENCE" => todo!(),
            "-X-ENDLIST" => todo!(),
            "-X-PLAYLIST-TYPE" => todo!(),
            "-X-I-FRAMES-ONLY" => todo!(),
            "-X-PART-INF" => todo!(),
            "-X-SERVER-CONTROL" => todo!(),
            "INF" => todo!(),
            "-X-BYTERANGE" => todo!(),
            "-X-DISCONTINUITY" => todo!(),
            "-X-KEY" => todo!(),
            "-X-MAP" => todo!(),
            "-X-PROGRAM-DATE-TIME" => todo!(),
            "-X-GAP" => todo!(),
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
            _ => Err(Self::unknown_name()),
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
    pub fn unknown_name() -> &'static str {
        "Unkown tag name."
    }
}
