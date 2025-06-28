use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.2
#[derive(Debug, Clone)]
pub struct Skip<'a> {
    skipped_segments: u64,
    recently_removed_dateranges: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for Skip<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.skipped_segments() == other.skipped_segments()
            && self.recently_removed_dateranges() == other.recently_removed_dateranges()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Skip<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::DecimalInteger(skipped_segments)) =
            attribute_list.get(SKIPPED_SEGMENTS)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(
                SKIPPED_SEGMENTS,
            ));
        };
        Ok(Self {
            skipped_segments: *skipped_segments,
            recently_removed_dateranges: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Skip<'a> {
    pub fn new(skipped_segments: u64, recently_removed_dateranges: Option<String>) -> Self {
        let recently_removed_dateranges = recently_removed_dateranges.map(Cow::Owned);
        let output_line = Cow::Owned(calculate_line(
            skipped_segments,
            &recently_removed_dateranges,
        ));
        Self {
            skipped_segments,
            recently_removed_dateranges,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn skipped_segments(&self) -> u64 {
        self.skipped_segments
    }

    pub fn recently_removed_dateranges(&self) -> Option<&str> {
        if let Some(recently_removed_dateranges) = &self.recently_removed_dateranges {
            Some(recently_removed_dateranges)
        } else {
            match self.attribute_list.get(RECENTLY_REMOVED_DATERANGES) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn set_skipped_segments(&mut self, skipped_segments: u64) {
        self.attribute_list.remove(SKIPPED_SEGMENTS);
        self.skipped_segments = skipped_segments;
        self.output_line_is_dirty = true;
    }

    pub fn set_recently_removed_dateranges(&mut self, recently_removed_dateranges: Option<String>) {
        self.attribute_list.remove(RECENTLY_REMOVED_DATERANGES);
        self.recently_removed_dateranges = recently_removed_dateranges.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.skipped_segments(),
            &self.recently_removed_dateranges().map(|x| x.into()),
        ));
        self.output_line_is_dirty = false;
    }
}

const SKIPPED_SEGMENTS: &str = "SKIPPED-SEGMENTS";
const RECENTLY_REMOVED_DATERANGES: &str = "RECENTLY-REMOVED-DATERANGES";

fn calculate_line<'a>(
    skipped_segments: u64,
    recently_removed_dateranges: &Option<Cow<'a, str>>,
) -> String {
    let mut line = format!("#EXT-X-SKIP:{SKIPPED_SEGMENTS}={skipped_segments}");
    if let Some(recently_removed_dateranges) = recently_removed_dateranges {
        line.push_str(
            format!(",{RECENTLY_REMOVED_DATERANGES}=\"{recently_removed_dateranges}\"").as_str(),
        );
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_recently_removed_dateranges_should_be_valid() {
        assert_eq!(
            "#EXT-X-SKIP:SKIPPED-SEGMENTS=100",
            Skip::new(100, None).into_inner().value()
        );
    }

    #[test]
    fn as_str_with_recently_removed_dateranges_shuold_be_valid() {
        assert_eq!(
            "#EXT-X-SKIP:SKIPPED-SEGMENTS=100,RECENTLY-REMOVED-DATERANGES=\"abc\t123\"",
            Skip::new(100, Some("abc\t123".to_string()))
                .into_inner()
                .value()
        );
    }
}
