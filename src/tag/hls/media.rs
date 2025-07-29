use crate::{
    error::{UnrecognizedEnumerationError, ValidationError, ValidationErrorValueKind},
    tag::{
        hls::{EnumeratedString, EnumeratedStringList, into_inner_tag},
        known::ParsedTag,
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
    utils::AsStaticCow,
};
use std::{borrow::Cow, collections::HashMap, fmt::Display, str::Split};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    /// Specifies an audio rendition.
    Audio,
    /// Specifies a video rendition.
    Video,
    /// Specifies a subtitles rendition.
    Subtitles,
    /// Typically, closed-caption media is carried in the video stream. Therefore, an EXT-X-MEDIA
    /// tag with TYPE of CLOSED-CAPTIONS does not specify a Rendition; the closed-caption media is
    /// present in the Media Segments of every video Rendition.
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
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for Type {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Type::Audio => Cow::Borrowed(AUDIO),
            Type::Video => Cow::Borrowed(VIDEO),
            Type::Subtitles => Cow::Borrowed(SUBTITLES),
            Type::ClosedCaptions => Cow::Borrowed(CLOSED_CAPTIONS),
        }
    }
}
impl From<Type> for Cow<'_, str> {
    fn from(value: Type) -> Self {
        value.as_cow()
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cea608InstreamId {
    Cc1,
    Cc2,
    Cc3,
    Cc4,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstreamId {
    /// The value identifies a Line 21 Data Services (CEA608) channel.
    Cea608(Cea608InstreamId),
    /// The value identifies a Digital Television Closed Captioning (CEA708) service block number.
    Cea708(u8),
}
impl<'a> TryFrom<&'a str> for InstreamId {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            CC1 => Ok(Self::Cea608(Cea608InstreamId::Cc1)),
            CC2 => Ok(Self::Cea608(Cea608InstreamId::Cc2)),
            CC3 => Ok(Self::Cea608(Cea608InstreamId::Cc3)),
            CC4 => Ok(Self::Cea608(Cea608InstreamId::Cc4)),
            s if s.starts_with(SERVICE) => Ok(Self::Cea708(
                s[7..]
                    .parse::<u8>()
                    .map_err(|_| UnrecognizedEnumerationError::new(s))?,
            )),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for InstreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for InstreamId {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            InstreamId::Cea608(Cea608InstreamId::Cc1) => Cow::Borrowed(CC1),
            InstreamId::Cea608(Cea608InstreamId::Cc2) => Cow::Borrowed(CC2),
            InstreamId::Cea608(Cea608InstreamId::Cc3) => Cow::Borrowed(CC3),
            InstreamId::Cea608(Cea608InstreamId::Cc4) => Cow::Borrowed(CC4),
            InstreamId::Cea708(id) => Cow::Owned(format!("{SERVICE}{id}")),
        }
    }
}
impl From<InstreamId> for Cow<'_, str> {
    fn from(value: InstreamId) -> Self {
        value.as_cow()
    }
}
impl From<InstreamId> for EnumeratedString<'_, InstreamId> {
    fn from(value: InstreamId) -> Self {
        Self::Known(value)
    }
}
const CC1: &str = "CC1";
const CC2: &str = "CC2";
const CC3: &str = "CC3";
const CC4: &str = "CC4";
const SERVICE: &str = "SERVICE";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaCharacteristicTag {
    /// Indicates that the rendition includes legible content that transcribes spoken dialog. It's
    /// possible for a legible media rendition to include both transcriptions of spoken dialog and
    /// descriptions of music and sound effects.
    TranscribesSpokenDialog,
    /// Indicates that the rendition includes legible content. Legible media may include
    /// transcriptions of spoken dialog and descriptions of music and sound effects.
    DescribesMusicAndSound,
    /// Indicates that subtitles have been edited for ease of reading. Closed caption tracks that
    /// carry "easy reader" captions, as the CEA-608 specification defines, should have this
    /// characteristic.
    EasyToRead,
    /// Indicates the media includes audible content that describes the visual portion of the
    /// presentation.
    DescribesVideo,
    /// Indicates that the Rendition was authored or translated programmatically.
    MachineGenerated,
}
impl<'a> TryFrom<&'a str> for MediaCharacteristicTag {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            TRANSCRIBES_SPOKEN_DIALOG => Ok(Self::TranscribesSpokenDialog),
            DESCRIBES_MUSIC_AND_SOUND => Ok(Self::DescribesMusicAndSound),
            EASY_TO_READ => Ok(Self::EasyToRead),
            DESCRIBES_VIDEO => Ok(Self::DescribesVideo),
            MACHINE_GENERATED => Ok(Self::MachineGenerated),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for MediaCharacteristicTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for MediaCharacteristicTag {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::TranscribesSpokenDialog => Cow::Borrowed(TRANSCRIBES_SPOKEN_DIALOG),
            Self::DescribesMusicAndSound => Cow::Borrowed(DESCRIBES_MUSIC_AND_SOUND),
            Self::EasyToRead => Cow::Borrowed(EASY_TO_READ),
            Self::DescribesVideo => Cow::Borrowed(DESCRIBES_VIDEO),
            Self::MachineGenerated => Cow::Borrowed(MACHINE_GENERATED),
        }
    }
}
impl From<MediaCharacteristicTag> for Cow<'_, str> {
    fn from(value: MediaCharacteristicTag) -> Self {
        value.as_cow()
    }
}
impl From<MediaCharacteristicTag> for EnumeratedString<'_, MediaCharacteristicTag> {
    fn from(value: MediaCharacteristicTag) -> Self {
        Self::Known(value)
    }
}
const TRANSCRIBES_SPOKEN_DIALOG: &str = "public.accessibility.transcribes-spoken-dialog";
const DESCRIBES_MUSIC_AND_SOUND: &str = "public.accessibility.describes-music-and-sound";
const EASY_TO_READ: &str = "public.easy-to-read";
const DESCRIBES_VIDEO: &str = "public.accessibility.describes-video";
const MACHINE_GENERATED: &str = "public.machine-generated";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelSpecialUsageIdentifier {
    /// The audio is binaural (either recorded or synthesized). It SHOULD NOT be dynamically
    /// spatialized. It is best suited for delivery to headphones.
    Binaural,
    /// The audio is pre-processed content that SHOULD NOT be dynamically spatialized. It is
    /// suitable to deliver to either headphones or speakers.
    Immersive,
    /// The audio is a downmix derivative of some other audio. If desired, the downmix may be used
    /// as a substitute for alternative Renditions in the same group with compatible attributes and
    /// a greater channel count. It MAY be dynamically spatialized.
    Downmix,
    /// The audio is prepared for routing to a specific speaker location. The associated value
    /// indicates count of channels prepared for specific routing. An example Display output is
    /// "BED-4".
    Bed(u32),
    /// The audio represents degrees of freedom. The associated value indicates a numerical value
    /// associated with degrees of freedom. Valid values for this special usage identifier are 3 or
    /// 6, indicating display outputs of "DOF-3" or "DOF-6".
    DegreesOfFreedom(u32),
}
impl<'a> TryFrom<&'a str> for ChannelSpecialUsageIdentifier {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            BINAURAL => Ok(Self::Binaural),
            IMMERSIVE => Ok(Self::Immersive),
            DOWNMIX => Ok(Self::Downmix),
            s if s.starts_with(BED) && s.bytes().nth(3) == Some(b'-') => {
                let count = s[4..]
                    .parse::<u32>()
                    .map_err(|_| UnrecognizedEnumerationError::new(s))?;
                Ok(Self::Bed(count))
            }
            s if s.starts_with(DOF) && s.bytes().nth(3) == Some(b'-') => {
                let count = s[4..]
                    .parse::<u32>()
                    .map_err(|_| UnrecognizedEnumerationError::new(s))?;
                Ok(Self::DegreesOfFreedom(count))
            }
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for ChannelSpecialUsageIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for ChannelSpecialUsageIdentifier {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Binaural => Cow::Borrowed(BINAURAL),
            Self::Immersive => Cow::Borrowed(IMMERSIVE),
            Self::Downmix => Cow::Borrowed(DOWNMIX),
            Self::Bed(n) => Cow::Owned(format!("{BED}-{n}")),
            Self::DegreesOfFreedom(n) => Cow::Owned(format!("{DOF}-{n}")),
        }
    }
}
impl From<ChannelSpecialUsageIdentifier> for Cow<'_, str> {
    fn from(value: ChannelSpecialUsageIdentifier) -> Self {
        value.as_cow()
    }
}
impl From<ChannelSpecialUsageIdentifier> for EnumeratedString<'_, ChannelSpecialUsageIdentifier> {
    fn from(value: ChannelSpecialUsageIdentifier) -> Self {
        Self::Known(value)
    }
}
const BINAURAL: &str = "BINAURAL";
const IMMERSIVE: &str = "IMMERSIVE";
const DOWNMIX: &str = "DOWNMIX";
const BED: &str = "BED";
const DOF: &str = "DOF";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioCodingIdentifier {
    /// A coding technique that allows up to 15 full range channels or objects, plus LFE channel, to
    /// be carried within a Dolby Digital Plus bitstream in a backward-compatible manner.
    JointObjectCoding,
    /// Signals the order of ambisonics (for example, an associated value of `3` indicates third
    /// order ambisonics).
    OrderOfAmbisonics(u32),
}
impl<'a> TryFrom<&'a str> for AudioCodingIdentifier {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            JOC => Ok(Self::JointObjectCoding),
            s if s.ends_with(OA) => {
                let len = s.len();
                let count = s[..(len - 2)]
                    .parse::<u32>()
                    .map_err(|_| UnrecognizedEnumerationError::new(s))?;
                Ok(Self::OrderOfAmbisonics(count))
            }
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for AudioCodingIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for AudioCodingIdentifier {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            AudioCodingIdentifier::JointObjectCoding => Cow::Borrowed(JOC),
            AudioCodingIdentifier::OrderOfAmbisonics(n) => Cow::Owned(format!("{n}{OA}")),
        }
    }
}
impl From<AudioCodingIdentifier> for Cow<'_, str> {
    fn from(value: AudioCodingIdentifier) -> Self {
        value.as_cow()
    }
}
impl From<AudioCodingIdentifier> for EnumeratedString<'_, AudioCodingIdentifier> {
    fn from(value: AudioCodingIdentifier) -> Self {
        Self::Known(value)
    }
}
const JOC: &str = "JOC";
const OA: &str = "OA";

