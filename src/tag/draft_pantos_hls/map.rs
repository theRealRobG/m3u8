use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.5
#[derive(Debug, PartialEq)]
pub struct Map<'a> {
    pub uri: &'a str,
    pub byterange: Option<MapByterange>,
}

#[derive(Debug, PartialEq)]
pub struct MapByterange {
    pub length: u64,
    pub offset: u64,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Map<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI") else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let byterange = 'byterange_match: {
            match attribute_list.remove("BYTERANGE") {
                Some(ParsedAttributeValue::QuotedString(byterange_str)) => {
                    let mut parts = byterange_str.split('@');
                    let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                        break 'byterange_match None;
                    };
                    let Some(Ok(offset)) = parts.next().map(str::parse::<u64>) else {
                        break 'byterange_match None;
                    };
                    if parts.next().is_some() {
                        break 'byterange_match None;
                    }
                    Some(MapByterange { length, offset })
                }
                _ => None,
            }
        };
        Ok(Self { uri, byterange })
    }
}
