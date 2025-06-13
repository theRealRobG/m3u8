use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.2
#[derive(Debug, PartialEq)]
pub struct Byterange {
    length: u64,
    offset: Option<u64>,
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

impl Byterange {
    pub fn new(length: u64, offset: Option<u64>) -> Self {
        Self { length, offset }
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn offset(&self) -> Option<u64> {
        self.offset
    }
}
