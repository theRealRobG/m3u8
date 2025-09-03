use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{
        UnknownTag,
        hls::{LazyAttribute, into_inner_tag},
    },
};
use std::{borrow::Cow, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-SKIP:<attribute-list>`).
#[derive(Debug, Clone)]
struct SkipAttributeList<'a> {
    /// Corresponds to the `SKIPPED-SEGMENTS` attribute.
    ///
    /// See [`Skip`] for a link to the HLS documentation for this attribute.
    skipped_segments: u64,
    /// Corresponds to the `RECENTLY-REMOVED-DATERANGES` attribute.
    ///
    /// See [`Skip`] for a link to the HLS documentation for this attribute.
    recently_removed_dateranges: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`SkipBuilder`] indicating that `skipped_segments` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct SkipSkippedSegmentsNeedsToBeSet;
/// Placeholder struct for [`SkipBuilder`] indicating that `skipped_segments` has been set.
#[derive(Debug, Clone, Copy)]
pub struct SkipSkippedSegmentsHasBeenSet;

/// A builder for convenience in constructing a [`Skip`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct SkipBuilder<'a, SkippedSegmentsStatus> {
    attribute_list: SkipAttributeList<'a>,
    skipped_segments_status: PhantomData<SkippedSegmentsStatus>,
}
impl<'a> SkipBuilder<'a, SkipSkippedSegmentsNeedsToBeSet> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: SkipAttributeList {
                skipped_segments: Default::default(),
                recently_removed_dateranges: Default::default(),
            },
            skipped_segments_status: PhantomData,
        }
    }
}
impl<'a> SkipBuilder<'a, SkipSkippedSegmentsHasBeenSet> {
    /// Finish building and construct the `Skip`.
    pub fn finish(self) -> Skip<'a> {
        Skip::new(self.attribute_list)
    }
}
impl<'a, SkippedSegmentsStatus> SkipBuilder<'a, SkippedSegmentsStatus> {
    /// Add the provided `skipped_segments` to the attributes built for `Skip`.
    pub fn with_skipped_segments(
        mut self,
        skipped_segments: u64,
    ) -> SkipBuilder<'a, SkipSkippedSegmentsHasBeenSet> {
        self.attribute_list.skipped_segments = skipped_segments;
        SkipBuilder {
            attribute_list: self.attribute_list,
            skipped_segments_status: PhantomData,
        }
    }
    /// Add the provided `recently_removed_dateranges` to the attributes built for `Skip`.
    pub fn with_recently_removed_dateranges(
        mut self,
        recently_removed_dateranges: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.attribute_list.recently_removed_dateranges = Some(recently_removed_dateranges.into());
        self
    }
}
impl<'a> Default for SkipBuilder<'a, SkipSkippedSegmentsNeedsToBeSet> {
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-SKIP` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-18#section-4.4.5.2>
#[derive(Debug, Clone)]
pub struct Skip<'a> {
    skipped_segments: u64,
    recently_removed_dateranges: LazyAttribute<'a, Cow<'a, str>>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Skip<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.skipped_segments() == other.skipped_segments()
            && self.recently_removed_dateranges() == other.recently_removed_dateranges()
    }
}

