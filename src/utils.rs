use std::borrow::Cow;

use crate::{
    date::{DateTime, DateTimeTimezoneOffset},
    error::{DateTimeSyntaxError, GenericSyntaxError, ParseNumberError},
    line::ParsedByteSlice,
};
use memchr::{memchr, memchr3};

pub trait AsStaticCow {
    fn as_cow(&self) -> Cow<'static, str>;
}

pub fn split_on_new_line<'a>(bytes: &'a [u8]) -> ParsedByteSlice<'a, &'a [u8]> {
    match memchr(b'\n', bytes) {
        Some(n) if n > 0 && bytes[n - 1] == b'\r' => ParsedByteSlice {
            parsed: &bytes[..(n - 1)],
            remaining: Some(&bytes[(n + 1)..]),
        },
        Some(n) => ParsedByteSlice {
            parsed: &bytes[..n],
            remaining: Some(&bytes[(n + 1)..]),
        },
        None => ParsedByteSlice {
            parsed: bytes,
            remaining: None,
        },
    }
}

pub(crate) fn str_from(bytes: &[u8]) -> &str {
    unsafe {
        // SAFETY: The input for bytes is always &str in this project, and I only break on single
        // byte characters, so this is safe to do unchecked.
        std::str::from_utf8_unchecked(bytes)
    }
}

pub fn parse_date_time_bytes<'a>(
    input: &'a [u8],
) -> Result<ParsedByteSlice<'a, DateTime>, DateTimeSyntaxError> {
    match input.get(4) {
        Some(b'-') => (),
        b => {
            return Err(DateTimeSyntaxError::UnexpectedYearToMonthSeparator(
                b.copied(),
            ));
        }
    };
    let date_fullyear = parse_u32(&input[..4]).map_err(DateTimeSyntaxError::InvalidYear)?;
    match input.get(7) {
        Some(b'-') => (),
        b => {
            return Err(DateTimeSyntaxError::UnexpectedMonthToDaySeparator(
                b.copied(),
            ));
        }
    };
    let date_month = parse_u8(&input[5..7]).map_err(DateTimeSyntaxError::InvalidMonth)?;
    match input.get(10) {
        // As per the note in https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
        // > NOTE: ISO 8601 defines date and time separated by "T". Applications using this syntax
        // > may choose, for the sake of readability, to specify a full-date and full-time separated
        // by (say) a space character.
        Some(b't') | Some(b'T') | Some(b' ') => (),
        b => return Err(DateTimeSyntaxError::UnexpectedDayHourSeparator(b.copied())),
    };
    let date_mday = parse_u8(&input[8..10]).map_err(DateTimeSyntaxError::InvalidDay)?;
    match input.get(13) {
        Some(b':') => (),
        b => {
            return Err(DateTimeSyntaxError::UnexpectedHourMinuteSeparator(
                b.copied(),
            ));
        }
    }
    let time_hour = parse_u8(&input[11..13]).map_err(DateTimeSyntaxError::InvalidHour)?;
    match input.get(16) {
        Some(b':') => (),
        b => {
            return Err(DateTimeSyntaxError::UnexpectedMinuteSecondSeparator(
                b.copied(),
            ));
        }
    };
    let time_minute = parse_u8(&input[14..16]).map_err(DateTimeSyntaxError::InvalidMinute)?;
    let time_offset_byte_index = match memchr3(b'Z', b'+', b'-', &input[16..]) {
        Some(n) => n + 16,
        None => match memchr(b'z', &input[16..]) {
            Some(n) => n + 16,
            None => return Err(GenericSyntaxError::UnexpectedEndOfLine)?,
        },
    };
    let time_offset_byte = input[time_offset_byte_index];
    let time_second = fast_float2::parse(&input[17..time_offset_byte_index])
        .map_err(|_| DateTimeSyntaxError::InvalidSecond)?;
    match time_offset_byte {
        b'Z' | b'z' => {
            let remaining = if input.get(time_offset_byte_index + 1).is_some() {
                split_on_new_line(&input[(time_offset_byte_index + 1)..])
            } else {
                ParsedByteSlice {
                    parsed: b"" as &[u8],
                    remaining: None,
                }
            };
            if !remaining.parsed.is_empty() {
                return Err(DateTimeSyntaxError::UnexpectedCharactersAfterTimezone);
            };
            let remaining = remaining.remaining;
            Ok(ParsedByteSlice {
                parsed: DateTime {
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
                },
                remaining,
            })
        }
        _ => {
            let multiplier = if time_offset_byte == b'-' { -1i8 } else { 1i8 };
            match input.get(time_offset_byte_index + 3) {
                Some(b':') => (),
                b => {
                    return Err(DateTimeSyntaxError::UnexpectedTimezoneHourMinuteSeparator(
                        b.copied(),
                    ));
                }
            };
            let timeoffset_hour =
                parse_u8(&input[(time_offset_byte_index + 1)..(time_offset_byte_index + 3)])
                    .map_err(DateTimeSyntaxError::InvalidTimezoneHour)? as i8;
            let timeoffset_hour = multiplier * timeoffset_hour;
            match input.get(time_offset_byte_index + 4) {
                Some(_) => (),
                None => return Err(GenericSyntaxError::UnexpectedEndOfLine)?,
            };
            let ParsedByteSlice { parsed, remaining } =
                split_on_new_line(&input[(time_offset_byte_index + 4)..]);
            let timeoffset_minute =
                parse_u8(parsed).map_err(DateTimeSyntaxError::InvalidTimezoneMinute)?;
            Ok(ParsedByteSlice {
                parsed: DateTime {
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
                },
                remaining,
            })
        }
    }
}

