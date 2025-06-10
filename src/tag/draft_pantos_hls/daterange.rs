use crate::{
    date::{self, DateTime},
    tag::value::{ParsedAttributeValue, ParsedTagValue},
};
use std::collections::HashMap;

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.1
#[derive(Debug, PartialEq)]
pub struct Daterange<'a> {
    pub id: &'a str,
    pub class: Option<&'a str>,
    pub start_date: DateTime,
    pub cue: Option<&'a str>,
    pub end_date: Option<DateTime>,
    pub duration: Option<f64>,
    pub planned_duration: Option<f64>,
    pub client_attributes: HashMap<&'a str, ParsedAttributeValue<'a>>,
    pub scte35_cmd: Option<&'a str>,
    pub scte35_out: Option<&'a str>,
    pub scte35_in: Option<&'a str>,
    pub end_on_next: bool,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Daterange<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(mut attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(id)) = attribute_list.remove("ID") else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let class = match attribute_list.remove("CLASS") {
            Some(ParsedAttributeValue::QuotedString(class)) => Some(class),
            _ => None,
        };
        let Some(start_date) = (match attribute_list.remove("START-DATE") {
            Some(ParsedAttributeValue::QuotedString(date_str)) => date::parse(date_str).ok(),
            _ => None,
        }) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let cue = match attribute_list.remove("CUE") {
            Some(ParsedAttributeValue::QuotedString(cue)) => Some(cue),
            _ => None,
        };
        let end_date = match attribute_list.remove("END-DATE") {
            Some(ParsedAttributeValue::QuotedString(date_str)) => date::parse(date_str).ok(),
            _ => None,
        };
        let duration = match attribute_list.remove("DURATION") {
            Some(d) => d.as_option_f64(),
            _ => None,
        };
        let planned_duration = match attribute_list.remove("PLANNED-DURATION") {
            Some(d) => d.as_option_f64(),
            _ => None,
        };
        // The specification indicates that the SCTE35-(CMD|OUT|IN) attributes are
        // represented as hexadecimal sequences. This implies that they should be parsed as
        // UnquotedString (given that section "4.2. Attribute Lists" indicates that a
        // "hexadecimal-sequence [is] an unquoted string of characters"); however, in
        // practice, I've found that some packagers have put this information in quoted
        // strings (containing the hexadecimal sequence), so I'll allow this parser to be
        // lenient on that requirement and accept both.
        let scte35_cmd = match attribute_list.remove("SCTE35-CMD") {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        let scte35_out = match attribute_list.remove("SCTE35-OUT") {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        let scte35_in = match attribute_list.remove("SCTE35-IN") {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        };
        let end_on_next = matches!(
            attribute_list.remove("END-ON-NEXT"),
            Some(ParsedAttributeValue::UnquotedString("YES"))
        );

        // Deal with client attributes last as I will drain the rest of the HashMap.
        let mut client_attributes = HashMap::new();
        for (key, value) in attribute_list.drain() {
            if key.starts_with("X-") {
                client_attributes.insert(key, value);
            }
        }
        Ok(Self {
            id,
            class,
            start_date,
            cue,
            end_date,
            duration,
            planned_duration,
            client_attributes,
            scte35_cmd,
            scte35_out,
            scte35_in,
            end_on_next,
        })
    }
}
