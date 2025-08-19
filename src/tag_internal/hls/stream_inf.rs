use crate::{
    error::{ParseTagValueError, UnrecognizedEnumerationError, ValidationError},
    tag::{
        DecimalResolution, UnknownTag,
        hls::{EnumeratedString, EnumeratedStringList, LazyAttribute, into_inner_tag},
    },
    utils::AsStaticCow,
};
use memchr::{memchr, memmem};
use std::{borrow::Cow, fmt::Display, marker::PhantomData};

/// Corresponds to the `#EXT-X-STREAM-INF:HDCP-LEVEL` attribute.
///
/// See [`StreamInf`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HdcpLevel {
    /// Indicates that the content does not require output copy protection.
    None,
    /// Indicates that the Variant Stream could fail to play unless the output is protected by
    /// High-bandwidth Digital Content Protection (HDCP) Type 0 or equivalent.
    Type0,
    /// Indicates that the Variant Stream could fail to play unless the output is protected by HDCP
    /// Type 1 or equivalent.
    Type1,
}
impl<'a> TryFrom<&'a str> for HdcpLevel {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            NONE => Ok(Self::None),
            TYPE_0 => Ok(Self::Type0),
            TYPE_1 => Ok(Self::Type1),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for HdcpLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for HdcpLevel {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::None => Cow::Borrowed(NONE),
            Self::Type0 => Cow::Borrowed(TYPE_0),
            Self::Type1 => Cow::Borrowed(TYPE_1),
        }
    }
}
impl From<HdcpLevel> for Cow<'_, str> {
    fn from(value: HdcpLevel) -> Self {
        value.as_cow()
    }
}
impl From<HdcpLevel> for EnumeratedString<'_, HdcpLevel> {
    fn from(value: HdcpLevel) -> Self {
        Self::Known(value)
    }
}
const NONE: &str = "NONE";
const TYPE_0: &str = "TYPE-0";
const TYPE_1: &str = "TYPE-1";

/// Corresponds to the `#EXT-X-STREAM-INF:VIDEO-RANGE` attribute.
///
/// See [`StreamInf`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoRange {
    /// The video in the Variant Stream is encoded using one of the following reference
    /// opto-electronic transfer characteristic functions specified by the TransferCharacteristics
    /// code point: 1, 6, 13, 14, 15. Note that different TransferCharacteristics code points can
    /// use the same transfer function.
    Sdr,
    /// The video in the Variant Stream is encoded using a reference opto-electronic transfer
    /// characteristic function specified by the TransferCharacteristics code point 18, or consists
    /// of such video mixed with video qualifying as SDR (see above).
    Hlg,
    /// The video in the Variant Stream is encoded using a reference opto-electronic transfer
    /// characteristic function specified by the TransferCharacteristics code point 16, or consists
    /// of such video mixed with video qualifying as SDR or HLG (see above).
    Pq,
}
impl<'a> TryFrom<&'a str> for VideoRange {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            SDR => Ok(Self::Sdr),
            HLG => Ok(Self::Hlg),
            PQ => Ok(Self::Pq),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for VideoRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for VideoRange {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Sdr => Cow::Borrowed(SDR),
            Self::Hlg => Cow::Borrowed(HLG),
            Self::Pq => Cow::Borrowed(PQ),
        }
    }
}
impl From<VideoRange> for Cow<'_, str> {
    fn from(value: VideoRange) -> Self {
        value.as_cow()
    }
}
impl From<VideoRange> for EnumeratedString<'_, VideoRange> {
    fn from(value: VideoRange) -> Self {
        Self::Known(value)
    }
}
const SDR: &str = "SDR";
const HLG: &str = "HLG";
const PQ: &str = "PQ";

/// Corresponds to the "Video Channel Specifier" within the `#EXT-X-STREAM-INF:REQ-VIDEO-LAYOUT`
/// attribute.
///
/// See [`StreamInf`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoChannelSpecifier {
    /// Indicates that both left and right eye images are present (stereoscopic).
    Stereo,
    /// Indicates that a single image is present (monoscopic).
    Mono,
}
impl<'a> TryFrom<&'a str> for VideoChannelSpecifier {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            CH_STEREO => Ok(Self::Stereo),
            CH_MONO => Ok(Self::Mono),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for VideoChannelSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for VideoChannelSpecifier {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Stereo => Cow::Borrowed(CH_STEREO),
            Self::Mono => Cow::Borrowed(CH_MONO),
        }
    }
}
impl From<VideoChannelSpecifier> for Cow<'_, str> {
    fn from(value: VideoChannelSpecifier) -> Self {
        value.as_cow()
    }
}
impl From<VideoChannelSpecifier> for EnumeratedString<'_, VideoChannelSpecifier> {
    fn from(value: VideoChannelSpecifier) -> Self {
        Self::Known(value)
    }
}
const CH_STEREO: &str = "CH-STEREO";
const CH_MONO: &str = "CH-MONO";

/// Corresponds to the "Video Projection Specifier" within the `#EXT-X-STREAM-INF:REQ-VIDEO-LAYOUT`
/// attribute.
///
/// See [`StreamInf`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoProjectionSpecifier {
    /// Indicates that there is no projection.
    Rectilinear,
    /// Indicates a 360 degree spherical projection.
    Equirectangular,
    /// Indicates a 180 degree spherical projection.
    HalfEquirectangular,
    /// Indicates that the image is a parametric spherical projection.
    ParametricImmersive,
}
impl<'a> TryFrom<&'a str> for VideoProjectionSpecifier {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            PROJ_RECT => Ok(Self::Rectilinear),
            PROJ_EQUI => Ok(Self::Equirectangular),
            PROJ_HEQU => Ok(Self::HalfEquirectangular),
            PROJ_PRIM => Ok(Self::ParametricImmersive),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for VideoProjectionSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for VideoProjectionSpecifier {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Rectilinear => Cow::Borrowed(PROJ_RECT),
            Self::Equirectangular => Cow::Borrowed(PROJ_EQUI),
            Self::HalfEquirectangular => Cow::Borrowed(PROJ_HEQU),
            Self::ParametricImmersive => Cow::Borrowed(PROJ_PRIM),
        }
    }
}
impl From<VideoProjectionSpecifier> for Cow<'_, str> {
    fn from(value: VideoProjectionSpecifier) -> Self {
        value.as_cow()
    }
}
impl From<VideoProjectionSpecifier> for EnumeratedString<'_, VideoProjectionSpecifier> {
    fn from(value: VideoProjectionSpecifier) -> Self {
        Self::Known(value)
    }
}
const PROJ_RECT: &str = "PROJ-RECT";
const PROJ_EQUI: &str = "PROJ-EQUI";
const PROJ_HEQU: &str = "PROJ-HEQU";
const PROJ_PRIM: &str = "PROJ-PRIM";

