use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{
        hls::{TagName, into_inner_tag},
        unknown,
        value::AttributeValue,
    },
};
use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-CONTENT-STEERING:<attribute-list>`).
///
/// See [`ContentSteering`] for a link to the HLS documentation for these attributes.
#[derive(Debug, PartialEq, Clone)]
struct ContentSteeringAttributeList<'a> {
    /// Corresponds to the `SERVER-URI` attribute.
    ///
    /// See [`ContentSteering`] for a link to the HLS documentation for this attribute.
    server_uri: Cow<'a, str>,
    /// Corresponds to the `PATHWAY-ID` attribute.
    ///
    /// See [`ContentSteering`] for a link to the HLS documentation for this attribute.
    pathway_id: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`ContentSteeringBuilder`] indicating that `server_uri` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct ContentSteeringServerUriNeedsToBeSet;
/// Placeholder struct for [`ContentSteeringBuilder`] indicating that `server_uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct ContentSteeringServerUriHasBeenSet;

/// A builder for convenience in constructing a [`ContentSteering`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, PartialEq, Clone)]
pub struct ContentSteeringBuilder<'a, ServerUriStatus> {
    attribute_list: ContentSteeringAttributeList<'a>,
    attribute_status: PhantomData<ServerUriStatus>,
}
impl<'a> ContentSteeringBuilder<'a, ContentSteeringServerUriNeedsToBeSet> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: ContentSteeringAttributeList {
                server_uri: Cow::Borrowed(""),
                pathway_id: Default::default(),
            },
            attribute_status: PhantomData,
        }
    }
}
impl<'a> ContentSteeringBuilder<'a, ContentSteeringServerUriHasBeenSet> {
    /// Finish building and construct the `ContentSteering`.
    pub fn finish(self) -> ContentSteering<'a> {
        ContentSteering::new(self.attribute_list)
    }
}
impl<'a, ServerUriStatus> ContentSteeringBuilder<'a, ServerUriStatus> {
    /// Add the provided `server_uri` to the attributes that are built into the `ContentSteering`.
    pub fn with_server_uri(
        mut self,
        server_uri: impl Into<Cow<'a, str>>,
    ) -> ContentSteeringBuilder<'a, ContentSteeringServerUriHasBeenSet> {
        self.attribute_list.server_uri = server_uri.into();
        ContentSteeringBuilder {
            attribute_list: self.attribute_list,
            attribute_status: PhantomData,
        }
    }

    /// Add the provided `pathway_id` to the attributes that are built into the `ContentSteering`.
    pub fn with_pathway_id(mut self, pathway_id: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.pathway_id = Some(pathway_id.into());
        self
    }
}
impl<'a> Default for ContentSteeringBuilder<'a, ContentSteeringServerUriNeedsToBeSet> {
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-CONTENT-STEERING` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.6>
#[derive(Debug, Clone)]
pub struct ContentSteering<'a> {
    server_uri: Cow<'a, str>,
    pathway_id: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, AttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                           // Used with Writer
    output_line_is_dirty: bool,                           // If should recalculate output_line
}

impl<'a> PartialEq for ContentSteering<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.server_uri() == other.server_uri() && self.pathway_id() == other.pathway_id()
    }
}

impl<'a> TryFrom<unknown::Tag<'a>> for ContentSteering<'a> {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_attribute_list()?;
        let Some(server_uri) = attribute_list
            .get(SERVER_URI)
            .and_then(AttributeValue::quoted)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(SERVER_URI));
        };
        Ok(Self {
            server_uri: Cow::Borrowed(server_uri),
            pathway_id: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> ContentSteering<'a> {
    /// Constructs a new `ContentSteering` tag.
    fn new(attribute_list: ContentSteeringAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let ContentSteeringAttributeList {
            server_uri,
            pathway_id,
        } = attribute_list;
        Self {
            server_uri,
            pathway_id,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `ContentSteering` as such:
    /// ```
    /// # use quick_m3u8::tag::hls::ContentSteering;
    /// let content_steering = ContentSteering::builder()
    ///     .with_server_uri("https://example.com/steering.json")
    ///     .with_pathway_id("1234")
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `server_uri`. The
    /// following fail to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::ContentSteering;
    /// let content_steering = ContentSteering::builder().finish();
    /// ```
    pub fn builder() -> ContentSteeringBuilder<'a, ContentSteeringServerUriNeedsToBeSet> {
        ContentSteeringBuilder::new()
    }

    /// Corresponds to the `SERVER-URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn server_uri(&self) -> &str {
        &self.server_uri
    }

    /// Corresponds to the `PATHWAY-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn pathway_id(&self) -> Option<&str> {
        if let Some(pathway_id) = &self.pathway_id {
            Some(pathway_id)
        } else {
            self.attribute_list
                .get(PATHWAY_ID)
                .and_then(AttributeValue::quoted)
        }
    }

    /// Sets the `SERVER-URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_server_uri(&mut self, server_uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(SERVER_URI);
        self.server_uri = server_uri.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `PATHWAY-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_pathway_id(&mut self, pathway_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(PATHWAY_ID);
        self.pathway_id = Some(pathway_id.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `PATHWAY-ID` attribute (set it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_pathway_id(&mut self) {
        self.attribute_list.remove(PATHWAY_ID);
        self.pathway_id = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&ContentSteeringAttributeList {
            server_uri: self.server_uri().into(),
            pathway_id: self.pathway_id().map(Into::into),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(ContentSteering);

const SERVER_URI: &str = "SERVER-URI";
const PATHWAY_ID: &str = "PATHWAY-ID";

fn calculate_line(attribute_list: &ContentSteeringAttributeList) -> Vec<u8> {
    let ContentSteeringAttributeList {
        server_uri,
        pathway_id,
    } = attribute_list;
    let mut line = format!(
        "#EXT{}:{}=\"{}\"",
        TagName::ContentSteering.as_str(),
        SERVER_URI,
        server_uri
    );
    if let Some(pathway_id) = pathway_id {
        line.push_str(format!(",{PATHWAY_ID}=\"{pathway_id}\"").as_str());
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn new_without_pathway_id_should_be_valid_line() {
        let tag = ContentSteering::builder()
            .with_server_uri("example.json")
            .finish();
        assert_eq!(
            b"#EXT-X-CONTENT-STEERING:SERVER-URI=\"example.json\"",
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_pathway_id_should_be_valid_line() {
        let tag = ContentSteering::builder()
            .with_server_uri("example.json")
            .with_pathway_id("1234")
            .finish();
        assert_eq!(
            b"#EXT-X-CONTENT-STEERING:SERVER-URI=\"example.json\",PATHWAY-ID=\"1234\"",
            tag.into_inner().value()
        );
    }

    mutation_tests!(
        ContentSteering::builder()
            .with_server_uri("server-uri.json")
            .with_pathway_id("1234")
            .finish(),
        (server_uri, "other-steering.json", @Attr="SERVER-URI=\"other-steering.json\""),
        (pathway_id, @Option "abcd", @Attr="PATHWAY-ID=\"abcd\"")
    );
}
