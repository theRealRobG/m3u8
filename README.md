# M3U8

## Basic usage

`m3u8` provides a `Reader` that can be used to extract HLS lines from an input string slice. An
example is provided below:
```rust
use m3u8::{
    config::ParsingOptionsBuilder,
    line::HlsLine,
    tag::hls::{ Endlist, Inf, M3u, Targetduration, Version },
    Reader,
};

const EXAMPLE_MANIFEST: &str = r#"#EXTM3U
#EXT-X-TARGETDURATION:10
#EXT-X-VERSION:3
#EXTINF:9.009,
first.ts
#EXTINF:9.009,
second.ts
#EXTINF:3.003,
third.ts
#EXT-X-ENDLIST
"#;

let mut reader = Reader::from_str(
    EXAMPLE_MANIFEST,
    ParsingOptionsBuilder::new()
        .with_parsing_for_all_tags()
        .build(),
);
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(M3u))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Targetduration::new(10)))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Version::new(3)))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(9.009, "")))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::uri("first.ts"))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(9.009, "")))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::uri("second.ts"))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(3.003, "")))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::uri("third.ts"))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Endlist))));
assert_eq!(reader.read_line(), Ok(None));
```

The above example demonstrates that a `HlsLine` is an with several potential cases. As per section
[4.1. Definition of a Playlist](https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.1):
> Each line is a URI, is blank, or starts with the character '#'. Lines that start with the
> character '#' are either comments or tags. Tags begin with #EXT.

The `HlsLine` in `m3u8` is defined as such:
```rust
use m3u8::tag::{
    known::{self, CustomTag, NoCustomTag},
    unknown,
};
use std::{borrow::Cow, fmt::Debug};

#[derive(Debug, PartialEq, Clone)]
pub enum HlsLine<'a, Custom = NoCustomTag>
where
    Custom: CustomTag<'a>,
{
    KnownTag(known::Tag<'a, Custom>),
    UnknownTag(unknown::Tag<'a>),
    Comment(Cow<'a, str>),
    Uri(Cow<'a, str>),
    Blank,
}
```

There are several things going on here so let's step through each.

### Blank, Comment, and Uri

These are fairly self expanatory. `HlsLine::Blank` represents a blank line that we encountered.
These could've been ignored, but for completeness, we leave them in.

Comments are lines that begin with `#` and are not followed by `EXT`. The slice refers to all
characters after the `#` up until (and not including) the new line (either `\r\n` or just `\n`). For
example, the line `# Hello!` would be parsed as `HlsLine::Comment(" Hello!")` (note the leading
space).

Uris are basically everything else. There is no validation during parsing that a URI line is a valid
URI. We just consume everything up until (and not including) the new line.

### UnknownTag

HLS defines 32 known tags that can occur in the playlist. Unknown tags are permitted (the
specification advises that they *SHOULD* be ignored). The presence of unknown tags are captured
within the `HlsLine::UnknownTag` case. The name of the tag is parsed and the value portion is split
out too, but no details other than the slice region of the value is defined. It is possible to set
the parsing options to ignore even known HLS tags, for example to improve parsing performance where
needed, and this is explained below.

### KnownTag

Known tags are broken out into two sub-cases:
* `Hls(hls::Tag<'a>)`
* `Custom(CustomTagAccess)`

The `Hls` case defines all 32 known tags as per the HLS specification. These are strongly typed
structs providing access to all defined values.

The `Custom` case allows for the library user to define their own custom known tag. Custom tag is
generic but must implement the `CustomTag` trait. This trait requires `Debug`, `PartialEq`, and also
`TryFrom<unknown::Tag<'a>, Error = ValidationError>` which is what is used to construct the tag from
parsed data. The trait includes `is_known_name(name: &str) -> bool` which has no `self` requirement,
as it is used as a test in the parser for whether `try_from` should be attempted for a given tag
name.

The custom tag implementation is wrapped in a `CustomTagAccess` struct which provides `AsRef` and
`AsMut` implementations to access the inner data. This struct allows the library to track if there
has ever been a mutable borrow of the inner custom tag. This is then used to decide on whether the
output line (for writing) needs to be recalculated based on the state of the tag at time of writing,
or if it is safe to use the original parsed data (and avoid any unnecessary allocation).

To enable writing, the custom tag must also implement `WritableCustomTag`. This introduces the
`into_writable_tag(self) -> WritableTag` requirement which consumes the tag in preparation for
writing. But as mentioned, this method is only called if there has been a mutable borrow of the
underlying custom tag, via `CustomTagAccess::as_mut(&mut self) -> &mut Custom`. If there is no
intention to write the parsed data then this trait implementation can be avoided.

