use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.5
#[derive(Debug, Clone)]
pub struct SessionKey<'a> {
    method: Cow<'a, str>,
    uri: Cow<'a, str>,
    iv: Option<Cow<'a, str>>,
    keyformat: Option<Cow<'a, str>>,
    keyformatversions: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for SessionKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.method() == other.method()
            && self.uri() == other.uri()
            && self.iv() == other.iv()
            && self.keyformat() == other.keyformat()
            && self.keyformatversions() == other.keyformatversions()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for SessionKey<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::UnquotedString(method)) = attribute_list.get(METHOD) else {
            return Err(super::ValidationError::MissingRequiredAttribute(METHOD));
        };
        let Some(ParsedAttributeValue::QuotedString(uri)) = attribute_list.get(URI) else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        Ok(Self {
            method: Cow::Borrowed(method),
            uri: Cow::Borrowed(uri),
            iv: None,
            keyformat: None,
            keyformatversions: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> SessionKey<'a> {
    pub fn new(
        method: String,
        uri: String,
        iv: Option<String>,
        keyformat: Option<String>,
        keyformatversions: Option<String>,
    ) -> Self {
        let method = Cow::Owned(method);
        let uri = Cow::Owned(uri);
        let iv = iv.map(Cow::Owned);
        let keyformat = keyformat.map(Cow::Owned);
        let keyformatversions = keyformatversions.map(Cow::Owned);
        let output_line = Cow::Owned(calculate_line(
            &method,
            &uri,
            &iv,
            &keyformat,
            &keyformatversions,
        ));
        Self {
            method,
            uri,
            iv,
            keyformat,
            keyformatversions,
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

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn iv(&self) -> Option<&str> {
        if let Some(iv) = &self.iv {
            Some(iv)
        } else {
            match self.attribute_list.get(IV) {
                Some(ParsedAttributeValue::UnquotedString(iv)) => Some(iv),
                _ => None,
            }
        }
    }

    pub fn keyformat(&self) -> &str {
        if let Some(keyformat) = &self.keyformat {
            keyformat
        } else {
            match self.attribute_list.get(KEYFORMAT) {
                Some(ParsedAttributeValue::QuotedString(keyformat)) => keyformat,
                _ => "identity",
            }
        }
    }

    pub fn keyformatversions(&self) -> Option<&str> {
        if let Some(keyformatversions) = &self.keyformatversions {
            Some(keyformatversions)
        } else {
            match self.attribute_list.get(KEYFORMATVERSIONS) {
                Some(ParsedAttributeValue::QuotedString(keyformatversions)) => {
                    Some(keyformatversions)
                }
                _ => None,
            }
        }
    }

    pub fn set_method(&mut self, method: String) {
        self.attribute_list.remove(METHOD);
        self.method = Cow::Owned(method);
        self.output_line_is_dirty = true;
    }

    pub fn set_uri(&mut self, uri: String) {
        self.attribute_list.remove(URI);
        self.uri = Cow::Owned(uri);
        self.output_line_is_dirty = true;
    }

    pub fn set_iv(&mut self, iv: Option<String>) {
        self.attribute_list.remove(IV);
        self.iv = iv.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_keyformat(&mut self, keyformat: String) {
        self.attribute_list.remove(KEYFORMAT);
        self.keyformat = Some(Cow::Owned(keyformat));
        self.output_line_is_dirty = true;
    }

    pub fn set_keyformatversions(&mut self, keyformatversions: Option<String>) {
        self.attribute_list.remove(KEYFORMATVERSIONS);
        self.keyformatversions = keyformatversions.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        let keyformat = self.keyformat();
        let keyformat = if keyformat == "identity" {
            None
        } else {
            Some(keyformat)
        };
        self.output_line = Cow::Owned(calculate_line(
            self.method(),
            self.uri(),
            &self.iv().map(|x| x.into()),
            &keyformat.map(|x| x.into()),
            &self.keyformatversions().map(|x| x.into()),
        ));
        self.output_line_is_dirty = false;
    }
}

const METHOD: &str = "METHOD";
const URI: &str = "URI";
const IV: &str = "IV";
const KEYFORMAT: &str = "KEYFORMAT";
const KEYFORMATVERSIONS: &str = "KEYFORMATVERSIONS";

fn calculate_line<'a>(
    method: &str,
    uri: &str,
    iv: &Option<Cow<'a, str>>,
    keyformat: &Option<Cow<'a, str>>,
    keyformatversions: &Option<Cow<'a, str>>,
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
                "SAMPLE-AES".to_string(),
                "skd://some-key-id".to_string(),
                Some("0xABCD".to_string()),
                Some("com.apple.streamingkeydelivery".to_string()),
                Some("1".to_string()),
            )
            .into_inner()
            .value()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            "#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"some-key-id\"",
            SessionKey::new(
                "SAMPLE-AES".to_string(),
                "some-key-id".to_string(),
                None,
                None,
                None
            )
            .into_inner()
            .value()
        )
    }
}
