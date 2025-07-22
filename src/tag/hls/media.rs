use crate::{
    error::{UnrecognizedEnumerationError, ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{EnumeratedString, TagInner},
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Audio,
    Video,
    Subtitles,
    ClosedCaptions,
}
impl<'a> TryFrom<&'a str> for Type {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            AUDIO => Ok(Self::Audio),
            VIDEO => Ok(Self::Video),
            SUBTITLES => Ok(Self::Subtitles),
            CLOSED_CAPTIONS => Ok(Self::ClosedCaptions),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Audio => write!(f, "{AUDIO}"),
            Type::Video => write!(f, "{VIDEO}"),
            Type::Subtitles => write!(f, "{SUBTITLES}"),
            Type::ClosedCaptions => write!(f, "{CLOSED_CAPTIONS}"),
        }
    }
}
impl From<Type> for EnumeratedString<'_, Type> {
    fn from(value: Type) -> Self {
        Self::Known(value)
    }
}
const AUDIO: &str = "AUDIO";
const VIDEO: &str = "VIDEO";
const SUBTITLES: &str = "SUBTITLES";
const CLOSED_CAPTIONS: &str = "CLOSED-CAPTIONS";

#[derive(Debug, PartialEq, Clone)]
pub struct MediaAttributeList<'a> {
    pub media_type: EnumeratedString<'a, Type>,
    pub name: Cow<'a, str>,
    pub group_id: Cow<'a, str>,
    pub uri: Option<Cow<'a, str>>,
    pub language: Option<Cow<'a, str>>,
    pub assoc_language: Option<Cow<'a, str>>,
    pub stable_rendition_id: Option<Cow<'a, str>>,
    pub default: bool,
    pub autoselect: bool,
    pub forced: bool,
    pub instream_id: Option<Cow<'a, str>>,
    pub bit_depth: Option<u64>,
    pub sample_rate: Option<u64>,
    pub characteristics: Option<Cow<'a, str>>,
    pub channels: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MediaBuilder<'a> {
    media_type: EnumeratedString<'a, Type>,
    name: Cow<'a, str>,
    group_id: Cow<'a, str>,
    uri: Option<Cow<'a, str>>,
    language: Option<Cow<'a, str>>,
    assoc_language: Option<Cow<'a, str>>,
    stable_rendition_id: Option<Cow<'a, str>>,
    default: bool,
    autoselect: bool,
    forced: bool,
    instream_id: Option<Cow<'a, str>>,
    bit_depth: Option<u64>,
    sample_rate: Option<u64>,
    characteristics: Option<Cow<'a, str>>,
    channels: Option<Cow<'a, str>>,
}
impl<'a> MediaBuilder<'a> {
    pub fn new(
        media_type: EnumeratedString<'a, Type>,
        name: impl Into<Cow<'a, str>>,
        group_id: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            media_type,
            name: name.into(),
            group_id: group_id.into(),
            uri: Default::default(),
            language: Default::default(),
            assoc_language: Default::default(),
            stable_rendition_id: Default::default(),
            default: Default::default(),
            autoselect: Default::default(),
            forced: Default::default(),
            instream_id: Default::default(),
            bit_depth: Default::default(),
            sample_rate: Default::default(),
            characteristics: Default::default(),
            channels: Default::default(),
        }
    }

    pub fn finish(self) -> Media<'a> {
        Media::new(MediaAttributeList {
            media_type: self.media_type,
            name: self.name,
            group_id: self.group_id,
            uri: self.uri,
            language: self.language,
            assoc_language: self.assoc_language,
            stable_rendition_id: self.stable_rendition_id,
            default: self.default,
            autoselect: self.autoselect,
            forced: self.forced,
            instream_id: self.instream_id,
            bit_depth: self.bit_depth,
            sample_rate: self.sample_rate,
            characteristics: self.characteristics,
            channels: self.channels,
        })
    }

    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> Self {
        self.uri = Some(uri.into());
        self
    }
    pub fn with_language(mut self, language: impl Into<Cow<'a, str>>) -> Self {
        self.language = Some(language.into());
        self
    }
    pub fn with_assoc_language(mut self, assoc_language: impl Into<Cow<'a, str>>) -> Self {
        self.assoc_language = Some(assoc_language.into());
        self
    }
    pub fn with_stable_rendition_id(
        mut self,
        stable_rendition_id: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.stable_rendition_id = Some(stable_rendition_id.into());
        self
    }
    pub fn with_default(mut self) -> Self {
        self.default = true;
        self
    }
    pub fn with_autoselect(mut self) -> Self {
        self.autoselect = true;
        self
    }
    pub fn with_forced(mut self) -> Self {
        self.forced = true;
        self
    }
    pub fn with_instream_id(mut self, instream_id: impl Into<Cow<'a, str>>) -> Self {
        self.instream_id = Some(instream_id.into());
        self
    }
    pub fn with_bit_depth(mut self, bit_depth: u64) -> Self {
        self.bit_depth = Some(bit_depth);
        self
    }
    pub fn with_sample_rate(mut self, sample_rate: u64) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }
    pub fn with_characteristics(mut self, characteristics: impl Into<Cow<'a, str>>) -> Self {
        self.characteristics = Some(characteristics.into());
        self
    }
    pub fn with_channels(mut self, channels: impl Into<Cow<'a, str>>) -> Self {
        self.channels = Some(channels.into());
        self
    }
}

