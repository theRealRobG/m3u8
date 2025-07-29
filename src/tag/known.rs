use crate::{
    error::ValidationError,
    tag::{
        hls,
        value::{MutableParsedAttributeValue, MutableSemiParsedTagValue, SemiParsedTagValue},
    },
    utils::split_on_new_line,
};
use std::{borrow::Cow, cmp::PartialEq, fmt::Debug};

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Tag<'a, Custom = NoCustomTag>
where
    Custom: CustomTag<'a>,
{
    // Clippy suggests that the `Tag` within the `Hls` case should be put in a Box, based on
    // https://rust-lang.github.io/rust-clippy/master/index.html#large_enum_variant
    //   > The largest variant contains at least 272 bytes; Boxing the large field
    //   > (hls::Tag) reduces the total size of the enum.
    //
    // However, the description also indicates:
    //   > This lint obviously cannot take the distribution of variants in your running program into
    //   > account. It is possible that the smaller variants make up less than 1% of all instances,
    //   > in which case the overhead is negligible and the boxing is counter-productive. Always
    //   > measure the change this lint suggests.
    //
    // In other words, the box only really makes sense, if there is a somewhat even distribution of
    // instances of each variant. If most instances are going to be the `Hls` case then we aren't
    // really saving on memory. Furthermore, putting the `Tag` in a `Box` incurrs a performance
    // penalty (validated with a Criterion bench), because we are now allocating and retrieving from
    // the heap.
    //
    // I believe that the vast majority of cases where the parser is being used we will be using
    // instances of the `Hls` variant, and therefore, I am not putting the `Tag` in a `Box` and so
    // ignoring the Clippy warning.
    Hls(hls::Tag<'a>),
    Custom(CustomTagAccess<'a, Custom>),
}

pub struct TagInner<'a> {
    pub(crate) output_line: Cow<'a, [u8]>,
}
impl<'a> TagInner<'a> {
    pub fn value(&self) -> &[u8] {
        split_on_new_line(&self.output_line).parsed
    }
}

pub trait IntoInnerTag<'a> {
    fn into_inner(self) -> TagInner<'a>;
}

/// Trait to define a custom tag implementation.
pub trait CustomTag<'a>:
    TryFrom<ParsedTag<'a>, Error = ValidationError> + Debug + PartialEq
{
    /// Check if the provided name is known for this custom tag implementation.
    ///
    /// This method is called before any attempt to parse the data into a CustomTag (it is the test
    /// for whether an attempt will be made to parse to CustomTag).
    fn is_known_name(name: &str) -> bool;
}
/// A custom tag implementation that allows for writing using [`crate::Writer`].
pub trait WritableCustomTag<'a>: CustomTag<'a> {
    /// Takes ownership of the custom tag and provides a value that is used for writing.
    ///
    /// This method is only called if there was a mutable borrow of the custom tag at some stage. If
    /// the tag was never mutably borrowed, then when writing, the library will use the original
    /// input data (thus avoiding unnecessary allocations).
    fn into_writable_tag(self) -> WritableTag<'a>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    pub(crate) custom_tag: Custom,
    pub(crate) is_dirty: bool,
    pub(crate) original_input: &'a [u8],
}

impl<'a, Custom> TryFrom<ParsedTag<'a>> for CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    type Error = ValidationError;

    fn try_from(value: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let original_input = value.original_input;
        let custom_tag = Custom::try_from(value)?;
        Ok(Self {
            custom_tag,
            is_dirty: false,
            original_input,
        })
    }
}
impl<'a, Custom> AsRef<Custom> for CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    fn as_ref(&self) -> &Custom {
        &self.custom_tag
    }
}
impl<'a, Custom> AsMut<Custom> for CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    fn as_mut(&mut self) -> &mut Custom {
        self.is_dirty = true;
        &mut self.custom_tag
    }
}

impl<'a, Custom> IntoInnerTag<'a> for CustomTagAccess<'a, Custom>
where
    Custom: WritableCustomTag<'a>,
{
    fn into_inner(self) -> TagInner<'a> {
        if self.is_dirty {
            self.custom_tag.into_inner()
        } else {
            TagInner {
                output_line: Cow::Borrowed(self.original_input),
            }
        }
    }
}

impl<'a, Custom> IntoInnerTag<'a> for Custom
where
    Custom: WritableCustomTag<'a>,
{
    fn into_inner(self) -> TagInner<'a> {
        let output = calculate_output(self);
        TagInner {
            output_line: Cow::Owned(output.into_bytes()),
        }
    }
}

