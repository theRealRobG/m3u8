use std::io::{self, BufRead};

use crate::{
    config::ParsingOptions,
    line::{HlsLine, ParsedLineSlice, parse},
};

pub struct Reader<R> {
    inner: R,
    options: ParsingOptions,
}

impl<'a> Reader<&'a str> {
    pub fn from_str(str: &'a str, options: ParsingOptions) -> Self {
        Self {
            inner: str,
            options,
        }
    }

    pub fn read_line(&mut self) -> Result<Option<HlsLine>, &'static str> {
        if self.inner.is_empty() {
            return Ok(None);
        };
        let ParsedLineSlice { parsed, remaining } = parse(self.inner, &self.options)?;
        std::mem::swap(&mut self.inner, &mut remaining.unwrap_or_default());
        Ok(Some(parsed))
    }
}

impl<R: BufRead> Reader<R> {
    pub fn from_reader(reader: R, options: ParsingOptions) -> Self {
        Self {
            inner: reader,
            options,
        }
    }

    pub fn read_line_into<'b>(
        &mut self,
        buf: &'b mut String,
    ) -> Result<Option<HlsLine<'b>>, &'static str> {
        let available = loop {
            match self.inner.fill_buf() {
                Ok(n) if n.is_empty() => return Ok(None),
                Ok(n) => break std::str::from_utf8(n).map_err(|_| "Not UTF-8")?,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => (),
                Err(_) => return Err("IO read error"),
            }
        };
        let total_len = available.len();
        buf.clear();
        buf.push_str(available);
        let ParsedLineSlice { parsed, remaining } = parse(buf.as_str(), &self.options)?;
        let remaining_len = remaining.map_or(0, |s| s.len());
        self.inner.consume(total_len - remaining_len);
        Ok(Some(parsed))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::ParsingOptionsBuilder,
        tag::hls::{
            endlist::Endlist, inf::Inf, m3u::M3u, targetduration::Targetduration, version::Version,
        },
    };

    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! reader_test {
        ($reader:tt, $method:tt $(, $buf:ident)?) => {
            for i in 0..=10 {
                let line = $reader.$method($(&mut $buf)?).unwrap();
                match i {
                    0 => assert_eq!(Some(HlsLine::from(M3u)), line),
                    1 => assert_eq!(Some(HlsLine::from(Targetduration::new(10))), line),
                    2 => assert_eq!(Some(HlsLine::from(Version::new(3))), line),
                    3 => assert_eq!(Some(HlsLine::from(Inf::new(9.009, String::new()))), line),
                    4 => assert_eq!(
                        Some(HlsLine::new_uri("http://media.example.com/first.ts")),
                        line
                    ),
                    5 => assert_eq!(Some(HlsLine::from(Inf::new(9.009, String::new()))), line),
                    6 => assert_eq!(
                        Some(HlsLine::new_uri("http://media.example.com/second.ts")),
                        line
                    ),
                    7 => assert_eq!(Some(HlsLine::from(Inf::new(3.003, String::new()))), line),
                    8 => assert_eq!(
                        Some(HlsLine::new_uri("http://media.example.com/third.ts")),
                        line
                    ),
                    9 => assert_eq!(Some(HlsLine::from(Endlist)), line),
                    10 => assert_eq!(None, line),
                    _ => panic!(),
                }
            }
        };
    }

    #[test]
    fn reader_from_str_should_read_as_expected() {
        let mut reader = Reader::from_str(
            EXAMPLE_MANIFEST,
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
        );
        reader_test!(reader, read_line);
    }

    #[test]
    fn reader_from_buf_read_should_read_as_expected() {
        let inner = EXAMPLE_MANIFEST.as_bytes();
        let mut reader = Reader::from_reader(
            inner,
            ParsingOptionsBuilder::new()
                .with_parsing_for_all_tags()
                .build(),
        );
        let mut string = String::new();
        reader_test!(reader, read_line_into, string);
    }
}

#[cfg(test)]
/// https://datatracker.ietf.org/doc/html/draft-pantos-hls-rfc8216bis-17#section-9.1
const EXAMPLE_MANIFEST: &str = r#"#EXTM3U
#EXT-X-TARGETDURATION:10
#EXT-X-VERSION:3
#EXTINF:9.009,
http://media.example.com/first.ts
#EXTINF:9.009,
http://media.example.com/second.ts
#EXTINF:3.003,
http://media.example.com/third.ts
#EXT-X-ENDLIST
"#;
