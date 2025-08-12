//! Types and methods related to associated values of [`Tag`].
//!
//! The definitions in this module provide the constructs necessary for both parsing to strongly
//! typed tags as well as writing when using [`crate::Writer`].

use crate::{
    error::ValidationError,
    tag::{
        hls, unknown,
        value::{WritableAttributeValue, WritableTagValue},
    },
    utils::split_on_new_line,
};
use std::{borrow::Cow, cmp::PartialEq, fmt::Debug};

/// Represents a HLS tag that is known to the library.
///
/// Known tags are split into two cases, those which are defined by the library, and those that are
/// custom defined by the user. The library makes an effort to reflect in types what is specified
/// via the latest `draft-pantos-hls-rfc8216` specification. The HLS specification also allows for
/// unknown tags which are intended to be ignored by clients; however, using that, special custom
/// implementations can be built up. Some notable examples are the `#EXT-X-SCTE35` tag defined in
/// [SCTE 35 standard] (which has been superceded by the SCTE35 attributes on `#EXT-X-DATERANGE`),
/// the `#EXT-X-IMAGE-STREAM-INF` tag (and associated tags) defined via [Roku Developers], the
/// `#EXT-X-PREFETCH` tag defined by [LHLS], and there are many more. We find that this flexibility
/// is a useful trait of HLS and so aim to support it here. For use cases where there is no need for
/// any custom tag parsing, the [`NoCustomTag`] implementation of [`CustomTag`] exists, and is the
/// default implementation of the generic `Custom` parameter in this enum.
///
/// [SCTE 35 standard]: https://account.scte.org/standards/library/catalog/scte-35-digital-program-insertion-cueing-message/
/// [Roku Developers]: https://developer.roku.com/docs/developer-program/media-playback/trick-mode/hls-and-dash.md#image-media-playlists-for-hls
/// [LHLS]: https://video-dev.github.io/hlsjs-rfcs/docs/0001-lhls
#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Tag<'a, Custom = NoCustomTag>
where
    Custom: CustomTag<'a>,
{
    // =============================================================================================
    //
    // Clippy suggests that the `Tag` within the `Hls` case should be put in a Box, based on
    // https://rust-lang.github.io/rust-clippy/master/index.html#large_enum_variant
    //   > The largest variant contains at least 272 bytes; Boxing the large field
    //   > (hls::Tag) reduces the total size of the enum.
    //
    // However, the description also indicates:
    //   > This lint obviously cannot take the distribution of variants in your running program into
    //   > account. It is possible that the smaller variants make up less than 1% of all instances,
    //   > in which case the overhead is negligible and the boxing is counter-productive. Always
    //   > measure the change this lint suggests.
    //
    // In other words, the box only really makes sense, if there is a somewhat even distribution of
    // instances of each variant. If most instances are going to be the `Hls` case then we aren't
    // really saving on memory. Furthermore, putting the `Tag` in a `Box` incurrs a performance
    // penalty (validated with a Criterion bench), because we are now allocating and retrieving from
    // the heap.
    //
    // I believe that the vast majority of cases where the parser is being used we will be using
    // instances of the `Hls` variant, and therefore, I am not putting the `Tag` in a `Box` and so
    // ignoring the Clippy warning.
    //
    // =============================================================================================
    /// Indicates that the tag found was one of the 32 known tags defined in the HLS specification
    /// that are supported here in this library _(and that were not ignored by
    /// [`crate::config::ParsingOptions`])_. See [`hls::Tag`] for a more complete documentation of
    /// all of the known tag types.
    Hls(hls::Tag<'a>),
    /// Indicates that the tag found was one matching the [`CustomTag`] definition that the user of
    /// the library has defined. The tag is wrapped in a [`CustomTagAccess`] struct (see that struct
    /// documentation for reasoning on why the wrapping exists) and can be borrowed via the
    /// [`CustomTagAccess::as_ref`] method or mutably borrowed via the [`CustomTagAccess::as_mut`]
    /// method. Refer to [`CustomTag`] for more information on how to define a custom tag.
    Custom(CustomTagAccess<'a, Custom>),
}

/// The inner data of a parsed tag.
///
/// This struct is primarily useful for the [`crate::Writer`], but can be used outside of writing,
/// if the user needs to have custom access on the byte-slice content of the tag. The slice the
/// inner data holds may come from a data source provided during parsing, or may be an owned
/// `Vec<u8>` if the tag was mutated or constructed using a builder method for the tag. When the
/// inner data is a byte slice of parsed data, it may be a slice of the rest of the playlist from
/// where the tag was found; however, the [`Self::value`] method ensures that only the relevant
/// bytes for this line are provided.
#[derive(Debug)]
pub struct TagInner<'a> {
    pub(crate) output_line: Cow<'a, [u8]>,
}
impl<'a> TagInner<'a> {
    /// Provides the value of the inner data.
    ///
    /// The method ensures that only data from this line is provided as the value (even if the slice
    /// of borrowed data extends past the line until the end of the playlist).
    pub fn value(&self) -> &[u8] {
        split_on_new_line(&self.output_line).parsed
    }
}

/// The ability to convert self into a [`TagInner`].
pub trait IntoInnerTag<'a> {
    /// Consume `self` and provide [`TagInner`].
    fn into_inner(self) -> TagInner<'a>;
}

