use crate::{
    error::{ValidationError, ValidationErrorValueKind},
    tag::{
        hls::TagInner,
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap};

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.1
#[derive(Debug, Clone)]
pub struct Media<'a> {
    media_type: Cow<'a, str>,
    group_id: Cow<'a, str>,
    name: Cow<'a, str>,
    uri: Option<Cow<'a, str>>,
    language: Option<Cow<'a, str>>,
    assoc_language: Option<Cow<'a, str>>,
    stable_rendition_id: Option<Cow<'a, str>>,
    default: Option<bool>,
    autoselect: Option<bool>,
    forced: Option<bool>,
    instream_id: Option<Cow<'a, str>>,
    bit_depth: Option<u64>,
    sample_rate: Option<u64>,
    characteristics: Option<Cow<'a, str>>,
    channels: Option<Cow<'a, str>>,
    attribute_list: HashMap<&'a str, ParsedAttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, str>,                                  // Used with Writer
    output_line_is_dirty: bool,                                 // If should recalculate output_line
}

impl<'a> PartialEq for Media<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.media_type() == other.media_type()
            && self.group_id() == other.group_id()
            && self.name() == other.name()
            && self.uri() == other.uri()
            && self.language() == other.language()
            && self.assoc_language() == other.assoc_language()
            && self.stable_rendition_id() == other.stable_rendition_id()
            && self.default() == other.default()
            && self.autoselect() == other.autoselect()
            && self.forced() == other.forced()
            && self.instream_id() == other.instream_id()
            && self.bit_depth() == other.bit_depth()
            && self.sample_rate() == other.sample_rate()
            && self.characteristics() == other.characteristics()
            && self.channels() == other.channels()
    }
}

impl<'a> TryFrom<ParsedTag<'a>> for Media<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(super::ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let Some(ParsedAttributeValue::UnquotedString(media_type)) = attribute_list.get(TYPE)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(TYPE));
        };
        let Some(ParsedAttributeValue::QuotedString(group_id)) = attribute_list.get(GROUP_ID)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(GROUP_ID));
        };
        let Some(ParsedAttributeValue::QuotedString(name)) = attribute_list.get(NAME) else {
            return Err(super::ValidationError::MissingRequiredAttribute(NAME));
        };
        Ok(Self {
            media_type: Cow::Borrowed(media_type),
            group_id: Cow::Borrowed(group_id),
            name: Cow::Borrowed(name),
            uri: None,
            language: None,
            assoc_language: None,
            stable_rendition_id: None,
            default: None,
            autoselect: None,
            forced: None,
            instream_id: None,
            bit_depth: None,
            sample_rate: None,
            characteristics: None,
            channels: None,
            attribute_list,
            output_line: Cow::Borrowed(tag.original_input),
            output_line_is_dirty: false,
        })
    }
}

