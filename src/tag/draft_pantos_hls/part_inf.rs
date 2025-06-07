use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.7
#[derive(Debug, PartialEq)]
pub struct PartInf {
    pub part_target: f64,
}

impl TryFrom<ParsedTagValue<'_>> for PartInf {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(Some(part_target)) = attribute_list
            .get("PART-TARGET")
            .map(ParsedAttributeValue::as_option_f64)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self { part_target })
    }
}
