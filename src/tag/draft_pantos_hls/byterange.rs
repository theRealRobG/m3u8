use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.2
#[derive(Debug, PartialEq)]
pub struct Byterange {
    pub length: u64,
    pub offset: Option<u64>,
}

impl TryFrom<ParsedTagValue<'_>> for Byterange {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        match value {
            ParsedTagValue::DecimalInteger(length) => Ok(Self {
                length,
                offset: None,
            }),
            ParsedTagValue::DecimalIntegerRange(length, offset) => Ok(Self {
                length,
                offset: Some(offset),
            }),
            _ => Err(super::ValidationError::unexpected_value_type()),
        }
    }
}
