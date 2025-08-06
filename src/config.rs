//! Configuration for reading HLS lines
//!
//! This module provides configuration options for [`crate::Reader`] along with helper API (such as
//! [`ParsingOptionsBuilder`]) for constructing config options.

use crate::tag::hls::TagName;
use std::collections::HashSet;

const ALL_KNOWN_HLS_TAG_NAMES: [TagName; 32] = [
    TagName::M3u,
    TagName::Version,
    TagName::IndependentSegments,
    TagName::Start,
    TagName::Define,
    TagName::Targetduration,
    TagName::MediaSequence,
    TagName::DiscontinuitySequence,
    TagName::Endlist,
    TagName::PlaylistType,
    TagName::IFramesOnly,
    TagName::PartInf,
    TagName::ServerControl,
    TagName::Inf,
    TagName::Byterange,
    TagName::Discontinuity,
    TagName::Key,
    TagName::Map,
    TagName::ProgramDateTime,
    TagName::Gap,
    TagName::Bitrate,
    TagName::Part,
    TagName::Daterange,
    TagName::Skip,
    TagName::PreloadHint,
    TagName::RenditionReport,
    TagName::Media,
    TagName::StreamInf,
    TagName::IFrameStreamInf,
    TagName::SessionData,
    TagName::SessionKey,
    TagName::ContentSteering,
];

/// Parsing options for the [`crate::Reader`] to follow.
///
/// For now the only option that can be set is `hls_tag_names_to_parse`. For convenience, a builder
/// struct [ParsingOptionsBuilder] has been provided, to make constructing this struct easier.
#[derive(Debug, PartialEq, Clone)]
pub struct ParsingOptions {
    /// The tag names that will be parsed by the [`crate::Reader`].
    ///
    /// HLS tags that are not included in this list will be parsed as
    /// [`crate::line::HlsLine::UnknownTag`].
    pub hls_tag_names_to_parse: HashSet<TagName>,
}

impl Default for ParsingOptions {
    fn default() -> Self {
        Self {
            hls_tag_names_to_parse: HashSet::from(ALL_KNOWN_HLS_TAG_NAMES),
        }
    }
}

impl ParsingOptions {
    /// Construct a new `ParsingOptions` using the provided set of tag names to parse.
    pub fn new(hls_tag_names_to_parse: HashSet<TagName>) -> Self {
        Self {
            hls_tag_names_to_parse,
        }
    }

    pub(crate) fn is_known_name(&self, name: &'_ str) -> bool {
        let Ok(tag_name) = TagName::try_from(name) else {
            return false;
        };
        self.hls_tag_names_to_parse.contains(&tag_name)
    }
}

/// A builder type to provide convenience for constructing [`ParsingOptions`].
#[derive(Default, Debug)]
pub struct ParsingOptionsBuilder {
    hls_tag_names_to_parse: HashSet<TagName>,
}

impl ParsingOptionsBuilder {
    /// Instantiate the builder.
    pub fn new() -> Self {
        Self {
            hls_tag_names_to_parse: HashSet::default(),
        }
    }

    /// Finish building, consume the builder, and generate the [`ParsingOptions`].
    pub fn build(self) -> ParsingOptions {
        ParsingOptions {
            hls_tag_names_to_parse: self.hls_tag_names_to_parse,
        }
    }

    /// Include parsing of all known HLS tags.
    pub fn with_parsing_for_all_tags(mut self) -> Self {
        self.hls_tag_names_to_parse.extend(ALL_KNOWN_HLS_TAG_NAMES);
        self
    }