/// Trait to define a custom tag implementation.
///
/// The trait comes in two parts:
/// 1. [`CustomTag::is_known_name`] which allows the library to know whether a tag line (line
///    prefixed with `#EXT`) should be considered a possible instance of this implementation.
/// 2. `TryFrom<unknown::Tag>` which is where the parsing into the custom tag instance is attempted.
///
/// The [`unknown::Tag`] struct provides the name of the tag and the value (if it exists), split out
/// and wrapped in a struct that provides parsing methods for several data types defined in the HLS
/// specification. The concept here is that when we are converting into our known tag we have the
/// right context to choose the best parsing method for the tag value type we expect. If we were to
/// try and parse values up front, then we would run into issues, like trying to distinguish between
/// an integer and a float if the mantissa (fractional part) is not present. Taking a lazy approach
/// to parsing helps us avoid these ambiguities, and also, provdies a performance improvement as we
/// do not waste attempts at parsing data in an unexpected format.
///
/// ## Single tag example
///
/// Suppose we have a proprietary extension of HLS where we have added the following tag:
/// ```text
/// EXT-X-JOKE
///
///    The EXT-X-JOKE tag allows a server to provide a joke to the client.
///    It is OPTIONAL. Its format is:
///
///    #EXT-X-JOKE:<attribute-list>
///
///    The following attributes are defined:
///
///       TYPE
///
///       The value is an enumerated-string; valid strings are DAD, PUN,
///       BAR, STORY, and KNOCK-KNOCK. This attribute is REQUIRED.
///
///       JOKE
///
///       The value is a quoted-string that includes the contents of the
///       joke. The value MUST be hilarious. Clients SHOULD reject the joke
///       if it does not ellicit at least a smile. If the TYPE is DAD, then
///       the client SHOULD groan on completion of the joke. This attribute
///       is REQUIRED.
/// ```
/// We may choose to model this tag as such (adding the derive attributes for convenience):
/// ```
/// #[derive(Debug, PartialEq, Clone)]
/// struct JokeTag<'a> {
///     joke_type: JokeType,
///     joke: &'a str,
/// }
///
/// #[derive(Debug, PartialEq, Clone)]
/// enum JokeType {
///     Dad,
///     Pun,
///     Bar,
///     Story,
///     KnockKnock,
/// }
/// ```
/// The first step we must take is to implement the parsing logic for this tag. To do that we must
/// implement the `TryFrom<unknown::Tag>` requirement. We may do this as follows:
/// ```
/// # use quick_m3u8::{
/// #     tag::{
/// #         unknown,
/// #         value::AttributeValue,
/// #     },
/// #     error::{ValidationError, ParseTagValueError, ParseAttributeValueError}
/// # };
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # struct JokeTag<'a> {
/// #     joke_type: JokeType,
/// #     joke: &'a str,
/// # }
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # enum JokeType {
/// #     Dad,
/// #     Pun,
/// #     Bar,
/// #     Story,
/// #     KnockKnock,
/// # }
/// impl<'a> TryFrom<unknown::Tag<'a>> for JokeTag<'a> {
///     type Error = ValidationError;
///
///     fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
///         // Ensure that the value of the tag corresponds to `<attribute-list>`
///         let list = tag
///             .value()
///             .ok_or(ParseTagValueError::UnexpectedEmpty)?
///             .try_as_attribute_list()?;
///         // Ensure that the `JOKE` attribute exists and is of the correct type.
///         let joke = list
///             .get("JOKE")
///             .and_then(AttributeValue::quoted)
///             .ok_or(ValidationError::MissingRequiredAttribute("JOKE"))?;
///         // Ensure that the `TYPE` attribute exists and is of the correct type. Note the
///         // difference that this type is `Unquoted` instead of `Quoted`, and so we use the helper
///         // method `unquoted` rather than `quoted`. This signifies the use of the HLS defined
///         // `enumerated-string` attribute value type.
///         let joke_type_str = list
///             .get("TYPE")
///             .and_then(AttributeValue::unquoted)
///             .ok_or(ValidationError::MissingRequiredAttribute("TYPE"))?
///             .try_as_utf_8()
///             .map_err(|e| ValidationError::from(
///                 ParseAttributeValueError::Utf8 { attr_name: "TYPE", error: e }
///             ))?;
///         // Translate the enumerated string value into the enum cases we support, otherwise,
///         // return an error.
///         let Some(joke_type) = (match joke_type_str {
///             "DAD" => Some(JokeType::Dad),
///             "PUN" => Some(JokeType::Pun),
///             "BAR" => Some(JokeType::Bar),
///             "STORY" => Some(JokeType::Story),
///             "KNOCK-KNOCK" => Some(JokeType::KnockKnock),
///             _ => None,
///         }) else {
///             return Err(ValidationError::InvalidEnumeratedString);
///         };
///         // Now we have our joke.
///         Ok(Self { joke_type, joke })
///     }
/// }
/// ```
/// Now we can simply implement the `CustomTag` requirement via the `is_known_name` method. Note
/// that the tag name is everything after `#EXT` (and before `:`), implying that the `-X-` is
/// included in the name:
/// ```
/// # use quick_m3u8::{
/// #     tag::{
/// #         known::CustomTag,
/// #         unknown,
/// #         value::AttributeValue,
/// #     },
/// #     error::{ValidationError, ParseTagValueError, ParseAttributeValueError}
/// # };
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # struct JokeTag<'a> {
/// #     joke_type: JokeType,
/// #     joke: &'a str,
/// # }
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # enum JokeType {
/// #     Dad,
/// #     Pun,
/// #     Bar,
/// #     Story,
/// #     KnockKnock,
/// # }
/// # impl<'a> TryFrom<unknown::Tag<'a>> for JokeTag<'a> {
/// #     type Error = ValidationError;
/// #
/// #     fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
/// #         // Ensure that the value of the tag corresponds to `<attribute-list>`
/// #         let list = tag
/// #             .value()
/// #             .ok_or(ParseTagValueError::UnexpectedEmpty)?
/// #             .try_as_attribute_list()?;
/// #         // Ensure that the `JOKE` attribute exists and is of the correct type.
/// #         let joke = list
/// #             .get("JOKE")
/// #             .and_then(AttributeValue::quoted)
/// #             .ok_or(ValidationError::MissingRequiredAttribute("JOKE"))?;
/// #         // Ensure that the `TYPE` attribute exists and is of the correct type. Note the
/// #         // difference that this type is `Unquoted` instead of `Quoted`, and so we use the helper
/// #         // method `unquoted` rather than `quoted`. This signifies the use of the HLS defined
/// #         // `enumerated-string` attribute value type.
/// #         let joke_type_str = list
/// #             .get("TYPE")
/// #             .and_then(AttributeValue::unquoted)
/// #             .ok_or(ValidationError::MissingRequiredAttribute("TYPE"))?
/// #             .try_as_utf_8()
/// #             .map_err(|e| ValidationError::from(
/// #                 ParseAttributeValueError::Utf8 { attr_name: "TYPE", error: e }
/// #             ))?;
/// #         // Translate the enumerated string value into the enum cases we support, otherwise,
/// #         // return an error.
/// #         let Some(joke_type) = (match joke_type_str {
/// #             "DAD" => Some(JokeType::Dad),
/// #             "PUN" => Some(JokeType::Pun),
/// #             "BAR" => Some(JokeType::Bar),
/// #             "STORY" => Some(JokeType::Story),
/// #             "KNOCK-KNOCK" => Some(JokeType::KnockKnock),
/// #             _ => None,
/// #         }) else {
/// #             return Err(ValidationError::InvalidEnumeratedString);
/// #         };
/// #         // Now we have our joke.
/// #         Ok(Self { joke_type, joke })
/// #     }
/// # }
/// impl<'a> CustomTag<'a> for JokeTag<'a> {
///     fn is_known_name(name: &str) -> bool {
///         name == "-X-JOKE"
///     }
/// }
/// ```
/// At this stage we are ready to use our tag, for example, as part of a [`crate::Reader`]. Below we
/// include an example playlist string and show parsing of the joke working. Note that we define our
/// custom tag with the reader using [`std::marker::PhantomData`] and the
/// [`crate::Reader::with_custom_from_str`] method.
/// ```
/// # use quick_m3u8::{
/// #     Reader, HlsLine,
/// #     config::ParsingOptions,
/// #     tag::{
/// #         known::{CustomTag, Tag},
/// #         unknown,
/// #         value::AttributeValue,
/// #         hls::{Version, Targetduration, M3u}
/// #     },
/// #     error::{ValidationError, ParseTagValueError, ParseAttributeValueError},
/// # };
/// # use std::marker::PhantomData;
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # struct JokeTag<'a> {
/// #     joke_type: JokeType,
/// #     joke: &'a str,
/// # }
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # enum JokeType {
/// #     Dad,
/// #     Pun,
/// #     Bar,
/// #     Story,
/// #     KnockKnock,
/// # }
/// # impl<'a> TryFrom<unknown::Tag<'a>> for JokeTag<'a> {
/// #     type Error = ValidationError;
/// #
/// #     fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
/// #         // Ensure that the value of the tag corresponds to `<attribute-list>`
/// #         let list = tag
/// #             .value()
/// #             .ok_or(ParseTagValueError::UnexpectedEmpty)?
/// #             .try_as_attribute_list()?;
/// #         // Ensure that the `JOKE` attribute exists and is of the correct type.
/// #         let joke = list
/// #             .get("JOKE")
/// #             .and_then(AttributeValue::quoted)
/// #             .ok_or(ValidationError::MissingRequiredAttribute("JOKE"))?;
/// #         // Ensure that the `TYPE` attribute exists and is of the correct type. Note the
/// #         // difference that this type is `Unquoted` instead of `Quoted`, and so we use the helper
/// #         // method `unquoted` rather than `quoted`. This signifies the use of the HLS defined
/// #         // `enumerated-string` attribute value type.
/// #         let joke_type_str = list
/// #             .get("TYPE")
/// #             .and_then(AttributeValue::unquoted)
/// #             .ok_or(ValidationError::MissingRequiredAttribute("TYPE"))?
/// #             .try_as_utf_8()
/// #             .map_err(|e| ValidationError::from(
/// #                 ParseAttributeValueError::Utf8 { attr_name: "TYPE", error: e }
/// #             ))?;
/// #         // Translate the enumerated string value into the enum cases we support, otherwise,
/// #         // return an error.
/// #         let Some(joke_type) = (match joke_type_str {
/// #             "DAD" => Some(JokeType::Dad),
/// #             "PUN" => Some(JokeType::Pun),
/// #             "BAR" => Some(JokeType::Bar),
/// #             "STORY" => Some(JokeType::Story),
/// #             "KNOCK-KNOCK" => Some(JokeType::KnockKnock),
/// #             _ => None,
/// #         }) else {
/// #             return Err(ValidationError::InvalidEnumeratedString);
/// #         };
/// #         // Now we have our joke.
/// #         Ok(Self { joke_type, joke })
/// #     }
/// # }
/// # impl<'a> CustomTag<'a> for JokeTag<'a> {
/// #     fn is_known_name(name: &str) -> bool {
/// #         name == "-X-JOKE"
/// #     }
/// # }
/// const EXAMPLE: &str = r#"#EXTM3U
/// #EXT-X-TARGETDURATION:10
/// #EXT-X-VERSION:3
/// #EXT-X-JOKE:TYPE=DAD,JOKE="Why did the bicycle fall over? Because it was two-tired!"
/// # Forgive me, I'm writing this library in my spare time during paternity leave, so this seems
/// # appropriate to me at this stage.
/// #EXTINF:9.009
/// segment.0.ts
/// "#;
///
/// let mut reader = Reader::with_custom_from_str(
///     EXAMPLE,
///     ParsingOptions::default(),
///     PhantomData::<JokeTag>,
/// );
/// // First 3 tags as expected
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(M3u))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Targetduration::new(10)))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Version::new(3)))));
/// // And the big reveal
/// match reader.read_line() {
///     Ok(Some(HlsLine::KnownTag(Tag::Custom(tag)))) => {
///         assert_eq!(
///             &JokeTag {
///                 joke_type: JokeType::Dad,
///                 joke: "Why did the bicycle fall over? Because it was two-tired!",
///             },
///             tag.as_ref()
///         );
///     }
///     r => panic!("unexpected result {r:?}"),
/// }
/// ```
///
/// ## Multiple tag example
///
/// The same concepts extend to defining multiple custom tags. For example, in 2018 (before the
/// standardization of LL-HLS), the good people at JWPlayer and hls.js proposed a new extension of
/// HLS to support low latency streaming. This proposal was captured in [hlsjs-rfcs-0001]. It added
/// two tags: `#EXT-X-PREFETCH:<URI>` and `#EXT-X-PREFETCH-DISCONTINUITY`. Below we make an attempt
/// to implement these as custom tags. We don't break for commentary as most of this was explained
/// in the example above. This example was chosen as the defined tag values are not `attribute-list`
/// and so we can demonstrate different tag parsing techniques.
/// ```
/// # use quick_m3u8::{HlsLine, Reader, config::ParsingOptions, tag::known::Tag, tag::hls::{M3u,
/// # Version, Targetduration, MediaSequence, DiscontinuitySequence, Inf, ProgramDateTime},
/// # date_time, tag::known::CustomTag, error::{ValidationError, ParseTagValueError}, tag::unknown,
/// # tag::value::TagValue};
/// # use std::marker::PhantomData;
/// #[derive(Debug, PartialEq, Clone)]
/// enum LHlsTag<'a> {
///     Discontinuity,
///     Prefetch(&'a str),
/// }
///
/// impl<'a> LHlsTag<'a> {
///     fn try_from_discontinuity(value: Option<TagValue>) -> Result<Self, ValidationError> {
///         match value {
///             Some(_) => Err(ValidationError::from(ParseTagValueError::NotEmpty)),
///             None => Ok(Self::Discontinuity)
///         }
///     }
///
///     fn try_from_prefetch(value: Option<TagValue<'a>>) -> Result<Self, ValidationError> {
///         // Note that the `TagValue` provides methods for parsing value data as defined in the
///         // HLS specification, as extracted from the existing tag definitions (there is specific
///         // definition for possible attribute-list value types; however, for general tag values,
///         // this has to be inferred from what tags are defined). `TagValue` does not provide a
///         // `try_as_utf_8` method, since the only tag that defines a text value is the
///         // `EXT-X-PLAYLIST-TYPE` tag, but this is an enumerated string (`EVENT` or `VOD`), and
///         // so we just offer `try_as_playlist_type`. Nevertheless, the inner data of `TagValue`
///         // is accessible, and so we can convert to UTF-8 ourselves here, as shown below.
///         let unparsed = value.ok_or(ParseTagValueError::UnexpectedEmpty)?;
///         let Ok(uri) = std::str::from_utf8(unparsed.0) else {
///             return Err(ValidationError::MissingRequiredAttribute("<URI>"));
///         };
///         Ok(Self::Prefetch(uri))
///     }
/// }
///
/// impl<'a> TryFrom<unknown::Tag<'a>> for LHlsTag<'a> {
///     type Error = ValidationError;
///
///     fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
///         match tag.name() {
///             "-X-PREFETCH-DISCONTINUITY" => Self::try_from_discontinuity(tag.value()),
///             "-X-PREFETCH" => Self::try_from_prefetch(tag.value()),
///             _ => Err(ValidationError::UnexpectedTagName),
///         }
///     }
/// }
///
/// impl<'a> CustomTag<'a> for LHlsTag<'a> {
///     fn is_known_name(name: &str) -> bool {
///         name == "-X-PREFETCH" || name == "-X-PREFETCH-DISCONTINUITY"
///     }
/// }
/// // This example is taken from the "Examples" section under the "Discontinuities" example.
/// const EXAMPLE: &str = r#"#EXTM3U
/// #EXT-X-VERSION:3
/// #EXT-X-TARGETDURATION:2
/// #EXT-X-MEDIA-SEQUENCE:0
/// #EXT-X-DISCONTINUITY-SEQUENCE:0
///
/// #EXT-X-PROGRAM-DATE-TIME:2018-09-05T20:59:06.531Z
/// #EXTINF:2.000
/// https://foo.com/bar/0.ts
/// #EXT-X-PROGRAM-DATE-TIME:2018-09-05T20:59:08.531Z
/// #EXTINF:2.000
/// https://foo.com/bar/1.ts
///
/// #EXT-X-PREFETCH-DISCONTINUITY
/// #EXT-X-PREFETCH:https://foo.com/bar/5.ts
/// #EXT-X-PREFETCH:https://foo.com/bar/6.ts"#;
///
/// let mut reader = Reader::with_custom_from_str(
///     EXAMPLE,
///     ParsingOptions::default(),
///     PhantomData::<LHlsTag>,
/// );
///
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(M3u))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Version::new(3)))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Targetduration::new(2)))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(MediaSequence::new(0)))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(DiscontinuitySequence::new(0)))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::Blank)));
/// assert_eq!(
///     reader.read_line(),
///     Ok(Some(HlsLine::from(ProgramDateTime::new(
///         date_time!(2018-09-05 T 20:59:06.531)
///     ))))
/// );
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(2.0, "")))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::Uri("https://foo.com/bar/0.ts".into()))));
/// assert_eq!(
///     reader.read_line(),
///     Ok(Some(HlsLine::from(ProgramDateTime::new(
///         date_time!(2018-09-05 T 20:59:08.531)
///     ))))
/// );
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(2.0, "")))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::Uri("https://foo.com/bar/1.ts".into()))));
/// assert_eq!(reader.read_line(), Ok(Some(HlsLine::Blank)));
///
/// match reader.read_line() {
///     Ok(Some(HlsLine::KnownTag(Tag::Custom(tag)))) => {
///         assert_eq!(&LHlsTag::Discontinuity, tag.as_ref());
///     }
///     r => panic!("unexpected result {r:?}"),
/// }
/// match reader.read_line() {
///     Ok(Some(HlsLine::KnownTag(Tag::Custom(tag)))) => {
///         assert_eq!(&LHlsTag::Prefetch("https://foo.com/bar/5.ts"), tag.as_ref());
///     }
///     r => panic!("unexpected result {r:?}"),
/// }
/// match reader.read_line() {
///     Ok(Some(HlsLine::KnownTag(Tag::Custom(tag)))) => {
///         assert_eq!(&LHlsTag::Prefetch("https://foo.com/bar/6.ts"), tag.as_ref());
///     }
///     r => panic!("unexpected result {r:?}"),
/// }
///
/// assert_eq!(reader.read_line(), Ok(None)); // end of example
/// ```
///
/// [hlsjs-rfcs-0001]: https://video-dev.github.io/hlsjs-rfcs/docs/0001-lhls
pub trait CustomTag<'a>:
    TryFrom<unknown::Tag<'a>, Error = ValidationError> + Debug + PartialEq
{
    /// Check if the provided name is known for this custom tag implementation.
    ///
    /// This method is called before any attempt to parse the data into a CustomTag (it is the test
    /// for whether an attempt will be made to parse to CustomTag).
    fn is_known_name(name: &str) -> bool;
}
/// A custom tag implementation that allows for writing using [`crate::Writer`].
///
/// If there is no intention to write the parsed data then this trait does not need to be
/// implemented for the [`CustomTag`]. We can extend the examples from [`CustomTag`] to also
/// implement this trait so that we can demonstrate writing the data to an output.
///
/// ## Single tag example
///
/// Recall that the single tag example was for the custom defined `#EXT-X-JOKE` tag. Here we show
/// how we may change the joke (e.g. if we are acting as a proxy) before writing to output. Note, in
/// a real implementation we would make the stored property a [`std::borrow::Cow`] and not require
/// the user to provide a string slice reference with the same lifetime as the parsed data, but this
/// is just extending an existing example for information purposes.
/// ```
/// # use quick_m3u8::{
/// #     Reader, HlsLine, Writer,
/// #     config::ParsingOptions,
/// #     tag::{
/// #         known::{CustomTag, Tag, WritableTag, WritableCustomTag},
/// #         unknown,
/// #         value::{
/// #             WritableTagValue, WritableAttributeValue, AttributeValue
/// #         },
/// #         hls::{Version, Targetduration, M3u}
/// #     },
/// #     error::{ValidationError, ParseTagValueError, ParseAttributeValueError}
/// # };
/// # use std::{marker::PhantomData, borrow::Cow, collections::HashMap};
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # struct JokeTag<'a> {
/// #     joke_type: JokeType,
/// #     joke: &'a str,
/// # }
/// #
/// # #[derive(Debug, PartialEq, Clone)]
/// # enum JokeType {
/// #     Dad,
/// #     Pun,
/// #     Bar,
/// #     Story,
/// #     KnockKnock,
/// # }
/// # impl<'a> TryFrom<unknown::Tag<'a>> for JokeTag<'a> {
/// #     type Error = ValidationError;
/// #
/// #     fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
/// #         // Ensure that the value of the tag corresponds to `<attribute-list>`
/// #         let list = tag
/// #             .value()
/// #             .ok_or(ParseTagValueError::UnexpectedEmpty)?
/// #             .try_as_attribute_list()?;
/// #         // Ensure that the `JOKE` attribute exists and is of the correct type.
/// #         let joke = list
/// #             .get("JOKE")
/// #             .and_then(AttributeValue::quoted)
/// #             .ok_or(ValidationError::MissingRequiredAttribute("JOKE"))?;
/// #         // Ensure that the `TYPE` attribute exists and is of the correct type. Note the
/// #         // difference that this type is `Unquoted` instead of `Quoted`, and so we use the helper
/// #         // method `unquoted` rather than `quoted`. This signifies the use of the HLS defined
/// #         // `enumerated-string` attribute value type.
/// #         let joke_type_str = list
/// #             .get("TYPE")
/// #             .and_then(AttributeValue::unquoted)
/// #             .ok_or(ValidationError::MissingRequiredAttribute("TYPE"))?
/// #             .try_as_utf_8()
/// #             .map_err(|e| ValidationError::from(
/// #                 ParseAttributeValueError::Utf8 { attr_name: "TYPE", error: e }
/// #             ))?;
/// #         // Translate the enumerated string value into the enum cases we support, otherwise,
/// #         // return an error.
/// #         let Some(joke_type) = (match joke_type_str {
/// #             "DAD" => Some(JokeType::Dad),
/// #             "PUN" => Some(JokeType::Pun),
/// #             "BAR" => Some(JokeType::Bar),
/// #             "STORY" => Some(JokeType::Story),
/// #             "KNOCK-KNOCK" => Some(JokeType::KnockKnock),
/// #             _ => None,
/// #         }) else {
/// #             return Err(ValidationError::InvalidEnumeratedString);
/// #         };
/// #         // Now we have our joke.
/// #         Ok(Self { joke_type, joke })
/// #     }
/// # }
/// # impl<'a> CustomTag<'a> for JokeTag<'a> {
/// #     fn is_known_name(name: &str) -> bool {
/// #         name == "-X-JOKE"
/// #     }
/// # }
/// impl JokeType {
///     fn as_str(self) -> &'static str {
///         match self {
///             JokeType::Dad => "DAD",
///             JokeType::Pun => "PUN",
///             JokeType::Bar => "BAR",
///             JokeType::Story => "STORY",
///             JokeType::KnockKnock => "KNOCK-KNOCK",
///         }
///     }
/// }
/// impl<'a> JokeTag<'a> {
///     fn set_joke(&mut self, joke: &'static str) {
///         self.joke = joke;
///     }
/// }
/// impl<'a> WritableCustomTag<'a> for JokeTag<'a> {
///     fn into_writable_tag(self) -> WritableTag<'a> {
///         // Note, that the `WritableTag` expects to have `name: Cow<'a, str>` and
///         // `value: WritableTagValue<'a>`; however, the `new` method accepts
///         // `impl Into<Cow<'a, str>>` for name, and `impl Into<WritableTagValue<'a>>` for value.
///         // The library provides convenience `From<T>` implementations for many types of `T` to
///         // `WritableTagValue`, so this may help in some cases with shortening how much needs to
///         // be written. Below we make use of `From<[(K, V); N]>` where `const N: usize`,
///         // `K: Into<Cow<'a, str>>`, and `V: Into<WritableAttributeValue>`.
///         WritableTag::new(
///             "-X-JOKE",
///             [
///                 (
///                     "TYPE",
///                     WritableAttributeValue::UnquotedString(self.joke_type.as_str().into()),
///                 ),
///                 (
///                     "JOKE",
///                     WritableAttributeValue::QuotedString(self.joke.into()),
///                 ),
///             ],
///         )
///     }
/// }
/// # const EXAMPLE: &str = r#"#EXTM3U
/// # #EXT-X-TARGETDURATION:10
/// # #EXT-X-VERSION:3
/// # #EXT-X-JOKE:TYPE=DAD,JOKE="Why did the bicycle fall over? Because it was two-tired!"
/// # #EXTINF:9.009
/// # segment.0.ts
/// # "#;
/// #
/// # let mut reader = Reader::with_custom_from_str(
/// #     EXAMPLE,
/// #     ParsingOptions::default(),
/// #     PhantomData::<JokeTag>,
/// # );
/// let mut writer = Writer::new(Vec::new());
/// // First 3 tags as expected
/// let Some(m3u) = reader.read_line()? else { return Ok(()) };
/// writer.write_custom_line(m3u)?;
/// let Some(targetduration) = reader.read_line()? else { return Ok(()) };
/// writer.write_custom_line(targetduration)?;
/// let Some(version) = reader.read_line()? else { return Ok(()) };
/// writer.write_custom_line(version)?;
/// // And the big reveal
/// match reader.read_line() {
///     Ok(Some(HlsLine::KnownTag(Tag::Custom(mut tag)))) => {
///         tag.as_mut().set_joke("What happens when a frog's car breaks down? It gets toad!");
///         writer.write_custom_line(HlsLine::from(tag))?;
///     }
///     r => panic!("unexpected result {r:?}"),
/// }
///
/// // Because the HashMap we return does not guarantee order of the attributes, we validate that
/// // the result is one of the expected outcomes.
/// const EXPECTED_1: &str = r#"#EXTM3U
/// #EXT-X-TARGETDURATION:10
/// #EXT-X-VERSION:3
/// #EXT-X-JOKE:TYPE=DAD,JOKE="What happens when a frog's car breaks down? It gets toad!"
/// "#;
/// const EXPECTED_2: &str = r#"#EXTM3U
/// #EXT-X-TARGETDURATION:10
/// #EXT-X-VERSION:3
/// #EXT-X-JOKE:JOKE="What happens when a frog's car breaks down? It gets toad!",TYPE=DAD
/// "#;
/// let inner_bytes = writer.into_inner();
/// let actual = std::str::from_utf8(&inner_bytes)?;
/// assert!(actual == EXPECTED_1 || actual == EXPECTED_2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Multiple tag example
///
/// Recall that the multiple tag example was for the [LHLS] extension to the specification. Here we
/// show how we may change the prefetch URL (e.g. if we are acting as a proxy) before writing to
/// output.
/// ```
/// # use quick_m3u8::{HlsLine, Reader, config::ParsingOptions, tag::known::Tag, tag::hls::{M3u,
/// # Version, Targetduration, MediaSequence, DiscontinuitySequence, Inf, ProgramDateTime},
/// # date_time, tag::known::CustomTag, error::{ValidationError, ParseTagValueError}, tag::unknown,
/// # tag::known::{WritableCustomTag, WritableTag}, tag::value::{TagValue, WritableTagValue},
/// # Writer};
/// # use std::{marker::PhantomData, io::Write};
/// #[derive(Debug, PartialEq, Clone)]
/// # enum LHlsTag<'a> {
/// #     Discontinuity,
/// #     Prefetch(&'a str),
/// # }
/// #
/// # impl<'a> LHlsTag<'a> {
/// #     fn try_from_discontinuity(value: Option<TagValue>) -> Result<Self, ValidationError> {
/// #         match value {
/// #             Some(_) => Err(ValidationError::from(ParseTagValueError::NotEmpty)),
/// #             None => Ok(Self::Discontinuity)
/// #         }
/// #     }
/// #
/// #     fn try_from_prefetch(value: Option<TagValue<'a>>) -> Result<Self, ValidationError> {
/// #         // Note that the `TagValue` provides methods for parsing value data as defined in the
/// #         // HLS specification, as extracted from the existing tag definitions (there is specific
/// #         // definition for possible attribute-list value types; however, for general tag values,
/// #         // this has to be inferred from what tags are defined). `TagValue` does not provide a
/// #         // `try_as_utf_8` method, since the only tag that defines a text value is the
/// #         // `EXT-X-PLAYLIST-TYPE` tag, but this is an enumerated string (`EVENT` or `VOD`), and
/// #         // so we just offer `try_as_playlist_type`. Nevertheless, the inner data of `TagValue`
/// #         // is accessible, and so we can convert to UTF-8 ourselves here, as shown below.
/// #         let unparsed = value.ok_or(ParseTagValueError::UnexpectedEmpty)?;
/// #         let Ok(uri) = std::str::from_utf8(unparsed.0) else {
/// #             return Err(ValidationError::MissingRequiredAttribute("<URI>"));
/// #         };
/// #         Ok(Self::Prefetch(uri))
/// #     }
/// # }
/// #
/// # impl<'a> TryFrom<unknown::Tag<'a>> for LHlsTag<'a> {
/// #     type Error = ValidationError;
/// #
/// #     fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
/// #         match tag.name() {
/// #             "-X-PREFETCH-DISCONTINUITY" => Self::try_from_discontinuity(tag.value()),
/// #             "-X-PREFETCH" => Self::try_from_prefetch(tag.value()),
/// #             _ => Err(ValidationError::UnexpectedTagName),
/// #         }
/// #     }
/// # }
/// #
/// # impl<'a> CustomTag<'a> for LHlsTag<'a> {
/// #     fn is_known_name(name: &str) -> bool {
/// #         name == "-X-PREFETCH" || name == "-X-PREFETCH-DISCONTINUITY"
/// #     }
/// # }
/// impl<'a> WritableCustomTag<'a> for LHlsTag<'a> {
///     fn into_writable_tag(self) -> WritableTag<'a> {
///         // Note, as mentioned above, the `WritableTag::new` method accepts types that implement
///         // `Into` the stored properties on the struct. Below we make use of `From<&str>` for
///         // `WritableTagValue` in the `Prefetch` case to cut down on boilerplate.
///         match self {
///             Self::Discontinuity => WritableTag::new(
///                 "-X-PREFETCH-DISCONTINUITY",
///                 WritableTagValue::Empty
///             ),
///             Self::Prefetch(uri) => WritableTag::new("-X-PREFETCH", uri),
///         }
///     }
/// }
/// # // This example is taken from the "Examples" section under the "Discontinuities" example.
/// # const EXAMPLE: &str = r#"#EXTM3U
/// # #EXT-X-VERSION:3
/// # #EXT-X-TARGETDURATION:2
/// # #EXT-X-MEDIA-SEQUENCE:0
/// # #EXT-X-DISCONTINUITY-SEQUENCE:0
/// #
/// # #EXT-X-PROGRAM-DATE-TIME:2018-09-05T20:59:06.531Z
/// # #EXTINF:2.000
/// # https://foo.com/bar/0.ts
/// # #EXT-X-PROGRAM-DATE-TIME:2018-09-05T20:59:08.531Z
/// # #EXTINF:2.000
/// # https://foo.com/bar/1.ts
/// #
/// # #EXT-X-PREFETCH-DISCONTINUITY
/// # #EXT-X-PREFETCH:https://foo.com/bar/5.ts
/// # #EXT-X-PREFETCH:https://foo.com/bar/6.ts"#;
/// #
/// # let mut reader = Reader::with_custom_from_str(
/// #     EXAMPLE,
/// #     ParsingOptions::default(),
/// #     PhantomData::<LHlsTag>,
/// # );
///
/// let mut writer = Writer::new(Vec::new());
/// let mut last_segment_index = 0;
/// loop {
///     match reader.read_line() {
///         Ok(Some(HlsLine::KnownTag(Tag::Custom(tag)))) => {
///             match tag.as_ref() {
///                 LHlsTag::Discontinuity => {
///                     writer.write_custom_line(HlsLine::from(tag))?;
///                 }
///                 LHlsTag::Prefetch(uri) => {
///                     // For demo purposes we make the URI segment numbers sequential.
///                     if let Some(last_component) = uri.split('/').last() {
///                         let new_uri = uri.replace(
///                             last_component,
///                             format!("{}.ts", last_segment_index + 1).as_str()
///                         );
///                         writer.write_custom_tag(LHlsTag::Prefetch(new_uri.as_str()))?;
///                     } else {
///                         writer.write_custom_line(HlsLine::from(tag))?;
///                     }
///                 }
///             };
///         }
///         Ok(Some(HlsLine::Uri(uri))) => {
///             last_segment_index = uri
///                 .split('/')
///                 .last()
///                 .and_then(|file| file.split('.').next())
///                 .and_then(|n| n.parse::<u32>().ok())
///                 .unwrap_or_default();
///             writer.write_line(HlsLine::Uri(uri))?;
///         }
///         Ok(Some(line)) => {
///             writer.write_custom_line(line)?;
///         }
///         Ok(None) => break,
///         Err(e) => {
///             writer.get_mut().write_all(e.errored_line.as_bytes())?;
///         }
///     }
/// }
/// const EXPECTED: &str = r#"#EXTM3U
/// #EXT-X-VERSION:3
/// #EXT-X-TARGETDURATION:2
/// #EXT-X-MEDIA-SEQUENCE:0
/// #EXT-X-DISCONTINUITY-SEQUENCE:0
///
/// #EXT-X-PROGRAM-DATE-TIME:2018-09-05T20:59:06.531Z
/// #EXTINF:2.000
/// https://foo.com/bar/0.ts
/// #EXT-X-PROGRAM-DATE-TIME:2018-09-05T20:59:08.531Z
/// #EXTINF:2.000
/// https://foo.com/bar/1.ts
///
/// #EXT-X-PREFETCH-DISCONTINUITY
/// #EXT-X-PREFETCH:https://foo.com/bar/2.ts
/// #EXT-X-PREFETCH:https://foo.com/bar/3.ts
/// "#;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [LHLS]: https://video-dev.github.io/hlsjs-rfcs/docs/0001-lhls
pub trait WritableCustomTag<'a>: CustomTag<'a> {
    /// Takes ownership of the custom tag and provides a value that is used for writing.
    ///
    /// This method is only called if there was a mutable borrow of the custom tag at some stage. If
    /// the tag was never mutably borrowed, then when writing, the library will use the original
    /// input data (thus avoiding unnecessary allocations).
    fn into_writable_tag(self) -> WritableTag<'a>;
}

