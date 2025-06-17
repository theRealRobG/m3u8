use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.5
#[derive(Debug, PartialEq)]
pub struct SessionKey<'a> {
    method: &'a str,
    uri: &'a str,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
}

impl<'a> TryFrom<ParsedTag<'a>> for SessionKey<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(method)) = attribute_list.get(METHOD) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            method,
            uri,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
        })
    }
}

impl<'a> SessionKey<'a> {
    pub fn new(
        method: &'a str,
        uri: &'a str,
        iv: Option<&'a str>,
        keyformat: Option<&'a str>,
        keyformatversions: Option<&'a str>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(METHOD, ParsedAttributeValue::UnquotedString(method));
        attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        if let Some(iv) = iv {
            attribute_list.insert(IV, ParsedAttributeValue::UnquotedString(iv));
        }
        if let Some(keyformat) = keyformat {
            attribute_list.insert(KEYFORMAT, ParsedAttributeValue::QuotedString(keyformat));
        }
        if let Some(keyformatversions) = keyformatversions {
            attribute_list.insert(
                KEYFORMATVERSIONS,
                ParsedAttributeValue::QuotedString(keyformatversions),
            );
        }
        Self {
            method,
            uri,
            attribute_list,
            output_line: Cow::Owned(
                calculate_line(method, uri, iv, keyformat, keyformatversions).into_bytes(),
            ),
        }
    }

    pub fn method(&self) -> &'a str {
        self.method
    }

    pub fn uri(&self) -> &'a str {
        self.uri
    }

    pub fn iv(&self) -> Option<&'a str> {
        match self.attribute_list.get(IV) {
            Some(ParsedAttributeValue::UnquotedString(iv)) => Some(iv),
            _ => None,
        }
    }

    pub fn keyformat(&self) -> &'a str {
        match self.attribute_list.get(KEYFORMAT) {
            Some(ParsedAttributeValue::QuotedString(keyformat)) => keyformat,
            _ => "identity",
        }
    }

    pub fn keyformatversions(&self) -> Option<&'a str> {
        match self.attribute_list.get(KEYFORMATVERSIONS) {
            Some(ParsedAttributeValue::QuotedString(keyformatversions)) => Some(keyformatversions),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
}

const METHOD: &str = "METHOD";
const URI: &str = "URI";
const IV: &str = "IV";
const KEYFORMAT: &str = "KEYFORMAT";
const KEYFORMATVERSIONS: &str = "KEYFORMATVERSIONS";

fn calculate_line(
    method: &str,
    uri: &str,
    iv: Option<&str>,
    keyformat: Option<&str>,
    keyformatversions: Option<&str>,
) -> String {
    let mut line = format!("#EXT-X-SESSION-KEY:{METHOD}={method},{URI}=\"{uri}\"");
    if let Some(iv) = iv {
        line.push_str(format!(",{IV}={iv}").as_str());
    }
    if let Some(keyformat) = keyformat {
        line.push_str(format!(",{KEYFORMAT}=\"{keyformat}\"").as_str());
    }
    if let Some(keyformatversions) = keyformatversions {
        line.push_str(format!(",{KEYFORMATVERSIONS}=\"{keyformatversions}\"").as_str());
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
            concat!(
                "#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"skd://some-key-id\",IV=0xABCD,",
                "KEYFORMAT=\"com.apple.streamingkeydelivery\",KEYFORMATVERSIONS=\"1\"",
            ),
            SessionKey::new(
                "SAMPLE-AES",
                "skd://some-key-id",
                Some("0xABCD"),
                Some("com.apple.streamingkeydelivery"),
                Some("1"),
            )
            .as_str()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"some-key-id\"",
            SessionKey::new("SAMPLE-AES", "some-key-id", None, None, None).as_str()
        )
    }
}
