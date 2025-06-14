use std::slice::Iter;

use crate::manifests::LONG_MEDIA_PLAYLIST;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use m3u8::{
    config::ParsingOptionsBuilder,
    line::{self, ParsedLineSlice},
};

mod manifests;

fn up_to_end_1(bytes: &[u8]) -> &[u8] {
    let index = bytes.iter().enumerate().find_map(|(i, b)| {
        if *b == b'\n' || *b == b'\r' {
            Some(i)
        } else {
            None
        }
    });
    match index {
        Some(index) => &bytes[..index],
        None => &bytes,
    }
}

fn up_to_end_2(bytes: &[u8]) -> &[u8] {
    let mut enumerated = bytes.iter().enumerate();
    loop {
        match enumerated.next() {
            Some((i, b'\n')) => return &bytes[..i],
            Some((i, b'\r')) => return &bytes[..i],
            None => return &bytes,
            _ => (),
        }
    }
}

fn up_to_end_3(bytes: &[u8]) -> &[u8] {
    let mut iterations = 0usize;
    let mut bytes_iter = bytes.iter();
    loop {
        iterations += 1;
        match bytes_iter.next() {
            Some(b'\n') => return &bytes[..(iterations - 1)],
            Some(b'\r') => return &bytes[..(iterations - 1)],
            None => return &bytes,
            _ => (),
        }
    }
}

fn up_to_end_4(bytes: &[u8]) -> &[u8] {
    let found = bytes
        .iter()
        .enumerate()
        .find(|(_, b)| **b == b'\n' || **b == b'\r');
    match found {
        Some((index, _)) => &bytes[..index],
        None => &bytes,
    }
}

pub(crate) fn str_from(bytes: &[u8]) -> &str {
    unsafe {
        // SAFETY: The input for bytes is always &str in this project, and I only break on single
        // byte characters, so this is safe to do unchecked.
        std::str::from_utf8_unchecked(bytes)
    }
}

pub fn validate_carriage_return_bytes(bytes: &mut Iter<'_, u8>) -> Result<(), &'static str> {
    let Some(b'\n') = bytes.next() else {
        return Err("Unexpected carriage return without following line feed");
    };
    Ok(())
}

fn take_until_end_of_bytes<'a>(
    mut bytes: Iter<'a, u8>,
) -> Result<ParsedLineSlice<'a, &'a str>, &'static str> {
    let input = bytes.as_slice();
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        match bytes.next() {
            None => {
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: None,
                });
            }
            Some(b'\r') => {
                validate_carriage_return_bytes(&mut bytes)?;
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: Some(str_from(bytes.as_slice())),
                });
            }
            Some(b'\n') => {
                return Ok(ParsedLineSlice {
                    parsed: str_from(&input[..(iterations - 1)]),
                    remaining: Some(str_from(bytes.as_slice())),
                });
            }
            _ => (),
        }
    }
}

fn take_until_end_of_bytes_2<'a>(
    bytes: Iter<'a, u8>,
) -> Result<ParsedLineSlice<'a, &'a str>, &'static str> {
    let input = bytes.as_slice();
    let eol = bytes
        .enumerate()
        .find(|(_, b)| **b == b'\n' || **b == b'\r');
    match eol {
        Some((index, b'\n')) => Ok(ParsedLineSlice {
            parsed: str_from(&input[..index]),
            remaining: Some(str_from(&input[(index + 1)..])),
        }),
        Some((index, b'\r')) => {
            let Some(b'\n') = input.iter().nth(index + 1) else {
                return Err("Unexpected carriage return without following line feed");
            };
            Ok(ParsedLineSlice {
                parsed: str_from(&input[..index]),
                remaining: Some(str_from(&input[(index + 2)..])),
            })
        }
        None => Ok(ParsedLineSlice {
            parsed: str_from(input),
            remaining: None,
        }),
        _ => panic!("Impossible"),
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let daterange_str_lf = concat!(
        r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#,
        "\n",
        r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#
    ).as_bytes();

    c.bench_function("up_to_end_1", |b| {
        b.iter(|| up_to_end_1(daterange_str_lf));
    });
    c.bench_function("up_to_end_2", |b| {
        b.iter(|| up_to_end_2(daterange_str_lf));
    });
    c.bench_function("up_to_end_3", |b| {
        b.iter(|| up_to_end_3(daterange_str_lf));
    });
    c.bench_function("up_to_end_4", |b| {
        b.iter(|| up_to_end_4(daterange_str_lf));
    });
    let bytes_iter = daterange_str_lf.iter();
    c.bench_with_input(
        BenchmarkId::new("take_until_end_of_bytes", "bytes iterator"),
        &bytes_iter,
        |b, input| {
            b.iter(|| take_until_end_of_bytes(input.clone()));
        },
    );
    c.bench_with_input(
        BenchmarkId::new("take_until_end_of_bytes_2", "bytes iterator"),
        &bytes_iter,
        |b, input| {
            b.iter(|| take_until_end_of_bytes_2(input.clone()));
        },
    );
}

// pub fn criterion_benchmark(c: &mut Criterion) {
//     let daterange_str = r#"#EXT-X-DATERANGE:ID="0x30-5-1749409044",START-DATE="2025-06-08T18:57:25Z",PLANNED-DURATION=60.000,SCTE35-OUT=0xfc303e0000000000000000b00506fe2587ed930028022643554549000000057fff00005265c00e1270636b5f455030333638373336353030313230010c6ad0769a"#;
//     let daterange_opt = ParsingOptionsBuilder::new()
//         .with_parsing_for_all_tags()
//         .build();

//     // Benchmark our own parsing of EXT-X-DATERANGE
//     assert!(line::parse(daterange_str, &daterange_opt).is_ok());
//     c.bench_function("Bench EXT-X-DATERANGE", |b| {
//         b.iter(|| line::parse(daterange_str, &daterange_opt));
//     });

//     // Check some longer parsing of a whole manifest, once with all tags parsing bench, and once
//     // no tag parsing, to compare the performance difference.
//     let playlist_all_tags_parse_options = ParsingOptionsBuilder::new()
//         .with_parsing_for_all_tags()
//         .build();
//     let playlist_no_tags_parse_options = ParsingOptionsBuilder::new().build();
//     let playlist_lines = LONG_MEDIA_PLAYLIST.lines();
//     for line in playlist_lines {
//         assert!(line::parse(line, &playlist_all_tags_parse_options).is_ok());
//         assert!(line::parse(line, &playlist_no_tags_parse_options).is_ok());
//     }
//     c.bench_function(
//         "Bench large playlist with full parsing on all known tags",
//         |b| {
//             b.iter(|| {
//                 let playlist_lines = LONG_MEDIA_PLAYLIST.lines();
//                 for line in playlist_lines {
//                     let _ = line::parse(line, &playlist_all_tags_parse_options);
//                 }
//             });
//         },
//     );
//     c.bench_function(
//         "Bench large playlist with no known tags being fully parsed",
//         |b| {
//             b.iter(|| {
//                 let playlist_lines = LONG_MEDIA_PLAYLIST.lines();
//                 for line in playlist_lines {
//                     let _ = line::parse(line, &playlist_no_tags_parse_options);
//                 }
//             });
//         },
//     );
// }

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
