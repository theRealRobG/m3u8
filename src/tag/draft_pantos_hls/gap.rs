use crate::tag::known::ParsedTag;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.7
#[derive(Debug, PartialEq)]
pub struct Gap;

impl TryFrom<ParsedTag<'_>> for Gap {
    type Error = &'static str;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Gap {
    pub fn as_str() -> &'static str {
        "#EXT-X-GAP"
    }
}
