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
pub struct KeyAttributeList<'a> {
    pub method: Cow<'a, str>,
    pub uri: Option<Cow<'a, str>>,
    pub iv: Option<Cow<'a, str>>,
    pub keyformat: Option<Cow<'a, str>>,
    pub keyformatversions: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct KeyBuilder<'a> {
    method: Cow<'a, str>,
    uri: Option<Cow<'a, str>>,
    iv: Option<Cow<'a, str>>,
    keyformat: Option<Cow<'a, str>>,
    keyformatversions: Option<Cow<'a, str>>,
}
impl<'a> KeyBuilder<'a> {
    pub fn new(method: impl Into<Cow<'a, str>>) -> Self {
        Self {
            method: method.into(),
            uri: Default::default(),
            iv: Default::default(),
            keyformat: Default::default(),
            keyformatversions: Default::default(),
        }
    }

    pub fn finish(self) -> Key<'a> {
        Key::new(KeyAttributeList {
            method: self.method,
            uri: self.uri,
            iv: self.iv,
            keyformat: self.keyformat,
            keyformatversions: self.keyformatversions,
        })
    }

    pub fn with_method(mut self, method: impl Into<Cow<'a, str>>) -> Self {
        self.method = method.into();
        self
    }

    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    pub fn with_iv(mut self, iv: impl Into<Cow<'a, str>>) -> Self {
        self.iv = Some(iv.into());
        self
    }

    pub fn with_keyformat(mut self, keyformat: impl Into<Cow<'a, str>>) -> Self {
        self.keyformat = Some(keyformat.into());
        self
    }

    pub fn with_keyformatversions(mut self, keyformatversions: impl Into<Cow<'a, str>>) -> Self {
        self.keyformatversions = Some(keyformatversions.into());
        self
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.4
#[derive(Debug, Clone)]
pub struct Key<'a> {
    method: Cow<'a, str>,
    uri: Option<Cow<'a, str>>,
    iv: Option<Cow<'a, str>>,
    keyformat: Option<Cow<'a, str>>,
    keyformatversions: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for Key<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.method() == other.method()
            && self.uri() == other.uri()
            && self.iv() == other.iv()
            && self.keyformat() == other.keyformat()
            && self.keyformatversions() == other.keyformatversions()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Key<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::UnquotedString(method)) = attribute_list.get(METHOD) else {
            return Err(super::ValidationError::MissingRequiredAttribute(METHOD));
        };
        Ok(Self {
            method: Cow::Borrowed(method),
            uri: None,
            iv: None,
            keyformat: None,
            keyformatversions: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Key<'a> {
    pub fn new(attribute_list: KeyAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let KeyAttributeList {
            method,
            uri,
            iv,
            keyformat,
            keyformatversions,
        } = attribute_list;
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

    pub fn builder(method: impl Into<Cow<'a, str>>) -> KeyBuilder<'a> {
        KeyBuilder::new(method)
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

    pub fn uri(&self) -> Option<&str> {
        if let Some(uri) = &self.uri {
            Some(uri)
        } else {
            match self.attribute_list.get(URI) {
                Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
                _ => None,
            }
        }
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

    pub fn set_method(&mut self, method: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(METHOD);
        self.method = method.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = Some(uri.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_uri(&mut self) {
        self.attribute_list.remove(URI);
        self.uri = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_iv(&mut self, iv: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(IV);
        self.iv = Some(iv.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_iv(&mut self) {
        self.attribute_list.remove(IV);
        self.iv = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_keyformat(&mut self, keyformat: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(KEYFORMAT);
        self.keyformat = Some(keyformat.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_keyformat(&mut self) {
        self.attribute_list.remove(KEYFORMAT);
        self.keyformat = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_keyformatversions(&mut self, keyformatversions: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(KEYFORMATVERSIONS);
        self.keyformatversions = Some(keyformatversions.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_keyformatversions(&mut self) {
        self.attribute_list.remove(KEYFORMATVERSIONS);
        self.keyformatversions = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        let keyformat = self.keyformat();
        let keyformat = if keyformat == "identity" {
            None
        } else {
            Some(keyformat)
        };
        self.output_line = Cow::Owned(calculate_line(&KeyAttributeList {
            method: self.method().into(),
            uri: self.uri().map(|x| x.into()),
            iv: self.iv().map(|x| x.into()),
            keyformat: keyformat.map(|x| x.into()),
            keyformatversions: self.keyformatversions().map(|x| x.into()),
        }));
        self.output_line_is_dirty = false;
    }
}

const METHOD: &str = "METHOD";
const URI: &str = "URI";
const IV: &str = "IV";
const KEYFORMAT: &str = "KEYFORMAT";
const KEYFORMATVERSIONS: &str = "KEYFORMATVERSIONS";

fn calculate_line(attribute_list: &KeyAttributeList) -> Vec<u8> {
    let KeyAttributeList {
        method,
        uri,
        iv,
        keyformat,
        keyformatversions,
    } = attribute_list;
    let mut line = format!("#EXT-X-KEY:{METHOD}={method}");
    if let Some(uri) = uri {
        line.push_str(format!(",{URI}=\"{uri}\"").as_str());
    }
    if let Some(iv) = iv {
        line.push_str(format!(",{IV}={iv}").as_str());
    }
    if let Some(keyformat) = keyformat {
        line.push_str(format!(",{KEYFORMAT}=\"{keyformat}\"").as_str());
    }
    if let Some(keyformatversions) = keyformatversions {
        line.push_str(format!(",{KEYFORMATVERSIONS}=\"{keyformatversions}\"").as_str());
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
            concat!(
                "#EXT-X-KEY:METHOD=SAMPLE-AES,URI=\"skd://some-key-id\",IV=0xABCD,",
                "KEYFORMAT=\"com.apple.streamingkeydelivery\",KEYFORMATVERSIONS=\"1\"",
            )
            .as_bytes(),
            Key::builder("SAMPLE-AES")
                .with_uri("skd://some-key-id")
                .with_iv("0xABCD")
                .with_keyformat("com.apple.streamingkeydelivery")
                .with_keyformatversions("1")
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-KEY:METHOD=NONE",
            Key::builder("NONE").finish().into_inner().value()
        )
    }

    mutation_tests!(
        Key::builder("SAMPLE-AES")
            .with_uri("skd://some-key-id")
            .with_iv("0xABCD")
            .with_keyformat("com.apple.streamingkeydelivery")
            .with_keyformatversions("1")
            .finish(),
        (method, "EXAMPLE", @Attr="METHOD=EXAMPLE"),
        (uri, @Option "example.key", @Attr="URI=\"example.key\""),
        (iv, @Option "0x1234", @Attr="IV=0x1234"),
        (keyformat, "example"; @Default="identity", @Attr="KEYFORMAT=\"example\""),
        (keyformatversions, @Option "example", @Attr="KEYFORMATVERSIONS=\"example\"")
    );
}
