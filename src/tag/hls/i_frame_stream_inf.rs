use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{DecimalResolution, ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

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
    output_line: Cow<'a, str>,                                  // Used with Writer
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
    pub fn new(
        uri: String,
        bandwidth: u64,
        average_bandwidth: Option<u64>,
        score: Option<f64>,
        codecs: Option<String>,
        supplemental_codecs: Option<String>,
        resolution: Option<DecimalResolution>,
        hdcp_level: Option<String>,
        allowed_cpc: Option<String>,
        video_range: Option<String>,
        req_video_layout: Option<String>,
        stable_variant_id: Option<String>,
        video: Option<String>,
        pathway_id: Option<String>,
    ) -> Self {
        let uri = Cow::Owned(uri);
        let codecs = codecs.map(Cow::Owned);
        let supplemental_codecs = supplemental_codecs.map(Cow::Owned);
        let hdcp_level = hdcp_level.map(Cow::Owned);
        let allowed_cpc = allowed_cpc.map(Cow::Owned);
        let video_range = video_range.map(Cow::Owned);
        let req_video_layout = req_video_layout.map(Cow::Owned);
        let stable_variant_id = stable_variant_id.map(Cow::Owned);
        let video = video.map(Cow::Owned);
        let pathway_id = pathway_id.map(Cow::Owned);
        let output_line = Cow::Owned(calculate_line(
            &uri,
            &bandwidth,
            &average_bandwidth,
            &score,
            &codecs,
            &supplemental_codecs,
            &resolution,
            &hdcp_level,
            &allowed_cpc,
            &video_range,
            &req_video_layout,
            &stable_variant_id,
            &video,
            &pathway_id,
        ));
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

    pub fn set_uri(&mut self, uri: String) {
        self.attribute_list.remove(URI);
        self.uri = Cow::Owned(uri);
        self.output_line_is_dirty = true;
    }

    pub fn set_bandwidth(&mut self, bandwidth: u64) {
        self.attribute_list.remove(BANDWIDTH);
        self.bandwidth = bandwidth;
        self.output_line_is_dirty = true;
    }

    pub fn set_average_bandwidth(&mut self, average_bandwidth: Option<u64>) {
        self.attribute_list.remove(AVERAGE_BANDWIDTH);
        self.average_bandwidth = average_bandwidth;
        self.output_line_is_dirty = true;
    }

    pub fn set_score(&mut self, score: Option<f64>) {
        self.attribute_list.remove(SCORE);
        self.score = score;
        self.output_line_is_dirty = true;
    }

    pub fn set_codecs(&mut self, codecs: Option<String>) {
        self.attribute_list.remove(CODECS);
        self.codecs = codecs.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_supplemental_codecs(&mut self, supplemental_codecs: Option<String>) {
        self.attribute_list.remove(SUPPLEMENTAL_CODECS);
        self.supplemental_codecs = supplemental_codecs.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_resolution(&mut self, resolution: Option<DecimalResolution>) {
        self.attribute_list.remove(RESOLUTION);
        self.resolution = resolution;
        self.output_line_is_dirty = true;
    }

    pub fn set_hdcp_level(&mut self, hdcp_level: Option<String>) {
        self.attribute_list.remove(HDCP_LEVEL);
        self.hdcp_level = hdcp_level.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_allowed_cpc(&mut self, allowed_cpc: Option<String>) {
        self.attribute_list.remove(ALLOWED_CPC);
        self.allowed_cpc = allowed_cpc.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_video_range(&mut self, video_range: Option<String>) {
        self.attribute_list.remove(VIDEO_RANGE);
        self.video_range = video_range.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_req_video_layout(&mut self, req_video_layout: Option<String>) {
        self.attribute_list.remove(REQ_VIDEO_LAYOUT);
        self.req_video_layout = req_video_layout.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_stable_variant_id(&mut self, stable_variant_id: Option<String>) {
        self.attribute_list.remove(STABLE_VARIANT_ID);
        self.stable_variant_id = stable_variant_id.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_video(&mut self, video: Option<String>) {
        self.attribute_list.remove(VIDEO);
        self.video = video.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_pathway_id(&mut self, pathway_id: Option<String>) {
        self.attribute_list.remove(PATHWAY_ID);
        self.pathway_id = pathway_id.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.uri(),
            &self.bandwidth(),
            &self.average_bandwidth(),
            &self.score(),
            &self.codecs().map(|x| x.into()),
            &self.supplemental_codecs().map(|x| x.into()),
            &self.resolution(),
            &self.hdcp_level().map(|x| x.into()),
            &self.allowed_cpc().map(|x| x.into()),
            &self.video_range().map(|x| x.into()),
            &self.req_video_layout().map(|x| x.into()),
            &self.stable_variant_id().map(|x| x.into()),
            &self.video().map(|x| x.into()),
            &self.pathway_id().map(|x| x.into()),
        ));
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

fn calculate_line<'a>(
    uri: &str,
    bandwidth: &u64,
    average_bandwidth: &Option<u64>,
    score: &Option<f64>,
    codecs: &Option<Cow<'a, str>>,
    supplemental_codecs: &Option<Cow<'a, str>>,
    resolution: &Option<DecimalResolution>,
    hdcp_level: &Option<Cow<'a, str>>,
    allowed_cpc: &Option<Cow<'a, str>>,
    video_range: &Option<Cow<'a, str>>,
    req_video_layout: &Option<Cow<'a, str>>,
    stable_variant_id: &Option<Cow<'a, str>>,
    video: &Option<Cow<'a, str>>,
    pathway_id: &Option<Cow<'a, str>>,
) -> String {
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
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-I-FRAME-STREAM-INF:URI=\"example.iframe.m3u8\",BANDWIDTH=10000000",
            IFrameStreamInf::new(
                "example.iframe.m3u8".to_string(),
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
            )
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
            ),
            IFrameStreamInf::new(
                "iframe.high.m3u8".to_string(),
                10000000,
                Some(9000000),
                Some(2.0),
                Some("hvc1.2.4.L153.b0,ec-3".to_string()),
                Some("dvh1.08.07/db4h".to_string()),
                Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                Some("TYPE-1".to_string()),
                Some("com.example.drm1:SMART-TV/PC".to_string()),
                Some("PQ".to_string()),
                Some("CH-STEREO,CH-MONO".to_string()),
                Some("1234".to_string()),
                Some("alternate-view".to_string()),
                Some("1234".to_string()),
            )
            .into_inner()
            .value()
        );
    }
}
