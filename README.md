# M3U8

## Basic usage

`m3u8` provides a `Reader` that can be used to extract HLS lines from an input string slice. An
example is provided below:
```rust
use m3u8::{
    config::ParsingOptionsBuilder,
    line::HlsLine,
    tag::hls::{
        endlist::Endlist, inf::Inf, m3u::M3u, targetduration::Targetduration, version::Version,
    },
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
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(9.009, String::new())))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::Uri("first.ts"))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(9.009, String::new())))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::Uri("second.ts"))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(3.003, String::new())))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::Uri("third.ts"))));
assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Endlist))));
assert_eq!(reader.read_line(), Ok(None));
```

The above example demonstrates that a `HlsLine` is an with several potential cases. As per section
[4.1. Definition of a Playlist](https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.1):
> Each line is a URI, is blank, or starts with the character '#'. Lines that start with the
> character '#' are either comments or tags. Tags begin with #EXT.

The `HlsLine` in `m3u8` is defined as such:
```rust
use m3u8::{
    error::ValidationError,
    tag::{
        known::{self, IsKnownName, NoCustomTag, ParsedTag, TagInformation},
        unknown,
    },
};
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub enum HlsLine<'a, CustomTag = NoCustomTag>
where
    CustomTag: TryFrom<ParsedTag<'a>, Error = ValidationError>
        + IsKnownName
        + TagInformation
        + Debug
        + PartialEq,
{
    KnownTag(known::Tag<'a, CustomTag>),
    UnknownTag(unknown::Tag<'a>),
    Comment(&'a str),
    Uri(&'a str),
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
* `Custom(CustomTag)`

The `Hls` case defines all 32 known tags as per the HLS specification. These are strongly typed
structs providing access to all defined values.

The `Custom` case allows for the library user to define their own custom known tag. Custom tag is
generic but must implement `TryFrom<ParsedTag<'a>, Error = ValidationError>` (to convert from parsed
information to the concrete known tag), `IsKnownName` (so that the parser knows if it should
consider calling `try_from` for the parsed tag), `TagInformation` (which is used with the `Writer`),
and `Debug` and `PartialEq` since `known::Tag` implements these.

