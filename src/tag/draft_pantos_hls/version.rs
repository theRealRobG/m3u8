use crate::tag::{known::ParsedTag, value::ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.2
#[derive(Debug, PartialEq)]
pub struct Version(u64);

impl TryFrom<ParsedTag<'_>> for Version {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(version) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self(version))
    }
}

impl Version {
    pub fn new(version: u64) -> Self {
        Self(version)
    }

    pub fn version(&self) -> u64 {
        self.0
    }
}
