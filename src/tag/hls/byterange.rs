use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{TagInner, TagName},
        known::ParsedTag,
        value::SemiParsedTagValue,
    },
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.2
#[derive(Debug, Clone)]
pub struct Byterange<'a> {
    length: u64,
    offset: Option<u64>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl PartialEq for Byterange<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.length() == other.length() && self.offset() == other.offset()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Byterange<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        match tag.value {
            SemiParsedTagValue::Unparsed(bytes) => {
                let (length, offset) = bytes.try_as_decimal_integer_range()?;
                Ok(Self {
                    length,
                    offset,
                    output_line: Cow::Borrowed(tag.original_input),
                    output_line_is_dirty: false,
                })
            }
            _ => Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            )),
        }
    }
}

impl<'a> Byterange<'a> {
    pub fn new(length: u64, offset: Option<u64>) -> Self {
        Self {
            length,
            offset,
            output_line: Cow::Owned(calculate_line(length, offset)),
            output_line_is_dirty: false,
        }
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn offset(&self) -> Option<u64> {
        self.offset
    }

    pub fn set_length(&mut self, length: u64) {
        self.length = length;
        self.output_line_is_dirty = true;
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.offset = Some(offset);
        self.output_line_is_dirty = true;
    }

    pub fn unset_offset(&mut self) {
        self.offset = None;
        self.output_line_is_dirty = true;
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.length(), self.offset()));
        self.output_line_is_dirty = false;
    }
}

fn calculate_line(length: u64, offset: Option<u64>) -> Vec<u8> {
    let mut line = format!("#EXT{}:{}", TagName::Byterange.as_str(), length);
    if let Some(offset) = offset {
        line.push_str(format!("@{}", offset).as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::hls::test_macro::mutation_tests;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_with_no_offset_should_be_valid_line() {
        let tag = Byterange::new(1024, None);
        assert_eq!(b"#EXT-X-BYTERANGE:1024", tag.into_inner().value());
    }

    #[test]
    fn new_with_offset_should_be_valid_line() {
        let tag = Byterange::new(1024, Some(512));
        assert_eq!(b"#EXT-X-BYTERANGE:1024@512", tag.into_inner().value());
    }

    mutation_tests!(
        Byterange::new(1024, Some(512)),
        (length, 100, @Attr=":100"),
        (offset, @Option 200, @Attr="@200")
    );
}
