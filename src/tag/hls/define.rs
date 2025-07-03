use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Name<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Name<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name() && self.value() == other.value()
    }
}

impl<'a> Name<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        let name = name.into();
        let value = value.into();
        let output_line = Cow::Owned(Self::calculate_line(&name, &value));
        Self {
            name,
            value,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_name(&mut self, name: impl Into<Cow<'a, str>>) {
        self.name = name.into();
        self.output_line_is_dirty = true;
    }

    pub fn set_value(&mut self, value: impl Into<Cow<'a, str>>) {
        self.value = value.into();
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(Self::calculate_line(self.name(), self.value()));
        self.output_line_is_dirty = false;
    }

    fn calculate_line(name: &str, value: &str) -> Vec<u8> {
        format!("#EXT-X-DEFINE:{NAME}=\"{name}\",{VALUE}=\"{value}\"").into_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Import<'a> {
    import: Cow<'a, str>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Import<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.import() == other.import()
    }
}

impl<'a> Import<'a> {
    pub fn new(import: impl Into<Cow<'a, str>>) -> Self {
        let import = import.into();
        let output_line = Cow::Owned(Self::calculate_line(&import));
        Self {
            import,
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

    pub fn import(&self) -> &str {
        &self.import
    }

    pub fn set_import(&mut self, import: impl Into<Cow<'a, str>>) {
        self.import = import.into();
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(Self::calculate_line(self.import()));
        self.output_line_is_dirty = false;
    }

    fn calculate_line(import: &str) -> Vec<u8> {
        format!("#EXT-X-DEFINE:{IMPORT}=\"{import}\"").into_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct Queryparam<'a> {
    queryparam: Cow<'a, str>,
    output_line: Cow<'a, [u8]>, // Used with Writer
    output_line_is_dirty: bool, // If should recalculate output_line
}

impl<'a> PartialEq for Queryparam<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.queryparam() == other.queryparam()
    }
}

impl<'a> Queryparam<'a> {
    pub fn new(queryparam: impl Into<Cow<'a, str>>) -> Self {
        let queryparam = queryparam.into();
        let output_line = Cow::Owned(Self::calculate_line(&queryparam));
        Self {
            queryparam,
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

    pub fn queryparam(&self) -> &str {
        &self.queryparam
    }

    pub fn set_queryparam(&mut self, queryparam: impl Into<Cow<'a, str>>) {
        self.queryparam = queryparam.into();
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(Self::calculate_line(self.queryparam()));
        self.output_line_is_dirty = false;
    }

    fn calculate_line(queryparam: &str) -> Vec<u8> {
        format!("#EXT-X-DEFINE:{QUERYPARAM}=\"{queryparam}\"").into_bytes()
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.2.3
#[derive(Debug, PartialEq, Clone)]
pub enum Define<'a> {
    Name(Name<'a>),
    Import(Import<'a>),
    Queryparam(Queryparam<'a>),
}

impl<'a> TryFrom<ParsedTag<'a>> for Define<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        if let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.get(NAME) {
            if let Some(ParsedAttributeValue::QuotedString(value)) = attribute_list.get(VALUE) {
                Ok(Self::Name(Name {
                    name: Cow::Borrowed(name),
                    value: Cow::Borrowed(value),
                    output_line: Cow::Borrowed(tag.original_input),
                    output_line_is_dirty: false,
                }))
            } else {
                Err(super::ValidationError::MissingRequiredAttribute(VALUE))
            }
        } else if let Some(ParsedAttributeValue::QuotedString(import)) = attribute_list.get(IMPORT)
        {
            Ok(Self::Import(Import {
                import: Cow::Borrowed(import),
                output_line: Cow::Borrowed(tag.original_input),
                output_line_is_dirty: false,
            }))
        } else if let Some(ParsedAttributeValue::QuotedString(queryparam)) =
            attribute_list.get(QUERYPARAM)
        {
            Ok(Self::Queryparam(Queryparam {
                queryparam: Cow::Borrowed(queryparam),
                output_line: Cow::Borrowed(tag.original_input),
                output_line_is_dirty: false,
            }))
        } else {
            Err(super::ValidationError::MissingRequiredAttribute(NAME))
        }
    }
}

impl<'a> Define<'a> {
    pub fn new_name(name: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self::Name(Name::new(name, value))
    }

    pub fn new_import(import: impl Into<Cow<'a, str>>) -> Self {
        Self::Import(Import::new(import))
    }

    pub fn new_queryparam(queryparam: impl Into<Cow<'a, str>>) -> Self {
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

    pub fn set_name_and_value(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<Cow<'a, str>>,
    ) {
        *self = Self::new_name(name, value);
    }

    pub fn set_import(&mut self, import: impl Into<Cow<'a, str>>) {
        *self = Self::new_import(import);
    }

    pub fn set_queryparam(&mut self, queryparam: impl Into<Cow<'a, str>>) {
        *self = Self::new_queryparam(queryparam);
    }

    pub fn into_inner(self) -> TagInner<'a> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_name_value_should_work() {
        assert_eq!(
            b"#EXT-X-DEFINE:NAME=\"name\",VALUE=\"value\"",
            Define::new_name("name", "value").into_inner().value()
        );
    }

    #[test]
    fn set_name_and_value_should_work() {
        let mut define = Define::new_import("import");
        define.set_name_and_value("name", "value");
        assert_eq!(
            b"#EXT-X-DEFINE:NAME=\"name\",VALUE=\"value\"",
            define.into_inner().value()
        );
    }

    #[test]
    fn new_import_should_work() {
        assert_eq!(
            b"#EXT-X-DEFINE:IMPORT=\"import\"",
            Define::new_import("import").into_inner().value()
        );
    }

    #[test]
    fn set_import_should_work() {
        let mut define = Define::new_queryparam("queryparam");
        define.set_import("import");
        assert_eq!(
            b"#EXT-X-DEFINE:IMPORT=\"import\"",
            define.into_inner().value()
        );
    }

    #[test]
    fn new_queryparam_should_work() {
        assert_eq!(
            b"#EXT-X-DEFINE:QUERYPARAM=\"queryparam\"",
            Define::new_queryparam("queryparam").into_inner().value()
        );
    }

    #[test]
    fn set_queryparam_should_work() {
        let mut define = Define::new_import("import");
        define.set_queryparam("queryparam");
        assert_eq!(
            b"#EXT-X-DEFINE:QUERYPARAM=\"queryparam\"",
            define.into_inner().value()
        );
    }

    #[cfg(test)]
    mod name_value {
        use super::*;
        use crate::tag::hls::test_macro::mutation_tests;
        use pretty_assertions::assert_eq;

        mutation_tests!(
            Name::new("name", "value"),
            (name, "other_name", @Attr="NAME=\"other_name\""),
            (value, "other_value", @Attr="VALUE=\"other_value\"")
        );
    }

    #[cfg(test)]
    mod import {
        use super::*;
        use crate::tag::hls::test_macro::mutation_tests;
        use pretty_assertions::assert_eq;

        mutation_tests!(
            Import::new("import"),
            (import, "other_import", @Attr="IMPORT=\"other_import\"")
        );
    }

    #[cfg(test)]
    mod queryparam {
        use super::*;
        use crate::tag::hls::test_macro::mutation_tests;
        use pretty_assertions::assert_eq;

        mutation_tests!(
            Queryparam::new("queryparam"),
            (queryparam, "other_query_param", @Attr="QUERYPARAM=\"other_query_param\"")
        );
    }
}