/// Corresponds to the `#EXT-X-STREAM-INF:REQ-VIDEO-LAYOUT` attribute.
///
/// See [`StreamInf`] for a link to the HLS documentation for this attribute.
///
/// The format described in HLS is an unordered, slash separated list of specifiers, where each
/// specifier value is an enumerated string. Due to the specifiers being unordered, the values
/// within a specifier must all share a common prefix. As of draft 18 there are two specifiers
/// defined:
/// * Channel, prefixed with `CH-`
/// * Projection, prefixed with `PROJ-`
///
/// `VideoLayout` abstracts these nuances of syntax away and provides typed access to the values.
/// To support forwards compatibility it also exposes any values with unknown prefixes via
/// [`Self::unknown_entries`]. And at any stage the user has an escape hatch to the inner `Cow<str>`
/// via [`Self::as_ref`].
#[derive(Debug, Clone, PartialEq)]
pub struct VideoLayout<'a> {
    inner: Cow<'a, str>,
}
impl<'a> VideoLayout<'a> {
    /// Construct a new `VideoLayout`.
    ///
    /// Note that `VideoChannelSpecifier` and `VideoProjectionSpecifier` can be used directly here.
    /// For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier, VideoProjectionSpecifier,
    /// # EnumeratedStringList};
    /// let video_layout = VideoLayout::new(
    ///     EnumeratedStringList::from([
    ///         VideoChannelSpecifier::Stereo, VideoChannelSpecifier::Mono
    ///     ]),
    ///     EnumeratedStringList::from([VideoProjectionSpecifier::ParametricImmersive]),
    /// );
    /// assert_eq!("CH-STEREO,CH-MONO/PROJ-PRIM", video_layout.as_ref());
    /// ```
    /// Since `&str` implements `Into<EnumeratedStringList>` we can also use string slice directly,
    /// but care should be taken to follow the correct format:
    /// ```
    /// # use quick_m3u8::tag::hls::VideoLayout;
    /// let video_layout = VideoLayout::new("CH-STEREO,CH-MONO", "PROJ-PRIM");
    /// assert_eq!("CH-STEREO,CH-MONO/PROJ-PRIM", video_layout.as_ref());
    /// ```
    /// The `From<&str>` implementation ensures that the order of specifiers does not impact the
    /// parsed value:
    /// ```
    /// # use quick_m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier, VideoProjectionSpecifier};
    /// let layout_1 = VideoLayout::from("CH-STEREO,CH-MONO/PROJ-PRIM");
    /// let layout_2 = VideoLayout::from("PROJ-PRIM/CH-STEREO,CH-MONO");
    /// assert_eq!(layout_1.channels(), layout_2.channels());
    /// assert_eq!(layout_1.projection(), layout_2.projection());
    /// assert_eq!(2, layout_1.channels().iter().count());
    /// assert!(layout_1.channels().contains(VideoChannelSpecifier::Stereo));
    /// assert!(layout_1.channels().contains(VideoChannelSpecifier::Mono));
    /// assert_eq!(1, layout_1.projection().iter().count());
    /// assert!(layout_1.projection().contains(VideoProjectionSpecifier::ParametricImmersive));
    /// ```
    pub fn new(
        channels: impl Into<EnumeratedStringList<'a, VideoChannelSpecifier>>,
        projection: impl Into<EnumeratedStringList<'a, VideoProjectionSpecifier>>,
    ) -> Self {
        let channels = channels.into();
        let projection = projection.into();
        let projection_empty = projection.is_empty();
        let channels_empty = channels.is_empty();
        let inner = match (channels_empty, projection_empty) {
            (true, true) => Cow::Borrowed(""),
            (true, false) => Cow::Owned(format!("{projection}")),
            (false, true) => Cow::Owned(format!("{channels}")),
            (false, false) => Cow::Owned(format!("{channels}/{projection}")),
        };
        Self { inner }
    }
}
impl VideoLayout<'_> {
    /// Defines the video channels.
    ///
    /// This collects all specifiers who's first element is prefixed with `CH`. The HLS
    /// specification stipulates that specifier values must all share the same prefix so checking
    /// the first value in the slash separated specifier is deemed enough. The
    /// [`EnumeratedStringList`] ensures that even invalid mixed members will be captured in the
    /// list, so information will not be lost.
    ///
    /// Example:
    /// ```
    /// # use quick_m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier};
    /// let video_layout = VideoLayout::new("CH-STEREO,CH-MONO", "");
    /// assert_eq!(2, video_layout.channels().iter().count());
    /// assert!(video_layout.channels().contains(VideoChannelSpecifier::Stereo));
    /// assert!(video_layout.channels().contains(VideoChannelSpecifier::Mono));
    /// assert_eq!("CH-STEREO,CH-MONO", video_layout.as_ref());
    /// ```
    /// Example with unknown specifier:
    /// ```
    /// # use quick_m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier};
    /// let video_layout = VideoLayout::new("CH-3D", "");
    /// assert_eq!(1, video_layout.channels().iter().count());
    /// assert!(video_layout.channels().contains("CH-3D"));
    /// assert_eq!("CH-3D", video_layout.as_ref());
    /// ```
    pub fn channels(&self) -> EnumeratedStringList<'_, VideoChannelSpecifier> {
        let split = self.inner.split('/');
        for entries in split {
            if entries.starts_with("CH") {
                return EnumeratedStringList::from(entries);
            }
        }
        EnumeratedStringList::from("")
    }
    /// Defines how a two-dimensional rectangular image must be transformed in order to display it
    /// faithfully to a viewer.
    ///
    /// This collects all specifiers who's first element is prefixed with `PROJ`. The HLS
    /// specification stipulates that specifier values must all share the same prefix so checking
    /// the first value in the slash separated specifier is deemed enough. The
    /// [`EnumeratedStringList`] ensures that even invalid mixed members will be captured in the
    /// list, so information will not be lost.
    ///
    /// Example:
    /// ```
    /// # use quick_m3u8::tag::hls::{VideoLayout, VideoProjectionSpecifier};
    /// let video_layout = VideoLayout::new("", "PROJ-EQUI,PROJ-HEQU");
    /// assert_eq!(2, video_layout.projection().iter().count());
    /// assert!(video_layout.projection().contains(VideoProjectionSpecifier::Equirectangular));
    /// assert!(video_layout.projection().contains(VideoProjectionSpecifier::HalfEquirectangular));
    /// assert_eq!("PROJ-EQUI,PROJ-HEQU", video_layout.as_ref());
    /// ```
    /// Example with unknown specifier:
    /// ```
    /// # use quick_m3u8::tag::hls::{VideoLayout};
    /// let video_layout = VideoLayout::new("", "PROJ-360");
    /// assert_eq!(1, video_layout.projection().iter().count());
    /// assert!(video_layout.projection().contains("PROJ-360"));
    /// assert_eq!("PROJ-360", video_layout.as_ref());
    /// ```
    pub fn projection(&self) -> EnumeratedStringList<'_, VideoProjectionSpecifier> {
        let split = self.inner.split('/');
        for entries in split {
            if entries.starts_with("PROJ") {
                return EnumeratedStringList::from(entries);
            }
        }
        EnumeratedStringList::from("")
    }
    /// At the time of writing the HLS specification only defined 2 entries (described here via
    /// [`Self::channels`] and [`Self::projection`]). In case more entries are added later, this
    /// method will expose those as a split on `'/'`, filtered to remove the `channels` and
    /// `projection` parts.
    ///
    /// For example:
    /// ```
    /// # use quick_m3u8::tag::hls::VideoLayout;
    /// let video_layout = VideoLayout::from("CH-STEREO/NEURAL-INJECT/PROJ-PRIM");
    /// let mut unknown = video_layout.unknown_entries();
    /// assert_eq!(Some("NEURAL-INJECT"), unknown.next());
    /// ```
    pub fn unknown_entries(&self) -> impl Iterator<Item = &str> {
        self.inner
            .split('/')
            .filter(|entries| !entries.starts_with("CH") && !entries.starts_with("PROJ"))
    }
}
impl<'a> From<&'a str> for VideoLayout<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            inner: Cow::Borrowed(value),
        }
    }
}
impl Display for VideoLayout<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
impl AsRef<str> for VideoLayout<'_> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl<'a> From<VideoLayout<'a>> for Cow<'a, str> {
    fn from(value: VideoLayout<'a>) -> Self {
        value.inner
    }
}

/// Corresponds to `#EXT-X-STREAM-INF:ALLOWED-CPC` values defined for FairPlay streaming.
///
/// See the [Apple HLS authoring specification for Apple devices] for more information.
///
/// [Apple HLS authoring specification for Apple devices]: https://developer.apple.com/documentation/http-live-streaming/hls-authoring-specification-for-apple-devices-appendixes#ALLOWED-CPC-values-for-FairPlay-Streaming
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FairPlayCpcLabel {
    /// Any Apple platform that supports FairPlay Streaming.
    AppleBaseline,
    /// Any Apple platform that supports FairPlay Streaming and guarantees enhanced content
    /// protection robustness (sufficient for studio 4K/HDR playback).
    AppleMain,
    /// Any non-Apple platform that supports FairPlay Streaming. For example, any AirPlay 2-enabled
    /// smart TV.
    Baseline,
    /// Any non-Apple platform that supports FairPlay Streaming and guarantees enhanced content
    /// protection robustness (sufficient for studio 4K/HDR playback).
    Main,
}
impl<'a> TryFrom<&'a str> for FairPlayCpcLabel {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            APPLE_BASELINE => Ok(Self::AppleBaseline),
            APPLE_MAIN => Ok(Self::AppleMain),
            BASELINE => Ok(Self::Baseline),
            MAIN => Ok(Self::Main),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for FairPlayCpcLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for FairPlayCpcLabel {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::AppleBaseline => Cow::Borrowed(APPLE_BASELINE),
            Self::AppleMain => Cow::Borrowed(APPLE_MAIN),
            Self::Baseline => Cow::Borrowed(BASELINE),
            Self::Main => Cow::Borrowed(MAIN),
        }
    }
}
impl From<FairPlayCpcLabel> for Cow<'_, str> {
    fn from(value: FairPlayCpcLabel) -> Self {
        value.as_cow()
    }
}
impl From<FairPlayCpcLabel> for EnumeratedString<'_, FairPlayCpcLabel> {
    fn from(value: FairPlayCpcLabel) -> Self {
        Self::Known(value)
    }
}
const APPLE_BASELINE: &str = "AppleBaseline";
const APPLE_MAIN: &str = "AppleMain";
const BASELINE: &str = "Baseline";
const MAIN: &str = "Main";

