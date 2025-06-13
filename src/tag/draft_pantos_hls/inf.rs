use crate::tag::value::ParsedTagValue;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.4.1
#[derive(Debug, PartialEq)]
pub struct Inf<'a> {
    duration: f64,
    title: &'a str,
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

impl<'a> Inf<'a> {
    pub fn new(duration: f64, title: &'a str) -> Self {
        Self { duration, title }
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn title(&self) -> &'a str {
        self.title
    }
}