impl<'a> TryFrom<UnknownTag<'a>> for Skip<'a> {
    type Error = ValidationError;

    fn try_from(tag: UnknownTag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_ordered_attribute_list()?;
        let mut skipped_segments = None;
        let mut recently_removed_dateranges = LazyAttribute::None;
        for (name, value) in attribute_list {
            match name {
                SKIPPED_SEGMENTS => {
                    skipped_segments = value
                        .unquoted()
                        .and_then(|v| v.try_as_decimal_integer().ok())
                }
                RECENTLY_REMOVED_DATERANGES => recently_removed_dateranges.found(value),
                _ => (),
            }
        }
        let Some(skipped_segments) = skipped_segments else {
            return Err(super::ValidationError::MissingRequiredAttribute(
                SKIPPED_SEGMENTS,
            ));
        };
        Ok(Self {
            skipped_segments,
            recently_removed_dateranges,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Skip<'a> {
    /// Constructs a new `Skip` tag.
    fn new(attribute_list: SkipAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let SkipAttributeList {
            skipped_segments,
            recently_removed_dateranges,
        } = attribute_list;
        Self {
            skipped_segments,
            recently_removed_dateranges: recently_removed_dateranges
                .map(LazyAttribute::new)
                .unwrap_or_default(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `Skip` as such:
    /// ```
    /// # use quick_m3u8::tag::hls::Skip;
    /// let skip = Skip::builder()
    ///     .with_skipped_segments(1000)
    ///     .with_recently_removed_dateranges("id_1\tid_2\tid_3")
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `skipped_segments`.
    /// The following fails to compile:
    /// ```compile_fail
    /// # use quick_m3u8::tag::hls::Skip;
    /// let skip = Skip::builder().finish();
    /// ```
    pub fn builder() -> SkipBuilder<'a, SkipSkippedSegmentsNeedsToBeSet> {
        SkipBuilder::new()
    }

    /// Corresponds to the `SKIPPED-SEGMENTS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn skipped_segments(&self) -> u64 {
        self.skipped_segments
    }

    /// Corresponds to the `RECENTLY-REMOVED-DATERANGES` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn recently_removed_dateranges(&self) -> Option<&str> {
        match &self.recently_removed_dateranges {
            LazyAttribute::UserDefined(s) => Some(s.as_ref()),
            LazyAttribute::Unparsed(v) => v.quoted(),
            LazyAttribute::None => None,
        }
    }

    /// Sets the `SKIPPED-SEGMENTS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_skipped_segments(&mut self, skipped_segments: u64) {
        self.skipped_segments = skipped_segments;
        self.output_line_is_dirty = true;
    }

    /// Sets the `RECENTLY-REMOVED-DATERANGES` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_recently_removed_dateranges(
        &mut self,
        recently_removed_dateranges: impl Into<Cow<'a, str>>,
    ) {
        self.recently_removed_dateranges
            .set(recently_removed_dateranges.into());
        self.output_line_is_dirty = true;
    }

    /// Unsets the `RECENTLY-REMOVED-DATERANGES` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_recently_removed_dateranges(&mut self) {
        self.recently_removed_dateranges.unset();
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&SkipAttributeList {
            skipped_segments: self.skipped_segments(),
            recently_removed_dateranges: self.recently_removed_dateranges().map(|x| x.into()),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Skip);

const SKIPPED_SEGMENTS: &str = "SKIPPED-SEGMENTS";
const RECENTLY_REMOVED_DATERANGES: &str = "RECENTLY-REMOVED-DATERANGES";

fn calculate_line(attribute_list: &SkipAttributeList) -> Vec<u8> {
    let SkipAttributeList {
        skipped_segments,
        recently_removed_dateranges,
    } = attribute_list;
    let mut line = format!("#EXT-X-SKIP:{SKIPPED_SEGMENTS}={skipped_segments}");
    if let Some(recently_removed_dateranges) = recently_removed_dateranges {
        line.push_str(
            format!(",{RECENTLY_REMOVED_DATERANGES}=\"{recently_removed_dateranges}\"").as_str(),
        );
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{IntoInnerTag, hls::test_macro::mutation_tests};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_recently_removed_dateranges_should_be_valid() {
        assert_eq!(
            b"#EXT-X-SKIP:SKIPPED-SEGMENTS=100",
            Skip::builder()
                .with_skipped_segments(100)
                .finish()
                .into_inner()
                .value()
        );
    }

    #[test]
    fn as_str_with_recently_removed_dateranges_shuold_be_valid() {
        assert_eq!(
            b"#EXT-X-SKIP:SKIPPED-SEGMENTS=100,RECENTLY-REMOVED-DATERANGES=\"abc\t123\"",
            Skip::builder()
                .with_skipped_segments(100)
                .with_recently_removed_dateranges("abc\t123")
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        Skip::builder()
            .with_skipped_segments(100)
            .with_recently_removed_dateranges("abc\t123")
            .finish(),
        (skipped_segments, 200, @Attr="SKIPPED-SEGMENTS=200"),
        (recently_removed_dateranges, @Option "efg", @Attr="RECENTLY-REMOVED-DATERANGES=\"efg\"")
    );
}
