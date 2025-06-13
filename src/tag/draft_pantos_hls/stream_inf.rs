use crate::tag::value::{DecimalResolution, ParsedAttributeValue, ParsedTagValue};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.2
#[derive(Debug)]
pub struct StreamInf<'a> {
    bandwidth: u64,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
    // This needs to exist because the user can construct an IFrameStreamInf with
    // `IFrameStreamInf::new()`, but will pass a `DecimalResolution`, not a `&str`. I can't convert
    // a `DecimalResolution` to a `&str` and so need to store it as is for later use.
    stored_decimal_resolution: Option<DecimalResolution>,
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

impl<'a> TryFrom<ParsedTagValue<'a>> for StreamInf<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::DecimalInteger(bandwidth)) = attribute_list.get(BANDWIDTH)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            bandwidth: *bandwidth,
            attribute_list,
            stored_decimal_resolution: None,
        })
    }
}

impl<'a> StreamInf<'a> {
    pub fn new(
        bandwidth: u64,
        average_bandwidth: Option<u64>,
        score: Option<f64>,
        codecs: Option<&'a str>,
        supplemental_codecs: Option<&'a str>,
        resolution: Option<DecimalResolution>,
        frame_rate: Option<f64>,
        hdcp_level: Option<&'a str>,
        allowed_cpc: Option<&'a str>,
        video_range: Option<&'a str>,
        req_video_layout: Option<&'a str>,
        stable_variant_id: Option<&'a str>,
        audio: Option<&'a str>,
        video: Option<&'a str>,
        subtitles: Option<&'a str>,
        closed_captions: Option<&'a str>,
        pathway_id: Option<&'a str>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(BANDWIDTH, ParsedAttributeValue::DecimalInteger(bandwidth));
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
        if let Some(frame_rate) = frame_rate {
            attribute_list.insert(
                FRAME_RATE,
                ParsedAttributeValue::SignedDecimalFloatingPoint(frame_rate),
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
        if let Some(audio) = audio {
            attribute_list.insert(AUDIO, ParsedAttributeValue::QuotedString(audio));
        }
        if let Some(video) = video {
            attribute_list.insert(VIDEO, ParsedAttributeValue::QuotedString(video));
        }
        if let Some(subtitles) = subtitles {
            attribute_list.insert(SUBTITLES, ParsedAttributeValue::QuotedString(subtitles));
        }
        if let Some(closed_captions) = closed_captions {
            attribute_list.insert(
                CLOSED_CAPTIONS,
                ParsedAttributeValue::QuotedString(closed_captions),
            );
        }
        if let Some(pathway_id) = pathway_id {
            attribute_list.insert(PATHWAY_ID, ParsedAttributeValue::QuotedString(pathway_id));
        }
        Self {
            bandwidth,
            attribute_list,
            stored_decimal_resolution: resolution,
        }
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
        if let Some(resolution) = self.stored_decimal_resolution {
            Some(resolution)
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

    pub fn frame_rate(&self) -> Option<f64> {
        match self.attribute_list.get(FRAME_RATE) {
            Some(v) => v.as_option_f64(),
            _ => None,
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

    pub fn audio(&self) -> Option<&'a str> {
        match self.attribute_list.get(AUDIO) {
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

    pub fn subtitles(&self) -> Option<&'a str> {
        match self.attribute_list.get(SUBTITLES) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn closed_captions(&self) -> Option<&'a str> {
        match self.attribute_list.get(CLOSED_CAPTIONS) {
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
}

const BANDWIDTH: &'static str = "BANDWIDTH";
const AVERAGE_BANDWIDTH: &'static str = "AVERAGE-BANDWIDTH";
const SCORE: &'static str = "SCORE";
const CODECS: &'static str = "CODECS";
const SUPPLEMENTAL_CODECS: &'static str = "SUPPLEMENTAL-CODECS";
const RESOLUTION: &'static str = "RESOLUTION";
const FRAME_RATE: &'static str = "FRAME-RATE";
const HDCP_LEVEL: &'static str = "HDCP-LEVEL";
const ALLOWED_CPC: &'static str = "ALLOWED-CPC";
const VIDEO_RANGE: &'static str = "VIDEO-RANGE";
const REQ_VIDEO_LAYOUT: &'static str = "REQ-VIDEO-LAYOUT";
const STABLE_VARIANT_ID: &'static str = "STABLE-VARIANT-ID";
const AUDIO: &'static str = "AUDIO";
const VIDEO: &'static str = "VIDEO";
const SUBTITLES: &'static str = "SUBTITLES";
const CLOSED_CAPTIONS: &'static str = "CLOSED-CAPTIONS";
const PATHWAY_ID: &'static str = "PATHWAY-ID";
