use crate::tag::known::ParsedTag;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.4
#[derive(Debug, PartialEq)]
pub struct Endlist;

impl TryFrom<ParsedTag<'_>> for Endlist {
    type Error = &'static str;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Endlist {
    pub fn as_str() -> &'static str {
        "#EXT-X-ENDLIST"
    }
}
