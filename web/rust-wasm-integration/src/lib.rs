use fluent_uri::{Uri, UriRef};
use m3u8::{
    Reader, Writer,
    config::ParsingOptionsBuilder,
    line::HlsLine,
    tag::{hls, known, unknown},
};
use std::{borrow::Cow, io::Write};
use wasm_bindgen::prelude::*;

macro_rules! handle_uri_attr {
    ($tag_ident:ident, URI OPTIONAL, $writer:tt, $tag:tt, $base_url:tt) => {{
        if let Some(uri) = $tag.uri() {
            if let Some(anchor) = anchor_string(uri, &$base_url) {
                $tag.set_uri(Some(anchor));
            }
        }
        write_hls_tag(&mut $writer, hls::Tag::$tag_ident($tag))?
    }};
    ($tag_ident:ident, URI REQUIRED, $writer:tt, $tag:tt, $base_url:tt) => {{
        let uri = $tag.uri();
        if let Some(anchor) = anchor_string(uri, &$base_url) {
            $tag.set_uri(anchor);
        }
        write_hls_tag(&mut $writer, hls::Tag::$tag_ident($tag))?
    }};
}

#[wasm_bindgen]
pub fn m3u8_to_html(m3u8: &str, base_url: &str) -> Result<String, JsError> {
    console_error_panic_hook::set_once();
    let base_url = Uri::parse(base_url)?;
    let mut reader = Reader::from_str(
        m3u8,
        ParsingOptionsBuilder::new()
            .with_parsing_for_map()
            .with_parsing_for_part()
            .with_parsing_for_preload_hint()
            .with_parsing_for_rendition_report()
            .with_parsing_for_media()
            .with_parsing_for_i_frame_stream_inf()
            .with_parsing_for_session_data()
            .with_parsing_for_content_steering()
            .build(),
    );
    let mut writer = Writer::new(Vec::new());

    while let Ok(Some(line)) = reader.read_line() {
        match line {
            HlsLine::Uri(uri) => write_uri(&mut writer, uri, &base_url)?,
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Map(mut tag))) => {
                handle_uri_attr!(Map, URI REQUIRED, writer, tag, base_url)
            }
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Part(mut tag))) => {
                handle_uri_attr!(Part, URI REQUIRED, writer, tag, base_url)
            }
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::PreloadHint(mut tag))) => {
                handle_uri_attr!(PreloadHint, URI REQUIRED, writer, tag, base_url)
            }
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::RenditionReport(mut tag))) => {
                handle_uri_attr!(RenditionReport, URI REQUIRED, writer, tag, base_url)
            }
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Media(mut tag))) => {
                handle_uri_attr!(Media, URI OPTIONAL, writer, tag, base_url)
            }
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::IFrameStreamInf(mut tag))) => {
                handle_uri_attr!(IFrameStreamInf, URI REQUIRED, writer, tag, base_url)
            }
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::SessionData(mut tag))) => {
                handle_uri_attr!(SessionData, URI OPTIONAL, writer, tag, base_url)
            }
            HlsLine::KnownTag(known::Tag::Hls(hls::Tag::ContentSteering(mut tag))) => {
                let uri = tag.server_uri();
                if let Some(anchor) = anchor_string(uri, &base_url) {
                    tag.set_server_uri(anchor);
                }
                // writer.write_hls_tag(hls::Tag::ContentSteering(tag))?
                write_hls_tag(&mut writer, hls::Tag::ContentSteering(tag))?;
            }
            line => write_line(&mut writer, line, &base_url)?,
        };
    }

    Ok(String::from_utf8_lossy(&writer.into_inner()).to_string())
}

fn anchor_str<'a>(uri: &'a str, base: &Uri<&str>, class: &'static str) -> Cow<'a, str> {
    if let Ok(absolute_url) = UriRef::parse(uri)
        .map_err(JsError::from)
        .and_then(|uri| uri.resolve_against(&base).map_err(JsError::from))
    {
        Cow::Owned(format!(
            "<a class=\"{}\" href=\"#{}\">{}</a>",
            class,
            absolute_url.as_str(),
            uri,
        ))
    } else {
        Cow::Borrowed(uri)
    }
}

fn anchor_string(uri: &str, base: &Uri<&str>) -> Option<String> {
    match anchor_str(uri, base, "attr") {
        Cow::Borrowed(_) => None,
        Cow::Owned(s) => Some(s),
    }
}

macro_rules! write_span_open {
    ($writer:tt, $tag:tt $(, $as_str:ident)?) => {
        let class = format!("EXT{}", $tag.name()$(.$as_str())?);
        $writer.get_mut().write_all(b"<span class=\"tag ")?;
        $writer.get_mut().write_all(class.as_bytes())?;
        $writer.get_mut().write_all(b"\">")?;
    };
}

macro_rules! write_span_close {
    ($writer:tt) => {
        $writer.get_mut().write_all(b"</span>")?;
    };
}

fn write_hls_tag<W: Write>(writer: &mut Writer<W>, tag: hls::Tag) -> Result<(), JsError> {
    write_span_open!(writer, tag, as_str);
    writer.write_hls_tag(tag)?;
    write_span_close!(writer);
    Ok(())
}

fn write_unknown_tag<W: Write>(writer: &mut Writer<W>, tag: unknown::Tag) -> Result<(), JsError> {
    write_span_open!(writer, tag);
    writer.write_line(HlsLine::from(tag))?;
    write_span_close!(writer);
    Ok(())
}

fn write_uri<W: Write>(writer: &mut Writer<W>, uri: &str, base: &Uri<&str>) -> Result<(), JsError> {
    writer.write_uri(&anchor_str(uri, base, "uri"))?;
    Ok(())
}

fn write_comment<W: Write>(writer: &mut Writer<W>, comment: &str) -> Result<(), JsError> {
    writer.get_mut().write_all(b"<span class=\"comment\">")?;
    writer.write_comment(comment)?;
    write_span_close!(writer);
    Ok(())
}

fn write_blank<W: Write>(writer: &mut Writer<W>) -> Result<(), JsError> {
    writer.write_blank()?;
    Ok(())
}

fn write_line<W: Write>(
    writer: &mut Writer<W>,
    line: HlsLine,
    base: &Uri<&str>,
) -> Result<(), JsError> {
    match line {
        HlsLine::KnownTag(tag) => match tag {
            known::Tag::Hls(tag) => write_hls_tag(writer, tag),
            known::Tag::Custom(_) => Ok(()), // No custom tags set so OK to ignore
        },
        HlsLine::UnknownTag(tag) => write_unknown_tag(writer, tag),
        HlsLine::Comment(comment) => write_comment(writer, comment),
        HlsLine::Uri(uri) => write_uri(writer, uri, base),
        HlsLine::Blank => write_blank(writer),
    }
}
