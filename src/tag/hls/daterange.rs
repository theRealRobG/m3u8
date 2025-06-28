use crate::{
    date::{self, DateTime},
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{TagInner, TagName},
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.1
#[derive(Debug, Clone)]
pub struct Daterange<'a> {
    id: Cow<'a, str>,
    start_date: DateTime,
    class: Option<Cow<'a, str>>,
    cue: Option<Cow<'a, str>>,
    end_date: Option<DateTime>,
    duration: Option<f64>,
    planned_duration: Option<f64>,
    extension_attributes: HashMap<Cow<'a, str>, InternalExtensionAttributeValue<'a>>,
    end_on_next: Option<bool>,
    scte35_cmd: Option<Cow<'a, str>>,
    scte35_out: Option<Cow<'a, str>>,
    scte35_in: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for Daterange<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
            && self.start_date() == other.start_date()
            && self.class() == other.class()
            && self.cue() == other.cue()
            && self.end_date() == other.end_date()
            && self.duration() == other.duration()
            && self.planned_duration() == other.planned_duration()
            && self.extension_attributes() == other.extension_attributes()
            && self.end_on_next() == other.end_on_next()
            && self.scte35_cmd() == other.scte35_cmd()
            && self.scte35_out() == other.scte35_out()
            && self.scte35_in() == other.scte35_in()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Daterange<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::QuotedString(id)) = attribute_list.get(ID) else {
            return Err(ValidationError::MissingRequiredAttribute(ID));
        };
        let Some(start_date) = (match attribute_list.get(START_DATE) {
            Some(ParsedAttributeValue::QuotedString(date_str)) => date::parse(date_str).ok(),
            _ => None,
        }) else {
            return Err(ValidationError::MissingRequiredAttribute(START_DATE));
        };
        Ok(Self {
            id: Cow::Borrowed(id),
            start_date,
            class: None,
            cue: None,
            end_date: None,
            duration: None,
            planned_duration: None,
            extension_attributes: HashMap::new(),
            end_on_next: None,
            scte35_cmd: None,
            scte35_out: None,
            scte35_in: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Daterange<'a> {
    pub fn new(
        id: String,
        class: Option<String>,
        start_date: DateTime,
        cue: Option<String>,
        end_date: Option<DateTime>,
        duration: Option<f64>,
        planned_duration: Option<f64>,
        mut extension_attributes: HashMap<String, OwnedExtensionAttributeValue>,
        scte35_cmd: Option<String>,
        scte35_out: Option<String>,
        scte35_in: Option<String>,
        end_on_next: bool,
    ) -> Self {
        let id = Cow::Owned(id);
        let class = class.map(Cow::Owned);
        let cue = cue.map(Cow::Owned);
        let scte35_cmd = scte35_cmd.map(Cow::Owned);
        let scte35_out = scte35_out.map(Cow::Owned);
        let scte35_in = scte35_in.map(Cow::Owned);
        let output_line = Cow::Owned(calculate_line(
            &id,
            &class,
            start_date,
            &cue,
            end_date,
            duration,
            planned_duration,
            &extension_attributes_from_owned(&extension_attributes),
            &scte35_cmd,
            &scte35_out,
            &scte35_in,
            end_on_next,
        ));
        Self {
            id,
            start_date,
            class,
            cue,
            end_date,
            duration,
            planned_duration,
            extension_attributes: extension_attributes
                .drain()
                .filter_map(|(key, value)| {
                    if !key.starts_with("X-") {
                        return None;
                    }
                    InternalExtensionAttributeValue::try_from(value)
                        .ok()
                        .map(|value| (Cow::Owned(key), value))
                })
                .collect(),
            end_on_next: Some(end_on_next),
            scte35_cmd,
            scte35_out,
            scte35_in,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    // === GETTERS ===

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn class(&self) -> Option<&str> {
        if let Some(class) = &self.class {
            Some(class)
        } else {
            match self.attribute_list.get(CLASS) {
                Some(ParsedAttributeValue::QuotedString(class)) => Some(class),
                _ => None,
            }
        }
    }

    pub fn start_date(&self) -> DateTime {
        self.start_date
    }

    pub fn cue(&self) -> Option<&str> {
        if let Some(cue) = &self.cue {
            Some(cue)
        } else {
            match self.attribute_list.get(CUE) {
                Some(ParsedAttributeValue::QuotedString(cue)) => Some(cue),
                _ => None,
            }
        }
    }

    pub fn end_date(&self) -> Option<DateTime> {
        if let Some(end_date) = self.end_date {
            Some(end_date)
        } else {
            match self.attribute_list.get(END_DATE) {
                Some(ParsedAttributeValue::QuotedString(date_str)) => date::parse(date_str).ok(),
                _ => None,
            }
        }
    }

    pub fn duration(&self) -> Option<f64> {
        if let Some(duration) = self.duration {
            Some(duration)
        } else {
            match self.attribute_list.get(DURATION) {
                Some(d) => d.as_option_f64(),
                _ => None,
            }
        }
    }

    pub fn planned_duration(&self) -> Option<f64> {
        if let Some(planned_duration) = self.planned_duration {
            Some(planned_duration)
        } else {
            match self.attribute_list.get(PLANNED_DURATION) {
                Some(d) => d.as_option_f64(),
                _ => None,
            }
        }
    }

    pub fn extension_attributes(&self) -> HashMap<&str, ExtensionAttributeValue> {
        let mut map = HashMap::new();
        for (key, value) in &self.attribute_list {
            if key.starts_with("X-") {
                if let Ok(value) = ExtensionAttributeValue::try_from(*value) {
                    map.insert(*key, value);
                }
            }
        }
        for (key, value) in &self.extension_attributes {
            map.insert(key as &str, ExtensionAttributeValue::from(value));
        }
        map
    }

    pub fn extension_attribute<'b>(&'a self, name: &'b str) -> Option<ExtensionAttributeValue<'a>> {
        self.extension_attributes().get(name).copied()
    }

    pub fn extension_attribute_keys(&self) -> HashSet<&str> {
        let mut set = HashSet::new();
        for key in self.extension_attributes().keys() {
            set.insert(*key);
        }
        set
    }

    pub fn end_on_next(&self) -> bool {
        if let Some(end_on_next) = self.end_on_next {
            end_on_next
        } else {
            matches!(
                self.attribute_list.get(END_ON_NEXT),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }

    // The specification indicates that the SCTE35-(CMD|OUT|IN) attributes are
    // represented as hexadecimal sequences. This implies that they should be parsed as
    // UnquotedString (given that section "4.2. Attribute Lists" indicates that a
    // "hexadecimal-sequence [is] an unquoted string of characters"); however, in
    // practice, I've found that some packagers have put this information in quoted
    // strings (containing the hexadecimal sequence), so I'll allow this parser to be
    // lenient on that requirement and accept both.

    pub fn scte35_cmd(&self) -> Option<&str> {
        if let Some(scte35_cmd) = &self.scte35_cmd {
            Some(scte35_cmd)
        } else {
            match self.attribute_list.get(SCTE35_CMD) {
                Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn scte35_out(&self) -> Option<&str> {
        if let Some(scte35_out) = &self.scte35_out {
            Some(scte35_out)
        } else {
            match self.attribute_list.get(SCTE35_OUT) {
                Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    pub fn scte35_in(&self) -> Option<&str> {
        if let Some(scte35_in) = &self.scte35_in {
            Some(scte35_in)
        } else {
            match self.attribute_list.get(SCTE35_IN) {
                Some(ParsedAttributeValue::UnquotedString(s)) => Some(s),
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }

    // === SETTERS ===

    pub fn set_id(&mut self, id: String) {
        self.attribute_list.remove(ID);
        self.id = Cow::Owned(id);
        self.output_line_is_dirty = true;
    }

    pub fn set_class(&mut self, class: Option<String>) {
        self.attribute_list.remove(CLASS);
        self.class = class.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_start_date(&mut self, start_date: DateTime) {
        self.attribute_list.remove(START_DATE);
        self.start_date = start_date;
        self.output_line_is_dirty = true;
    }

    pub fn set_cue(&mut self, cue: Option<String>) {
        self.attribute_list.remove(CUE);
        self.cue = cue.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_end_date(&mut self, end_date: Option<DateTime>) {
        self.attribute_list.remove(END_DATE);
        self.end_date = end_date;
        self.output_line_is_dirty = true;
    }

    pub fn set_duration(&mut self, duration: Option<f64>) {
        self.attribute_list.remove(DURATION);
        self.duration = duration;
        self.output_line_is_dirty = true;
    }

    pub fn set_planned_duration(&mut self, planned_duration: Option<f64>) {
        self.attribute_list.remove(PLANNED_DURATION);
        self.planned_duration = planned_duration;
        self.output_line_is_dirty = true;
    }

    pub fn set_extension_attribute(
        &mut self,
        name: String,
        value: Option<OwnedExtensionAttributeValue>,
    ) {
        if !name.starts_with("X-") {
            return;
        }
        if let Some(value) = value {
            if let Ok(value) = InternalExtensionAttributeValue::try_from(value) {
                self.attribute_list.remove(name.as_str());
                self.extension_attributes.insert(Cow::Owned(name), value);
                self.output_line_is_dirty = true;
            }
        } else {
            self.attribute_list.remove(name.as_str());
            self.extension_attributes.remove(name.as_str());
            self.output_line_is_dirty = true;
        }
    }

    pub fn set_end_on_next(&mut self, end_on_next: bool) {
        self.attribute_list.remove(END_ON_NEXT);
        self.end_on_next = Some(end_on_next);
        self.output_line_is_dirty = true;
    }

    pub fn set_scte35_cmd(&mut self, scte35_cmd: Option<String>) {
        self.attribute_list.remove(SCTE35_CMD);
        self.scte35_cmd = scte35_cmd.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_scte35_out(&mut self, scte35_out: Option<String>) {
        self.attribute_list.remove(SCTE35_OUT);
        self.scte35_out = scte35_out.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    pub fn set_scte35_in(&mut self, scte35_in: Option<String>) {
        self.attribute_list.remove(SCTE35_IN);
        self.scte35_in = scte35_in.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.id(),
            &self.class().map(|x| x.into()),
            self.start_date(),
            &self.cue().map(|x| x.into()),
            self.end_date(),
            self.duration(),
            self.planned_duration(),
            &self.extension_attributes(),
            &self.scte35_cmd().map(|x| x.into()),
            &self.scte35_out().map(|x| x.into()),
            &self.scte35_in().map(|x| x.into()),
            self.end_on_next(),
        ));
        self.output_line_is_dirty = false;
    }
}

#[derive(Debug, PartialEq, Clone)]
enum InternalExtensionAttributeValue<'a> {
    QuotedString(Cow<'a, str>),
    HexadecimalSequence(Cow<'a, str>),
    SignedDecimalFloatingPoint(f64),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExtensionAttributeValue<'a> {
    QuotedString(&'a str),
    HexadecimalSequence(&'a str),
    SignedDecimalFloatingPoint(f64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OwnedExtensionAttributeValue {
    QuotedString(String),
    HexadecimalSequence(String),
    SignedDecimalFloatingPoint(f64),
}

impl<'a> TryFrom<ParsedAttributeValue<'a>> for InternalExtensionAttributeValue<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedAttributeValue<'a>) -> Result<Self, Self::Error> {
        match value {
            ParsedAttributeValue::DecimalInteger(n) => {
                Ok(Self::SignedDecimalFloatingPoint(n as f64))
            }
            ParsedAttributeValue::SignedDecimalFloatingPoint(n) => {
                Ok(Self::SignedDecimalFloatingPoint(n))
            }
            ParsedAttributeValue::QuotedString(s) => Ok(Self::QuotedString(Cow::Borrowed(s))),
            ParsedAttributeValue::UnquotedString(s) if is_hexadecimal_sequence(s) => {
                Ok(Self::HexadecimalSequence(Cow::Borrowed(s)))
            }
            ParsedAttributeValue::UnquotedString(_) => Err("Invalid extension attribute value"),
        }
    }
}

impl<'a> TryFrom<ParsedAttributeValue<'a>> for ExtensionAttributeValue<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedAttributeValue<'a>) -> Result<Self, Self::Error> {
        match value {
            ParsedAttributeValue::DecimalInteger(n) => {
                Ok(Self::SignedDecimalFloatingPoint(n as f64))
            }
            ParsedAttributeValue::SignedDecimalFloatingPoint(n) => {
                Ok(Self::SignedDecimalFloatingPoint(n))
            }
            ParsedAttributeValue::QuotedString(s) => Ok(Self::QuotedString(s)),
            ParsedAttributeValue::UnquotedString(s) if is_hexadecimal_sequence(s) => {
                Ok(Self::HexadecimalSequence(s))
            }
            ParsedAttributeValue::UnquotedString(_) => Err("Invalid extension attribute value"),
        }
    }
}

impl<'a> From<&'a InternalExtensionAttributeValue<'a>> for ExtensionAttributeValue<'a> {
    fn from(value: &'a InternalExtensionAttributeValue) -> Self {
        match value {
            InternalExtensionAttributeValue::QuotedString(cow) => Self::QuotedString(cow),
            InternalExtensionAttributeValue::HexadecimalSequence(cow) => {
                Self::HexadecimalSequence(cow)
            }
            InternalExtensionAttributeValue::SignedDecimalFloatingPoint(n) => {
                Self::SignedDecimalFloatingPoint(*n)
            }
        }
    }
}

impl<'a> TryFrom<OwnedExtensionAttributeValue> for InternalExtensionAttributeValue<'a> {
    type Error = &'static str;

    fn try_from(value: OwnedExtensionAttributeValue) -> Result<Self, Self::Error> {
        match value {
            OwnedExtensionAttributeValue::QuotedString(s) => Ok(Self::QuotedString(Cow::Owned(s))),
            OwnedExtensionAttributeValue::SignedDecimalFloatingPoint(n) => {
                Ok(Self::SignedDecimalFloatingPoint(n))
            }
            OwnedExtensionAttributeValue::HexadecimalSequence(s) => {
                if is_hexadecimal_sequence(&s) {
                    Ok(Self::HexadecimalSequence(Cow::Owned(s)))
                } else {
                    Err("Invalid extension attribute value")
                }
            }
        }
    }
}

fn is_hexadecimal_sequence(s: &str) -> bool {
    let mut bytes = s.bytes();
    if bytes.next() != Some(b'0') {
        return false;
    }
    let x = bytes.next();
    if x != Some(b'x') && x != Some(b'X') {
        return false;
    }
    if !bytes.all(|b| b.is_ascii_hexdigit()) {
        return false;
    }
    true
}

const ID: &str = "ID";
const CLASS: &str = "CLASS";
const START_DATE: &str = "START-DATE";
const CUE: &str = "CUE";
const END_DATE: &str = "END-DATE";
const DURATION: &str = "DURATION";
const PLANNED_DURATION: &str = "PLANNED-DURATION";
const SCTE35_CMD: &str = "SCTE35-CMD";
const SCTE35_OUT: &str = "SCTE35-OUT";
const SCTE35_IN: &str = "SCTE35-IN";
const END_ON_NEXT: &str = "END-ON-NEXT";
const YES: &str = "YES";

fn calculate_line<'a>(
    id: &str,
    class: &Option<Cow<'a, str>>,
    start_date: DateTime,
    cue: &Option<Cow<'a, str>>,
    end_date: Option<DateTime>,
    duration: Option<f64>,
    planned_duration: Option<f64>,
    client_attributes: &HashMap<&str, ExtensionAttributeValue<'a>>,
    scte35_cmd: &Option<Cow<'a, str>>,
    scte35_out: &Option<Cow<'a, str>>,
    scte35_in: &Option<Cow<'a, str>>,
    end_on_next: bool,
) -> String {
    let mut line = format!(
        "#EXT{}:{}=\"{}\",{}=\"{}\"",
        TagName::Daterange.as_str(),
        ID,
        id,
        START_DATE,
        start_date,
    );
    if let Some(class) = class {
        line.push_str(format!(",{}=\"{}\"", CLASS, class).as_str());
    }
    if let Some(cue) = cue {
        line.push_str(format!(",{}=\"{}\"", CUE, cue).as_str());
    }
    if let Some(end_date) = end_date {
        line.push_str(format!(",{}=\"{}\"", END_DATE, end_date).as_str());
    }
    if let Some(duration) = duration {
        line.push_str(format!(",{}={}", DURATION, duration).as_str());
    }
    if let Some(planned_duration) = planned_duration {
        line.push_str(format!(",{}={}", PLANNED_DURATION, planned_duration).as_str());
    }
    for (key, value) in client_attributes {
        line.push(',');
        line.push_str(key);
        line.push('=');
        match value {
            ExtensionAttributeValue::HexadecimalSequence(s) => {
                line.push_str(s);
            }
            ExtensionAttributeValue::QuotedString(s) => {
                line.push('"');
                line.push_str(s);
                line.push('"');
            }
            ExtensionAttributeValue::SignedDecimalFloatingPoint(d) => {
                line.push_str(format!("{}", d).as_str());
            }
        };
    }
    if let Some(scte35_cmd) = scte35_cmd {
        line.push_str(format!(",{}={}", SCTE35_CMD, scte35_cmd).as_str());
    }
    if let Some(scte35_out) = scte35_out {
        line.push_str(format!(",{}={}", SCTE35_OUT, scte35_out).as_str());
    }
    if let Some(scte35_in) = scte35_in {
        line.push_str(format!(",{}={}", SCTE35_IN, scte35_in).as_str());
    }
    if end_on_next {
        line.push_str(",END-ON-NEXT=YES");
    }
    line
}

fn extension_attributes_from_owned(
    extension_attributes: &HashMap<String, OwnedExtensionAttributeValue>,
) -> HashMap<&str, ExtensionAttributeValue> {
    let mut intermediate_hash_map = HashMap::new();
    let user_set_extension_attrs: Vec<(&str, ExtensionAttributeValue)> = extension_attributes
        .iter()
        .filter_map(|(key, value)| {
            if !key.starts_with("X-") {
                return None;
            }
            match value {
                OwnedExtensionAttributeValue::QuotedString(s) => Some((
                    key.as_str(),
                    ExtensionAttributeValue::QuotedString(s.as_str()),
                )),
                OwnedExtensionAttributeValue::HexadecimalSequence(s) => {
                    if is_hexadecimal_sequence(s) {
                        Some((
                            key.as_str(),
                            ExtensionAttributeValue::HexadecimalSequence(s.as_str()),
                        ))
                    } else {
                        None
                    }
                }
                OwnedExtensionAttributeValue::SignedDecimalFloatingPoint(d) => Some((
                    key.as_str(),
                    ExtensionAttributeValue::SignedDecimalFloatingPoint(*d),
                )),
            }
        })
        .collect();
    for (key, value) in user_set_extension_attrs {
        intermediate_hash_map.insert(key, value);
    }
    intermediate_hash_map
}

#[cfg(test)]
mod tests {
    use crate::date::DateTimeTimezoneOffset;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_with_no_optionals_should_be_valid() {
        let tag = Daterange::new(
            "some-id".to_string(),
            None,
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 14,
                time_hour: 23,
                time_minute: 41,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -5,
                    time_minute: 0,
                },
            },
            None,
            None,
            None,
            None,
            HashMap::new(),
            None,
            None,
            None,
            false,
        );
        assert_eq!(
            "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"2025-06-14T23:41:42.000-05:00\"",
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_optionals_should_be_valid() {
        let tag = Daterange::new(
            "some-id".to_string(),
            Some("com.example.class".to_string()),
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 14,
                time_hour: 23,
                time_minute: 41,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -5,
                    time_minute: 0,
                },
            },
            Some("ONCE".to_string()),
            Some(DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 14,
                time_hour: 23,
                time_minute: 43,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -5,
                    time_minute: 0,
                },
            }),
            Some(120.0),
            Some(180.0),
            HashMap::new(),
            Some("0xABCD".to_string()),
            Some("0xABCD".to_string()),
            Some("0xABCD".to_string()),
            true,
        );
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"2025-06-14T23:41:42.000-05:00\",",
                "CLASS=\"com.example.class\",CUE=\"ONCE\",",
                "END-DATE=\"2025-06-14T23:43:42.000-05:00\",DURATION=120,PLANNED-DURATION=180,",
                "SCTE35-CMD=0xABCD,SCTE35-OUT=0xABCD,SCTE35-IN=0xABCD,END-ON-NEXT=YES"
            ),
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_optionals_and_some_client_attributes_should_be_valid() {
        let tag = Daterange::new(
            "some-id".to_string(),
            None as Option<String>,
            DateTime {
                date_fullyear: 2025,
                date_month: 6,
                date_mday: 14,
                time_hour: 23,
                time_minute: 41,
                time_second: 42.0,
                timezone_offset: DateTimeTimezoneOffset {
                    time_hour: -5,
                    time_minute: 0,
                },
            },
            None as Option<String>,
            None,
            None,
            None,
            HashMap::from([
                (
                    "X-COM-EXAMPLE-A".to_string(),
                    OwnedExtensionAttributeValue::QuotedString("Example A".to_string()),
                ),
                (
                    "X-COM-EXAMPLE-B".to_string(),
                    OwnedExtensionAttributeValue::SignedDecimalFloatingPoint(42.0),
                ),
                (
                    "X-COM-EXAMPLE-C".to_string(),
                    OwnedExtensionAttributeValue::HexadecimalSequence("0xABCD".to_string()),
                ),
            ]),
            None as Option<String>,
            None as Option<String>,
            None as Option<String>,
            false,
        );
        // Client attributes can come in any order (due to it being a HashMap) so we need to be a
        // little more creative in validating the tag format.
        let tag_inner = tag.into_inner();
        let tag_as_str = tag_inner.value();
        let mut found_a = false;
        let mut found_b = false;
        let mut found_c = false;
        for (index, split) in tag_as_str.split(',').enumerate() {
            match index {
                0 => assert_eq!("#EXT-X-DATERANGE:ID=\"some-id\"", split),
                1 => assert_eq!("START-DATE=\"2025-06-14T23:41:42.000-05:00\"", split),
                2 | 3 | 4 => {
                    if split.starts_with("X-COM-EXAMPLE-A") {
                        if found_a {
                            panic!("Already found A")
                        }
                        found_a = true;
                        assert_eq!("X-COM-EXAMPLE-A=\"Example A\"", split);
                    } else if split.starts_with("X-COM-EXAMPLE-B") {
                        if found_b {
                            panic!("Already found B")
                        }
                        found_b = true;
                        assert_eq!("X-COM-EXAMPLE-B=42", split);
                    } else if split.starts_with("X-COM-EXAMPLE-C") {
                        if found_c {
                            panic!("Already found C")
                        }
                        found_c = true;
                        assert_eq!("X-COM-EXAMPLE-C=0xABCD", split);
                    } else {
                        panic!("Unexpected attribute at index {}", index);
                    }
                }
                _ => panic!("Too many attributes"),
            }
        }
        assert!(found_a);
        assert!(found_b);
        assert!(found_c);
    }

    #[test]
    fn mutation_should_work() {
        let mut daterange = Daterange::new(
            "some-id".to_string(),
            None,
            DateTime::default(),
            Some("ONCE".to_string()),
            None,
            None,
            None,
            HashMap::new(),
            None,
            None,
            None,
            false,
        );
        assert_eq!(
            "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"1970-01-01T00:00:00.000Z\",CUE=\"ONCE\"",
            daterange.clone().into_inner().value()
        );
        daterange.set_id("another-id".to_string());
        daterange.set_class(Some("com.example.test".to_string()));
        daterange.set_cue(None);
        daterange.set_extension_attribute(
            "X-EXAMPLE".to_string(),
            Some(OwnedExtensionAttributeValue::QuotedString(
                "TEST".to_string(),
            )),
        );
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"another-id\",START-DATE=\"1970-01-01T00:00:00.000Z\",",
                "CLASS=\"com.example.test\",X-EXAMPLE=\"TEST\"",
            ),
            daterange.into_inner().value()
        );
    }
}
