use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{EnumeratedString, into_inner_tag, key::Method},
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-SESSION-KEY:<attribute-list>`).
///
/// See [`SessionKey`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct SessionKeyAttributeList<'a> {
    /// Corresponds to the `METHOD` attribute.
    ///
    /// See [`SessionKey`] for a link to the HLS documentation for this attribute.
    method: Cow<'a, str>,
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`SessionKey`] for a link to the HLS documentation for this attribute.
    uri: Cow<'a, str>,
    /// Corresponds to the `IV` attribute.
    ///
    /// See [`SessionKey`] for a link to the HLS documentation for this attribute.
    iv: Option<Cow<'a, str>>,
    /// Corresponds to the `KEYFORMAT` attribute.
    ///
    /// See [`SessionKey`] for a link to the HLS documentation for this attribute.
    keyformat: Option<Cow<'a, str>>,
    /// Corresponds to the `KEYFORMATVERSIONS` attribute.
    ///
    /// See [`SessionKey`] for a link to the HLS documentation for this attribute.
    keyformatversions: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`SessionKeyBuilder`] indicating that `method` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct SessionKeyMethodNeedsToBeSet;
/// Placeholder struct for [`SessionKeyBuilder`] indicating that `uri` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct SessionKeyUriNeedsToBeSet;
/// Placeholder struct for [`SessionKeyBuilder`] indicating that `method` has been set.
#[derive(Debug, Clone, Copy)]
pub struct SessionKeyMethodHasBeenSet;
/// Placeholder struct for [`SessionKeyBuilder`] indicating that `uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct SessionKeyUriHasBeenSet;

/// A builder for convenience in constructing a [`SessionKey`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct SessionKeyBuilder<'a, MethodStatus, UriStatus> {
    attribute_list: SessionKeyAttributeList<'a>,
    method_status: PhantomData<MethodStatus>,
    uri_status: PhantomData<UriStatus>,
}
impl<'a> SessionKeyBuilder<'a, SessionKeyMethodNeedsToBeSet, SessionKeyUriNeedsToBeSet> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: SessionKeyAttributeList {
                method: Cow::Borrowed(""),
                uri: Cow::Borrowed(""),
                iv: Default::default(),
                keyformat: Default::default(),
                keyformatversions: Default::default(),
            },
            method_status: PhantomData,
            uri_status: PhantomData,
        }
    }
}
impl<'a> SessionKeyBuilder<'a, SessionKeyMethodHasBeenSet, SessionKeyUriHasBeenSet> {
    /// Finish building and construct the `SessionKey`.
    pub fn finish(self) -> SessionKey<'a> {
        SessionKey::new(self.attribute_list)
    }
}
impl<'a, MethodStatus, UriStatus> SessionKeyBuilder<'a, MethodStatus, UriStatus> {
    /// Add the provided `method` to the attributes built into `SessionKey`.
    pub fn with_method(
        mut self,
        method: impl Into<Cow<'a, str>>,
    ) -> SessionKeyBuilder<'a, SessionKeyMethodHasBeenSet, UriStatus> {
        self.attribute_list.method = method.into();
        SessionKeyBuilder {
            attribute_list: self.attribute_list,
            method_status: PhantomData,
            uri_status: PhantomData,
        }
    }
    /// Add the provided `uri` to the attributes built into `SessionKey`.
    pub fn with_uri(
        mut self,
        uri: impl Into<Cow<'a, str>>,
    ) -> SessionKeyBuilder<'a, MethodStatus, SessionKeyUriHasBeenSet> {
        self.attribute_list.uri = uri.into();
        SessionKeyBuilder {
            attribute_list: self.attribute_list,
            method_status: PhantomData,
            uri_status: PhantomData,
        }
    }
    /// Add the provided `iv` to the attributes built into `SessionKey`.
    pub fn with_iv(mut self, iv: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.iv = Some(iv.into());
        self
    }
    /// Add the provided `keyformat` to the attributes built into `SessionKey`.
    pub fn with_keyformat(mut self, keyformat: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.keyformat = Some(keyformat.into());
        self
    }
    /// Add the provided `keyformatversions` to the attributes built into `SessionKey`.
    pub fn with_keyformatversions(mut self, keyformatversions: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.keyformatversions = Some(keyformatversions.into());
        self
    }
}

