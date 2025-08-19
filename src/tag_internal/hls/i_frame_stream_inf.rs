use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{
        DecimalResolution, UnknownTag,
        hls::{
            EnumeratedString, LazyAttribute, into_inner_tag,
            stream_inf::{HdcpLevel, VideoLayout, VideoRange},
        },
    },
};
use std::{borrow::Cow, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-I-FRAME-STREAM-INF:<attribute-list>`)
///
/// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct IFrameStreamInfAttributeList<'a> {
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    uri: Cow<'a, str>,
    /// Corresponds to the BANDWIDTH attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    bandwidth: u64,
    /// Corresponds to the AVERAGE-BANDWIDTH attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    average_bandwidth: Option<u64>,
    /// Corresponds to the SCORE attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    score: Option<f64>,
    /// Corresponds to the CODECS attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    codecs: Option<Cow<'a, str>>,
    /// Corresponds to the SUPPLEMENTAL-CODECS attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    supplemental_codecs: Option<Cow<'a, str>>,
    /// Corresponds to the RESOLUTION attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    resolution: Option<DecimalResolution>,
    /// Corresponds to the HDCP-LEVEL attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    hdcp_level: Option<Cow<'a, str>>,
    /// Corresponds to the ALLOWED-CPC attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    allowed_cpc: Option<Cow<'a, str>>,
    /// Corresponds to the VIDEO-RANGE attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    video_range: Option<Cow<'a, str>>,
    /// Corresponds to the REQ-VIDEO-LAYOUT attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    req_video_layout: Option<Cow<'a, str>>,
    /// Corresponds to the STABLE-VARIANT-ID attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    stable_variant_id: Option<Cow<'a, str>>,
    /// Corresponds to the VIDEO attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    video: Option<Cow<'a, str>>,
    /// Corresponds to the PATHWAY-ID attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pathway_id: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`IFrameStreamInfBuilder`] indicating that `uri` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct IFrameStreamInfUriNeedsToBeSet;
/// Placeholder struct for [`IFrameStreamInfBuilder`] indicating that `bandwidth` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct IFrameStreamInfBandwidthNeedsToBeSet;
/// Placeholder struct for [`IFrameStreamInfBuilder`] indicating that `uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct IFrameStreamInfUriHasBeenSet;
/// Placeholder struct for [`IFrameStreamInfBuilder`] indicating that `bandwidth` has been set.
#[derive(Debug, Clone, Copy)]
pub struct IFrameStreamInfBandwidthHasBeenSet;

/// A builder for convenience in constructing a [`IFrameStreamInf`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug)]
pub struct IFrameStreamInfBuilder<'a, UriStatus, BandwidthStatus> {
    attribute_list: IFrameStreamInfAttributeList<'a>,
    uri_status: PhantomData<UriStatus>,
    bandwidth_status: PhantomData<BandwidthStatus>,
}
impl<'a>
    IFrameStreamInfBuilder<'a, IFrameStreamInfUriNeedsToBeSet, IFrameStreamInfBandwidthNeedsToBeSet>
{
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: IFrameStreamInfAttributeList {
                uri: Cow::Borrowed(""),
                bandwidth: Default::default(),
                average_bandwidth: Default::default(),
                score: Default::default(),
                codecs: Default::default(),
                supplemental_codecs: Default::default(),
                resolution: Default::default(),
                hdcp_level: Default::default(),
                allowed_cpc: Default::default(),
                video_range: Default::default(),
                req_video_layout: Default::default(),
                stable_variant_id: Default::default(),
                video: Default::default(),
                pathway_id: Default::default(),
            },
            uri_status: PhantomData,
            bandwidth_status: PhantomData,
        }
    }
}
impl<'a>
    IFrameStreamInfBuilder<'a, IFrameStreamInfUriHasBeenSet, IFrameStreamInfBandwidthHasBeenSet>
{
    /// Finish building and construct the `IFrameStreamInf`.
    pub fn finish(self) -> IFrameStreamInf<'a> {
        IFrameStreamInf::new(self.attribute_list)
    }
}
impl<'a, UriStatus, BandwidthStatus> IFrameStreamInfBuilder<'a, UriStatus, BandwidthStatus> {
    /// Add the provided `uri` to the attributes built into `IFrameStreamInf`.
    pub fn with_uri(
        mut self,
        uri: impl Into<Cow<'a, str>>,
    ) -> IFrameStreamInfBuilder<'a, IFrameStreamInfUriHasBeenSet, BandwidthStatus> {
        self.attribute_list.uri = uri.into();
        IFrameStreamInfBuilder {
            attribute_list: self.attribute_list,
            uri_status: PhantomData,
            bandwidth_status: PhantomData,
        }
    }

    /// Add the provided `bandwidth` to the attributes built into `IFrameStreamInf`.
    pub fn with_bandwidth(
        mut self,
        bandwidth: u64,
    ) -> IFrameStreamInfBuilder<'a, UriStatus, IFrameStreamInfBandwidthHasBeenSet> {
        self.attribute_list.bandwidth = bandwidth;
        IFrameStreamInfBuilder {
            attribute_list: self.attribute_list,
            uri_status: PhantomData,
            bandwidth_status: PhantomData,
        }
    }

    /// Add the provided `average_bandwidth` to the attributes built into `IFrameStreamInf`.
    pub fn with_average_bandwidth(mut self, average_bandwidth: u64) -> Self {
        self.attribute_list.average_bandwidth = Some(average_bandwidth);
        self
    }

    /// Add the provided `score` to the attributes built into `IFrameStreamInf`.
    pub fn with_score(mut self, score: f64) -> Self {
        self.attribute_list.score = Some(score);
        self
    }

    /// Add the provided `codecs` to the attributes built into `IFrameStreamInf`.
    pub fn with_codecs(mut self, codecs: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.codecs = Some(codecs.into());
        self
    }

    /// Add the provided `supplemental_codecs` to the attributes built into `IFrameStreamInf`.
    pub fn with_supplemental_codecs(
        mut self,
        supplemental_codecs: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.attribute_list.supplemental_codecs = Some(supplemental_codecs.into());
        self
    }

    /// Add the provided `resolution` to the attributes built into `IFrameStreamInf`.
    pub fn with_resolution(mut self, resolution: DecimalResolution) -> Self {
        self.attribute_list.resolution = Some(resolution);
        self
    }

    /// Add the provided `hdcp_level` to the attributes built into `IFrameStreamInf`.
    ///
    /// Note that [`HdcpLevel`] implements `Into<Cow<str>>` and therefore can be used directly here.
    /// For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{IFrameStreamInfBuilder, HdcpLevel};
    /// let builder = IFrameStreamInfBuilder::new()
    ///     .with_uri("uri")
    ///     .with_bandwidth(10000000)
    ///     .with_hdcp_level(HdcpLevel::Type1);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use quick_m3u8::tag::hls::{IFrameStreamInfBuilder, HdcpLevel};
    /// let builder = IFrameStreamInfBuilder::new()
    ///     .with_uri("uri")
    ///     .with_bandwidth(10000000)
    ///     .with_hdcp_level("TYPE-1");
    /// ```
    pub fn with_hdcp_level(mut self, hdcp_level: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.hdcp_level = Some(hdcp_level.into());
        self
    }

    /// Add the provided `allowed_cpc` to the attributes built into `IFrameStreamInf`.
    pub fn with_allowed_cpc(mut self, allowed_cpc: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.allowed_cpc = Some(allowed_cpc.into());
        self
    }

    /// Add the provided `video_range` to the attributes built into `IFrameStreamInf`.
    ///
    /// Note that [`VideoRange`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{IFrameStreamInfBuilder, VideoRange};
    /// let builder = IFrameStreamInfBuilder::new()
    ///     .with_uri("uri")
    ///     .with_bandwidth(10000000)
    ///     .with_video_range(VideoRange::Pq);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use quick_m3u8::tag::hls::{IFrameStreamInfBuilder, VideoRange};
    /// let builder = IFrameStreamInfBuilder::new()
    ///     .with_uri("uri")
    ///     .with_bandwidth(10000000)
    ///     .with_video_range("PQ");
    /// ```
    pub fn with_video_range(mut self, video_range: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.video_range = Some(video_range.into());
        self
    }

    /// Add the provided `req_video_layout` to the attributes built into `IFrameStreamInf`.
    ///
    /// Note that [`VideoLayout`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. For example:
    /// ```
    /// # use quick_m3u8::tag::hls::{
    /// # IFrameStreamInfBuilder, VideoLayout, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoProjectionSpecifier
    /// # };
    /// let builder = IFrameStreamInfBuilder::new()
    ///     .with_uri("uri")
    ///     .with_bandwidth(10000000)
    ///     .with_req_video_layout(VideoLayout::new(
    ///         EnumeratedStringList::from([VideoChannelSpecifier::Stereo]),
    ///         EnumeratedStringList::from([VideoProjectionSpecifier::Equirectangular]),
    ///     ));
    /// ```
    /// Alternatively, a string slice can be used, but care should be taken to follow the correct
    /// syntax defined for `REQ-VIDEO-LAYOUT`.
    /// ```
    /// # use quick_m3u8::tag::hls::{
    /// # IFrameStreamInfBuilder, VideoLayout, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoProjectionSpecifier
    /// # };
    /// let builder = IFrameStreamInfBuilder::new()
    ///     .with_uri("uri")
    ///     .with_bandwidth(10000000)
    ///     .with_req_video_layout("CH-STEREO/PROJ-EQUI");
    /// ```
    pub fn with_req_video_layout(mut self, req_video_layout: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.req_video_layout = Some(req_video_layout.into());
        self
    }

    /// Add the provided `stable_variant_id` to the attributes built into `IFrameStreamInf`.
    pub fn with_stable_variant_id(mut self, stable_variant_id: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.stable_variant_id = Some(stable_variant_id.into());
        self
    }

    /// Add the provided `video` to the attributes built into `IFrameStreamInf`.
    pub fn with_video(mut self, video: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.video = Some(video.into());
        self
    }

    /// Add the provided `pathway_id` to the attributes built into `IFrameStreamInf`.
    pub fn with_pathway_id(mut self, pathway_id: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.pathway_id = Some(pathway_id.into());
        self
    }
}
impl<'a> Default
    for IFrameStreamInfBuilder<
        'a,
        IFrameStreamInfUriNeedsToBeSet,
        IFrameStreamInfBandwidthNeedsToBeSet,
    >
{
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-I-FRAME-STREAM-INF` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.3>
#[derive(Debug, Clone)]
pub struct IFrameStreamInf<'a> {
    uri: Cow<'a, str>,
    bandwidth: u64,
    average_bandwidth: LazyAttribute<'a, u64>,
    score: LazyAttribute<'a, f64>,
    codecs: LazyAttribute<'a, Cow<'a, str>>,
    supplemental_codecs: LazyAttribute<'a, Cow<'a, str>>,
    resolution: LazyAttribute<'a, DecimalResolution>,
    hdcp_level: LazyAttribute<'a, Cow<'a, str>>,
    allowed_cpc: LazyAttribute<'a, Cow<'a, str>>,
    video_range: LazyAttribute<'a, Cow<'a, str>>,
    req_video_layout: LazyAttribute<'a, Cow<'a, str>>,
    stable_variant_id: LazyAttribute<'a, Cow<'a, str>>,
    video: LazyAttribute<'a, Cow<'a, str>>,
    pathway_id: LazyAttribute<'a, Cow<'a, str>>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for IFrameStreamInf<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri() == other.uri()
            && self.bandwidth() == other.bandwidth()
            && self.average_bandwidth() == other.average_bandwidth()
            && self.score() == other.score()
            && self.codecs() == other.codecs()
            && self.supplemental_codecs() == other.supplemental_codecs()
            && self.resolution() == other.resolution()
            && self.hdcp_level() == other.hdcp_level()
            && self.allowed_cpc() == other.allowed_cpc()
            && self.video_range() == other.video_range()
            && self.req_video_layout() == other.req_video_layout()
            && self.stable_variant_id() == other.stable_variant_id()
            && self.video() == other.video()
            && self.pathway_id() == other.pathway_id()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for IFrameStreamInf<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_ordered_attribute_list()?;
        let mut uri = None;
        let mut bandwidth = None;
        let mut average_bandwidth = LazyAttribute::None;
        let mut score = LazyAttribute::None;
        let mut codecs = LazyAttribute::None;
        let mut supplemental_codecs = LazyAttribute::None;
        let mut resolution = LazyAttribute::None;
        let mut hdcp_level = LazyAttribute::None;
        let mut allowed_cpc = LazyAttribute::None;
        let mut video_range = LazyAttribute::None;
        let mut req_video_layout = LazyAttribute::None;
        let mut stable_variant_id = LazyAttribute::None;
        let mut video = LazyAttribute::None;
        let mut pathway_id = LazyAttribute::None;
        for (name, value) in attribute_list {
            match name {
                URI => uri = value.quoted(),
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
                HDCP_LEVEL => hdcp_level.found(value),
                ALLOWED_CPC => allowed_cpc.found(value),
                VIDEO_RANGE => video_range.found(value),
                REQ_VIDEO_LAYOUT => req_video_layout.found(value),
                STABLE_VARIANT_ID => stable_variant_id.found(value),
                VIDEO => video.found(value),
                PATHWAY_ID => pathway_id.found(value),
                _ => (),
            }
        }
        let Some(uri) = uri else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        let Some(bandwidth) = bandwidth else {
            return Err(super::ValidationError::MissingRequiredAttribute(BANDWIDTH));
        };
        Ok(Self {
            uri: Cow::Borrowed(uri),
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
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> IFrameStreamInf<'a> {
    /// Constructs a new `IFrameStreamInf` tag.
    fn new(attribute_list: IFrameStreamInfAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let IFrameStreamInfAttributeList {
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
        } = attribute_list;
        Self {
            uri,
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
            hdcp_level: hdcp_level.map(LazyAttribute::new).unwrap_or_default(),
            allowed_cpc: allowed_cpc.map(LazyAttribute::new).unwrap_or_default(),
            video_range: video_range.map(LazyAttribute::new).unwrap_or_default(),
            req_video_layout: req_video_layout.map(LazyAttribute::new).unwrap_or_default(),
            stable_variant_id: stable_variant_id
                .map(LazyAttribute::new)
                .unwrap_or_default(),
            video: video.map(LazyAttribute::new).unwrap_or_default(),
            pathway_id: pathway_id.map(LazyAttribute::new).unwrap_or_default(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `IFrameStreamInf` as such:
    /// ```
    /// # use quick_m3u8::tag::{DecimalResolution, hls::{IFrameStreamInf, HdcpLevel,
    /// # VideoRange}};
    /// let i_frame_stream_inf = IFrameStreamInf::builder()
    ///     .with_uri("uri")
    ///     .with_bandwidth(10000000)
    ///     .with_codecs("hvc1.2.4.L153.b0")
    ///     .with_supplemental_codecs("dvh1.08.07/db4h")
    ///     .with_resolution(DecimalResolution { width: 3840, height: 2160 })
    ///     .with_hdcp_level(HdcpLevel::Type1)
    ///     .with_video_range(VideoRange::Hlg)
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `uri` AND `bandwidth`.
    /// Each of the following fail to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::IFrameStreamInf;
    /// let i_frame_stream_inf = IFrameStreamInf::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::IFrameStreamInf;
    /// let i_frame_stream_inf = IFrameStreamInf::builder().with_uri("uri").finish();
    /// ```
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::IFrameStreamInf;
    /// let i_frame_stream_inf = IFrameStreamInf::builder().with_bandwidth(10000000).finish();
    /// ```
    pub fn builder() -> IFrameStreamInfBuilder<
        'a,
        IFrameStreamInfUriNeedsToBeSet,
        IFrameStreamInfBandwidthNeedsToBeSet,
    > {
        IFrameStreamInfBuilder::new()
    }

    // === GETTERS ===

    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> &str {
        &self.uri
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
        match &self.average_bandwidth {
            LazyAttribute::UserDefined(b) => Some(*b),
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
        match &self.score {
            LazyAttribute::UserDefined(s) => Some(*s),
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
        match &self.resolution {
            LazyAttribute::UserDefined(r) => Some(*r),
            LazyAttribute::Unparsed(v) => v
                .unquoted()
                .and_then(|v| v.try_as_decimal_resolution().ok()),
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
    /// # use quick_m3u8::tag::hls::{IFrameStreamInf, HdcpLevel};
    /// use quick_m3u8::tag::hls::GetKnown;
    ///
    /// let tag = IFrameStreamInf::builder()
    ///     .with_uri("uri")
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
    pub fn allowed_cpc(&self) -> Option<&str> {
        match &self.allowed_cpc {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
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
    /// # use quick_m3u8::tag::hls::{IFrameStreamInf, VideoRange};
    /// use quick_m3u8::tag::hls::GetKnown;
    ///
    /// let tag = IFrameStreamInf::builder()
    ///     .with_uri("uri")
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
    /// See [`crate::tag::hls::StreamInf::req_video_layout`] for more information on usage of
    /// [`VideoLayout`].
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

    // === SETTERS ===

    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.uri = uri.into();
        self.output_line_is_dirty = true;
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

    /// Unsets the `SUPPLEMENTAL-CODECS` attribute.
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
    /// See [`crate::tag::hls::StreamInf::set_req_video_layout`] for more information on how to use
    /// this method.
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
        self.output_line = Cow::Owned(calculate_line(&IFrameStreamInfAttributeList {
            uri: self.uri().into(),
            bandwidth: self.bandwidth(),
            average_bandwidth: self.average_bandwidth(),
            score: self.score(),
            codecs: self.codecs().map(|x| x.into()),
            supplemental_codecs: self.supplemental_codecs().map(|x| x.into()),
            resolution: self.resolution(),
            hdcp_level: self.hdcp_level().map(|x| x.into()),
            allowed_cpc: self.allowed_cpc().map(|x| x.into()),
            video_range: self.video_range().map(|x| x.into()),
            req_video_layout: self.req_video_layout().map(|x| x.into()),
            stable_variant_id: self.stable_variant_id().map(|x| x.into()),
            video: self.video().map(|x| x.into()),
            pathway_id: self.pathway_id().map(|x| x.into()),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(IFrameStreamInf);

const URI: &str = "URI";
const BANDWIDTH: &str = "BANDWIDTH";
const AVERAGE_BANDWIDTH: &str = "AVERAGE-BANDWIDTH";
const SCORE: &str = "SCORE";
const CODECS: &str = "CODECS";
const SUPPLEMENTAL_CODECS: &str = "SUPPLEMENTAL-CODECS";
const RESOLUTION: &str = "RESOLUTION";
const HDCP_LEVEL: &str = "HDCP-LEVEL";
const ALLOWED_CPC: &str = "ALLOWED-CPC";
const VIDEO_RANGE: &str = "VIDEO-RANGE";
const REQ_VIDEO_LAYOUT: &str = "REQ-VIDEO-LAYOUT";
const STABLE_VARIANT_ID: &str = "STABLE-VARIANT-ID";
const VIDEO: &str = "VIDEO";
const PATHWAY_ID: &str = "PATHWAY-ID";

fn calculate_line(attribute_list: &IFrameStreamInfAttributeList) -> Vec<u8> {
    let IFrameStreamInfAttributeList {
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
    } = attribute_list;
    let mut line = format!("#EXT-X-I-FRAME-STREAM-INF:{URI}=\"{uri}\",{BANDWIDTH}={bandwidth}");
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
    if let Some(video) = video {
        line.push_str(format!(",{VIDEO}=\"{video}\"").as_str());
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
            b"#EXT-X-I-FRAME-STREAM-INF:URI=\"example.iframe.m3u8\",BANDWIDTH=10000000",
            IFrameStreamInf::builder()
                .with_uri("example.iframe.m3u8")
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
                "#EXT-X-I-FRAME-STREAM-INF:URI=\"iframe.high.m3u8\",BANDWIDTH=10000000,",
                "AVERAGE-BANDWIDTH=9000000,SCORE=2.0,CODECS=\"hvc1.2.4.L153.b0,ec-3\",",
                "SUPPLEMENTAL-CODECS=\"dvh1.08.07/db4h\",RESOLUTION=3840x2160,HDCP-LEVEL=TYPE-1,",
                "ALLOWED-CPC=\"com.example.drm1:SMART-TV/PC\",VIDEO-RANGE=PQ,",
                "REQ-VIDEO-LAYOUT=\"CH-STEREO,CH-MONO\",STABLE-VARIANT-ID=\"1234\",",
                "VIDEO=\"alternate-view\",PATHWAY-ID=\"1234\""
            )
            .as_bytes(),
            IFrameStreamInf::builder()
                .with_uri("iframe.high.m3u8")
                .with_bandwidth(10000000)
                .with_average_bandwidth(9000000)
                .with_score(2.0)
                .with_codecs("hvc1.2.4.L153.b0,ec-3")
                .with_supplemental_codecs("dvh1.08.07/db4h")
                .with_resolution(DecimalResolution {
                    width: 3840,
                    height: 2160
                })
                .with_hdcp_level("TYPE-1")
                .with_allowed_cpc("com.example.drm1:SMART-TV/PC")
                .with_video_range("PQ")
                .with_req_video_layout("CH-STEREO,CH-MONO")
                .with_stable_variant_id("1234")
                .with_video("alternate-view")
                .with_pathway_id("1234")
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        // Initial value
        IFrameStreamInf::builder()
            .with_uri("iframe.high.m3u8")
            .with_bandwidth(10000000)
            .with_average_bandwidth(9000000)
            .with_score(2.0)
            .with_codecs("hvc1.2.4.L153.b0,ec-3")
            .with_supplemental_codecs("dvh1.08.07/db4h")
            .with_resolution(DecimalResolution {
                width: 3840,
                height: 2160
            })
            .with_hdcp_level("TYPE-1")
            .with_allowed_cpc("com.example.drm1:SMART-TV/PC")
            .with_video_range("PQ")
            .with_req_video_layout("CH-STEREO,CH-MONO")
            .with_stable_variant_id("1234")
            .with_video("alternate-view")
            .with_pathway_id("1234")
            .finish(),
        // Mutations and expected attributes
        (uri, "example", @Attr="URI=\"example\""),
        (bandwidth, 100, @Attr="BANDWIDTH=100"),
        (average_bandwidth, @Option 100, @Attr="AVERAGE-BANDWIDTH=100"),
        (score, @Option 5.0, @Attr="SCORE=5.0"),
        (codecs, @Option "example", @Attr="CODECS=\"example\""),
        (supplemental_codecs, @Option "example", @Attr="SUPPLEMENTAL-CODECS=\"example\""),
        (
            resolution,
            @Option DecimalResolution {
                width: 100,
                height: 100
            },
            @Attr="RESOLUTION=100x100"
        ),
        (hdcp_level, @Option EnumeratedString::Known(HdcpLevel::None), @Attr="HDCP-LEVEL=NONE"),
        (allowed_cpc, @Option "EXAMPLE", @Attr="ALLOWED-CPC=\"EXAMPLE\""),
        (video_range, @Option EnumeratedString::Known(VideoRange::Hlg), @Attr="VIDEO-RANGE=HLG"),
        (
            req_video_layout,
            @Option VideoLayout::new(["CH-MONO"], ""),
            @Attr="REQ-VIDEO-LAYOUT=\"CH-MONO\""
        ),
        (stable_variant_id, @Option "ABCD", @Attr="STABLE-VARIANT-ID=\"ABCD\""),
        (video, @Option "video", @Attr="VIDEO=\"video\""),
        (pathway_id, @Option "ABCD", @Attr="PATHWAY-ID=\"ABCD\"")
    );
}
