use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{hls::TagInner, known::ParsedTag, value::ParsedTagValue},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.2
#[derive(Debug, Clone)]
pub struct MediaSequence<'a> {
    media_sequence: u64,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for MediaSequence<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.media_sequence() == other.media_sequence()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for MediaSequence<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(d) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        Ok(Self {
            media_sequence: d,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> MediaSequence<'a> {
    pub fn new(media_sequence: u64) -> Self {
        Self {
            media_sequence,
            output_line: Cow::Owned(calculate_line(media_sequence)),
            output_line_is_dirty: false,
        }
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn media_sequence(&self) -> u64 {
        self.media_sequence
    }

    pub fn set_media_sequence(&mut self, media_sequence: u64) {
        self.media_sequence = media_sequence;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.media_sequence()));
        self.output_line_is_dirty = false;
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
            MediaSequence::new(100).into_inner().value()
        );
    }
}
