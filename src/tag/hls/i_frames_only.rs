use crate::{
    error::ValidationError,
    tag::{hls::into_inner_tag, known::ParsedTag},
};

/// Corresponds to the #EXT-X-I-FRAMES-ONLY tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.6>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IFramesOnly;

impl TryFrom<ParsedTag<'_>> for IFramesOnly {
    type Error = ValidationError;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

into_inner_tag!(IFramesOnly @Static b"#EXT-X-I-FRAMES-ONLY");
