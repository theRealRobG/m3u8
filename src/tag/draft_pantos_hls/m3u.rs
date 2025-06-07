use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.1
#[derive(Debug, PartialEq)]
pub struct M3u;

impl TryFrom<ParsedTagValue<'_>> for M3u {
    type Error = &'static str;

    fn try_from(_: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
