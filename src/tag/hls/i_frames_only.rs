use crate::tag::{hls::TagInner, known::ParsedTag};
use std::borrow::Cow;

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
    pub(crate) fn into_inner(self) -> TagInner<'static> {
        TagInner {
            output_line: Cow::Borrowed("#EXT-X-I-FRAMES-ONLY"),
        }
    }
}
