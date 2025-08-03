use crate::{
    error::ValidationError,
    tag::{hls::into_inner_tag, known::ParsedTag},
};

/// Corresponds to the #EXTM3U tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.1>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct M3u;

impl TryFrom<ParsedTag<'_>> for M3u {
    type Error = ValidationError;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

into_inner_tag!(M3u @Static b"#EXTM3U");