// Copied from https://stackoverflow.com/a/74629224/7039100
///////////////////////////////////////////////////////////
const F64_BITS: u64 = 64;
const F64_EXPONENT_BITS: u64 = 11;
const F64_EXPONENT_MAX: u64 = (1 << F64_EXPONENT_BITS) - 1;
const F64_EXPONENT_BIAS: u64 = 1023;
const F64_FRACTION_BITS: u64 = 52;

pub fn f64_to_u64(f: f64) -> Option<u64> {
    let bits = f.to_bits();
    let sign = bits & (1 << (F64_EXPONENT_BITS + F64_FRACTION_BITS)) != 0;
    let exponent = (bits >> F64_FRACTION_BITS) & ((1 << F64_EXPONENT_BITS) - 1);
    let fraction = bits & ((1 << F64_FRACTION_BITS) - 1);
    match (sign, exponent, fraction) {
        (_, 0, 0) => {
            debug_assert!(f == 0.0);
            Some(0)
        }
        (true, _, _) => {
            debug_assert!(f < 0.0);
            None
        }
        (_, F64_EXPONENT_MAX, 0) => {
            debug_assert!(f.is_infinite());
            None
        }
        (_, F64_EXPONENT_MAX, _) => {
            debug_assert!(f.is_nan());
            None
        }
        (_, 0, _) => {
            debug_assert!(f.is_subnormal());
            None
        }
        _ => {
            if exponent < F64_EXPONENT_BIAS {
                debug_assert!(f < 1.0);
                None
            } else {
                let mantissa = fraction | (1 << F64_FRACTION_BITS);
                let left_shift = exponent as i64 - (F64_EXPONENT_BIAS + F64_FRACTION_BITS) as i64;
                if left_shift < 0 {
                    let right_shift = (-left_shift) as u64;
                    if mantissa & (1 << (right_shift - 1)) != 0 {
                        debug_assert!(f.fract() != 0.0);
                        None
                    } else {
                        Some(mantissa >> right_shift)
                    }
                } else if left_shift > (F64_BITS - F64_FRACTION_BITS - 1) as i64 {
                    debug_assert!(f > 2.0f64.powi(63));
                    None
                } else {
                    Some(mantissa << left_shift)
                }
            }
        }
    }
}
///////////////////////////////////////////////////////////

// Directly copied from https://users.rust-lang.org/t/parse-number-from-u8/104487/6
macro_rules! parse_num_impl {
    ($fn_name:ident -> $ty:ident) => {
        pub fn $fn_name(bytes: &[u8]) -> Result<$ty, ParseNumberError> {
            if bytes.is_empty() {
                return Err(ParseNumberError::Empty);
            }
            let mut n: $ty = 0;
            for &byte in bytes {
                let digit = match byte.checked_sub(b'0') {
                    None => return Err(ParseNumberError::InvalidDigit(byte)),
                    Some(digit) if digit > 9 => {
                        return Err(ParseNumberError::InvalidDigit(byte));
                    }
                    Some(digit) => {
                        debug_assert!((0..=9).contains(&digit));
                        $ty::from(digit)
                    }
                };
                n = n
                    .checked_mul(10)
                    .and_then(|n| n.checked_add(digit))
                    .ok_or_else(|| ParseNumberError::NumberTooBig)?;
            }
            Ok(n)
        }
    };
}