impl<'a> Media<'a> {
    pub fn new(
        media_type: String,
        name: String,
        group_id: String,
        uri: Option<String>,
        language: Option<String>,
        assoc_language: Option<String>,
        stable_rendition_id: Option<String>,
        default: bool,
        autoselect: bool,
        forced: bool,
        instream_id: Option<String>,
        bit_depth: Option<u64>,
        sample_rate: Option<u64>,
        characteristics: Option<String>,
        channels: Option<String>,
    ) -> Self {
        let media_type = Cow::Owned(media_type);
        let name = Cow::Owned(name);
        let group_id = Cow::Owned(group_id);
        let uri = uri.map(Cow::Owned);
        let language = language.map(Cow::Owned);
        let assoc_language = assoc_language.map(Cow::Owned);
        let stable_rendition_id = stable_rendition_id.map(Cow::Owned);
        let instream_id = instream_id.map(Cow::Owned);
        let characteristics = characteristics.map(Cow::Owned);
        let channels = channels.map(Cow::Owned);
        let output_line = Cow::Owned(calculate_line(
            &media_type,
            &name,
            &group_id,
            &uri,
            &language,
            &assoc_language,
            &stable_rendition_id,
            default,
            autoselect,
            forced,
            &instream_id,
            bit_depth,
            sample_rate,
            &characteristics,
            &channels,
        ));
        Self {
            media_type,
            group_id,
            name,
            uri,
            language,
            assoc_language,
            stable_rendition_id,
            default: Some(default),
            autoselect: Some(autoselect),
            forced: Some(forced),
            instream_id,
            bit_depth,
            sample_rate,
            characteristics,
            channels,
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

    pub fn media_type(&self) -> &str {
        &self.media_type
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn group_id(&self) -> &str {
        &self.group_id
    }
    pub fn uri(&self) -> Option<&str> {
        if let Some(uri) = &self.uri {
            Some(uri)
        } else {
            match self.attribute_list.get(URI) {
                Some(ParsedAttributeValue::QuotedString(uri)) => Some(uri),
                _ => None,
            }
        }
    }
    pub fn language(&self) -> Option<&str> {
        if let Some(language) = &self.language {
            Some(language)
        } else {
            match self.attribute_list.get(LANGUAGE) {
                Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
                _ => None,
            }
        }
    }
    pub fn assoc_language(&self) -> Option<&str> {
        if let Some(assoc_language) = &self.assoc_language {
            Some(assoc_language)
        } else {
            match self.attribute_list.get(ASSOC_LANGUAGE) {
                Some(ParsedAttributeValue::QuotedString(language)) => Some(language),
                _ => None,
            }
        }
    }
    pub fn stable_rendition_id(&self) -> Option<&str> {
        if let Some(stable_rendition_id) = &self.stable_rendition_id {
            Some(stable_rendition_id)
        } else {
            match self.attribute_list.get(STABLE_RENDITION_ID) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }
    pub fn default(&self) -> bool {
        if let Some(default) = self.default {
            default
        } else {
            matches!(
                self.attribute_list.get(DEFAULT),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }
    pub fn autoselect(&self) -> bool {
        if let Some(autoselect) = self.autoselect {
            autoselect
        } else {
            matches!(
                self.attribute_list.get(AUTOSELECT),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }
    pub fn forced(&self) -> bool {
        if let Some(forced) = self.forced {
            forced
        } else {
            matches!(
                self.attribute_list.get(FORCED),
                Some(ParsedAttributeValue::UnquotedString(YES))
            )
        }
    }
    pub fn instream_id(&self) -> Option<&str> {
        if let Some(instream_id) = &self.instream_id {
            Some(instream_id)
        } else {
            match self.attribute_list.get(INSTREAM_ID) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(s),
                _ => None,
            }
        }
    }
    pub fn bit_depth(&self) -> Option<u64> {
        if let Some(bit_depth) = self.bit_depth {
            Some(bit_depth)
        } else {
            match self.attribute_list.get(BIT_DEPTH) {
                Some(ParsedAttributeValue::DecimalInteger(d)) => Some(*d),
                _ => None,
            }
        }
    }
    pub fn sample_rate(&self) -> Option<u64> {
        if let Some(sample_rate) = self.sample_rate {
            Some(sample_rate)
        } else {
            match self.attribute_list.get(SAMPLE_RATE) {
                Some(ParsedAttributeValue::DecimalInteger(rate)) => Some(*rate),
                _ => None,
            }
        }
    }
    pub fn characteristics(&self) -> Option<&str> {
        if let Some(characteristics) = &self.characteristics {
            Some(characteristics)
        } else {
            match self.attribute_list.get(CHARACTERISTICS) {
                Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
                _ => None,
            }
        }
    }
    pub fn channels(&self) -> Option<&str> {
        if let Some(channels) = &self.channels {
            Some(channels)
        } else {
            match self.attribute_list.get(CHANNELS) {
                Some(ParsedAttributeValue::QuotedString(c)) => Some(c),
                _ => None,
            }
        }
    }

    pub fn set_media_type(&mut self, media_type: String) {
        self.attribute_list.remove(TYPE);
        self.media_type = Cow::Owned(media_type);
        self.output_line_is_dirty = true;
    }
    pub fn set_name(&mut self, name: String) {
        self.attribute_list.remove(NAME);
        self.name = Cow::Owned(name);
        self.output_line_is_dirty = true;
    }
    pub fn set_group_id(&mut self, group_id: String) {
        self.attribute_list.remove(GROUP_ID);
        self.group_id = Cow::Owned(group_id);
        self.output_line_is_dirty = true;
    }
    pub fn set_uri(&mut self, uri: Option<String>) {
        self.attribute_list.remove(URI);
        self.uri = uri.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }
    pub fn set_language(&mut self, language: Option<String>) {
        self.attribute_list.remove(LANGUAGE);
        self.language = language.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }
    pub fn set_assoc_language(&mut self, assoc_language: Option<String>) {
        self.attribute_list.remove(ASSOC_LANGUAGE);
        self.assoc_language = assoc_language.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }
    pub fn set_stable_rendition_id(&mut self, stable_rendition_id: Option<String>) {
        self.attribute_list.remove(STABLE_RENDITION_ID);
        self.stable_rendition_id = stable_rendition_id.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }
    pub fn set_default(&mut self, default: bool) {
        self.attribute_list.remove(DEFAULT);
        self.default = Some(default);
        self.output_line_is_dirty = true;
    }
    pub fn set_autoselect(&mut self, autoselect: bool) {
        self.attribute_list.remove(AUTOSELECT);
        self.autoselect = Some(autoselect);
        self.output_line_is_dirty = true;
    }
    pub fn set_forced(&mut self, forced: bool) {
        self.attribute_list.remove(FORCED);
        self.forced = Some(forced);
        self.output_line_is_dirty = true;
    }
    pub fn set_instream_id(&mut self, instream_id: Option<String>) {
        self.attribute_list.remove(INSTREAM_ID);
        self.instream_id = instream_id.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }
    pub fn set_bit_depth(&mut self, bit_depth: Option<u64>) {
        self.attribute_list.remove(BIT_DEPTH);
        self.bit_depth = bit_depth;
        self.output_line_is_dirty = true;
    }
    pub fn set_sample_rate(&mut self, sample_rate: Option<u64>) {
        self.attribute_list.remove(SAMPLE_RATE);
        self.sample_rate = sample_rate;
        self.output_line_is_dirty = true;
    }
    pub fn set_characteristics(&mut self, characteristics: Option<String>) {
        self.attribute_list.remove(CHARACTERISTICS);
        self.characteristics = characteristics.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }
    pub fn set_channels(&mut self, channels: Option<String>) {
        self.attribute_list.remove(CHANNELS);
        self.channels = channels.map(Cow::Owned);
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(
            self.media_type(),
            self.name(),
            self.group_id(),
            &self.uri().map(|x| x.into()),
            &self.language().map(|x| x.into()),
            &self.assoc_language().map(|x| x.into()),
            &self.stable_rendition_id().map(|x| x.into()),
            self.default(),
            self.autoselect(),
            self.forced(),
            &self.instream_id().map(|x| x.into()),
            self.bit_depth(),
            self.sample_rate(),
            &self.characteristics().map(|x| x.into()),
            &self.channels().map(|x| x.into()),
        ));
        self.output_line_is_dirty = false;
    }
}

const TYPE: &str = "TYPE";
const URI: &str = "URI";
const GROUP_ID: &str = "GROUP-ID";
const LANGUAGE: &str = "LANGUAGE";
const ASSOC_LANGUAGE: &str = "ASSOC-LANGUAGE";
const NAME: &str = "NAME";
const STABLE_RENDITION_ID: &str = "STABLE-RENDITION-ID";
const DEFAULT: &str = "DEFAULT";
const AUTOSELECT: &str = "AUTOSELECT";
const FORCED: &str = "FORCED";
const INSTREAM_ID: &str = "INSTREAM-ID";
const BIT_DEPTH: &str = "BIT-DEPTH";
const SAMPLE_RATE: &str = "SAMPLE-RATE";
const CHARACTERISTICS: &str = "CHARACTERISTICS";
const CHANNELS: &str = "CHANNELS";
const YES: &str = "YES";

fn calculate_line<'a>(
    media_type: &str,
    name: &str,
    group_id: &str,
    uri: &Option<Cow<'a, str>>,
    language: &Option<Cow<'a, str>>,
    assoc_language: &Option<Cow<'a, str>>,
    stable_rendition_id: &Option<Cow<'a, str>>,
    default: bool,
    autoselect: bool,
    forced: bool,
    instream_id: &Option<Cow<'a, str>>,
    bit_depth: Option<u64>,
    sample_rate: Option<u64>,
    characteristics: &Option<Cow<'a, str>>,
    channels: &Option<Cow<'a, str>>,
) -> String {
    let mut line =
        format!("#EXT-X-MEDIA:{TYPE}={media_type},{NAME}=\"{name}\",{GROUP_ID}=\"{group_id}\"");
    if let Some(uri) = uri {
        line.push_str(format!(",{URI}=\"{uri}\"").as_str());
    }
    if let Some(language) = language {
        line.push_str(format!(",{LANGUAGE}=\"{language}\"").as_str());
    }
    if let Some(assoc_language) = assoc_language {
        line.push_str(format!(",{ASSOC_LANGUAGE}=\"{assoc_language}\"").as_str());
    }
    if let Some(stable_rendition_id) = stable_rendition_id {
        line.push_str(format!(",{STABLE_RENDITION_ID}=\"{stable_rendition_id}\"").as_str());
    }
    if default {
        line.push_str(format!(",{DEFAULT}={YES}").as_str());
    }
    if autoselect {
        line.push_str(format!(",{AUTOSELECT}={YES}").as_str());
    }
    if forced {
        line.push_str(format!(",{FORCED}={YES}").as_str());
    }
    if let Some(instream_id) = instream_id {
        line.push_str(format!(",{INSTREAM_ID}=\"{instream_id}\"").as_str());
    }
    if let Some(bit_depth) = bit_depth {
        line.push_str(format!(",{BIT_DEPTH}={bit_depth}").as_str());
    }
    if let Some(sample_rate) = sample_rate {
        line.push_str(format!(",{SAMPLE_RATE}={sample_rate}").as_str());
    }
    if let Some(characteristics) = characteristics {
        line.push_str(format!(",{CHARACTERISTICS}=\"{characteristics}\"").as_str());
    }
    if let Some(channels) = channels {
        line.push_str(format!(",{CHANNELS}=\"{channels}\"").as_str());
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn as_str_with_no_options_should_be_valid() {
        assert_eq!(
            concat!(
                "#EXT-X-MEDIA:",
                "TYPE=CLOSED-CAPTIONS,",
                "NAME=\"English\",",
                "GROUP-ID=\"cc\",",
                "INSTREAM-ID=\"CC1\""
            ),
            Media::new(
                "CLOSED-CAPTIONS".to_string(),
                "English".to_string(),
                "cc".to_string(),
                None,
                None,
                None,
                None,
                false,
                false,
                false,
                Some("CC1".to_string()),
                None,
                None,
                None,
                None,
            )
            .into_inner()
            .value()
        );
    }

    #[test]
    fn as_str_with_options_should_be_valid() {
        assert_eq!(
            concat!(
                "#EXT-X-MEDIA:",
                "TYPE=AUDIO,",
                "NAME=\"English\",",
                "GROUP-ID=\"stereo\",",
                "URI=\"audio/en/stereo.m3u8\",",
                "LANGUAGE=\"en\",",
                "ASSOC-LANGUAGE=\"en\",",
                "STABLE-RENDITION-ID=\"1234\",",
                "DEFAULT=YES,",
                "AUTOSELECT=YES,",
                "FORCED=YES,",
                "BIT-DEPTH=8,",
                "SAMPLE-RATE=48000,",
                "CHARACTERISTICS=\"public.accessibility.describes-video\",",
                "CHANNELS=\"2\"",
            ),
            Media::new(
                "AUDIO".to_string(),
                "English".to_string(),
                "stereo".to_string(),
                Some("audio/en/stereo.m3u8".to_string()),
                Some("en".to_string()),
                Some("en".to_string()),
                Some("1234".to_string()),
                true,
                true,
                true,
                None,
                Some(8),
                Some(48000),
                Some("public.accessibility.describes-video".to_string()),
                Some("2".to_string()),
            )
            .into_inner()
            .value()
        );
    }
}
