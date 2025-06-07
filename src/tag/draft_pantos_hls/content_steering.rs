use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.6
#[derive(Debug, PartialEq)]
pub struct ContentSteering<'a> {
    pub server_uri: &'a str,
    pub pathway_id: Option<&'a str>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for ContentSteering<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(server_uri)) =
            attribute_list.remove("SERVER-URI")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let pathway_id = match attribute_list.remove("PATHWAY-ID") {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        Ok(Self {
            server_uri,
            pathway_id,
        })
    }
}