/// Corresponds to the `#EXT-X-SESSION-KEY` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.5>
#[derive(Debug, Clone)]
pub struct SessionKey<'a> {
    method: Cow<'a, str>,
    uri: Cow<'a, str>,
    iv: Option<Cow<'a, str>>,
    keyformat: Option<Cow<'a, str>>,
    keyformatversions: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
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
    /// Constructs a new `SessionKey` tag.
    fn new(attribute_list: SessionKeyAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let SessionKeyAttributeList {
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

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `SessionKey` as such:
    /// ```
    /// # use m3u8::tag::hls::{SessionKey, Method};
    /// let session_key = SessionKey::builder()
    ///     .with_method(Method::SampleAes)
    ///     .with_uri("skd://1234")
    ///     .with_keyformat("com.apple.streamingkeydelivery")
    ///     .with_keyformatversions("1")
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `method` AND `uri`.
    /// Each of the following fail to compile:
    /// ```compile_fail
    /// # use m3u8::tag::hls::SessionKey;
    /// let session_key = SessionKey::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::SessionKey;
    /// let session_key = SessionKey::builder().method(Method::SampleAes).finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::SessionKey;
    /// let session_key = SessionKey::builder().with_uri("skd://1234").finish();
    /// ```
    pub fn builder()
    -> SessionKeyBuilder<'a, SessionKeyMethodNeedsToBeSet, SessionKeyUriNeedsToBeSet> {
        SessionKeyBuilder::new()
    }

    /// Corresponds to the `METHOD` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn method(&self) -> EnumeratedString<Method> {
        EnumeratedString::from(self.method.as_ref())
    }

    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Corresponds to the `IV` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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

    /// Corresponds to the `KEYFORMAT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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

    /// Corresponds to the `KEYFORMATVERSIONS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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

    /// Sets the `METHOD` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_method(&mut self, method: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(METHOD);
        self.method = method.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `IV` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_iv(&mut self, iv: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(IV);
        self.iv = Some(iv.into());
        self.output_line_is_dirty = true;
    }

    /// Unset the `IV` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_iv(&mut self) {
        self.attribute_list.remove(IV);
        self.iv = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `KEYFORMAT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_keyformat(&mut self, keyformat: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(KEYFORMAT);
        self.keyformat = Some(keyformat.into());
        self.output_line_is_dirty = true;
    }

    /// Unset the `KEYFORMAT` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_keyformat(&mut self) {
        self.attribute_list.remove(KEYFORMAT);
        self.keyformat = None;
        self.output_line_is_dirty = true;
    }

    /// Sets the `KEYFORMATVERSIONS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_keyformatversions(&mut self, keyformatversions: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(KEYFORMATVERSIONS);
        self.keyformatversions = Some(keyformatversions.into());
        self.output_line_is_dirty = true;
    }

    /// Unset the `KEYFORMATVERSIONS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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
        self.output_line = Cow::Owned(calculate_line(&SessionKeyAttributeList {
            method: self.method().into(),
            uri: self.uri().into(),
            iv: self.iv().map(|x| x.into()),
            keyformat: keyformat.map(|x| x.into()),
            keyformatversions: self.keyformatversions().map(|x| x.into()),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(SessionKey);

const METHOD: &str = "METHOD";
const URI: &str = "URI";
const IV: &str = "IV";
const KEYFORMAT: &str = "KEYFORMAT";
const KEYFORMATVERSIONS: &str = "KEYFORMATVERSIONS";

fn calculate_line(attribute_list: &SessionKeyAttributeList) -> Vec<u8> {
    let SessionKeyAttributeList {
        method,
        uri,
        iv,
        keyformat,
        keyformatversions,
    } = attribute_list;
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
            concat!(
                "#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"skd://some-key-id\",IV=0xABCD,",
                "KEYFORMAT=\"com.apple.streamingkeydelivery\",KEYFORMATVERSIONS=\"1\"",
            )
            .as_bytes(),
            SessionKey::builder()
                .with_method("SAMPLE-AES")
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
            b"#EXT-X-SESSION-KEY:METHOD=SAMPLE-AES,URI=\"some-key-id\"",
            SessionKey::builder()
                .with_method("SAMPLE-AES")
                .with_uri("some-key-id")
                .finish()
                .into_inner()
                .value()
        )
    }

    mutation_tests!(
        SessionKey::builder()
            .with_method("SAMPLE-AES")
            .with_uri("skd://some-key-id")
            .with_iv("0xABCD")
            .with_keyformat("com.apple.streamingkeydelivery")
            .with_keyformatversions("1")
            .finish(),
        (method, EnumeratedString::<Method>::Unknown("example"), @Attr="METHOD=example"),
        (uri, "example", @Attr="URI=\"example\""),
        (iv, @Option "0x1234", @Attr="IV=0x1234"),
        (keyformat, "example"; @Default="identity", @Attr="KEYFORMAT=\"example\""),
        (keyformatversions, @Option "example", @Attr="KEYFORMATVERSIONS=\"example\"")
    );
}
