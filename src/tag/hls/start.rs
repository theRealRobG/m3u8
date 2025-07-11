use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, PartialEq, Clone)]
pub struct StartAttributeList {
    pub time_offset: f64,
    pub precise: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StartBuilder {
    time_offset: f64,
    precise: bool,
}
impl StartBuilder {
    pub fn new(time_offset: f64) -> Self {
        Self {
            time_offset,
            precise: Default::default(),
        }
    }

    pub fn finish<'a>(self) -> Start<'a> {
        Start::new(StartAttributeList {
            time_offset: self.time_offset,
            precise: self.precise,
        })
    }

    pub fn with_precise(mut self) -> Self {
        self.precise = true;
        self
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2
#[derive(Debug, Clone)]
pub struct Start<'a> {
    time_offset: f64,
    precise: Option<bool>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for Start<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.time_offset() == other.time_offset() && self.precise() == other.precise()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Start<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(Some(time_offset)) = attribute_list
            .get(TIME_OFFSET)
            .map(ParsedAttributeValue::as_option_f64)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(
                TIME_OFFSET,
            ));
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
    pub fn new(attribute_list: StartAttributeList) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let StartAttributeList {
            time_offset,
            precise,
        } = attribute_list;
        Self {
            time_offset,
            precise: Some(precise),
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn builder(time_offset: f64) -> StartBuilder {
        StartBuilder::new(time_offset)
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
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
        self.output_line = Cow::Owned(calculate_line(&StartAttributeList {
            time_offset: self.time_offset(),
            precise: self.precise(),
        }));
        self.output_line_is_dirty = false;
    }
}

const TIME_OFFSET: &str = "TIME-OFFSET";
const PRECISE: &str = "PRECISE";
const YES: &str = "YES";

fn calculate_line(attribute_list: &StartAttributeList) -> Vec<u8> {
    let StartAttributeList {
        time_offset,
        precise,
    } = attribute_list;
    if *precise {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset},{PRECISE}={YES}").into_bytes()
    } else {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset}").into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::tag::hls::test_macro::mutation_tests;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_without_precise_should_be_valid() {
        assert_eq!(
            b"#EXT-X-START:TIME-OFFSET=-42",
            Start::builder(-42.0).finish().into_inner().value()
        )
    }

    #[test]
    fn as_str_with_precise_should_be_valid() {
        assert_eq!(
            b"#EXT-X-START:TIME-OFFSET=-42,PRECISE=YES",
            Start::builder(-42.0)
                .with_precise()
                .finish()
                .into_inner()
                .value()
        )
    }

    mutation_tests!(
        Start::builder(-42.0).finish(),
        (time_offset, 10.0, @Attr="TIME-OFFSET=10"),
        (precise, true, @Attr="PRECISE=YES")
    );
}
