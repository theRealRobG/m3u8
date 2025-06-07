use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.3
#[derive(Debug, PartialEq)]
pub struct PreloadHint<'a> {
    pub hint_type: &'a str,
    pub uri: &'a str,
    pub byterange_start: u64,
    pub byterange_length: Option<u64>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for PreloadHint<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(hint_type)) = attribute_list.remove("TYPE")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI") else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let byterange_start = match attribute_list.remove("BYTERANGE-START") {
            Some(ParsedAttributeValue::DecimalInteger(start)) => start,
            _ => 0,
        };
        let byterange_length = match attribute_list.remove("BYTERANGE-LENGTH") {
            Some(ParsedAttributeValue::DecimalInteger(length)) => Some(length),
            _ => None,
        };
        Ok(Self {
            hint_type,
            uri,
            byterange_start,
            byterange_length,
        })
    }
}
