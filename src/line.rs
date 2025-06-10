use crate::{
    config::ParsingOptions,
    tag::{
        self,
        known::{self, IsKnownName, NoCustomTag, ParsedTag},
        unknown,
    },
};
use std::{cmp::PartialEq, fmt::Debug};

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)] // See comment on crate::tag::known::Tag.
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

pub fn parse<'a>(input: &'a str, options: &ParsingOptions) -> Result<HlsLine<'a>, &'static str> {
    parse_with_custom::<NoCustomTag>(input, options)
}

pub fn parse_with_custom<'a, 'b, CustomTag>(
    input: &'a str,
    options: &'b ParsingOptions,
) -> Result<HlsLine<'a, CustomTag>, &'static str>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str> + IsKnownName + Debug + PartialEq,
{
    // Attempt to parse tag, and if failed, pass back input for further parsing.
    let mut chars = input.chars();
    match chars.next() {
        Some('#') => {
            let Some('E') = chars.next() else {
                return Ok(HlsLine::Comment(&input[1..]));
            };
            let Some('X') = chars.next() else {
                return Ok(HlsLine::Comment(&input[1..]));
            };
            let Some('T') = chars.next() else {
                return Ok(HlsLine::Comment(&input[1..]));
            };
            let input = chars.as_str();
            let tag = tag::unknown::parse(input)?;
            if options.is_known_name(tag.name) || CustomTag::is_known_name(tag.name) {
                let tag_value = tag::value::parse(tag.value)?;
                let parsed_tag = ParsedTag {
                    name: tag.name,
                    value: tag_value,
                };
                Ok(HlsLine::KnownTag(known::Tag::try_from(parsed_tag)?))
            } else {
                Ok(HlsLine::UnknownTag(tag))
            }
        }
        None => Ok(HlsLine::Blank),
        Some('\r') | Some('\n') => {
            if chars.all(|c| c.is_ascii_whitespace()) {
                Ok(HlsLine::Blank)
            } else {
                Err("Unexpected whitespace in URI line")
            }
        }
        _ => Ok(HlsLine::Uri(input)),
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
            Ok(HlsLine::Uri("hello/world.m3u8")),
            parse("hello/world.m3u8", &ParsingOptions::default())
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(Ok(HlsLine::Blank), parse("", &ParsingOptions::default()));
    }

    #[test]
    fn comment() {
        assert_eq!(
            Ok(HlsLine::Comment("Comment")),
            parse("#Comment", &ParsingOptions::default())
        );
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok(HlsLine::KnownTag(known::Tag::Hls(
                draft_pantos_hls::Tag::M3u(M3u)
            ))),
            parse("#EXTM3U", &ParsingOptions::default())
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
            Ok(HlsLine::KnownTag(known::Tag::Custom(TestTag {
                greeting_type: "GREETING",
                message: "Hello, World!",
                times: 42,
                score: None,
            }))),
            parse_with_custom::<TestTag>(
                "#EXT-X-TEST-TAG:TYPE=GREETING,MESSAGE=\"Hello, World!\",TIMES=42",
                &ParsingOptions::default()
            )
        );
    }

    #[test]
    fn avoiding_parsing_known_tag_when_configured_to_avoid_via_parsing_options() {
        assert_eq!(
            Ok(HlsLine::KnownTag(known::Tag::Hls(
                draft_pantos_hls::Tag::Start(Start {
                    time_offset: -18.0,
                    precise: false
                })
            ))),
            parse("#EXT-X-START:TIME-OFFSET=-18", &ParsingOptions::default())
        );
        assert_eq!(
            Ok(HlsLine::UnknownTag(unknown::Tag {
                name: "-X-START",
                value: "TIME-OFFSET=-18"
            })),
            parse(
                "#EXT-X-START:TIME-OFFSET=-18",
                &ParsingOptionsBuilder::new()
                    .with_parsing_for_all_tags()
                    .without_parsing_for_start()
                    .build()
            )
        );
    }
}
