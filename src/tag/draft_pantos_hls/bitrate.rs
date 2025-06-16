use crate::{
    tag::{draft_pantos_hls::TagName, known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.8
#[derive(Debug, PartialEq)]
pub struct Bitrate<'a> {
    bitrate: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for Bitrate<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(rate) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            bitrate: rate,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> Bitrate<'a> {
    pub fn new(bitrate: u64) -> Self {
        Self {
            bitrate,
            output_line: Cow::Owned(calculate_line(bitrate).into_bytes()),
        }
    }

    pub fn bitrate(&self) -> u64 {
        self.bitrate
    }

    pub fn as_str(&self) -> &str {
        match self.output_line {
            Cow::Borrowed(bytes) => split_by_first_lf(str_from(bytes)).parsed,
            Cow::Owned(ref bytes) => str_from(bytes.as_slice()),
        }
    }
}

fn calculate_line(bitrate: u64) -> String {
    format!("#EXT{}:{}", TagName::Bitrate.as_str(), bitrate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_should_be_valid_as_str() {
        let tag = Bitrate::new(10000000);
        assert_eq!("#EXT-X-BITRATE:10000000", tag.as_str())
    }
}