pub(crate) fn calculate_output<'a, Custom: WritableCustomTag<'a>>(custom_tag: Custom) -> String {
    let tag = custom_tag.into_writable_tag();
    match tag.value {
        MutableSemiParsedTagValue::Empty => format!("#EXT{}", tag.name),
        MutableSemiParsedTagValue::DecimalFloatingPointWithOptionalTitle(n, t) => {
            if t.is_empty() {
                format!("#EXT{}:{}", tag.name, n)
            } else {
                format!("#EXT{}:{},{}", tag.name, n, t)
            }
        }
        MutableSemiParsedTagValue::AttributeList(list) => {
            let attrs = list
                .iter()
                .map(|(k, v)| match v {
                    MutableParsedAttributeValue::DecimalInteger(n) => format!("{k}={n}"),
                    MutableParsedAttributeValue::SignedDecimalFloatingPoint(n) => {
                        format!("{k}={n:?}")
                    }
                    MutableParsedAttributeValue::QuotedString(s) => format!("{k}=\"{s}\""),
                    MutableParsedAttributeValue::UnquotedString(s) => format!("{k}={s}"),
                })
                .collect::<Vec<String>>();
            let value = attrs.join(",");
            format!("#EXT{}:{}", tag.name, value)
        }
        MutableSemiParsedTagValue::Unparsed(bytes) => {
            format!(
                "#EXT{}:{}",
                tag.name,
                String::from_utf8_lossy(bytes.0.as_ref())
            )
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParsedTag<'a> {
    pub name: &'a str,
    pub value: SemiParsedTagValue<'a>,
    pub original_input: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct WritableTag<'a> {
    pub name: Cow<'a, str>,
    pub value: MutableSemiParsedTagValue<'a>,
}
impl<'a> WritableTag<'a> {
    pub fn new(
        name: impl Into<Cow<'a, str>>,
        value: impl Into<MutableSemiParsedTagValue<'a>>,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NoCustomTag;
impl TryFrom<ParsedTag<'_>> for NoCustomTag {
    type Error = ValidationError;

    fn try_from(_: ParsedTag) -> Result<Self, Self::Error> {
        Err(ValidationError::NotImplemented)
    }
}
impl CustomTag<'_> for NoCustomTag {
    fn is_known_name(_: &str) -> bool {
        false
    }
}
impl WritableCustomTag<'_> for NoCustomTag {
    fn into_writable_tag(self) -> WritableTag<'static> {
        WritableTag::new("-NO-TAG", SemiParsedTagValue::Empty)
    }
}

impl<'a, Custom> TryFrom<ParsedTag<'a>> for Tag<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        if Custom::is_known_name(tag.name) {
            let original_input = tag.original_input;
            let custom_tag = Custom::try_from(tag)?;
            Ok(Self::Custom(CustomTagAccess {
                custom_tag,
                is_dirty: false,
                original_input,
            }))
        } else {
            Ok(Self::Hls(hls::Tag::try_from(tag)?))
        }
    }
}

