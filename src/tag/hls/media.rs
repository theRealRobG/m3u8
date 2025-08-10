use crate::{
    error::{ParseTagValueError, UnrecognizedEnumerationError, ValidationError},
    tag::{
        hls::{EnumeratedString, EnumeratedStringList, into_inner_tag},
        unknown,
        value::{AttributeValue, UnquotedAttributeValue},
    },
    utils::AsStaticCow,
};
use std::{borrow::Cow, collections::HashMap, fmt::Display, marker::PhantomData, str::Split};

/// Corresponds to the `#EXT-X-MEDIA:TYPE` attribute.
///
/// See [`Media`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
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
impl<'a> TryFrom<&'a str> for MediaType {
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
impl Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_cow())
    }
}
impl AsStaticCow for MediaType {
    fn as_cow(&self) -> Cow<'static, str> {
        match self {
            MediaType::Audio => Cow::Borrowed(AUDIO),
            MediaType::Video => Cow::Borrowed(VIDEO),
            MediaType::Subtitles => Cow::Borrowed(SUBTITLES),
            MediaType::ClosedCaptions => Cow::Borrowed(CLOSED_CAPTIONS),
        }
    }
}
impl From<MediaType> for Cow<'_, str> {
    fn from(value: MediaType) -> Self {
        value.as_cow()
    }
}
impl From<MediaType> for EnumeratedString<'_, MediaType> {
    fn from(value: MediaType) -> Self {
        Self::Known(value)
    }
}
const AUDIO: &str = "AUDIO";
const VIDEO: &str = "VIDEO";
const SUBTITLES: &str = "SUBTITLES";
const CLOSED_CAPTIONS: &str = "CLOSED-CAPTIONS";

/// Corresponds to the `#EXT-X-MEDIA:INSTREAM-ID` attribute when it is describing a Line 21 Data
/// Services (CEA608) channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cea608InstreamId {
    /// CC1 as per CEA-608 specification.
    Cc1,
    /// CC2 as per CEA-608 specification.
    Cc2,
    /// CC3 as per CEA-608 specification.
    Cc3,
    /// CC4 as per CEA-608 specification.
    Cc4,
}
/// Corresponds to the `#EXT-X-MEDIA:INSTREAM-ID` attribute.
///
/// Note, as of draft 18, it is valid to use `INSTREAM-ID` to indicate other types of media that are
/// contained within the Media Segments. Specifically:
/// > For all other [than CLOSED-CAPTIONS] types, the mechanism for carrying a Rendition and mapping
/// > from the INSTREAM-ID to the content within the segment is defined by the segment, sample or
/// > bitstream format. The value is a string containing characters from the set [A-Z], [a-z],
/// > [0-9], and '.'. If the value does not match any alternative content, the client SHOULD ignore
/// > this and treat it as if no INSTREAM-ID was provided in the EXT-X-MEDIA tag.
///
/// Given that `InstreamId` is wrapped within [`EnumeratedString`] when exposed on [`Media`], and
/// `EnumeratedString` already has an [`EnumeratedString::Unknown`] case, we will not add support
/// for "unknown" `INSTREAM-ID` formats here and leave it to be exposed via the `Unknown` case of
/// `EnumeratedString` instead.
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

/// Corresponds to some of the available values for the `#EXT-X-MEDIA:CHARACTERISTICS` attribute.
///
/// These are just the values that are called out as recognized in the HLS specification. There may
/// be many other characteristics; however, given that this is wrapped in [`EnumeratedString`] when
/// exposed on [`Media`], and that already has an [`EnumeratedString::Unknown`] case, we don't need
/// to support the "unknown" case here.
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

