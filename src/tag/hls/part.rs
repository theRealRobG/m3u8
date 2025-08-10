use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{
        hls::into_inner_tag,
        unknown,
        value::{AttributeValue, UnquotedAttributeValue},
    },
};
use std::{borrow::Cow, collections::HashMap, fmt::Display, marker::PhantomData};

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
    byterange: Option<PartByterange>,
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
    pub fn with_byterange(mut self, byterange: PartByterange) -> Self {
        self.attribute_list.byterange = Some(byterange);
        self
    }
    /// Add the provided `gap` to the attributes built into `Part`.
    pub fn with_gap(mut self) -> Self {
        self.attribute_list.gap = true;
        self
    }
}

/// Corresponds to the `#EXT-X-PART` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.9>
#[derive(Debug, Clone)]
pub struct Part<'a> {
    uri: Cow<'a, str>,
    duration: f64,
    independent: Option<bool>,
    byterange: Option<PartByterange>,
    gap: Option<bool>,
    attribute_list: HashMap<&'a str, AttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                           // Used with Writer
    output_line_is_dirty: bool,                           // If should recalculate output_line
}
/// Corresponds to the value of the `#EXT-X-PART:BYTERANGE` attribute.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PartByterange {
    /// Corresponds to the length component in the value (`n` in `<n>[@<o>]`).
    ///
    /// See [`Part`] for a link to the HLS documentation for this attribute.
    pub length: u64,
    /// Corresponds to the offset component in the value (`o` in `<n>[@<o>]`).
    ///
    /// See [`Part`] for a link to the HLS documentation for this attribute.
    pub offset: Option<u64>,
}
impl Display for PartByterange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(offset) = self.offset {
            write!(f, "{}@{}", self.length, offset)
        } else {
            write!(f, "{}", self.length)
        }
    }
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

impl<'a> TryFrom<unknown::Tag<'a>> for Part<'a> {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_attribute_list()?;
        let Some(uri) = attribute_list.get(URI).and_then(AttributeValue::quoted) else {
            return Err(super::ValidationError::MissingRequiredAttribute(URI));
        };
        let Some(duration) = attribute_list
            .get(DURATION)
            .and_then(AttributeValue::unquoted)
            .and_then(|v| v.try_as_decimal_floating_point().ok())
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(DURATION));
        };
        Ok(Self {
            uri: Cow::Borrowed(uri),
            duration,
            independent: None,
            byterange: None,
            gap: None,
            attribute_list,
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
            independent: Some(independent),
            byterange,
            gap: Some(gap),
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `Part` as such:
    /// ```
    /// # use m3u8::tag::hls::{Part, PartByterange};
    /// let part = Part::builder()
    ///     .with_uri("part.100.0.mp4")
    ///     .with_duration(0.5)
    ///     .with_independent()
    ///     .with_byterange(PartByterange { length: 1024, offset: None })
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `uri` AND `duration`.
    /// Each of the following fail to compile:
    /// ```compile_fail
    /// # use m3u8::tag::hls::Part;
    /// let part = Part::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Part;
    /// let part = Part::builder().with_uri("uri").finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Part;
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
        if let Some(independent) = self.independent {
            independent
        } else {
            matches!(
                self.attribute_list.get(INDEPENDENT),
                Some(AttributeValue::Unquoted(UnquotedAttributeValue(b"YES")))
            )
        }
    }

    /// Corresponds to the `BYTERANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn byterange(&self) -> Option<PartByterange> {
        if let Some(byterange) = self.byterange {
            Some(byterange)
        } else {
            self.attribute_list
                .get(BYTERANGE)
                .and_then(AttributeValue::quoted)
                .and_then(|range| {
                    let mut parts = range.splitn(2, '@');
                    let Some(Ok(length)) = parts.next().map(str::parse::<u64>) else {
                        return None;
                    };
                    let offset = match parts.next().map(str::parse::<u64>) {
                        Some(Ok(d)) => Some(d),
                        None => None,
                        Some(Err(_)) => return None,
                    };
                    Some(PartByterange { length, offset })
                })
        }
    }

    /// Corresponds to the `GAP` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn gap(&self) -> bool {
        if let Some(gap) = self.gap {
            gap
        } else {
            matches!(
                self.attribute_list.get(GAP),
                Some(AttributeValue::Unquoted(UnquotedAttributeValue(b"YES")))
            )
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
    /// Sets the `DURATION` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_duration(&mut self, duration: f64) {
        self.attribute_list.remove(DURATION);
        self.duration = duration;
        self.output_line_is_dirty = true;
    }
    /// Sets the `INDEPENDENT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_independent(&mut self, independent: bool) {
        self.attribute_list.remove(INDEPENDENT);
        self.independent = Some(independent);
        self.output_line_is_dirty = true;
    }
    /// Sets the `BYTERANGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_byterange(&mut self, byterange: PartByterange) {
        self.attribute_list.remove(BYTERANGE);
        self.byterange = Some(byterange);
        self.output_line_is_dirty = true;
    }
    /// Unsets the `BYTERANGE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_byterange(&mut self) {
        self.attribute_list.remove(BYTERANGE);
        self.byterange = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `GAP` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_gap(&mut self, gap: bool) {
        self.attribute_list.remove(GAP);
        self.gap = Some(gap);
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
const YES: &str = "YES";

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
        line.push_str(format!(",{INDEPENDENT}={YES}").as_str());
    }
    if let Some(byterange) = byterange {
        line.push_str(format!(",{BYTERANGE}=\"{byterange}\"").as_str());
    }
    if *gap {
        line.push_str(format!(",{GAP}={YES}").as_str());
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
                .with_byterange(PartByterange {
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
                .with_byterange(PartByterange { length: 1024, offset: Some(512) })
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
            .with_byterange(PartByterange { length: 1024, offset: Some(512) })
            .finish(),
        (uri, "example", @Attr="URI=\"example\""),
        (duration, 1.0, @Attr="DURATION=1"),
        (independent, true, @Attr="INDEPENDENT=YES"),
        (
            byterange,
            @Option PartByterange { length: 100, offset: Some(200) },
            @Attr="BYTERANGE=\"100@200\""
        ),
        (gap, true, @Attr="GAP=YES")
    );
}
