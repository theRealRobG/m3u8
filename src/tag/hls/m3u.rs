use crate::tag::{hls::TagInner, known::ParsedTag};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.1
#[derive(Debug, PartialEq)]
pub struct M3u;

impl TryFrom<ParsedTag<'_>> for M3u {
    type Error = &'static str;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl M3u {
    pub(crate) fn into_inner(self) -> TagInner<'static> {
        TagInner {
            output_line: Cow::Borrowed("#EXTM3U"),
        }
    }
}
