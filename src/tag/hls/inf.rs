use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{hls::TagInner, known::ParsedTag, value::ParsedTagValue},
};
use std::borrow::Cow;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.1
#[derive(Debug)]
pub struct Inf<'a> {
    duration: f64,
    title: Cow<'a, str>,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Inf<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.duration() == other.duration() && self.title() == other.title()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Inf<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        match tag.value {
            ParsedTagValue::DecimalInteger(d) => Ok(Self {
                duration: d as f64,
                title: Cow::Borrowed(""),
                output_line: Cow::Borrowed(tag.original_input),
                output_line_is_dirty: false,
            }),
            ParsedTagValue::DecimalFloatingPointWithOptionalTitle(duration, title) => Ok(Self {
                duration,
                title: Cow::Borrowed(title),
                output_line: Cow::Borrowed(tag.original_input),
                output_line_is_dirty: false,
            }),
            _ => Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            )),
        }
    }
}

impl<'a> Inf<'a> {
    pub fn new(duration: f64, title: String) -> Self {
        let title = Cow::Owned(title);
        let output_line = Cow::Owned(calculate_line(duration, &title));
        Self {
            duration,
            title,
            output_line,
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

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
        self.output_line_is_dirty = true;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = Cow::Owned(title);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.duration(), self.title()));
        self.output_line_is_dirty = false;
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
        assert_eq!(
            "#EXTINF:6",
            Inf::new(6.0, "".to_string()).into_inner().value()
        );
        assert_eq!(
            "#EXTINF:6.006",
            Inf::new(6.006, "".to_string()).into_inner().value()
        );
    }

    #[test]
    fn as_str_should_be_valid_with_some_title() {
        assert_eq!(
            "#EXTINF:6,title",
            Inf::new(6.0, "title".to_string()).into_inner().value()
        );
        assert_eq!(
            "#EXTINF:6.006,title",
            Inf::new(6.006, "title".to_string()).into_inner().value()
        );
    }
}
