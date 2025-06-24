use crate::tag::{hls::TagInner, known::ParsedTag, value::ParsedTagValue};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.2
#[derive(Debug)]
pub struct Version<'a> {
    version: u64,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Version<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.version() == other.version()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Version<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(version) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            version,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Version<'a> {
    pub fn new(version: u64) -> Self {
        Self {
            version,
            output_line: Cow::Owned(calculate_line(version)),
            output_line_is_dirty: false,
        }
    }

    pub(crate) fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn set_version(&mut self, version: u64) {
        self.version = version;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.version()));
        self.output_line_is_dirty = false;
    }
}

fn calculate_line(version: u64) -> String {
    format!("#EXT-X-VERSION:{version}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!("#EXT-X-VERSION:10", Version::new(10).into_inner().value());
    }
}
