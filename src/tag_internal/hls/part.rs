use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{
        AttributeValue, DecimalIntegerRange, UnknownTag, UnquotedAttributeValue,
        hls::{LazyAttribute, into_inner_tag},
    },
};
use std::{borrow::Cow, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-PART:<attribute-list>`).
///
/// See [`Part`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct PartAttributeList<'a> {
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Part`] for a link to the HLS documentation for this attribute.
    uri: Cow<'a, str>,
    /// Corresponds to the `DURATION` attribute.
    ///
    /// See [`Part`] for a link to the HLS documentation for this attribute.
    duration: f64,
    /// Corresponds to the `INDEPENDENT` attribute.
    ///
    /// See [`Part`] for a link to the HLS documentation for this attribute.
    independent: bool,
    /// Corresponds to the `BYTERANGE` attribute.
    ///
    /// See [`Part`] for a link to the HLS documentation for this attribute.
    byterange: Option<DecimalIntegerRange>,
    /// Corresponds to the `GAP` attribute.
    ///
    /// See [`Part`] for a link to the HLS documentation for this attribute.
    gap: bool,
}

/// Placeholder struct for [`PartBuilder`] indicating that `uri` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct PartUriNeedsToBeSet;
/// Placeholder struct for [`PartBuilder`] indicating that `duration` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct PartDurationNeedsToBeSet;
/// Placeholder struct for [`PartBuilder`] indicating that `uri` has been set.
#[derive(Debug, Clone, Copy)]
pub struct PartUriHasBeenSet;
/// Placeholder struct for [`PartBuilder`] indicating that `duration` has been set.
#[derive(Debug, Clone, Copy)]
pub struct PartDurationHasBeenSet;

/// A builder for convenience in constructing a [`Part`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct PartBuilder<'a, UriStatus, DurationStatus> {
    attribute_list: PartAttributeList<'a>,
    uri_status: PhantomData<UriStatus>,
    duration_status: PhantomData<DurationStatus>,
}
impl<'a> PartBuilder<'a, PartUriNeedsToBeSet, PartDurationNeedsToBeSet> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: PartAttributeList {
                uri: Cow::Borrowed(""),
                duration: Default::default(),
                independent: Default::default(),
                byterange: Default::default(),
                gap: Default::default(),
            },
            uri_status: PhantomData,
            duration_status: PhantomData,
        }
    }
}
impl<'a> PartBuilder<'a, PartUriHasBeenSet, PartDurationHasBeenSet> {
    /// Finish building and construct the `Part`.
    pub fn finish(self) -> Part<'a> {
        Part::new(self.attribute_list)
    }
}
impl<'a, UriStatus, DurationStatus> PartBuilder<'a, UriStatus, DurationStatus> {
    /// Add the provided `uri` to the attributes built into `Part`.
    pub fn with_uri(
        mut self,
        uri: impl Into<Cow<'a, str>>,
    ) -> PartBuilder<'a, PartUriHasBeenSet, DurationStatus> {
        self.attribute_list.uri = uri.into();
        PartBuilder {
            attribute_list: self.attribute_list,
            uri_status: PhantomData,
            duration_status: PhantomData,
        }
    }
    /// Add the provided `duration` to the attributes built into `Part`.
    pub fn with_duration(
        mut self,
        duration: f64,
    ) -> PartBuilder<'a, UriStatus, PartDurationHasBeenSet> {
        self.attribute_list.duration = duration;
        PartBuilder {
            attribute_list: self.attribute_list,
            uri_status: PhantomData,
            duration_status: PhantomData,
        }
    }
    /// Add the provided `independent` to the attributes built into `Part`.
    pub fn with_independent(mut self) -> Self {
        self.attribute_list.independent = true;
        self
    }
    /// Add the provided `byterange` to the attributes built into `Part`.
    pub fn with_byterange(mut self, byterange: DecimalIntegerRange) -> Self {
        self.attribute_list.byterange = Some(byterange);
        self
    }
    /// Add the provided `gap` to the attributes built into `Part`.
    pub fn with_gap(mut self) -> Self {
        self.attribute_list.gap = true;
        self
    }
}
impl<'a> Default for PartBuilder<'a, PartUriNeedsToBeSet, PartDurationNeedsToBeSet> {
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-PART` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#section-4.4.4.9>
#[derive(Debug, Clone)]
pub struct Part<'a> {
    uri: Cow<'a, str>,
    duration: f64,
    independent: LazyAttribute<'a, bool>,
    byterange: LazyAttribute<'a, DecimalIntegerRange>,
    gap: LazyAttribute<'a, bool>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Part<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri() == other.uri()
            && self.duration() == other.duration()
            && self.independent() == other.independent()
            && self.byterange() == other.byterange()
            && self.gap() == other.gap()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for Part<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_ordered_attribute_list()?;
        let mut uri = None;
        let mut duration = None;
        let mut independent = LazyAttribute::None;
        let mut byterange = LazyAttribute::None;
        let mut gap = LazyAttribute::None;
        for (name, value) in attribute_list {
            match name {
                URI => uri = value.quoted(),
                DURATION => {
                    duration = value
                        .unquoted()
                        .and_then(|v| v.try_as_decimal_floating_point().ok())
                }
                INDEPENDENT => independent.found(value),
                BYTERANGE => byterange.found(value),
                GAP => gap.found(value),
                _ => (),
            }
        }
        let Some(uri) = uri else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        let Some(duration) = duration else {
            return Err(super::ValidationError::MissingRequiredAttribute(DURATION));
        };
        Ok(Self {
            uri: Cow::Borrowed(uri),
            duration,
            independent,
            byterange,
            gap,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Part<'a> {
    /// Constructs a new `Part` tag.
    fn new(attribute_list: PartAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let PartAttributeList {
            uri,
            duration,
            independent,
            byterange,
            gap,
        } = attribute_list;
        Self {
            uri,
            duration,
            independent: LazyAttribute::new(independent),
            byterange: byterange.map(LazyAttribute::new).unwrap_or_default(),
            gap: LazyAttribute::new(gap),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `Part` as such:
    /// ```
    /// # use quick_m3u8::tag::{DecimalIntegerRange, hls::Part};
    /// let part = Part::builder()
    ///     .with_uri("part.100.0.mp4")
    ///     .with_duration(0.5)
    ///     .with_independent()
    ///     .with_byterange(DecimalIntegerRange { length: 1024, offset: None })
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `uri` AND `duration`.
    /// Each of the following fail to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::Part;
    /// let part = Part::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::Part;
    /// let part = Part::builder().with_uri("uri").finish();
    /// ```
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::Part;
    /// let part = Part::builder().with_duration(0.5).finish();
    /// ```
    pub fn builder() -> PartBuilder<'a, PartUriNeedsToBeSet, PartDurationNeedsToBeSet> {
        PartBuilder::new()
    }

    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Corresponds to the `DURATION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// Corresponds to the `INDEPENDENT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn independent(&self) -> bool {
        match &self.independent {
            LazyAttribute::UserDefined(b) => *b,
            LazyAttribute::Unparsed(v) => {
                matches!(v, AttributeValue::Unquoted(UnquotedAttributeValue(YES)))
            }
            LazyAttribute::None => false,
        }
    }

    /// Corresponds to the `BYTERANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn byterange(&self) -> Option<DecimalIntegerRange> {
        match &self.byterange {
            LazyAttribute::UserDefined(b) => Some(*b),
            LazyAttribute::Unparsed(v) => v
                .quoted()
                .and_then(|s| DecimalIntegerRange::try_from(s).ok()),
            LazyAttribute::None => None,
        }
    }

    /// Corresponds to the `GAP` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn gap(&self) -> bool {
        match &self.gap {
            LazyAttribute::UserDefined(b) => *b,
            LazyAttribute::Unparsed(v) => {
                matches!(v, AttributeValue::Unquoted(UnquotedAttributeValue(YES)))
            }
            LazyAttribute::None => false,
        }
    }

    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.uri = uri.into();
        self.output_line_is_dirty = true;
    }
    /// Sets the `DURATION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
        self.output_line_is_dirty = true;
    }
    /// Sets the `INDEPENDENT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_independent(&mut self, independent: bool) {
        self.independent.set(independent);
        self.output_line_is_dirty = true;
    }
    /// Sets the `BYTERANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_byterange(&mut self, byterange: DecimalIntegerRange) {
        self.byterange.set(byterange);
        self.output_line_is_dirty = true;
    }
    /// Unsets the `BYTERANGE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_byterange(&mut self) {
        self.byterange.unset();
        self.output_line_is_dirty = true;
    }
    /// Sets the `GAP` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_gap(&mut self, gap: bool) {
        self.gap.set(gap);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&PartAttributeList {
            uri: self.uri().into(),
            duration: self.duration(),
            independent: self.independent(),
            byterange: self.byterange(),
            gap: self.gap(),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Part);

