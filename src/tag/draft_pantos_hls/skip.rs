use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.2
#[derive(Debug, PartialEq)]
pub struct Skip<'a> {
    skipped_segments: u64,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for Skip<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::DecimalInteger(skipped_segments)) =
            attribute_list.get(SKIPPED_SEGMENTS)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            skipped_segments: *skipped_segments,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> Skip<'a> {
    pub fn new(skipped_segments: u64, recently_removed_dateranges: Option<&'a str>) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(
            SKIPPED_SEGMENTS,
            ParsedAttributeValue::DecimalInteger(skipped_segments),
        );
        if let Some(recently_removed_dateranges) = recently_removed_dateranges {
            attribute_list.insert(
                RECENTLY_REMOVED_DATERANGES,
                ParsedAttributeValue::QuotedString(recently_removed_dateranges),
            );
        }
        Self {
            skipped_segments,
            attribute_list,
            output_line: Cow::Owned(
                calculate_line(skipped_segments, recently_removed_dateranges).into_bytes(),
            ),
        }
    }

    pub fn skipped_segments(&self) -> u64 {
        self.skipped_segments
    }

    pub fn recently_removed_dateranges(&self) -> Option<&'a str> {
        match self.attribute_list.get(RECENTLY_REMOVED_DATERANGES) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const SKIPPED_SEGMENTS: &'static str = "SKIPPED-SEGMENTS";
const RECENTLY_REMOVED_DATERANGES: &'static str = "RECENTLY-REMOVED-DATERANGES";

fn calculate_line(skipped_segments: u64, recently_removed_dateranges: Option<&str>) -> String {
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
            Skip::new(100, None).as_str()
        );
    }

    #[test]
    fn as_str_with_recently_removed_dateranges_shuold_be_valid() {
        assert_eq!(
            "#EXT-X-SKIP:SKIPPED-SEGMENTS=100,RECENTLY-REMOVED-DATERANGES=\"abc\t123\"",
            Skip::new(100, Some("abc\t123")).as_str()
        );
    }
}
