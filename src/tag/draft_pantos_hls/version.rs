use std::borrow::Cow;

use crate::{
    tag::{known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.2
#[derive(Debug, PartialEq)]
pub struct Version<'a> {
    version: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for Version<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(version) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            version,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> Version<'a> {
    pub fn new(version: u64) -> Self {
        Self {
            version,
            output_line: Cow::Owned(calculate_line(version).into_bytes()),
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
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
        assert_eq!("#EXT-X-VERSION:10", Version::new(10).as_str());
    }
}
