use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{
            EnumeratedString, into_inner_tag,
            stream_inf::{HdcpLevel, VideoLayout, VideoRange},
        },
        known::ParsedTag,
        value::{DecimalResolution, ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

/// The attribute list for the tag (`#EXT-X-I-FRAME-STREAM-INF:<attribute-list>`)
///
/// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
pub struct IFrameStreamInfAttributeList<'a> {
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub uri: Cow<'a, str>,
    /// Corresponds to the BANDWIDTH attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub bandwidth: u64,
    /// Corresponds to the AVERAGE-BANDWIDTH attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub average_bandwidth: Option<u64>,
    /// Corresponds to the SCORE attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub score: Option<f64>,
    /// Corresponds to the CODECS attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub codecs: Option<Cow<'a, str>>,
    /// Corresponds to the SUPPLEMENTAL-CODECS attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub supplemental_codecs: Option<Cow<'a, str>>,
    /// Corresponds to the RESOLUTION attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub resolution: Option<DecimalResolution>,
    /// Corresponds to the HDCP-LEVEL attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub hdcp_level: Option<Cow<'a, str>>,
    /// Corresponds to the ALLOWED-CPC attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub allowed_cpc: Option<Cow<'a, str>>,
    /// Corresponds to the VIDEO-RANGE attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub video_range: Option<Cow<'a, str>>,
    /// Corresponds to the REQ-VIDEO-LAYOUT attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub req_video_layout: Option<Cow<'a, str>>,
    /// Corresponds to the STABLE-VARIANT-ID attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub stable_variant_id: Option<Cow<'a, str>>,
    /// Corresponds to the VIDEO attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub video: Option<Cow<'a, str>>,
    /// Corresponds to the PATHWAY-ID attribute.
    ///
    /// See [`IFrameStreamInf`] for a link to the HLS documentation for this attribute.
    pub pathway_id: Option<Cow<'a, str>>,
}

/// A builder for convenience in constructing a [`IFrameStreamInf`].
#[derive(Debug)]
pub struct IFrameStreamInfBuilder<'a> {
    uri: Cow<'a, str>,
    bandwidth: u64,
    average_bandwidth: Option<u64>,
    score: Option<f64>,
    codecs: Option<Cow<'a, str>>,
    supplemental_codecs: Option<Cow<'a, str>>,
    resolution: Option<DecimalResolution>,
    hdcp_level: Option<Cow<'a, str>>,
    allowed_cpc: Option<Cow<'a, str>>,
    video_range: Option<Cow<'a, str>>,
    req_video_layout: Option<Cow<'a, str>>,
    stable_variant_id: Option<Cow<'a, str>>,
    video: Option<Cow<'a, str>>,
    pathway_id: Option<Cow<'a, str>>,
}
impl<'a> IFrameStreamInfBuilder<'a> {
    /// Creates a new builder.
    pub fn new(uri: impl Into<Cow<'a, str>>, bandwidth: u64) -> Self {
        Self {
            uri: uri.into(),
            bandwidth,
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
        }
    }

