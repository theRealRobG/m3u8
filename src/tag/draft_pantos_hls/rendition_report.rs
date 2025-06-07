use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.4
#[derive(Debug, PartialEq)]
pub struct RenditionReport<'a> {
    pub uri: &'a str,
    pub last_msn: u64,
    pub last_part: Option<u64>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for RenditionReport<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI") else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::DecimalInteger(last_msn)) =
            attribute_list.remove("LAST-MSN")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let last_part = match attribute_list.remove("LAST-PART") {
            Some(ParsedAttributeValue::DecimalInteger(part)) => Some(part),
            _ => None,
        };
        Ok(Self {
            uri,
            last_msn,
            last_part,
        })
    }
}
