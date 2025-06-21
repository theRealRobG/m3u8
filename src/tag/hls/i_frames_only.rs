use crate::tag::known::ParsedTag;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.6
#[derive(Debug, PartialEq)]
pub struct IFramesOnly;

impl TryFrom<ParsedTag<'_>> for IFramesOnly {
    type Error = &'static str;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl IFramesOnly {
    pub fn as_str() -> &'static str {
        "#EXT-X-I-FRAMES-ONLY"
    }
}