    /// Finish building and construct the `IFrameStreamInf`.
    pub fn finish(self) -> IFrameStreamInf<'a> {
        IFrameStreamInf::new(IFrameStreamInfAttributeList {
            uri: self.uri,
            bandwidth: self.bandwidth,
            average_bandwidth: self.average_bandwidth,
            score: self.score,
            codecs: self.codecs,
            supplemental_codecs: self.supplemental_codecs,
            resolution: self.resolution,
            hdcp_level: self.hdcp_level,
            allowed_cpc: self.allowed_cpc,
            video_range: self.video_range,
            req_video_layout: self.req_video_layout,
            stable_variant_id: self.stable_variant_id,
            video: self.video,
            pathway_id: self.pathway_id,
        })
    }

    /// Add the provided `average_bandwidth` to the attributes built into `IFrameStreamInf`.
    pub fn with_average_bandwidth(mut self, average_bandwidth: u64) -> Self {
        self.average_bandwidth = Some(average_bandwidth);
        self
    }

    /// Add the provided `score` to the attributes built into `IFrameStreamInf`.
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = Some(score);
        self
    }

    /// Add the provided `codecs` to the attributes built into `IFrameStreamInf`.
    pub fn with_codecs(mut self, codecs: impl Into<Cow<'a, str>>) -> Self {
        self.codecs = Some(codecs.into());
        self
    }

    /// Add the provided `supplemental_codecs` to the attributes built into `IFrameStreamInf`.
    pub fn with_supplemental_codecs(
        mut self,
        supplemental_codecs: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.supplemental_codecs = Some(supplemental_codecs.into());
        self
    }

    /// Add the provided `resolution` to the attributes built into `IFrameStreamInf`.
    pub fn with_resolution(mut self, resolution: DecimalResolution) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Add the provided `hdcp_level` to the attributes built into `IFrameStreamInf`.
    ///
    /// Note that [`HdcpLevel`] implements `Into<Cow<str>>` and therefore can be used directly here.
    /// For example:
    /// ```
    /// # use m3u8::tag::hls::{IFrameStreamInfBuilder, HdcpLevel};
    /// let builder = IFrameStreamInfBuilder::new("uri", 10000000)
    ///     .with_hdcp_level(HdcpLevel::Type1);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use m3u8::tag::hls::{IFrameStreamInfBuilder, HdcpLevel};
    /// let builder = IFrameStreamInfBuilder::new("uri", 10000000)
    ///     .with_hdcp_level("TYPE-1");
    /// ```
    pub fn with_hdcp_level(mut self, hdcp_level: impl Into<Cow<'a, str>>) -> Self {
        self.hdcp_level = Some(hdcp_level.into());
        self
    }

    /// Add the provided `allowed_cpc` to the attributes built into `IFrameStreamInf`.
    pub fn with_allowed_cpc(mut self, allowed_cpc: impl Into<Cow<'a, str>>) -> Self {
        self.allowed_cpc = Some(allowed_cpc.into());
        self
    }

    /// Add the provided `video_range` to the attributes built into `IFrameStreamInf`.
    ///
    /// Note that [`VideoRange`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. For example:
    /// ```
    /// # use m3u8::tag::hls::{IFrameStreamInfBuilder, VideoRange};
    /// let builder = IFrameStreamInfBuilder::new("uri", 10000000)
    ///     .with_video_range(VideoRange::Pq);
    /// ```
    /// Alternatively, a string slice can be used:
    /// ```
    /// # use m3u8::tag::hls::{IFrameStreamInfBuilder, VideoRange};
    /// let builder = IFrameStreamInfBuilder::new("uri", 10000000)
    ///     .with_video_range("PQ");
    /// ```
    pub fn with_video_range(mut self, video_range: impl Into<Cow<'a, str>>) -> Self {
        self.video_range = Some(video_range.into());
        self
    }

    /// Add the provided `req_video_layout` to the attributes built into `IFrameStreamInf`.
    ///
    /// Note that [`VideoLayout`] implements `Into<Cow<str>>` and therefore can be used directly
    /// here. For example:
    /// ```
    /// # use m3u8::tag::hls::{
    /// # IFrameStreamInfBuilder, VideoLayout, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoProjectionSpecifier
    /// # };
    /// let builder = IFrameStreamInfBuilder::new("uri", 10000000)
    ///     .with_req_video_layout(VideoLayout::new(
    ///         EnumeratedStringList::from([VideoChannelSpecifier::Stereo]),
    ///         EnumeratedStringList::from([VideoProjectionSpecifier::Equirectangular]),
    ///     ));
    /// ```
    /// Alternatively, a string slice can be used, but care should be taken to follow the correct
    /// syntax defined for `REQ-VIDEO-LAYOUT`.
    /// ```
    /// # use m3u8::tag::hls::{
    /// # IFrameStreamInfBuilder, VideoLayout, EnumeratedStringList, VideoChannelSpecifier,
    /// # VideoProjectionSpecifier
    /// # };
    /// let builder = IFrameStreamInfBuilder::new("uri", 10000000)
    ///     .with_req_video_layout("CH-STEREO/PROJ-EQUI");
    /// ```
    pub fn with_req_video_layout(mut self, req_video_layout: impl Into<Cow<'a, str>>) -> Self {
        self.req_video_layout = Some(req_video_layout.into());
        self
    }

    /// Add the provided `stable_variant_id` to the attributes built into `IFrameStreamInf`.
    pub fn with_stable_variant_id(mut self, stable_variant_id: impl Into<Cow<'a, str>>) -> Self {
        self.stable_variant_id = Some(stable_variant_id.into());
        self
    }

    /// Add the provided `video` to the attributes built into `IFrameStreamInf`.
    pub fn with_video(mut self, video: impl Into<Cow<'a, str>>) -> Self {
        self.video = Some(video.into());
        self
    }

    /// Add the provided `pathway_id` to the attributes built into `IFrameStreamInf`.
    pub fn with_pathway_id(mut self, pathway_id: impl Into<Cow<'a, str>>) -> Self {
        self.pathway_id = Some(pathway_id.into());
        self
    }
}

