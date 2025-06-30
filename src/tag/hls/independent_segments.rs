use crate::{
    error::ValidationError,
    tag::{hls::TagInner, known::ParsedTag},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.1
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IndependentSegments;

impl TryFrom<ParsedTag<'_>> for IndependentSegments {
    type Error = ValidationError;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl IndependentSegments {
    pub fn into_inner(self) -> TagInner<'static> {
        TagInner {
            output_line: Cow::Borrowed(b"#EXT-X-INDEPENDENT-SEGMENTS"),
        }
    }
}
