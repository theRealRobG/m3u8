use crate::{
    config::ParsingOptions,
    tag::{
        self,
        known::{self, IsKnownName, NoCustomTag, ParsedTag},
        unknown,
    },
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

pub fn parse(input: &str, options: ParsingOptions) -> IResult<&str, HlsLine> {
    parse_with_custom::<NoCustomTag>(input, options)
}

pub fn parse_with_custom<'a, CustomTag>(
    input: &'a str,
    options: ParsingOptions,
) -> IResult<&'a str, HlsLine<'a, CustomTag>>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str> + IsKnownName + Debug + PartialEq,
{
    // Attempt to parse tag, and if failed, pass back input for further parsing.
    let (input, opt_tag) = opt(complete::tag("#EXT")).parse(input)?;
    if opt_tag.is_some() {
        let (input, tag) = tag::unknown::parse(input)?;
        if options.is_known_name(tag.name) || CustomTag::is_known_name(tag.name) {
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
    use crate::{
        config::ParsingOptionsBuilder,
        tag::draft_pantos_hls::{self, m3u::M3u, start::Start},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn uri_line() {
        assert_eq!(
            Ok(("", HlsLine::Uri("hello/world.m3u8"))),
            parse("hello/world.m3u8", ParsingOptions::default())
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(
            Ok(("", HlsLine::Blank)),
            parse("", ParsingOptions::default())
        );
    }

    #[test]
    fn comment() {
        assert_eq!(
            Ok(("", HlsLine::Comment("Comment"))),
            parse("#Comment", ParsingOptions::default())
        );
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok((
                "",
                HlsLine::KnownTag(known::Tag::Hls(Box::new(draft_pantos_hls::Tag::M3u(M3u))))
            )),
            parse("#EXTM3U", ParsingOptions::default())
        );
    }

    #[test]
    fn custom_tag() {
        // Set up custom tag
        #[derive(Debug, PartialEq)]
        struct TestTag<'a> {
            greeting_type: &'a str,
            message: &'a str,
            times: u64,
            score: Option<f64>,
        }
        impl<'a> TryFrom<ParsedTag<'a>> for TestTag<'a> {
            type Error = &'static str;

            fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
                match tag.value {
                    tag::value::ParsedTagValue::AttributeList(list) => {
                        let Some(tag::value::ParsedAttributeValue::UnquotedString(greeting_type)) =
                            list.get("TYPE")
                        else {
                            return Err("Missing TYPE attriubte.");
                        };
                        let Some(tag::value::ParsedAttributeValue::QuotedString(message)) =
                            list.get("MESSAGE")
                        else {
                            return Err("Missing MESSAGE attriubte.");
                        };
                        let Some(tag::value::ParsedAttributeValue::DecimalInteger(times)) =
                            list.get("TIMES")
                        else {
                            return Err("Missing TIMES attriubte.");
                        };
                        let score = list
                            .get("SCORE")
                            .map(tag::value::ParsedAttributeValue::as_option_f64)
                            .flatten();
                        Ok(Self {
                            greeting_type,
                            message,
                            times: *times,
                            score,
                        })
                    }
                    _ => Err("Unexpected tag value."),
                }
            }
        }
        impl IsKnownName for TestTag<'_> {
            fn is_known_name(name: &str) -> bool {
                name == "-X-TEST-TAG"
            }
        }
        // Test
        assert_eq!(
            Ok((
                "",
                HlsLine::KnownTag(known::Tag::Custom(TestTag {
                    greeting_type: "GREETING",
                    message: "Hello, World!",
                    times: 42,
                    score: None,
                }))
            )),
            parse_with_custom::<TestTag>(
                "#EXT-X-TEST-TAG:TYPE=GREETING,MESSAGE=\"Hello, World!\",TIMES=42",
                ParsingOptions::default()
            )
        );
    }

    #[test]
    fn avoiding_parsing_known_tag_when_configured_to_avoid_via_parsing_options() {
        assert_eq!(
            Ok((
                "",
                HlsLine::KnownTag(known::Tag::Hls(Box::new(draft_pantos_hls::Tag::Start(
                    Start {
                        time_offset: -18.0,
                        precise: false
                    }
                ))))
            )),
            parse("#EXT-X-START:TIME-OFFSET=-18", ParsingOptions::default())
        );
        assert_eq!(
            Ok((
                "",
                HlsLine::UnknownTag(unknown::Tag {
                    name: "-X-START",
                    value: "TIME-OFFSET=-18"
                })
            )),
            parse(
                "#EXT-X-START:TIME-OFFSET=-18",
                ParsingOptionsBuilder::new()
                    .with_parsing_for_all_tags()
                    .without_parsing_for_start()
                    .build()
            )
        );
    }
}
