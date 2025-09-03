use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{UnknownTag, hls::into_inner_tag},
};
use std::borrow::Cow;

/// Corresponds to the `#EXT-X-VERSION` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#section-4.4.1.2>
#[derive(Debug, Clone)]
pub struct Version<'a> {
    version: u64,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Version<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.version() == other.version()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for Version<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let version = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_decimal_integer()?;
        Ok(Self {
            version,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Version<'a> {
    /// Construct a new `Version` tag.
    pub fn new(version: u64) -> Self {
        Self {
            version,
            output_line: Cow::Owned(calculate_line(version)),
            output_line_is_dirty: false,
        }
    }

    /// Corresponds to the value of the tag (`#EXT-X-VERSION:<n>`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Sets the value of the tag.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_version(&mut self, version: u64) {
        self.version = version;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.version()));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Version);

fn calculate_line(version: u64) -> Vec<u8> {
    format!("#EXT-X-VERSION:{version}").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(b"#EXT-X-VERSION:10", Version::new(10).into_inner().value());
    }

    mutation_tests!(Version::new(10), (version, 20, @Attr=":20"));
}