/// Corresponds to the `#EXT-X-STREAM-INF:ALLOWED-CPC` attribute.
///
/// See [`StreamInf`] for a link to the HLS documentation for this attribute.
///
/// The format described in HLS is a comma-separated list of `KEYFORMAT` to allowed Content
/// Protection Configuration (CPC) labels (where `KEYFORMAT` is separated from the labels by `:`).
/// Apple only defines the FairPlay keyformat which is identified by the string
/// `"com.apple.streamingkeydelivery"`. That being said, `KEYFORMAT` (as defined within the text for
/// the `EXT-X-KEY` tag) does not stipulate any limitations on what characters can be included in
/// the identifier, therefore we cannot make a generic list of mappings (since the `KEYFORMAT`
/// string could contain `,` or `/`, or `:`, indeed, the Widevine `KEYFORMAT` includes `:` as it is
/// `urn:uuid:edef8ba9-79d6-4ace-a3c8-27dcd51d21ed`). Due to this, the abstraction we offer requires
/// the user to test against a known format they are checking against.
///
/// It is always possible to return back to working with strings directly by using the
/// [`Self::as_ref`] method.
#[derive(Debug, PartialEq, Clone)]
pub struct AllowedCpc<'a> {
    inner: Cow<'a, str>,
}
impl<'a> AllowedCpc<'a> {
    /// The `KEYFORMAT` identifier for FairPlay DRM.
    pub const fn fair_play_keyformat() -> &'static str {
        "com.apple.streamingkeydelivery"
    }

    /// Provides an iterator over the `ALLOWED-CPC` values present for FairPlay streaming.
    ///
    /// For example
    /// ```
    /// # use quick_m3u8::tag::hls::{AllowedCpc, FairPlayCpcLabel, EnumeratedString};
    /// let allowed_cpc = AllowedCpc::from("com.apple.streamingkeydelivery:AppleMain/Main");
    /// let mut fair_play_allowed = allowed_cpc.allowed_cpc_for_fair_play();
    /// assert_eq!(
    ///     Some(EnumeratedString::Known(FairPlayCpcLabel::AppleMain)),
    ///     fair_play_allowed.next()
    /// );
    /// assert_eq!(
    ///     Some(EnumeratedString::Known(FairPlayCpcLabel::Main)),
    ///     fair_play_allowed.next()
    /// );
    /// assert_eq!(None, fair_play_allowed.next());
    /// ```
    pub fn allowed_cpc_for_fair_play(
        &self,
    ) -> impl Iterator<Item = EnumeratedString<'_, FairPlayCpcLabel>> {
        self.allowed_cpc_for_keyformat(Self::fair_play_keyformat())
            .map(EnumeratedString::from)
    }

    /// Inserts the provied FairPlay CPC label for the FairPlay `KEYFORMAT` entry.
    ///
    /// If there is no FairPlay `KEYFORMAT` entry, then this creates it with the provided value.
    ///
    /// For example, if entry exists:
    /// ```
    /// # use quick_m3u8::tag::hls::{AllowedCpc, FairPlayCpcLabel, EnumeratedString};
    /// let mut allowed_cpc = AllowedCpc::from("com.apple.streamingkeydelivery:AppleMain");
    /// allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::Main);
    /// assert_eq!("com.apple.streamingkeydelivery:AppleMain/Main", allowed_cpc.as_ref());
    /// ```
    /// For example, if entry does not exist:
    /// ```
    /// # use quick_m3u8::tag::hls::{AllowedCpc, FairPlayCpcLabel, EnumeratedString};
    /// let mut allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV/PC");
    /// allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::Main);
    /// assert_eq!(
    ///     "com.example.drm1:SMART-TV/PC,com.apple.streamingkeydelivery:Main",
    ///     allowed_cpc.as_ref()
    /// );
    /// ```
    ///
    /// The value returns true if the insert was successful, and false otherwise.
    /// ```
    /// # use quick_m3u8::tag::hls::{AllowedCpc, FairPlayCpcLabel, EnumeratedString};
    /// let mut allowed_cpc = AllowedCpc::from("com.apple.streamingkeydelivery:AppleMain");
    /// assert_eq!(
    ///     false,
    ///     allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::AppleMain),
    ///     "AppleMain already exists and so the insert will return false"
    /// );
    /// assert_eq!(
    ///     true,
    ///     allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::Main),
    ///     "Main does not exist already and so the insert will return true"
    /// );
    /// ```
    pub fn insert_cpc_for_fair_play(
        &mut self,
        cpc_label: impl Into<EnumeratedString<'a, FairPlayCpcLabel>>,
    ) -> bool {
        self.insert_cpc_for_keyformat(Self::fair_play_keyformat(), cpc_label.into().as_cow())
    }

    /// Removes the provied FairPlay CPC label from the FairPlay `KEYFORMAT` entry.
    ///
    /// If the label is the last for the FairPlay `KEYFORMAT` entry, then this removes the entry.
    ///
    /// For example, if not last label:
    /// ```
    /// # use quick_m3u8::tag::hls::{AllowedCpc, FairPlayCpcLabel, EnumeratedString};
    /// let mut allowed_cpc = AllowedCpc::from("com.apple.streamingkeydelivery:AppleMain/Main");
    /// allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::Main);
    /// assert_eq!("com.apple.streamingkeydelivery:AppleMain", allowed_cpc.as_ref());
    /// ```
    /// For example, if label is last:
    /// ```
    /// # use quick_m3u8::tag::hls::{AllowedCpc, FairPlayCpcLabel, EnumeratedString};
    /// let mut allowed_cpc = AllowedCpc::from("com.apple.streamingkeydelivery:AppleMain");
    /// allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::AppleMain);
    /// assert_eq!("", allowed_cpc.as_ref());
    /// ```
    ///
    /// The value returns true if the remove was successful, and false otherwise.
    /// ```
    /// # use quick_m3u8::tag::hls::{AllowedCpc, FairPlayCpcLabel, EnumeratedString};
    /// let mut allowed_cpc = AllowedCpc::from("com.apple.streamingkeydelivery:AppleMain");
    /// assert_eq!(
    ///     false,
    ///     allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::Main),
    ///     "Main is not in the list of labels and so the remove will return false"
    /// );
    /// assert_eq!(
    ///     true,
    ///     allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::AppleMain),
    ///     "AppleMain is in the list of labels and so the remove will return true"
    /// );
    /// ```
    pub fn remove_cpc_for_fair_play(
        &mut self,
        cpc_label: impl Into<EnumeratedString<'a, FairPlayCpcLabel>>,
    ) -> bool {
        self.remove_cpc_for_keyformat(Self::fair_play_keyformat(), cpc_label.into().as_cow())
    }

    /// Indicates whether the list of `KEYFORMAT` mappings is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Provides an iterator over the `ALLOWED-CPC` values present for the given `KEYFORMAT`.
    ///
    /// For example
    /// ```
    /// # use quick_m3u8::tag::hls::AllowedCpc;
    /// let allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV/PC");
    /// let mut drm1_allowed = allowed_cpc.allowed_cpc_for_keyformat("com.example.drm1");
    /// assert_eq!(Some("SMART-TV"), drm1_allowed.next());
    /// assert_eq!(Some("PC"), drm1_allowed.next());
    /// assert_eq!(None, drm1_allowed.next());
    /// ```
    pub fn allowed_cpc_for_keyformat(
        &self,
        keyformat: impl AsRef<str>,
    ) -> impl Iterator<Item = &str> {
        if let Some((start, end)) = self.keyformat_value_start_and_end_indices(keyformat) {
            self.inner_value(start, end)
        } else {
            ""
        }
        .split_terminator('/')
    }

    /// Inserts the CPC label for the given `KEYFORMAT`.
    ///
    /// If there is no `KEYFORMAT` entry of the provided type, then this creates it with the
    /// provided CPC value.
    ///
    /// For example, if entry exists:
    /// ```
    /// # use quick_m3u8::tag::hls::AllowedCpc;
    /// let mut allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV");
    /// allowed_cpc.insert_cpc_for_keyformat("com.example.drm1", "PC");
    /// assert_eq!("com.example.drm1:SMART-TV/PC", allowed_cpc.as_ref());
    /// ```
    /// For example, if entry does not exist:
    /// ```
    /// # use quick_m3u8::tag::hls::AllowedCpc;
    /// let mut allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV");
    /// allowed_cpc.insert_cpc_for_keyformat("com.example.drm2", "HW");
    /// assert_eq!("com.example.drm1:SMART-TV,com.example.drm2:HW", allowed_cpc.as_ref());
    /// ```
    ///
    /// The value returns true if the insert was successful, and false otherwise.
    /// ```
    /// # use quick_m3u8::tag::hls::AllowedCpc;
    /// let mut allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV");
    /// assert_eq!(
    ///     false,
    ///     allowed_cpc.insert_cpc_for_keyformat("com.example.drm1", "SMART-TV"),
    ///     "SMART-TV already exists and so the insert will return false"
    /// );
    /// assert_eq!(
    ///     true,
    ///     allowed_cpc.insert_cpc_for_keyformat("com.example.drm1", "PC"),
    ///     "PC does not exist already and so the insert will return true"
    /// );
    /// ```
    pub fn insert_cpc_for_keyformat(
        &mut self,
        keyformat: impl AsRef<str>,
        cpc_label: impl AsRef<str>,
    ) -> bool {
        if let Some((start, end)) = self.keyformat_value_start_and_end_indices(&keyformat) {
            let value = self.inner_value(start, end);
            if value.split_terminator('/').any(|c| c == cpc_label.as_ref()) {
                false
            } else {
                let value_is_empty = value.is_empty();
                let mut new_string = std::mem::take(&mut self.inner).into_owned();
                if let Some(end) = end {
                    if value_is_empty {
                        new_string.insert_str(end, cpc_label.as_ref());
                    } else {
                        let string = format!("/{}", cpc_label.as_ref());
                        new_string.insert_str(end, &string);
                    }
                } else if value_is_empty {
                    new_string.push_str(cpc_label.as_ref());
                } else {
                    let string = format!("/{}", cpc_label.as_ref());
                    new_string.push_str(&string);
                }
                self.inner = Cow::Owned(new_string);
                true
            }
        } else {
            let mut new_string = std::mem::take(&mut self.inner).into_owned();
            if !new_string.is_empty() {
                new_string.push(',');
            }
            new_string.push_str(keyformat.as_ref());
            new_string.push(':');
            new_string.push_str(cpc_label.as_ref());
            self.inner = Cow::Owned(new_string);
            true
        }
    }

    /// Removes the CPC label from the provided `KEYFORMAT`.
    ///
    /// If the label is the last for the `KEYFORMAT` entry, then this removes the entry.
    ///
    /// For example, if not last label:
    /// ```
    /// # use quick_m3u8::tag::hls::AllowedCpc;
    /// let mut allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV/PC");
    /// allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "PC");
    /// assert_eq!("com.example.drm1:SMART-TV", allowed_cpc.as_ref());
    /// ```
    /// For example, if label is last:
    /// ```
    /// # use quick_m3u8::tag::hls::AllowedCpc;
    /// let mut allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV");
    /// allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "SMART-TV");
    /// assert_eq!("", allowed_cpc.as_ref());
    /// ```
    ///
    /// The value returns true if the remove was successful, and false otherwise.
    /// ```
    /// # use quick_m3u8::tag::hls::AllowedCpc;
    /// let mut allowed_cpc = AllowedCpc::from("com.example.drm1:SMART-TV");
    /// assert_eq!(
    ///     false,
    ///     allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "PC"),
    ///     "PC is not in the list of labels and so the remove will return false"
    /// );
    /// assert_eq!(
    ///     true,
    ///     allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "SMART-TV"),
    ///     "SMART-TV is in the list of labels and so the remove will return true"
    /// );
    /// ```
    pub fn remove_cpc_for_keyformat(
        &mut self,
        keyformat: impl AsRef<str>,
        cpc_label: impl AsRef<str>,
    ) -> bool {
        let keyformat_length = keyformat.as_ref().len();
        if let Some((start, end)) = self.keyformat_value_start_and_end_indices(keyformat) {
            let value = self.inner_value(start, end);
            if value.split_terminator('/').any(|c| c == cpc_label.as_ref()) {
                let new_value = value
                    .split_terminator('/')
                    .filter(|c| *c != cpc_label.as_ref())
                    .collect::<Vec<&str>>()
                    .join("/");
                let mut new_string = std::mem::take(&mut self.inner).into_owned();
                if let Some(end) = end {
                    if new_value.is_empty() {
                        let start = start - (keyformat_length + 1); // +1 for ':'
                        let end = end + 1;
                        new_string.drain(start..end);
                    } else {
                        new_string.insert_str(end, &new_value);
                        new_string.drain(start..end);
                    }
                } else if new_value.is_empty() {
                    let start = start - (keyformat_length + 1); // +1 for ':'
                    new_string.drain(start..);
                } else {
                    new_string.drain(start..);
                    new_string.push_str(&new_value);
                }
                if new_string.as_bytes().iter().next_back() == Some(&b',') {
                    new_string.pop();
                }
                self.inner = Cow::Owned(new_string);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// This overrides the default `to_owned` provided as part of `#[derive(Clone)]`.
    ///
    /// The reason this exists is to provide better lifetime semantics by completely breaking ties
    /// to the reference data. This is done by converting the inner into an owned String.
    ///
    /// This method is important as otherwise it won't be possible to take a value from a tag,
    /// mutate the list, and then set it back onto the tag. Providing this method makes that
    /// possible.
    pub fn to_owned<'b>(&self) -> AllowedCpc<'b> {
        AllowedCpc::from(self.to_string())
    }

    fn keyformat_value_start_and_end_indices(
        &self,
        keyformat: impl AsRef<str>,
    ) -> Option<(usize, Option<usize>)> {
        let keyformat_bytes = keyformat.as_ref().as_bytes();
        let inner_bytes = self.inner.as_bytes();
        let bytes_count = keyformat_bytes.len();
        let finder = memmem::find_iter(inner_bytes, keyformat_bytes);
        for i in finder {
            // if the match does not end with b':' then it is not a real match and we should
            // continue searching
            if inner_bytes[i + bytes_count] == b':' {
                // we have a real match so there is at least something (though maybe just b',')
                let start = i + bytes_count + 1;
                if let Some(n) = memchr(b',', &inner_bytes[start..]) {
                    let end = start + n;
                    return Some((start, Some(end)));
                } else {
                    return Some((start, None));
                }
            }
        }
        None
    }

    fn inner_value(&self, start: usize, end: Option<usize>) -> &str {
        if let Some(end) = end {
            &self.inner[start..end]
        } else {
            &self.inner[start..]
        }
    }
}
impl<'a> AsRef<str> for AllowedCpc<'a> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl Display for AllowedCpc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl From<String> for AllowedCpc<'_> {
    fn from(value: String) -> Self {
        Self {
            inner: Cow::Owned(value),
        }
    }
}
impl<'a> From<&'a str> for AllowedCpc<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            inner: Cow::Borrowed(value),
        }
    }
}
impl<'a> From<AllowedCpc<'a>> for Cow<'a, str> {
    fn from(value: AllowedCpc<'a>) -> Self {
        value.inner
    }
}
impl<const N: usize> From<[FairPlayCpcLabel; N]> for AllowedCpc<'_> {
    fn from(value: [FairPlayCpcLabel; N]) -> Self {
        let mut list = Self::from("");
        for item in value {
            list.insert_cpc_for_fair_play(item);
        }
        list
    }
}
impl From<Vec<FairPlayCpcLabel>> for AllowedCpc<'_> {
    fn from(value: Vec<FairPlayCpcLabel>) -> Self {
        let mut list = Self::from("");
        for item in value {
            list.insert_cpc_for_fair_play(item);
        }
        list
    }
}

