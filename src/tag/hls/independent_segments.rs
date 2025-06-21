use crate::tag::known::ParsedTag;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.1
#[derive(Debug, PartialEq)]
pub struct IndependentSegments;

impl TryFrom<ParsedTag<'_>> for IndependentSegments {
    type Error = &'static str;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl IndependentSegments {
    pub fn as_str() -> &'static str {
        "#EXT-X-INDEPENDENT-SEGMENTS"
    }
}
