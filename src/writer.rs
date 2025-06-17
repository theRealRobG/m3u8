use crate::line::HlsLine;
use std::io::{self, Write};

#[derive(Clone)]
pub struct Writer<W>
where
    W: Write,
{
    /// underlying writer
    writer: W,
}

impl<W> Writer<W>
where
    W: Write,
{
    /// Creates a `Writer` from a generic writer.
    pub const fn new(inner: W) -> Writer<W> {
        Writer { writer: inner }
    }

    /// Consumes this `Writer`, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Get a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Get a reference to the underlying writer.
    pub const fn get_ref(&self) -> &W {
        &self.writer
    }

    pub fn write_line<'a, Line: Into<HlsLine<'a>>>(&mut self, line: Line) -> io::Result<()> {
        match line.into() {
            HlsLine::Blank => self.writer.write_all(b"\n"),
            HlsLine::Comment(c) => {
                self.writer.write_all(b"#")?;
                self.writer.write_all(c.as_bytes())
            }
            HlsLine::Uri(u) => self.writer.write_all(u.as_bytes()),
            HlsLine::UnknownTag(t) => self.writer.write_all(t.as_str().as_bytes()),
            HlsLine::KnownTag(t) => todo!(),
        }
    }
}
