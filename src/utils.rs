use crate::{
    date::{DateTime, DateTimeTimezoneOffset},
    line::ParsedLineSlice,
    tag::value::ParsedTagValue,
};
use std::{borrow::Cow, slice::Iter};

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

pub(crate) fn str_from(bytes: &[u8]) -> &str {
    unsafe {
        // SAFETY: The input for bytes is always &str in this project, and I only break on single
        // byte characters, so this is safe to do unchecked.
        std::str::from_utf8_unchecked(bytes)
    }
}

/// Expectation is that bytes has already been iterated through until either `b't'` or `b':'`.
/// Calling this from the beginning of a DateTime will fail. Iterate partially through first and
/// then use this method for the rest.
pub fn parse_date_time_bytes<'a>(
    input: &'a str,
    mut bytes: Iter<'a, u8>,
    break_byte: u8,
) -> Result<ParsedLineSlice<'a, ParsedTagValue<'a>>, &'static str> {
    let date_bytes = input.as_bytes();
    let Ok(date_fullyear) = input[..4].parse::<u32>() else {
        return Err("Invalid year in DateTimeMsec value");
    };
    let Some(b'-') = date_bytes.get(4) else {
        return Err("Invalid DateTimeMsec value");
    };
    let Ok(date_month) = input[5..7].parse::<u8>() else {
        return Err("Invalid month in DateTimeMsec value");
    };
    let Some(b'-') = date_bytes.get(7) else {
        return Err("Invalid DateTimeMsec value");
    };
    let Ok(date_mday) = input[8..10].parse::<u8>() else {
        return Err("Invalid day in DateTimeMsec value");
    };
    if break_byte == b't' {
        let Some(b't') = date_bytes.get(10) else {
            return Err("Invalid DateTimeMsec value");
        };
        bytes.next();
        bytes.next();
        let Some(b':') = bytes.next() else {
            return Err("Invalid DateTimeMsec value");
        };
    } else {
        let Some(b'T') = date_bytes.get(10) else {
            return Err("Invalid DateTimeMsec value");
        };
    }
    let Ok(time_hour) = input[11..13].parse::<u8>() else {
        return Err("Invalid hour in DateTimeMsec value");
    };
    bytes.next();
    bytes.next();
    let Some(b':') = bytes.next() else {
        return Err("Invalid DateTimeMsec value");
    };
    let mut byte_count = 17;
    let Ok(time_minute) = input[14..16].parse::<u8>() else {
        return Err("Invalid minute in DateTimeMsec value");
    };
    let time_offset_byte = 'time_offset_loop: loop {
        let Some(&byte) = bytes.next() else {
            break 'time_offset_loop None;
        };
        byte_count += 1;
        match byte {
            b'Z' | b'z' | b'+' | b'-' => break 'time_offset_loop Some(byte),
            b'\r' | b'\n' => return Err("Unexpected end of line in DateTimeMsec value"),
            b'0'..=b'9' | b'.' => (),
            _ => return Err("Invalid second in DateTimeMsec value"),
        }
    };
    let Some(time_offset_byte) = time_offset_byte else {
        return Err("Unexpected end of line in DateTimeMsec value");
    };
    let Ok(time_second) = input[17..(byte_count - 1)].parse::<f64>() else {
        return Err("Invalid second in DateTimeMsec value");
    };
    match time_offset_byte {
        b'Z' | b'z' => {
            let remaining = take_until_end_of_bytes(bytes)?;
            if !remaining.parsed.is_empty() {
                return Err("Unexpected characteres after timezone in DateTimeMsec value");
            };
            let remaining = remaining.remaining;
            Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DateTimeMsec(DateTime {
                    date_fullyear,
                    date_month,
                    date_mday,
                    time_hour,
                    time_minute,
                    time_second,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: 0,
                        time_minute: 0,
                    },
                }),
                remaining,
            })
        }
        _ => {
            let multiplier = if time_offset_byte == b'-' { -1i8 } else { 1i8 };
            bytes.next();
            bytes.next();
            let Some(b':') = bytes.next() else {
                return Err("Invalid DateTimeMsec value");
            };
            let Ok(timeoffset_hour) = input[byte_count..(byte_count + 2)].parse::<i8>() else {
                return Err("Invalid time offset hour in DateTimeMsec value");
            };
            let timeoffset_hour = multiplier * timeoffset_hour;
            bytes.next();
            bytes.next();
            let remaining = take_until_end_of_bytes(bytes)?;
            if !remaining.parsed.is_empty() {
                return Err("Unexpected characteres after timezone in DateTimeMsec value");
            };
            let remaining = remaining.remaining;
            let Ok(timeoffset_minute) = input[(byte_count + 3)..(byte_count + 5)].parse::<u8>()
            else {
                return Err("Invalid time offset minute in DateTimeMsec value");
            };
            Ok(ParsedLineSlice {
                parsed: ParsedTagValue::DateTimeMsec(DateTime {
                    date_fullyear,
                    date_month,
                    date_mday,
                    time_hour,
                    time_minute,
                    time_second,
                    timezone_offset: DateTimeTimezoneOffset {
                        time_hour: timeoffset_hour,
                        time_minute: timeoffset_minute,
                    },
                }),
                remaining,
            })
        }
    }
}

