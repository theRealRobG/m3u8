use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{
        UnknownTag,
        hls::{TagName, into_inner_tag},
    },
};
use std::borrow::Cow;

/// Corresponds to the `#EXT-X-BITRATE` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#section-4.4.4.8>
#[derive(Debug, Clone)]
pub struct Bitrate<'a> {
    bitrate: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl PartialEq for Bitrate<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.bitrate() == other.bitrate()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for Bitrate<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let bitrate = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_decimal_integer()?;
        Ok(Self {
            bitrate,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Bitrate<'a> {
    /// Construct a new `Bitrate` tag.
    pub fn new(bitrate: u64) -> Self {
        Self {
            bitrate,
            output_line: Cow::Owned(calculate_line(bitrate)),
            output_line_is_dirty: false,
        }
    }

    /// Corresponds to the value of the tag (`#EXT-X-BITRATE:<rate>`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn bitrate(&self) -> u64 {
        self.bitrate
    }

    /// Sets the value of the tag.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_bitrate(&mut self, bitrate: u64) {
        self.bitrate = bitrate;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.bitrate()));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Bitrate);

fn calculate_line(bitrate: u64) -> Vec<u8> {
    format!("#EXT{}:{}", TagName::Bitrate.as_str(), bitrate).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn new_should_be_valid_as_str() {
        let tag = Bitrate::new(10000000);
        assert_eq!(b"#EXT-X-BITRATE:10000000", tag.into_inner().value())
    }

    mutation_tests!(Bitrate::new(100), (bitrate, 200, @Attr=":200"));
}
