use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::into_inner_tag,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-RENDITION-REPORT:<attribute-list>`).
///
/// See [`RenditionReport`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct RenditionReportAttributeList<'a> {
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`RenditionReport`] for a link to the HLS documentation for this attribute.
    uri: Cow<'a, str>,
    /// Corresponds to the `LAST-MSN` attribute.
    ///
    /// See [`RenditionReport`] for a link to the HLS documentation for this attribute.
    last_msn: u64,
    /// Corresponds to the `LAST-PART` attribute.
    ///
    /// See [`RenditionReport`] for a link to the HLS documentation for this attribute.
    last_part: Option<u64>,
}

/// Placeholder struct for [`RenditionReportBuilder`] indicating that `uri` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct RenditionReportUriNeedsToBeSet;
/// Placeholder struct for [`RenditionReportBuilder`] indicating that `last_msn` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct RenditionReportLastMsnNeedsToBeSet;
/// Placeholder struct for [`RenditionReportBuilder`] indicating that `uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct RenditionReportUriHasBeenSet;
/// Placeholder struct for [`RenditionReportBuilder`] indicating that `last_msn` has been set.
#[derive(Debug, Clone, Copy)]
pub struct RenditionReportLastMsnHasBeenSet;

/// A builder for convenience in constructing a [`RenditionReport`].
#[derive(Debug, Clone)]
pub struct RenditionReportBuilder<'a, UriStatus, LastMsnStatus> {
    attribute_list: RenditionReportAttributeList<'a>,
    uri_status: PhantomData<UriStatus>,
    last_msn_status: PhantomData<LastMsnStatus>,
}
impl<'a>
    RenditionReportBuilder<'a, RenditionReportUriNeedsToBeSet, RenditionReportLastMsnNeedsToBeSet>
{
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: RenditionReportAttributeList {
                uri: Cow::Borrowed(""),
                last_msn: Default::default(),
                last_part: Default::default(),
            },
            uri_status: PhantomData,
            last_msn_status: PhantomData,
        }
    }
}
impl<'a>
    RenditionReportBuilder<'a, RenditionReportUriHasBeenSet, RenditionReportLastMsnHasBeenSet>
{
    /// Finish building and construct the `RenditionReport`.
    pub fn finish(self) -> RenditionReport<'a> {
        RenditionReport::new(self.attribute_list)
    }
}
impl<'a, UriStatus, LastMsnStatus> RenditionReportBuilder<'a, UriStatus, LastMsnStatus> {
    /// Add the provided `uri` to the attributes built into `RenditionReport`.
    pub fn with_uri(
        mut self,
        uri: impl Into<Cow<'a, str>>,
    ) -> RenditionReportBuilder<'a, RenditionReportUriHasBeenSet, LastMsnStatus> {
        self.attribute_list.uri = uri.into();
        RenditionReportBuilder {
            attribute_list: self.attribute_list,
            uri_status: PhantomData,
            last_msn_status: PhantomData,
        }
    }
    /// Add the provided `last_msn` to the attributes built into `RenditionReport`.
    pub fn with_last_msn(
        mut self,
        last_msn: u64,
    ) -> RenditionReportBuilder<'a, UriStatus, RenditionReportLastMsnHasBeenSet> {
        self.attribute_list.last_msn = last_msn;
        RenditionReportBuilder {
            attribute_list: self.attribute_list,
            uri_status: PhantomData,
            last_msn_status: PhantomData,
        }
    }
    /// Add the provided `last_part` to the attributes built into `RenditionReport`.
    pub fn with_last_part(mut self, last_part: u64) -> Self {
        self.attribute_list.last_part = Some(last_part);
        self
    }
}

/// Corresponds to the `#EXT-X-RENDITION-REPORT` tag.
///
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
    /// Constructs a new `RenditionReport` tag.
    fn new(attribute_list: RenditionReportAttributeList<'a>) -> Self {
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

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `RenditionReport` as such:
    /// ```
    /// # use m3u8::tag::hls::RenditionReport;
    /// let rendition_report = RenditionReport::builder()
    ///     .with_uri("hi.m3u8")
    ///     .with_last_msn(100)
    ///     .with_last_part(3)
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `uri` AND `last_msn`.
    /// Each of the following fail to compile:
    /// ```compile_fail
    /// # use m3u8::tag::hls::RenditionReport;
    /// let rendition_report = RenditionReport::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::RenditionReport;
    /// let rendition_report = RenditionReport::builder().with_uri("hi.m3u8").finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::RenditionReport;
    /// let rendition_report = RenditionReport::builder().with_last_msn(100).finish();
    /// ```
    pub fn builder() -> RenditionReportBuilder<
        'a,
        RenditionReportUriNeedsToBeSet,
        RenditionReportLastMsnNeedsToBeSet,
    > {
        RenditionReportBuilder::new()
    }

    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Corresponds to the `LAST-MSN` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn last_msn(&self) -> u64 {
        self.last_msn
    }

    /// Corresponds to the `LAST-PART` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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

    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }

    /// Sets the `LAST-MSN` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_last_msn(&mut self, last_msn: u64) {
        self.attribute_list.remove(LAST_MSN);
        self.last_msn = last_msn;
        self.output_line_is_dirty = true;
    }

    /// Sets the `LAST-PART` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_last_part(&mut self, last_part: u64) {
        self.attribute_list.remove(LAST_PART);
        self.last_part = Some(last_part);
        self.output_line_is_dirty = true;
    }

    /// Unsets the `LAST-PART` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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
            RenditionReport::builder()
                .with_uri("low.m3u8")
                .with_last_msn(100)
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-RENDITION-REPORT:URI=\"low.m3u8\",LAST-MSN=100,LAST-PART=2",
            RenditionReport::builder()
                .with_uri("low.m3u8")
                .with_last_msn(100)
                .with_last_part(2)
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        RenditionReport::builder()
            .with_uri("low.m3u8")
            .with_last_msn(100)
            .with_last_part(2)
            .finish(),
        (uri, "high.m3u8", @Attr="URI=\"high.m3u8\""),
        (last_msn, 200, @Attr="LAST-MSN=200"),
        (last_part, @Option 3, @Attr="LAST-PART=3")
    );
}
