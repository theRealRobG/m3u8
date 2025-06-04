use crate::tag::{self, unknown::UnknownTag};
use nom::{IResult, Parser, bytes::complete, combinator::opt};

#[derive(Debug, PartialEq)]
pub enum HlsLine<'a> {
    Tag(UnknownTag<'a>),
    Comment(&'a str),
    Uri(&'a str),
    Blank,
}

pub fn parse(input: &str) -> IResult<&str, HlsLine> {
    // Attempt to parse tag, and if failed, pass back input for further parsing.
    let (input, opt_tag) = opt(complete::tag("#EXT")).parse(input)?;
    if opt_tag.is_some() {
        let (rest, tag) = tag::unknown::parse(input)?;
        return Ok((rest, HlsLine::Tag(tag)));
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
    use pretty_assertions::assert_eq;

    #[test]
    fn uri_line() {
        assert_eq!(
            Ok(("", HlsLine::Uri("hello/world.m3u8"))),
            parse("hello/world.m3u8")
        )
    }

    #[test]
    fn blank_line() {
        assert_eq!(Ok(("", HlsLine::Blank)), parse(""));
    }

    #[test]
    fn comment() {
        assert_eq!(Ok(("", HlsLine::Comment("Comment"))), parse("#Comment"));
    }

    #[test]
    fn basic_tag() {
        assert_eq!(
            Ok((
                "",
                HlsLine::Tag(UnknownTag {
                    name: "M3U",
                    value: ""
                })
            )),
            parse("#EXTM3U")
        );
    }
}
