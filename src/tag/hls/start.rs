use crate::tag::{
    hls::TagInner,
    known::ParsedTag,
    value::{ParsedAttributeValue, ParsedTagValue},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2
#[derive(Debug)]
pub struct Start<'a> {
    time_offset: f64,
    precise: Option<bool>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for Start<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.time_offset() == other.time_offset() && self.precise() == other.precise()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Start<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(Some(time_offset)) = attribute_list
            .get("TIME-OFFSET")
            .map(ParsedAttributeValue::as_option_f64)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            time_offset,
            precise: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Start<'a> {
    pub fn new(time_offset: f64, precise: bool) -> Self {
        Self {
            time_offset,
            precise: Some(precise),
            attribute_list: HashMap::new(),
            output_line: Cow::Owned(calculate_line(time_offset, precise)),
            output_line_is_dirty: false,
        }
    }

    pub(crate) fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn time_offset(&self) -> f64 {
        self.time_offset
    }

    pub fn precise(&self) -> bool {
        if let Some(precise) = self.precise {
            precise
        } else {
            matches!(
                self.attribute_list.get(PRECISE),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }

    pub fn set_time_offset(&mut self, time_offset: f64) {
        self.attribute_list.remove(TIME_OFFSET);
        self.time_offset = time_offset;
        self.output_line_is_dirty = true;
    }

    pub fn set_precise(&mut self, precise: bool) {
        self.attribute_list.remove(PRECISE);
        self.precise = Some(precise);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(self.time_offset(), self.precise()));
        self.output_line_is_dirty = false;
    }
}

const TIME_OFFSET: &str = "TIME-OFFSET";
const PRECISE: &str = "PRECISE";
const YES: &str = "YES";

fn calculate_line(time_offset: f64, precise: bool) -> String {
    if precise {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset},{PRECISE}={YES}")
    } else {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_without_precise_should_be_valid() {
        assert_eq!(
            "#EXT-X-START:TIME-OFFSET=-42",
            Start::new(-42.0, false).into_inner().value()
        )
    }

    #[test]
    fn as_str_with_precise_should_be_valid() {
        assert_eq!(
            "#EXT-X-START:TIME-OFFSET=-42,PRECISE=YES",
            Start::new(-42.0, true).into_inner().value()
        )
    }
}