const URI: &str = "URI";
const DURATION: &str = "DURATION";
const INDEPENDENT: &str = "INDEPENDENT";
const BYTERANGE: &str = "BYTERANGE";
const GAP: &str = "GAP";
const YES: &[u8] = b"YES";

fn calculate_line(attribute_list: &PartAttributeList) -> Vec<u8> {
    let PartAttributeList {
        uri,
        duration,
        independent,
        byterange,
        gap,
    } = attribute_list;
    let mut line = format!("#EXT-X-PART:{URI}=\"{uri}\",{DURATION}={duration}");
    if *independent {
        line.push_str(",INDEPENDENT=YES");
    }
    if let Some(byterange) = byterange {
        line.push_str(format!(",{BYTERANGE}=\"{byterange}\"").as_str());
    }
    if *gap {
        line.push_str(",GAP=YES");
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5",
            Part::builder()
                .with_uri("part.1.0.mp4")
                .with_duration(0.5)
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_no_byterange_offset_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5,INDEPENDENT=YES,BYTERANGE=\"1024\",GAP=YES",
            Part::builder()
                .with_uri("part.1.0.mp4")
                .with_duration(0.5)
                .with_independent()
                .with_byterange(DecimalIntegerRange {
                    length: 1024,
                    offset: None
                })
                .with_gap()
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_options_with_byterange_offset_should_be_valid() {
        assert_eq!(
            b"#EXT-X-PART:URI=\"part.1.0.mp4\",DURATION=0.5,INDEPENDENT=YES,BYTERANGE=\"1024@512\",GAP=YES",
            Part::builder()
                .with_uri("part.1.0.mp4")
                .with_duration(0.5)
                .with_independent()
                .with_byterange(DecimalIntegerRange { length: 1024, offset: Some(512) })
                .with_gap()
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        Part::builder()
            .with_uri("part.1.0.mp4")
            .with_duration(0.5)
            .with_byterange(DecimalIntegerRange { length: 1024, offset: Some(512) })
            .finish(),
        (uri, "example", @Attr="URI=\"example\""),
        (duration, 1.0, @Attr="DURATION=1"),
        (independent, true, @Attr="INDEPENDENT=YES"),
        (
            byterange,
            @Option DecimalIntegerRange { length: 100, offset: Some(200) },
            @Attr="BYTERANGE=\"100@200\""
        ),
        (gap, true, @Attr="GAP=YES")
    );
}
