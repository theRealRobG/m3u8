use crate::{
    date::{self, DateTime},
    tag::value::{ParsedAttributeValue, ParsedTagValue},
};
use std::collections::{HashMap, HashSet};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.1
#[derive(Debug)]
pub struct Daterange<'a> {
    id: &'a str,
    start_date: DateTime,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
    // This needs to exist because the user can construct a Daterange with `Daterange::new()`, but
    // will pass a `DateTime`, not a `&str`. I can't convert a `DateTime` to a `&str` and so need to
    // store it as is for later use.
    stored_end_date: Option<DateTime>,
}

impl<'a> PartialEq for Daterange<'a> {
    fn eq(&self, other: &Self) -> bool {
        let equal_req_props = self.id == other.id && self.start_date == other.start_date;
        let equal_optional_props = self.class() == other.class()
            && self.cue() == other.cue()
            && self.end_date() == other.end_date()
            && self.duration() == other.duration()
            && self.planned_duration() == other.planned_duration()
            && self.end_on_next() == other.end_on_next()
            && self.scte35_cmd() == other.scte35_cmd()
            && self.scte35_out() == other.scte35_out()
            && self.scte35_in() == other.scte35_in();
        let equal_client_attrs = self.client_attribute_keys() == other.client_attribute_keys()
            && self
                .client_attribute_keys()
                .iter()
                .all(|key| self.client_attribute(key) == other.client_attribute(key));
        equal_req_props && equal_optional_props && equal_client_attrs
    }
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Daterange<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::QuotedString(id)) = attribute_list.get(ID) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(start_date) = (match attribute_list.get(START_DATE) {
            Some(ParsedAttributeValue::QuotedString(date_str)) => date::parse(date_str).ok(),
            _ => None,
        }) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            id,
            start_date,
            attribute_list,
            stored_end_date: None,
        })
    }
}

impl<'a> Daterange<'a> {
    pub fn new(
        id: &'a str,
        class: Option<&'a str>,
        start_date: DateTime,
        cue: Option<&'a str>,
        end_date: Option<DateTime>,
        duration: Option<f64>,
        planned_duration: Option<f64>,
        client_attributes: HashMap<&'a str, ParsedAttributeValue<'a>>,
        scte35_cmd: Option<&'a str>,
        scte35_out: Option<&'a str>,
        scte35_in: Option<&'a str>,
        end_on_next: bool,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        if let Some(class) = class {
            attribute_list.insert(CLASS, ParsedAttributeValue::QuotedString(class));
        }
        if let Some(cue) = cue {
            attribute_list.insert(CUE, ParsedAttributeValue::QuotedString(cue));
        }
        if let Some(duration) = duration {
            attribute_list.insert(
                DURATION,
                ParsedAttributeValue::SignedDecimalFloatingPoint(duration),
            );
        }
        if let Some(planned_duration) = planned_duration {
            attribute_list.insert(
                PLANNED_DURATION,
                ParsedAttributeValue::SignedDecimalFloatingPoint(planned_duration),
            );
        }
        for (key, value) in client_attributes {
            attribute_list.insert(key, value);
        }
        if let Some(scte35_cmd) = scte35_cmd {
            attribute_list.insert(SCTE35_CMD, ParsedAttributeValue::UnquotedString(scte35_cmd));
        }
        if let Some(scte35_out) = scte35_out {
            attribute_list.insert(SCTE35_OUT, ParsedAttributeValue::UnquotedString(scte35_out));
        }
        if let Some(scte35_in) = scte35_in {
            attribute_list.insert(SCTE35_IN, ParsedAttributeValue::UnquotedString(scte35_in));
        }
        if end_on_next {
            attribute_list.insert(END_ON_NEXT, ParsedAttributeValue::UnquotedString(YES));
        }
        Self {
            id,
            start_date,
            attribute_list,
            stored_end_date: end_date,
        }
    }

    pub fn id(&self) -> &str {
        self.id
    }

    pub fn class(&self) -> Option<&str> {
        match self.attribute_list.get(CLASS) {
            Some(ParsedAttributeValue::QuotedString(class)) => Some(class),
            _ => None,
        }
    }

    pub fn start_date(&self) -> DateTime {
        self.start_date
    }

    pub fn cue(&self) -> Option<&str> {
        match self.attribute_list.get(CUE) {
            Some(ParsedAttributeValue::QuotedString(cue)) => Some(cue),
            _ => None,
        }
    }

    pub fn end_date(&self) -> Option<DateTime> {
        if let Some(end_date) = self.stored_end_date {
            Some(end_date)
        } else {
            match self.attribute_list.get(END_DATE) {
                Some(ParsedAttributeValue::QuotedString(date_str)) => date::parse(date_str).ok(),
                _ => None,
            }
        }
    }

    pub fn duration(&self) -> Option<f64> {
        match self.attribute_list.get(DURATION) {
            Some(d) => d.as_option_f64(),
            _ => None,
        }
    }

    pub fn planned_duration(&self) -> Option<f64> {
        match self.attribute_list.get(PLANNED_DURATION) {
            Some(d) => d.as_option_f64(),
            _ => None,
        }
    }

    pub fn client_attribute<'b>(&'a self, name: &'b str) -> Option<&'a ParsedAttributeValue<'a>> {
        if !name.starts_with("X-") {
            return None;
        }
        self.attribute_list.get(name)
    }

    pub fn client_attribute_keys(&self) -> HashSet<&str> {
        let mut set = HashSet::new();
        for (&key, _) in &self.attribute_list {
            if key.starts_with("X-") {
                set.insert(key);
            }
        }
        set
    }

    pub fn end_on_next(&self) -> bool {
        matches!(
            self.attribute_list.get(END_ON_NEXT),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }

    // The specification indicates that the SCTE35-(CMD|OUT|IN) attributes are
    // represented as hexadecimal sequences. This implies that they should be parsed as
    // UnquotedString (given that section "4.2. Attribute Lists" indicates that a
    // "hexadecimal-sequence [is] an unquoted string of characters"); however, in
    // practice, I've found that some packagers have put this information in quoted
    // strings (containing the hexadecimal sequence), so I'll allow this parser to be
    // lenient on that requirement and accept both.

    pub fn scte35_cmd(&self) -> Option<&str> {
        match self.attribute_list.get(SCTE35_CMD) {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn scte35_out(&self) -> Option<&str> {
        match self.attribute_list.get(SCTE35_OUT) {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }

    pub fn scte35_in(&self) -> Option<&str> {
        match self.attribute_list.get(SCTE35_IN) {
            Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }
}

const ID: &'static str = "ID";
const CLASS: &'static str = "CLASS";
const START_DATE: &'static str = "START-DATE";
const CUE: &'static str = "CUE";
const END_DATE: &'static str = "END-DATE";
const DURATION: &'static str = "DURATION";
const PLANNED_DURATION: &'static str = "PLANNED-DURATION";
const SCTE35_CMD: &'static str = "SCTE35-CMD";
const SCTE35_OUT: &'static str = "SCTE35-OUT";
const SCTE35_IN: &'static str = "SCTE35-IN";
const END_ON_NEXT: &'static str = "END-ON-NEXT";
const YES: &'static str = "YES";
