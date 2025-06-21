use crate::tag::known::ParsedTag;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.3
#[derive(Debug, PartialEq)]
pub struct Discontinuity;

impl TryFrom<ParsedTag<'_>> for Discontinuity {
    type Error = &'static str;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Discontinuity {
    pub fn as_str() -> &'static str {
        "#EXT-X-DISCONTINUITY"
    }
}
