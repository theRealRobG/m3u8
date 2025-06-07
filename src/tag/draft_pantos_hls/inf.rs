use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.1
#[derive(Debug, PartialEq)]
pub struct Inf<'a> {
    pub duration: f64,
    pub title: &'a str,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Inf<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        match value {
            ParsedTagValue::DecimalInteger(d) => Ok(Self {
                duration: d as f64,
                title: "",
            }),
            ParsedTagValue::DecimalFloatingPointWithOptionalTitle(duration, title) => {
                Ok(Self { duration, title })
            }
            _ => Err(super::ValidationError::unexpected_value_type()),
        }
    }
}
