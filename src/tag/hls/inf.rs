use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{hls::into_inner_tag, unknown},
};
use std::borrow::Cow;

/// Corresponds to the `#EXTINF` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.1>
#[derive(Debug, Clone)]
pub struct Inf<'a> {
    duration: f64,
    title: Cow<'a, str>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Inf<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.duration() == other.duration() && self.title() == other.title()
    }
}

impl<'a> TryFrom<unknown::Tag<'a>> for Inf<'a> {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
        let (duration, title) = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_decimal_floating_point_with_title()?;
        Ok(Self {
            duration,
            title: Cow::Borrowed(title),
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Inf<'a> {
    /// Constructs a new `Inf`.
    pub fn new(duration: f64, title: impl Into<Cow<'a, str>>) -> Self {
        let title = title.into();
        let output_line = Cow::Owned(calculate_line(duration, &title));
        Self {
            duration,
            title,
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Corresponds to the duration component of the tag value (`duration` in
    /// `#EXTINF:<duration>,[<title>]`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// Corresponds to the title component of the tag value (`title` in
    /// `#EXTINF:<duration>,[<title>]`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Sets the duration component value.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
        self.output_line_is_dirty = true;
    }

    /// Sets the title component value.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_title(&mut self, title: impl Into<Cow<'a, str>>) {
        self.title = title.into();
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.duration(), self.title()));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Inf);

fn calculate_line(duration: f64, title: &str) -> Vec<u8> {
    if title.is_empty() {
        format!("#EXTINF:{duration}").into_bytes()
    } else {
        format!("#EXTINF:{duration},{title}").into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid_with_empty_title() {
        assert_eq!(
            b"#EXTINF:6",
            Inf::new(6.0, "".to_string()).into_inner().value()
        );
        assert_eq!(
            b"#EXTINF:6.006",
            Inf::new(6.006, "".to_string()).into_inner().value()
        );
    }

    #[test]
    fn as_str_should_be_valid_with_some_title() {
        assert_eq!(
            b"#EXTINF:6,title",
            Inf::new(6.0, "title".to_string()).into_inner().value()
        );
        assert_eq!(
            b"#EXTINF:6.006,title",
            Inf::new(6.006, "title".to_string()).into_inner().value()
        );
    }

    mutation_tests!(
        Inf::new(6.006, "hello"),
        (duration, 10.0, @Attr="10"),
        (title, "world", @Attr=",world")
    );
}
