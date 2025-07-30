use crate::{
    error::ValidationError,
    tag::{hls::into_inner_tag, known::ParsedTag},
};

/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.4>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Endlist;

impl TryFrom<ParsedTag<'_>> for Endlist {
    type Error = ValidationError;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

into_inner_tag!(Endlist @Static b"#EXT-X-ENDLIST");
