use crate::tag::value::{HlsPlaylistType, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.5
#[derive(Debug, PartialEq)]
pub struct PlaylistType(pub HlsPlaylistType);

impl TryFrom<ParsedTagValue<'_>> for PlaylistType {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::TypeEnum(t) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self(t))
    }
}
