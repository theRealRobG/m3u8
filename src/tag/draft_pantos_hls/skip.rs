use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.2
#[derive(Debug, PartialEq)]
pub struct Skip<'a> {
    pub skipped_segments: u64,
    pub recently_removed_dateranges: Option<&'a str>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Skip<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::DecimalInteger(skipped_segments)) =
            attribute_list.remove("SKIPPED-SEGMENTS")
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let recently_removed_dateranges = match attribute_list.remove("RECENTLY-REMOVED-DATERANGES")
        {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        Ok(Self {
            skipped_segments,
            recently_removed_dateranges,
        })
    }
}
