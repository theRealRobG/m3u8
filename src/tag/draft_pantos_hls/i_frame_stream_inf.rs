use crate::tag::value::{DecimalResolution, ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.3
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

impl<'a> TryFrom<ParsedTagValue<'a>> for IFrameStreamInf<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI") else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::DecimalInteger(bandwidth)) =
            attribute_list.remove("BANDWIDTH")
        else {
            return Err(super::ValidationError::missing_required_attribute());
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
        Ok(Self {
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
        })
    }
}