/// The attribute list for the tag (`#EXT-X-STREAM-INF:<attribute-list>`).
///
/// See [`StreamInf`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct StreamInfAttributeList<'a> {
    /// Corresponds to the `BANDWIDTH` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    bandwidth: u64,
    /// Corresponds to the `AVERAGE-BANDWIDTH` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    average_bandwidth: Option<u64>,
    /// Corresponds to the `SCORE` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    score: Option<f64>,
    /// Corresponds to the `CODECS` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    codecs: Option<Cow<'a, str>>,
    /// Corresponds to the `SUPPLEMENTAL-CODECS` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    supplemental_codecs: Option<Cow<'a, str>>,
    /// Corresponds to the `RESOLUTION` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    resolution: Option<DecimalResolution>,
    /// Corresponds to the `FRAME-RATE` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    frame_rate: Option<f64>,
    /// Corresponds to the `HDCP-LEVEL` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    hdcp_level: Option<Cow<'a, str>>,
    /// Corresponds to the `ALLOWED-CPC` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    allowed_cpc: Option<Cow<'a, str>>,
    /// Corresponds to the `VIDEO-RANGE` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    video_range: Option<Cow<'a, str>>,
    /// Corresponds to the `REQ-VIDEO-LAYOUT` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    req_video_layout: Option<Cow<'a, str>>,
    /// Corresponds to the `STABLE-VARIANT-ID` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    stable_variant_id: Option<Cow<'a, str>>,
    /// Corresponds to the `AUDIO` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    audio: Option<Cow<'a, str>>,
    /// Corresponds to the `VIDEO` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    video: Option<Cow<'a, str>>,
    /// Corresponds to the `SUBTITLES` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    subtitles: Option<Cow<'a, str>>,
    /// Corresponds to the `CLOSED-CAPTIONS` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    closed_captions: Option<Cow<'a, str>>,
    /// Corresponds to the `PATHWAY-ID` attribute.
    ///
    /// See [`StreamInf`] for a link to the HLS documentation for this attribute.
    pathway_id: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`StreamInfBuilder`] indicating that `bandwidth` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct StreamInfBandwidthNeedsToBeSet;
/// Placeholder struct for [`StreamInfBuilder`] indicating that `bandwidth` has been set.
#[derive(Debug, Clone, Copy)]
pub struct StreamInfBandwidthHasBeenSet;