We can demonstrate the usage of `CustomTag` by implementing it for the custom image media playlist
definition found on the Roku developer website here:
https://developer.roku.com/docs/developer-program/media-playback/trick-mode/hls-and-dash.md
```rust
use m3u8::{
    Reader,
    config::ParsingOptions,
    error::{ValidationError, ParseTagValueError, ParseAttributeValueError},
    line::HlsLine,
    tag::{
        hls::Inf,
        known::{self, CustomTag, WritableCustomTag, WritableTag},
        unknown,
        value::{
            DecimalResolution, MutableParsedAttributeValue, MutableSemiParsedTagValue,
            AttributeValue,
        },
    },
};
use std::{collections::HashMap, marker::PhantomData};

// To support multiple custom tags the preferred strategy is to encapsulate each within a single
// enum. For this example I am only demonstrating an implementation for the media playlist tags
// defined in the Roku developer docs (just to shorten the example).
#[derive(Debug, PartialEq, Clone)]
pub enum CustomImageTag {
    ImagesOnly,
    Tiles(Tiles),
}
// Here we specialize into our own strongly typed structure what m3u8 was able to parse from the
// input data.
impl TryFrom<unknown::Tag<'_>> for CustomImageTag {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag) -> Result<Self, Self::Error> {
        match tag.name() {
            "-X-IMAGES-ONLY" => Ok(CustomImageTag::ImagesOnly),
            "-X-TILES" => Ok(CustomImageTag::Tiles(Tiles::try_from(tag)?)),
            _ => Err(ValidationError::UnexpectedTagName),
        }
    }
}
// This is used to know when m3u8 should consider parsing the information as a known tag.
impl CustomTag<'_> for CustomImageTag {
    fn is_known_name(name: &str) -> bool {
        match name {
            "-X-IMAGES-ONLY" | "-X-TILES" => true,
            _ => false,
        }
    }
}
// This is used by the m3u8::Writer to handle writing of custom tag implementations.
impl<'a> WritableCustomTag<'a> for CustomImageTag {
    fn into_writable_tag(self) -> known::WritableTag<'a> {
        let name = match self {
            CustomImageTag::ImagesOnly => "-X-IMAGES-ONLY",
            CustomImageTag::Tiles(_) => "-X-TILES",
        };
        let value = match self {
            Self::ImagesOnly => MutableSemiParsedTagValue::Empty,
            Self::Tiles(tiles) => MutableSemiParsedTagValue::from([
                (
                    "RESOLUTION",
                    MutableParsedAttributeValue::UnquotedString(
                        format!("{}", tiles.resolution).into(),
                    ),
                ),
                (
                    "LAYOUT",
                    MutableParsedAttributeValue::UnquotedString(format!("{}", tiles.layout).into()),
                ),
                (
                    "DURATION",
                    MutableParsedAttributeValue::SignedDecimalFloatingPoint(tiles.duration),
                ),
            ]),
        };
        WritableTag::new(name, value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tiles {
    pub resolution: DecimalResolution,
    pub layout: DecimalResolution,
    pub duration: f64,
}
impl TryFrom<unknown::Tag<'_>> for Tiles {
    type Error = ValidationError;

    fn try_from(tag: unknown::Tag) -> Result<Self, Self::Error> {
        let attribute_list = tag
            .value()
            .ok_or(ParseTagValueError::UnexpectedEmpty)?
            .try_as_attribute_list()?;
        let resolution = attribute_list
            .get("RESOLUTION")
            .and_then(AttributeValue::unquoted)
            .ok_or(ValidationError::MissingRequiredAttribute("RESOLUTION"))?
            .try_as_decimal_resolution()
            .map_err(|e| {
                ValidationError::from(ParseAttributeValueError::DecimalResolution {
                    attr_name: "RESOLUTION",
                    error: e,
                })
            })?;
        let layout = attribute_list
            .get("LAYOUT")
            .and_then(AttributeValue::unquoted)
            .ok_or(ValidationError::MissingRequiredAttribute("LAYOUT"))?
            .try_as_decimal_resolution()
            .map_err(|e| {
                ValidationError::from(ParseAttributeValueError::DecimalResolution {
                    attr_name: "LAYOUT",
                    error: e,
                })
            })?;
        let duration = attribute_list
            .get("DURATION")
            .and_then(AttributeValue::unquoted)
            .ok_or(ValidationError::MissingRequiredAttribute("DURATION"))?
            .try_as_decimal_floating_point()
            .map_err(|e| {
                ValidationError::from(ParseAttributeValueError::DecimalFloatingPoint {
                    attr_name: "DURATION",
                    error: e,
                })
            })?;
        Ok(Self {
            resolution,
            layout,
            duration,
        })
    }
}

// Below we can demonstrate that the correct parsing occurs.
const EXAMPLE_LINES: &str = r#"#EXT-X-IMAGES-ONLY
#EXT-X-TILES:RESOLUTION=320x180,LAYOUT=5x4,DURATION=3.003
#EXTINF:60.06,Indicates 20 320x180 images (laid out 5x4) each to be displayed for 3.003s
image.1.jpeg"#;

// The `with_custom_from_str` method allows you to specify the custom tag type via providing the
// type in PhantomData as the third parameter.
let mut reader = Reader::with_custom_from_str(
    EXAMPLE_LINES,
    ParsingOptions::default(),
    PhantomData::<CustomImageTag>,
);
match reader.read_line() {
    Ok(Some(HlsLine::KnownTag(known::Tag::Custom(tag)))) => {
        assert_eq!(tag.as_ref(), &CustomImageTag::ImagesOnly)
    }
    l => panic!("unexpected line {l:?}"),
}
match reader.read_line() {
    Ok(Some(HlsLine::KnownTag(known::Tag::Custom(tag)))) => {
        assert_eq!(
            tag.as_ref(),
            &CustomImageTag::Tiles(Tiles {
                resolution: DecimalResolution {
                    width: 320,
                    height: 180
                },
                layout: DecimalResolution {
                    width: 5,
                    height: 4
                },
                duration: 3.003
            })
        )
    }
    l => panic!("unexpected line {l:?}"),
}
assert_eq!(
    reader.read_line(),
    Ok(Some(HlsLine::from(Inf::new(
        60.06,
        String::from(
            "Indicates 20 320x180 images (laid out 5x4) each to be displayed for 3.003s"
        )
    ))))
);
assert_eq!(
    reader.read_line(),
    Ok(Some(HlsLine::Uri("image.1.jpeg".into())))
);
```