/// Wrapper around a [`CustomTag`] implementation for access control.
///
/// The wrapper allows the library to selectively decide when it will call the
/// [`WritableCustomTag::into_writable_tag`] method. When there has been no mutable reference borrow
/// of the custom tag ([`Self::as_mut`]) then the [`Self::into_inner`] implementation will use the
/// original parsed byte-slice directly (rather than allocate any new strings to construct a new
/// line).
#[derive(Debug, PartialEq, Clone)]
pub struct CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    pub(crate) custom_tag: Custom,
    pub(crate) is_dirty: bool,
    pub(crate) original_input: &'a [u8],
}

impl<'a, Custom> TryFrom<unknown::Tag<'a>> for CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    type Error = ValidationError;

    fn try_from(value: unknown::Tag<'a>) -> Result<Self, Self::Error> {
        let original_input = value.original_input;
        let custom_tag = Custom::try_from(value)?;
        Ok(Self {
            custom_tag,
            is_dirty: false,
            original_input,
        })
    }
}
impl<'a, Custom> AsRef<Custom> for CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    fn as_ref(&self) -> &Custom {
        &self.custom_tag
    }
}
impl<'a, Custom> AsMut<Custom> for CustomTagAccess<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    fn as_mut(&mut self) -> &mut Custom {
        self.is_dirty = true;
        &mut self.custom_tag
    }
}

