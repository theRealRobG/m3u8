use crate::{
    date::{self, DateTime},
    error::{UnrecognizedEnumerationError, ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{EnumeratedString, EnumeratedStringList, TagName, into_inner_tag},
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
    utils::AsStaticCow,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Display,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cue {
    /// Indicates that an action is to be triggered before playback of the primary asset begins,
    /// regardless of where playback begins in the primary asset.
    Pre,
    /// Indicates that an action is to be triggered after the primary asset has been played to its
    /// end without error.
    Post,
    /// Indicates that an action is to be triggered once. It SHOULD NOT be triggered again, even if
    /// the user replays the portion of the primary asset that includes the trigger point.
    Once,
}
impl<'a> TryFrom<&'a str> for Cue {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            PRE => Ok(Self::Pre),
            POST => Ok(Self::Post),
            ONCE => Ok(Self::Once),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for Cue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for Cue {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Cue::Pre => Cow::Borrowed(PRE),
            Cue::Post => Cow::Borrowed(POST),
            Cue::Once => Cow::Borrowed(ONCE),
        }
    }
}
impl From<Cue> for Cow<'_, str> {
    fn from(value: Cue) -> Self {
        value.as_cow()
    }
}
impl From<Cue> for EnumeratedString<'_, Cue> {
    fn from(value: Cue) -> Self {
        Self::Known(value)
    }
}
const PRE: &str = "PRE";
const POST: &str = "POST";
const ONCE: &str = "ONCE";

#[derive(Debug, PartialEq, Clone)]
pub struct DaterangeAttributeList<'a> {
    pub id: Cow<'a, str>,
    pub start_date: DateTime,
    pub class: Option<Cow<'a, str>>,
    pub cue: Option<Cow<'a, str>>,
    pub end_date: Option<DateTime>,
    pub duration: Option<f64>,
    pub planned_duration: Option<f64>,
    pub extension_attributes: HashMap<Cow<'a, str>, ExtensionAttributeValue<'a>>,
    pub end_on_next: bool,
    pub scte35_cmd: Option<Cow<'a, str>>,
    pub scte35_out: Option<Cow<'a, str>>,
    pub scte35_in: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DaterangeBuilder<'a> {
    id: Cow<'a, str>,
    start_date: DateTime,
    class: Option<Cow<'a, str>>,
    cue: Option<Cow<'a, str>>,
    end_date: Option<DateTime>,
    duration: Option<f64>,
    planned_duration: Option<f64>,
    extension_attributes: HashMap<Cow<'a, str>, ExtensionAttributeValue<'a>>,
    end_on_next: bool,
    scte35_cmd: Option<Cow<'a, str>>,
    scte35_out: Option<Cow<'a, str>>,
    scte35_in: Option<Cow<'a, str>>,
}
impl<'a> DaterangeBuilder<'a> {
    pub fn new(id: impl Into<Cow<'a, str>>, start_date: DateTime) -> Self {
        Self {
            id: id.into(),
            start_date,
            class: Default::default(),
            cue: Default::default(),
            end_date: Default::default(),
            duration: Default::default(),
            planned_duration: Default::default(),
            extension_attributes: Default::default(),
            end_on_next: Default::default(),
            scte35_cmd: Default::default(),
            scte35_out: Default::default(),
            scte35_in: Default::default(),
        }
    }