### Enumerated strings

HLS defines an `enumerated-string` in
[section 4.2](https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2) as:
>*  enumerated-string: an unquoted character string from a set that is
>   explicitly defined by the AttributeName.  An enumerated-string
>   will never contain double quotes ("), commas (,), or whitespace.

By principle, this library is very lax on validation, and does not want to get in the way of
extracting information from playlists being parsed if it detects issues. As such, for enumerated
string attributes, we do not restrict to only the defined enumerated values; however, the library
does provide a convenience type for reasoning about the known values, while also exposing any that
are found which are unknown. The library has made best efforts to be convenient about the usage of
enumerated strings (in terms of strong typing) while still maintaining the flexibility of being
forward compatible against new cases introduced in specification updates (or just custom or bad
content).

Below is an example demonstrating the flexibility as applied to the `VideoRange` enumerated string
on the `EXT-X-STREAM-INF` tag:
```rust
use m3u8::{
    Reader,
    config::ParsingOptionsBuilder,
    line::HlsLine,
    tag::{
        hls::{self, EnumeratedString, VideoRange},
        known,
    },
};

let mut reader = Reader::from_str(
    "#EXT-X-STREAM-INF:BANDWIDTH=10000000,VIDEO-RANGE=SDR",
    ParsingOptionsBuilder::new()
        .with_parsing_for_stream_inf()
        .build(),
);
match reader.read_line() {
    Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::StreamInf(mut tag))))) => {
        // You can see that the value is strongly typed using the VideoRange enum.
        assert_eq!(
            Some(EnumeratedString::Known(VideoRange::Sdr)),
            tag.video_range()
        );
        // We can set the value on the tag to whatever we want (impl Into<Cow<str>>).
        tag.set_video_range("EXAMPLE");
        // Now we can see that we still can obtain the underlying value even though it is
        // unknown to the library.
        assert_eq!(
            Some(EnumeratedString::Unknown("EXAMPLE")),
            tag.video_range()
        );
        // All of the `EnumeratedString` implementations also implement `Into<Cow<str>>`.
        // This means that they can be used to set values on tags. Moreover, all of the
        // known types that `EnumeratedString` has wrapped also implement `Into<Cow<str>>`,
        // and so those can be used directly when setting values.
        tag.set_video_range(VideoRange::Pq);
        assert_eq!(
            Some(EnumeratedString::Known(VideoRange::Pq)),
            tag.video_range()
        );
    }
    _ => panic!("oh no, my demo failed"),
}
```

### Enumerated string lists

HLS defines an `enumerated-string-list` in
[section 4.2](https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2) as:
>*  enumerated-string-list: a quoted-string containing a comma-
>   separated list of enumerated-strings from a set that is explicitly
>   defined by the AttributeName.  Each enumerated-string in the list
>   is a string consisting of characters valid in an enumerated-
>   string.  The list SHOULD NOT repeat any enumerated-string.  To
>   support forward compatibility, clients MUST ignore any
>   unrecognized enumerated-strings in an enumerated-string-list.

Similar to the reasoning given above for `EnumeratedString` the library also aims to be flexible in
how these values are exposed. With string lists, there is an extra goal, that the library does not
want to unconditionally penalize performance for the added convenience, and so we've avoided using a
`Vec<T>` as it would introduce a heap allocation (just to read the string slice). The library
provides the `EnumeratedStringList` struct as a wrapper around the string contents to provide access
to enumerated values within (as split by `,`). The `contains` method provides a way to check if a
value exists in the list, and the `is_empty` method checks if there are any values in the list. For
convenience of mutating existing values on a tag, the `insert` and `remove` methods have been
provided, but normally to use this `to_owned` will also have to be called (`clone` is not enough),
as otherwise Rust will not allow mutation with an existing reference to tag data (a custom
`to_owned` method was introduced that erases the existing lifetimes to new ones so that it severs
the link from the tag).

Below is an example demonstrating the flexibility as applied to the `Cue` enumerated string list on
the `EXT-X-DATERANGE` tag:
```rust
use m3u8::{
    Reader,
    config::ParsingOptionsBuilder,
    line::HlsLine,
    tag::{
        hls::{self, EnumeratedString, EnumeratedStringList, Cue},
        known,
    },
};

let mut reader = Reader::from_str(
    r#"#EXT-X-DATERANGE:ID="1",START-DATE="2025-07-24T15:26:34.000Z",CUE="ONCE""#,
    ParsingOptionsBuilder::new()
        .with_parsing_for_daterange()
        .build(),
);
match reader.read_line() {
    Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Daterange(mut tag))))) => {
        // Since the `contains` method accepts `impl Into<Cow<str>>`, and all of the library
        // implementations of `EnumeratedString` also implement `Into<Cow<str>>`, we can use
        // the enumeration directly to validate if a value is contained within the list. The
        // following are essentially equivalent
        assert!(tag.cue().unwrap().contains(Cue::Once));
        assert!(tag.cue().unwrap().contains(EnumeratedString::Known(Cue::Once)));
        assert!(tag.cue().unwrap().contains("ONCE"));
        // We can set the value on the tag to whatever we want (impl Into<Cow<str>>).
        tag.set_cue("ONE,TWO,THREE");
        // We can still access the enumerated string list methods as expected.
        let mut cue = tag.cue().unwrap();
        assert!(cue.contains("ONE"));
        assert!(cue.contains("TWO"));
        assert!(cue.contains("THREE"));
        // We can also mutate the string list (even with these custom values) and mix in
        // known values (again, because it's all based on Cow<str> underneath).
        cue.remove("TWO");
        cue.insert(Cue::Post);
        assert!(!cue.contains("TWO"));
        assert!(cue.contains(Cue::Post));
        // At any stage we can escape hatch out of the string list to work with the inner
        // Cow<str>
        assert_eq!("ONE,THREE,POST", cue.as_ref());
        // We can now set our mutated `cue` back onto the tag; however, we must call
        // `to_owned` first.
        tag.set_cue(cue.to_owned());
        assert_eq!("ONE,THREE,POST", tag.cue().unwrap().as_ref());
        // We can also construct completely new enumerated string lists, and the library
        // provides a few convenience `Into` implementations to make this easier. The
        // following are essentially equivalent (though the `Vec` approach means a heap
        // allocation, while using an array (if the size is known) can avoid this).
        tag.set_cue(EnumeratedStringList::from([Cue::Pre, Cue::Once]));
        assert_eq!("PRE,ONCE", tag.cue().unwrap().as_ref());
        tag.set_cue(EnumeratedStringList::from(vec![Cue::Pre, Cue::Once]));
        assert_eq!("PRE,ONCE", tag.cue().unwrap().as_ref());
        tag.set_cue(EnumeratedStringList::<Cue>::from("PRE,ONCE"));
        assert_eq!("PRE,ONCE", tag.cue().unwrap().as_ref());
    }
    _ => panic!("oh no, my demo failed"),
}
```

### Slash separated lists

HLS defines a few attribute values using "slash separated lists". This library currently supports
(strongly types) the `EXT-X-MEDIA:CHANNELS` attribute and the `EXT-X-STREAM-INF:REQ-VIDEO-LAYOUT`
attribute. In the future it may also add support for `EXT-X-STREAM-INF:ALLOWED-CPC`. The support for
each of these are built on top of what has been discussed above for `EnumeratedString` and
`EnumeratedStringList`.

Below is a demonstration of the features that each wrapping type provides:
```rust
use m3u8::{
    Reader,
    config::ParsingOptionsBuilder,
    line::HlsLine,
    tag::{
        hls::{
            self,
            AudioCodingIdentifier, ChannelSpecialUsageIdentifier, Channels, ValidChannels,
            VideoChannelSpecifier, VideoProjectionSpecifier,
        },
        known,
    },
};

let tags = concat!(
    r#"#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="A",NAME="A",URI="a.m3u8",CHANNELS="2""#,
    "\n",
    r#"#EXT-X-STREAM-INF:BANDWIDTH=10000000,REQ-VIDEO-LAYOUT="CH-STEREO""#,
);
let mut reader = Reader::from_str(
    tags,
    ParsingOptionsBuilder::new()
        .with_parsing_for_media()
        .with_parsing_for_stream_inf()
        .build(),
);
match reader.read_line() {
    Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Media(mut tag))))) => {
        // Since channels must provide a valid count, the return value is `Channels` which
        // is an enum that has a `Valid` and `Invalid` case. To use the `Valid` case this
        // must be extracted.
        let channels = tag.channels();
        let valid_channels = channels.as_ref().and_then(|c| c.valid()).unwrap();
        assert_eq!(2, valid_channels.count());
        assert!(valid_channels.spatial_audio().is_empty());
        assert!(valid_channels.special_usage().is_empty());
        // The library supports the newly defined OA (order of ambisonics), BED, and DOF
        // (degrees of freedom) identifiers.
        tag.set_channels("16/3OA/BED-4,DOF-6");
        let channels = tag.channels();
        let valid_channels = channels.as_ref().and_then(|c| c.valid()).unwrap();
        assert_eq!(16, valid_channels.count());
        assert!(
            valid_channels
                .spatial_audio()
                .contains(AudioCodingIdentifier::OrderOfAmbisonics(3))
        );
        assert!(
            valid_channels
                .special_usage()
                .contains(ChannelSpecialUsageIdentifier::Bed(4))
        );
        assert!(
            valid_channels
                .special_usage()
                .contains(ChannelSpecialUsageIdentifier::DegreesOfFreedom(6))
        );
        // Since it's all built on top of enumerated string lists discussed above, the same
        // convenience initializers exist.
        tag.set_channels(Channels::Valid(ValidChannels::new(
            12,
            [AudioCodingIdentifier::JointObjectCoding],
            [ChannelSpecialUsageIdentifier::Binaural],
        )));
        assert_eq!("12/JOC/BINAURAL", tag.channels().unwrap().as_ref());
    }
    _ => panic!("oh no, my demo failed"),
}
match reader.read_line() {
    Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::StreamInf(mut tag))))) => {
        // REQ-VIDEO-LAYOUT has no mandatory parameters and so there is no "valid/invalid"
        // wrapping enum.
        let video_layout = tag.req_video_layout().unwrap();
        assert!(
            video_layout
                .channels()
                .contains(VideoChannelSpecifier::Stereo)
        );
        assert!(video_layout.projection().is_empty());
        // The library supports the newly defined projection specifiers.
        tag.set_req_video_layout("CH-STEREO/PROJ-PRIM");
        let video_layout = tag.req_video_layout().unwrap();
        assert!(
            video_layout
                .channels()
                .contains(VideoChannelSpecifier::Stereo)
        );
        assert!(
            video_layout
                .projection()
                .contains(VideoProjectionSpecifier::ParametricImmersive)
        );
        // HLS defines that the slash separated usage identifier entries list is unordered,
        // which is why values from each entry have a common prefix. The library accounts
        // for this and makes no assumption on ordering of entries.
        tag.set_req_video_layout("PROJ-PRIM/CH-STEREO");
        let video_layout = tag.req_video_layout().unwrap();
        assert!(
            video_layout
                .channels()
                .contains(VideoChannelSpecifier::Stereo)
        );
        assert!(
            video_layout
                .projection()
                .contains(VideoProjectionSpecifier::ParametricImmersive)
        );
        // To support forwards compatibility while still maintaining useful strong types for
        // what we know today, the `VideoLayout` also exposes any unknown entries via an
        // iterator (that is a filter on a `Split<char>` removing entries prefixed with
        // `"CH"` and `"PROJ"`).
        tag.set_req_video_layout("AA-HELLO/BB-WORLD");
        let video_layout = tag.req_video_layout().unwrap();
        let mut unknown_entries = video_layout.unknown_entries();
        assert_eq!(Some("AA-HELLO"), unknown_entries.next());
        assert_eq!(Some("BB-WORLD"), unknown_entries.next());
    }
    _ => panic!("oh no, my demo failed"),
}
```

## Configuring known tags for parsing

The parsing function allows the user to specify a subset of the known HLS tags that they would like
to parse fully into `m3u8::tag::hls::Tag` instances. For example, if information from
only `EXTINF` tags are desired, then the user can specify the parsing options using the
`ParsingOptionsBuilder` as
```rust
use m3u8::config::ParsingOptionsBuilder;
// Parse only EXTINF
ParsingOptionsBuilder::new().with_parsing_for_inf().build();
```
Alternatively, if most tags are desired, but a few tags can be ignored, then the user can set all
tags for parsing and remove the undesired tags as such:
```rust
use m3u8::config::ParsingOptionsBuilder;
// Parse everything except from EXT-X-BITRATE and EXT-X-PROGRAM-DATE-TIME
ParsingOptionsBuilder::new()
    .with_parsing_for_all_tags()
    .without_parsing_for_bitrate()
    .without_parsing_for_program_date_time()
    .build();
```

It may be quite desirable to avoid parsing of tags that are not needed as this can add quite
considerable performance overhead. Unknown tags make no attempt to parse or validate the value
portion of the tag (the part after `:`) and just return the name of the tag along with the `&str`
for the rest of the line. Running locally as of commit 6fcc38a67bf0eee0769b7e85f82599d1da6eb56d the
following benchmark shows that when parsing a large playlist, including all tags in the parse is
about 2x slower than including no tags in the parse (`2.3842 ms` vs `1.1364 ms`).
```sh
Large playlist, all tags, using Reader::from_str, no writing
                        time:   [2.3793 ms 2.3842 ms 2.3891 ms]
Large playlist, no tags, using Reader::from_str, no writing
                        time:   [1.1357 ms 1.1364 ms 1.1372 ms]
```

Some basic validation can still be done on `m3u8::tag::unknown::Tag`. For example, the name can be
converted to a `m3u8::tag::hls::TagName` and then you can check the `TagType` for some
generic reasoning on the tag position/semantics without parsing the values:
```rust
use m3u8::{
    error::ValidationError,
    tag::{
        hls::{TagName, TagType},
        unknown::Tag,
    },
};
fn handle_unknown_tag(tag: Tag) -> Result<(), ValidationError> {
    let tag_name = TagName::try_from(tag.name())?;
    match tag_name.tag_type() {
        TagType::Basic => todo!("handle_basic_tag"),
        TagType::MediaOrMultivariantPlaylist => todo!("handle_media_or_multivariant_playlist_tag"),
        TagType::MediaPlaylist => todo!("handle_media_playlist_tag"),
        TagType::MediaSegment => todo!("handle_media_segment_tag"),
        TagType::MediaMetadata => todo!("handle_media_metadata_tag"),
        TagType::MultivariantPlaylist => todo!("handle_multivariant_playlist_tag"),
    }
}
```

If there is a specific scenario where more information on a value is desired (other than just having
`&str`), then the user can use the `m3u8::tag::value::parse` method directly on the unknown
`tag.value`. To then get the full `m3u8::tag::hls::Tag` the user can pass the result
into `Tag::try_from`.

## Writing

The library provides a `Writer` to write parsed tags back into data. For example, this can be used
to parse a playlist, mutate the data, then write back the changed tags. Below is a toy example:
```rust
use m3u8::{
    config::ParsingOptions,
    line::HlsLine,
    tag::{hls, known},
    Reader, Writer,
};
use std::io;

let input_lines = concat!(
    "#EXTINF:4.00008,\n",
    "fileSequence268.mp4\n",
    "#EXTINF:4.00008,\n",
    "fileSequence269.mp4\n",
);
let mut reader = Reader::from_str(input_lines, ParsingOptions::default());
let mut writer = Writer::new(Vec::new());

let mut added_hello = false;
while let Ok(Some(line)) = reader.read_line() {
    match line {
        HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(mut inf))) => {
            if added_hello {
                inf.set_title(String::from("World!"));
            } else {
                inf.set_title(String::from("Hello,"));
                added_hello = true;
            }
            writer.write_line(HlsLine::from(inf)).unwrap()
        }
        line => writer.write_line(line).unwrap(),
    };
}

let expected_output_lines = concat!(
    "#EXTINF:4.00008,Hello,\n",
    "fileSequence268.mp4\n",
    "#EXTINF:4.00008,World!\n",
    "fileSequence269.mp4\n",
);
assert_eq!(
    expected_output_lines,
    String::from_utf8_lossy(&writer.into_inner())
);
```

Internally, all the known HLS tags (within `m3u8::tag::hls`) implement mutability using
`std::borrow::Cow`, and only construct new strings to represent the HLS line on mutation. This means
that if no mutation occurs then there are no string allocations when reading `from_str` and none
directly by `m3u8` during writing (no guarantees on what the implementation of `Write` used as input
to the `Writer::new` does). This is with the aim of optimizing reading and writing performance.

## More complex example - HLS Playlist Delta Update

A more complex example of using this library can be found within the 
[benches/delta_update_bench.rs](./benches/delta_update_bench.rs) benchmark. Here we have a fairly
thorough implementation of HLS Playlist Delta Updates intended to work with any given playlist. One
could imagine using this implementation in a proxy layer (e.g. a CDN edge function) in front of any
origin server, so as to add delta update functionality even where not supported at the origin, in an
efficient way (especially assuming that appropriate caching layers are present). At time of writing
this benchmark (commit 8665329a44aa45a2a59b158f10a6ce2b01aa31d4) the time taken to run this delta
update on a massive playlist (27,985 lines, resulting in 9,204 skipped segments) is measured as
`2.3001 ms` (running locally, Chip: Apple M1 Max, Memory: 64 GB).
```sh
Playlist delta update implementation using this library
                        time:   [2.2995 ms 2.3001 ms 2.3007 ms]
```

### Comparison with alternative libraries

#### m3u8-rs

[m3u8-rs](https://crates.io/crates/m3u8-rs) is the most popular m3u8 parser I've found on crates.io,
with 669,004 all time downloads at time of writing. Below the implementation of delta update using
our library in the benchmark file is an implementation using `m3u8-rs`. I've noted in comments that
there was significant difficulty implementing a delta update using this library, and ultimately, I
have not been able to do it correctly. There are several issues:
* Necessary tags EXT-X-SKIP and EXT-X-SERVER-CONTROL are not supported.
* The body of the playlist is only described in terms of segments, so media metadata tags must
  be associated to a segment, but this means that we cannot write daterange without having an
  associated segment.
* EXT-X-DATERANGE belongs to a segment, but the segment only has an Option of a daterange, and
  not a Vec. This means that we lose many daterange tags during parsing as it is quite normal
  to have more than one daterange together (e.g. chapter end followed by chapter start).

I've tried to work around the first two points; however, the last one makes it very difficult to
work with the library.

Nevertheless, I do have some implementation made, and so can compare some results from running the
bench locally:
```sh
Playlist delta update implementation using m3u8-rs library
                        time:   [6.5710 ms 6.5784 ms 6.5900 ms]
```

These results show that the implementation we've made using this library is almost 3x faster than
the implementation we've made using `m3u8-rs`.

#### hls_m3u8

[hls_m3u8](https://crates.io/crates/hls_m3u8) seems to be the second most popular m3u8 parser that I
can see on crates.io, with 72,351 all time downloads at time of writing. Below the m3u8-rs
implementation is the hls_m3u8 implementation. It is very similar to the m3u8-rs implementation and
interestingly shares many of the same limitations, and some more. In addition to all of the same
problems listed above, hls_m3u8 also has the following issues:
* There is no control over the EXT-X-VERSION tag. The library determines this itself based on what
  it knows from the tags that have been included; however, the library is out of date with respect
  to the HLS specification, and so it outputs an incorrect version tag.
* The library decides that it must have EXT-X-KEY tag on every segment, regardless of the source
  manifest, and will put EXT-X-KEY tags everywhere. Interestingly, since we removed the key tag with
  the delta update, the library interprets this to mean that there is no DRM, and puts key tags with
  METHOD=NONE everywhere. This is quite broken.
* The library does output unknown tags for the playlist, but dumps them at the bottom of the
  playlist, which is espcially problematic for any valid HLS tags that the library does not yet
  recognize.

Again, similar to m3u8-rs, it isn't really possible to implement a delta update using this library.
Perhaps my test is unfair? I really thought that delta updates seemed like a good use-case to test
as (for me at least) it had a realistic application, was relatively simple, but yet complex enough
to test the library implementation details. Anyway, from what I could produce, here are the results:

```sh
Playlist delta update implementation using hls_m3u8 library
                        time:   [5.0737 ms 5.0783 ms 5.0834 ms]
```

These results show that the implementation we've made using this library is about 2.2x faster than
the implementation we've made using `hls_m3u8`.

# HLS Specification

The parsing rules have been derived from the HLS specification listed here:
https://datatracker.ietf.org/doc/draft-pantos-hls-rfc8216bis/

At the time of writing draft 17 was used.

The following ABNF for a line has been interpreted from the specification:
```abnf
; 4.1. [...] Each line is a URI, is blank, or starts with the character '#'.
; Lines that start with the character '#' are either comments or tags.
;
hls-line                      = tag
                              / comment
                              / uri
                              / blank

; 4.1. [...] Tags begin with #EXT. They are case sensitive. All other lines
; that begin with '#' are comments and SHOULD be ignored.
;
tag                           = "#EXT" tag-name [":" tag-value]

; A specification for tag name format is not given, other than the set of
; names that are defined within HLS. We could make this an enumeration of
; only the defined tags; however, I prefer to have the flexibility to allow
; for any name, in case of future extension or custom tag definitions.
;
tag-name                      = 1*(ALPHA / DIGIT / "-")

; Examples:
; decimal-integer        -> #EXT-X-BYTERANGE:<n>[@<o>]
; type-enum              -> #EXT-X-PLAYLIST-TYPE:<type-enum>
; decimal-floating-point -> #EXTINF:<duration>,[<title>]
; date-time-msec         -> #EXT-X-PROGRAM-DATE-TIME:<date-time-msec>
; attribute-list         -> #EXT-X-START:<attribute-list>
;
tag-value                     = decimal-integer ["@" decimal-integer]
                              / type-enum
                              / decimal-floating-point ["," *(WSP / VCHAR)]
                              / date-time-msec
                              / attribute-list

; 4.2. [...] An attribute-list is a comma-separated list of attribute/value
; pairs with no whitespace. An attribute/value pair has the following
; syntax:
;     AttributeName=AttributeValue
;
attribute-list                = attribute-name "=" attribute-value
                                *("," attribute-name "=" attribute-value)

; 4.2. [...] An AttributeName is an unquoted string containing characters
; from the set [A-Z], [0-9], and '-'.
;
attribute-name                = 1*(uppercase / DIGIT / "-")

; 4.2. [...] An AttributeValue is one of the following:
; * decimal-integer
; * hexadecimal-sequence
; * decimal-floating-point
; * signed-decimal-floating-point
; * quoted-string
; * enumerated-string
; * enumerated-string-list
; * decimal-resolution
;
attribute-value               = decimal-integer
                              / hexadecimal-sequence
                              / decimal-floating-point
                              / signed-decimal-floating-point
                              / quoted-string
                              / enumerated-string
                              / enumerated-string-list
                              / decimal-resolution

; 4.2. [...] an unquoted string of characters from the set [0-9] expressing
; an integer in base-10 arithmetic in the range from 0 to 2^64-1
; (18446744073709551615). A decimal-integer may be from 1 to 20 characters
; long.
;
decimal-integer               = 1*20DIGIT

; 4.2. [...] an unquoted string of characters from the set [0-9] and [A-F]
; that is prefixed with 0x or 0X. The maximum length of a hexadecimal-
; sequence depends on its AttributeNames.
;
hexadecimal-sequence          = ("0x" / "0X") 1*HEXDIG

; 4.2. [...] an unquoted string of characters from the set [0-9] and '.'
; that expresses a non-negative floating-point number in decimal positional
; notation.
;
decimal-floating-point        = 1*DIGIT ["." 1*DIGIT]

; 4.2. [...] an unquoted string of characters from the set [0-9], '-', and
; '.' that expresses a signed floating-point number in decimal positional
; notation.
;
signed-decimal-floating-point = ["-"] 1*DIGIT ["." 1*DIGIT]

; 4.2. [...] a string of characters within a pair of double quotes (0x22).
; The following characters MUST NOT appear in a quoted-string: line feed
; (0xA), carriage return (0xD), or double quote (0x22). The string MUST be
; non-empty, unless specifically allowed. Quoted-string AttributeValues
; SHOULD be constructed so that byte-wise comparison is sufficient to test
; two quoted-string AttributeValues for equality. Note that this implies
; case-sensitive comparison.
;
quoted-string                 = DQUOTE
                                *(%x20-21 / %x23-7E)
                                DQUOTE

; 4.2. [...] an unquoted character string from a set that is explicitly
; defined by the AttributeName. An enumerated-string will never contain
; double quotes ("), commas (,), or whitespace.
;
enumerated-string             = *(%x20-21 / %x23-2B / %x2D-7E)

; 4.2. [...] a quoted-string containing a comma-separated list of
; enumerated-strings from a set that is explicitly defined by the
; AttributeName. Each enumerated-string in the list is a string consisting
; of characters valid in an enumerated-string. The list SHOULD NOT repeat
; any enumerated-string. To support forward compatibility, clients MUST
; ignore any unrecognized enumerated-strings in an enumerated-string-list.
;
enumerated-string-list        = DQUOTE
                                enumerated-string
                                *("," enumerated-string)
                                DQUOTE

; 4.2. [...] two decimal-integers separated by the "x" character. The first
; integer is a horizontal pixel dimension (width); the second is a vertical
; pixel dimension (height).
;
decimal-resolution            = 1*20DIGIT "x" 1*20DIGIT

; 4.4.3.5. [...] format is #EXT-X-PLAYLIST-TYPE:<type-enum> where type-enum
; is either EVENT or VOD.
;
type-enum                     = "EVENT" / "VOD"

; 4.4.4.6. [...] format is #EXT-X-PROGRAM-DATE-TIME:<date-time-msec> where
; date-time-msec is an ISO/IEC 8601:2004 date/time representation, such as
; YYYY-MM-DDThh:mm:ss.SSSZ. It SHOULD indicate a time zone and fractional
; parts of seconds, to at least millisecond accuracy. If no time zone is
; indicated, the client SHOULD treat the time zone as UTC.
;
date-time-msec                = <date-time@[RFC3339]>

; 4.1. [...] Tags begin with #EXT. They are case sensitive. All other lines
; that begin with '#' are comments and SHOULD be ignored.
comment                       = VCHAR

; A - Z
uppercase                     = %x41-5A
```

The `date-time` import from [RFC3339](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6)
at time of writing is copied below:
```abnf
date-fullyear   = 4DIGIT
date-month      = 2DIGIT  ; 01-12
date-mday       = 2DIGIT  ; 01-28, 01-29, 01-30, 01-31 based on
                            ; month/year
time-hour       = 2DIGIT  ; 00-23
time-minute     = 2DIGIT  ; 00-59
time-second     = 2DIGIT  ; 00-58, 00-59, 00-60 based on leap second
                            ; rules
time-secfrac    = "." 1*DIGIT
time-numoffset  = ("+" / "-") time-hour ":" time-minute
time-offset     = "Z" / time-numoffset

partial-time    = time-hour ":" time-minute ":" time-second
                    [time-secfrac]
full-date       = date-fullyear "-" date-month "-" date-mday
full-time       = partial-time time-offset

date-time       = full-date "T" full-time
```
