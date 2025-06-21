use crate::{
    config::ParsingOptions,
    tag::{
        self,
        known::{self, IsKnownName, NoCustomTag, ParsedTag, TagInformation},
        unknown,
    },
    utils::{str_from, take_until_end_of_bytes},
};
use std::{cmp::PartialEq, fmt::Debug};

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)] // See comment on crate::tag::known::Tag.
pub enum HlsLine<'a, CustomTag = NoCustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str>
        + IsKnownName
        + TagInformation
        + Debug
        + PartialEq,
{
    KnownTag(known::Tag<'a, CustomTag>),
    UnknownTag(unknown::Tag<'a>),
    Comment(&'a str),
    Uri(&'a str),
    Blank,
}

#[derive(Debug, PartialEq)]
pub struct ParsedLineSlice<'a, T>
where
    T: Debug + PartialEq,
{
    pub parsed: T,
    pub remaining: Option<&'a str>,
}

pub fn parse<'a>(
    input: &'a str,
    options: &ParsingOptions,
) -> Result<ParsedLineSlice<'a, HlsLine<'a>>, &'static str> {
    parse_with_custom::<NoCustomTag>(input, options)
}

pub fn parse_with_custom<'a, 'b, CustomTag>(
    input: &'a str,
    options: &'b ParsingOptions,
) -> Result<ParsedLineSlice<'a, HlsLine<'a, CustomTag>>, &'static str>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = &'static str>
        + IsKnownName
        + TagInformation
        + Debug
        + PartialEq,
{
    // Attempt to parse tag, and if failed, pass back input for further parsing.
    let mut bytes = input.as_bytes().iter();
    match bytes.next() {
        Some(b'#') => {
            let Some(b'E') = bytes.next() else {
                let comment = take_until_end_of_bytes(input.as_bytes()[1..].iter())?;
                return Ok(ParsedLineSlice {
                    parsed: HlsLine::Comment(comment.parsed),
                    remaining: comment.remaining,
                });
            };
            let Some(b'X') = bytes.next() else {
                let comment = take_until_end_of_bytes(input.as_bytes()[1..].iter())?;
                return Ok(ParsedLineSlice {
                    parsed: HlsLine::Comment(comment.parsed),
                    remaining: comment.remaining,
                });
            };
            let Some(b'T') = bytes.next() else {
                let comment = take_until_end_of_bytes(input.as_bytes()[1..].iter())?;
                return Ok(ParsedLineSlice {
                    parsed: HlsLine::Comment(comment.parsed),
                    remaining: comment.remaining,
                });
            };
            let original_input = input;
            let input = str_from(bytes.as_slice());
            let tag = tag::unknown::parse_assuming_ext_taken(input, original_input)?;
            if options.is_known_name(tag.parsed.name) || CustomTag::is_known_name(tag.parsed.name) {
                let value_slice = match tag.remaining {
                    None => ParsedLineSlice {
                        parsed: tag::value::ParsedTagValue::Empty,
                        remaining: None,
                    },
                    Some(remaining) => tag::value::new_parse(remaining)?,
                };
                let parsed_tag = ParsedTag {
                    name: tag.parsed.name,
                    value: value_slice.parsed,
                    original_input,
                };
                Ok(ParsedLineSlice {
                    parsed: HlsLine::KnownTag(known::Tag::try_from(parsed_tag)?),
                    remaining: value_slice.remaining,
                })
            } else {
                Ok(ParsedLineSlice {
                    parsed: HlsLine::UnknownTag(tag.parsed),
                    remaining: tag.remaining,
                })
            }
        }
        None => Ok(ParsedLineSlice {
            parsed: HlsLine::Blank,
            remaining: None,
        }),
        _ => {
            let rest_of_line = take_until_end_of_bytes(input.as_bytes().iter())?;
            Ok(ParsedLineSlice {
                parsed: HlsLine::Uri(rest_of_line.parsed),
                remaining: rest_of_line.remaining,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        config::ParsingOptionsBuilder,
        tag::{
            hls::{self, m3u::M3u, start::Start},
            value::{ParsedAttributeValue, ParsedTagValue},
        },
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn uri_line() {
        assert_eq!(
            Ok(HlsLine::Uri("hello/world.m3u8")),
            parse("hello/world.m3u8", &ParsingOptions::default()).map(|p| p.parsed)
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(
            Ok(HlsLine::Blank),
            parse("", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn comment() {
        assert_eq!(
            Ok(HlsLine::Comment("Comment")),
            parse("#Comment", &ParsingOptions::default()).map(|p| p.parsed)
        );
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::M3u(M3u)))),
            parse("#EXTM3U", &ParsingOptions::default()).map(|p| p.parsed)
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
        impl<'a> TagInformation for TestTag<'a> {
            fn name(&self) -> &str {
                "-X-TEST-TAG"
            }

            fn value(&self) -> tag::value::ParsedTagValue {
                let mut attribute_list = HashMap::new();
                attribute_list.insert(
                    "TYPE",
                    ParsedAttributeValue::UnquotedString(self.greeting_type),
                );
                attribute_list.insert("MESSAGE", ParsedAttributeValue::QuotedString(self.message));
                attribute_list.insert("TIMES", ParsedAttributeValue::DecimalInteger(self.times));
                if let Some(score) = self.score {
                    attribute_list.insert(
                        "SCORE",
                        ParsedAttributeValue::SignedDecimalFloatingPoint(score),
                    );
                }
                ParsedTagValue::AttributeList(attribute_list)
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
            .map(|p| p.parsed)
        );
    }

    #[test]
    fn avoiding_parsing_known_tag_when_configured_to_avoid_via_parsing_options() {
        assert_eq!(
            Ok(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Start(
                Start::new(-18.0, false)
            )))),
            parse("#EXT-X-START:TIME-OFFSET=-18", &ParsingOptions::default()).map(|p| p.parsed)
        );
        assert_eq!(
            Ok(HlsLine::UnknownTag(unknown::Tag {
                name: "-X-START",
                remaining: Some("TIME-OFFSET=-18"),
                original_input: "#EXT-X-START:TIME-OFFSET=-18",
            })),
            parse(
                "#EXT-X-START:TIME-OFFSET=-18",
                &ParsingOptionsBuilder::new()
                    .with_parsing_for_all_tags()
                    .without_parsing_for_start()
                    .build()
            )
            .map(|p| p.parsed)
        );
    }
}