/// Corresponds to the "supplementary indications of special channel usage" parameter in the
/// `#EXT-X-MEDIA:CHANNELS` attribute.
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
            s if s.starts_with(BED) && s.as_bytes().get(3) == Some(&b'-') => {
                let count = s[4..]
                    .parse::<u32>()
                    .map_err(|_| UnrecognizedEnumerationError::new(s))?;
                Ok(Self::Bed(count))
            }
            s if s.starts_with(DOF) && s.as_bytes().get(3) == Some(&b'-') => {
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

/// Corresponds to the "presence of spatial audio of some kind" parameter in the
/// `#EXT-X-MEDIA:CHANNELS` attribute.
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

/// Describes a `CHANNELS` attribute value that has at least a valid `count`.
///
/// The format described in HLS is a slash separated list of parameters. At the time of writing
/// there were 3 defined parameters:
/// * The count of audio channels
/// * An indicator of spatial audio of some kind
/// * Supplementary indication of special channel usage
#[derive(Debug, Clone, PartialEq)]
pub struct ValidChannels<'a> {
    count: u32,
    inner: Cow<'a, str>,
}
impl<'a> ValidChannels<'a> {
    /// Construct a new `ValidChannels`.
    ///
    /// Note that `AudioCodingIdentifier` and `ChannelSpecialUsageIdentifier` can be used directly
    /// here. For example:
    /// ```
    /// # use m3u8::tag::hls::{EnumeratedStringList, ValidChannels, AudioCodingIdentifier,
    /// # ChannelSpecialUsageIdentifier};
    /// let channels = ValidChannels::new(
    ///     16,
    ///     EnumeratedStringList::from([AudioCodingIdentifier::JointObjectCoding]),
    ///     EnumeratedStringList::from([ChannelSpecialUsageIdentifier::Binaural])
    /// );
    /// ```
    /// Since `&str` implements `Into<EnumeratedStringList>` we can also use string slice directly,
    /// but care should be taken to follow the correct format:
    /// ```
    /// # use m3u8::tag::hls::ValidChannels;
    /// let channels = ValidChannels::new(16, "JOC", "BINAURAL");
    /// ```
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
        match value.split('/').next().map(str::parse::<u32>) {
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
/// Corresponds to the `#EXT-X-MEDIA:CHANNELS` attribute value.
#[derive(Debug, Clone, PartialEq)]
pub enum Channels<'a> {
    /// The value contained at least a valid channels count.
    Valid(ValidChannels<'a>),
    /// The value was malformed and unrecognized. The original data is still provided.
    Invalid(&'a str),
}
/// Used to provide a convenience accessor on `Option<Channels>` for the `ValidChannels` case. For
/// example:
/// ```
/// # use m3u8::tag::hls::{Channels, ValidChannels };
/// use m3u8::tag::hls::GetValid;
///
/// let some_channels = Some(Channels::Valid(ValidChannels::new(6, "", "")));
/// let some_valid_channels = some_channels.valid();
/// assert_eq!(6, some_valid_channels.expect("should be defined").count());
/// ```
pub trait GetValid<'a> {
    /// Convenience accessor for the valid case.
    fn valid(self) -> Option<ValidChannels<'a>>;
}
impl<'a> GetValid<'a> for Option<Channels<'a>> {
    fn valid(self) -> Option<ValidChannels<'a>> {
        match self {
            Some(Channels::Valid(channels)) => Some(channels),
            Some(Channels::Invalid(_)) | None => None,
        }
    }
}
impl<'a> Channels<'a> {
    /// Convenience accessor for the valid case.
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

/// The attribute list for the tag (`#EXT-X-MEDIA:<attribute-list>`).
///
/// See [`Media`] for a link to the HLS documentation for this attribute.
#[derive(Debug, Clone)]
struct MediaAttributeList<'a> {
    /// Corresponds to the `TYPE` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    media_type: Cow<'a, str>,
    /// Corresponds to the `NAME` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    name: Cow<'a, str>,
    /// Corresponds to the `GROUP-ID` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    group_id: Cow<'a, str>,
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    uri: Option<Cow<'a, str>>,
    /// Corresponds to the `LANGUAGE` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    language: Option<Cow<'a, str>>,
    /// Corresponds to the `ASSOC-LANGUAGE` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    assoc_language: Option<Cow<'a, str>>,
    /// Corresponds to the `STABLE-RENDITION-ID` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    stable_rendition_id: Option<Cow<'a, str>>,
    /// Corresponds to the `DEFAULT` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    default: bool,
    /// Corresponds to the `AUTOSELECT` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    autoselect: bool,
    /// Corresponds to the `FORCED` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    forced: bool,
    /// Corresponds to the `INSTREAM-ID` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    instream_id: Option<Cow<'a, str>>,
    /// Corresponds to the `BIT-DEPTH` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    bit_depth: Option<u64>,
    /// Corresponds to the `SAMPLE-RATE` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    sample_rate: Option<u64>,
    /// Corresponds to the `CHARACTERISTICS` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    characteristics: Option<Cow<'a, str>>,
    /// Corresponds to the `CHANNELS` attribute.
    ///
    /// See [`Media`] for a link to the HLS documentation for this attribute.
    channels: Option<Cow<'a, str>>,
}