    pub fn finish(self) -> Daterange<'a> {
        Daterange::new(DaterangeAttributeList {
            id: self.id,
            start_date: self.start_date,
            class: self.class,
            cue: self.cue,
            end_date: self.end_date,
            duration: self.duration,
            planned_duration: self.planned_duration,
            extension_attributes: self.extension_attributes,
            end_on_next: self.end_on_next,
            scte35_cmd: self.scte35_cmd,
            scte35_out: self.scte35_out,
            scte35_in: self.scte35_in,
        })
    }

    pub fn with_class(mut self, class: impl Into<Cow<'a, str>>) -> Self {
        self.class = Some(class.into());
        self
    }

    pub fn with_cue(mut self, cue: impl Into<Cow<'a, str>>) -> Self {
        self.cue = Some(cue.into());
        self
    }

    pub fn with_end_date(mut self, end_date: DateTime) -> Self {
        self.end_date = Some(end_date);
        self
    }

    pub fn with_duration(mut self, duration: f64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_planned_duration(mut self, planned_duration: f64) -> Self {
        self.planned_duration = Some(planned_duration);
        self
    }

    pub fn with_extension_attribute(
        mut self,
        extension_attribute_name: impl Into<Cow<'a, str>>,
        extension_attribute_value: ExtensionAttributeValue<'a>,
    ) -> Self {
        self.extension_attributes
            .insert(extension_attribute_name.into(), extension_attribute_value);
        self
    }

    pub fn with_extension_attributes(
        mut self,
        extension_attributes: HashMap<Cow<'a, str>, ExtensionAttributeValue<'a>>,
    ) -> Self {
        self.extension_attributes = extension_attributes;
        self
    }

    pub fn with_end_on_next(mut self) -> Self {
        self.end_on_next = true;
        self
    }

    pub fn with_scte35_cmd(mut self, scte35_cmd: impl Into<Cow<'a, str>>) -> Self {
        self.scte35_cmd = Some(scte35_cmd.into());
        self
    }

    pub fn with_scte35_out(mut self, scte35_out: impl Into<Cow<'a, str>>) -> Self {
        self.scte35_out = Some(scte35_out.into());
        self
    }

    pub fn with_scte35_in(mut self, scte35_in: impl Into<Cow<'a, str>>) -> Self {
        self.scte35_in = Some(scte35_in.into());
        self
    }
}

