use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::into_inner_tag,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, PartialEq, Clone)]
pub struct SkipAttributeList<'a> {
    pub skipped_segments: u64,
    pub recently_removed_dateranges: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SkipBuilder<'a> {
    skipped_segments: u64,
    recently_removed_dateranges: Option<Cow<'a, str>>,
}
impl<'a> SkipBuilder<'a> {
    pub fn new(skipped_segments: u64) -> Self {
        Self {
            skipped_segments,
            recently_removed_dateranges: Default::default(),
        }
    }

    pub fn finish(self) -> Skip<'a> {
        Skip::new(SkipAttributeList {
            skipped_segments: self.skipped_segments,
            recently_removed_dateranges: self.recently_removed_dateranges,
        })
    }

    pub fn with_recently_removed_dateranges(
        mut self,
        recently_removed_dateranges: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.recently_removed_dateranges = Some(recently_removed_dateranges.into());
        self
    }
}

/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.2>
#[derive(Debug, Clone)]
pub struct Skip<'a> {
    skipped_segments: u64,
    recently_removed_dateranges: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
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
    pub fn new(attribute_list: SkipAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let SkipAttributeList {
            skipped_segments,
            recently_removed_dateranges,
        } = attribute_list;
        Self {
            skipped_segments,
            recently_removed_dateranges,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn builder(skipped_segments: u64) -> SkipBuilder<'a> {
        SkipBuilder::new(skipped_segments)
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

    pub fn set_recently_removed_dateranges(
        &mut self,
        recently_removed_dateranges: impl Into<Cow<'a, str>>,
    ) {
        self.attribute_list.remove(RECENTLY_REMOVED_DATERANGES);
        self.recently_removed_dateranges = Some(recently_removed_dateranges.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_recently_removed_dateranges(&mut self) {
        self.attribute_list.remove(RECENTLY_REMOVED_DATERANGES);
        self.recently_removed_dateranges = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&SkipAttributeList {
            skipped_segments: self.skipped_segments(),
            recently_removed_dateranges: self.recently_removed_dateranges().map(|x| x.into()),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Skip);

const SKIPPED_SEGMENTS: &str = "SKIPPED-SEGMENTS";
const RECENTLY_REMOVED_DATERANGES: &str = "RECENTLY-REMOVED-DATERANGES";

fn calculate_line(attribute_list: &SkipAttributeList) -> Vec<u8> {
    let SkipAttributeList {
        skipped_segments,
        recently_removed_dateranges,
    } = attribute_list;
    let mut line = format!("#EXT-X-SKIP:{SKIPPED_SEGMENTS}={skipped_segments}");
    if let Some(recently_removed_dateranges) = recently_removed_dateranges {
        line.push_str(
            format!(",{RECENTLY_REMOVED_DATERANGES}=\"{recently_removed_dateranges}\"").as_str(),
        );
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_recently_removed_dateranges_should_be_valid() {
        assert_eq!(
            b"#EXT-X-SKIP:SKIPPED-SEGMENTS=100",
            Skip::builder(100).finish().into_inner().value()
        );
    }

    #[test]
    fn as_str_with_recently_removed_dateranges_shuold_be_valid() {
        assert_eq!(
            b"#EXT-X-SKIP:SKIPPED-SEGMENTS=100,RECENTLY-REMOVED-DATERANGES=\"abc\t123\"",
            Skip::builder(100)
                .with_recently_removed_dateranges("abc\t123")
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        Skip::builder(100)
            .with_recently_removed_dateranges("abc\t123")
            .finish(),
        (skipped_segments, 200, @Attr="SKIPPED-SEGMENTS=200"),
        (recently_removed_dateranges, @Option "efg", @Attr="RECENTLY-REMOVED-DATERANGES=\"efg\"")
    );
}
