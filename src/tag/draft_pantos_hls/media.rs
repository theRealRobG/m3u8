use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.1
#[derive(Debug, PartialEq)]
pub struct Media<'a> {
    pub media_type: &'a str,
    pub uri: Option<&'a str>,
    pub group_id: &'a str,
    pub language: Option<&'a str>,
    pub assoc_language: Option<&'a str>,
    pub name: &'a str,
    pub stable_rendition_id: Option<&'a str>,
    pub default: bool,
    pub autoselect: bool,
    pub forced: bool,
    pub instream_id: Option<&'a str>,
    pub bit_depth: Option<u64>,
    pub sample_rate: Option<u64>,
    pub characteristics: Option<&'a str>,
    pub channels: Option<&'a str>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Media<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(media_type)) = attribute_list.remove("TYPE")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let uri = match attribute_list.remove("URI") {
            Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
            _ => None,
        };
        let Some(ParsedAttributeValue::QuotedString(group_id)) = attribute_list.remove("GROUP-ID")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let language = match attribute_list.remove("LANGUAGE") {
            Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
            _ => None,
        };
        let assoc_language = match attribute_list.remove("ASSOC-LANGUAGE") {
            Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
            _ => None,
        };
        let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.remove("NAME") else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let stable_rendition_id = match attribute_list.remove("STABLE-RENDITION-ID") {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        let default = matches!(
            attribute_list.remove("DEFAULT"),
            Some(ParsedAttributeValue::UnquotedString("YES"))
        );
        let autoselect = matches!(
            attribute_list.remove("AUTOSELECT"),
            Some(ParsedAttributeValue::UnquotedString("YES"))
        );
        let forced = matches!(
            attribute_list.remove("FORCED"),
            Some(ParsedAttributeValue::UnquotedString("YES"))
        );
        let instream_id = match attribute_list.remove("INSTREAM-ID") {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        let bit_depth = match attribute_list.remove("BIT-DEPTH") {
            Some(ParsedAttributeValue::DecimalInteger(d)) => Some(d),
            _ => None,
        };
        let sample_rate = match attribute_list.remove("SAMPLE-RATE") {
            Some(ParsedAttributeValue::DecimalInteger(rate)) => Some(rate),
            _ => None,
        };
        let characteristics = match attribute_list.remove("CHARACTERISTICS") {
            Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
            _ => None,
        };
        let channels = match attribute_list.remove("CHANNELS") {
            Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
            _ => None,
        };
        Ok(Self {
            media_type,
            uri,
            group_id,
            language,
            assoc_language,
            name,
            stable_rendition_id,
            default,
            autoselect,
            forced,
            instream_id,
            bit_depth,
            sample_rate,
            characteristics,
            channels,
        })
    }
}
