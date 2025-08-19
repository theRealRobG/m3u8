use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{UnknownTag, hls::into_inner_tag},
};
use std::borrow::Cow;

/// Corresponds to the `#EXT-X-PART-INF` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.7>
#[derive(Debug, Clone)]
pub struct PartInf<'a> {
    part_target: f64,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for PartInf<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.part_target() == other.part_target()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for PartInf<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_ordered_attribute_list()?;
        let Some(part_target) = attribute_list.iter().find_map(|(name, value)| {
            if *name == PART_TARGET {
                value
                    .unquoted()
                    .and_then(|v| v.try_as_decimal_floating_point().ok())
            } else {
                None
            }
        }) else {
            return Err(super::ValidationError::MissingRequiredAttribute(
                PART_TARGET,
            ));
        };
        Ok(Self {
            part_target,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> PartInf<'a> {
    /// Construct a new `PartInf` tag.
    pub fn new(part_target: f64) -> Self {
        Self {
            part_target,
            output_line: Cow::Owned(calculate_line(part_target)),
            output_line_is_dirty: false,
        }
    }

    /// Corresponds to the `PART-TARGET` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn part_target(&self) -> f64 {
        self.part_target
    }

    /// Sets `PART-TARGET` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_part_target(&mut self, part_target: f64) {
        self.part_target = part_target;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.part_target()));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(PartInf);

const PART_TARGET: &str = "PART-TARGET";

fn calculate_line(part_target: f64) -> Vec<u8> {
    format!("#EXT-X-PART-INF:{PART_TARGET}={part_target}").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PART-INF:PART-TARGET=0.5",
            PartInf::new(0.5).into_inner().value()
        );
    }

    mutation_tests!(PartInf::new(0.5), (part_target, 1.0, @Attr="PART-TARGET=1"));
}
