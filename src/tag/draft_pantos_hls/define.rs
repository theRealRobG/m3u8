use crate::{
    tag::{
        known::ParsedTag,
        value::{ParsedAttributeValue, ParsedTagValue},
    },
    utils::{split_by_first_lf, str_from},
};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct Name<'a> {
    name: &'a str,
    value: &'a str,
    output_line: Cow<'a, [u8]>, // Used with Writer
}
impl<'a> Name<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        let output_line = Cow::Owned(Self::calculate_line(name, value).into_bytes());
        Self {
            name,
            value,
            output_line,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn value(&self) -> &'a str {
        self.value
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }

    fn calculate_line(name: &'a str, value: &'a str) -> String {
        format!("#EXT-X-DEFINE:{NAME}=\"{name}\",{VALUE}=\"{value}\"")
    }
}

#[derive(Debug, PartialEq)]
pub struct Import<'a> {
    import: &'a str,
    output_line: Cow<'a, [u8]>, // Used with Writer
}
impl<'a> Import<'a> {
    pub fn new(import: &'a str) -> Self {
        let output_line = Cow::Owned(Self::calculate_line(import).into_bytes());
        Self {
            import,
            output_line,
        }
    }

    pub fn import(&self) -> &'a str {
        self.import
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }

    fn calculate_line(import: &'a str) -> String {
        format!("#EXT-X-DEFINE:{IMPORT}=\"{import}\"")
    }
}

#[derive(Debug, PartialEq)]
pub struct Queryparam<'a> {
    queryparam: &'a str,
    output_line: Cow<'a, [u8]>, // Used with Writer
}
impl<'a> Queryparam<'a> {
    pub fn new(queryparam: &'a str) -> Self {
        let output_line = Cow::Owned(Self::calculate_line(queryparam).into_bytes());
        Self {
            queryparam,
            output_line,
        }
    }

    pub fn queryparam(&self) -> &'a str {
        self.queryparam
    }

    pub fn as_str(&self) -> &str {
        split_by_first_lf(str_from(&self.output_line)).parsed
    }

    fn calculate_line(queryparam: &'a str) -> String {
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
                    name,
                    value,
                    output_line: Cow::Borrowed(tag.original_input.as_bytes()),
                }))
            } else {
                Err(super::ValidationError::missing_required_attribute())
            }
        } else if let Some(ParsedAttributeValue::QuotedString(import)) =
            attribute_list.get("IMPORT")
        {
            Ok(Self::Import(Import {
                import,
                output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            }))
        } else if let Some(ParsedAttributeValue::QuotedString(queryparam)) =
            attribute_list.get("QUERYPARAM")
        {
            Ok(Self::Queryparam(Queryparam {
                queryparam,
                output_line: Cow::Borrowed(tag.original_input.as_bytes()),
            }))
        } else {
            Err(super::ValidationError::missing_required_attribute())
        }
    }
}

impl<'a> Define<'a> {
    pub fn new_name(name: &'a str, value: &'a str) -> Self {
        Self::Name(Name::new(name, value))
    }

    pub fn new_import(import: &'a str) -> Self {
        Self::Import(Import::new(import))
    }

    pub fn new_queryparam(queryparam: &'a str) -> Self {
        Self::Queryparam(Queryparam::new(queryparam))
    }

    pub fn name(&self) -> Option<&'a str> {
        match self {
            Self::Name(name) => Some(name.name()),
            _ => None,
        }
    }

    pub fn value(&self) -> Option<&'a str> {
        match self {
            Self::Name(name) => Some(name.value()),
            _ => None,
        }
    }

    pub fn import(&self) -> Option<&'a str> {
        match self {
            Self::Import(import) => Some(import.import()),
            _ => None,
        }
    }

    pub fn queryparam(&self) -> Option<&'a str> {
        match self {
            Self::Queryparam(queryparam) => Some(queryparam.queryparam()),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Define::Name(name) => name.as_str(),
            Define::Import(import) => import.as_str(),
            Define::Queryparam(queryparam) => queryparam.as_str(),
        }
    }
}

const NAME: &str = "NAME";
const VALUE: &str = "VALUE";
const IMPORT: &str = "IMPORT";
const QUERYPARAM: &str = "QUERYPARAM";
