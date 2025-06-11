use crate::line::ParsedLineSlice;
use std::str::Chars;

pub fn take_until_end_of_line<'a>(
    mut input_chars: Chars<'a>,
) -> Result<ParsedLineSlice<'a, &'a str>, &'static str> {
    let input = input_chars.as_str();
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        match input_chars.next() {
            None => {
                return Ok(ParsedLineSlice {
                    parsed: &input[..(iterations - 1)],
                    remaining: None,
                });
            }
            Some('\r') => {
                validate_carriage_return(&mut input_chars)?;
                return Ok(ParsedLineSlice {
                    parsed: &input[..(iterations - 1)],
                    remaining: Some(input_chars.as_str()),
                });
            }
            Some('\n') => {
                return Ok(ParsedLineSlice {
                    parsed: &input[..(iterations - 1)],
                    remaining: Some(input_chars.as_str()),
                });
            }
            _ => (),
        }
    }
}

pub fn validate_carriage_return(input_chars: &mut Chars<'_>) -> Result<(), &'static str> {
    let Some('\n') = input_chars.next() else {
        return Err("Unexpected carriage return without following line feed");
    };
    Ok(())
}
