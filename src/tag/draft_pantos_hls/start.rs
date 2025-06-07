use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2
#[derive(Debug, PartialEq)]
pub struct Start {
    pub time_offset: f64,
    pub precise: bool,
}

impl TryFrom<ParsedTagValue<'_>> for Start {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(Some(time_offset)) = attribute_list
            .get("TIME-OFFSET")
            .map(ParsedAttributeValue::as_option_f64)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let precise = attribute_list
            .get("PRECISE")
            .map(|v| v.as_option_unquoted_str() == Some("YES"))
            .unwrap_or(false);
        Ok(Self {
            time_offset,
            precise,
        })
    }
}
