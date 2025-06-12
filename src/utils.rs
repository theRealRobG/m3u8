use crate::line::ParsedLineSlice;
use std::slice::Iter;

pub fn take_until_end_of_bytes<'a>(
    mut bytes: Iter<'a, u8>,
) -> Result<ParsedLineSlice<'a, &'a str>, &'static str> {
    let input = bytes.as_slice();
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        match bytes.next() {
            None => {
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: None,
                });
            }
            Some(b'\r') => {
                validate_carriage_return_bytes(&mut bytes)?;
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: Some(str_from(bytes.as_slice())),
                });
            }
            Some(b'\n') => {
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: Some(str_from(bytes.as_slice())),
                });
            }
            _ => (),
        }
    }
}

pub fn validate_carriage_return_bytes(bytes: &mut Iter<'_, u8>) -> Result<(), &'static str> {
    let Some(b'\n') = bytes.next() else {
        return Err("Unexpected carriage return without following line feed");
    };
    Ok(())
}

pub fn str_from(bytes: &[u8]) -> &str {
    unsafe {
        // SAFETY: The input for bytes is always &str in this project, and I only break on single
        // byte characters, so this is safe to do unchecked.
        std::str::from_utf8_unchecked(bytes)
    }
}
