use crate::{
    tag::{
        known::ParsedTag,
        value::{DecimalResolution, ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.3
#[derive(Debug)]
pub struct IFrameStreamInf<'a> {
    uri: &'a str,
    bandwidth: u64,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    // This needs to exist because the user can construct an IFrameStreamInf with
    // `IFrameStreamInf::new()`, but will pass a `DecimalResolution`, not a `&str`. I can't convert
    // a `DecimalResolution` to a `&str` and so need to store it as is for later use.
    stored_decimal_resolution: Option<DecimalResolution>,
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
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::DecimalInteger(bandwidth)) = attribute_list.get(BANDWIDTH)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            uri,
            bandwidth: *bandwidth,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            stored_decimal_resolution: None,
        })
    }
}

impl<'a> IFrameStreamInf<'a> {
    pub fn new(
        uri: &'a str,
        bandwidth: u64,
        average_bandwidth: Option<u64>,
        score: Option<f64>,
        codecs: Option<&'a str>,
        supplemental_codecs: Option<&'a str>,
        resolution: Option<DecimalResolution>,
        hdcp_level: Option<&'a str>,
        allowed_cpc: Option<&'a str>,
        video_range: Option<&'a str>,
        req_video_layout: Option<&'a str>,
        stable_variant_id: Option<&'a str>,
        video: Option<&'a str>,
        pathway_id: Option<&'a str>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        if let Some(average_bandwidth) = average_bandwidth {
            attribute_list.insert(
                AVERAGE_BANDWIDTH,
                ParsedAttributeValue::DecimalInteger(average_bandwidth),
            );
        }
        if let Some(score) = score {
            attribute_list.insert(
                SCORE,
                ParsedAttributeValue::SignedDecimalFloatingPoint(score),
            );
        }
        if let Some(codecs) = codecs {
            attribute_list.insert(CODECS, ParsedAttributeValue::QuotedString(codecs));
        }
        if let Some(supplemental_codecs) = supplemental_codecs {
            attribute_list.insert(
                SUPPLEMENTAL_CODECS,
                ParsedAttributeValue::QuotedString(supplemental_codecs),
            );
        }
        if let Some(hdcp_level) = hdcp_level {
            attribute_list.insert(HDCP_LEVEL, ParsedAttributeValue::UnquotedString(hdcp_level));
        }
        if let Some(allowed_cpc) = allowed_cpc {
            attribute_list.insert(ALLOWED_CPC, ParsedAttributeValue::QuotedString(allowed_cpc));
        }
        if let Some(video_range) = video_range {
            attribute_list.insert(
                VIDEO_RANGE,
                ParsedAttributeValue::UnquotedString(video_range),
            );
        }
        if let Some(req_video_layout) = req_video_layout {
            attribute_list.insert(
                REQ_VIDEO_LAYOUT,
                ParsedAttributeValue::QuotedString(req_video_layout),
            );
        }
        if let Some(stable_variant_id) = stable_variant_id {
            attribute_list.insert(
                STABLE_VARIANT_ID,
                ParsedAttributeValue::QuotedString(stable_variant_id),
            );
        }
        if let Some(video) = video {
            attribute_list.insert(VIDEO, ParsedAttributeValue::QuotedString(video));
        }
        if let Some(pathway_id) = pathway_id {
            attribute_list.insert(PATHWAY_ID, ParsedAttributeValue::QuotedString(pathway_id));
        }
        Self {
            uri,
            bandwidth,
            attribute_list,
            output_line: Cow::Owned(
                calculate_line(
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
                )
                .into_bytes(),
            ),
            stored_decimal_resolution: resolution,
        }
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn bandwidth(&self) -> u64 {
        self.bandwidth
    }

    pub fn average_bandwidth(&self) -> Option<u64> {
        match self.attribute_list.get(AVERAGE_BANDWIDTH) {
            Some(ParsedAttributeValue::DecimalInteger(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn score(&self) -> Option<f64> {
        match self.attribute_list.get(SCORE) {
            Some(value) => value.as_option_f64(),
            _ => None,
        }
    }

    pub fn codecs(&self) -> Option<&'a str> {
        match self.attribute_list.get(CODECS) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn supplemental_codecs(&self) -> Option<&'a str> {
        match self.attribute_list.get(SUPPLEMENTAL_CODECS) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn resolution(&self) -> Option<DecimalResolution> {
        if let Some(decimal_resolution) = self.stored_decimal_resolution {
            Some(decimal_resolution)
        } else {
            match self.attribute_list.get(RESOLUTION) {
                Some(ParsedAttributeValue::UnquotedString(r)) => {
                    let mut split = r.split('x');
                    let Some(Ok(width)) = split.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    let Some(Ok(height)) = split.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    if split.next().is_some() {
                        return None;
                    };
                    Some(DecimalResolution { width, height })
                }
                _ => None,
            }
        }
    }

    pub fn hdcp_level(&self) -> Option<&'a str> {
        match self.attribute_list.get(HDCP_LEVEL) {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn allowed_cpc(&self) -> Option<&'a str> {
        match self.attribute_list.get(ALLOWED_CPC) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn video_range(&self) -> Option<&'a str> {
        match self.attribute_list.get(VIDEO_RANGE) {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn req_video_layout(&self) -> Option<&'a str> {
        match self.attribute_list.get(REQ_VIDEO_LAYOUT) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn stable_variant_id(&self) -> Option<&'a str> {
        match self.attribute_list.get(STABLE_VARIANT_ID) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn video(&self) -> Option<&'a str> {
        match self.attribute_list.get(VIDEO) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn pathway_id(&self) -> Option<&'a str> {
        match self.attribute_list.get(PATHWAY_ID) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
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

fn calculate_line(
    uri: &str,
    bandwidth: u64,
    average_bandwidth: Option<u64>,
    score: Option<f64>,
    codecs: Option<&str>,
    supplemental_codecs: Option<&str>,
    resolution: Option<DecimalResolution>,
    hdcp_level: Option<&str>,
    allowed_cpc: Option<&str>,
    video_range: Option<&str>,
    req_video_layout: Option<&str>,
    stable_variant_id: Option<&str>,
    video: Option<&str>,
    pathway_id: Option<&str>,
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
                "example.iframe.m3u8",
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
            .as_str()
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
                "iframe.high.m3u8",
                10000000,
                Some(9000000),
                Some(2.0),
                Some("hvc1.2.4.L153.b0,ec-3"),
                Some("dvh1.08.07/db4h"),
                Some(DecimalResolution {
                    width: 3840,
                    height: 2160
                }),
                Some("TYPE-1"),
                Some("com.example.drm1:SMART-TV/PC"),
                Some("PQ"),
                Some("CH-STEREO,CH-MONO"),
                Some("1234"),
                Some("alternate-view"),
                Some("1234"),
            )
            .as_str()
        );
    }
}
