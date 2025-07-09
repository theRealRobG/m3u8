use criterion::{Criterion, criterion_group, criterion_main};
use m3u8::{
    Reader, Writer,
    config::ParsingOptionsBuilder,
    line::{self, HlsLine},
    tag::{hls, known},
};
use std::hint::black_box;

const LONG_MEDIA_PLAYLIST: &'static str = include_str!("long_media_playlist.m3u8");

macro_rules! reader_match {
    (MUTATE, $reader:ident, $writer:ident) => {
        match black_box($reader.read_line()) {
            Ok(Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(mut tag))))) => {
                tag.set_title(black_box(String::from("TEST")));
                black_box($writer.write_line(HlsLine::from(tag)).unwrap());
            }
            Ok(Some(line)) => {
                black_box($writer.write_line(line).unwrap());
            }
            Ok(None) => break,
            Err(e) => panic!("{}", e),
        }
    };
    (NO_MUTATE, $reader:ident, $writer:ident) => {
        match black_box($reader.read_line()) {
            Ok(Some(line)) => {
                black_box($writer.write_line(line).unwrap());
            }
            Ok(None) => break,
            Err(e) => panic!("{}", e),
        }
    };
    (NO_WRITE, $reader:ident) => {
        match black_box($reader.read_line()) {
            Ok(Some(_)) => (),
            Ok(None) => break,
            Err(e) => panic!("{}", e),
        }
    };
}

macro_rules! reader_bench {
    ($c:ident, $id:literal, $options:ident, $method:ident $([$as_bytes:ident])?, $mutate:ident) => {
        $c.bench_function($id, |b| {
            b.iter(|| {
                let options = $options.clone();
                let mut reader = Reader::$method((black_box(LONG_MEDIA_PLAYLIST$(.$as_bytes())?)), options);
                let mut writer = Writer::new(Vec::new());
                loop {
                    reader_match!($mutate, reader, writer)
                }
            });
        });
    };
    (NO_WRITE, $c:ident, $id:literal, $options:ident, $method:ident $([$as_bytes:ident])?) => {
        $c.bench_function($id, |b| {
            b.iter(|| {
                let options = $options.clone();
                let mut reader = Reader::$method(black_box(LONG_MEDIA_PLAYLIST$(.$as_bytes())?), options);
                loop {
                    reader_match!(NO_WRITE, reader)
                }
            });
        });
    };
}

pub fn criterion_benchmark(c: &mut Criterion) {
    // let daterange_str = r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#;
    // let daterange_opt = ParsingOptionsBuilder::new()
    //     .with_parsing_for_all_tags()
    //     .build();

    // // Benchmark our own parsing of EXT-X-DATERANGE
    // assert!(line::parse(daterange_str, &daterange_opt).is_ok());
    // c.bench_function("Bench EXT-X-DATERANGE", |b| {
    //     b.iter(|| line::parse(daterange_str, &daterange_opt));
    // });

    // Check some longer parsing of a whole manifest, once with all tags parsing bench, and once
    // no tag parsing, to compare the performance difference.
    let playlist_all_tags_parse_options = ParsingOptionsBuilder::new()
        .with_parsing_for_all_tags()
        .build();
    let playlist_no_tags_parse_options = ParsingOptionsBuilder::new().build();
    let playlist_lines = LONG_MEDIA_PLAYLIST.lines();
    for line in playlist_lines {
        assert!(line::parse(line, &playlist_all_tags_parse_options).is_ok());
        assert!(line::parse(line, &playlist_no_tags_parse_options).is_ok());
    }
    let playlist = hls_m3u8::MediaPlaylist::try_from(LONG_MEDIA_PLAYLIST).unwrap();
    assert_eq!(4, playlist.target_duration.as_secs());
    assert_eq!(541647, playlist.media_sequence);
    // no write benches
    reader_bench!(
        NO_WRITE,
        c,
        "Large playlist, all tags, using Reader::from_str, no writing",
        playlist_all_tags_parse_options,
        from_str
    );
    reader_bench!(
        NO_WRITE,
        c,
        "Large playlist, no tags, using Reader::from_str, no writing",
        playlist_no_tags_parse_options,
        from_str
    );
    // Quick test against competition
    c.bench_function("Large playlist, using hls_m3u8, no writing", |b| {
        b.iter(|| {
            let _ = black_box(
                hls_m3u8::MediaPlaylist::try_from(black_box(LONG_MEDIA_PLAYLIST)).unwrap(),
            );
        });
    });
    // reader_bench!(
    //     NO_WRITE,
    //     c,
    //     "Large playlist, all tags, using Reader::from_reader, no writing",
    //     playlist_all_tags_parse_options,
    //     from_reader[as_bytes]
    // );
    // reader_bench!(
    //     NO_WRITE,
    //     c,
    //     "Large playlist, no tags, using Reader::from_reader, no writing",
    //     playlist_no_tags_parse_options,
    //     from_reader[as_bytes]
    // );
    // // from_str benches
    // reader_bench!(
    //     c,
    //     "Large playlist, all tags, Reader::from_str and Writer, no mutation",
    //     playlist_all_tags_parse_options,
    //     from_str,
    //     NO_MUTATE
    // );
    // reader_bench!(
    //     c,
    //     "Large playlist, all tags, Reader::from_str and Writer, mutation on EXTINF",
    //     playlist_all_tags_parse_options,
    //     from_str,
    //     MUTATE
    // );
    // reader_bench!(
    //     c,
    //     "Large playlist, no tags, Reader::from_str and Writer, no mutation",
    //     playlist_no_tags_parse_options,
    //     from_str,
    //     NO_MUTATE
    // );
    // // try_from_reader benches
    // reader_bench!(
    //     c,
    //     "Large playlist, all tags, Reader::from_reader and Writer, no mutation",
    //     playlist_all_tags_parse_options,
    //     from_reader[as_bytes],
    //     NO_MUTATE
    // );
    // reader_bench!(
    //     c,
    //     "Large playlist, all tags, Reader::from_reader and Writer, mutation on EXTINF",
    //     playlist_all_tags_parse_options,
    //     from_reader[as_bytes],
    //     MUTATE
    // );
    // reader_bench!(
    //     c,
    //     "Large playlist, no tags, Reader::from_reader and Writer, no mutation",
    //     playlist_no_tags_parse_options,
    //     from_reader[as_bytes],
    //     NO_MUTATE
    // );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
