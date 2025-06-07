use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.4
#[derive(Debug, PartialEq)]
pub struct Endlist;

impl TryFrom<ParsedTagValue<'_>> for Endlist {
    type Error = &'static str;

    fn try_from(_: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
