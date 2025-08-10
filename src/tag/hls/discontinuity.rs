use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{hls::into_inner_tag, unknown},
};

/// Corresponds to the `#EXT-X-DISCONTINUITY` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.3>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Discontinuity;

impl TryFrom<unknown::Tag<'_>> for Discontinuity {
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

into_inner_tag!(Discontinuity @Static b"#EXT-X-DISCONTINUITY");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::value::TagValue;
    use pretty_assertions::assert_eq;

    #[test]
    fn succeeds_if_empty() {
        let tag = unknown::Tag {
            name: "-X-DISCONTINUITY",
            value: None,
            original_input: b"#EXT-X-DISCONTINUITY",
            validation_error: None,
        };
        assert_eq!(Ok(Discontinuity), Discontinuity::try_from(tag));
    }

    #[test]
    fn fails_if_not_empty() {
        let tag = unknown::Tag {
            name: "-X-DISCONTINUITY",
            value: Some(TagValue(b"100")),
            original_input: b"#EXT-X-DISCONTINUITY:100",
            validation_error: None,
        };
        assert_eq!(
            Err(ValidationError::ErrorExtractingTagValue(
                ParseTagValueError::NotEmpty
            )),
            Discontinuity::try_from(tag)
        );
    }
}
