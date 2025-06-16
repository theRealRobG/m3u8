use crate::tag::{known::ParsedTag, value::ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.1
#[derive(Debug, PartialEq)]
pub struct Targetduration(u64);

impl TryFrom<ParsedTag<'_>> for Targetduration {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(d) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self(d))
    }
}

impl Targetduration {
    pub fn new(target_duration: u64) -> Self {
        Self(target_duration)
    }

    pub fn target_duration(&self) -> u64 {
        self.0
    }
}