parse_num_impl!(parse_u64 -> u64);
parse_num_impl!(parse_u32 -> u32);
parse_num_impl!(parse_u8 -> u8);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date_time;
    use pretty_assertions::assert_eq;

    #[test]
    fn date_time_parse_with_space_for_day_hour_separator_still_works() {
        assert_eq!(
            date_time!(2025-08-02 T 20:33:45.123 -05:00),
            parse_date_time_bytes(b"2025-08-02 20:33:45.123-05:00")
                .unwrap()
                .parsed
        );
    }

    #[test]
    fn split_on_new_line_should_have_no_remaining_when_no_new_line() {
        assert_eq!(
            ParsedByteSlice {
                parsed: b"test" as &[u8],
                remaining: None,
            },
            split_on_new_line(b"test")
        );
    }

    #[test]
    fn split_on_new_line_should_remove_lf() {
        assert_eq!(
            ParsedByteSlice {
                parsed: b"test" as &[u8],
                remaining: Some(b"remaining"),
            },
            split_on_new_line(b"test\nremaining")
        );
    }

    #[test]
    fn split_on_new_line_should_remove_crlf() {
        assert_eq!(
            ParsedByteSlice {
                parsed: b"test" as &[u8],
                remaining: Some(b"remaining"),
            },
            split_on_new_line(b"test\r\nremaining")
        );
    }

    #[test]
    fn split_on_new_line_should_have_empty_remaining_if_lf_last_char() {
        assert_eq!(
            ParsedByteSlice {
                parsed: b"test" as &[u8],
                remaining: Some(b""),
            },
            split_on_new_line(b"test\n")
        );
    }

    #[test]
    fn split_on_new_line_should_have_empty_remaining_if_crlf_last_char() {
        assert_eq!(
            ParsedByteSlice {
                parsed: b"test" as &[u8],
                remaining: Some(b""),
            },
            split_on_new_line(b"test\r\n")
        );
    }

    #[test]
    fn split_on_new_line_when_lf_is_first_byte_should_not_panic() {
        assert_eq!(
            ParsedByteSlice {
                parsed: b"" as &[u8],
                remaining: Some(b"test"),
            },
            split_on_new_line(b"\ntest")
        );
    }

    // Copied from https://stackoverflow.com/a/74629224/7039100
    mod f64_to_u64_tests {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn zero() {
            assert_eq!(f64_to_u64(0.0), Some(0));
            assert_eq!(f64_to_u64(-0.0), Some(0));
        }

        #[test]
        fn positive() {
            assert_eq!(f64_to_u64(1.0), Some(1));
            assert_eq!(f64_to_u64(2.0), Some(2));
            assert_eq!(f64_to_u64(3.0), Some(3));
            assert_eq!(f64_to_u64(2.0f64.powi(52)), Some(1 << 52));
            assert_eq!(f64_to_u64(2.0f64.powi(53)), Some(1 << 53));
            assert_eq!(f64_to_u64(2.0f64.powi(63)), Some(1 << 63));
            assert_eq!(f64_to_u64(1.5 * 2.0f64.powi(63)), Some(11 << 62));
            assert_eq!(f64_to_u64(1.75 * 2.0f64.powi(63)), Some(111 << 61));
        }

        #[test]
        fn too_big() {
            assert_eq!(f64_to_u64(2.0f64.powi(64)), None);
        }

        #[test]
        fn fractional() {
            assert_eq!(f64_to_u64(0.5), None);
            assert_eq!(f64_to_u64(1.5), None);
            assert_eq!(f64_to_u64(2.5), None);
        }

        #[test]
        fn negative() {
            assert_eq!(f64_to_u64(-1.0), None);
            assert_eq!(f64_to_u64(-2.0), None);
            assert_eq!(f64_to_u64(-3.0), None);
            assert_eq!(
                f64_to_u64(-(2.0f64.powi(f64::MANTISSA_DIGITS as i32))),
                None
            );
        }

        #[test]
        fn infinity() {
            assert_eq!(f64_to_u64(f64::INFINITY), None);
            assert_eq!(f64_to_u64(-f64::INFINITY), None);
        }

        #[test]
        fn nan() {
            assert_eq!(f64_to_u64(f64::NAN), None);
        }
    }
}
