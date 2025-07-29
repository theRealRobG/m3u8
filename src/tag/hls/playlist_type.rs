use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::{IntoInnerTag, ParsedTag},
        value::{HlsPlaylistType, SemiParsedTagValue},
    },
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.5
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PlaylistType(HlsPlaylistType);

impl TryFrom<ParsedTag<'_>> for PlaylistType {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'_>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::Unparsed(bytes) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        Ok(Self(bytes.try_as_hls_playlist_type()?))
    }
}

impl PlaylistType {
    pub fn new(playlist_type: HlsPlaylistType) -> Self {
        Self(playlist_type)
    }

    pub fn playlist_type(&self) -> HlsPlaylistType {
        self.0
    }

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
