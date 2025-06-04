use crate::tag::{
    self, draft_pantos_hls,
    known::{self, IsKnownName, NoCustomTag, ParsedTag},
    unknown,
};
use nom::{IResult, Parser, bytes::complete, combinator::opt};
use std::{cmp::PartialEq, fmt::Debug};

#[derive(Debug, PartialEq)]
pub enum HlsLine<'a, CustomTag = NoCustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str> + IsKnownName + Debug + PartialEq,
{
    KnownTag(known::Tag<'a, CustomTag>),
    UnknownTag(unknown::Tag<'a>),
    Comment(&'a str),
    Uri(&'a str),
    Blank,
}

pub fn parse<'a, CustomTag>(input: &'a str) -> IResult<&'a str, HlsLine<'a, CustomTag>>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str> + IsKnownName + Debug + PartialEq,
{
    // Attempt to parse tag, and if failed, pass back input for further parsing.
    let (input, opt_tag) = opt(complete::tag("#EXT")).parse(input)?;
    if opt_tag.is_some() {
        let (input, tag) = tag::unknown::parse(input)?;
        if draft_pantos_hls::Tag::is_known_name(tag.name) || CustomTag::is_known_name(tag.name) {
            let (input, tag_value) = tag::value::parse(tag.value)?;
            let parsed_tag = ParsedTag {
                name: tag.name,
                value: tag_value,
            };
            return match known::Tag::try_from(parsed_tag) {
                Ok(tag) => Ok((input, HlsLine::KnownTag(tag))),
                Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Alpha,
                ))),
            };
        } else {
            return Ok((input, HlsLine::UnknownTag(tag)));
        }
    }
    // Attempt to parse comment, and if failed, pass back input for further parsing.
    let (input, opt_comment) = opt(complete::tag("#")).parse(input)?;
    if opt_comment.is_some() {
        return Ok(("", HlsLine::Comment(input)));
    }
    if input.is_empty() {
        // If input is empty then this is a blank line.
        Ok((input, HlsLine::Blank))
    } else {
        // Otherwise this is considered a URI line.
        Ok(("", HlsLine::Uri(input)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::draft_pantos_hls::M3u;
    use pretty_assertions::assert_eq;

    #[test]
    fn uri_line() {
        assert_eq!(
            Ok(("", HlsLine::Uri("hello/world.m3u8"))),
            parse::<NoCustomTag>("hello/world.m3u8")
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(Ok(("", HlsLine::Blank)), parse::<NoCustomTag>(""));
    }

    #[test]
    fn comment() {
        assert_eq!(
            Ok(("", HlsLine::Comment("Comment"))),
            parse::<NoCustomTag>("#Comment")
        );
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok((
                "",
                HlsLine::KnownTag(known::Tag::Hls(draft_pantos_hls::Tag::M3u(M3u)))
            )),
            parse::<NoCustomTag>("#EXTM3U")
        );
    }

    #[test]
    fn custom_tag() {
        // Set up custom tag
        #[derive(Debug, PartialEq)]
        struct TestTag {
            number: u64,
        }
        impl TryFrom<ParsedTag<'_>> for TestTag {
            type Error = &'static str;

            fn try_from(tag: ParsedTag<'_>) -> Result<Self, Self::Error> {
                match tag.value {
                    tag::value::ParsedTagValue::DecimalInteger(number) => Ok(Self { number }),
                    _ => Err("Unexpected tag value."),
                }
            }
        }
        impl IsKnownName for TestTag {
            fn is_known_name(name: &str) -> bool {
                name == "-X-TEST-TAG"
            }
        }

        assert_eq!(
            Ok((
                "",
                HlsLine::KnownTag(known::Tag::Custom(TestTag { number: 42 }))
            )),
            parse::<TestTag>("#EXT-X-TEST-TAG:42")
        );
    }
}
