use crate::{
    date::{self, DateTime},
    tag::{
        draft_pantos_hls::TagName,
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.1
#[derive(Debug)]
pub struct Daterange<'a> {
    id: &'a str,
    start_date: DateTime,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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

impl<'a> TryFrom<ParsedTag<'a>> for Daterange<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
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
            output_line: Cow::Borrowed(tag.original_input.as_bytes()),
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
        for (key, value) in client_attributes.clone() {
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
            output_line: Cow::Owned(
                calculate_line(
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
                )
                .into_bytes(),
            ),
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
        for &key in self.attribute_list.keys() {
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

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }
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

fn calculate_line(
    id: &str,
    class: Option<&str>,
    start_date: DateTime,
    cue: Option<&str>,
    end_date: Option<DateTime>,
    duration: Option<f64>,
    planned_duration: Option<f64>,
    client_attributes: HashMap<&str, ParsedAttributeValue>,
    scte35_cmd: Option<&str>,
    scte35_out: Option<&str>,
    scte35_in: Option<&str>,
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
        let value = match value {
            ParsedAttributeValue::DecimalInteger(n) => format!("{}", n),
            ParsedAttributeValue::SignedDecimalFloatingPoint(n) => format!("{:?}", n),
            ParsedAttributeValue::QuotedString(s) => format!("\"{}\"", s),
            ParsedAttributeValue::UnquotedString(s) => s.to_string(),
        };
        line.push_str(format!(",{}={}", key, value).as_str());
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

#[cfg(test)]
mod tests {
    use crate::date::DateTimeTimezoneOffset;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_with_no_optionals_should_be_valid() {
        let tag = Daterange::new(
            "some-id",
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
            tag.as_str()
        );
    }

    #[test]
    fn new_with_optionals_should_be_valid() {
        let tag = Daterange::new(
            "some-id",
            Some("com.example.class"),
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
            Some("ONCE"),
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
            Some("0xABCD"),
            Some("0xABCD"),
            Some("0xABCD"),
            true,
        );
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"2025-06-14T23:41:42.000-05:00\",",
                "CLASS=\"com.example.class\",CUE=\"ONCE\",",
                "END-DATE=\"2025-06-14T23:43:42.000-05:00\",DURATION=120,PLANNED-DURATION=180,",
                "SCTE35-CMD=0xABCD,SCTE35-OUT=0xABCD,SCTE35-IN=0xABCD,END-ON-NEXT=YES"
            ),
            tag.as_str()
        );
    }

    #[test]
    fn new_with_optionals_and_some_client_attributes_should_be_valid() {
        let tag = Daterange::new(
            "some-id",
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
            HashMap::from([
                (
                    "X-COM-EXAMPLE-A",
                    ParsedAttributeValue::QuotedString("Example A"),
                ),
                (
                    "X-COM-EXAMPLE-B",
                    ParsedAttributeValue::SignedDecimalFloatingPoint(42.0),
                ),
                (
                    "X-COM-EXAMPLE-C",
                    ParsedAttributeValue::UnquotedString("0xABCD"),
                ),
            ]),
            None,
            None,
            None,
            false,
        );
        // Client attributes can come in any order (due to it being a HashMap) so we need to be a
        // little more creative in validating the tag format.
        let tag_as_str = tag.as_str();
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
                        assert_eq!("X-COM-EXAMPLE-B=42.0", split);
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
}
