use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{DecimalResolution, ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

pub struct IFrameStreamInfAttributeList<'a> {
    pub uri: Cow<'a, str>,
    pub bandwidth: u64,
    pub average_bandwidth: Option<u64>,
    pub score: Option<f64>,
    pub codecs: Option<Cow<'a, str>>,
    pub supplemental_codecs: Option<Cow<'a, str>>,
    pub resolution: Option<DecimalResolution>,
    pub hdcp_level: Option<Cow<'a, str>>,
    pub allowed_cpc: Option<Cow<'a, str>>,
    pub video_range: Option<Cow<'a, str>>,
    pub req_video_layout: Option<Cow<'a, str>>,
    pub stable_variant_id: Option<Cow<'a, str>>,
    pub video: Option<Cow<'a, str>>,
    pub pathway_id: Option<Cow<'a, str>>,
}

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

    pub fn with_average_bandwidth(mut self, average_bandwidth: u64) -> Self {
        self.average_bandwidth = Some(average_bandwidth);
        self
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = Some(score);
        self
    }

    pub fn with_codecs(mut self, codecs: impl Into<Cow<'a, str>>) -> Self {
        self.codecs = Some(codecs.into());
        self
    }

    pub fn with_supplemental_codecs(
        mut self,
        supplemental_codecs: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.supplemental_codecs = Some(supplemental_codecs.into());
        self
    }

    pub fn with_resolution(mut self, resolution: DecimalResolution) -> Self {
        self.resolution = Some(resolution);
        self
    }

    pub fn with_hdcp_level(mut self, hdcp_level: impl Into<Cow<'a, str>>) -> Self {
        self.hdcp_level = Some(hdcp_level.into());
        self
    }

    pub fn with_allowed_cpc(mut self, allowed_cpc: impl Into<Cow<'a, str>>) -> Self {
        self.allowed_cpc = Some(allowed_cpc.into());
        self
    }

    pub fn with_video_range(mut self, video_range: impl Into<Cow<'a, str>>) -> Self {
        self.video_range = Some(video_range.into());
        self
    }

    pub fn with_req_video_layout(mut self, req_video_layout: impl Into<Cow<'a, str>>) -> Self {
        self.req_video_layout = Some(req_video_layout.into());
        self
    }

    pub fn with_stable_variant_id(mut self, stable_variant_id: impl Into<Cow<'a, str>>) -> Self {
        self.stable_variant_id = Some(stable_variant_id.into());
        self
    }

    pub fn with_video(mut self, video: impl Into<Cow<'a, str>>) -> Self {
        self.video = Some(video.into());
        self
    }

    pub fn with_pathway_id(mut self, pathway_id: impl Into<Cow<'a, str>>) -> Self {
        self.pathway_id = Some(pathway_id.into());
        self
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.3
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

    pub fn builder(uri: impl Into<Cow<'a, str>>, bandwidth: u64) -> IFrameStreamInfBuilder<'a> {
        IFrameStreamInfBuilder::new(uri, bandwidth)
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    // === GETTERS ===

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn bandwidth(&self) -> u64 {
        self.bandwidth
    }

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

    pub fn resolution(&self) -> Option<DecimalResolution> {
        if let Some(decimal_resolution) = self.resolution {
            Some(decimal_resolution)
        } else {
            match self.attribute_list.get(RESOLUTION) {
                Some(ParsedAttributeValue::UnquotedString(r)) => {
                    let mut split = r.splitn(2, 'x');
                    let Some(Ok(width)) = split.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    let Some(Ok(height)) = split.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    Some(DecimalResolution { width, height })
                }
                _ => None,
            }
        }
    }

    pub fn hdcp_level(&self) -> Option<&str> {
        if let Some(hdcp_level) = &self.hdcp_level {
            Some(hdcp_level)
        } else {
            match self.attribute_list.get(HDCP_LEVEL) {
                Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

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

    pub fn video_range(&self) -> Option<&str> {
        if let Some(video_range) = &self.video_range {
            Some(video_range)
        } else {
            match self.attribute_list.get(VIDEO_RANGE) {
                Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn req_video_layout(&self) -> Option<&str> {
        if let Some(req_video_layout) = &self.req_video_layout {
            Some(req_video_layout)
        } else {
            match self.attribute_list.get(REQ_VIDEO_LAYOUT) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

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

    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_bandwidth(&mut self, bandwidth: u64) {
        self.attribute_list.remove(BANDWIDTH);
        self.bandwidth = bandwidth;
        self.output_line_is_dirty = true;
    }

    pub fn set_average_bandwidth(&mut self, average_bandwidth: u64) {
        self.attribute_list.remove(AVERAGE_BANDWIDTH);
        self.average_bandwidth = Some(average_bandwidth);
        self.output_line_is_dirty = true;
    }

    pub fn unset_average_bandwidth(&mut self) {
        self.attribute_list.remove(AVERAGE_BANDWIDTH);
        self.average_bandwidth = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_score(&mut self, score: f64) {
        self.attribute_list.remove(SCORE);
        self.score = Some(score);
        self.output_line_is_dirty = true;
    }

    pub fn unset_score(&mut self) {
        self.attribute_list.remove(SCORE);
        self.score = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_codecs(&mut self, codecs: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CODECS);
        self.codecs = Some(codecs.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_codecs(&mut self) {
        self.attribute_list.remove(CODECS);
        self.codecs = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_supplemental_codecs(&mut self, supplemental_codecs: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(SUPPLEMENTAL_CODECS);
        self.supplemental_codecs = Some(supplemental_codecs.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_supplemental_codecs(&mut self) {
        self.attribute_list.remove(SUPPLEMENTAL_CODECS);
        self.supplemental_codecs = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_resolution(&mut self, resolution: DecimalResolution) {
        self.attribute_list.remove(RESOLUTION);
        self.resolution = Some(resolution);
        self.output_line_is_dirty = true;
    }

    pub fn unset_resolution(&mut self) {
        self.attribute_list.remove(RESOLUTION);
        self.resolution = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_hdcp_level(&mut self, hdcp_level: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(HDCP_LEVEL);
        self.hdcp_level = Some(hdcp_level.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_hdcp_level(&mut self) {
        self.attribute_list.remove(HDCP_LEVEL);
        self.hdcp_level = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_allowed_cpc(&mut self, allowed_cpc: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(ALLOWED_CPC);
        self.allowed_cpc = Some(allowed_cpc.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_allowed_cpc(&mut self) {
        self.attribute_list.remove(ALLOWED_CPC);
        self.allowed_cpc = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_video_range(&mut self, video_range: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(VIDEO_RANGE);
        self.video_range = Some(video_range.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_video_range(&mut self) {
        self.attribute_list.remove(VIDEO_RANGE);
        self.video_range = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_req_video_layout(&mut self, req_video_layout: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(REQ_VIDEO_LAYOUT);
        self.req_video_layout = Some(req_video_layout.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_req_video_layout(&mut self) {
        self.attribute_list.remove(REQ_VIDEO_LAYOUT);
        self.req_video_layout = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_stable_variant_id(&mut self, stable_variant_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(STABLE_VARIANT_ID);
        self.stable_variant_id = Some(stable_variant_id.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_stable_variant_id(&mut self) {
        self.attribute_list.remove(STABLE_VARIANT_ID);
        self.stable_variant_id = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_video(&mut self, video: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(VIDEO);
        self.video = Some(video.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_video(&mut self) {
        self.attribute_list.remove(VIDEO);
        self.video = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_pathway_id(&mut self, pathway_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(PATHWAY_ID);
        self.pathway_id = Some(pathway_id.into());
        self.output_line_is_dirty = true;
    }

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

fn calculate_line<'a>(attribute_list: &IFrameStreamInfAttributeList) -> Vec<u8> {
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
    use crate::tag::hls::test_macro::mutation_tests;
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
        (hdcp_level, @Option "NONE", @Attr="HDCP-LEVEL=NONE"),
        (allowed_cpc, @Option "EXAMPLE", @Attr="ALLOWED-CPC=\"EXAMPLE\""),
        (video_range, @Option "HLG", @Attr="VIDEO-RANGE=HLG"),
        (req_video_layout, @Option "CH-MONO", @Attr="REQ-VIDEO-LAYOUT=\"CH-MONO\""),
        (stable_variant_id, @Option "ABCD", @Attr="STABLE-VARIANT-ID=\"ABCD\""),
        (video, @Option "video", @Attr="VIDEO=\"video\""),
        (pathway_id, @Option "ABCD", @Attr="PATHWAY-ID=\"ABCD\"")
    );
}
