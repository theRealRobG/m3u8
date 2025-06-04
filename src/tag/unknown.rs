use nom::{
    IResult,
    bytes::complete::{self, take_till},
};

#[derive(Debug, PartialEq)]
pub struct UnknownTag<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub fn parse(input: &str) -> IResult<&str, UnknownTag> {
    let (input, name) = take_till(|c| c == ':')(input)?;
    if input.starts_with(':') {
        let (value, _) = complete::tag(":")(input)?;
        Ok(("", UnknownTag { name, value }))
    } else {
        Ok((input, UnknownTag { name, value: "" }))
    }
}