impl<'a, Custom> IntoInnerTag<'a> for CustomTagAccess<'a, Custom>
where
    Custom: WritableCustomTag<'a>,
{
    fn into_inner(self) -> TagInner<'a> {
        if self.is_dirty {
            self.custom_tag.into_inner()
        } else {
            TagInner {
                output_line: Cow::Borrowed(self.original_input),
            }
        }
    }
}

impl<'a, Custom> IntoInnerTag<'a> for Custom
where
    Custom: WritableCustomTag<'a>,
{
    fn into_inner(self) -> TagInner<'a> {
        let output = calculate_output(self);
        TagInner {
            output_line: Cow::Owned(output.into_bytes()),
        }
    }
}

pub(crate) fn calculate_output<'a, Custom: WritableCustomTag<'a>>(custom_tag: Custom) -> String {
    let tag = custom_tag.into_writable_tag();
    match tag.value {
        WritableTagValue::Empty => format!("#EXT{}", tag.name),
        WritableTagValue::DecimalFloatingPointWithOptionalTitle(n, t) => {
            if t.is_empty() {
                format!("#EXT{}:{n}", tag.name)
            } else {
                format!("#EXT{}:{n},{t}", tag.name)
            }
        }
        WritableTagValue::DecimalInteger(n) => format!("#EXT{}:{n}", tag.name),
        WritableTagValue::DecimalIntegerRange(n, Some(o)) => format!("#EXT{}:{n}@{o}", tag.name),
        WritableTagValue::DecimalIntegerRange(n, None) => format!("#EXT{}:{n}", tag.name),
        WritableTagValue::DateTime(d) => format!("#EXT{}:{d}", tag.name),
        WritableTagValue::AttributeList(list) => {
            let attrs = list
                .iter()
                .map(|(k, v)| match v {
                    WritableAttributeValue::DecimalInteger(n) => format!("{k}={n}"),
                    WritableAttributeValue::SignedDecimalFloatingPoint(n) => {
                        format!("{k}={n:?}")
                    }
                    WritableAttributeValue::DecimalResolution(r) => {
                        format!("{k}={}x{}", r.width, r.height)
                    }
                    WritableAttributeValue::QuotedString(s) => format!("{k}=\"{s}\""),
                    WritableAttributeValue::UnquotedString(s) => format!("{k}={s}"),
                })
                .collect::<Vec<String>>();
            let value = attrs.join(",");
            format!("#EXT{}:{}", tag.name, value)
        }
        WritableTagValue::Utf8(s) => format!("#EXT{}:{s}", tag.name),
    }
}

