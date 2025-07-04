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
pub struct PreloadHintAttributeList<'a> {
    pub hint_type: Cow<'a, str>,
    pub uri: Cow<'a, str>,
    pub byterange_start: Option<u64>,
    pub byterange_length: Option<u64>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PreloadHintBuilder<'a> {
    hint_type: Cow<'a, str>,
    uri: Cow<'a, str>,
    byterange_start: Option<u64>,
    byterange_length: Option<u64>,
}
impl<'a> PreloadHintBuilder<'a> {
    pub fn new(hint_type: impl Into<Cow<'a, str>>, uri: impl Into<Cow<'a, str>>) -> Self {
        Self {
            hint_type: hint_type.into(),
            uri: uri.into(),
            byterange_start: Default::default(),
            byterange_length: Default::default(),
        }
    }

    pub fn finish(self) -> PreloadHint<'a> {
        PreloadHint::new(PreloadHintAttributeList {
            hint_type: self.hint_type,
            uri: self.uri,
            byterange_start: self.byterange_start,
            byterange_length: self.byterange_length,
        })
    }

    pub fn with_byterange_start(mut self, byterange_start: u64) -> Self {
        self.byterange_start = Some(byterange_start);
        self
    }

    pub fn with_byterange_length(mut self, byterange_length: u64) -> Self {
        self.byterange_length = Some(byterange_length);
        self
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.3
#[derive(Debug, Clone)]
pub struct PreloadHint<'a> {
    hint_type: Cow<'a, str>,
    uri: Cow<'a, str>,
    byterange_start: Option<u64>,
    byterange_length: Option<u64>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for PreloadHint<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.hint_type() == other.hint_type()
            && self.uri() == other.uri()
            && self.byterange_start() == other.byterange_start()
            && self.byterange_length() == other.byterange_length()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for PreloadHint<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::UnquotedString(hint_type)) = attribute_list.get(TYPE) else {
            return Err(ValidationError::MissingRequiredAttribute(TYPE));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(ValidationError::MissingRequiredAttribute(URI));
        };
        Ok(Self {
            hint_type: Cow::Borrowed(hint_type),
            uri: Cow::Borrowed(uri),
            byterange_start: None,
            byterange_length: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> PreloadHint<'a> {
    pub fn new(attribute_list: PreloadHintAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let PreloadHintAttributeList {
            hint_type,
            uri,
            byterange_start,
            byterange_length,
        } = attribute_list;
        Self {
            hint_type,
            uri,
            byterange_start,
            byterange_length,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn builder(
        hint_type: impl Into<Cow<'a, str>>,
        uri: impl Into<Cow<'a, str>>,
    ) -> PreloadHintBuilder<'a> {
        PreloadHintBuilder::new(hint_type, uri)
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn hint_type(&self) -> &str {
        &self.hint_type
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn byterange_start(&self) -> u64 {
        if let Some(byterange_start) = self.byterange_start {
            byterange_start
        } else {
            match self.attribute_list.get(BYTERANGE_START) {
                Some(ParsedAttributeValue::DecimalInteger(start)) => *start,
                _ => 0,
            }
        }
    }

    pub fn byterange_length(&self) -> Option<u64> {
        if let Some(byterange_length) = self.byterange_length {
            Some(byterange_length)
        } else {
            match self.attribute_list.get(BYTERANGE_LENGTH) {
                Some(ParsedAttributeValue::DecimalInteger(length)) => Some(*length),
                _ => None,
            }
        }
    }

    pub fn set_hint_type(&mut self, hint_type: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(TYPE);
        self.hint_type = hint_type.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_byterange_start(&mut self, byterange_start: u64) {
        self.attribute_list.remove(BYTERANGE_START);
        self.byterange_start = Some(byterange_start);
        self.output_line_is_dirty = true;
    }

    pub fn unset_byterange_start(&mut self) {
        self.attribute_list.remove(BYTERANGE_START);
        self.byterange_start = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_byterange_length(&mut self, byterange_length: u64) {
        self.attribute_list.remove(BYTERANGE_LENGTH);
        self.byterange_length = Some(byterange_length);
        self.output_line_is_dirty = true;
    }

    pub fn unset_byterange_length(&mut self) {
        self.attribute_list.remove(BYTERANGE_LENGTH);
        self.byterange_length = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        let byterange_start = if self.byterange_start() == 0 {
            None
        } else {
            Some(self.byterange_start())
        };
        self.output_line = Cow::Owned(calculate_line(&PreloadHintAttributeList {
            hint_type: self.hint_type().into(),
            uri: self.uri().into(),
            byterange_start,
            byterange_length: self.byterange_length(),
        }));
        self.output_line_is_dirty = false;
    }
}

const TYPE: &str = "TYPE";
const URI: &str = "URI";
const BYTERANGE_START: &str = "BYTERANGE-START";
const BYTERANGE_LENGTH: &str = "BYTERANGE-LENGTH";

fn calculate_line(attribute_list: &PreloadHintAttributeList) -> Vec<u8> {
    let PreloadHintAttributeList {
        hint_type,
        uri,
        byterange_start,
        byterange_length,
    } = attribute_list;
    let mut line = format!("#EXT-X-PRELOAD-HINT:{TYPE}={hint_type},URI=\"{uri}\"");
    if let Some(byterange_start) = byterange_start {
        line.push_str(format!(",{BYTERANGE_START}={byterange_start}").as_str());
    }
    if let Some(byterange_length) = byterange_length {
        line.push_str(format!(",{BYTERANGE_LENGTH}={byterange_length}").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::hls::test_macro::mutation_tests;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\"",
            PreloadHint::builder("PART", "part.2.mp4")
                .finish()
                .into_inner()
                .value()
        )
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\",BYTERANGE-START=512,BYTERANGE-LENGTH=1024",
            PreloadHint::builder("PART", "part.2.mp4")
                .with_byterange_start(512)
                .with_byterange_length(1024)
                .finish()
                .into_inner()
                .value()
        )
    }

    mutation_tests!(
        PreloadHint::builder("PART", "part.2.mp4")
            .with_byterange_start(512)
            .with_byterange_length(1024)
            .finish(),
        (hint_type, "PART", @Attr="TYPE=PART"),
        (uri, "part.2.mp4", @Attr="URI=\"part.2.mp4\""),
        (byterange_start, 100; @Default=0, @Attr="BYTERANGE-START=100"),
        (byterange_length, @Option 200, @Attr="BYTERANGE-LENGTH=200")
    );
}
