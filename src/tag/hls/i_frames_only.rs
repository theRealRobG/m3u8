use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{hls::into_inner_tag, unknown},
};

/// Corresponds to the `#EXT-X-I-FRAMES-ONLY` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.6>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IFramesOnly;

impl TryFrom<unknown::Tag<'_>> for IFramesOnly {
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

into_inner_tag!(IFramesOnly @Static b"#EXT-X-I-FRAMES-ONLY");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::value::TagValue;
    use pretty_assertions::assert_eq;

    #[test]
    fn succeeds_if_empty() {
        let tag = unknown::Tag {
            name: "-X-I-FRAMES-ONLY",
            value: None,
            original_input: b"#EXT-X-I-FRAMES-ONLY",
            validation_error: None,
        };
        assert_eq!(Ok(IFramesOnly), IFramesOnly::try_from(tag));
    }

    #[test]
    fn fails_if_not_empty() {
        let tag = unknown::Tag {
            name: "-X-I-FRAMES-ONLY",
            value: Some(TagValue(b"100")),
            original_input: b"#EXT-X-I-FRAMES-ONLY:100",
            validation_error: None,
        };
        assert_eq!(
            Err(ValidationError::ErrorExtractingTagValue(
                ParseTagValueError::NotEmpty
            )),
            IFramesOnly::try_from(tag)
        );
    }
}
