use crate::tag::draft_pantos_hls::TagName;
use std::collections::HashSet;

pub const ALL_KNOWN_HLS_TAG_NAMES: [TagName; 32] = [
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

pub struct ParsingOptions {
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
    pub fn new(hls_tag_names_to_parse: HashSet<TagName>) -> Self {
        Self {
            hls_tag_names_to_parse,
        }
    }

    pub fn is_known_name(&self, name: &'_ str) -> bool {
        let Ok(tag_name) = TagName::try_from(name) else {
            return false;
        };
        self.hls_tag_names_to_parse.contains(&tag_name)
    }
}

#[derive(Default)]
pub struct ParsingOptionsBuilder {
    hls_tag_names_to_parse: HashSet<TagName>,
}

impl ParsingOptionsBuilder {
    pub fn new() -> Self {
        Self {
            hls_tag_names_to_parse: HashSet::default(),
        }
    }

    pub fn build(self) -> ParsingOptions {
        ParsingOptions {
            hls_tag_names_to_parse: self.hls_tag_names_to_parse,
        }
    }

    pub fn with_parsing_for_all_tags(mut self) -> Self {
        self.hls_tag_names_to_parse.extend(ALL_KNOWN_HLS_TAG_NAMES);
        self
    }

    pub fn with_parsing_for_m3u(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::M3u);
        self
    }

    pub fn without_parsing_for_m3u(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::M3u);
        self
    }

    pub fn with_parsing_for_version(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Version);
        self
    }

    pub fn without_parsing_for_version(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Version);
        self
    }

    pub fn with_parsing_for_independent_segments(mut self) -> Self {
        self.hls_tag_names_to_parse
            .insert(TagName::IndependentSegments);
        self
    }

    pub fn without_parsing_for_independent_segments(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::IndependentSegments);
        self
    }

    pub fn with_parsing_for_start(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Start);
        self
    }

    pub fn without_parsing_for_start(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Start);
        self
    }

    pub fn with_parsing_for_define(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Define);
        self
    }

    pub fn without_parsing_for_define(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Define);
        self
    }

    pub fn with_parsing_for_targetduration(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Targetduration);
        self
    }

    pub fn without_parsing_for_targetduration(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Targetduration);
        self
    }

    pub fn with_parsing_for_media_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::MediaSequence);
        self
    }

    pub fn without_parsing_for_media_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::MediaSequence);
        self
    }

    pub fn with_parsing_for_discontinuity_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse
            .insert(TagName::DiscontinuitySequence);
        self
    }

    pub fn without_parsing_for_discontinuity_sequence(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::DiscontinuitySequence);
        self
    }

    pub fn with_parsing_for_endlist(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Endlist);
        self
    }

    pub fn without_parsing_for_endlist(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Endlist);
        self
    }

    pub fn with_parsing_for_playlist_type(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::PlaylistType);
        self
    }

    pub fn without_parsing_for_playlist_type(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::PlaylistType);
        self
    }

    pub fn with_parsing_for_i_frames_only(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::IFramesOnly);
        self
    }

    pub fn without_parsing_for_i_frames_only(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::IFramesOnly);
        self
    }

    pub fn with_parsing_for_part_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::PartInf);
        self
    }

    pub fn without_parsing_for_part_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::PartInf);
        self
    }

    pub fn with_parsing_for_server_control(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::ServerControl);
        self
    }

    pub fn without_parsing_for_server_control(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::ServerControl);
        self
    }

    pub fn with_parsing_for_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Inf);
        self
    }

    pub fn without_parsing_for_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Inf);
        self
    }

    pub fn with_parsing_for_byterange(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Byterange);
        self
    }

    pub fn without_parsing_for_byterange(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Byterange);
        self
    }

    pub fn with_parsing_for_discontinuity(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Discontinuity);
        self
    }

    pub fn without_parsing_for_discontinuity(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Discontinuity);
        self
    }

    pub fn with_parsing_for_key(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Key);
        self
    }

    pub fn without_parsing_for_key(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Key);
        self
    }

    pub fn with_parsing_for_map(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Map);
        self
    }

    pub fn without_parsing_for_map(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Map);
        self
    }

    pub fn with_parsing_for_program_date_time(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::ProgramDateTime);
        self
    }

    pub fn without_parsing_for_program_date_time(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::ProgramDateTime);
        self
    }

    pub fn with_parsing_for_gap(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Gap);
        self
    }

    pub fn without_parsing_for_gap(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Gap);
        self
    }

    pub fn with_parsing_for_bitrate(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Bitrate);
        self
    }

    pub fn without_parsing_for_bitrate(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Bitrate);
        self
    }

    pub fn with_parsing_for_part(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Part);
        self
    }

    pub fn without_parsing_for_part(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Part);
        self
    }

    pub fn with_parsing_for_daterange(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Daterange);
        self
    }

    pub fn without_parsing_for_daterange(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Daterange);
        self
    }

    pub fn with_parsing_for_skip(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Skip);
        self
    }

    pub fn without_parsing_for_skip(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Skip);
        self
    }

    pub fn with_parsing_for_preload_hint(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::PreloadHint);
        self
    }

    pub fn without_parsing_for_preload_hint(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::PreloadHint);
        self
    }

    pub fn with_parsing_for_rendition_report(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::RenditionReport);
        self
    }

    pub fn without_parsing_for_rendition_report(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::RenditionReport);
        self
    }

    pub fn with_parsing_for_media(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::Media);
        self
    }

    pub fn without_parsing_for_media(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::Media);
        self
    }

    pub fn with_parsing_for_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::StreamInf);
        self
    }

    pub fn without_parsing_for_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::StreamInf);
        self
    }

    pub fn with_parsing_for_i_frame_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::IFrameStreamInf);
        self
    }

    pub fn without_parsing_for_i_frame_stream_inf(mut self) -> Self {
        self.hls_tag_names_to_parse
            .remove(&TagName::IFrameStreamInf);
        self
    }

    pub fn with_parsing_for_session_data(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::SessionData);
        self
    }

    pub fn without_parsing_for_session_data(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::SessionData);
        self
    }

    pub fn with_parsing_for_session_key(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::SessionKey);
        self
    }

    pub fn without_parsing_for_session_key(mut self) -> Self {
        self.hls_tag_names_to_parse.remove(&TagName::SessionKey);
        self
    }

    pub fn with_parsing_for_content_steering(mut self) -> Self {
        self.hls_tag_names_to_parse.insert(TagName::ContentSteering);
        self
    }

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
