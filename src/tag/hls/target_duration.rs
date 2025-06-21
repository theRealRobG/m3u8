use crate::{
    tag::{known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.1
#[derive(Debug, PartialEq)]
pub struct Targetduration<'a> {
    target_duration: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for Targetduration<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::DecimalInteger(d) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        Ok(Self {
            target_duration: d,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> Targetduration<'a> {
    pub fn new(target_duration: u64) -> Self {
        Self {
            target_duration,
            output_line: Cow::Owned(calculate_line(target_duration).into_bytes()),
        }
    }

    pub fn target_duration(&self) -> u64 {
        self.target_duration
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

fn calculate_line(target_duration: u64) -> String {
    format!("#EXT-X-TARGETDURATION:{target_duration}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!("#EXT-X-TARGETDURATION:10", Targetduration::new(10).as_str());
    }
}
