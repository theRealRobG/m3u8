use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.3
#[derive(Debug, PartialEq)]
pub struct PreloadHint<'a> {
    hint_type: &'a str,
    uri: &'a str,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for PreloadHint<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(hint_type)) = attribute_list.get(TYPE) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            hint_type,
            uri,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> PreloadHint<'a> {
    pub fn new(
        hint_type: &'a str,
        uri: &'a str,
        byterange_start: Option<u64>,
        byterange_length: Option<u64>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(TYPE, ParsedAttributeValue::UnquotedString(hint_type));
        attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        if let Some(byterange_start) = byterange_start {
            attribute_list.insert(
                BYTERANGE_START,
                ParsedAttributeValue::DecimalInteger(byterange_start),
            );
        }
        if let Some(byterange_length) = byterange_length {
            attribute_list.insert(
                BYTERANGE_LENGTH,
                ParsedAttributeValue::DecimalInteger(byterange_length),
            );
        }
        Self {
            hint_type,
            uri,
            attribute_list,
            output_line: Cow::Owned(
                calculate_line(hint_type, uri, byterange_start, byterange_length).into_bytes(),
            ),
        }
    }

    pub fn hint_type(&self) -> &'a str {
        self.hint_type
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn byterange_start(&self) -> u64 {
        match self.attribute_list.get(BYTERANGE_START) {
            Some(ParsedAttributeValue::DecimalInteger(start)) => *start,
            _ => 0,
        }
    }

    pub fn byterange_length(&self) -> Option<u64> {
        match self.attribute_list.get(BYTERANGE_LENGTH) {
            Some(ParsedAttributeValue::DecimalInteger(length)) => Some(*length),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const TYPE: &str = "TYPE";
const URI: &str = "URI";
const BYTERANGE_START: &str = "BYTERANGE-START";
const BYTERANGE_LENGTH: &str = "BYTERANGE-LENGTH";

fn calculate_line(
    hint_type: &str,
    uri: &str,
    byterange_start: Option<u64>,
    byterange_length: Option<u64>,
) -> String {
    let mut line = format!("#EXT-X-PRELOAD-HINT:{TYPE}={hint_type},URI=\"{uri}\"");
    if let Some(byterange_start) = byterange_start {
        line.push_str(format!(",{BYTERANGE_START}={byterange_start}").as_str());
    }
    if let Some(byterange_length) = byterange_length {
        line.push_str(format!(",{BYTERANGE_LENGTH}={byterange_length}").as_str());
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\"",
            PreloadHint::new("PART", "part.2.mp4", None, None).as_str()
        )
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.2.mp4\",BYTERANGE-START=512,BYTERANGE-LENGTH=1024",
            PreloadHint::new("PART", "part.2.mp4", Some(512), Some(1024)).as_str()
        )
    }
}
