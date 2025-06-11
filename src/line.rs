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

pub fn another_thing() {
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    println!("Hello, World!");
    // println!("Hello, World!");
    //
    // Uncommenting the last println! results in a 20% (100µs) degradation in time with the
    // line_parse_bench. If I remove the `[profile.bench]` section from Cargo.toml then it only
    // requires 2 lines of println!.
    //
    // I've captured the difference in two flame graphs:
    //   - flamegraph-500µs.svg
    //   - flamegraph-600µs.svg
    //
    // The difference seems to be that in the faster time there are no calls to
    // `<nom::bytes::Tag<T,Error> as nom::internal::Parser<I>>::process` nor
    // `m3u8::tag::unknown::parse`, which suggests to me that those were being inlined, and are no
    // longer being inlined after uncommenting the last println! above.
    //
    // Can someone confirm that my interpretation is correct? Is this the main reason for the slow
    // down? And why is there this difference in inlining? And why does the setting of
    // `profile.bench.debug = true` increase the number of necessary println! statements before the
    // slow down?
    //
    // More info:
    //   - I run the bench via `cargo bench`.
    //   - I captured the flamegraph via `cargo flamegraph --bench line_parse_bench -- --bench`.
    //   - `cargo install --list` shows
    //     flamegraph v0.6.8:
    //         cargo-flamegraph
    //         flamegraph
    //   - My hardware:
    //       - Model Name: MacBook Pro
    //       - Model Identifier: MacBookPro18,2
    //       - Chip: Apple M1 Max
    //       - Total Number of Cores: 10 (8 performance and 2 efficiency)
    //       - Memory: 64 GB
    //       - System Firmware Version: 11881.101.1
    //       - OS Loader Version: 11881.101.1
    //   - My software:
    //       - System Version: macOS 15.4.1 (24E263)
    //       - Kernel Version: Darwin 24.4.0
    //   - Time taken with last println! commented consistently around 500µs.
    //   - Time taken with last println! uncommented consistently around 600µs.
}

pub fn parse<'a>(input: &'a str, options: &ParsingOptions) -> IResult<&'a str, HlsLine<'a>> {
    parse_with_custom::<NoCustomTag>(input, options)
}

pub fn parse_with_custom<'a, 'b, CustomTag>(
    input: &'a str,
    options: &'b ParsingOptions,
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
            parse("hello/world.m3u8", &ParsingOptions::default())
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(
            Ok(("", HlsLine::Blank)),
            parse("", &ParsingOptions::default())
        );
    }

    #[test]
    fn comment() {
        assert_eq!(
            Ok(("", HlsLine::Comment("Comment"))),
            parse("#Comment", &ParsingOptions::default())
        );
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok((
                "",
                HlsLine::KnownTag(known::Tag::Hls(draft_pantos_hls::Tag::M3u(M3u)))
            )),
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
                &ParsingOptions::default()
            )
        );
    }

    #[test]
    fn avoiding_parsing_known_tag_when_configured_to_avoid_via_parsing_options() {
        assert_eq!(
            Ok((
                "",
                HlsLine::KnownTag(known::Tag::Hls(draft_pantos_hls::Tag::Start(Start {
                    time_offset: -18.0,
                    precise: false
                })))
            )),
            parse("#EXT-X-START:TIME-OFFSET=-18", &ParsingOptions::default())
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
                &ParsingOptionsBuilder::new()
                    .with_parsing_for_all_tags()
                    .without_parsing_for_start()
                    .build()
            )
        );
    }
}
