use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{hls::into_inner_tag, unknown},
};

/// Corresponds to the `#EXTM3U` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.1.1>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct M3u;

impl TryFrom<unknown::Tag<'_>> for M3u {
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

into_inner_tag!(M3u @Static b"#EXTM3U");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::value::TagValue;
    use pretty_assertions::assert_eq;

    #[test]
    fn succeeds_if_empty() {
        let tag = unknown::Tag {
            name: "M3U",
            value: None,
            original_input: b"#EXTM3U",
            validation_error: None,
        };
        assert_eq!(Ok(M3u), M3u::try_from(tag));
    }

    #[test]
    fn fails_if_not_empty() {
        let tag = unknown::Tag {
            name: "M3U",
            value: Some(TagValue(b"100")),
            original_input: b"#EXTM3U:100",
            validation_error: None,
        };
        assert_eq!(
            Err(ValidationError::ErrorExtractingTagValue(
                ParseTagValueError::NotEmpty
            )),
            M3u::try_from(tag)
        );
    }
}