/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.5.1>
#[derive(Debug, Clone)]
pub struct Daterange<'a> {
    id: Cow<'a, str>,
    start_date: DateTime,
    class: Option<Cow<'a, str>>,
    cue: Option<Cow<'a, str>>,
    end_date: Option<DateTime>,
    duration: Option<f64>,
    planned_duration: Option<f64>,
    extension_attributes: HashMap<Cow<'a, str>, ExtensionAttributeValue<'a>>,
    end_on_next: Option<bool>,
    scte35_cmd: Option<Cow<'a, str>>,
    scte35_out: Option<Cow<'a, str>>,
    scte35_in: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
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
    pub fn new(attribute_list: DaterangeAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let DaterangeAttributeList {
            id,
            start_date,
            class,
            cue,
            end_date,
            duration,
            planned_duration,
            extension_attributes,
            end_on_next,
            scte35_cmd,
            scte35_out,
            scte35_in,
        } = attribute_list;
        Self {
            id,
            start_date,
            class,
            cue,
            end_date,
            duration,
            planned_duration,
            extension_attributes,
            end_on_next: Some(end_on_next),
            scte35_cmd,
            scte35_out,
            scte35_in,
            attribute_list: HashMap::new(),
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub fn builder(id: impl Into<Cow<'a, str>>, start_date: DateTime) -> DaterangeBuilder<'a> {
        DaterangeBuilder::new(id, start_date)
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

    pub fn cue(&self) -> Option<EnumeratedStringList<Cue>> {
        if let Some(cue) = &self.cue {
            Some(EnumeratedStringList::from(cue.as_ref()))
        } else {
            match self.attribute_list.get(CUE) {
                Some(ParsedAttributeValue::QuotedString(cue)) => {
                    Some(EnumeratedStringList::from(*cue))
                }
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
        if let Some(a) = self.extension_attributes.get(name) {
            Some(ExtensionAttributeValue::from(a))
        } else if let Some(a) = self.attribute_list.get(name) {
            ExtensionAttributeValue::try_from(*a).ok()
        } else {
            None
        }
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

    pub fn set_id(&mut self, id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(ID);
        self.id = id.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_class(&mut self, class: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CLASS);
        self.class = Some(class.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_class(&mut self) {
        self.attribute_list.remove(CLASS);
        self.class = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_start_date(&mut self, start_date: DateTime) {
        self.attribute_list.remove(START_DATE);
        self.start_date = start_date;
        self.output_line_is_dirty = true;
    }

    pub fn set_cue(&mut self, cue: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CUE);
        self.cue = Some(cue.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_cue(&mut self) {
        self.attribute_list.remove(CUE);
        self.cue = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_end_date(&mut self, end_date: DateTime) {
        self.attribute_list.remove(END_DATE);
        self.end_date = Some(end_date);
        self.output_line_is_dirty = true;
    }

    pub fn unset_end_date(&mut self) {
        self.attribute_list.remove(END_DATE);
        self.end_date = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_duration(&mut self, duration: f64) {
        self.attribute_list.remove(DURATION);
        self.duration = Some(duration);
        self.output_line_is_dirty = true;
    }

    pub fn unset_duration(&mut self) {
        self.attribute_list.remove(DURATION);
        self.duration = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_planned_duration(&mut self, planned_duration: f64) {
        self.attribute_list.remove(PLANNED_DURATION);
        self.planned_duration = Some(planned_duration);
        self.output_line_is_dirty = true;
    }

    pub fn unset_planned_duration(&mut self) {
        self.attribute_list.remove(PLANNED_DURATION);
        self.planned_duration = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_extension_attribute(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: ExtensionAttributeValue<'a>,
    ) {
        let name = name.into();
        if !name.starts_with("X-") {
            return;
        }
        self.attribute_list.retain(|k, _| *k != name);
        self.extension_attributes.insert(name, value);
        self.output_line_is_dirty = true;
    }

    pub fn unset_extension_attribute(&mut self, name: impl Into<Cow<'a, str>>) {
        let name = name.into();
        if !name.starts_with("X-") {
            return;
        }
        self.attribute_list.retain(|k, _| *k != name);
        self.extension_attributes.remove(&name);
        self.output_line_is_dirty = true;
    }

    pub fn set_end_on_next(&mut self, end_on_next: bool) {
        self.attribute_list.remove(END_ON_NEXT);
        self.end_on_next = Some(end_on_next);
        self.output_line_is_dirty = true;
    }

    pub fn set_scte35_cmd(&mut self, scte35_cmd: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(SCTE35_CMD);
        self.scte35_cmd = Some(scte35_cmd.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_scte35_cmd(&mut self) {
        self.attribute_list.remove(SCTE35_CMD);
        self.scte35_cmd = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_scte35_out(&mut self, scte35_out: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(SCTE35_OUT);
        self.scte35_out = Some(scte35_out.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_scte35_out(&mut self) {
        self.attribute_list.remove(SCTE35_OUT);
        self.scte35_out = None;
        self.output_line_is_dirty = true;
    }

    pub fn set_scte35_in(&mut self, scte35_in: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(SCTE35_IN);
        self.scte35_in = Some(scte35_in.into());
        self.output_line_is_dirty = true;
    }

    pub fn unset_scte35_in(&mut self) {
        self.attribute_list.remove(SCTE35_IN);
        self.scte35_in = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&DaterangeAttributeList {
            id: self.id().into(),
            class: self.class().map(|x| x.into()),
            start_date: self.start_date(),
            cue: self.cue().map(|x| x.into()),
            end_date: self.end_date(),
            duration: self.duration(),
            planned_duration: self.planned_duration(),
            extension_attributes: {
                let mut map = HashMap::new();
                for (key, value) in self.extension_attributes() {
                    map.insert(Cow::Borrowed(key), value);
                }
                map
            },
            scte35_cmd: self.scte35_cmd().map(|x| x.into()),
            scte35_out: self.scte35_out().map(|x| x.into()),
            scte35_in: self.scte35_in().map(|x| x.into()),
            end_on_next: self.end_on_next(),
        }));
        self.output_line_is_dirty = false;
    }
}

into_inner_tag!(Daterange);

#[derive(Debug, PartialEq, Clone)]
pub enum ExtensionAttributeValue<'a> {
    QuotedString(Cow<'a, str>),
    HexadecimalSequence(Cow<'a, str>),
    SignedDecimalFloatingPoint(f64),
}
impl<'a> ExtensionAttributeValue<'a> {
    pub fn quoted_string(quoted_string: impl Into<Cow<'a, str>>) -> Self {
        Self::QuotedString(quoted_string.into())
    }

    pub fn hexadecimal_sequence(hexadecimal_sequence: impl Into<Cow<'a, str>>) -> Self {
        Self::HexadecimalSequence(hexadecimal_sequence.into())
    }

    pub fn signed_decimal_floating_point(signed_decimal_floating_point: f64) -> Self {
        Self::SignedDecimalFloatingPoint(signed_decimal_floating_point)
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
            ParsedAttributeValue::QuotedString(s) => Ok(Self::QuotedString(Cow::Borrowed(s))),
            ParsedAttributeValue::UnquotedString(s) if is_hexadecimal_sequence(s) => {
                Ok(Self::HexadecimalSequence(Cow::Borrowed(s)))
            }
            ParsedAttributeValue::UnquotedString(_) => Err("Invalid extension attribute value"),
        }
    }
}

impl<'a> From<&'a ExtensionAttributeValue<'a>> for ExtensionAttributeValue<'a> {
    fn from(value: &'a ExtensionAttributeValue<'a>) -> Self {
        match value {
            Self::QuotedString(cow) => Self::QuotedString(Cow::Borrowed(cow)),
            Self::HexadecimalSequence(cow) => Self::HexadecimalSequence(Cow::Borrowed(cow)),
            Self::SignedDecimalFloatingPoint(d) => Self::SignedDecimalFloatingPoint(*d),
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

fn calculate_line(attribute_list: &DaterangeAttributeList) -> Vec<u8> {
    let DaterangeAttributeList {
        id,
        start_date,
        class,
        cue,
        end_date,
        duration,
        planned_duration,
        extension_attributes,
        end_on_next,
        scte35_cmd,
        scte35_out,
        scte35_in,
    } = attribute_list;
    let mut line = format!(
        "#EXT{}:{}=\"{}\",{}=\"{}\"",
        TagName::Daterange.as_str(),
        ID,
        id,
        START_DATE,
        start_date,
    );
    if let Some(class) = class {
        line.push_str(format!(",{CLASS}=\"{class}\"").as_str());
    }
    if let Some(cue) = cue {
        line.push_str(format!(",{CUE}=\"{cue}\"").as_str());
    }
    if let Some(end_date) = end_date {
        line.push_str(format!(",{END_DATE}=\"{end_date}\"").as_str());
    }
    if let Some(duration) = duration {
        line.push_str(format!(",{DURATION}={duration}").as_str());
    }
    if let Some(planned_duration) = planned_duration {
        line.push_str(format!(",{PLANNED_DURATION}={planned_duration}").as_str());
    }
    for (key, value) in extension_attributes {
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
                line.push_str(format!("{d}").as_str());
            }
        };
    }
    if let Some(scte35_cmd) = scte35_cmd {
        line.push_str(format!(",{SCTE35_CMD}={scte35_cmd}").as_str());
    }
    if let Some(scte35_out) = scte35_out {
        line.push_str(format!(",{SCTE35_OUT}={scte35_out}").as_str());
    }
    if let Some(scte35_in) = scte35_in {
        line.push_str(format!(",{SCTE35_IN}={scte35_in}").as_str());
    }
    if *end_on_next {
        line.push_str(",END-ON-NEXT=YES");
    }
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        date_time,
        tag::{hls::test_macro::mutation_tests, known::IntoInnerTag},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn new_with_no_optionals_should_be_valid() {
        let tag =
            Daterange::builder("some-id", date_time!(2025-06-14 T 23:41:42.000 -05:00)).finish();
        assert_eq!(
            b"#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"2025-06-14T23:41:42.000-05:00\"",
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_optionals_should_be_valid() {
        let tag = Daterange::builder("some-id", date_time!(2025-06-14 T 23:41:42.000 -05:00))
            .with_class("com.example.class")
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .with_end_date(date_time!(2025-06-14 T 23:43:42.000 -05:00))
            .with_duration(120.0)
            .with_planned_duration(180.0)
            .with_scte35_cmd("0xABCD")
            .with_scte35_out("0xABCD")
            .with_scte35_in("0xABCD")
            .with_end_on_next()
            .finish();
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"2025-06-14T23:41:42.000-05:00\",",
                "CLASS=\"com.example.class\",CUE=\"ONCE\",",
                "END-DATE=\"2025-06-14T23:43:42.000-05:00\",DURATION=120,PLANNED-DURATION=180,",
                "SCTE35-CMD=0xABCD,SCTE35-OUT=0xABCD,SCTE35-IN=0xABCD,END-ON-NEXT=YES"
            )
            .as_bytes(),
            tag.into_inner().value()
        );
    }

    #[test]
    fn new_with_optionals_and_some_client_attributes_should_be_valid() {
        let tag = Daterange::builder("some-id", date_time!(2025-06-14 T 23:41:42.000 -05:00))
            .with_extension_attribute(
                "X-COM-EXAMPLE-A",
                ExtensionAttributeValue::QuotedString("Example A".into()),
            )
            .with_extension_attribute(
                "X-COM-EXAMPLE-B",
                ExtensionAttributeValue::SignedDecimalFloatingPoint(42.0),
            )
            .with_extension_attribute(
                "X-COM-EXAMPLE-C",
                ExtensionAttributeValue::HexadecimalSequence("0xABCD".into()),
            )
            .finish();
        // Client attributes can come in any order (due to it being a HashMap) so we need to be a
        // little more creative in validating the tag format.
        let tag_inner = tag.into_inner();
        let tag_as_bytes = tag_inner.value();
        let mut found_a = false;
        let mut found_b = false;
        let mut found_c = false;
        for (index, split) in tag_as_bytes.split(|b| b == &b',').enumerate() {
            match index {
                0 => assert_eq!(b"#EXT-X-DATERANGE:ID=\"some-id\"", split),
                1 => assert_eq!(b"START-DATE=\"2025-06-14T23:41:42.000-05:00\"", split),
                2 | 3 | 4 => {
                    if split.starts_with(b"X-COM-EXAMPLE-A") {
                        if found_a {
                            panic!("Already found A")
                        }
                        found_a = true;
                        assert_eq!(b"X-COM-EXAMPLE-A=\"Example A\"", split);
                    } else if split.starts_with(b"X-COM-EXAMPLE-B") {
                        if found_b {
                            panic!("Already found B")
                        }
                        found_b = true;
                        assert_eq!(b"X-COM-EXAMPLE-B=42", split);
                    } else if split.starts_with(b"X-COM-EXAMPLE-C") {
                        if found_c {
                            panic!("Already found C")
                        }
                        found_c = true;
                        assert_eq!(b"X-COM-EXAMPLE-C=0xABCD", split);
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
        let mut daterange = Daterange::builder("some-id", DateTime::default())
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .with_extension_attribute(
                "X-TO-REMOVE",
                ExtensionAttributeValue::QuotedString("remove me".into()),
            )
            .finish();
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"some-id\",START-DATE=\"1970-01-01T00:00:00.000Z\",",
                "CUE=\"ONCE\",X-TO-REMOVE=\"remove me\"",
            )
            .as_bytes(),
            daterange.clone().into_inner().value()
        );
        daterange.set_id("another-id");
        daterange.set_class("com.example.test");
        daterange.unset_cue();
        daterange.set_extension_attribute(
            "X-EXAMPLE",
            ExtensionAttributeValue::QuotedString("TEST".into()),
        );
        daterange.unset_extension_attribute("X-TO-REMOVE");
        assert_eq!(
            concat!(
                "#EXT-X-DATERANGE:ID=\"another-id\",START-DATE=\"1970-01-01T00:00:00.000Z\",",
                "CLASS=\"com.example.test\",X-EXAMPLE=\"TEST\"",
            )
            .as_bytes(),
            daterange.into_inner().value()
        );
    }

    #[test]
    fn mutating_cue_works_as_expected() {
        let mut daterange = Daterange::builder("some-id", DateTime::default())
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .finish();
        let mut cue = daterange.cue().unwrap();
        cue.insert(Cue::Pre);
        daterange.set_cue(cue.to_owned());
        assert_eq!(
            EnumeratedStringList::from([Cue::Once, Cue::Pre]),
            daterange.cue().unwrap()
        );
    }

    mutation_tests!(
        Daterange::builder("some-id", date_time!(2025-06-14 T 23:41:42.000 -05:00))
            .with_class("com.example.class")
            .with_cue(EnumeratedStringList::from([Cue::Once]))
            .with_end_date(date_time!(2025-06-14 T 23:43:42.000 -05:00))
            .with_duration(120.0)
            .with_planned_duration(180.0)
            .with_scte35_cmd("0xABCD")
            .with_scte35_out("0xABCD")
            .with_scte35_in("0xABCD")
            .finish(),
        (id, "another-id", @Attr="ID=\"another-id\""),
        (start_date, DateTime::default(), @Attr="START-DATE=\"1970-01-01T00:00:00.000Z\""),
        (class, @Option "com.test.class", @Attr="CLASS=\"com.test.class\""),
        (cue, @Option EnumeratedStringList::from([Cue::Once, Cue::Pre]), @Attr="CUE=\"ONCE,PRE\""),
        (end_date, @Option DateTime::default(), @Attr="END-DATE=\"1970-01-01T00:00:00.000Z\""),
        (duration, @Option 60.0, @Attr="DURATION=60"),
        (planned_duration, @Option 80.0, @Attr="PLANNED-DURATION=80"),
        (scte35_cmd, @Option "0x1234", @Attr="SCTE35-CMD=0x1234"),
        (scte35_out, @Option "0x1234", @Attr="SCTE35-OUT=0x1234"),
        (scte35_in, @Option "0x1234", @Attr="SCTE35-IN=0x1234"),
        (end_on_next, true, @Attr="END-ON-NEXT=YES")
    );
}
