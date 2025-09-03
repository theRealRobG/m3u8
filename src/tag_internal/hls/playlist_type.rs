use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{HlsPlaylistType, IntoInnerTag, UnknownTag, hls::TagInner},
};
use std::borrow::Cow;

/// Corresponds to the `#EXT-X-PLAYLIST-TYPE` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#section-4.4.3.5>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PlaylistType(HlsPlaylistType);

impl TryFrom<UnknownTag<'_>> for PlaylistType {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'_>) -> Result<Self, Self::Error> {
        Ok(Self(
            tag.value()
                .ok_or(ParseTagValueError::UnexpectedEmpty)?
                .try_as_playlist_type()?,
        ))
    }
}

impl PlaylistType {
    /// Construct a new `PlaylistType` tag.
    pub fn new(playlist_type: HlsPlaylistType) -> Self {
        Self(playlist_type)
    }

    /// Corresponds to the value of the tag (`#EXT-X-PLAYLIST-TYPE:<type-enum>`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn playlist_type(&self) -> HlsPlaylistType {
        self.0
    }

    /// Sets the value of the tag.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_playlist_type(&mut self, playlist_type: HlsPlaylistType) {
        self.0 = playlist_type;
    }
}

impl IntoInnerTag<'static> for PlaylistType {
    fn into_inner(self) -> TagInner<'static> {
        match self.0 {
            HlsPlaylistType::Event => TagInner {
                output_line: Cow::Borrowed(b"#EXT-X-PLAYLIST-TYPE:EVENT"),
            },
            HlsPlaylistType::Vod => TagInner {
                output_line: Cow::Borrowed(b"#EXT-X-PLAYLIST-TYPE:VOD"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::hls::test_macro::mutation_tests;
    use pretty_assertions::assert_eq;

    #[test]
    fn event_as_str_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PLAYLIST-TYPE:EVENT",
            PlaylistType(HlsPlaylistType::Event).into_inner().value()
        );
    }

    #[test]
    fn vod_as_str_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PLAYLIST-TYPE:VOD",
            PlaylistType(HlsPlaylistType::Vod).into_inner().value()
        );
    }

    mutation_tests!(
        PlaylistType(HlsPlaylistType::Vod),
        (playlist_type, HlsPlaylistType::Event, @Attr=":EVENT")
    );
}