/// Corresponds to the `#EXT-X-I-FRAME-STREAM-INF` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.3>
#[derive(Debug, Clone)]
pub struct IFrameStreamInf<'a> {
    uri: Cow<'a, str>,
    bandwidth: u64,
    average_bandwidth: Option<u64>,
    score: Option<f64>,
    codecs: Option<Cow<'a, str>>,
    supplemental_codecs: Option<Cow<'a, str>>,
    resolution: Option<DecimalResolution>,
    hdcp_level: Option<Cow<'a, str>>,
    allowed_cpc: Option<Cow<'a, str>>,
    video_range: Option<Cow<'a, str>>,
    req_video_layout: Option<Cow<'a, str>>,
    stable_variant_id: Option<Cow<'a, str>>,
    video: Option<Cow<'a, str>>,
    pathway_id: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
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

impl<'a> TryFrom<ParsedTag<'a>> for IFrameStreamInf<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        let Some(ParsedAttributeValue::DecimalInteger(bandwidth)) = attribute_list.get(BANDWIDTH)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(BANDWIDTH));
        };
        Ok(Self {
            uri: Cow::Borrowed(uri),
            bandwidth: *bandwidth,
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
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> IFrameStreamInf<'a> {
    /// Constructs a new `IFrameStreamInf` tag.
    pub fn new(attribute_list: IFrameStreamInfAttributeList<'a>) -> Self {
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
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `IFrameStreamInf` as such:
    /// ```
    /// # use m3u8::tag::{value::DecimalResolution, hls::{IFrameStreamInf, HdcpLevel, VideoRange}};
    /// let i_frame_stream_inf = IFrameStreamInf::builder("uri", 10000000)
    ///     .with_codecs("hvc1.2.4.L153.b0")
    ///     .with_supplemental_codecs("dvh1.08.07/db4h")
    ///     .with_resolution(DecimalResolution { width: 3840, height: 2160 })
    ///     .with_hdcp_level(HdcpLevel::Type1)
    ///     .with_video_range(VideoRange::Hlg)
    ///     .finish();
    /// ```
    pub fn builder(uri: impl Into<Cow<'a, str>>, bandwidth: u64) -> IFrameStreamInfBuilder<'a> {
        IFrameStreamInfBuilder::new(uri, bandwidth)
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
        if let Some(decimal_resolution) = self.resolution {
            Some(decimal_resolution)
        } else {
            match self.attribute_list.get(RESOLUTION) {
                Some(ParsedAttributeValue::UnquotedString(r)) => {
                    DecimalResolution::try_from(*r).ok()
                }
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
    /// # use m3u8::tag::hls::{IFrameStreamInf, HdcpLevel};
    /// use m3u8::tag::hls::GetKnown;
    ///
    /// let tag = IFrameStreamInf::builder("uri", 10000000)
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
    /// # use m3u8::tag::hls::{IFrameStreamInf, VideoRange};
    /// use m3u8::tag::hls::GetKnown;
    ///
    /// let tag = IFrameStreamInf::builder("uri", 10000000)
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
    /// See [`crate::tag::hls::StreamInf::req_video_layout`] for more information on usage of
    /// [`VideoLayout`].
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

    // === SETTERS ===

    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
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

    /// Unsets the `SUPPLEMENTAL-CODECS` attribute.
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
    /// See [`crate::tag::hls::StreamInf::set_req_video_layout`] for more information on how to use
    /// this method.
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
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-I-FRAME-STREAM-INF:URI=\"example.iframe.m3u8\",BANDWIDTH=10000000",
            IFrameStreamInf::builder("example.iframe.m3u8", 10000000)
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
            IFrameStreamInf::builder("iframe.high.m3u8", 10000000)
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
        IFrameStreamInf::builder("iframe.high.m3u8", 10000000)
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
