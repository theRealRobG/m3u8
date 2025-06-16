use crate::{
    tag::{known::ParsedTag, value::ParsedTagValue},
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.1
#[derive(Debug)]
pub struct Inf<'a> {
    duration: f64,
    title: &'a str,
    output_line: Cow<'a, [u8]>, // Used with Writer
}

impl<'a> PartialEq for Inf<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.duration() == other.duration() && self.title() == other.title()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Inf<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        match tag.value {
            ParsedTagValue::DecimalInteger(d) => Ok(Self {
                duration: d as f64,
                title: "",
                output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            }),
            ParsedTagValue::DecimalFloatingPointWithOptionalTitle(duration, title) => Ok(Self {
                duration,
                title,
                output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            }),
            _ => Err(super::ValidationError::unexpected_value_type()),
        }
    }
}

impl<'a> Inf<'a> {
    pub fn new(duration: f64, title: &'a str) -> Self {
        let output_line = Cow::Owned(calculate_line(duration, title).into_bytes());
        Self {
            duration,
            title,
            output_line,
        }
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn title(&self) -> &'a str {
        self.title
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

fn calculate_line(duration: f64, title: &str) -> String {
    if title.is_empty() {
        format!("#EXTINF:{duration}")
    } else {
        format!("#EXTINF:{duration},{title}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid_with_empty_title() {
        assert_eq!("#EXTINF:6", Inf::new(6.0, "").as_str());
        assert_eq!("#EXTINF:6.006", Inf::new(6.006, "").as_str());
    }

    #[test]
    fn as_str_should_be_valid_with_some_title() {
        assert_eq!("#EXTINF:6,title", Inf::new(6.0, "title").as_str());
        assert_eq!("#EXTINF:6.006,title", Inf::new(6.006, "title").as_str());
    }
}
