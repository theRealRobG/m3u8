use crate::{
    tag::{known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.2
#[derive(Debug, PartialEq)]
pub struct MediaSequence<'a> {
    media_sequence: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for MediaSequence<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(d) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            media_sequence: d,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> MediaSequence<'a> {
    pub fn new(media_sequence: u64) -> Self {
        Self {
            media_sequence,
            output_line: Cow::Owned(calculate_line(media_sequence).into_bytes()),
        }
    }

    pub fn media_sequence(&self) -> u64 {
        self.media_sequence
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

fn calculate_line(media_sequence: u64) -> String {
    format!("#EXT-X-MEDIA-SEQUENCE:{media_sequence}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            "#EXT-X-MEDIA-SEQUENCE:100",
            MediaSequence::new(100).as_str()
        );
    }
}
