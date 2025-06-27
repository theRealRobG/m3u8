# M3U8

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
for the rest of the line. Running locally as of commit `c28f2e776f03a446af367a153dcb2d95764186cc`
the following benchmark shows that when parsing a large playlist, including all tags in the parse is
about 4x slower than including no tags in the parse (`2.0689 ms` vs `495.53 µs` == `0.49553 ms`).
```sh
Benchmarking Bench large playlist with full parsing on all known tags: Collecting 100 samples in estimated 5.1923 s (2500 iterations
Bench large playlist with full parsing on all known tags
                        time:   [2.0669 ms 2.0689 ms 2.0719 ms]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) high mild
  8 (8.00%) high severe

Benchmarking Bench large playlist with no known tags being fully parsed: Collecting 100 samples in estimated 5.0151 s (10k iteration
Bench large playlist with no known tags being fully parsed
                        time:   [494.48 µs 495.53 µs 496.72 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
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
