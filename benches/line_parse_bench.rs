use crate::{comparison_daterange_parser::parse, manifests::LONG_MEDIA_PLAYLIST};
use criterion::{Criterion, criterion_group, criterion_main};
use m3u8::{config::ParsingOptionsBuilder, line};

mod comparison_daterange_parser;
mod manifests;

pub fn criterion_benchmark(c: &mut Criterion) {
    let daterange_str = r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#;
    let daterange_opt = ParsingOptionsBuilder::new()
        .with_parsing_for_all_tags()
        .build();

    // Benchmark our own parsing of EXT-X-DATERANGE
    assert!(line::parse(daterange_str, &daterange_opt).is_ok());
    c.bench_function("Bench EXT-X-DATERANGE", |b| {
        b.iter(|| line::parse(daterange_str, &daterange_opt));
    });

    // In a separate project I needed to parse out SCTE35-OUT from a daterange tag. Back then I
    // wrote a really ugly parser with very focused logic for parsing of just the information I
    // needed from the EXT-X-DATERANGE tag. The parsing code is at least performant and so I'm going
    // to try and beat it with the parser in this project (or at least equal it).
    assert!(parse(daterange_str).is_some());
    c.bench_function("Bench EXT-X-DATERANGE with dumb comparison parser", |b| {
        b.iter(|| parse(daterange_str));
    });

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
    c.bench_function(
        "Bench large playlist with full parsing on all known tags",
        |b| {
            b.iter(|| {
                let playlist_lines = LONG_MEDIA_PLAYLIST.lines();
                for line in playlist_lines {
                    let _ = line::parse(line, &playlist_all_tags_parse_options);
                }
            });
        },
    );
    c.bench_function(
        "Bench large playlist with no known tags being fully parsed",
        |b| {
            b.iter(|| {
                let playlist_lines = LONG_MEDIA_PLAYLIST.lines();
                for line in playlist_lines {
                    let _ = line::parse(line, &playlist_no_tags_parse_options);
                }
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