We can demonstrate the usage of `CustomTag` by implementing it for the custom image media playlist
definition found on the Roku developer website here:
https://developer.roku.com/docs/developer-program/media-playback/trick-mode/hls-and-dash.md
```rust
use m3u8::{
    config::ParsingOptions,
    error::{ValidationError, ValidationErrorValueKind},
    line::HlsLine,
    tag::{
        hls::inf::Inf,
        known::{IsKnownName, ParsedTag, TagInformation},
        value::{ParsedAttributeValue, SemiParsedTagValue},
    },
    Reader,
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
}

// To support multiple custom tags the preferred strategy is to encapsulate each within a single
// enum. For this example I am only demonstrating an implementation for the media playlist tags
// defined in the Roku developer docs.
#[derive(Debug, PartialEq)]
pub enum CustomImageTag<'a> {
    ImagesOnly,
    Tiles(Tiles<'a>),
}
// Here we specialize into our own strongly typed structure what m3u8 was able to parse from the
// input data.
impl<'a> TryFrom<ParsedTag<'a>> for CustomImageTag<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        match tag.name {
            "-X-IMAGES-ONLY" => Ok(CustomImageTag::ImagesOnly),
            "-X-TILES" => Ok(CustomImageTag::Tiles(Tiles::try_from(tag)?)),
            _ => Err(ValidationError::UnexpectedTagName),
        }
    }
}
// This is used to know when m3u8 parsing should consider parsing the information as a known tag.
impl IsKnownName for CustomImageTag<'_> {
    fn is_known_name(name: &str) -> bool {
        match name {
            "-X-IMAGES-ONLY" | "-X-TILES" => true,
            _ => false,
        }
    }
}
// This is used by the m3u8::Writer to handle writing of custom tag implementations.
impl<'a> TagInformation for CustomImageTag<'a> {
    fn name(&self) -> &str {
        match self {
            Self::ImagesOnly => "-X-IMAGES-ONLY",
            Self::Tiles(_) => "-X-TILES",
        }
    }

    fn value(&self) -> SemiParsedTagValue {
        match self {
            Self::ImagesOnly => SemiParsedTagValue::Empty,
            Self::Tiles(tiles) => SemiParsedTagValue::AttributeList(HashMap::from([
                (
                    "RESOLUTION",
                    ParsedAttributeValue::UnquotedString(tiles.original_resolution_str),
                ),
                (
                    "LAYOUT",
                    ParsedAttributeValue::UnquotedString(tiles.original_layout_str),
                ),
                (
                    "DURATION",
                    ParsedAttributeValue::SignedDecimalFloatingPoint(tiles.duration),
                ),
            ])),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Tiles<'a> {
    pub resolution: Resolution,
    pub layout: Resolution,
    pub duration: f64,
    // The original slices are needed for when we convert back to ParsedTagValue in the
    // TagInformation impl.
    original_resolution_str: &'a str,
    original_layout_str: &'a str,
}
impl<'a> TryFrom<ParsedTag<'a>> for Tiles<'a> {
    type Error = ValidationError;

    fn try_from(tag: ParsedTag<'a>) -> Result<Self, Self::Error> {
        let SemiParsedTagValue::AttributeList(attribute_list) = tag.value else {
            return Err(ValidationError::UnexpectedValueType(
                ValidationErrorValueKind::from(&tag.value),
            ));
        };
        let (resolution, r_str) = try_resolution_from("RESOLUTION", &attribute_list)?;
        let (layout, l_str) = try_resolution_from("LAYOUT", &attribute_list)?;
        // Note, `m3u8` is not able to distinguish what *should* be an "integer" vs what *should* be
        // a float without decimal precision, and so a number without decimals will be parsed as the
        // DecimalInteger case. To help extract float values a helper method is provided on
        // ParsedAttributeValue as demonstrated below:
        let Some(duration) = attribute_list
            .get("DURATION")
            .map(ParsedAttributeValue::as_option_f64)
            .flatten()
        else {
            return Err(ValidationError::MissingRequiredAttribute("DURATION"));
        };
        Ok(Self {
            resolution,
            layout,
            duration,
            original_resolution_str: r_str,
            original_layout_str: l_str,
        })
    }
}
fn try_resolution_from<'a, 'b>(
    attr_name: &'static str,
    attribute_list: &'a HashMap<&'b str, ParsedAttributeValue<'b>>,
) -> Result<(Resolution, &'b str), ValidationError> {
    let Some(ParsedAttributeValue::UnquotedString(s)) = attribute_list.get(attr_name) else {
        return Err(ValidationError::MissingRequiredAttribute(attr_name));
    };
    let mut split = s.splitn(2, 'x');
    let Some(Ok(width)) = split.next().map(str::parse::<u64>) else {
        return Err(ValidationError::MissingRequiredAttribute(attr_name));
    };
    let Some(Ok(height)) = split.next().map(str::parse::<u64>) else {
        return Err(ValidationError::MissingRequiredAttribute(attr_name));
    };
    Ok((Resolution { width, height }, s))
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
assert_eq!(
    reader.read_line(),
    Ok(Some(HlsLine::from(CustomImageTag::ImagesOnly)))
);
assert_eq!(
    reader.read_line(),
    Ok(Some(HlsLine::from(CustomImageTag::Tiles(Tiles {
        resolution: Resolution {
            width: 320,
            height: 180
        },
        layout: Resolution {
            width: 5,
            height: 4
        },
        duration: 3.003,
        original_resolution_str: "320x180",
        original_layout_str: "5x4"
    }))))
);
assert_eq!(
    reader.read_line(),
    Ok(Some(HlsLine::from(Inf::new(
        60.06,
        String::from(
            "Indicates 20 320x180 images (laid out 5x4) each to be displayed for 3.003s"
        )
    ))))
);
assert_eq!(reader.read_line(), Ok(Some(HlsLine::Uri("image.1.jpeg"))));
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
for the rest of the line. Running locally as of commit `6fcc38a67bf0eee0769b7e85f82599d1da6eb56d`
the following benchmark shows that when parsing a large playlist, including all tags in the parse is
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
        HlsLine::KnownTag(tag) => match tag {
            known::Tag::Hls(tag) => match tag {
                hls::Tag::Inf(mut inf) => {
                    if added_hello {
                        inf.set_title(String::from("World!"));
                    } else {
                        inf.set_title(String::from("Hello,"));
                        added_hello = true;
                    }
                    writer.write_hls_tag(hls::Tag::Inf(inf)).unwrap()
                }
                tag => writer.write_hls_tag(tag).unwrap(),
            },
            known::Tag::Custom(tag) => writer.write_custom_tag(tag).unwrap(),
        },
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
