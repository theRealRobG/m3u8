use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.6
#[derive(Debug, PartialEq)]
pub struct IFramesOnly;

impl TryFrom<ParsedTagValue<'_>> for IFramesOnly {
    type Error = &'static str;

    fn try_from(_: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
