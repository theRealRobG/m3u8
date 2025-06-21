use crate::tag::{
    known::ParsedTag,
    value::{HlsPlaylistType, ParsedTagValue},
};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.5
#[derive(Debug, PartialEq)]
pub struct PlaylistType(HlsPlaylistType);

impl TryFrom<ParsedTag<'_>> for PlaylistType {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'_>) -> Result<Self, Self::Error> {
        let ParsedTagValue::TypeEnum(t) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self(t))
    }
}

impl PlaylistType {
    pub fn new(playlist_type: HlsPlaylistType) -> Self {
        Self(playlist_type)
    }

    pub fn playlist_type(&self) -> HlsPlaylistType {
        self.0
    }

    pub fn as_str(&self) -> &'static str {
        match self.0 {
            HlsPlaylistType::Event => "#EXT-X-PLAYLIST-TYPE:EVENT",
            HlsPlaylistType::Vod => "#EXT-X-PLAYLIST-TYPE:VOD",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn event_as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-PLAYLIST-TYPE:EVENT",
            PlaylistType(HlsPlaylistType::Event).as_str()
        );
    }

    #[test]
    fn vod_as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-PLAYLIST-TYPE:VOD",
            PlaylistType(HlsPlaylistType::Vod).as_str()
        );
    }
}
