use crate::tag::{draft_pantos_hls, value::ParsedTagValue};
use std::{cmp::PartialEq, fmt::Debug};

#[derive(Debug, PartialEq)]
pub enum Tag<'a, CustomTag = NoCustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str> + IsKnownName + Debug + PartialEq,
{
    // Tag is in a Box based on the advice of cargo-clippy. The largest variant contains at least
    // 272 bytes; Boxing the large field (draft_pantos_hls::Tag) reduces the total size of the enum.
    // https://rust-lang.github.io/rust-clippy/master/index.html#large_enum_variant
    Hls(Box<draft_pantos_hls::Tag<'a>>),
    Custom(CustomTag),
}

pub trait IsKnownName {
    fn is_known_name(name: &str) -> bool;
}

pub struct ParsedTag<'a> {
    pub name: &'a str,
    pub value: ParsedTagValue<'a>,
}

#[derive(Debug, PartialEq)]
pub struct NoCustomTag;
impl TryFrom<ParsedTag<'_>> for NoCustomTag {
    type Error = &'static str;

    fn try_from(_: ParsedTag<'_>) -> Result<Self, Self::Error> {
        Err("No custom tag set.")
    }
}
impl IsKnownName for NoCustomTag {
    fn is_known_name(_: &str) -> bool {
        false
    }
}

impl<'a, CustomTag> TryFrom<ParsedTag<'a>> for Tag<'a, CustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str> + IsKnownName + Debug + PartialEq,
{
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        if CustomTag::is_known_name(tag.name) {
            Ok(Self::Custom(CustomTag::try_from(tag)?))
        } else {
            Ok(Self::Hls(Box::new(draft_pantos_hls::Tag::try_from(tag)?)))
        }
    }
}