    /// Include parsing of [`crate::tag::hls::M3u`].
    pub fn with_parsing_for_m3u(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::M3u);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::M3u`].
    pub fn without_parsing_for_m3u(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::M3u);
        self
    }

    /// Include parsing of [`crate::tag::hls::Version`].
    pub fn with_parsing_for_version(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Version);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Version`].
    pub fn without_parsing_for_version(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Version);
        self
    }

    /// Include parsing of [`crate::tag::hls::IndependentSegments`].
    pub fn with_parsing_for_independent_segments(mut self) -> Self {
        self.hls_tag_names_to_parse
            .insert(TagName::IndependentSegments);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::IndependentSegments`].
    pub fn without_parsing_for_independent_segments(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::IndependentSegments);
        self
    }

    /// Include parsing of [`crate::tag::hls::Start`].
    pub fn with_parsing_for_start(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Start);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Start`].
    pub fn without_parsing_for_start(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Start);
        self
    }

    /// Include parsing of [`crate::tag::hls::Define`].
    pub fn with_parsing_for_define(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Define);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Define`].
    pub fn without_parsing_for_define(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Define);
        self
    }

    /// Include parsing of [`crate::tag::hls::Targetduration`].
    pub fn with_parsing_for_targetduration(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Targetduration);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Targetduration`].
    pub fn without_parsing_for_targetduration(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Targetduration);
        self
    }

    /// Include parsing of [`crate::tag::hls::MediaSequence`].
    pub fn with_parsing_for_media_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::MediaSequence);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::MediaSequence`].
    pub fn without_parsing_for_media_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::MediaSequence);
        self
    }

    /// Include parsing of [`crate::tag::hls::DiscontinuitySequence`].
    pub fn with_parsing_for_discontinuity_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse
            .insert(TagName::DiscontinuitySequence);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::DiscontinuitySequence`].
    pub fn without_parsing_for_discontinuity_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::DiscontinuitySequence);
        self
    }

    /// Include parsing of [`crate::tag::hls::Endlist`].
    pub fn with_parsing_for_endlist(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Endlist);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Endlist`].
    pub fn without_parsing_for_endlist(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Endlist);
        self
    }

    /// Include parsing of [`crate::tag::hls::PlaylistType`].
    pub fn with_parsing_for_playlist_type(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::PlaylistType);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::PlaylistType`].
    pub fn without_parsing_for_playlist_type(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::PlaylistType);
        self
    }

    /// Include parsing of [`crate::tag::hls::IFramesOnly`].
    pub fn with_parsing_for_i_frames_only(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::IFramesOnly);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::IFramesOnly`].
    pub fn without_parsing_for_i_frames_only(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::IFramesOnly);
        self
    }

    /// Include parsing of [`crate::tag::hls::PartInf`].
    pub fn with_parsing_for_part_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::PartInf);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::PartInf`].
    pub fn without_parsing_for_part_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::PartInf);
        self
    }

    /// Include parsing of [`crate::tag::hls::ServerControl`].
    pub fn with_parsing_for_server_control(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::ServerControl);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::ServerControl`].
    pub fn without_parsing_for_server_control(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::ServerControl);
        self
    }

    /// Include parsing of [`crate::tag::hls::Inf`].
    pub fn with_parsing_for_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Inf);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Inf`].
    pub fn without_parsing_for_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Inf);
        self
    }

    /// Include parsing of [`crate::tag::hls::Byterange`].
    pub fn with_parsing_for_byterange(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Byterange);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Byterange`].
    pub fn without_parsing_for_byterange(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Byterange);
        self
    }

    /// Include parsing of [`crate::tag::hls::Discontinuity`].
    pub fn with_parsing_for_discontinuity(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Discontinuity);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Discontinuity`].
    pub fn without_parsing_for_discontinuity(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Discontinuity);
        self
    }

    /// Include parsing of [`crate::tag::hls::Key`].
    pub fn with_parsing_for_key(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Key);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Key`].
    pub fn without_parsing_for_key(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Key);
        self
    }

    /// Include parsing of [`crate::tag::hls::Map`].
    pub fn with_parsing_for_map(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Map);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Map`].
    pub fn without_parsing_for_map(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Map);
        self
    }

    /// Include parsing of [`crate::tag::hls::ProgramDateTime`].
    pub fn with_parsing_for_program_date_time(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::ProgramDateTime);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::ProgramDateTime`].
    pub fn without_parsing_for_program_date_time(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::ProgramDateTime);
        self
    }

    /// Include parsing of [`crate::tag::hls::Gap`].
    pub fn with_parsing_for_gap(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Gap);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Gap`].
    pub fn without_parsing_for_gap(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Gap);
        self
    }

    /// Include parsing of [`crate::tag::hls::Bitrate`].
    pub fn with_parsing_for_bitrate(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Bitrate);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Bitrate`].
    pub fn without_parsing_for_bitrate(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Bitrate);
        self
    }

    /// Include parsing of [`crate::tag::hls::Part`].
    pub fn with_parsing_for_part(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Part);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Part`].
    pub fn without_parsing_for_part(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Part);
        self
    }

    /// Include parsing of [`crate::tag::hls::Daterange`].
    pub fn with_parsing_for_daterange(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Daterange);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Daterange`].
    pub fn without_parsing_for_daterange(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Daterange);
        self
    }

    /// Include parsing of [`crate::tag::hls::Skip`].
    pub fn with_parsing_for_skip(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Skip);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Skip`].
    pub fn without_parsing_for_skip(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Skip);
        self
    }

    /// Include parsing of [`crate::tag::hls::PreloadHint`].
    pub fn with_parsing_for_preload_hint(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::PreloadHint);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::PreloadHint`].
    pub fn without_parsing_for_preload_hint(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::PreloadHint);
        self
    }

    /// Include parsing of [`crate::tag::hls::RenditionReport`].
    pub fn with_parsing_for_rendition_report(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::RenditionReport);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::RenditionReport`].
    pub fn without_parsing_for_rendition_report(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::RenditionReport);
        self
    }

    /// Include parsing of [`crate::tag::hls::Media`].
    pub fn with_parsing_for_media(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Media);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::Media`].
    pub fn without_parsing_for_media(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Media);
        self
    }

    /// Include parsing of [`crate::tag::hls::StreamInf`].
    pub fn with_parsing_for_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::StreamInf);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::StreamInf`].
    pub fn without_parsing_for_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::StreamInf);
        self
    }

    /// Include parsing of [`crate::tag::hls::IFrameStreamInf`].
    pub fn with_parsing_for_i_frame_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::IFrameStreamInf);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::IFrameStreamInf`].
    pub fn without_parsing_for_i_frame_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::IFrameStreamInf);
        self
    }

    /// Include parsing of [`crate::tag::hls::SessionData`].
    pub fn with_parsing_for_session_data(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::SessionData);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::SessionData`].
    pub fn without_parsing_for_session_data(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::SessionData);
        self
    }

    /// Include parsing of [`crate::tag::hls::SessionKey`].
    pub fn with_parsing_for_session_key(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::SessionKey);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::SessionKey`].
    pub fn without_parsing_for_session_key(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::SessionKey);
        self
    }

    /// Include parsing of [`crate::tag::hls::ContentSteering`].
    pub fn with_parsing_for_content_steering(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::ContentSteering);
        self
    }

    /// Ignore parsing of [`crate::tag::hls::ContentSteering`].
    pub fn without_parsing_for_content_steering(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::ContentSteering);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn builder_with_all_tag_names() {
        let options = ParsingOptionsBuilder::new()
            .with_parsing_for_all_tags()
            .build();
        let mut count = 0;
        for name in options.hls_tag_names_to_parse {
            count += 1;
            assert!(ALL_KNOWN_HLS_TAG_NAMES.contains(&name));
        }
        assert_eq!(32, count);
    }

    #[test]
    fn builder_with_some_tag_names() {
        let options = ParsingOptionsBuilder::new()
            .with_parsing_for_bitrate()
            .with_parsing_for_byterange()
            .with_parsing_for_daterange()
            .build();
        assert!(options.hls_tag_names_to_parse.contains(&TagName::Bitrate));
        assert!(options.hls_tag_names_to_parse.contains(&TagName::Byterange));
        assert!(options.hls_tag_names_to_parse.contains(&TagName::Daterange));
        assert_eq!(3, options.hls_tag_names_to_parse.len());
    }

    #[test]
    fn builder_with_removing_some_tag_names() {
        let options = ParsingOptionsBuilder::new()
            .with_parsing_for_all_tags()
            .without_parsing_for_define()
            .without_parsing_for_i_frame_stream_inf();
        assert!(!options.hls_tag_names_to_parse.contains(&TagName::Define));
        assert!(
            !options
                .hls_tag_names_to_parse
                .contains(&TagName::IFrameStreamInf)
        );
        assert_eq!(30, options.hls_tag_names_to_parse.len());
    }
}
