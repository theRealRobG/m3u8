use crate::{
    error::{UnrecognizedEnumerationError, ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{EnumeratedString, EnumeratedStringList, into_inner_tag},
        known::ParsedTag,
        value::{DecimalResolution, ParsedAttributeValue, SemiParsedTagValue},
    },
    utils::AsStaticCow,
};
use std::{borrow::Cow, collections::HashMap, fmt::Display, marker::PhantomData};

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

/// Corresponds to the `#EXT-X-REQ-VIDEO-LAYOUT` attribute.
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
    /// # use m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier, VideoProjectionSpecifier,
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
    /// # use m3u8::tag::hls::VideoLayout;
    /// let video_layout = VideoLayout::new("CH-STEREO,CH-MONO", "PROJ-PRIM");
    /// assert_eq!("CH-STEREO,CH-MONO/PROJ-PRIM", video_layout.as_ref());
    /// ```
    /// The `From<&str>` implementation ensures that the order of specifiers does not impact the
    /// parsed value:
    /// ```
    /// # use m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier, VideoProjectionSpecifier};
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
    /// # use m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier};
    /// let video_layout = VideoLayout::new("CH-STEREO,CH-MONO", "");
    /// assert_eq!(2, video_layout.channels().iter().count());
    /// assert!(video_layout.channels().contains(VideoChannelSpecifier::Stereo));
    /// assert!(video_layout.channels().contains(VideoChannelSpecifier::Mono));
    /// assert_eq!("CH-STEREO,CH-MONO", video_layout.as_ref());
    /// ```
    /// Example with unknown specifier:
    /// ```
    /// # use m3u8::tag::hls::{VideoLayout, VideoChannelSpecifier};
    /// let video_layout = VideoLayout::new("CH-3D", "");
    /// assert_eq!(1, video_layout.channels().iter().count());
    /// assert!(video_layout.channels().contains("CH-3D"));
    /// assert_eq!("CH-3D", video_layout.as_ref());
    /// ```
    pub fn channels(&self) -> EnumeratedStringList<VideoChannelSpecifier> {
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
    /// # use m3u8::tag::hls::{VideoLayout, VideoProjectionSpecifier};
    /// let video_layout = VideoLayout::new("", "PROJ-EQUI,PROJ-HEQU");
    /// assert_eq!(2, video_layout.projection().iter().count());
    /// assert!(video_layout.projection().contains(VideoProjectionSpecifier::Equirectangular));
    /// assert!(video_layout.projection().contains(VideoProjectionSpecifier::HalfEquirectangular));
    /// assert_eq!("PROJ-EQUI,PROJ-HEQU", video_layout.as_ref());
    /// ```
    /// Example with unknown specifier:
    /// ```
    /// # use m3u8::tag::hls::{VideoLayout};
    /// let video_layout = VideoLayout::new("", "PROJ-360");
    /// assert_eq!(1, video_layout.projection().iter().count());
    /// assert!(video_layout.projection().contains("PROJ-360"));
    /// assert_eq!("PROJ-360", video_layout.as_ref());
    /// ```
    pub fn projection(&self) -> EnumeratedStringList<VideoProjectionSpecifier> {
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
    /// # use m3u8::tag::hls::VideoLayout;
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
    /// # use m3u8::tag::hls::{StreamInfBuilder, HdcpLevel};
    /// let builder = StreamInfBuilder::new()
    ///     .with_hdcp_level(HdcpLevel::Type1);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use m3u8::tag::hls::{StreamInfBuilder, HdcpLevel};
    /// let builder = StreamInfBuilder::new()
    ///     .with_hdcp_level("TYPE-1");
    /// ```
    pub fn with_hdcp_level(mut self, hdcp_level: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.hdcp_level = Some(hdcp_level.into());
        self
    }
    /// Add the provided `allowed_cpc` to the attributes built into `StreamInf`.
    pub fn with_allowed_cpc(mut self, allowed_cpc: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.allowed_cpc = Some(allowed_cpc.into());
        self
    }
    /// Add the provided `video_range` to the attributes built into `StreamInf`.
    ///
    /// Note that [`VideoRange`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. For example:
    /// ```
    /// # use m3u8::tag::hls::{StreamInfBuilder, VideoRange};
    /// let builder = StreamInfBuilder::new()
    ///     .with_video_range(VideoRange::Pq);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use m3u8::tag::hls::{StreamInfBuilder, VideoRange};
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
    /// # use m3u8::tag::hls::{
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
    /// # use m3u8::tag::hls::{
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

/// Corresponds to the `#EXT-X-STREAM-INF` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.2>
#[derive(Debug, Clone)]
pub struct StreamInf<'a> {
    bandwidth: u64,
    average_bandwidth: Option<u64>,
    score: Option<f64>,
    codecs: Option<Cow<'a, str>>,
    supplemental_codecs: Option<Cow<'a, str>>,
    resolution: Option<DecimalResolution>,
    frame_rate: Option<f64>,
    hdcp_level: Option<Cow<'a, str>>,
    allowed_cpc: Option<Cow<'a, str>>,
    video_range: Option<Cow<'a, str>>,
    req_video_layout: Option<Cow<'a, str>>,
    stable_variant_id: Option<Cow<'a, str>>,
    audio: Option<Cow<'a, str>>,
    video: Option<Cow<'a, str>>,
    subtitles: Option<Cow<'a, str>>,
    closed_captions: Option<Cow<'a, str>>,
    pathway_id: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
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

impl<'a> TryFrom<ParsedTag<'a>> for StreamInf<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::DecimalInteger(bandwidth)) = attribute_list.get(BANDWIDTH)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(BANDWIDTH));
        };
        Ok(Self {
            bandwidth: *bandwidth,
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
            attribute_list,
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
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `StreamInf` as such:
    /// ```
    /// # use m3u8::tag::{value::DecimalResolution, hls::{StreamInf, HdcpLevel, VideoRange}};
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
    /// # use m3u8::tag::hls::StreamInf;
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
        if let Some(average_bandwidth) = self.average_bandwidth {
            Some(average_bandwidth)
        } else {
            match self.attribute_list.get(AVERAGE_BANDWIDTH) {
                Some(ParsedAttributeValue::DecimalInteger(b)) => Some(*b),
                _ => None,
            }
        }
    }

    /// Corresponds to the `SCORE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn score(&self) -> Option<f64> {
        if let Some(score) = self.score {
            Some(score)
        } else {
            match self.attribute_list.get(SCORE) {
                Some(value) => value.as_option_f64(),
                _ => None,
            }
        }
    }

    /// Corresponds to the `CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn codecs(&self) -> Option<&str> {
        if let Some(codecs) = &self.codecs {
            Some(codecs)
        } else {
            match self.attribute_list.get(CODECS) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `SUPPLEMENTAL-CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn supplemental_codecs(&self) -> Option<&str> {
        if let Some(supplemental_codecs) = &self.supplemental_codecs {
            Some(supplemental_codecs)
        } else {
            match self.attribute_list.get(SUPPLEMENTAL_CODECS) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `RESOLUTION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn resolution(&self) -> Option<DecimalResolution> {
        if let Some(resolution) = self.resolution {
            Some(resolution)
        } else {
            match self.attribute_list.get(RESOLUTION) {
                Some(ParsedAttributeValue::UnquotedString(r)) => {
                    DecimalResolution::try_from(*r).ok()
                }
                _ => None,
            }
        }
    }

    /// Corresponds to the `FRAME-RATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn frame_rate(&self) -> Option<f64> {
        if let Some(frame_rate) = self.frame_rate {
            Some(frame_rate)
        } else {
            match self.attribute_list.get(FRAME_RATE) {
                Some(v) => v.as_option_f64(),
                _ => None,
            }
        }
    }

    /// Corresponds to the `HDCP-LEVEL` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note that the convenience [`crate::tag::hls::GetKnown`] trait exists to make accessing the
    /// known case easier:
    /// ```
    /// # use m3u8::tag::hls::{StreamInf, HdcpLevel};
    /// use m3u8::tag::hls::GetKnown;
    ///
    /// let tag = StreamInf::builder()
    ///     .with_bandwidth(10000000)
    ///     .with_hdcp_level(HdcpLevel::Type0)
    ///     .finish();
    /// assert_eq!(Some(HdcpLevel::Type0), tag.hdcp_level().known());
    /// ```
    pub fn hdcp_level(&self) -> Option<EnumeratedString<HdcpLevel>> {
        if let Some(hdcp_level) = &self.hdcp_level {
            Some(EnumeratedString::from(hdcp_level.as_ref()))
        } else {
            match self.attribute_list.get(HDCP_LEVEL) {
                Some(ParsedAttributeValue::UnquotedString(s)) => Some(EnumeratedString::from(*s)),
                _ => None,
            }
        }
    }

    /// Corresponds to the `ALLOWED-CPC` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn allowed_cpc(&self) -> Option<&str> {
        if let Some(allowed_cpc) = &self.allowed_cpc {
            Some(allowed_cpc)
        } else {
            match self.attribute_list.get(ALLOWED_CPC) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `VIDEO-RANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note that the convenience [`crate::tag::hls::GetKnown`] trait exists to make accessing the
    /// known case easier:
    /// ```
    /// # use m3u8::tag::hls::{StreamInf, VideoRange};
    /// use m3u8::tag::hls::GetKnown;
    ///
    /// let tag = StreamInf::builder()
    ///     .with_bandwidth(10000000)
    ///     .with_video_range(VideoRange::Pq)
    ///     .finish();
    /// assert_eq!(Some(VideoRange::Pq), tag.video_range().known());
    /// ```
    pub fn video_range(&self) -> Option<EnumeratedString<VideoRange>> {
        if let Some(video_range) = &self.video_range {
            Some(EnumeratedString::from(video_range.as_ref()))
        } else {
            match self.attribute_list.get(VIDEO_RANGE) {
                Some(ParsedAttributeValue::UnquotedString(s)) => Some(EnumeratedString::from(*s)),
                _ => None,
            }
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
    /// # use m3u8::{Reader, HlsLine, config::ParsingOptions, tag::{known, hls}};
    /// # use m3u8::tag::hls::{StreamInf, VideoChannelSpecifier, VideoProjectionSpecifier};
    /// let tag = r#"#EXT-X-STREAM-INF:BANDWIDTH=10000000,REQ-VIDEO-LAYOUT="PROJ-PRIM/CH-STEREO""#;
    /// let mut reader = Reader::from_str(tag, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::StreamInf(stream_inf))))) => {
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
    pub fn req_video_layout(&self) -> Option<VideoLayout> {
        if let Some(req_video_layout) = &self.req_video_layout {
            Some(VideoLayout::from(req_video_layout.as_ref()))
        } else {
            match self.attribute_list.get(REQ_VIDEO_LAYOUT) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(VideoLayout::from(*s)),
                _ => None,
            }
        }
    }

    /// Corresponds to the `STABLE-VARIANT-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn stable_variant_id(&self) -> Option<&str> {
        if let Some(stable_variant_id) = &self.stable_variant_id {
            Some(stable_variant_id)
        } else {
            match self.attribute_list.get(STABLE_VARIANT_ID) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `AUDIO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn audio(&self) -> Option<&str> {
        if let Some(audio) = &self.audio {
            Some(audio)
        } else {
            match self.attribute_list.get(AUDIO) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `VIDEO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn video(&self) -> Option<&str> {
        if let Some(video) = &self.video {
            Some(video)
        } else {
            match self.attribute_list.get(VIDEO) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `SUBTITLES` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn subtitles(&self) -> Option<&str> {
        if let Some(subtitles) = &self.subtitles {
            Some(subtitles)
        } else {
            match self.attribute_list.get(SUBTITLES) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `CLOSED-CAPTIONS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn closed_captions(&self) -> Option<&str> {
        if let Some(closed_captions) = &self.closed_captions {
            Some(closed_captions)
        } else {
            match self.attribute_list.get(CLOSED_CAPTIONS) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Corresponds to the `PATHWAY-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn pathway_id(&self) -> Option<&str> {
        if let Some(pathway_id) = &self.pathway_id {
            Some(pathway_id)
        } else {
            match self.attribute_list.get(PATHWAY_ID) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    /// Sets the `BANDWIDTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_bandwidth(&mut self, bandwidth: u64) {
        self.attribute_list.remove(BANDWIDTH);
        self.bandwidth = bandwidth;
        self.output_line_is_dirty = true;
    }

    /// Sets the `AVERAGE-BANDWIDTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_average_bandwidth(&mut self, average_bandwidth: u64) {
        self.attribute_list.remove(AVERAGE_BANDWIDTH);
        self.average_bandwidth = Some(average_bandwidth);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `AVERAGE-BANDWIDTH` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_average_bandwidth(&mut self) {
        self.attribute_list.remove(AVERAGE_BANDWIDTH);
        self.average_bandwidth = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `SCORE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_score(&mut self, score: f64) {
        self.attribute_list.remove(SCORE);
        self.score = Some(score);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SCORE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_score(&mut self) {
        self.attribute_list.remove(SCORE);
        self.score = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_codecs(&mut self, codecs: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CODECS);
        self.codecs = Some(codecs.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `CODECS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_codecs(&mut self) {
        self.attribute_list.remove(CODECS);
        self.codecs = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `SUPPLEMENTAL-CODECS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_supplemental_codecs(&mut self, supplemental_codecs: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(SUPPLEMENTAL_CODECS);
        self.supplemental_codecs = Some(supplemental_codecs.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SUPPLEMENTAL-CODECS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_supplemental_codecs(&mut self) {
        self.attribute_list.remove(SUPPLEMENTAL_CODECS);
        self.supplemental_codecs = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `RESOLUTION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_resolution(&mut self, resolution: DecimalResolution) {
        self.attribute_list.remove(RESOLUTION);
        self.resolution = Some(resolution);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `RESOLUTION` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_resolution(&mut self) {
        self.attribute_list.remove(RESOLUTION);
        self.resolution = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `FRAME-RATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_frame_rate(&mut self, frame_rate: f64) {
        self.attribute_list.remove(FRAME_RATE);
        self.frame_rate = Some(frame_rate);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `FRAME-RATE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_frame_rate(&mut self) {
        self.attribute_list.remove(FRAME_RATE);
        self.frame_rate = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `HDCP-LEVEL` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_hdcp_level(&mut self, hdcp_level: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(HDCP_LEVEL);
        self.hdcp_level = Some(hdcp_level.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `HDCP-LEVEL` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_hdcp_level(&mut self) {
        self.attribute_list.remove(HDCP_LEVEL);
        self.hdcp_level = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `ALLOWED-CPC` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_allowed_cpc(&mut self, allowed_cpc: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(ALLOWED_CPC);
        self.allowed_cpc = Some(allowed_cpc.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `ALLOWED-CPC` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_allowed_cpc(&mut self) {
        self.attribute_list.remove(ALLOWED_CPC);
        self.allowed_cpc = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `VIDEO-RANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_video_range(&mut self, video_range: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(VIDEO_RANGE);
        self.video_range = Some(video_range.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `VIDEO-RANGE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_video_range(&mut self) {
        self.attribute_list.remove(VIDEO_RANGE);
        self.video_range = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `REQ-VIDEO-LAYOUT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Given that `VideoLayout` implements `Into<Cow<str>>` it is possible to work with
    /// `VideoLayout` directly here. For example:
    /// ```
    /// # use m3u8::tag::hls::{StreamInf, EnumeratedStringList, VideoChannelSpecifier, VideoLayout};
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
    /// # use m3u8::tag::hls::{StreamInf, EnumeratedStringList, VideoChannelSpecifier, VideoLayout,
    /// # VideoProjectionSpecifier};
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
    /// # use m3u8::{Reader, HlsLine, config::ParsingOptions, tag::{known, hls}};
    /// # use m3u8::tag::hls::{StreamInf, VideoChannelSpecifier, VideoProjectionSpecifier,
    /// # VideoLayout};
    /// let tag = r#"#EXT-X-STREAM-INF:BANDWIDTH=10000000,REQ-VIDEO-LAYOUT="PROJ-PRIM/CH-STEREO""#;
    /// let mut reader = Reader::from_str(tag, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::StreamInf(mut stream_inf))))) => {
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
        self.attribute_list.remove(REQ_VIDEO_LAYOUT);
        self.req_video_layout = Some(req_video_layout.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `REQ-VIDEO-LAYOUT` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_req_video_layout(&mut self) {
        self.attribute_list.remove(REQ_VIDEO_LAYOUT);
        self.req_video_layout = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `STABLE-VARIANT-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_stable_variant_id(&mut self, stable_variant_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(STABLE_VARIANT_ID);
        self.stable_variant_id = Some(stable_variant_id.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `STABLE-VARIANT-ID` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_stable_variant_id(&mut self) {
        self.attribute_list.remove(STABLE_VARIANT_ID);
        self.stable_variant_id = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `AUDIO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_audio(&mut self, audio: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(AUDIO);
        self.audio = Some(audio.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `AUDIO` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_audio(&mut self) {
        self.attribute_list.remove(AUDIO);
        self.audio = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `VIDEO` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_video(&mut self, video: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(VIDEO);
        self.video = Some(video.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `VIDEO` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_video(&mut self) {
        self.attribute_list.remove(VIDEO);
        self.video = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `SUBTITLES` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_subtitles(&mut self, subtitles: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(SUBTITLES);
        self.subtitles = Some(subtitles.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `SUBTITLES` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_subtitles(&mut self) {
        self.attribute_list.remove(SUBTITLES);
        self.subtitles = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `CLOSED-CAPTIONS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_closed_captions(&mut self, closed_captions: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CLOSED_CAPTIONS);
        self.closed_captions = Some(closed_captions.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `CLOSED-CAPTIONS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_closed_captions(&mut self) {
        self.attribute_list.remove(CLOSED_CAPTIONS);
        self.closed_captions = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `PATHWAY-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_pathway_id(&mut self, pathway_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(PATHWAY_ID);
        self.pathway_id = Some(pathway_id.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `PATHWAY-ID` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_pathway_id(&mut self) {
        self.attribute_list.remove(PATHWAY_ID);
        self.pathway_id = None;
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
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
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
        (allowed_cpc, @Option "example", @Attr="ALLOWED-CPC=\"example\""),
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
