use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.4
#[derive(Debug, PartialEq)]
pub struct SessionData<'a> {
    pub data_id: &'a str,
    pub value: Option<&'a str>,
    pub uri: Option<&'a str>,
    pub format: &'a str,
    pub language: Option<&'a str>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for SessionData<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(data_id)) = attribute_list.remove("DATA-ID")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let value = match attribute_list.remove("VALUE") {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        let uri = match attribute_list.remove("URI") {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        let format = match attribute_list.remove("FORMAT") {
            Some(ParsedAttributeValue::UnquotedString(s)) => s,
            _ => "JSON",
        };
        let language = match attribute_list.remove("LANGUAGE") {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        Ok(Self {
            data_id,
            value,
            uri,
            format,
            language,
        })
    }
}