/// Placeholder struct for [`MediaBuilder`] indicating that `media_type` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct MediaTypeNeedsToBeSet;
/// Placeholder struct for [`MediaBuilder`] indicating that `name` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct MediaNameNeedsToBeSet;
/// Placeholder struct for [`MediaBuilder`] indicating that `group_id` needs to be set.
#[derive(Debug, Clone, Copy)]
pub struct MediaGroupIdNeedsToBeSet;
/// Placeholder struct for [`MediaBuilder`] indicating that `media_type` has been set.
#[derive(Debug, Clone, Copy)]
pub struct MediaTypeHasBeenSet;
/// Placeholder struct for [`MediaBuilder`] indicating that `name` has been set.
#[derive(Debug, Clone, Copy)]
pub struct MediaNameHasBeenSet;
/// Placeholder struct for [`MediaBuilder`] indicating that `group_id` has been set.
#[derive(Debug, Clone, Copy)]
pub struct MediaGroupIdHasBeenSet;

/// A builder for convenience in constructing a [`Media`].
///
/// Builder pattern inspired by [Sguaba]
///
/// [Sguaba]: https://github.com/helsing-ai/sguaba/blob/8dadfe066197551b0601e01676f8d13ef1168785/src/directions.rs#L271-L291
#[derive(Debug, Clone)]
pub struct MediaBuilder<'a, TypeStatus, NameStatus, GroupIdStatus> {
    attribute_list: MediaAttributeList<'a>,
    type_status: PhantomData<TypeStatus>,
    name_status: PhantomData<NameStatus>,
    group_id_status: PhantomData<GroupIdStatus>,
}
impl<'a> MediaBuilder<'a, MediaTypeNeedsToBeSet, MediaNameNeedsToBeSet, MediaGroupIdNeedsToBeSet> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            attribute_list: MediaAttributeList {
                media_type: Cow::Borrowed(""),
                name: Cow::Borrowed(""),
                group_id: Cow::Borrowed(""),
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
            },
            type_status: PhantomData,
            name_status: PhantomData,
            group_id_status: PhantomData,
        }
    }
}
impl<'a> MediaBuilder<'a, MediaTypeHasBeenSet, MediaNameHasBeenSet, MediaGroupIdHasBeenSet> {
    /// Finish building and construct the `Media`.
    pub fn finish(self) -> Media<'a> {
        Media::new(self.attribute_list)
    }
}
impl<'a, TypeStatus, NameStatus, GroupIdStatus>
    MediaBuilder<'a, TypeStatus, NameStatus, GroupIdStatus>
{
    /// Add the provided `media_type` to the attributes built into `Media`.
    pub fn with_media_type(
        mut self,
        media_type: impl Into<Cow<'a, str>>,
    ) -> MediaBuilder<'a, MediaTypeHasBeenSet, NameStatus, GroupIdStatus> {
        self.attribute_list.media_type = media_type.into();
        MediaBuilder {
            attribute_list: self.attribute_list,
            type_status: PhantomData,
            name_status: PhantomData,
            group_id_status: PhantomData,
        }
    }
    /// Add the provided `name` to the attributes built into `Media`.
    pub fn with_name(
        mut self,
        name: impl Into<Cow<'a, str>>,
    ) -> MediaBuilder<'a, TypeStatus, MediaNameHasBeenSet, GroupIdStatus> {
        self.attribute_list.name = name.into();
        MediaBuilder {
            attribute_list: self.attribute_list,
            type_status: PhantomData,
            name_status: PhantomData,
            group_id_status: PhantomData,
        }
    }
    /// Add the provided `group_id` to the attributes built into `Media`.
    pub fn with_group_id(
        mut self,
        group_id: impl Into<Cow<'a, str>>,
    ) -> MediaBuilder<'a, TypeStatus, NameStatus, MediaGroupIdHasBeenSet> {
        self.attribute_list.group_id = group_id.into();
        MediaBuilder {
            attribute_list: self.attribute_list,
            type_status: PhantomData,
            name_status: PhantomData,
            group_id_status: PhantomData,
        }
    }
    /// Add the provided `uri` to the attributes built into `Media`.
    pub fn with_uri(mut self, uri: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.uri = Some(uri.into());
        self
    }
    /// Add the provided `language` to the attributes built into `Media`.
    pub fn with_language(mut self, language: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.language = Some(language.into());
        self
    }
    /// Add the provided `assoc_language` to the attributes built into `Media`.
    pub fn with_assoc_language(mut self, assoc_language: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.assoc_language = Some(assoc_language.into());
        self
    }
    /// Add the provided `stable_rendition_id` to the attributes built into `Media`.
    pub fn with_stable_rendition_id(
        mut self,
        stable_rendition_id: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.attribute_list.stable_rendition_id = Some(stable_rendition_id.into());
        self
    }
    /// Add the provided `default` to the attributes built into `Media`.
    pub fn with_default(mut self) -> Self {
        self.attribute_list.default = true;
        self
    }
    /// Add the provided `autoselect` to the attributes built into `Media`.
    pub fn with_autoselect(mut self) -> Self {
        self.attribute_list.autoselect = true;
        self
    }
    /// Add the provided `forced` to the attributes built into `Media`.
    pub fn with_forced(mut self) -> Self {
        self.attribute_list.forced = true;
        self
    }
    /// Add the provided `instream_id` to the attributes built into `Media`.
    pub fn with_instream_id(mut self, instream_id: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.instream_id = Some(instream_id.into());
        self
    }
    /// Add the provided `bit_depth` to the attributes built into `Media`.
    pub fn with_bit_depth(mut self, bit_depth: u64) -> Self {
        self.attribute_list.bit_depth = Some(bit_depth);
        self
    }
    /// Add the provided `sample_rate` to the attributes built into `Media`.
    pub fn with_sample_rate(mut self, sample_rate: u64) -> Self {
        self.attribute_list.sample_rate = Some(sample_rate);
        self
    }
    /// Add the provided `characteristics` to the attributes built into `Media`.
    pub fn with_characteristics(mut self, characteristics: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.characteristics = Some(characteristics.into());
        self
    }
    /// Add the provided `channels` to the attributes built into `Media`.
    ///
    /// Note that [`ValidChannels`] implements `Into<Cow<str>>` and therefore can be
    /// used directly here. For example:
    /// ```
    /// # use m3u8::tag::hls::{
    /// # MediaBuilder, ValidChannels, MediaType, EnumeratedStringList, AudioCodingIdentifier,
    /// # ChannelSpecialUsageIdentifier
    /// # };
    /// let builder = MediaBuilder::new()
    ///     .with_media_type(MediaType::Audio)
    ///     .with_name("ENGLISH")
    ///     .with_group_id("SPECIAL-GROUP")
    ///     .with_channels(ValidChannels::new(
    ///         16,
    ///         EnumeratedStringList::from([AudioCodingIdentifier::JointObjectCoding]),
    ///         EnumeratedStringList::from([ChannelSpecialUsageIdentifier::Binaural]),
    ///     ));
    /// ```
    /// Alternatively, a string slice can be used, but care should be taken to follow the correct
    /// syntax defined for `CHANNELS`.
    /// ```
    /// # use m3u8::tag::hls::{ MediaBuilder, MediaType };
    /// let builder = MediaBuilder::new()
    ///     .with_media_type(MediaType::Audio)
    ///     .with_name("ENGLISH")
    ///     .with_group_id("SPECIAL-GROUP")
    ///     .with_channels("16/JOC/BINAURAL");
    /// ```
    pub fn with_channels(mut self, channels: impl Into<Cow<'a, str>>) -> Self {
        self.attribute_list.channels = Some(channels.into());
        self
    }
}