/// A tag representation that makes writing from custom tags easier.
///
/// This is provided so that custom tag implementations may provide an output that does not depend
/// on having parsed data to derive the write output from. This helps with mutability as well as
/// allowing for custom tags to be constructed from scratch (without being parsed from source data).
#[derive(Debug, PartialEq)]
pub struct WritableTag<'a> {
    /// The name of the tag.
    ///
    /// This must include everything after the `#EXT` prefix and before the `:` or new line. For
    /// example, `#EXTM3U` has name `M3U`, `#EXT-X-VERSION:3` has name `-X-VERSION`, etc.
    pub name: Cow<'a, str>,
    /// The value of the tag.
    ///
    /// The [`WritableTagValue`] provides data types that allow for owned data (rather than just
    /// borrowed references from parsed input data). See the enum documentation for more information
    /// on what values can be defined.
    pub value: WritableTagValue<'a>,
}
impl<'a> WritableTag<'a> {
    /// Create a new tag.
    ///
    /// ## Examples
    /// ### Empty
    /// ```
    /// # use quick_m3u8::tag::known::WritableTag;
    /// # use quick_m3u8::tag::value::WritableTagValue;
    /// WritableTag::new("-X-EXAMPLE", WritableTagValue::Empty);
    /// ```
    /// produces a tag that would write as `#EXT-X-EXAMPLE`.
    /// ### Integer
    /// ```
    /// # use quick_m3u8::tag::known::WritableTag;
    /// # use quick_m3u8::tag::value::WritableTagValue;
    /// # use std::borrow::Cow;
    /// let explicit = WritableTag::new(
    ///     Cow::Borrowed("-X-EXAMPLE"),
    ///     WritableTagValue::DecimalInteger(42),
    /// );
    /// // Or, with convenience `From<u64>`
    /// let terse = WritableTag::new("-X-EXAMPLE", 42);
    /// assert_eq!(explicit, terse);
    /// ```
    /// produces a tag that would write as `#EXT-X-EXAMPLE:42`.
    /// ### Integer range
    /// ```
    /// # use quick_m3u8::tag::known::WritableTag;
    /// # use quick_m3u8::tag::value::WritableTagValue;
    /// # use std::borrow::Cow;
    /// let explicit = WritableTag::new(
    ///     Cow::Borrowed("-X-EXAMPLE"),
    ///     WritableTagValue::DecimalIntegerRange(1024, Some(512)),
    /// );
    /// // Or, with convenience `From<(u64, Option<u64>)>`
    /// let terse = WritableTag::new("-X-EXAMPLE", (1024, Some(512)));
    /// assert_eq!(explicit, terse);
    /// ```
    /// produces a tag that would write as `#EXT-X-EXAMPLE:1024@512`.
    /// ### Float with title
    /// ```
    /// # use quick_m3u8::tag::known::WritableTag;
    /// # use quick_m3u8::tag::value::WritableTagValue;
    /// # use std::borrow::Cow;
    /// let explicit = WritableTag::new(
    ///     Cow::Borrowed("-X-EXAMPLE"),
    ///     WritableTagValue::DecimalFloatingPointWithOptionalTitle(3.14, "pi".into()),
    /// );
    /// // Or, with convenience `From<(f64, impl Into<Cow<str>>)>`
    /// let terse = WritableTag::new("-X-EXAMPLE", (3.14, "pi"));
    /// assert_eq!(explicit, terse);
    /// ```
    /// produces a tag that would write as `#EXT-X-EXAMPLE:3.14,pi`.
    /// ### Date time
    /// ```
    /// # use quick_m3u8::date_time;
    /// # use quick_m3u8::tag::known::WritableTag;
    /// # use quick_m3u8::tag::value::WritableTagValue;
    /// # use std::borrow::Cow;
    /// let explicit = WritableTag::new(
    ///     Cow::Borrowed("-X-EXAMPLE"),
    ///     WritableTagValue::DateTime(date_time!(2025-08-10 T 21:51:42.123 -05:00)),
    /// );
    /// // Or, with convenience `From<DateTime>`
    /// let terse = WritableTag::new("-X-EXAMPLE", date_time!(2025-08-10 T 21:51:42.123 -05:00));
    /// assert_eq!(explicit, terse);
    /// ```
    /// produces a tag that would write as `#EXT-X-EXAMPLE:2025-08-10T21:51:42.123-05:00`.
    /// ### Attribute list
    /// ```
    /// # use quick_m3u8::tag::known::WritableTag;
    /// # use quick_m3u8::tag::value::{WritableTagValue, WritableAttributeValue};
    /// # use std::collections::HashMap;
    /// # use std::borrow::Cow;
    /// let explicit = WritableTag::new(
    ///     Cow::Borrowed("-X-EXAMPLE"),
    ///     WritableTagValue::AttributeList(HashMap::from([
    ///         ("VALUE".into(), WritableAttributeValue::DecimalInteger(42)),
    ///     ])),
    /// );
    /// // Or, with convenience `From<[(K, V); N]>`
    /// let terse = WritableTag::new("-X-EXAMPLE", [("VALUE", 42)]);
    /// assert_eq!(explicit, terse);
    /// ```
    /// produces a tag that would write as `#EXT-X-EXAMPLE:VALUE=42`.
    /// ### UTF-8
    /// ```
    /// # use quick_m3u8::tag::known::WritableTag;
    /// # use quick_m3u8::tag::value::{WritableTagValue, WritableAttributeValue};
    /// # use std::borrow::Cow;
    /// let explicit = WritableTag::new(
    ///     Cow::Borrowed("-X-EXAMPLE"),
    ///     WritableTagValue::Utf8(Cow::Borrowed("HELLO")),
    /// );
    /// // Or, with convenience `From<&str>`
    /// let terse = WritableTag::new("-X-EXAMPLE", "HELLO");
    /// assert_eq!(explicit, terse);
    /// ```
    /// produces a tag that would write as `#EXT-X-EXAMPLE:HELLO`.
    pub fn new(name: impl Into<Cow<'a, str>>, value: impl Into<WritableTagValue<'a>>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Implementation of [`CustomTag`] for convenience default `HlsLine::Custom` implementation.
///
/// Given that `HlsLine` takes a generic parameter, if this struct did not exist, then the user
/// would always have to define some custom tag implementation to use the library. This would add
/// unintended complexity. Therefore, this struct comes with the library, and provides the default
/// implementation of `CustomTag`. This implementation ensures that it is never parsed from source
/// data, because [`Self::is_known_name`] always returns false.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NoCustomTag;
impl TryFrom<unknown::Tag<'_>> for NoCustomTag {
    type Error = ValidationError;

    fn try_from(_: unknown::Tag) -> Result<Self, Self::Error> {
        Err(ValidationError::NotImplemented)
    }
}
impl CustomTag<'_> for NoCustomTag {
    fn is_known_name(_: &str) -> bool {
        false
    }
}
impl WritableCustomTag<'_> for NoCustomTag {
    fn into_writable_tag(self) -> WritableTag<'static> {
        WritableTag::new("-NO-TAG", WritableTagValue::Empty)
    }
}

