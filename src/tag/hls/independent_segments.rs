use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{hls::into_inner_tag, unknown},
};

/// Corresponds to the `#EXT-X-INDEPENDENT-SEGMENTS` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.1>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IndependentSegments;

impl TryFrom<unknown::Tag<'_>> for IndependentSegments {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag<'_>) -> Result<Self, Self::Error> {
        if tag.value().is_some() {
            return Err(ValidationError::ErrorExtractingTagValue(
                ParseTagValueError::NotEmpty,
            ));
        }
        Ok(Self)
    }
}

into_inner_tag!(IndependentSegments @Static b"#EXT-X-INDEPENDENT-SEGMENTS");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::value::TagValue;
    use pretty_assertions::assert_eq;

    #[test]
    fn succeeds_if_empty() {
        let tag = unknown::Tag {
            name: "-X-INDEPENDENT-SEGMENTS",
            value: None,
            original_input: b"#EXT-X-INDEPENDENT-SEGMENTS",
            validation_error: None,
        };
        assert_eq!(Ok(IndependentSegments), IndependentSegments::try_from(tag));
    }

    #[test]
    fn fails_if_not_empty() {
        let tag = unknown::Tag {
            name: "-X-INDEPENDENT-SEGMENTS",
            value: Some(TagValue(b"100")),
            original_input: b"#EXT-X-INDEPENDENT-SEGMENTS:100",
            validation_error: None,
        };
        assert_eq!(
            Err(ValidationError::ErrorExtractingTagValue(
                ParseTagValueError::NotEmpty
            )),
            IndependentSegments::try_from(tag)
        );
    }
}