impl<'a, Custom> IntoInnerTag<'a> for Tag<'a, Custom>
where
    Custom: WritableCustomTag<'a>,
{
    fn into_inner(self) -> TagInner<'a> {
        match self {
            Tag::Hls(tag) => tag.into_inner(),
            Tag::Custom(tag) => tag.into_inner(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Reader, Writer, config::ParsingOptions, error::ValidationErrorValueKind, line::HlsLine,
        tag::value::ParsedAttributeValue,
    };
    use pretty_assertions::assert_eq;
    use std::marker::PhantomData;

    #[derive(Debug, PartialEq)]
    struct TestTag {
        mutated: bool,
    }
    impl TryFrom<ParsedTag<'_>> for TestTag {
        type Error = ValidationError;
        fn try_from(tag: ParsedTag<'_>) -> Result<Self, Self::Error> {
            let SemiParsedTagValue::AttributeList(list) = tag.value else {
                return Err(ValidationError::UnexpectedValueType(
                    ValidationErrorValueKind::from(&tag.value),
                ));
            };
            let Some(ParsedAttributeValue::UnquotedString(mutated_str)) = list.get("MUTATED")
            else {
                return Err(ValidationError::MissingRequiredAttribute("MUTATED"));
            };
            match *mutated_str {
                "NO" => Ok(Self { mutated: false }),
                "YES" => Ok(Self { mutated: true }),
                _ => Err(ValidationError::InvalidEnumeratedString),
            }
        }
    }
    impl CustomTag<'_> for TestTag {
        fn is_known_name(name: &str) -> bool {
            name == "-X-TEST-TAG"
        }
    }
    impl WritableCustomTag<'_> for TestTag {
        fn into_writable_tag(self) -> WritableTag<'static> {
            let value = if self.mutated { "YES" } else { "NO" };
            WritableTag::new(
                "-X-TEST-TAG",
                [(
                    "MUTATED",
                    MutableParsedAttributeValue::UnquotedString(value.into()),
                )],
            )
        }
    }

    #[test]
    fn custom_tag_should_be_mutable() {
        let data = "#EXT-X-TEST-TAG:MUTATED=NO";
        let mut reader =
            Reader::with_custom_from_str(data, ParsingOptions::default(), PhantomData::<TestTag>);
        let mut writer = Writer::new(Vec::new());
        match reader.read_line() {
            Ok(Some(HlsLine::KnownTag(Tag::Custom(mut tag)))) => {
                assert_eq!(false, tag.as_ref().mutated);
                tag.as_mut().mutated = true;
                assert_eq!(true, tag.as_ref().mutated);
                writer
                    .write_custom_line(HlsLine::from(tag))
                    .expect("should not fail write");
            }
            l => panic!("unexpected line {l:?}"),
        }
        assert_eq!(
            "#EXT-X-TEST-TAG:MUTATED=YES\n",
            std::str::from_utf8(&writer.into_inner()).expect("should be valid str")
        );
    }

    // This implementation we'll set the writable tag output to a value not related to the tag to
    // demonstrate that it is only accessed for the output when mutated.
    #[derive(Debug, PartialEq)]
    struct WeirdTag {
        number: f64,
    }
    impl TryFrom<ParsedTag<'_>> for WeirdTag {
        type Error = ValidationError;
        fn try_from(tag: ParsedTag<'_>) -> Result<Self, Self::Error> {
            let SemiParsedTagValue::Unparsed(v) = tag.value else {
                return Err(ValidationError::UnexpectedValueType(
                    ValidationErrorValueKind::from(&tag.value),
                ));
            };
            let Ok(number) = v.try_as_float() else {
                return Err(ValidationError::MissingRequiredAttribute("self"));
            };
            Ok(Self { number })
        }
    }
    impl CustomTag<'_> for WeirdTag {
        fn is_known_name(name: &str) -> bool {
            name == "-X-WEIRD-TAG"
        }
    }
    impl WritableCustomTag<'_> for WeirdTag {
        fn into_writable_tag(self) -> WritableTag<'static> {
            WritableTag::new("-X-WEIRD-TAG", [("SO-WEIRD", 999)])
        }
    }

    #[test]
    fn custom_tag_should_only_use_into_writable_tag_when_mutated() {
        let data = "#EXT-X-WEIRD-TAG:42";
        let mut reader =
            Reader::with_custom_from_str(data, ParsingOptions::default(), PhantomData::<WeirdTag>);
        let mut writer = Writer::new(Vec::new());
        match reader.read_line() {
            Ok(Some(HlsLine::KnownTag(Tag::Custom(tag)))) => {
                assert_eq!(42.0, tag.as_ref().number);
                writer
                    .write_custom_line(HlsLine::from(tag))
                    .expect("should not fail write");
            }
            l => panic!("unexpected line {l:?}"),
        }
        assert_eq!(
            "#EXT-X-WEIRD-TAG:42\n",
            std::str::from_utf8(&writer.into_inner()).expect("should be valid str")
        );

        // Now re-run the test with mutation
        let mut reader =
            Reader::with_custom_from_str(data, ParsingOptions::default(), PhantomData::<WeirdTag>);
        let mut writer = Writer::new(Vec::new());
        match reader.read_line() {
            Ok(Some(HlsLine::KnownTag(Tag::Custom(mut tag)))) => {
                assert_eq!(42.0, tag.as_ref().number);
                tag.as_mut().number = 69.0;
                writer
                    .write_custom_line(HlsLine::from(tag))
                    .expect("should not fail write");
            }
            l => panic!("unexpected line {l:?}"),
        }
        assert_eq!(
            "#EXT-X-WEIRD-TAG:SO-WEIRD=999\n",
            std::str::from_utf8(&writer.into_inner()).expect("should be valid str")
        );
    }
}