impl<'a, Custom> TryFrom<unknown::Tag<'a>> for Tag<'a, Custom>
where
    Custom: CustomTag<'a>,
{
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag<'a>) -> Result<Self, Self::Error> {
        if Custom::is_known_name(tag.name) {
            let original_input = tag.original_input;
            let custom_tag = Custom::try_from(tag)?;
            Ok(Self::Custom(CustomTagAccess {
                custom_tag,
                is_dirty: false,
                original_input,
            }))
        } else {
            Ok(Self::Hls(hls::Tag::try_from(tag)?))
        }
    }
}

impl<'a, Custom> IntoInnerTag<'a> for Tag<'a, Custom>
where
    Custom: WritableCustomTag<'a>,
{
    fn into_inner(self) -> TagInner<'a> {
        match self {
            Tag::Hls(tag) => tag.into_inner(),
            Tag::Custom(tag) => tag.into_inner(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Reader, Writer, config::ParsingOptions, error::ParseTagValueError, line::HlsLine,
        tag::value::AttributeValue,
    };
    use pretty_assertions::assert_eq;
    use std::marker::PhantomData;

    #[derive(Debug, PartialEq)]
    struct TestTag {
        mutated: bool,
    }
    impl TryFrom<unknown::Tag<'_>> for TestTag {
        type Error = ValidationError;
        fn try_from(tag: unknown::Tag<'_>) -> Result<Self, Self::Error> {
            let list = tag
                .value()
                .ok_or(ParseTagValueError::UnexpectedEmpty)?
                .try_as_attribute_list()?;
            let Some(mutated_str) = list
                .get("MUTATED")
                .and_then(AttributeValue::unquoted)
                .and_then(|v| v.try_as_utf_8().ok())
            else {
                return Err(ValidationError::MissingRequiredAttribute("MUTATED"));
            };
            match mutated_str {
                "NO" => Ok(Self { mutated: false }),
                "YES" => Ok(Self { mutated: true }),
                _ => Err(ValidationError::InvalidEnumeratedString),
            }
        }
    }
    impl CustomTag<'_> for TestTag {
        fn is_known_name(name: &str) -> bool {
            name == "-X-TEST-TAG"
        }
    }
    impl WritableCustomTag<'_> for TestTag {
        fn into_writable_tag(self) -> WritableTag<'static> {
            let value = if self.mutated { "YES" } else { "NO" };
            WritableTag::new(
                "-X-TEST-TAG",
                [(
                    "MUTATED",
                    WritableAttributeValue::UnquotedString(value.into()),
                )],
            )
        }
    }

    #[test]
    fn custom_tag_should_be_mutable() {
        let data = "#EXT-X-TEST-TAG:MUTATED=NO";
        let mut reader =
            Reader::with_custom_from_str(data, ParsingOptions::default(), PhantomData::<TestTag>);
        let mut writer = Writer::new(Vec::new());
        match reader.read_line() {
            Ok(Some(HlsLine::KnownTag(Tag::Custom(mut tag)))) => {
                assert_eq!(false, tag.as_ref().mutated);
                tag.as_mut().mutated = true;
                assert_eq!(true, tag.as_ref().mutated);
                writer
                    .write_custom_line(HlsLine::from(tag))
                    .expect("should not fail write");
            }
            l => panic!("unexpected line {l:?}"),
        }
        assert_eq!(
            "#EXT-X-TEST-TAG:MUTATED=YES\n",
            std::str::from_utf8(&writer.into_inner()).expect("should be valid str")
        );
    }

    // This implementation we'll set the writable tag output to a value not related to the tag to
    // demonstrate that it is only accessed for the output when mutated.
    #[derive(Debug, PartialEq)]
    struct WeirdTag {
        number: f64,
    }
    impl TryFrom<unknown::Tag<'_>> for WeirdTag {
        type Error = ValidationError;
        fn try_from(tag: unknown::Tag<'_>) -> Result<Self, Self::Error> {
            let number = tag
                .value()
                .ok_or(ParseTagValueError::UnexpectedEmpty)?
                .try_as_decimal_floating_point()?;
            Ok(Self { number })
        }
    }
    impl CustomTag<'_> for WeirdTag {
        fn is_known_name(name: &str) -> bool {
            name == "-X-WEIRD-TAG"
        }
    }
    impl WritableCustomTag<'_> for WeirdTag {
        fn into_writable_tag(self) -> WritableTag<'static> {
            WritableTag::new("-X-WEIRD-TAG", [("SO-WEIRD", 999)])
        }
    }

    #[test]
    fn custom_tag_should_only_use_into_writable_tag_when_mutated() {
        let data = "#EXT-X-WEIRD-TAG:42";
        let mut reader =
            Reader::with_custom_from_str(data, ParsingOptions::default(), PhantomData::<WeirdTag>);
        let mut writer = Writer::new(Vec::new());
        match reader.read_line() {
            Ok(Some(HlsLine::KnownTag(Tag::Custom(tag)))) => {
                assert_eq!(42.0, tag.as_ref().number);
                writer
                    .write_custom_line(HlsLine::from(tag))
                    .expect("should not fail write");
            }
            l => panic!("unexpected line {l:?}"),
        }
        assert_eq!(
            "#EXT-X-WEIRD-TAG:42\n",
            std::str::from_utf8(&writer.into_inner()).expect("should be valid str")
        );

        // Now re-run the test with mutation
        let mut reader =
            Reader::with_custom_from_str(data, ParsingOptions::default(), PhantomData::<WeirdTag>);
        let mut writer = Writer::new(Vec::new());
        match reader.read_line() {
            Ok(Some(HlsLine::KnownTag(Tag::Custom(mut tag)))) => {
                assert_eq!(42.0, tag.as_ref().number);
                tag.as_mut().number = 69.0;
                writer
                    .write_custom_line(HlsLine::from(tag))
                    .expect("should not fail write");
            }
            l => panic!("unexpected line {l:?}"),
        }
        assert_eq!(
            "#EXT-X-WEIRD-TAG:SO-WEIRD=999\n",
            std::str::from_utf8(&writer.into_inner()).expect("should be valid str")
        );
    }
}
