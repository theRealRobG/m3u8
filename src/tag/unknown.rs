use nom::{
    IResult,
    bytes::complete::{self, take_till},
};

#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub fn parse(input: &str) -> IResult<&str, Tag> {
    let (input, name) = take_till(|c| c == ':')(input)?;
    if input.starts_with(':') {
        let (value, _) = complete::tag(":")(input)?;
        Ok(("", Tag { name, value }))
    } else {
        Ok((input, Tag { name, value: "" }))
    }
}
