use crate::{date::DateTime, tag::value::ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.6
#[derive(Debug, PartialEq)]
pub struct ProgramDateTime(pub DateTime);

impl TryFrom<ParsedTagValue<'_>> for ProgramDateTime {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DateTimeMsec(date_time) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self(date_time))
    }
}
