use nom::{
    IResult, Parser,
    bytes::{
        complete::{tag, take_while_m_n},
        take_while,
    },
    character::complete::one_of,
    combinator::{map_res, rest},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DateTime {
    pub date_fullyear: u32,
    pub date_month: u8,
    pub date_mday: u8,
    pub time_hour: u8,
    pub time_minute: u8,
    pub time_second: f64,
    pub timezone_offset: DateTimeTimezoneOffset,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DateTimeTimezoneOffset {
    pub time_hour: i8,
    pub time_minute: u8,
}

pub fn parse(input: &str) -> IResult<&str, DateTime> {
    map_res(
        (
            take_while_m_n(4, 4, |c: char| c.is_ascii_digit()),
            tag::<_, _, nom::error::Error<&str>>("-"),
            take_while_m_n(2, 2, |c: char| c.is_ascii_digit()),
            tag("-"),
            take_while_m_n(2, 2, |c: char| c.is_ascii_digit()),
            tag("T"),
            take_while_m_n(2, 2, |c: char| c.is_ascii_digit()),
            tag(":"),
            take_while_m_n(2, 2, |c: char| c.is_ascii_digit()),
            tag(":"),
            take_while(|c: char| c.is_ascii_digit() || c == '.'),
            rest,
        ),
        |(
            date_fullyear,
            _,
            date_month,
            _,
            date_mday,
            _,
            time_hour,
            _,
            time_minute,
            _,
            time_second,
            timezone_offset,
        )| {
            fn parse_error(_: std::num::ParseIntError) -> nom::error::Error<&'static str> {
                nom::error::Error::new("", nom::error::ErrorKind::Digit)
            }
            fn parse_float_error(_: std::num::ParseFloatError) -> nom::error::Error<&'static str> {
                nom::error::Error::new("", nom::error::ErrorKind::Float)
            }
            let date_fullyear = date_fullyear.parse::<u32>().map_err(parse_error)?;
            let date_month = date_month.parse::<u8>().map_err(parse_error)?;
            let date_mday = date_mday.parse::<u8>().map_err(parse_error)?;
            let time_hour = time_hour.parse::<u8>().map_err(parse_error)?;
            let time_minute = time_minute.parse::<u8>().map_err(parse_error)?;
            let time_second = time_second.parse::<f64>().map_err(parse_float_error)?;
            let (_, timezone_offset) = parse_time_offset(timezone_offset)
                .map_err(|e| nom::error::Error::new("", e.kind()))?;
            Ok::<DateTime, nom::error::Error<&str>>(DateTime {
                date_fullyear,
                date_month,
                date_mday,
                time_hour,
                time_minute,
                time_second,
                timezone_offset,
            })
        },
    )
    .parse(input)
}

fn parse_time_offset(input: &str) -> IResult<&str, DateTimeTimezoneOffset> {
    let (input, tag) = one_of("Z+-")(input)?;
    match tag {
        'Z' => Ok((
            input,
            DateTimeTimezoneOffset {
                time_hour: 0,
                time_minute: 0,
            },
        )),
        '+' => {
            let (input, (time_hour, time_minute)) = parse_hours_minutes_time(input)?;
            Ok((
                input,
                DateTimeTimezoneOffset {
                    time_hour,
                    time_minute,
                },
            ))
        }
        '-' => {
            let (input, (time_hour, time_minute)) = parse_hours_minutes_time(input)?;
            Ok((
                input,
                DateTimeTimezoneOffset {
                    time_hour: -time_hour,
                    time_minute,
                },
            ))
        }
        _ => panic!("Char must be 'Z', '+', or '-'."),
    }
}

fn parse_hours_minutes_time(input: &str) -> IResult<&str, (i8, u8)> {
    let (input, time_hour) = take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, time_minute) = take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)?;
    let Ok(time_hour) = time_hour.parse::<i8>() else {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Digit,
        )));
    };
    let Ok(time_minute) = time_minute.parse::<u8>() else {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Digit,
        )));
    };
    Ok((input, (time_hour, time_minute)))
}

trait GetKind {
    fn kind(&self) -> nom::error::ErrorKind;
}
impl<T> GetKind for nom::Err<nom::error::Error<T>> {
    fn kind(&self) -> nom::error::ErrorKind {
        match self {
            nom::Err::Incomplete(_) => nom::error::ErrorKind::Alpha,
            nom::Err::Error(error) => error.code,
            nom::Err::Failure(error) => error.code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn no_timezone() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.148,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            },
            parse("2025-06-04T13:50:42.148Z").unwrap().1
        );
    }

    #[test]
    fn plus_timezone() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.148,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 3,
                    time_minute: 0
                }
            },
            parse("2025-06-04T13:50:42.148+03:00").unwrap().1
        );
    }

    #[test]
    fn negative_timezone() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.148,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -1,
                    time_minute: 30
                }
            },
            parse("2025-06-04T13:50:42.148-01:30").unwrap().1
        );
    }

    #[test]
    fn no_fractional_seconds() {
        assert_eq!(
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 4,
                time_hour: 13,
                time_minute: 50,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: 0,
                    time_minute: 0
                }
            },
            parse("2025-06-04T13:50:42Z").unwrap().1
        );
    }
}
