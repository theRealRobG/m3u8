use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{UnknownTag, hls::into_inner_tag},
};

/// Corresponds to the `#EXT-X-ENDLIST` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.4>
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Endlist;

impl TryFrom<UnknownTag<'_>> for Endlist {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'_>) -> Result<Self, Self::Error> {
        if tag.value().is_some() {
            return Err(ValidationError::ErrorExtractingTagValue(
                ParseTagValueError::NotEmpty,
            ));
        }
        Ok(Self)
    }
}

into_inner_tag!(Endlist @Static b"#EXT-X-ENDLIST");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::TagValue;
    use pretty_assertions::assert_eq;

    #[test]
    fn succeeds_if_empty() {
        let tag = UnknownTag {
            name: "-X-ENDLIST",
            value: None,
            original_input: b"#EXT-X-ENDLIST",
            validation_error: None,
        };
        assert_eq!(Ok(Endlist), Endlist::try_from(tag));
    }

    #[test]
    fn fails_if_not_empty() {
        let tag = UnknownTag {
            name: "-X-ENDLIST",
            value: Some(TagValue(b"100")),
            original_input: b"#EXT-X-ENDLIST:100",
            validation_error: None,
        };
        assert_eq!(
            Err(ValidationError::ErrorExtractingTagValue(
                ParseTagValueError::NotEmpty
            )),
            Endlist::try_from(tag)
        );
    }
}
