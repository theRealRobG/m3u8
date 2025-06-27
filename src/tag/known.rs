use crate::{
    error::ValidationError,
    tag::{hls, value::ParsedTagValue},
};
use std::{cmp::PartialEq, fmt::Debug};

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Tag<'a, CustomTag = NoCustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + PartialEq,
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
    Custom(CustomTag),
}

pub trait IsKnownName {
    fn is_known_name(name: &str) -> bool;
}

pub trait TagInformation {
    fn name(&self) -> &str;
    fn value(&self) -> ParsedTagValue;
}

#[derive(Debug, PartialEq)]
pub struct ParsedTag<'a> {
    pub name: &'a str,
    pub value: ParsedTagValue<'a>,
    pub(crate) original_input: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct NoCustomTag;
impl TryFrom<ParsedTag<'_>> for NoCustomTag {
    type Error = ValidationError;

    fn try_from(_: ParsedTag) -> Result<Self, Self::Error> {
        Err(ValidationError::NotImplemented)
    }
}
impl IsKnownName for NoCustomTag {
    fn is_known_name(_: &str) -> bool {
        false
    }
}
impl TagInformation for NoCustomTag {
    fn name(&self) -> &str {
        "-NO-TAG"
    }

    fn value(&self) -> ParsedTagValue {
        ParsedTagValue::Empty
    }
}

impl<'a, CustomTag> TryFrom<ParsedTag<'a>> for Tag<'a, CustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + PartialEq,
{
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        if CustomTag::is_known_name(tag.name) {
            Ok(Self::Custom(CustomTag::try_from(tag)?))
        } else {
            Ok(Self::Hls(hls::Tag::try_from(tag)?))
        }
    }
}