struct Test<'a> {
    reference: Cow<'a, [u8]>,
}

impl Test<'_> {
    fn set<T>(&mut self, s: T)
    where
        T: Into<String>,
    {
        self.reference = Cow::Owned(s.into().into_bytes());
    }

    fn value(&self) -> &str {
        match self.reference {
            Cow::Borrowed(bytes) => str_from(bytes),
            Cow::Owned(ref bytes) => str_from(bytes.as_slice()),
        }
    }
}

impl<'a> From<&'a str> for Test<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            reference: Cow::Borrowed(value.as_bytes()),
        }
    }
}

impl From<String> for Test<'_> {
    fn from(value: String) -> Self {
        Self {
            reference: Cow::Owned(value.into_bytes()),
        }
    }
}

#[test]
fn robtest() {
    let x = "123";
    let mut test = Test::from(x);
    assert_eq!("123", test.value());
    test.set("abc");
    assert_eq!("abc", test.value());
    let thing = b"abcde";
    let index = thing
        .iter()
        .enumerate()
        .find_map(|(i, b)| if *b == b'c' { Some(i) } else { None })
        .unwrap();
    assert_eq!(*b"ab", thing[..index]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_str_eq};

    #[test]
    fn end_bytes_test() {
        let daterange_str_no_new_line = r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#;
        let daterange_str_crlf = concat!(
            r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#,
            "\r\n",
            r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#
        );
        let daterange_str_lf = concat!(
            r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#,
            "\n",
            r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#
        );
        let daterange_str_lf_no_next = concat!(
            r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#,
            "\n",
        );

        for str in [
            daterange_str_no_new_line,
            daterange_str_crlf,
            daterange_str_lf,
            daterange_str_lf_no_next,
        ] {
            let slice_1 = take_until_end_of_bytes(dbg!(str).as_bytes().iter()).unwrap();
            let slice_2 = take_until_end_of_bytes_2(str.as_bytes().iter()).unwrap();
            // Slice 1
            assert_str_eq!(daterange_str_no_new_line, slice_1.parsed);
            if str == daterange_str_no_new_line {
                assert_eq!(None, slice_1.remaining);
            } else if str == daterange_str_lf_no_next {
                assert_str_eq!("", slice_1.remaining.unwrap());
            } else {
                assert_str_eq!(daterange_str_no_new_line, slice_1.remaining.unwrap());
            }
            // Slice 2
            assert_str_eq!(daterange_str_no_new_line, slice_2.parsed);
            if str == daterange_str_no_new_line {
                assert_eq!(None, slice_2.remaining);
            } else if str == daterange_str_lf_no_next {
                assert_str_eq!("", slice_2.remaining.unwrap());
            } else {
                assert_str_eq!(daterange_str_no_new_line, slice_2.remaining.unwrap());
            }
        }
    }
}

fn up_to_end_1(bytes: &[u8]) -> &[u8] {
    let index = bytes.iter().enumerate().find_map(|(i, b)| {
        if *b == b'\n' || *b == b'\r' {
            Some(i)
        } else {
            None
        }
    });
    match index {
        Some(index) => &bytes[..index],
        None => &bytes,
    }
}

fn up_to_end_2(bytes: &[u8]) -> &[u8] {
    let mut enumerated = bytes.iter().enumerate();
    loop {
        match enumerated.next() {
            Some((i, b'\n')) => return &bytes[..i],
            Some((i, b'\r')) => return &bytes[..i],
            None => return &bytes,
            _ => (),
        }
    }
}

fn up_to_end_3(bytes: &[u8]) -> &[u8] {
    let mut iterations = 0usize;
    let mut bytes_iter = bytes.iter();
    loop {
        iterations += 1;
        match bytes_iter.next() {
            Some(b'\n') => return &bytes[..(iterations - 1)],
            Some(b'\r') => return &bytes[..(iterations - 1)],
            None => return &bytes,
            _ => (),
        }
    }
}

fn up_to_end_4(bytes: &[u8]) -> &[u8] {
    let found = bytes
        .iter()
        .enumerate()
        .find(|(_, b)| **b == b'\n' || **b == b'\r');
    match found {
        Some((index, _)) => &bytes[..index],
        None => &bytes,
    }
}

pub fn take_until_end_of_bytes_2<'a>(
    bytes: Iter<'a, u8>,
) -> Result<ParsedLineSlice<'a, &'a str>, &'static str> {
    let input = bytes.as_slice();
    let eol = bytes
        .enumerate()
        .find(|(_, b)| **b == b'\n' || **b == b'\r');
    match eol {
        Some((index, b'\n')) => Ok(ParsedLineSlice {
            parsed: str_from(&input[..index]),
            remaining: Some(str_from(&input[(index + 1)..])),
        }),
        Some((index, b'\r')) => {
            let Some(b'\n') = input.iter().nth(index + 1) else {
                return Err("Unexpected carriage return without following line feed");
            };
            Ok(ParsedLineSlice {
                parsed: str_from(&input[..index]),
                remaining: Some(str_from(&input[(index + 2)..])),
            })
        }
        None => Ok(ParsedLineSlice {
            parsed: str_from(input),
            remaining: None,
        }),
        _ => panic!("Impossible"),
    }
}
