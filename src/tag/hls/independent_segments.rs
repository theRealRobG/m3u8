use crate::{
    error::ValidationError,
    tag::{hls::into_inner_tag, known::ParsedTag},
};

/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.1>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IndependentSegments;

impl TryFrom<ParsedTag<'_>> for IndependentSegments {
    type Error = ValidationError;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

into_inner_tag!(IndependentSegments @Static b"#EXT-X-INDEPENDENT-SEGMENTS");