/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.1
#[derive(Debug, Clone)]
pub struct Media<'a> {
    media_type: EnumeratedString<'a, Type>,
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
    output_line: Cow<'a, [u8]>,                                 // Used with Writer
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
            media_type: EnumeratedString::from(*media_type),
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
    pub fn new(attribute_list: MediaAttributeList<'a>) -> Self {
        let output_line = Cow::Owned(calculate_line(&attribute_list));
        let MediaAttributeList {
            media_type,
            name,
            group_id,
            uri,
            language,
            assoc_language,
            stable_rendition_id,
            default,
            autoselect,
            forced,
            instream_id,
            bit_depth,
            sample_rate,
            characteristics,
            channels,
        } = attribute_list;
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

    pub fn builder(
        media_type: EnumeratedString<'a, Type>,
        name: impl Into<Cow<'a, str>>,
        group_id: impl Into<Cow<'a, str>>,
    ) -> MediaBuilder<'a> {
        MediaBuilder::new(media_type, name, group_id)
    }

    pub fn into_inner(mut self) -> TagInner<'a> {
        if self.output_line_is_dirty {
            self.recalculate_output_line();
        }
        TagInner {
            output_line: self.output_line,
        }
    }

    pub fn media_type(&self) -> EnumeratedString<'a, Type> {
        self.media_type
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

    pub fn set_media_type(&mut self, media_type: EnumeratedString<'a, Type>) {
        self.attribute_list.remove(TYPE);
        self.media_type = media_type;
        self.output_line_is_dirty = true;
    }
    pub fn set_name(&mut self, name: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(NAME);
        self.name = name.into();
        self.output_line_is_dirty = true;
    }
    pub fn set_group_id(&mut self, group_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(GROUP_ID);
        self.group_id = group_id.into();
        self.output_line_is_dirty = true;
    }
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = Some(uri.into());
        self.output_line_is_dirty = true;
    }
    pub fn unset_uri(&mut self) {
        self.attribute_list.remove(URI);
        self.uri = None;
        self.output_line_is_dirty = true;
    }
    pub fn set_language(&mut self, language: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(LANGUAGE);
        self.language = Some(language.into());
        self.output_line_is_dirty = true;
    }
    pub fn unset_language(&mut self) {
        self.attribute_list.remove(LANGUAGE);
        self.language = None;
        self.output_line_is_dirty = true;
    }
    pub fn set_assoc_language(&mut self, assoc_language: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(ASSOC_LANGUAGE);
        self.assoc_language = Some(assoc_language.into());
        self.output_line_is_dirty = true;
    }
    pub fn unset_assoc_language(&mut self) {
        self.attribute_list.remove(ASSOC_LANGUAGE);
        self.assoc_language = None;
        self.output_line_is_dirty = true;
    }
    pub fn set_stable_rendition_id(&mut self, stable_rendition_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(STABLE_RENDITION_ID);
        self.stable_rendition_id = Some(stable_rendition_id.into());
        self.output_line_is_dirty = true;
    }
    pub fn unset_stable_rendition_id(&mut self) {
        self.attribute_list.remove(STABLE_RENDITION_ID);
        self.stable_rendition_id = None;
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
    pub fn set_instream_id(&mut self, instream_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(INSTREAM_ID);
        self.instream_id = Some(instream_id.into());
        self.output_line_is_dirty = true;
    }
    pub fn unset_instream_id(&mut self) {
        self.attribute_list.remove(INSTREAM_ID);
        self.instream_id = None;
        self.output_line_is_dirty = true;
    }
    pub fn set_bit_depth(&mut self, bit_depth: u64) {
        self.attribute_list.remove(BIT_DEPTH);
        self.bit_depth = Some(bit_depth);
        self.output_line_is_dirty = true;
    }
    pub fn unset_bit_depth(&mut self) {
        self.attribute_list.remove(BIT_DEPTH);
        self.bit_depth = None;
        self.output_line_is_dirty = true;
    }
    pub fn set_sample_rate(&mut self, sample_rate: u64) {
        self.attribute_list.remove(SAMPLE_RATE);
        self.sample_rate = Some(sample_rate);
        self.output_line_is_dirty = true;
    }
    pub fn unset_sample_rate(&mut self) {
        self.attribute_list.remove(SAMPLE_RATE);
        self.sample_rate = None;
        self.output_line_is_dirty = true;
    }
    pub fn set_characteristics(&mut self, characteristics: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CHARACTERISTICS);
        self.characteristics = Some(characteristics.into());
        self.output_line_is_dirty = true;
    }
    pub fn unset_characteristics(&mut self) {
        self.attribute_list.remove(CHARACTERISTICS);
        self.characteristics = None;
        self.output_line_is_dirty = true;
    }
    pub fn set_channels(&mut self, channels: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CHANNELS);
        self.channels = Some(channels.into());
        self.output_line_is_dirty = true;
    }
    pub fn unset_channels(&mut self) {
        self.attribute_list.remove(CHANNELS);
        self.channels = None;
        self.output_line_is_dirty = true;
    }

    fn recalculate_output_line(&mut self) {
        self.output_line = Cow::Owned(calculate_line(&MediaAttributeList {
            media_type: self.media_type().into(),
            name: self.name().into(),
            group_id: self.group_id().into(),
            uri: self.uri().map(|x| x.into()),
            language: self.language().map(|x| x.into()),
            assoc_language: self.assoc_language().map(|x| x.into()),
            stable_rendition_id: self.stable_rendition_id().map(|x| x.into()),
            default: self.default(),
            autoselect: self.autoselect(),
            forced: self.forced(),
            instream_id: self.instream_id().map(|x| x.into()),
            bit_depth: self.bit_depth(),
            sample_rate: self.sample_rate(),
            characteristics: self.characteristics().map(|x| x.into()),
            channels: self.channels().map(|x| x.into()),
        }));
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

fn calculate_line(attribute_list: &MediaAttributeList) -> Vec<u8> {
    let MediaAttributeList {
        media_type,
        name,
        group_id,
        uri,
        language,
        assoc_language,
        stable_rendition_id,
        default,
        autoselect,
        forced,
        instream_id,
        bit_depth,
        sample_rate,
        characteristics,
        channels,
    } = attribute_list;
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
    if *default {
        line.push_str(format!(",{DEFAULT}={YES}").as_str());
    }
    if *autoselect {
        line.push_str(format!(",{AUTOSELECT}={YES}").as_str());
    }
    if *forced {
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
    line.into_bytes()
}

#[cfg(test)]
mod tests {
    use crate::tag::hls::test_macro::mutation_tests;

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
            )
            .as_bytes(),
            Media::builder(Type::ClosedCaptions.into(), "English", "cc")
                .with_instream_id("CC1")
                .finish()
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
            )
            .as_bytes(),
            Media::builder(Type::Audio.into(), "English", "stereo")
                .with_uri("audio/en/stereo.m3u8")
                .with_language("en")
                .with_assoc_language("en")
                .with_stable_rendition_id("1234")
                .with_default()
                .with_autoselect()
                .with_forced()
                .with_bit_depth(8)
                .with_sample_rate(48000)
                .with_characteristics("public.accessibility.describes-video")
                .with_channels("2")
                .finish()
                .into_inner()
                .value()
        );
    }

    mutation_tests!(
        Media::builder(Type::Audio.into(), "English", "stereo")
            .with_uri("audio/en/stereo.m3u8")
            .with_language("en")
            .with_assoc_language("en")
            .with_stable_rendition_id("1234")
            .with_instream_id("ID1")
            .with_bit_depth(8)
            .with_sample_rate(48000)
            .with_characteristics("public.accessibility.describes-video")
            .with_channels("2")
            .finish(),
        (media_type, EnumeratedString::Known(Type::Video), @Attr="TYPE=VIDEO"),
        (name, "Spanish", @Attr="NAME=\"Spanish\""),
        (group_id, "surround", @Attr="GROUP-ID=\"surround\""),
        (uri, @Option "example", @Attr="URI=\"example\""),
        (language, @Option "es", @Attr="LANGUAGE=\"es\""),
        (assoc_language, @Option "es", @Attr="ASSOC-LANGUAGE=\"es\""),
        (stable_rendition_id, @Option "abcd", @Attr="STABLE-RENDITION-ID=\"abcd\""),
        (default, true, @Attr="DEFAULT=YES"),
        (autoselect, true, @Attr="AUTOSELECT=YES"),
        (forced, true, @Attr="FORCED=YES"),
        (instream_id, @Option "ID2", @Attr="INSTREAM-ID=\"ID2\""),
        (bit_depth, @Option 10, @Attr="BIT-DEPTH=10"),
        (sample_rate, @Option 42, @Attr="SAMPLE-RATE=42"),
        (characteristics, @Option "example", @Attr="CHARACTERISTICS=\"example\""),
        (channels, @Option "6", @Attr="CHANNELS=\"6\"")
    );
}
