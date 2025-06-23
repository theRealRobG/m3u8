use crate::tag::{
    hls::TagInner,
    known::ParsedTag,
    value::{ParsedAttributeValue, ParsedTagValue},
};
use std::borrow::Cow;

#[derive(Debug)]
pub struct Name<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Name<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.value() == other.value()
    }
}

impl<'a> Name<'a> {
    pub fn new(name: String, value: String) -> Self {
        let name = Cow::Owned(name);
        let value = Cow::Owned(value);
        let output_line = Cow::Owned(Self::calculate_line(&name, &value));
        Self {
            name,
            value,
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub(crate) fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Cow::Owned(name);
        self.output_line_is_dirty = true;
    }

    pub fn set_value(&mut self, value: String) {
        self.value = Cow::Owned(value);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(Self::calculate_line(
            &self.name().into(),
            &self.value().into(),
        ));
        self.output_line_is_dirty = false;
    }

    fn calculate_line<'b>(name: &Cow<'b, str>, value: &Cow<'b, str>) -> String {
        format!("#EXT-X-DEFINE:{NAME}=\"{name}\",{VALUE}=\"{value}\"")
    }
}

#[derive(Debug)]
pub struct Import<'a> {
    import: Cow<'a, str>,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Import<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.import() == other.import()
    }
}

impl<'a> Import<'a> {
    pub fn new(import: String) -> Self {
        let import = Cow::Owned(import);
        let output_line = Cow::Owned(Self::calculate_line(&import));
        Self {
            import,
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub(crate) fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn import(&self) -> &str {
        &self.import
    }

    pub fn set_import(&mut self, import: String) {
        self.import = Cow::Owned(import);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(Self::calculate_line(&self.import().into()));
        self.output_line_is_dirty = false;
    }

    fn calculate_line<'b>(import: &'b Cow<'b, str>) -> String {
        format!("#EXT-X-DEFINE:{IMPORT}=\"{import}\"")
    }
}

#[derive(Debug)]
pub struct Queryparam<'a> {
    queryparam: Cow<'a, str>,
    output_line: Cow<'a, str>,  // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Queryparam<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.queryparam() == other.queryparam()
    }
}

impl<'a> Queryparam<'a> {
    pub fn new(queryparam: String) -> Self {
        let queryparam = Cow::Owned(queryparam);
        let output_line = Cow::Owned(Self::calculate_line(&queryparam));
        Self {
            queryparam,
            output_line,
            output_line_is_dirty: false,
        }
    }

    pub(crate) fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn queryparam(&self) -> &str {
        &self.queryparam
    }

    pub fn set_queryparam(&mut self, queryparam: String) {
        self.queryparam = Cow::Owned(queryparam);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(Self::calculate_line(&self.queryparam().into()));
        self.output_line_is_dirty = false;
    }

    fn calculate_line<'b>(queryparam: &'b Cow<'b, str>) -> String {
        format!("#EXT-X-DEFINE:{QUERYPARAM}=\"{queryparam}\"")
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.3
#[derive(Debug, PartialEq)]
pub enum Define<'a> {
    Name(Name<'a>),
    Import(Import<'a>),
    Queryparam(Queryparam<'a>),
}

impl<'a> TryFrom<ParsedTag<'a>> for Define<'a> {
    type Error = &'static str;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        if let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.get("NAME") {
            if let Some(ParsedAttributeValue::QuotedString(value)) = attribute_list.get("VALUE") {
                Ok(Self::Name(Name {
                    name: Cow::Borrowed(name),
                    value: Cow::Borrowed(value),
                    output_line: Cow::Borrowed(tag.original_input),
                    output_line_is_dirty: false,
                }))
            } else {
                Err(super::ValidationError::missing_required_attribute())
            }
        } else if let Some(ParsedAttributeValue::QuotedString(import)) =
            attribute_list.get("IMPORT")
        {
            Ok(Self::Import(Import {
                import: Cow::Borrowed(import),
                output_line: Cow::Borrowed(tag.original_input),
                output_line_is_dirty: false,
            }))
        } else if let Some(ParsedAttributeValue::QuotedString(queryparam)) =
            attribute_list.get("QUERYPARAM")
        {
            Ok(Self::Queryparam(Queryparam {
                queryparam: Cow::Borrowed(queryparam),
                output_line: Cow::Borrowed(tag.original_input),
                output_line_is_dirty: false,
            }))
        } else {
            Err(super::ValidationError::missing_required_attribute())
        }
    }
}

impl<'a> Define<'a> {
    pub fn new_name(name: String, value: String) -> Self {
        Self::Name(Name::new(name, value))
    }

    pub fn new_import(import: String) -> Self {
        Self::Import(Import::new(import))
    }

    pub fn new_queryparam(queryparam: String) -> Self {
        Self::Queryparam(Queryparam::new(queryparam))
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Name(name) => Some(name.name()),
            _ => None,
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Name(name) => Some(name.value()),
            _ => None,
        }
    }

    pub fn import(&self) -> Option<&str> {
        match self {
            Self::Import(import) => Some(import.import()),
            _ => None,
        }
    }

    pub fn queryparam(&self) -> Option<&str> {
        match self {
            Self::Queryparam(queryparam) => Some(queryparam.queryparam()),
            _ => None,
        }
    }

    pub fn set_name_and_value(&mut self, name: String, value: String) {
        *self = Self::new_name(name, value);
    }

    pub fn set_import(&mut self, import: String) {
        *self = Self::new_import(import);
    }

    pub fn set_queryparam(&mut self, queryparam: String) {
        *self = Self::new_queryparam(queryparam);
    }

    pub(crate) fn into_inner(self) -> TagInner<'a> {
        match self {
            Define::Name(name) => name.into_inner(),
            Define::Import(import) => import.into_inner(),
            Define::Queryparam(queryparam) => queryparam.into_inner(),
        }
    }
}

const NAME: &str = "NAME";
const VALUE: &str = "VALUE";
const IMPORT: &str = "IMPORT";
const QUERYPARAM: &str = "QUERYPARAM";
