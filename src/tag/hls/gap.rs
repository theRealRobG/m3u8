use crate::{
    error::ValidationError,
    tag::{hls::TagInner, known::ParsedTag},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.7
#[derive(Debug, PartialEq)]
pub struct Gap;

impl TryFrom<ParsedTag<'_>> for Gap {
    type Error = ValidationError;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl Gap {
    pub(crate) fn into_inner(self) -> TagInner<'static> {
        TagInner {
            output_line: Cow::Borrowed("#EXT-X-GAP"),
        }
    }
}