/// A builder for convenience in constructing a [`StreamInf`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct StreamInfBuilder<'a, BandwidthStatus> {
    attribute_list: StreamInfAttributeList<'a>,
    bandwidth_status: PhantomData<BandwidthStatus>,
}
impl<'a> StreamInfBuilder<'a, StreamInfBandwidthNeedsToBeSet> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: StreamInfAttributeList {
                bandwidth: Default::default(),
                average_bandwidth: Default::default(),
                score: Default::default(),
                codecs: Default::default(),
                supplemental_codecs: Default::default(),
                resolution: Default::default(),
                frame_rate: Default::default(),
                hdcp_level: Default::default(),
                allowed_cpc: Default::default(),
                video_range: Default::default(),
                req_video_layout: Default::default(),
                stable_variant_id: Default::default(),
                audio: Default::default(),
                video: Default::default(),
                subtitles: Default::default(),
                closed_captions: Default::default(),
                pathway_id: Default::default(),
            },
            bandwidth_status: PhantomData,
        }
    }
}
impl<'a> StreamInfBuilder<'a, StreamInfBandwidthHasBeenSet> {
    /// Finish building and construct the `StreamInf`.
    pub fn finish(self) -> StreamInf<'a> {
        StreamInf::new(self.attribute_list)
    }
}
impl<'a, BandwidthStatus> StreamInfBuilder<'a, BandwidthStatus> {
    /// Add the provided `bandwidth` to the attributes built into `StreamInf`.
    pub fn with_bandwidth(
        mut self,
        bandwidth: u64,
    ) -> StreamInfBuilder<'a, StreamInfBandwidthHasBeenSet> {
        self.attribute_list.bandwidth = bandwidth;
        StreamInfBuilder {
            attribute_list: self.attribute_list,
            bandwidth_status: PhantomData,
        }
    }
    /// Add the provided `average_bandwidth` to the attributes built into `StreamInf`.
    pub fn with_average_bandwidth(mut self, average_bandwidth: u64) -> Self {
        self.attribute_list.average_bandwidth = Some(average_bandwidth);
        self
    }
    /// Add the provided `score` to the attributes built into `StreamInf`.
    pub fn with_score(mut self, score: f64) -> Self {
        self.attribute_list.score = Some(score);
        self
    }
    /// Add the provided `codecs` to the attributes built into `StreamInf`.
    pub fn with_codecs(mut self, codecs: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.codecs = Some(codecs.into());
        self
    }
    /// Add the provided `supplemental_codecs` to the attributes built into `StreamInf`.
    pub fn with_supplemental_codecs(
        mut self,
        supplemental_codecs: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.attribute_list.supplemental_codecs = Some(supplemental_codecs.into());
        self
    }
    /// Add the provided `resolution` to the attributes built into `StreamInf`.
    pub fn with_resolution(mut self, resolution: DecimalResolution) -> Self {
        self.attribute_list.resolution = Some(resolution);
        self
    }
    /// Add the provided `frame_rate` to the attributes built into `StreamInf`.
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.attribute_list.frame_rate = Some(frame_rate);
        self
    }
    /// Add the provided `hdcp_level` to the attributes built into `StreamInf`.
    ///
    /// Note that [`HdcpLevel`] implements `Into<Cow<str>>` and therefore can be used directly here.
    /// For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInfBuilder, HdcpLevel};
    /// let builder = StreamInfBuilder::new()
    ///     .with_hdcp_level(HdcpLevel::Type1);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInfBuilder, HdcpLevel};
    /// let builder = StreamInfBuilder::new()
    ///     .with_hdcp_level("TYPE-1");
    /// ```
    pub fn with_hdcp_level(mut self, hdcp_level: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.hdcp_level = Some(hdcp_level.into());
        self
    }
    /// Add the provided `allowed_cpc` to the attributes built into `StreamInf`.
    ///
    /// Note that [`AllowedCpc`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. `AllowedCpc` has several convenience initializers to help make this easier. For
    /// example, when setting FairPlay values specifically, the array format can be used:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInfBuilder, AllowedCpc, FairPlayCpcLabel};
    /// let builder = StreamInfBuilder::new()
    ///     .with_allowed_cpc(AllowedCpc::from([
    ///         FairPlayCpcLabel::AppleMain, FairPlayCpcLabel::Main
    ///     ]));
    /// ```
    /// Alternatively, a string slice can be used, but care should be taken to follow the correct
    /// syntax defined for `ALLOWED-CPC`.
    /// ```
    /// # use quick_m3u8::tag::hls::StreamInfBuilder;
    /// let builder = StreamInfBuilder::new()
    ///     .with_allowed_cpc("com.apple.streamingkeydelivery:AppleMain/Main");
    /// ```
    pub fn with_allowed_cpc(mut self, allowed_cpc: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.allowed_cpc = Some(allowed_cpc.into());
        self
    }
    /// Add the provided `video_range` to the attributes built into `StreamInf`.
    ///
    /// Note that [`VideoRange`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInfBuilder, VideoRange};
    /// let builder = StreamInfBuilder::new()
    ///     .with_video_range(VideoRange::Pq);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInfBuilder, VideoRange};
    /// let builder = StreamInfBuilder::new()
    ///     .with_video_range("PQ");
    /// ```
    pub fn with_video_range(mut self, video_range: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.video_range = Some(video_range.into());
        self
    }
    /// Add the provided `req_video_layout` to the attributes built into `StreamInf`.
    ///
    /// Note that [`VideoLayout`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{
    /// # StreamInfBuilder, VideoLayout, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoProjectionSpecifier
    /// # };
    /// let builder = StreamInfBuilder::new()
    ///     .with_req_video_layout(VideoLayout::new(
    ///         EnumeratedStringList::from([VideoChannelSpecifier::Stereo]),
    ///         EnumeratedStringList::from([VideoProjectionSpecifier::Equirectangular]),
    ///     ));
    /// ```
    /// Alternatively, a string slice can be used, but care should be taken to follow the correct
    /// syntax defined for `REQ-VIDEO-LAYOUT`.
    /// ```
    /// # use quick_m3u8::tag::hls::{
    /// # StreamInfBuilder, VideoLayout, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoProjectionSpecifier
    /// # };
    /// let builder = StreamInfBuilder::new()
    ///     .with_req_video_layout("CH-STEREO/PROJ-EQUI");
    /// ```
    pub fn with_req_video_layout(mut self, req_video_layout: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.req_video_layout = Some(req_video_layout.into());
        self
    }
    /// Add the provided `stable_variant_id` to the attributes built into `StreamInf`.
    pub fn with_stable_variant_id(mut self, stable_variant_id: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.stable_variant_id = Some(stable_variant_id.into());
        self
    }
    /// Add the provided `audio` to the attributes built into `StreamInf`.
    pub fn with_audio(mut self, audio: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.audio = Some(audio.into());
        self
    }
    /// Add the provided `video` to the attributes built into `StreamInf`.
    pub fn with_video(mut self, video: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.video = Some(video.into());
        self
    }
    /// Add the provided `subtitles` to the attributes built into `StreamInf`.
    pub fn with_subtitles(mut self, subtitles: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.subtitles = Some(subtitles.into());
        self
    }
    /// Add the provided `closed_captions` to the attributes built into `StreamInf`.
    pub fn with_closed_captions(mut self, closed_captions: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.closed_captions = Some(closed_captions.into());
        self
    }
    /// Add the provided `pathway_id` to the attributes built into `StreamInf`.
    pub fn with_pathway_id(mut self, pathway_id: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.pathway_id = Some(pathway_id.into());
        self
    }
}
impl<'a> Default for StreamInfBuilder<'a, StreamInfBandwidthNeedsToBeSet> {
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-STREAM-INF` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.2>
#[derive(Debug, Clone)]
pub struct StreamInf<'a> {
    bandwidth: u64,
    average_bandwidth: LazyAttribute<'a, u64>,
    score: LazyAttribute<'a, f64>,
    codecs: LazyAttribute<'a, Cow<'a, str>>,
    supplemental_codecs: LazyAttribute<'a, Cow<'a, str>>,
    resolution: LazyAttribute<'a, DecimalResolution>,
    frame_rate: LazyAttribute<'a, f64>,
    hdcp_level: LazyAttribute<'a, Cow<'a, str>>,
    allowed_cpc: LazyAttribute<'a, Cow<'a, str>>,
    video_range: LazyAttribute<'a, Cow<'a, str>>,
    req_video_layout: LazyAttribute<'a, Cow<'a, str>>,
    stable_variant_id: LazyAttribute<'a, Cow<'a, str>>,
    audio: LazyAttribute<'a, Cow<'a, str>>,
    video: LazyAttribute<'a, Cow<'a, str>>,
    subtitles: LazyAttribute<'a, Cow<'a, str>>,
    closed_captions: LazyAttribute<'a, Cow<'a, str>>,
    pathway_id: LazyAttribute<'a, Cow<'a, str>>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for StreamInf<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.bandwidth() == other.bandwidth()
            && self.average_bandwidth() == other.average_bandwidth()
            && self.score() == other.score()
            && self.codecs() == other.codecs()
            && self.supplemental_codecs() == other.supplemental_codecs()
            && self.resolution() == other.resolution()
            && self.frame_rate() == other.frame_rate()
            && self.hdcp_level() == other.hdcp_level()
            && self.allowed_cpc() == other.allowed_cpc()
            && self.video_range() == other.video_range()
            && self.req_video_layout() == other.req_video_layout()
            && self.stable_variant_id() == other.stable_variant_id()
            && self.audio() == other.audio()
            && self.video() == other.video()
            && self.subtitles() == other.subtitles()
            && self.closed_captions() == other.closed_captions()
            && self.pathway_id() == other.pathway_id()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for StreamInf<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_ordered_attribute_list()?;
        let mut bandwidth = None;
        let mut average_bandwidth = LazyAttribute::None;
        let mut score = LazyAttribute::None;
        let mut codecs = LazyAttribute::None;
        let mut supplemental_codecs = LazyAttribute::None;
        let mut resolution = LazyAttribute::None;
        let mut frame_rate = LazyAttribute::None;
        let mut hdcp_level = LazyAttribute::None;
        let mut allowed_cpc = LazyAttribute::None;
        let mut video_range = LazyAttribute::None;
        let mut req_video_layout = LazyAttribute::None;
        let mut stable_variant_id = LazyAttribute::None;
        let mut audio = LazyAttribute::None;
        let mut video = LazyAttribute::None;
        let mut subtitles = LazyAttribute::None;
        let mut closed_captions = LazyAttribute::None;
        let mut pathway_id = LazyAttribute::None;
        for (name, value) in attribute_list {
            match name {
                BANDWIDTH => {
                    bandwidth = value
                        .unquoted()
                        .and_then(|v| v.try_as_decimal_integer().ok())
                }
                AVERAGE_BANDWIDTH => average_bandwidth.found(value),
                SCORE => score.found(value),
                CODECS => codecs.found(value),
                SUPPLEMENTAL_CODECS => supplemental_codecs.found(value),
                RESOLUTION => resolution.found(value),
                FRAME_RATE => frame_rate.found(value),
                HDCP_LEVEL => hdcp_level.found(value),
                ALLOWED_CPC => allowed_cpc.found(value),
                VIDEO_RANGE => video_range.found(value),
                REQ_VIDEO_LAYOUT => req_video_layout.found(value),
                STABLE_VARIANT_ID => stable_variant_id.found(value),
                AUDIO => audio.found(value),
                VIDEO => video.found(value),
                SUBTITLES => subtitles.found(value),
                CLOSED_CAPTIONS => closed_captions.found(value),
                PATHWAY_ID => pathway_id.found(value),
                _ => (),
            }
        }
        let Some(bandwidth) = bandwidth else {
            return Err(super::ValidationError::MissingRequiredAttribute(BANDWIDTH));
        };
        Ok(Self {
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
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> StreamInf<'a> {
    /// Constructs a new `StreamInf` tag.
    fn new(attribute_list: StreamInfAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let StreamInfAttributeList {
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
        } = attribute_list;
        Self {
            bandwidth,
            average_bandwidth: average_bandwidth
                .map(LazyAttribute::new)
                .unwrap_or_default(),
            score: score.map(LazyAttribute::new).unwrap_or_default(),
            codecs: codecs.map(LazyAttribute::new).unwrap_or_default(),
            supplemental_codecs: supplemental_codecs
                .map(LazyAttribute::new)
                .unwrap_or_default(),
            resolution: resolution.map(LazyAttribute::new).unwrap_or_default(),
            frame_rate: frame_rate.map(LazyAttribute::new).unwrap_or_default(),
            hdcp_level: hdcp_level.map(LazyAttribute::new).unwrap_or_default(),
            allowed_cpc: allowed_cpc.map(LazyAttribute::new).unwrap_or_default(),
            video_range: video_range.map(LazyAttribute::new).unwrap_or_default(),
            req_video_layout: req_video_layout.map(LazyAttribute::new).unwrap_or_default(),
            stable_variant_id: stable_variant_id
                .map(LazyAttribute::new)
                .unwrap_or_default(),
            audio: audio.map(LazyAttribute::new).unwrap_or_default(),
            video: video.map(LazyAttribute::new).unwrap_or_default(),
            subtitles: subtitles.map(LazyAttribute::new).unwrap_or_default(),
            closed_captions: closed_captions.map(LazyAttribute::new).unwrap_or_default(),
            pathway_id: pathway_id.map(LazyAttribute::new).unwrap_or_default(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `StreamInf` as such:
    /// ```
    /// # use quick_m3u8::tag::{DecimalResolution, hls::{StreamInf, HdcpLevel, VideoRange}};
    /// let stream_inf = StreamInf::builder()
    ///     .with_bandwidth(10000000)
    ///     .with_codecs("hvc1.2.4.L153.b0")
    ///     .with_supplemental_codecs("dvh1.08.07/db4h")
    ///     .with_resolution(DecimalResolution { width: 3840, height: 2160 })
    ///     .with_hdcp_level(HdcpLevel::Type1)
    ///     .with_video_range(VideoRange::Hlg)
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `bandwidth`. The
    /// following fails to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::StreamInf;
    /// let stream_inf = StreamInf::builder().finish();
    /// ```
    pub fn builder() -> StreamInfBuilder<'a, StreamInfBandwidthNeedsToBeSet> {
        StreamInfBuilder::new()
    }

    /// Corresponds to the `BANDWIDTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn bandwidth(&self) -> u64 {
        self.bandwidth
    }

    /// Corresponds to the `AVERAGE-BANDWIDTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn average_bandwidth(&self) -> Option<u64> {
        match self.average_bandwidth {
            LazyAttribute::UserDefined(n) => Some(n),
            LazyAttribute::Unparsed(v) => {
                v.unquoted().and_then(|v| v.try_as_decimal_integer().ok())
            }
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `SCORE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn score(&self) -> Option<f64> {
        match self.score {
            LazyAttribute::UserDefined(n) => Some(n),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|v| v.try_as_decimal_floating_point().ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn codecs(&self) -> Option<&str> {
        match &self.codecs {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `SUPPLEMENTAL-CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn supplemental_codecs(&self) -> Option<&str> {
        match &self.supplemental_codecs {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `RESOLUTION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn resolution(&self) -> Option<DecimalResolution> {
        match self.resolution {
            LazyAttribute::UserDefined(d) => Some(d),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|v| v.try_as_decimal_resolution().ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `FRAME-RATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn frame_rate(&self) -> Option<f64> {
        match self.frame_rate {
            LazyAttribute::UserDefined(n) => Some(n),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|v| v.try_as_decimal_floating_point().ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `HDCP-LEVEL` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note that the convenience [`crate::tag::hls::GetKnown`] trait exists to make accessing the
    /// known case easier:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInf, HdcpLevel};
    /// use quick_m3u8::tag::hls::GetKnown;
    ///
    /// let tag = StreamInf::builder()
    ///     .with_bandwidth(10000000)
    ///     .with_hdcp_level(HdcpLevel::Type0)
    ///     .finish();
    /// assert_eq!(Some(HdcpLevel::Type0), tag.hdcp_level().known());
    /// ```
    pub fn hdcp_level(&self) -> Option<EnumeratedString<'_, HdcpLevel>> {
        match &self.hdcp_level {
            LazyAttribute::UserDefined(s) => Some(EnumeratedString::from(s.as_ref())),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|v| v.try_as_utf_8().ok())
                .map(EnumeratedString::from),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `ALLOWED-CPC` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// The `AllowedCpc` struct provides a strongly typed convenience wrapper around the string
    /// value of the `ALLOWED-CPC` attribute. It abstracts the access to the CPC label values for
    /// each `KEYFORMAT` entry; however, the user must define the `KEYFORMAT` they are looking for,
    /// with the exception of FairPlay where the struct provides convenience access methods.
    /// ```
    /// # use quick_m3u8::{Reader, HlsLine, config::ParsingOptions, tag::{KnownTag, hls}};
    /// # use quick_m3u8::tag::hls::{StreamInf, EnumeratedString, AllowedCpc, FairPlayCpcLabel};
    /// let tag = concat!(
    ///     "#EXT-X-STREAM-INF:BANDWIDTH=10000000,",
    ///     r#"ALLOWED-CPC="com.apple.streamingkeydelivery:AppleMain/Main,com.example.drm2:HW""#,
    /// );
    /// let mut reader = Reader::from_str(tag, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::StreamInf(mut stream_inf))))) => {
    ///         let mut allowed_cpc = stream_inf.allowed_cpc().expect("should be defined");
    ///         // Check FairPlay CPC labels
    ///         let mut fair_play = allowed_cpc.allowed_cpc_for_fair_play();
    ///         assert_eq!(
    ///             Some(EnumeratedString::Known(FairPlayCpcLabel::AppleMain)),
    ///             fair_play.next()
    ///         );
    ///         assert_eq!(
    ///             Some(EnumeratedString::Known(FairPlayCpcLabel::Main)),
    ///             fair_play.next()
    ///         );
    ///         assert_eq!(None, fair_play.next());
    ///         drop(fair_play);
    ///         // Check com.example.drm2 CPC labels
    ///         let mut drm2 = allowed_cpc.allowed_cpc_for_keyformat("com.example.drm2");
    ///         assert_eq!(Some("HW"), drm2.next());
    ///         assert_eq!(None, drm2.next());
    ///         drop(drm2);
    ///         // We can also mutate the retrieved `AllowedCpc`
    ///         allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::Main);
    ///         allowed_cpc.remove_cpc_for_keyformat("com.example.drm2", "HW");
    ///         allowed_cpc.insert_cpc_for_keyformat("com.example.drm1", "SMART-TV");
    ///         // And set it back on the tag
    ///         stream_inf.set_allowed_cpc(allowed_cpc.to_owned());
    ///         let new_allowed_cpc = stream_inf.allowed_cpc().expect("should be defined");
    ///         // And we can get the underlying string value via the `as_ref` method
    ///         assert_eq!(
    ///             "com.apple.streamingkeydelivery:AppleMain,com.example.drm1:SMART-TV",
    ///             new_allowed_cpc.as_ref(),
    ///         );
    ///         // We also have convenience initializers for FairPlay specific CPC labels, as
    ///         // demonstrated below
    ///         stream_inf.set_allowed_cpc(AllowedCpc::from([
    ///             FairPlayCpcLabel::AppleBaseline, FairPlayCpcLabel::Baseline
    ///         ]));
    ///         assert_eq!(
    ///             "com.apple.streamingkeydelivery:AppleBaseline/Baseline",
    ///             stream_inf.allowed_cpc().expect("should be defined").as_ref()
    ///         );
    ///     }
    ///     r => panic!("unexpected result {r:?}"),
    /// }
    /// ```
    pub fn allowed_cpc(&self) -> Option<AllowedCpc<'_>> {
        match &self.allowed_cpc {
            LazyAttribute::UserDefined(s) => Some(AllowedCpc::from(s.as_ref())),
            LazyAttribute::Unparsed(v) => v.quoted().map(AllowedCpc::from),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `VIDEO-RANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note that the convenience [`crate::tag::hls::GetKnown`] trait exists to make accessing the
    /// known case easier:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInf, VideoRange};
    /// use quick_m3u8::tag::hls::GetKnown;
    ///
    /// let tag = StreamInf::builder()
    ///     .with_bandwidth(10000000)
    ///     .with_video_range(VideoRange::Pq)
    ///     .finish();
    /// assert_eq!(Some(VideoRange::Pq), tag.video_range().known());
    /// ```
    pub fn video_range(&self) -> Option<EnumeratedString<'_, VideoRange>> {
        match &self.video_range {
            LazyAttribute::UserDefined(s) => Some(EnumeratedString::from(s.as_ref())),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|v| v.try_as_utf_8().ok())
                .map(EnumeratedString::from),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `REQ-VIDEO-LAYOUT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// The `VideoLayout` struct provides a strongly typed wrapper around the string value of the
    /// `REQ-VIDEO-LAYOUT` attribute. It abstracts the slash separated list and the syntax around
    /// it. We use [`EnumeratedStringList`] to provide a pseudo-set-like abstraction over each of
    /// the "specifiers" contained in the attribute value. This does not allocate to the heap (as
    /// would be the case with a `Vec` or `HashSet`) so is relatively little cost over using the
    /// `&str` directly but provides convenience types and methods. For example:
    /// ```
    /// # use quick_m3u8::{Reader, HlsLine, config::ParsingOptions, tag::{KnownTag, hls}};
    /// # use quick_m3u8::tag::hls::{StreamInf, VideoChannelSpecifier, VideoProjectionSpecifier};
    /// let tag = r#"#EXT-X-STREAM-INF:BANDWIDTH=10000000,REQ-VIDEO-LAYOUT="PROJ-PRIM/CH-STEREO""#;
    /// let mut reader = Reader::from_str(tag, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::StreamInf(stream_inf))))) => {
    ///         let video_layout = stream_inf.req_video_layout().expect("should be defined");
    ///         // Check channels specifiers
    ///         assert_eq!(1, video_layout.channels().iter().count());
    ///         assert!(video_layout.channels().contains(VideoChannelSpecifier::Stereo));
    ///         // Check projection specifiers
    ///         assert_eq!(1, video_layout.projection().iter().count());
    ///         assert!(
    ///             video_layout
    ///                 .projection()
    ///                 .contains(VideoProjectionSpecifier::ParametricImmersive)
    ///         );
    ///         // Validate no unknown entries
    ///         assert_eq!(0, video_layout.unknown_entries().count());
    ///         // At any stage we can escape-hatch to the inner `&str` representation:
    ///         assert_eq!("PROJ-PRIM/CH-STEREO", video_layout.as_ref());
    ///     }
    ///     r => panic!("unexpected result {r:?}"),
    /// }
    /// ```
    pub fn req_video_layout(&self) -> Option<VideoLayout<'_>> {
        match &self.req_video_layout {
            LazyAttribute::UserDefined(s) => Some(VideoLayout::from(s.as_ref())),
            LazyAttribute::Unparsed(v) => v.quoted().map(VideoLayout::from),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `STABLE-VARIANT-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn stable_variant_id(&self) -> Option<&str> {
        match &self.stable_variant_id {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `AUDIO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn audio(&self) -> Option<&str> {
        match &self.audio {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `VIDEO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn video(&self) -> Option<&str> {
        match &self.video {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `SUBTITLES` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn subtitles(&self) -> Option<&str> {
        match &self.subtitles {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `CLOSED-CAPTIONS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn closed_captions(&self) -> Option<&str> {
        match &self.closed_captions {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `PATHWAY-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn pathway_id(&self) -> Option<&str> {
        match &self.pathway_id {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Sets the `BANDWIDTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_bandwidth(&mut self, bandwidth: u64) {
        self.bandwidth = bandwidth;
        self.output_line_is_dirty = true;
    }

    /// Sets the `AVERAGE-BANDWIDTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_average_bandwidth(&mut self, average_bandwidth: u64) {
        self.average_bandwidth.set(average_bandwidth);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `AVERAGE-BANDWIDTH` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_average_bandwidth(&mut self) {
        self.average_bandwidth.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `SCORE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_score(&mut self, score: f64) {
        self.score.set(score);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SCORE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_score(&mut self) {
        self.score.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_codecs(&mut self, codecs: impl Into<Cow<'a, str>>) {
        self.codecs.set(codecs.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `CODECS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_codecs(&mut self) {
        self.codecs.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `SUPPLEMENTAL-CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_supplemental_codecs(&mut self, supplemental_codecs: impl Into<Cow<'a, str>>) {
        self.supplemental_codecs.set(supplemental_codecs.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SUPPLEMENTAL-CODECS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_supplemental_codecs(&mut self) {
        self.supplemental_codecs.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `RESOLUTION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_resolution(&mut self, resolution: DecimalResolution) {
        self.resolution.set(resolution);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `RESOLUTION` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_resolution(&mut self) {
        self.resolution.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `FRAME-RATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_frame_rate(&mut self, frame_rate: f64) {
        self.frame_rate.set(frame_rate);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `FRAME-RATE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_frame_rate(&mut self) {
        self.frame_rate.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `HDCP-LEVEL` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_hdcp_level(&mut self, hdcp_level: impl Into<Cow<'a, str>>) {
        self.hdcp_level.set(hdcp_level.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `HDCP-LEVEL` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_hdcp_level(&mut self) {
        self.hdcp_level.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `ALLOWED-CPC` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_allowed_cpc(&mut self, allowed_cpc: impl Into<Cow<'a, str>>) {
        self.allowed_cpc.set(allowed_cpc.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `ALLOWED-CPC` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_allowed_cpc(&mut self) {
        self.allowed_cpc.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `VIDEO-RANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_video_range(&mut self, video_range: impl Into<Cow<'a, str>>) {
        self.video_range.set(video_range.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `VIDEO-RANGE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_video_range(&mut self) {
        self.video_range.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `REQ-VIDEO-LAYOUT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Given that `VideoLayout` implements `Into<Cow<str>>` it is possible to work with
    /// `VideoLayout` directly here. For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInf, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoLayout};
    /// # let mut stream_inf = StreamInf::builder().with_bandwidth(10000000).finish();
    /// stream_inf.set_req_video_layout(VideoLayout::new(
    ///     EnumeratedStringList::from([
    ///         VideoChannelSpecifier::Stereo, VideoChannelSpecifier::Mono
    ///     ]),
    ///     "",
    /// ));
    /// assert_eq!(
    ///     "CH-STEREO,CH-MONO",
    ///     stream_inf.req_video_layout().expect("must be defined").as_ref()
    /// );
    /// ```
    /// It is also possible to set with a `&str` directly, but care should be taken to ensure the
    /// correct syntax is followed:
    /// ```
    /// # use quick_m3u8::tag::hls::{StreamInf, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoLayout, VideoProjectionSpecifier};
    /// # let mut stream_inf = StreamInf::builder().with_bandwidth(10000000).finish();
    /// stream_inf.set_req_video_layout("CH-STEREO,CH-MONO/PROJ-HEQU");
    /// let video_layout = stream_inf.req_video_layout().expect("should be defined");
    /// assert_eq!(2, video_layout.channels().iter().count());
    /// assert!(video_layout.channels().contains(VideoChannelSpecifier::Stereo));
    /// assert!(video_layout.channels().contains(VideoChannelSpecifier::Mono));
    /// assert_eq!(1, video_layout.projection().iter().count());
    /// assert!(video_layout.projection().contains(VideoProjectionSpecifier::HalfEquirectangular));
    /// ```
    /// The [`EnumeratedStringList`] provides some pseudo-set-like operations to help with mutating
    /// an existing value. Note, `to_owned` will need to be used on each of the string lists if
    /// setting back on the tag:
    /// ```
    /// # use quick_m3u8::{Reader, HlsLine, config::ParsingOptions, tag::{KnownTag, hls}};
    /// # use quick_m3u8::tag::hls::{StreamInf, VideoChannelSpecifier, VideoProjectionSpecifier,
    /// # VideoLayout};
    /// let tag = r#"#EXT-X-STREAM-INF:BANDWIDTH=10000000,REQ-VIDEO-LAYOUT="PROJ-PRIM/CH-STEREO""#;
    /// let mut reader = Reader::from_str(tag, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(KnownTag::Hls(hls::Tag::StreamInf(mut stream_inf))))) => {
    ///         let video_layout = stream_inf.req_video_layout().expect("should be defined");
    ///         let mut channels = video_layout.channels();
    ///         channels.insert(VideoChannelSpecifier::Mono);
    ///         let mut projection = video_layout.projection();
    ///         projection.remove(VideoProjectionSpecifier::ParametricImmersive);
    ///         stream_inf.set_req_video_layout(VideoLayout::new(
    ///             channels.to_owned(),
    ///             projection.to_owned()
    ///         ));
    ///         
    ///         let new_video_layout = stream_inf.req_video_layout().expect("should be defined");
    ///         assert_eq!("CH-STEREO,CH-MONO", new_video_layout.as_ref());
    ///     }
    ///     r => panic!("unexpected result {r:?}"),
    /// }
    /// ```
    pub fn set_req_video_layout(&mut self, req_video_layout: impl Into<Cow<'a, str>>) {
        self.req_video_layout.set(req_video_layout.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `REQ-VIDEO-LAYOUT` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_req_video_layout(&mut self) {
        self.req_video_layout.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `STABLE-VARIANT-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_stable_variant_id(&mut self, stable_variant_id: impl Into<Cow<'a, str>>) {
        self.stable_variant_id.set(stable_variant_id.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `STABLE-VARIANT-ID` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_stable_variant_id(&mut self) {
        self.stable_variant_id.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `AUDIO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_audio(&mut self, audio: impl Into<Cow<'a, str>>) {
        self.audio.set(audio.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `AUDIO` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_audio(&mut self) {
        self.audio.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `VIDEO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_video(&mut self, video: impl Into<Cow<'a, str>>) {
        self.video.set(video.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `VIDEO` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_video(&mut self) {
        self.video.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `SUBTITLES` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_subtitles(&mut self, subtitles: impl Into<Cow<'a, str>>) {
        self.subtitles.set(subtitles.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SUBTITLES` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_subtitles(&mut self) {
        self.subtitles.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `CLOSED-CAPTIONS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_closed_captions(&mut self, closed_captions: impl Into<Cow<'a, str>>) {
        self.closed_captions.set(closed_captions.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `CLOSED-CAPTIONS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_closed_captions(&mut self) {
        self.closed_captions.unset();
        self.output_line_is_dirty = true;
    }

    /// Sets the `PATHWAY-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_pathway_id(&mut self, pathway_id: impl Into<Cow<'a, str>>) {
        self.pathway_id.set(pathway_id.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `PATHWAY-ID` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_pathway_id(&mut self) {
        self.pathway_id.unset();
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&StreamInfAttributeList {
            bandwidth: self.bandwidth(),
            average_bandwidth: self.average_bandwidth(),
            score: self.score(),
            codecs: self.codecs().map(|x| x.into()),
            supplemental_codecs: self.supplemental_codecs().map(|x| x.into()),
            resolution: self.resolution(),
            frame_rate: self.frame_rate(),
            hdcp_level: self.hdcp_level().map(|x| x.into()),
            allowed_cpc: self.allowed_cpc().map(|x| x.into()),
            video_range: self.video_range().map(|x| x.into()),
            req_video_layout: self.req_video_layout().map(|x| x.into()),
            stable_variant_id: self.stable_variant_id().map(|x| x.into()),
            audio: self.audio().map(|x| x.into()),
            video: self.video().map(|x| x.into()),
            subtitles: self.subtitles().map(|x| x.into()),
            closed_captions: self.closed_captions().map(|x| x.into()),
            pathway_id: self.pathway_id().map(|x| x.into()),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(StreamInf);

const BANDWIDTH: &str = "BANDWIDTH";
const AVERAGE_BANDWIDTH: &str = "AVERAGE-BANDWIDTH";
const SCORE: &str = "SCORE";
const CODECS: &str = "CODECS";
const SUPPLEMENTAL_CODECS: &str = "SUPPLEMENTAL-CODECS";
const RESOLUTION: &str = "RESOLUTION";
const FRAME_RATE: &str = "FRAME-RATE";
const HDCP_LEVEL: &str = "HDCP-LEVEL";
const ALLOWED_CPC: &str = "ALLOWED-CPC";
const VIDEO_RANGE: &str = "VIDEO-RANGE";
const REQ_VIDEO_LAYOUT: &str = "REQ-VIDEO-LAYOUT";
const STABLE_VARIANT_ID: &str = "STABLE-VARIANT-ID";
const AUDIO: &str = "AUDIO";
const VIDEO: &str = "VIDEO";
const SUBTITLES: &str = "SUBTITLES";
const CLOSED_CAPTIONS: &str = "CLOSED-CAPTIONS";
const PATHWAY_ID: &str = "PATHWAY-ID";

fn calculate_line(attribute_list: &StreamInfAttributeList) -> Vec<u8> {
    let StreamInfAttributeList {
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
    } = attribute_list;
    let mut line = format!("#EXT-X-STREAM-INF:{BANDWIDTH}={bandwidth}");
    if let Some(average_bandwidth) = average_bandwidth {
        line.push_str(format!(",{AVERAGE_BANDWIDTH}={average_bandwidth}").as_str());
    }
    if let Some(score) = score {
        line.push_str(format!(",{SCORE}={score:?}").as_str());
    }
    if let Some(codecs) = codecs {
        line.push_str(format!(",{CODECS}=\"{codecs}\"").as_str());
    }
    if let Some(supplemental_codecs) = supplemental_codecs {
        line.push_str(format!(",{SUPPLEMENTAL_CODECS}=\"{supplemental_codecs}\"").as_str());
    }
    if let Some(resolution) = resolution {
        line.push_str(format!(",{RESOLUTION}={resolution}").as_str());
    }
    if let Some(frame_rate) = frame_rate {
        line.push_str(format!(",{FRAME_RATE}={frame_rate}").as_str());
    }
    if let Some(hdcp_level) = hdcp_level {
        line.push_str(format!(",{HDCP_LEVEL}={hdcp_level}").as_str());
    }
    if let Some(allowed_cpc) = allowed_cpc {
        line.push_str(format!(",{ALLOWED_CPC}=\"{allowed_cpc}\"").as_str());
    }
    if let Some(video_range) = video_range {
        line.push_str(format!(",{VIDEO_RANGE}={video_range}").as_str());
    }
    if let Some(req_video_layout) = req_video_layout {
        line.push_str(format!(",{REQ_VIDEO_LAYOUT}=\"{req_video_layout}\"").as_str());
    }
    if let Some(stable_variant_id) = stable_variant_id {
        line.push_str(format!(",{STABLE_VARIANT_ID}=\"{stable_variant_id}\"").as_str());
    }
    if let Some(audio) = audio {
        line.push_str(format!(",{AUDIO}=\"{audio}\"").as_str());
    }
    if let Some(video) = video {
        line.push_str(format!(",{VIDEO}=\"{video}\"").as_str());
    }
    if let Some(subtitles) = subtitles {
        line.push_str(format!(",{SUBTITLES}=\"{subtitles}\"").as_str());
    }
    if let Some(closed_captions) = closed_captions {
        line.push_str(format!(",{CLOSED_CAPTIONS}=\"{closed_captions}\"").as_str());
    }
    if let Some(pathway_id) = pathway_id {
        line.push_str(format!(",{PATHWAY_ID}=\"{pathway_id}\"").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-STREAM-INF:BANDWIDTH=10000000",
            StreamInf::builder()
                .with_bandwidth(10000000)
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            concat!(
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
            .as_bytes(),
            StreamInf::builder()
                .with_bandwidth(10000000)
                .with_average_bandwidth(9000000)
                .with_score(2.0)
                .with_codecs("hvc1.2.4.L153.b0,ec-3")
                .with_supplemental_codecs("dvh1.08.07/db4h")
                .with_resolution(DecimalResolution {
                    width: 3840,
                    height: 2160
                })
                .with_frame_rate(23.976)
                .with_hdcp_level("TYPE-1")
                .with_allowed_cpc("com.example.drm1:SMART-TV/PC")
                .with_video_range("PQ")
                .with_req_video_layout("CH-STEREO,CH-MONO")
                .with_stable_variant_id("1234")
                .with_audio("surround")
                .with_video("alternate-view")
                .with_subtitles("subs")
                .with_closed_captions("cc")
                .with_pathway_id("1234")
                .finish()
                .into_inner()
                .value()
        )
    }

    #[test]
    fn new_view_presentation_entries_displays_as_expected() {
        assert_eq!(
            "CH-STEREO",
            format!("{}", VideoLayout::new([VideoChannelSpecifier::Stereo], ""))
        );
        assert_eq!(
            "CH-MONO,CH-STEREO",
            format!(
                "{}",
                VideoLayout::new(
                    [VideoChannelSpecifier::Mono, VideoChannelSpecifier::Stereo],
                    ""
                )
            )
        );
        assert_eq!(
            "PROJ-EQUI",
            format!(
                "{}",
                VideoLayout::new("", [VideoProjectionSpecifier::Equirectangular])
            )
        );
        assert_eq!(
            "CH-STEREO/PROJ-PRIM",
            format!(
                "{}",
                VideoLayout::new(
                    [VideoChannelSpecifier::Stereo],
                    [VideoProjectionSpecifier::ParametricImmersive]
                )
            )
        );
    }

    #[test]
    fn view_presentation_entries_ordering_does_not_matter() {
        let entries = VideoLayout::from("CH-STEREO/PROJ-HEQU");
        assert!(
            entries.channels().contains(VideoChannelSpecifier::Stereo),
            "should contain Stereo entry"
        );
        assert!(
            entries
                .projection()
                .contains(VideoProjectionSpecifier::HalfEquirectangular),
            "should contain HalfEqurectangular entry"
        );

        let entries = VideoLayout::from("PROJ-HEQU/CH-STEREO");
        assert!(
            entries.channels().contains(VideoChannelSpecifier::Stereo),
            "should contain Stereo entry"
        );
        assert!(
            entries
                .projection()
                .contains(VideoProjectionSpecifier::HalfEquirectangular),
            "should contain HalfEqurectangular entry"
        );
    }

    #[test]
    fn allowed_cpc_should_get_cpc_labels_for_keyformat_as_iterator() {
        let allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:SMART-TV/PC,",
                "com.example.drm2:HW,",
                "com.apple.streamingkeydelivery:AppleMain/Main",
            )),
        };
        let mut drm1_cpc = allowed_cpc.allowed_cpc_for_keyformat("com.example.drm1");
        assert_eq!(Some("SMART-TV"), drm1_cpc.next());
        assert_eq!(Some("PC"), drm1_cpc.next());
        assert_eq!(None, drm1_cpc.next());
        let mut drm2_cpc = allowed_cpc.allowed_cpc_for_keyformat("com.example.drm2");
        assert_eq!(Some("HW"), drm2_cpc.next());
        assert_eq!(None, drm2_cpc.next());
        let mut fairplay_cpc = allowed_cpc.allowed_cpc_for_fair_play();
        assert_eq!(
            Some(EnumeratedString::Known(FairPlayCpcLabel::AppleMain)),
            fairplay_cpc.next()
        );
        assert_eq!(
            Some(EnumeratedString::Known(FairPlayCpcLabel::Main)),
            fairplay_cpc.next()
        );
    }

    #[test]
    fn allowed_cpc_should_return_empty_if_missing_keyformat() {
        let allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed("com.example.drm1:SMART-TV/PC"),
        };
        assert!(
            allowed_cpc
                .allowed_cpc_for_keyformat("com.example.drm2")
                .count()
                == 0
        );
        assert!(allowed_cpc.allowed_cpc_for_fair_play().count() == 0);
    }

    #[test]
    fn allowed_cpc_should_return_empty_if_no_cpc_labels() {
        let allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:,",
                "com.apple.streamingkeydelivery:",
            )),
        };
        assert!(
            allowed_cpc
                .allowed_cpc_for_keyformat("com.example.drm1")
                .count()
                == 0
        );
        assert!(allowed_cpc.allowed_cpc_for_fair_play().count() == 0);
    }

    #[test]
    fn allowed_cpc_remove_should_remove_cpc_label() {
        let mut allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:SMART-TV/PC,",
                "com.example.drm2:HW,",
                "com.apple.streamingkeydelivery:AppleMain/Main",
            )),
        };
        assert!(
            allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::Main),
            "remove should be successful"
        );
        assert!(
            allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "SMART-TV"),
            "remove should be successful"
        );
        assert_eq!(
            "com.example.drm1:PC,com.example.drm2:HW,com.apple.streamingkeydelivery:AppleMain",
            allowed_cpc.as_ref()
        );
    }

    #[test]
    fn allowed_cpc_remove_should_remove_keyformat_when_last_cpc_label() {
        let mut allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:SMART-TV/PC,",
                "com.example.drm2:HW,",
                "com.apple.streamingkeydelivery:AppleMain",
            )),
        };
        assert!(
            allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::AppleMain),
            "remove should be successful"
        );
        assert!(
            allowed_cpc.remove_cpc_for_keyformat("com.example.drm2", "HW"),
            "remove should be successful"
        );
        assert_eq!("com.example.drm1:SMART-TV/PC", allowed_cpc.as_ref());
        assert!(
            allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "SMART-TV"),
            "remove should be successful"
        );
        assert!(
            allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "PC"),
            "remove should be successful"
        );
        assert_eq!("", allowed_cpc.as_ref());
    }

    #[test]
    fn allowed_cpc_insert_should_insert_cpc_label() {
        let mut allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:SMART-TV/PC,",
                "com.apple.streamingkeydelivery:AppleMain",
            )),
        };
        assert!(
            allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::Main),
            "insert should be successful"
        );
        assert!(
            allowed_cpc.insert_cpc_for_keyformat("com.example.drm1", "MOBILE"),
            "insert should be successful"
        );
        assert_eq!(
            "com.example.drm1:SMART-TV/PC/MOBILE,com.apple.streamingkeydelivery:AppleMain/Main",
            allowed_cpc.as_ref()
        );
    }

    #[test]
    fn allowed_cpc_insert_should_create_key_format_if_does_not_exist() {
        let mut allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed("com.example.drm1:PC"),
        };
        assert!(
            allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::Baseline),
            "insert should be successful"
        );
        assert!(
            allowed_cpc.insert_cpc_for_keyformat("com.example.drm2", "SW"),
            "insert should be successful"
        );
        assert_eq!(
            "com.example.drm1:PC,com.apple.streamingkeydelivery:Baseline,com.example.drm2:SW",
            allowed_cpc.as_ref()
        );
    }

    #[test]
    fn allowed_cpc_insert_should_insert_even_if_keyformat_is_empty() {
        let mut allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:,",
                "com.apple.streamingkeydelivery:",
            )),
        };
        assert!(
            allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::AppleMain),
            "remove should be successful"
        );
        assert!(
            allowed_cpc.insert_cpc_for_keyformat("com.example.drm1", "PC"),
            "remove should be successful"
        );
        assert_eq!(
            "com.example.drm1:PC,com.apple.streamingkeydelivery:AppleMain",
            allowed_cpc.as_ref()
        );
    }

    #[test]
    fn allowed_cpc_insert_returns_false_if_trying_to_insert_existing_label() {
        let mut allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:SMART-TV/PC,",
                "com.apple.streamingkeydelivery:AppleMain",
            )),
        };
        assert_eq!(
            false,
            allowed_cpc.insert_cpc_for_fair_play(FairPlayCpcLabel::AppleMain)
        );
        assert_eq!(
            false,
            allowed_cpc.insert_cpc_for_keyformat("com.example.drm1", "PC")
        );
        assert_eq!(
            "com.example.drm1:SMART-TV/PC,com.apple.streamingkeydelivery:AppleMain",
            allowed_cpc.as_ref()
        );
    }

    #[test]
    fn allowed_cpc_remove_returns_false_if_trying_to_remove_non_existing_label() {
        let mut allowed_cpc = AllowedCpc {
            inner: Cow::Borrowed(concat!(
                "com.example.drm1:SMART-TV/PC,",
                "com.apple.streamingkeydelivery:AppleMain",
            )),
        };
        assert_eq!(
            false,
            allowed_cpc.remove_cpc_for_fair_play(FairPlayCpcLabel::Main)
        );
        assert_eq!(
            false,
            allowed_cpc.remove_cpc_for_keyformat("com.example.drm1", "MOBILE")
        );
        assert_eq!(
            false,
            allowed_cpc.remove_cpc_for_keyformat("com.example.drm2", "HW")
        );
        assert_eq!(
            "com.example.drm1:SMART-TV/PC,com.apple.streamingkeydelivery:AppleMain",
            allowed_cpc.as_ref()
        );
    }

    mutation_tests!(
        StreamInf::builder()
            .with_bandwidth(10000000)
            .with_average_bandwidth(9000000)
            .with_score(2.0)
            .with_codecs("hvc1.2.4.L153.b0,ec-3")
            .with_supplemental_codecs("dvh1.08.07/db4h")
            .with_resolution(DecimalResolution {
                width: 3840,
                height: 2160
            })
            .with_frame_rate(23.976)
            .with_hdcp_level("TYPE-1")
            .with_allowed_cpc("com.example.drm1:SMART-TV/PC")
            .with_video_range("PQ")
            .with_req_video_layout("CH-STEREO,CH-MONO")
            .with_stable_variant_id("1234")
            .with_audio("surround")
            .with_video("alternate-view")
            .with_subtitles("subs")
            .with_closed_captions("cc")
            .with_pathway_id("1234")
            .finish(),
        (bandwidth, 100, @Attr="BANDWIDTH=100"),
        (average_bandwidth, @Option 200, @Attr="AVERAGE-BANDWIDTH=200"),
        (score, @Option 1.0, @Attr="SCORE=1"),
        (codecs, @Option "example", @Attr="CODECS=\"example\""),
        (supplemental_codecs, @Option "example", @Attr="SUPPLEMENTAL-CODECS=\"example\""),
        (resolution, @Option DecimalResolution { width: 2, height: 4 }, @Attr="RESOLUTION=2x4"),
        (frame_rate, @Option 60.0, @Attr="FRAME-RATE=60"),
        (hdcp_level, @Option EnumeratedString::Known(HdcpLevel::None), @Attr="HDCP-LEVEL=NONE"),
        (
            allowed_cpc,
            @Option AllowedCpc::from("com.example.drm2:HW"),
            @Attr="ALLOWED-CPC=\"com.example.drm2:HW\""
        ),
        (video_range, @Option EnumeratedString::Known(VideoRange::Hlg), @Attr="VIDEO-RANGE=HLG"),
        (
            req_video_layout,
            @Option VideoLayout::new([VideoChannelSpecifier::Stereo], ""),
            @Attr="REQ-VIDEO-LAYOUT=\"CH-STEREO\""
        ),
        (stable_variant_id, @Option "abcd", @Attr="STABLE-VARIANT-ID=\"abcd\""),
        (audio, @Option "stereo", @Attr="AUDIO=\"stereo\""),
        (video, @Option "video", @Attr="VIDEO=\"video\""),
        (subtitles, @Option "subtitles", @Attr="SUBTITLES=\"subtitles\""),
        (closed_captions, @Option "example", @Attr="CLOSED-CAPTIONS=\"example\""),
        (pathway_id, @Option "abcd", @Attr="PATHWAY-ID=\"abcd\"")
    );
}
