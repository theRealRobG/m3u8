use std::collections::HashMap;

use crate::tag::value::{ParsedAttributeValue, ParsedTagValue};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.1
#[derive(Debug, PartialEq)]
pub struct Media<'a> {
    media_type: &'a str,
    group_id: &'a str,
    name: &'a str,
    // Original attribute list
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>,
}

impl<'a> TryFrom<ParsedTagValue<'a>> for Media<'a> {
    type Error = &'static str;

    fn try_from(value: ParsedTagValue<'a>) -> Result<Self, Self::Error> {
        let ParsedTagValue::AttributeList(attribute_list) = value else {
            return Err(super::ValidationError::unexpected_value_type());
        };
        let Some(ParsedAttributeValue::UnquotedString(media_type)) = attribute_list.get(TYPE)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::QuotedString(group_id)) = attribute_list.get(GROUP_ID)
        else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.get(NAME) else {
            return Err(super::ValidationError::missing_required_attribute());
        };
        Ok(Self {
            media_type,
            group_id,
            name,
            attribute_list,
        })
    }
}

impl<'a> Media<'a> {
    pub fn new(
        media_type: &'a str,
        name: &'a str,
        group_id: &'a str,
        uri: Option<&'a str>,
        language: Option<&'a str>,
        assoc_language: Option<&'a str>,
        stable_rendition_id: Option<&'a str>,
        default: bool,
        autoselect: bool,
        forced: bool,
        instream_id: Option<&'a str>,
        bit_depth: Option<u64>,
        sample_rate: Option<u64>,
        characteristics: Option<&'a str>,
        channels: Option<&'a str>,
    ) -> Self {
        let mut attribute_list = HashMap::new();
        attribute_list.insert(TYPE, ParsedAttributeValue::UnquotedString(media_type));
        attribute_list.insert(NAME, ParsedAttributeValue::QuotedString(name));
        attribute_list.insert(GROUP_ID, ParsedAttributeValue::QuotedString(group_id));
        if let Some(uri) = uri {
            attribute_list.insert(URI, ParsedAttributeValue::QuotedString(uri));
        }
        if let Some(language) = language {
            attribute_list.insert(LANGUAGE, ParsedAttributeValue::QuotedString(language));
        }
        if let Some(assoc_language) = assoc_language {
            attribute_list.insert(
                ASSOC_LANGUAGE,
                ParsedAttributeValue::QuotedString(assoc_language),
            );
        }
        if let Some(stable_rendition_id) = stable_rendition_id {
            attribute_list.insert(
                STABLE_RENDITION_ID,
                ParsedAttributeValue::QuotedString(stable_rendition_id),
            );
        }
        if default {
            attribute_list.insert(DEFAULT, ParsedAttributeValue::UnquotedString(YES));
        }
        if autoselect {
            attribute_list.insert(AUTOSELECT, ParsedAttributeValue::UnquotedString(YES));
        }
        if forced {
            attribute_list.insert(FORCED, ParsedAttributeValue::UnquotedString(YES));
        }
        if let Some(instream_id) = instream_id {
            attribute_list.insert(INSTREAM_ID, ParsedAttributeValue::QuotedString(instream_id));
        }
        if let Some(bit_depth) = bit_depth {
            attribute_list.insert(BIT_DEPTH, ParsedAttributeValue::DecimalInteger(bit_depth));
        }
        if let Some(sample_rate) = sample_rate {
            attribute_list.insert(
                SAMPLE_RATE,
                ParsedAttributeValue::DecimalInteger(sample_rate),
            );
        }
        if let Some(characteristics) = characteristics {
            attribute_list.insert(
                CHARACTERISTICS,
                ParsedAttributeValue::QuotedString(characteristics),
            );
        }
        if let Some(channels) = channels {
            attribute_list.insert(CHANNELS, ParsedAttributeValue::QuotedString(channels));
        }
        Self {
            media_type,
            group_id,
            name,
            attribute_list,
        }
    }

    pub fn media_type(&self) -> &'a str {
        self.media_type
    }
    pub fn name(&self) -> &'a str {
        self.name
    }
    pub fn group_id(&self) -> &'a str {
        self.group_id
    }
    pub fn uri(&self) -> Option<&'a str> {
        match self.attribute_list.get(URI) {
            Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
            _ => None,
        }
    }
    pub fn language(&self) -> Option<&'a str> {
        match self.attribute_list.get(LANGUAGE) {
            Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
            _ => None,
        }
    }
    pub fn assoc_language(&self) -> Option<&'a str> {
        match self.attribute_list.get(ASSOC_LANGUAGE) {
            Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
            _ => None,
        }
    }
    pub fn stable_rendition_id(&self) -> Option<&'a str> {
        match self.attribute_list.get(STABLE_RENDITION_ID) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }
    pub fn default(&self) -> bool {
        matches!(
            self.attribute_list.get(DEFAULT),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }
    pub fn autoselect(&self) -> bool {
        matches!(
            self.attribute_list.get(AUTOSELECT),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }
    pub fn forced(&self) -> bool {
        matches!(
            self.attribute_list.get(FORCED),
            Some(ParsedAttributeValue::UnquotedString(YES))
        )
    }
    pub fn instream_id(&self) -> Option<&'a str> {
        match self.attribute_list.get(INSTREAM_ID) {
            Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
            _ => None,
        }
    }
    pub fn bit_depth(&self) -> Option<u64> {
        match self.attribute_list.get(BIT_DEPTH) {
            Some(ParsedAttributeValue::DecimalInteger(d)) => Some(*d),
            _ => None,
        }
    }
    pub fn sample_rate(&self) -> Option<u64> {
        match self.attribute_list.get(SAMPLE_RATE) {
            Some(ParsedAttributeValue::DecimalInteger(rate)) => Some(*rate),
            _ => None,
        }
    }
    pub fn characteristics(&self) -> Option<&'a str> {
        match self.attribute_list.get(CHARACTERISTICS) {
            Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
            _ => None,
        }
    }
    pub fn channels(&self) -> Option<&'a str> {
        match self.attribute_list.get(CHANNELS) {
            Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
            _ => None,
        }
    }
}

const TYPE: &'static str = "TYPE";
const URI: &'static str = "URI";
const GROUP_ID: &'static str = "GROUP-ID";
const LANGUAGE: &'static str = "LANGUAGE";
const ASSOC_LANGUAGE: &'static str = "ASSOC-LANGUAGE";
const NAME: &'static str = "NAME";
const STABLE_RENDITION_ID: &'static str = "STABLE-RENDITION-ID";
const DEFAULT: &'static str = "DEFAULT";
const AUTOSELECT: &'static str = "AUTOSELECT";
const FORCED: &'static str = "FORCED";
const INSTREAM_ID: &'static str = "INSTREAM-ID";
const BIT_DEPTH: &'static str = "BIT-DEPTH";
const SAMPLE_RATE: &'static str = "SAMPLE-RATE";
const CHARACTERISTICS: &'static str = "CHARACTERISTICS";
const CHANNELS: &'static str = "CHANNELS";
const YES: &'static str = "YES";
