use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.8
#[derive(Debug, PartialEq)]
pub struct Bitrate(pub u64);

impl TryFrom<ParsedTagValue<'_>> for Bitrate {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(rate) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self(rate))
    }
}
