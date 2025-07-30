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
pub struct RenditionReportAttributeList<'a> {
    pub uri: Cow<'a, str>,
    pub last_msn: u64,
    pub last_part: Option<u64>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RenditionReportBuilder<'a> {
    uri: Cow<'a, str>,
    last_msn: u64,
    last_part: Option<u64>,
}
impl<'a> RenditionReportBuilder<'a> {
    pub fn new(uri: impl Into<Cow<'a, str>>, last_msn: u64) -> Self {
        Self {
            uri: uri.into(),
            last_msn,
            last_part: Default::default(),
        }
    }

    pub fn finish(self) -> RenditionReport<'a> {
        RenditionReport::new(RenditionReportAttributeList {
            uri: self.uri,
            last_msn: self.last_msn,
            last_part: self.last_part,
        })
    }

    pub fn with_last_part(mut self, last_part: u64) -> Self {
        self.last_part = Some(last_part);
        self
    }
}

/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.4>
#[derive(Debug, Clone)]
pub struct RenditionReport<'a> {
    uri: Cow<'a, str>,
    last_msn: u64,
    last_part: Option<u64>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for RenditionReport<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri() == other.uri()
            && self.last_msn() == other.last_msn()
            && self.last_part() == other.last_part()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for RenditionReport<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        let Some(ParsedAttributeValue::DecimalInteger(last_msn)) = attribute_list.get(LAST_MSN)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(LAST_MSN));
        };
        Ok(Self {
            uri: Cow::Borrowed(uri),
            last_msn: *last_msn,
            last_part: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> RenditionReport<'a> {
    pub fn new(attribute_list: RenditionReportAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let RenditionReportAttributeList {
            uri,
            last_msn,
            last_part,
        } = attribute_list;
        Self {
            uri,
            last_msn,
            last_part,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn builder(uri: impl Into<Cow<'a, str>>, last_msn: u64) -> RenditionReportBuilder<'a> {
        RenditionReportBuilder::new(uri, last_msn)
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn last_msn(&self) -> u64 {
        self.last_msn
    }

    pub fn last_part(&self) -> Option<u64> {
        if let Some(last_part) = self.last_part {
            Some(last_part)
        } else {
            match self.attribute_list.get(LAST_PART) {
                Some(ParsedAttributeValue::DecimalInteger(part)) => Some(*part),
                _ => None,
            }
        }
    }

    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_last_msn(&mut self, last_msn: u64) {
        self.attribute_list.remove(LAST_MSN);
        self.last_msn = last_msn;
        self.output_line_is_dirty = true;
    }

    pub fn set_last_part(&mut self, last_part: u64) {
        self.attribute_list.remove(LAST_PART);
        self.last_part = Some(last_part);
        self.output_line_is_dirty = true;
    }

    pub fn unset_last_part(&mut self) {
        self.attribute_list.remove(LAST_PART);
        self.last_part = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&RenditionReportAttributeList {
            uri: self.uri().into(),
            last_msn: self.last_msn(),
            last_part: self.last_part(),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(RenditionReport);

const URI: &str = "URI";
const LAST_MSN: &str = "LAST-MSN";
const LAST_PART: &str = "LAST-PART";

fn calculate_line(attribute_list: &RenditionReportAttributeList) -> Vec<u8> {
    let RenditionReportAttributeList {
        uri,
        last_msn,
        last_part,
    } = attribute_list;
    let mut line = format!("#EXT-X-RENDITION-REPORT:{URI}=\"{uri}\",{LAST_MSN}={last_msn}");
    if let Some(last_part) = last_part {
        line.push_str(format!(",{LAST_PART}={last_part}").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-RENDITION-REPORT:URI=\"low.m3u8\",LAST-MSN=100",
            RenditionReport::builder("low.m3u8", 100)
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-RENDITION-REPORT:URI=\"low.m3u8\",LAST-MSN=100,LAST-PART=2",
            RenditionReport::builder("low.m3u8", 100)
                .with_last_part(2)
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        RenditionReport::builder("low.m3u8", 100).with_last_part(2).finish(),
        (uri, "high.m3u8", @Attr="URI=\"high.m3u8\""),
        (last_msn, 200, @Attr="LAST-MSN=200"),
        (last_part, @Option 3, @Attr="LAST-PART=3")
    );
}
