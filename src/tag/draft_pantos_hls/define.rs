use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.3
#[derive(Debug, PartialEq)]
pub enum Define<'a> {
    Name { name: &'a str, value: &'a str },
    Import(&'a str),
    Queryparam(&'a str),
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Define<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        if let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.get("NAME") {
            if let Some(ParsedAttributeValue::QuotedString(value)) = attribute_list.get("VALUE") {
                Ok(Self::Name { name, value })
            } else {
                Err(super::ValidationError::missing_required_attribute())
            }
        } else if let Some(ParsedAttributeValue::QuotedString(import)) =
            attribute_list.get("IMPORT")
        {
            Ok(Self::Import(import))
        } else if let Some(ParsedAttributeValue::QuotedString(queryparam)) =
            attribute_list.get("QUERYPARAM")
        {
            Ok(Self::Queryparam(queryparam))
        } else {
            Err(super::ValidationError::missing_required_attribute())
        }
    }
}