/// Corresponds to the `#EXT-X-MEDIA` tag.
///
/// <https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.6.1>
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
    attribute_list: HashMap<&'a str, AttributeValue<'a>>, // Original attribute list
    output_line: Cow<'a, [u8]>,                           // Used with Writer
    output_line_is_dirty: bool,                           // If should recalculate output_line
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

impl<'a> TryFrom<unknown::Tag<'a>> for Media<'a> {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_attribute_list()?;
        let Some(media_type) = attribute_list
            .get(TYPE)
            .and_then(AttributeValue::unquoted)
            .and_then(|v| v.try_as_utf_8().ok())
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(TYPE));
        };
        let Some(group_id) = attribute_list
            .get(GROUP_ID)
            .and_then(AttributeValue::quoted)
        else {
            return Err(super::ValidationError::MissingRequiredAttribute(GROUP_ID));
        };
        let Some(name) = attribute_list.get(NAME).and_then(AttributeValue::quoted) else {
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
    /// Constructs a new `Media` tag.
    fn new(attribute_list: MediaAttributeList<'a>) -> Self {
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

    /// Starts a builder for producing `Self`.
    ///
    /// For example, we could construct a `Media` as such:
    /// ```
    /// # use m3u8::tag::hls::{Media, MediaType, EnumeratedStringList, MediaCharacteristicTag,
    /// # ValidChannels, AudioCodingIdentifier, ChannelSpecialUsageIdentifier};
    /// let media = Media::builder()
    ///     .with_media_type(MediaType::Audio)
    ///     .with_name("ENGLISH")
    ///     .with_group_id("SPECIAL-GROUP")
    ///     .with_uri("special-audio.m3u8")
    ///     .with_language("en")
    ///     .with_default()
    ///     .with_autoselect()
    ///     .with_characteristics(EnumeratedStringList::from([
    ///         MediaCharacteristicTag::DescribesVideo,
    ///         MediaCharacteristicTag::MachineGenerated,
    ///     ]))
    ///     .with_channels(ValidChannels::new(
    ///         16,
    ///         EnumeratedStringList::from([AudioCodingIdentifier::JointObjectCoding]),
    ///         EnumeratedStringList::from([ChannelSpecialUsageIdentifier::Binaural]),
    ///     ))
    ///     .finish();
    /// ```
    /// Note that the `finish` method is only callable if the builder has set `media_type`, `name`,
    /// AND `group_id`. Each of the following fail to compile:
    /// ```compile_fail
    /// # use m3u8::tag::hls::Media;
    /// let media = Media::builder().finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Media;
    /// let media = Media::builder().with_media_type(MediaType::Audio).finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Media;
    /// let media = Media::builder().with_name("test").finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Media;
    /// let media = Media::builder().with_group_id("test").finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Media;
    /// let media = Media::builder()
    ///     .with_media_type(MediaType::Audio)
    ///     .with_name("test")
    ///     .finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Media;
    /// let media = Media::builder()
    ///     .with_media_type(MediaType::Audio)
    ///     .with_group_id("test")
    ///     .finish();
    /// ```
    /// ```compile_fail
    /// # use m3u8::tag::hls::Media;
    /// let media = Media::builder()
    ///     .name("test")
    ///     .with_group_id("test")
    ///     .finish();
    /// ```
    pub fn builder()
    -> MediaBuilder<'a, MediaTypeNeedsToBeSet, MediaNameNeedsToBeSet, MediaGroupIdNeedsToBeSet>
    {
        MediaBuilder::new()
    }

    /// Corresponds to the `TYPE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn media_type(&self) -> EnumeratedString<MediaType> {
        EnumeratedString::from(self.media_type.as_ref())
    }
    /// Corresponds to the `NAME` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Corresponds to the `GROUP-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn group_id(&self) -> &str {
        &self.group_id
    }
    /// Corresponds to the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn uri(&self) -> Option<&str> {
        if let Some(uri) = &self.uri {
            Some(uri)
        } else {
            self.attribute_list
                .get(URI)
                .and_then(AttributeValue::quoted)
        }
    }
    /// Corresponds to the `LANGUAGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn language(&self) -> Option<&str> {
        if let Some(language) = &self.language {
            Some(language)
        } else {
            self.attribute_list
                .get(LANGUAGE)
                .and_then(AttributeValue::quoted)
        }
    }
    /// Corresponds to the `ASSOC-LANGUAGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn assoc_language(&self) -> Option<&str> {
        if let Some(assoc_language) = &self.assoc_language {
            Some(assoc_language)
        } else {
            self.attribute_list
                .get(ASSOC_LANGUAGE)
                .and_then(AttributeValue::quoted)
        }
    }
    /// Corresponds to the `STABLE-RENDITION-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn stable_rendition_id(&self) -> Option<&str> {
        if let Some(stable_rendition_id) = &self.stable_rendition_id {
            Some(stable_rendition_id)
        } else {
            self.attribute_list
                .get(STABLE_RENDITION_ID)
                .and_then(AttributeValue::quoted)
        }
    }
    /// Corresponds to the `DEFAULT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn default(&self) -> bool {
        if let Some(default) = self.default {
            default
        } else {
            matches!(
                self.attribute_list.get(DEFAULT),
                Some(AttributeValue::Unquoted(UnquotedAttributeValue(b"YES")))
            )
        }
    }
    /// Corresponds to the `AUTOSELECT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn autoselect(&self) -> bool {
        if let Some(autoselect) = self.autoselect {
            autoselect
        } else {
            matches!(
                self.attribute_list.get(AUTOSELECT),
                Some(AttributeValue::Unquoted(UnquotedAttributeValue(b"YES")))
            )
        }
    }
    /// Corresponds to the `FORCED` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn forced(&self) -> bool {
        if let Some(forced) = self.forced {
            forced
        } else {
            matches!(
                self.attribute_list.get(FORCED),
                Some(AttributeValue::Unquoted(UnquotedAttributeValue(b"YES")))
            )
        }
    }
    /// Corresponds to the `INSTREAM-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Note that the convenience [`crate::tag::hls::GetKnown`] trait exists to make accessing the
    /// known case easier:
    /// ```
    /// # use m3u8::tag::hls::{Media, MediaType, InstreamId, Cea608InstreamId};
    /// use m3u8::tag::hls::GetKnown;
    ///
    /// let tag = Media::builder()
    ///     .with_media_type(MediaType::ClosedCaptions)
    ///     .with_name("name")
    ///     .with_group_id("id")
    ///     .with_instream_id(InstreamId::Cea608(Cea608InstreamId::Cc1))
    ///     .finish();
    /// assert_eq!(Some(InstreamId::Cea608(Cea608InstreamId::Cc1)), tag.instream_id().known());
    /// ```
    pub fn instream_id(&self) -> Option<EnumeratedString<InstreamId>> {
        if let Some(instream_id) = &self.instream_id {
            Some(EnumeratedString::from(instream_id.as_ref()))
        } else {
            self.attribute_list
                .get(INSTREAM_ID)
                .and_then(AttributeValue::quoted)
                .map(EnumeratedString::from)
        }
    }
    /// Corresponds to the `BIT-DEPTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn bit_depth(&self) -> Option<u64> {
        if let Some(bit_depth) = self.bit_depth {
            Some(bit_depth)
        } else {
            self.attribute_list
                .get(BIT_DEPTH)
                .and_then(AttributeValue::unquoted)
                .and_then(|v| v.try_as_decimal_integer().ok())
        }
    }
    /// Corresponds to the `SAMPLE-RATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn sample_rate(&self) -> Option<u64> {
        if let Some(sample_rate) = self.sample_rate {
            Some(sample_rate)
        } else {
            self.attribute_list
                .get(SAMPLE_RATE)
                .and_then(AttributeValue::unquoted)
                .and_then(|v| v.try_as_decimal_integer().ok())
        }
    }
    /// Corresponds to the `CHARACTERISTICS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn characteristics(&self) -> Option<EnumeratedStringList<MediaCharacteristicTag>> {
        if let Some(characteristics) = &self.characteristics {
            Some(EnumeratedStringList::from(characteristics.as_ref()))
        } else {
            self.attribute_list
                .get(CHARACTERISTICS)
                .and_then(AttributeValue::quoted)
                .map(EnumeratedStringList::from)
        }
    }
    /// Corresponds to the `CHANNELS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// The `Channels` enum provides a strongly typed wrapper around the string value of the
    /// `CHANNELS` attribute. Given that we need at least a valid `count` but internally we store
    /// the raw value of the attribute (as `Cow<str>` so we only validate UTF-8), the translation to
    /// a valid "channels" struct may fail. This is why we expose `Channels` as an enum with a
    /// [`Channels::Valid`] case and [`ValidChannels`] is really the strongly typed wrapper.
    ///
    /// [`ValidChannels`] abstracts the slash separated list and the syntax around it. We use
    /// [`EnumeratedStringList`] to provide a pseudo-set-like abstraction over each of the "spatial"
    /// and "special" parameters of the `CHANNELS` attribute. This does not allocate to the heap (as
    /// would be the case with a `Vec` or `HashSet`) so is relatively little cost over using the
    /// `&str` directly but provides convenience types and methods. For example:
    /// ```
    /// # use m3u8::{Reader, HlsLine, config::ParsingOptions, tag::{known, hls}};
    /// # use m3u8::tag::hls::{Media, ChannelSpecialUsageIdentifier};
    /// // NOTE: This trait needs to be in scope to use the convenience `valid` method that the
    /// // library defines on `Option<Channels>` to get to `Option<ValidChannels>`.
    /// use m3u8::tag::hls::GetValid;
    ///
    /// let tag = r#"#EXT-X-MEDIA:TYPE=AUDIO,NAME="a",GROUP-ID="a",CHANNELS="6/-/DOWNMIX,BED-4""#;
    /// let mut reader = Reader::from_str(tag, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Media(media))))) => {
    ///         let channels = media.channels().valid().expect("should be defined");
    ///         assert_eq!(6, channels.count());
    ///         assert!(channels.spatial_audio().is_empty());
    ///         assert_eq!(2, channels.special_usage().iter().count());
    ///         assert!(channels.special_usage().contains(ChannelSpecialUsageIdentifier::Downmix));
    ///         assert!(channels.special_usage().contains(ChannelSpecialUsageIdentifier::Bed(4)));
    ///         // At any stage we can escape-hatch to the inner `&str` representation:
    ///         assert_eq!("6/-/DOWNMIX,BED-4", channels.as_ref());
    ///     }
    ///     r => panic!("unexpected result {r:?}"),
    /// }
    /// ```
    pub fn channels(&self) -> Option<Channels> {
        if let Some(channels) = &self.channels {
            Some(Channels::from(channels.as_ref()))
        } else {
            self.attribute_list
                .get(CHANNELS)
                .and_then(AttributeValue::quoted)
                .map(Channels::from)
        }
    }

    /// Sets the `TYPE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_media_type(&mut self, media_type: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(TYPE);
        self.media_type = media_type.into();
        self.output_line_is_dirty = true;
    }
    /// Sets the `NAME` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_name(&mut self, name: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(NAME);
        self.name = name.into();
        self.output_line_is_dirty = true;
    }
    /// Sets the `GROUP-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_group_id(&mut self, group_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(GROUP_ID);
        self.group_id = group_id.into();
        self.output_line_is_dirty = true;
    }
    /// Sets the `URI` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_uri(&mut self, uri: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(URI);
        self.uri = Some(uri.into());
        self.output_line_is_dirty = true;
    }
    /// Unsets the `URI` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_uri(&mut self) {
        self.attribute_list.remove(URI);
        self.uri = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `LANGUAGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_language(&mut self, language: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(LANGUAGE);
        self.language = Some(language.into());
        self.output_line_is_dirty = true;
    }
    /// Unsets the `LANGUAGE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_language(&mut self) {
        self.attribute_list.remove(LANGUAGE);
        self.language = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `ASSOC-LANGUAGE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_assoc_language(&mut self, assoc_language: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(ASSOC_LANGUAGE);
        self.assoc_language = Some(assoc_language.into());
        self.output_line_is_dirty = true;
    }
    /// Unsets the `ASSOC-LANGUAGE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_assoc_language(&mut self) {
        self.attribute_list.remove(ASSOC_LANGUAGE);
        self.assoc_language = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `STABLE-RENDITION-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_stable_rendition_id(&mut self, stable_rendition_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(STABLE_RENDITION_ID);
        self.stable_rendition_id = Some(stable_rendition_id.into());
        self.output_line_is_dirty = true;
    }
    /// Unsets the `STABLE-RENDITION-ID` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_stable_rendition_id(&mut self) {
        self.attribute_list.remove(STABLE_RENDITION_ID);
        self.stable_rendition_id = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `DEFAULT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_default(&mut self, default: bool) {
        self.attribute_list.remove(DEFAULT);
        self.default = Some(default);
        self.output_line_is_dirty = true;
    }
    /// Sets the `AUTOSELECT` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_autoselect(&mut self, autoselect: bool) {
        self.attribute_list.remove(AUTOSELECT);
        self.autoselect = Some(autoselect);
        self.output_line_is_dirty = true;
    }
    /// Sets the `FORCED` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_forced(&mut self, forced: bool) {
        self.attribute_list.remove(FORCED);
        self.forced = Some(forced);
        self.output_line_is_dirty = true;
    }
    /// Sets the `INSTREAM-ID` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_instream_id(&mut self, instream_id: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(INSTREAM_ID);
        self.instream_id = Some(instream_id.into());
        self.output_line_is_dirty = true;
    }
    /// Unsets the `INSTREAM-ID` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_instream_id(&mut self) {
        self.attribute_list.remove(INSTREAM_ID);
        self.instream_id = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `BIT-DEPTH` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_bit_depth(&mut self, bit_depth: u64) {
        self.attribute_list.remove(BIT_DEPTH);
        self.bit_depth = Some(bit_depth);
        self.output_line_is_dirty = true;
    }
    /// Unsets the `BIT-DEPTH` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_bit_depth(&mut self) {
        self.attribute_list.remove(BIT_DEPTH);
        self.bit_depth = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `SAMPLE-RATE` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_sample_rate(&mut self, sample_rate: u64) {
        self.attribute_list.remove(SAMPLE_RATE);
        self.sample_rate = Some(sample_rate);
        self.output_line_is_dirty = true;
    }
    /// Unsets the `SAMPLE-RATE` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_sample_rate(&mut self) {
        self.attribute_list.remove(SAMPLE_RATE);
        self.sample_rate = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `CHARACTERISTICS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn set_characteristics(&mut self, characteristics: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CHARACTERISTICS);
        self.characteristics = Some(characteristics.into());
        self.output_line_is_dirty = true;
    }
    /// Unsets the `CHARACTERISTICS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    pub fn unset_characteristics(&mut self) {
        self.attribute_list.remove(CHARACTERISTICS);
        self.characteristics = None;
        self.output_line_is_dirty = true;
    }
    /// Sets the `CHANNELS` attribute.
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
    ///
    /// Given that `ValidChannels` implements `Into<Cow<str>>` it is possible to work with
    /// `ValidChannels` directly here. For example:
    /// ```
    /// # use m3u8::tag::hls::{Media, EnumeratedStringList, AudioCodingIdentifier, ValidChannels,
    /// # GetValid};
    /// # let mut media = Media::builder()
    /// #     .with_media_type("u")
    /// #     .with_name("n")
    /// #     .with_group_id("g")
    /// #     .finish();
    /// media.set_channels(ValidChannels::new(
    ///     12,
    ///     EnumeratedStringList::from([AudioCodingIdentifier::JointObjectCoding]),
    ///     "",
    /// ));
    /// assert_eq!(
    ///     "12/JOC",
    ///     media.channels().valid().expect("must be defined").as_ref()
    /// );
    /// ```
    /// It is also possible to set with a `&str` directly, but care should be taken to ensure the
    /// correct syntax is followed:
    /// ```
    /// # use m3u8::tag::hls::{Media, EnumeratedStringList, AudioCodingIdentifier, ValidChannels,
    /// # ChannelSpecialUsageIdentifier, GetValid};
    /// # let mut media = Media::builder()
    /// #     .with_media_type("u")
    /// #     .with_name("n")
    /// #     .with_group_id("g")
    /// #     .finish();
    /// media.set_channels("16/3OA/IMMERSIVE,BED-4,DOF-6");
    /// let channels = media.channels().valid().expect("should be defined");
    /// assert_eq!(16, channels.count());
    /// assert_eq!(1, channels.spatial_audio().iter().count());
    /// assert!(channels.spatial_audio().contains(AudioCodingIdentifier::OrderOfAmbisonics(3)));
    /// assert_eq!(3, channels.special_usage().iter().count());
    /// assert!(channels.special_usage().contains(ChannelSpecialUsageIdentifier::Immersive));
    /// assert!(channels.special_usage().contains(ChannelSpecialUsageIdentifier::Bed(4)));
    /// assert!(
    ///     channels.special_usage().contains(ChannelSpecialUsageIdentifier::DegreesOfFreedom(6))
    /// );
    /// ```
    /// The [`EnumeratedStringList`] provides some pseudo-set-like operations to help with mutating
    /// an existing value. Note, `to_owned` will need to be used on each of the string lists if
    /// setting back on the tag:
    /// ```
    /// # use m3u8::{Reader, HlsLine, config::ParsingOptions, tag::{known, hls}};
    /// # use m3u8::tag::hls::{Media, AudioCodingIdentifier, ChannelSpecialUsageIdentifier,
    /// # ValidChannels, GetValid};
    /// let tag = r#"#EXT-X-MEDIA:TYPE=AUDIO,NAME="a",GROUP-ID="a",CHANNELS="6/-/DOWNMIX,BED-4""#;
    /// let mut reader = Reader::from_str(tag, ParsingOptions::default());
    /// match reader.read_line() {
    ///     Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Media(mut media))))) => {
    ///         let channels = media.channels().valid().expect("should be defined");
    ///         let mut spatial = channels.spatial_audio();
    ///         spatial.insert(AudioCodingIdentifier::JointObjectCoding);
    ///         let mut special = channels.special_usage();
    ///         special.remove(ChannelSpecialUsageIdentifier::Bed(4));
    ///         media.set_channels(ValidChannels::new(
    ///             6,
    ///             spatial.to_owned(),
    ///             special.to_owned()
    ///         ));
    ///         
    ///         let new_channels = media.channels().valid().expect("should be defined");
    ///         assert_eq!("6/JOC/DOWNMIX", new_channels.as_ref());
    ///     }
    ///     r => panic!("unexpected result {r:?}"),
    /// }
    /// ```
    pub fn set_channels(&mut self, channels: impl Into<Cow<'a, str>>) {
        self.attribute_list.remove(CHANNELS);
        self.channels = Some(channels.into());
        self.output_line_is_dirty = true;
    }
    /// Unsets the `CHANNELS` attribute (sets it to `None`).
    ///
    /// See [`Self`] for a link to the HLS documentation for this attribute.
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
            Media::builder()
                .with_media_type(MediaType::ClosedCaptions)
                .with_name("English")
                .with_group_id("cc")
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
            Media::builder()
                .with_media_type(MediaType::Audio)
                .with_name("English")
                .with_group_id("stereo")
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
        Media::builder()
            .with_media_type(MediaType::Audio)
            .with_name("English")
            .with_group_id("stereo")
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
        (media_type, EnumeratedString::Known(MediaType::Video), @Attr="TYPE=VIDEO"),
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
