#![warn(missing_docs)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/theRealRobG/m3u8/refs/heads/main/quick-m3u8-logo.ico"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/theRealRobG/m3u8/refs/heads/main/quick-m3u8-logo.svg"
)]

//! # quick-m3u8
//!
//! quick-m3u8 aims to be a high performance [M3U8] reader and writer. The API is event-driven (like
//! SAX for XML) rather than serializing into a complete object model (like DOM for XML). The syntax
//! (and name) is inspired by [quick-xml]. The [`crate::Reader`] attempts to be almost zero-copy
//! while still supporting mutation of the parsed data by utilizing [`std::borrow::Cow`] (Copy On
//! Write) as much as possible.
//!
//! When parsing M3U8 data quick-m3u8 aims to be very lenient when it comes to validation. The
//! philosophy is that the library does not want to get in the way of extracting meaningful
//! information from the input data. If the library took a strict approach to validating adherence
//! to all of the HLS specification, then there could be cases where a playlist is rejected, but a
//! client may have accepted it. For example, consider the requirement for segment duration rounding
//! declared by the [EXT-X-TARGETDURATION] tag:
//! > The EXTINF duration of each Media Segment in a Playlist file, when rounded to the nearest
//! > integer, MUST be less than or equal to the Target Duration.
//!
//! Despite this requirement, many players will tolerate several long-running segments in a playlist
//! just fine, so interpreting this rule very strictly could lead to the manifest being rejected
//! before it even reaches the video player (assuming this library is being used in a server
//! implementation).
//!
//! Therefore, validating the sanity of the parsed values is deliberately left to the user of the
//! library. Some examples of values that are not validated are:
//! * URI lines are not validated as being valid URIs (any line that isn't blank and does not start
//!   with `#` is considered to be a URI line).
//! * Enumerated strings (within [attribute-lists]) are not validated to have no whitespace.
//! * A tag with a known name that fails the `TryFrom<ParsedTag>` conversion does not fail the line
//!   and instead is presented as [`crate::tag::unknown::Tag`].
//!
//! With that being said, the library does validate proper UTF-8 conversion from `&[u8]` input,
//! enumerated strings are wrapped in a convenience type ([`crate::tag::hls::EnumeratedString`])
//! that exposes strongly typed enumerations when the value is valid, and the `TryFrom<ParsedTag>`
//! implementation for all of the HLS tags supported by quick-m3u8 ensure that the required
//! attributes are present.
//!
//! # Usage
//!
//! Usage is broken up into reading and writing.
//!
//! ## Reading
//!
//! The main entry point for using the library is the [`crate::Reader`]. This provides an interface
//! for reading lines from an input data source. For example, consider the [Simple Media Playlist]:
//! ```
//! const EXAMPLE_MANIFEST: &str = r#"#EXTM3U
//! #EXT-X-TARGETDURATION:10
//! #EXT-X-VERSION:3
//! #EXTINF:9.009,
//! first.ts
//! #EXTINF:9.009,
//! second.ts
//! #EXTINF:3.003,
//! third.ts
//! #EXT-X-ENDLIST
//! "#;
//! ```
//! We can use the Reader to read information about each line in the playlist as such:
//! ```
//! # use m3u8::{
//! #     config::ParsingOptionsBuilder,
//! #     line::HlsLine,
//! #     tag::hls::{
//! #         endlist::Endlist, inf::Inf, m3u::M3u, targetduration::Targetduration, version::Version,
//! #     },
//! #     Reader,
//! # };
//! #
//! # const EXAMPLE_MANIFEST: &str = r#"#EXTM3U
//! # #EXT-X-TARGETDURATION:10
//! # #EXT-X-VERSION:3
//! # #EXTINF:9.009,
//! # first.ts
//! # #EXTINF:9.009,
//! # second.ts
//! # #EXTINF:3.003,
//! # third.ts
//! # #EXT-X-ENDLIST
//! # "#;
//! let mut reader = Reader::from_str(
//!     EXAMPLE_MANIFEST,
//!     ParsingOptionsBuilder::new()
//!         .with_parsing_for_all_tags()
//!         .build(),
//! );
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(M3u))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Targetduration::new(10)))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Version::new(3)))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(9.009, "")))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::uri("first.ts"))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(9.009, "")))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::uri("second.ts"))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Inf::new(3.003, "")))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::uri("third.ts"))));
//! assert_eq!(reader.read_line(), Ok(Some(HlsLine::from(Endlist))));
//! assert_eq!(reader.read_line(), Ok(None));
//! ```
//!
//! The example is basic but demonstrates a few points already. Firstly, `HlsLine` (the result of
//! `read_line`) is an `enum`, which is in line with [Section 4.1] that states:
//! > Each line is a URI, is blank, or starts with the character '#'. Lines that start with the
//! > character '#' are either comments or tags. Tags begin with #EXT.
//!
//! Each case of the [`crate::line::HlsLine`] is documented thoroughly; however, it's worth
//! mentioning that in addition to what the HLS specification defines, the library also allows for
//! `UnknownTag` (which is a tag, based on the `#EXT` prefix, but not one that we know about), and
//! also allows `CustomTag`. The [`crate::tag::known::CustomTag`] is a means for the user of the
//! library to define support for their own custom tag specification in addition to what is provided
//! via the HLS specification. The documentation for `CustomTag` provides more details on how that
//! is achieved.
//!
//! The `Reader` also takes a configuration that allows the user to select what HLS tags the reader
//! should parse. [`crate::config::ParsingOptions`] provides more details, but in short, better
//! performance can be squeezed out by only parsing the tags that you need.
//!
//! ## Writing
//!
//! The other component to quick-m3u8 is [`crate::Writer`]. This allows the user to write to a given
//! [`std::io::Write`] the parsed (or constructed) HLS lines. It should be noted that when writing,
//! if no mutation of a tag has occurred, then the original reference slice of the line will be
//! used. This allows us to avoid unnecessary allocations.
//!
//! A common use-case for reading and then writing is to modify a HLS playlist, perhaps in transit,
//! in a proxy layer. Below is a toy example; however, the repo benchmark demonstrates a more
//! complex example of how one may implement a HLS delta update (acting as a proxy layer).
//! ```
//! # use m3u8::{
//! #     config::ParsingOptions,
//! #     line::HlsLine,
//! #     tag::{hls, known},
//! #     Reader, Writer,
//! # };
//! # use std::io;
//! let input_lines = concat!(
//!     "#EXTINF:4.00008,\n",
//!     "fileSequence268.mp4\n",
//!     "#EXTINF:4.00008,\n",
//!     "fileSequence269.mp4\n",
//! );
//! let mut reader = Reader::from_str(input_lines, ParsingOptions::default());
//! let mut writer = Writer::new(Vec::new());
//!
//! let mut added_hello = false;
//! while let Ok(Some(line)) = reader.read_line() {
//!     match line {
//!         HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(mut inf))) => {
//!             if added_hello {
//!                 inf.set_title(" World!");
//!             } else {
//!                 inf.set_title(" Hello,");
//!                 added_hello = true;
//!             }
//!             writer.write_hls_tag(hls::Tag::Inf(inf))?
//!         }
//!         line => writer.write_line(line)?,
//!     };
//! }
//!
//! let expected_output_lines = concat!(
//!     "#EXTINF:4.00008, Hello,\n",
//!     "fileSequence268.mp4\n",
//!     "#EXTINF:4.00008, World!\n",
//!     "fileSequence269.mp4\n",
//! );
//! assert_eq!(
//!     expected_output_lines,
//!     String::from_utf8_lossy(&writer.into_inner())
//! );
//! # Ok::<(), io::Error>(())
//! ```
//!
//! [M3U8]: https://datatracker.ietf.org/doc/draft-pantos-hls-rfc8216bis/
//! [quick-xml]: https://crates.io/crates/quick-xml
//! [EXT-X-TARGETDURATION]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.4.3.1
//! [attribute-lists]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.2
//! [Simple Media Playlist]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-9.1
//! [Section 4.1]: https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-4.1

pub mod config;
pub mod date;
pub mod error;
pub mod line;
mod reader;
pub mod tag;
mod utils;
mod writer;

pub use line::HlsLine;
pub use reader::Reader;
pub use writer::Writer;

// This allows the Rust compiler to validate any Rust snippets in my README, which seems like a very
// cool trick. I saw this technique in clap-rs/clap, for example:
// https://github.com/clap-rs/clap/blob/4d7ab1483cd0f0849668d274aa2fb6358872eca9/clap_complete_nushell/src/lib.rs#L239-L241
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
