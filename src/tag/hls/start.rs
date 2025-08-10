use crate::{
    error::{ParseTagValueError, ValidationError},
    tag::{
        hls::into_inner_tag,
        unknown,
        value::{AttributeValue, UnquotedAttributeValue},
    },
};
use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

/// The attribute list for the tag (`#EXT-X-START:<attribute-list>`).
///
/// See [`Start`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy)]
struct StartAttributeList {
    /// Corresponds to the `TIME-OFFSET` attribute.
    ///
    /// See [`Start`] for a link to the HLS documentation for this attribute.
    time_offset: f64,
    /// Corresponds to the `PRECISE` attribute.
    ///
    /// See [`Start`] for a link to the HLS documentation for this attribute.
    precise: bool,
}

/// Placeholder struct for [`StartBuilder`] indicating that `time_offset` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct StartTimeOffsetNeedsToBeSet;
/// Placeholder struct for [`StartBuilder`] indicating that `time_offset` has been set.
#[derive(Debug, Clone, Copy)]
pub struct StartTimeOffsetHasBeenSet;

/// A builder for convenience in constructing a [`Start`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone, Copy)]
pub struct StartBuilder<TimeOffsetStatus> {
    attribute_list: StartAttributeList,
    time_offset_status: PhantomData<TimeOffsetStatus>,
}
impl StartBuilder<StartTimeOffsetNeedsToBeSet> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: StartAttributeList {
                time_offset: Default::default(),
                precise: Default::default(),
            },
            time_offset_status: PhantomData,
        }
    }
}
impl StartBuilder<StartTimeOffsetHasBeenSet> {
    /// Finish building and construct the `Start`.
    pub fn finish<'a>(self) -> Start<'a> {
        Start::new(self.attribute_list)
    }
}
impl<TimeOffsetStatus> StartBuilder<TimeOffsetStatus> {
    /// Add the provided `time_offset` to the attributes built into `Start`.
    pub fn with_time_offset(mut self, time_offset: f64) -> StartBuilder<StartTimeOffsetHasBeenSet> {
        self.attribute_list.time_offset = time_offset;
        StartBuilder {
            attribute_list: self.attribute_list,
            time_offset_status: PhantomData,
        }
    }
    /// Add the provided `precise` to the attributes built into `Start`.
    pub fn with_precise(mut self) -> Self {
        self.attribute_list.precise = true;
        self
    }
}
impl Default for StartBuilder<StartTimeOffsetNeedsToBeSet> {
    fn default() -> Self {
        Self::new()
    }
}

/// Corresponds to the `#EXT-X-START` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.2>
#[derive(Debug, Clone)]
pub struct Start<'a> {
    time_offset: f64,
    precise: Option<bool>,
    attribute_list: HashMap<&'a str, AttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                           // Used with Writer
    output_line_is_dirty: bool,                           // If should recalculate output_line
}

impl<'a> PartialEq for Start<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.time_offset() == other.time_offset() && self.precise() == other.precise()
    }
}

impl<'a> TryFrom<unknown::Tag<'a>> for Start<'a> {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_attribute_list()?;
        let Some(time_offset) = attribute_list
            .get(TIME_OFFSET)
            .and_then(AttributeValue::unquoted)
            .and_then(|v| v.try_as_decimal_floating_point().ok())
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(
                TIME_OFFSET,
            ));
        };
        Ok(Self {
            time_offset,
            precise: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Start<'a> {
    /// Constructs a new `Start` tag.
    fn new(attribute_list: StartAttributeList) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let StartAttributeList {
            time_offset,
            precise,
        } = attribute_list;
        Self {
            time_offset,
            precise: Some(precise),
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `Start` as such:
    /// ```
    /// # use m3u8::tag::hls::Start;
    /// let start = Start::builder()
    ///     .with_time_offset(-18.0)
    ///     .with_precise()
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `time_offset`. The
    /// following fails to compile:
    /// ```compile_fail
    /// # use m3u8::tag::hls::Start;
    /// let start = Start::builder().finish();
    /// ```
    pub fn builder() -> StartBuilder<StartTimeOffsetNeedsToBeSet> {
        StartBuilder::new()
    }

    /// Corresponds to the `TIME-OFFSET` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn time_offset(&self) -> f64 {
        self.time_offset
    }

    /// Corresponds to the `PRECISE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn precise(&self) -> bool {
        if let Some(precise) = self.precise {
            precise
        } else {
            matches!(
                self.attribute_list.get(PRECISE),
                Some(AttributeValue::Unquoted(UnquotedAttributeValue(YES)))
            )
        }
    }

    /// Sets the `TIME-OFFSET` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_time_offset(&mut self, time_offset: f64) {
        self.attribute_list.remove(TIME_OFFSET);
        self.time_offset = time_offset;
        self.output_line_is_dirty = true;
    }

    /// Sets the `PRECISE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_precise(&mut self, precise: bool) {
        self.attribute_list.remove(PRECISE);
        self.precise = Some(precise);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&StartAttributeList {
            time_offset: self.time_offset(),
            precise: self.precise(),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Start);

const TIME_OFFSET: &str = "TIME-OFFSET";
const PRECISE: &str = "PRECISE";
const YES: &[u8] = b"YES";

fn calculate_line(attribute_list: &StartAttributeList) -> Vec<u8> {
    let StartAttributeList {
        time_offset,
        precise,
    } = attribute_list;
    if *precise {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset},{PRECISE}=YES").into_bytes()
    } else {
        format!("#EXT-X-START:{TIME_OFFSET}={time_offset}").into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_without_precise_should_be_valid() {
        assert_eq!(
            b"#EXT-X-START:TIME-OFFSET=-42",
            Start::builder()
                .with_time_offset(-42.0)
                .finish()
                .into_inner()
                .value()
        )
    }

    #[test]
    fn as_str_with_precise_should_be_valid() {
        assert_eq!(
            b"#EXT-X-START:TIME-OFFSET=-42,PRECISE=YES",
            Start::builder()
                .with_time_offset(-42.0)
                .with_precise()
                .finish()
                .into_inner()
                .value()
        )
    }

    mutation_tests!(
        Start::builder().with_time_offset(-42.0).finish(),
        (time_offset, 10.0, @Attr="TIME-OFFSET=10"),
        (precise, true, @Attr="PRECISE=YES")
    );
}
