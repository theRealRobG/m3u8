pub mod config;
pub mod date;
pub mod error;
pub mod line;
mod reader;
pub mod tag;
mod utils;
mod writer;

pub use reader::Reader;
pub use writer::Writer;

// This allows the Rust compiler to validate any Rust snippets in my README, which seems like a very
// cool trick. I saw this technique in clap-rs/clap, for example:
// https://github.com/clap-rs/clap/blob/4d7ab1483cd0f0849668d274aa2fb6358872eca9/clap_complete_nushell/src/lib.rs#L239-L241
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
