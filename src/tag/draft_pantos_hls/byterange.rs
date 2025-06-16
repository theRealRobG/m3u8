use crate::{
    tag::{draft_pantos_hls::TagName, known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.2
#[derive(Debug, PartialEq)]
pub struct Byterange<'a> {
    length: u64,
    offset: Option<u64>,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for Byterange<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        match tag.value {
            ParsedTagValue::DecimalInteger(length) => Ok(Self {
                length,
                offset: None,
                output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            }),
            ParsedTagValue::DecimalIntegerRange(length, offset) => Ok(Self {
                length,
                offset: Some(offset),
                output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            }),
            _ => Err(super::ValidationError::unexpected_value_type()),
        }
    }
}

impl<'a> Byterange<'a> {
    pub fn new(length: u64, offset: Option<u64>) -> Self {
        Self {
            length,
            offset,
            output_line: Cow::Owned(calculate_line(length, offset).into_bytes()),
        }
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn offset(&self) -> Option<u64> {
        self.offset
    }

    pub fn as_str(&self) -> &str {
        match self.output_line {
            Cow::Borrowed(bytes) => split_by_first_lf(str_from(bytes)).parsed,
            Cow::Owned(ref bytes) => str_from(bytes.as_slice()),
        }
    }
}

fn calculate_line(length: u64, offset: Option<u64>) -> String {
    let mut line = format!("#EXT{}:{}", TagName::Byterange.as_str(), length);
    if let Some(offset) = offset {
        line.push_str(format!("@{}", offset).as_str());
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_with_no_offset_should_be_valid_line() {
        let tag = Byterange::new(1024, None);
        assert_eq!("#EXT-X-BYTERANGE:1024", tag.as_str());
    }

    #[test]
    fn new_with_offset_should_be_valid_line() {
        let tag = Byterange::new(1024, Some(512));
        assert_eq!("#EXT-X-BYTERANGE:1024@512", tag.as_str());
    }
}
