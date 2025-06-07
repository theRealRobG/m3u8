use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.4
#[derive(Debug, PartialEq)]
pub struct Key<'a> {
    pub method: &'a str,
    pub uri: Option<&'a str>,
    pub iv: Option<&'a str>,
    pub keyformat: &'a str,
    pub keyformatversions: Option<&'a str>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Key<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(method)) = attribute_list.remove("METHOD")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let uri = match attribute_list.remove("URI") {
            Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
            _ => None,
        };
        let iv = match attribute_list.remove("IV") {
            Some(ParsedAttributeValue::UnquotedString(iv)) => Some(iv),
            _ => None,
        };
        let keyformat = match attribute_list.remove("KEYFORMAT") {
            Some(ParsedAttributeValue::QuotedString(keyformat)) => keyformat,
            _ => "identity",
        };
        let keyformatversions = match attribute_list.remove("KEYFORMATVERSIONS") {
            Some(ParsedAttributeValue::QuotedString(keyformatversions)) => Some(keyformatversions),
            _ => None,
        };
        Ok(Self {
            method,
            uri,
            iv,
            keyformat,
            keyformatversions,
        })
    }
}