#[derive(Debug, Clone, PartialEq)]
pub struct ValidChannels<'a> {
    count: u32,
    inner: Cow<'a, str>,
}
impl<'a> ValidChannels<'a> {
    pub fn new(
        count: u32,
        spatial_audio: impl Into<EnumeratedStringList<'a, AudioCodingIdentifier>>,
        special_usage: impl Into<EnumeratedStringList<'a, ChannelSpecialUsageIdentifier>>,
    ) -> Self {
        let spatial_audio = spatial_audio.into();
        let special_usage = special_usage.into();
        let special_usage_empty = special_usage.is_empty();
        let spatial_audio_empty = spatial_audio.is_empty();
        let inner = match (spatial_audio_empty, special_usage_empty) {
            (true, true) => Cow::Owned(format!("{count}")),
            (true, false) => Cow::Owned(format!("{count}/-/{special_usage}")),
            (false, true) => Cow::Owned(format!("{count}/{spatial_audio}")),
            (false, false) => Cow::Owned(format!("{count}/{spatial_audio}/{special_usage}")),
        };
        Self { count, inner }
    }
}
impl ValidChannels<'_> {
    /// Incicates a count of audio, incicating the maximum number of independent, simultaneous audio
    /// channels present in any Media Segment in the Rendition. For example, an AC-3 5.1 Rendition
    /// would have a value of `6`.
    pub fn count(&self) -> u32 {
        self.count
    }
    /// Identifies the presence of spatial audio of some kind, for example, object-based audio, in
    /// the Rendition. This is described as a list of audio coding identifiers (which can be codec
    /// specific).
    pub fn spatial_audio(&self) -> EnumeratedStringList<AudioCodingIdentifier> {
        let mut split = self.inner.splitn(3, '/');
        split.next();
        let Some(aci_str) = split.next().map(str::trim) else {
            return EnumeratedStringList::from("");
        };
        if aci_str == "-" {
            return EnumeratedStringList::from("");
        }
        EnumeratedStringList::from(aci_str)
    }
    /// Provides supplementary indications of special channel usage that are necessary for informed
    /// selection and processing.
    pub fn special_usage(&self) -> EnumeratedStringList<ChannelSpecialUsageIdentifier> {
        let mut split = self.inner.splitn(4, '/');
        split.next();
        split.next();
        let Some(sui_str) = split.next().map(str::trim) else {
            return EnumeratedStringList::from("");
        };
        if sui_str == "-" {
            return EnumeratedStringList::from("");
        }
        EnumeratedStringList::from(sui_str)
    }
    /// At the time of writing the HLS specification only defined 3 parameters (described here via
    /// [`Self::count`], [`Self::spatial_audio`], and [`Self::special_usage`]). In case more
    /// parameters are added later, this method will expose those as a split on `'/'`.
    pub fn unknown_parameters(&self) -> Split<char> {
        let mut split = self.inner.split('/');
        split.next();
        split.next();
        split.next();
        split
    }
}
impl<'a> TryFrom<&'a str> for ValidChannels<'a> {
    type Error = UnrecognizedEnumerationError<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.splitn(2, '/').next().map(str::parse::<u32>) {
            Some(Ok(count)) => Ok(Self {
                count,
                inner: Cow::Borrowed(value),
            }),
            _ => Err(UnrecognizedEnumerationError::new(value)),
        }
    }
}
impl Display for ValidChannels<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
impl AsRef<str> for ValidChannels<'_> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl<'a> From<ValidChannels<'a>> for Cow<'a, str> {
    fn from(value: ValidChannels<'a>) -> Self {
        value.inner
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Channels<'a> {
    Valid(ValidChannels<'a>),
    Invalid(&'a str),
}
impl<'a> Channels<'a> {
    pub fn valid(&self) -> Option<&ValidChannels<'a>> {
        match self {
            Channels::Valid(valid_channels) => Some(valid_channels),
            Channels::Invalid(_) => None,
        }
    }
}
impl AsRef<str> for Channels<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Channels::Valid(valid_channels) => valid_channels.as_ref(),
            Channels::Invalid(s) => s,
        }
    }
}
impl<'a> From<&'a str> for Channels<'a> {
    fn from(value: &'a str) -> Self {
        match ValidChannels::try_from(value) {
            Ok(valid) => Self::Valid(valid),
            Err(e) => Self::Invalid(e.value),
        }
    }
}
impl<'a> From<Channels<'a>> for Cow<'a, str> {
    fn from(value: Channels<'a>) -> Self {
        match value {
            Channels::Valid(valid_channels) => Cow::from(valid_channels),
            Channels::Invalid(s) => Cow::Borrowed(s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MediaAttributeList<'a> {
    pub media_type: Cow<'a, str>,
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
    media_type: Cow<'a, str>,
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
        media_type: impl Into<Cow<'a, str>>,
        name: impl Into<Cow<'a, str>>,
        group_id: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            media_type: media_type.into(),
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
        media_type: impl Into<Cow<'a, str>>,
        name: impl Into<Cow<'a, str>>,
        group_id: impl Into<Cow<'a, str>>,
    ) -> MediaBuilder<'a> {
        MediaBuilder::new(media_type, name, group_id)
    }

    pub fn media_type(&self) -> EnumeratedString<Type> {
        EnumeratedString::from(self.media_type.as_ref())
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
    pub fn instream_id(&self) -> Option<EnumeratedString<InstreamId>> {
        if let Some(instream_id) = &self.instream_id {
            Some(EnumeratedString::from(instream_id.as_ref()))
        } else {
            match self.attribute_list.get(INSTREAM_ID) {
                Some(ParsedAttributeValue::QuotedString(s)) => Some(EnumeratedString::from(*s)),
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
    pub fn characteristics(&self) -> Option<EnumeratedStringList<MediaCharacteristicTag>> {
        if let Some(characteristics) = &self.characteristics {
            Some(EnumeratedStringList::from(characteristics.as_ref()))
        } else {
            match self.attribute_list.get(CHARACTERISTICS) {
                Some(ParsedAttributeValue::QuotedString(c)) => Some(EnumeratedStringList::from(*c)),
                _ => None,
            }
        }
    }
    pub fn channels(&self) -> Option<Channels> {
        if let Some(channels) = &self.channels {
            Some(Channels::from(channels.as_ref()))
        } else {
            match self.attribute_list.get(CHANNELS) {
                Some(ParsedAttributeValue::QuotedString(c)) => Some(Channels::from(*c)),
                _ => None,
            }
        }
    }

    pub fn set_media_type(&mut self, media_type: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(TYPE);
        self.media_type = media_type.into();
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

into_inner_tag!(Media);

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
    use super::*;
    use crate::tag::{hls::test_macro::mutation_tests, known::IntoInnerTag};
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
            Media::builder(Type::ClosedCaptions, "English", "cc")
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
            Media::builder(Type::Audio, "English", "stereo")
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
        Media::builder(Type::Audio, "English", "stereo")
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
        (
            instream_id,
            @Option EnumeratedString::<InstreamId>::Unknown("ID2"),
            @Attr="INSTREAM-ID=\"ID2\""
        ),
        (bit_depth, @Option 10, @Attr="BIT-DEPTH=10"),
        (sample_rate, @Option 42, @Attr="SAMPLE-RATE=42"),
        (
            characteristics,
            @Option EnumeratedStringList::<MediaCharacteristicTag>::from("example"),
            @Attr="CHARACTERISTICS=\"example\""
        ),
        (channels, @Option Channels::Valid(ValidChannels::new(6, "", "")), @Attr="CHANNELS=\"6\"")
    );

    #[test]
    fn instream_id_cea_708_values_parse_and_display_as_expected() {
        let s = "SERVICE42";
        assert_eq!(Ok(InstreamId::Cea708(42)), InstreamId::try_from(s));
        assert_eq!(s, format!("{}", InstreamId::Cea708(42)));
    }

    #[test]
    fn channel_special_usage_identifier_bed_parses_and_displays_as_expected() {
        let s = "BED-4";
        assert_eq!(
            Ok(ChannelSpecialUsageIdentifier::Bed(4)),
            ChannelSpecialUsageIdentifier::try_from(s)
        );
        assert_eq!(s, format!("{}", ChannelSpecialUsageIdentifier::Bed(4)));
    }

    #[test]
    fn channel_special_usage_identifier_dof_parses_and_displays_as_expected() {
        let s = "DOF-3";
        assert_eq!(
            Ok(ChannelSpecialUsageIdentifier::DegreesOfFreedom(3)),
            ChannelSpecialUsageIdentifier::try_from(s)
        );
        assert_eq!(
            s,
            format!("{}", ChannelSpecialUsageIdentifier::DegreesOfFreedom(3))
        );
    }

    #[test]
    fn audio_coding_identifier_order_of_ambisonics_parses_and_displays_as_expected() {
        let s = "3OA";
        assert_eq!(
            Ok(AudioCodingIdentifier::OrderOfAmbisonics(3)),
            AudioCodingIdentifier::try_from(s)
        );
        assert_eq!(
            s,
            format!("{}", AudioCodingIdentifier::OrderOfAmbisonics(3))
        );
    }

    #[test]
    fn new_channels_displays_as_expected() {
        let test_instances = channels_test_instances();
        assert_eq!("2", format!("{}", test_instances[0]));
        assert_eq!("2", test_instances[0].as_ref());

        assert_eq!("12/JOC", format!("{}", test_instances[1]));
        assert_eq!("12/JOC", test_instances[1].as_ref());

        assert_eq!("16/3OA/IMMERSIVE,DOF-6", format!("{}", test_instances[2]));
        assert_eq!("16/3OA/IMMERSIVE,DOF-6", test_instances[2].as_ref());
    }

    #[test]
    fn channels_count_works_as_expected() {
        let test_instances = channels_test_instances();
        assert_eq!(2, test_instances[0].count());
        assert_eq!(12, test_instances[1].count());
        assert_eq!(16, test_instances[2].count());
    }

    #[test]
    fn channels_spatial_works_as_expected() {
        let test_instances = channels_test_instances();
        assert!(
            test_instances[0].spatial_audio().is_empty(),
            "spatial should be empty"
        );
        assert!(
            test_instances[1]
                .spatial_audio()
                .contains(AudioCodingIdentifier::JointObjectCoding),
            "spatial should contain JOC"
        );
        assert!(
            test_instances[2]
                .spatial_audio()
                .contains(AudioCodingIdentifier::OrderOfAmbisonics(3)),
            "spatial should contain 3OA"
        );
    }

    #[test]
    fn channels_special_works_as_expected() {
        let test_instances = channels_test_instances();
        assert!(
            test_instances[0].special_usage().is_empty(),
            "special should be empty"
        );
        assert!(
            test_instances[1].special_usage().is_empty(),
            "special should be empty"
        );
        assert!(
            test_instances[2]
                .special_usage()
                .contains(ChannelSpecialUsageIdentifier::Immersive),
            "spatial should contain 3OA"
        );
        assert!(
            test_instances[2]
                .special_usage()
                .contains(ChannelSpecialUsageIdentifier::DegreesOfFreedom(6))
        )
    }

    fn channels_test_instances<'a>() -> [ValidChannels<'a>; 3] {
        [
            ValidChannels::new(2, "", ""),
            ValidChannels::new(12, [AudioCodingIdentifier::JointObjectCoding], ""),
            ValidChannels::new(
                16,
                [AudioCodingIdentifier::OrderOfAmbisonics(3)],
                [
                    ChannelSpecialUsageIdentifier::Immersive,
                    ChannelSpecialUsageIdentifier::DegreesOfFreedom(6),
                ],
            ),
        ]
    }
}
