use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.9
#[derive(Debug, PartialEq)]
pub struct Part<'a> {
    pub uri: &'a str,
    pub duration: f64,
    pub independent: bool,
    pub byterange: Option<PartByterange>,
    pub gap: bool,
}
#[derive(Debug, PartialEq)]
pub struct PartByterange {
    pub length: u64,
    pub offset: Option<u64>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Part<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.remove("URI") else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(duration) = (match attribute_list.remove("DURATION") {
            Some(a) => a.as_option_f64(),
            _ => None,
        }) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let independent = matches!(
            attribute_list.remove("INDEPENDENT"),
            Some(ParsedAttributeValue::UnquotedString("YES"))
        );
        let byterange = 'byterange_match: {
            match attribute_list.remove("BYTERANGE") {
                Some(ParsedAttributeValue::QuotedString(range)) => {
                    let mut parts = range.split('@');
                    let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                        break 'byterange_match None;
                    };
                    let offset = match parts.next().map(str::parse::<u64>) {
                        Some(Ok(d)) => Some(d),
                        None => None,
                        Some(Err(_)) => break 'byterange_match None,
                    };
                    if parts.next().is_some() {
                        break 'byterange_match None;
                    }
                    Some(PartByterange { length, offset })
                }
                _ => None,
            }
        };
        let gap = matches!(
            attribute_list.remove("GAP"),
            Some(ParsedAttributeValue::UnquotedString("YES"))
        );
        Ok(Self {
            uri,
            duration,
            independent,
            byterange,
            gap,
        })
    }
}
