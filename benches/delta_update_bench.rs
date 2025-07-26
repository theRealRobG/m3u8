use criterion::{Criterion, criterion_group, criterion_main};
use m3u8::{
    Reader, Writer,
    config::ParsingOptionsBuilder,
    line::HlsLine,
    tag::{
        hls::{
            self, TagName, TagType, server_control::ServerControl, skip::Skip, version::Version,
        },
        known,
    },
};
use m3u8_rs::{ExtTag, parse_media_playlist_res};
use pretty_assertions::assert_eq;
use std::{error::Error, hint::black_box, io::Write};

const LONG_MEDIA_PLAYLIST: &str = include_str!("long_media_playlist.m3u8");
const EXPECTED_OUTPUT_PLAYLIST: &str = r#"#EXTM3U
#EXT-X-TARGETDURATION:4
#EXT-X-MEDIA-SEQUENCE:541647
#EXT-X-VERSION:9
#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=24.0
#EXT-X-DATERANGE:ID="0x10-2-1654866008",START-DATE="2022-06-10T13:00:09Z",PLANNED-DURATION=299.484,SCTE35-OUT="0xFC303100000000000000FFF00506FE00A84094001B021943554549000000027FFF00019B47470E053131313131100100FC61B6AE"
#EXT-X-DATERANGE:ID="0x10-2-1654866008",START-DATE="2022-06-10T13:00:09Z",END-DATE="2022-06-10T13:05:08Z",SCTE35-IN="0xFC303100000000000000FFF00506FE00A84094001B021943554549000000027FFF00019B47470E053131313131100100FC61B6AE"
#EXT-X-DATERANGE:ID="0x22-1-1654866308",START-DATE="2022-06-10T13:05:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE02443D39001B021943554549000000017FFF0000F748B60E05313131313122011671F30A5A"
#EXT-X-DATERANGE:ID="0x30-5-1654866308",START-DATE="2022-06-10T13:05:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE02443D39001B021943554549000000057FFF0000F748B60E05313131313130011663D5F99E"
#EXT-X-DATERANGE:ID="0x22-1-1654866308",START-DATE="2022-06-10T13:05:09Z",END-DATE="2022-06-10T13:08:09Z",SCTE35-IN="0xFC303100000000000000FFF00506FE033B964B001B021943554549000000017FFF0000F748B60E0531313131312301163BE5B24D"
#EXT-X-DATERANGE:ID="0x30-5-1654866308",START-DATE="2022-06-10T13:05:09Z",END-DATE="2022-06-10T13:08:09Z",SCTE35-IN="0xFC303100000000000000FFF00506FE033B964B001B021943554549000000057FFF0000F748B60E05313131313131011629C34189"
#EXT-X-DATERANGE:ID="0x22-1-1654866788",START-DATE="2022-06-10T13:13:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE04D792F0001B021943554549000000017FFF0000F748B60E053131313131220215E442D76F"
#EXT-X-DATERANGE:ID="0x30-5-1654866788",START-DATE="2022-06-10T13:13:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE04D792F0001B021943554549000000057FFF0000F748B60E053131313131300215F66424AB"
#EXT-X-DATERANGE:ID="0x22-1-1654866788",START-DATE="2022-06-10T13:13:09Z",END-DATE="2022-06-10T13:16:09Z",SCTE35-IN="0xFC303100000000000000FFF00506FE05CEEC02001B021943554549000000017FFF0000F748B60E0531313131312302155455A59A"
#EXT-X-DATERANGE:ID="0x30-5-1654866788",START-DATE="2022-06-10T13:13:09Z",END-DATE="2022-06-10T13:16:09Z",SCTE35-IN="0xFC303100000000000000FFF00506FE05CEEC02001B021943554549000000057FFF0000F748B60E0531313131313102154673565E"
#EXT-X-DATERANGE:ID="0x22-1-1654867269",START-DATE="2022-06-10T13:21:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE076AE8A7001B021943554549000000017FFF0000F748B60E05313131313122031460B1CDF8"
#EXT-X-DATERANGE:ID="0x30-5-1654867269",START-DATE="2022-06-10T13:21:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE076AE8A7001B021943554549000000057FFF0000F748B60E05313131313130031472973E3C"
#EXT-X-DATERANGE:ID="0x22-1-1654867269",START-DATE="2022-06-10T13:21:09Z",END-DATE="2022-06-10T13:24:09Z",SCTE35-IN="0xFC303100000000000000FFF00506FE086241B9001B021943554549000000017FFF0000F748B60E0531313131312303141487A6F1"
#EXT-X-DATERANGE:ID="0x30-5-1654867269",START-DATE="2022-06-10T13:21:09Z",END-DATE="2022-06-10T13:24:09Z",SCTE35-IN="0xFC303100000000000000FFF00506FE086241B9001B021943554549000000057FFF0000F748B60E05313131313131031406A15535"
#EXT-X-DATERANGE:ID="0x22-1-1654867749",START-DATE="2022-06-10T13:29:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE09FE3E5E001B021943554549000000017FFF0000F748B60E05313131313122041349328790"
#EXT-X-DATERANGE:ID="0x30-5-1654867749",START-DATE="2022-06-10T13:29:09Z",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE09FE3E5E001B021943554549000000057FFF0000F748B60E0531313131313004135B147454"
#EXT-X-SKIP:SKIPPED-SEGMENTS=9204
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=77.611,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=77.611,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:26.820Z
#EXTINF:3.904,
1652717346750item-01item_Segment-550851.mp4
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=81.515,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=81.515,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:30.724Z
#EXTINF:3.904,
1652717346750item-01item_Segment-550852.mp4
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=85.419,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=85.419,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:34.628Z
#EXTINF:3.903,
1652717346750item-01item_Segment-550853.mp4
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=89.322,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=89.322,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:38.531Z
#EXTINF:3.904,
1652717346750item-01item_Segment-550854.mp4
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=93.226,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=93.226,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:42.435Z
#EXTINF:3.904,
1652717346750item-01item_Segment-550855.mp4
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=97.130,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=97.130,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:46.339Z
#EXTINF:3.904,
1652717346750item-01item_Segment-550856.mp4
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=101.034,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=101.034,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:50.243Z
#EXTINF:3.904,
1652717346750item-01item_Segment-550857.mp4
"#;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut output_playlist = vec![];

    // Run once to validate output as expected
    make_delta_update(LONG_MEDIA_PLAYLIST.as_bytes(), &mut output_playlist)
        .expect("should not fail");
    assert_eq!(
        EXPECTED_OUTPUT_PLAYLIST,
        std::str::from_utf8(&output_playlist.clone()).expect("output should be valid string")
    );
    // Then run the bench
    c.bench_function(
        "Playlist delta update implementation using this library",
        |b| {
            b.iter(|| {
                output_playlist = vec![];
                black_box(
                    make_delta_update(LONG_MEDIA_PLAYLIST.as_bytes(), &mut output_playlist)
                        .expect("should not fail"),
                );
            });
        },
    );

    // Run once to validate output as expected for m3u8-rs
    //
    // As noted below, this example does not provide an accurate delta update, but we still want to
    // check the speed of the implementation.
    output_playlist = vec![];
    make_delta_update_using_m3u8_rs(LONG_MEDIA_PLAYLIST.as_bytes(), &mut output_playlist)
        .expect("should not fail");
    assert_eq!(
        KIND_OF_EXPECTED_M3U8_RS_OUTPUT,
        std::str::from_utf8(&output_playlist.clone()).expect("output should be valid string")
    );
    // Then run the bench
    c.bench_function(
        "Playlist delta update implementation using m3u8-rs library",
        |b| {
            b.iter(|| {
                output_playlist = vec![];
                black_box(
                    make_delta_update_using_m3u8_rs(
                        LONG_MEDIA_PLAYLIST.as_bytes(),
                        &mut output_playlist,
                    )
                    .expect("should not fail"),
                );
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// =================================================================================================
//
// Delta update implementation using this library
//
// In this file we are going to implement delta update using this library, but also, with some other
// open source available m3u8 parsers. The goal is to see how this library compares against others
// when it comes to a realistic task such as implementing a playlist delta update. Below is an
// implementation using this library.
//
// =================================================================================================

// We set some state variables to help with the delta update.
#[derive(Default)]
struct DeltaUpdateState<'a> {
    skip_until: Option<f64>, // EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL (or 6 * target duration)
    lines: Vec<HlsLine<'a>>, // Temporary lines that will be trimmed by delta update
    segments_count: u64,     // Count of URI lines found in lines
    removed_count: u64,      // Count of how many URI lines were removed
    drain_lines_counter: u8, // Cyclic counter used to trigger drain of lines
    did_write_version: bool, // If we found/wrote the version tag
    did_write_server_control: bool, // If we found/wrote the server control tag
    existing_server_control: Option<ServerControl<'a>>, // existing tag found before target duration
}

fn make_delta_update<W: Write>(input: &[u8], output: &mut W) -> Result<(), Box<dyn Error>> {
    let parsing_options = ParsingOptionsBuilder::new()
        .with_parsing_for_m3u()
        .with_parsing_for_version()
        .with_parsing_for_targetduration()
        .with_parsing_for_server_control()
        .with_parsing_for_inf()
        .build();
    let mut reader = Reader::from_bytes(input, parsing_options);

    // Wrap the output in a m3u8::Writer to make writing HLS lines easier.
    let mut writer = Writer::new(output);

    // Here we initialize some state variables to help with the delta update.
    let mut state = DeltaUpdateState::default();

    // This check is required to validate that this seems like a valid playlist.
    match reader.read_line().map_err(|e| e.error)? {
        Some(HlsLine::KnownTag(known::Tag::Hls(hls::Tag::M3u(tag)))) => {
            writer.write_line(HlsLine::from(tag))?;
        }
        line => return Err(format!("unexpected first line: {line:?}").into()),
    }

    // This is where we remove the lines that we should for delta update, from what we have
    // collected, before writing them to output.
    let drain_lines = |state: &mut DeltaUpdateState, writer: &mut Writer<&mut W>| {
        let Some(skip_until) = state.skip_until else {
            return Ok::<(), std::io::Error>(());
        };
        let mut backwards_segment_duration = 0.0;
        let mut drain_end = 0;
        let mut uri_count = 0;
        for (index, line) in state.lines.iter().enumerate().rev() {
            match line {
                HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Inf(tag))) => {
                    backwards_segment_duration += tag.duration()
                }
                HlsLine::Uri(_) => {
                    if backwards_segment_duration >= skip_until {
                        drain_end = index + 1;
                        break;
                    } else {
                        uri_count += 1;
                    }
                }
                _ => (),
            }
        }
        let protected_lines = state
            .lines
            .drain(..drain_end)
            .filter(|line| !is_media_segment_tag(line));
        state.removed_count += state.segments_count - uri_count;
        state.segments_count = uri_count;
        for line in protected_lines {
            writer.write_line(line)?;
        }
        Ok(())
    };

    // We do an initial loop through until the first media segment to ensure that we have set or
    // updated the EXT-X-VERSION and EXT-X-SERVER-CONTROL. This may look overly complicated, and
    // perhaps it is for this simple demo purpose; however, we're being quite thorough here to cover
    // all cases for any potential upstream playlist we may be dealing with. For example, consider
    // that this could be a function we are running at edge or via some other proxy, so that we may
    // not be able to completely trust the source media playlist. In that case, we wouldn't want a
    // bad expectation to block delivery of the playlist to our users.
    loop {
        match reader.read_line() {
            Ok(Some(line)) => match line {
                HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Targetduration(ref tag))) => {
                    // We use EXT-X-TARGETDURATION to set the CAN-SKIP-UNTIL.
                    let calculated_skip_until = 6.0 * (tag.target_duration() as f64);
                    if state.skip_until.is_none() {
                        state.skip_until = Some(calculated_skip_until);
                    }
                    writer.write_line(line)?;
                }
                HlsLine::KnownTag(known::Tag::Hls(hls::Tag::ServerControl(mut tag))) => {
                    // If the upstream playlist already has EXT-X-SERVER-CONTROL, then we either use
                    // the CAN-SKIP-UNTIL that exists, or update the tag to use the value we are
                    // defining. If we don't know the value yet (if the EXT-X-SERVER-CONTROL appears
                    // before the EXT-X-TARGETDURATION) then we store this tag for later use.
                    if let Some(existing_skip_until) = state.skip_until {
                        if let Some(can_skip_until) = tag.can_skip_until() {
                            state.skip_until = Some(can_skip_until);
                        } else {
                            tag.set_can_skip_until(existing_skip_until);
                        }
                        writer.write_line(HlsLine::from(tag))?;
                        state.did_write_server_control = true;
                    } else if let Some(can_skip_until) = tag.can_skip_until() {
                        state.skip_until = Some(can_skip_until);
                        writer.write_line(HlsLine::from(tag))?;
                        state.did_write_server_control = true;
                    } else {
                        state.existing_server_control = Some(tag);
                    }
                }
                HlsLine::KnownTag(known::Tag::Hls(hls::Tag::Version(mut tag))) => {
                    if tag.version() < 9 {
                        tag.set_version(9);
                    }
                    writer.write_line(HlsLine::from(tag))?;
                    state.did_write_version = true;
                }
                // Once we find the first media segment tag, we stop and make sure that we have
                // written out the updated EXT-X-VERSION and EXT-X-SERVER-CONTROL tags necessary for
                // the delta update.
                line if is_media_segment_tag(&line) => {
                    // We could handle this more elegantly, for example, just write out the rest of
                    // the playlist and make no delta update; however, for the purpose of this demo,
                    // this is good enough. This should never happen as it implies that there was no
                    // EXT-X-TARGETDURATION before reaching segments which is not permitted in HLS
                    // (or at least, the tag is marked as REQUIRED, and I believe the implication is
                    // that it is required before any Media Segment Tags).
                    let Some(skip_until) = state.skip_until else {
                        return Err(
                            format!("skip until must be defined before media segments").into()
                        );
                    };
                    // The minimum version if we are introducing EXT-X-SKIP is 9.
                    if !state.did_write_version {
                        writer.write_hls_tag(hls::Tag::Version(Version::new(9)))?;
                    }
                    // We may not have written EXT-X-SERVER-CONTROL yet, if we found it before the
                    // EXT-X-TARGETDURATION tag, or if it was not present at all in the playlist. We
                    // make sure it has been written with CAN-SKIP-UNTIL attribute below.
                    if !state.did_write_server_control {
                        if let Some(mut server_control) =
                            std::mem::take(&mut state.existing_server_control)
                        {
                            server_control.set_can_skip_until(skip_until);
                            writer.write_line(HlsLine::from(server_control))?;
                        } else {
                            writer.write_hls_tag(hls::Tag::ServerControl(
                                ServerControl::builder()
                                    .with_can_skip_until(skip_until)
                                    .finish(),
                            ))?;
                        }
                    }
                    if let HlsLine::Uri(_) = line {
                        state.segments_count += 1;
                    }
                    state.lines.push(line);
                    break;
                }
                line => {
                    writer.write_line(line)?;
                }
            },
            Ok(None) => break, // Reached end of playlist
            // We also demonstrate here that even if there are errors in reading the line we can
            // still utilize the input.
            Err(e) => writer.get_mut().write_all(e.errored_line)?,
        }
    }

    // We now loop through the rest of the playlist and add lines to our lines vector, occasionally
    // draining to stop it growing too large (perhaps paying memory resizing penaties).
    loop {
        // To stop the Vec of lines getting too big we periodically drain it. This counter is
        // managed by wrapping around on a u8 and draining each time we reach 0 (every 256 lines).
        state.drain_lines_counter = state.drain_lines_counter.wrapping_add(1);
        if state.drain_lines_counter == 0 {
            drain_lines(&mut state, &mut writer)?;
        }
        // We add any line to lines, but for URI lines, we need to ensure we update the count of
        // segments. This is important in this implementation, because later we drain without
        // counting how many segments are drained (segments are not equivalent to lines/index range)
        // and we use the total count minus the count we are not removing to know how many were
        // removed.
        match reader.read_line() {
            Ok(Some(line)) => {
                if let HlsLine::Uri(_) = line {
                    state.segments_count += 1;
                }
                state.lines.push(line);
            }
            Ok(None) => break,
            Err(e) => return Err(Box::new(e.error)),
        }
    }

    // We do one final drain to make sure we're up to date.
    drain_lines(&mut state, &mut writer)?;

    // We need to write out how many segments we've skipped.
    writer.write_hls_tag(hls::Tag::Skip(Skip::builder(state.removed_count).finish()))?;

    // Finally, write out whatever we have left.
    for line in state.lines {
        writer.write_line(line)?;
    }

    // And flush out the inner writer contents, and we're done!
    let output = writer.into_inner();
    output.flush()?;

    Ok(())
}

// Helper function to validate if any line should be considered as "4.4.4. Media Segment Tags" (to
// be removed in a delta update).
fn is_media_segment_tag(line: &HlsLine) -> bool {
    let tag_name = match line {
        HlsLine::KnownTag(tag) => match tag {
            known::Tag::Hls(tag) => Some(tag.name()),
            known::Tag::Custom(_) => unimplemented!(),
        },
        HlsLine::UnknownTag(tag) => TagName::try_from(tag.name()).ok(),
        HlsLine::Uri(_) => return true,
        _ => None,
    };
    if let Some(tag_name) = tag_name {
        match tag_name.tag_type() {
            TagType::MediaSegment => true,
            _ => false,
        }
    } else {
        true
    }
}

// =================================================================================================
//
// Delta update implementation using m3u8-rs
//
// It must be noted that I found implementing the delta update using m3u8-rs impossible (within
// reason). There are several issues:
//   * Necessary tags EXT-X-SKIP and EXT-X-SERVER-CONTROL are not supported.
//   * The body of the playlist is only described in terms of segments, so media metadata tags must
//     be associated to a segment, but this means that we cannot write daterange without having an
//     associated segment.
//   * EXT-X-DATERANGE belongs to a segment, but the segment only has an Option of a daterange, and
//     not a Vec. This means that we lose many daterange tags during parsing as it is quite normal
//     to have more than one daterange together (e.g. chapter end followed by chapter start).
//
// I've tried to work around the first two points; however, the last one makes it very difficult to
// work with the library.
//
// =================================================================================================

fn make_delta_update_using_m3u8_rs<W: Write>(
    input: &[u8],
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    let mut media_playlist = parse_media_playlist_res(input).map_err(|e| e.to_string())?;
    let skip_until = 6.0 * (media_playlist.target_duration as f32);
    if media_playlist.version.unwrap_or_default() < 9 {
        media_playlist.version = Some(9);
    }
    let mut existing_server_control: Option<ExtTag> = None;
    // m3u8-rs does not support the EXT-X-SERVER-CONTROL tag. This means I have to work with the
    // ExtTag struct provided via the `unknown_tags` vec. Also, perhaps a bug, but unknown_tags are
    // not written at the media playlist level (they are for "MasterPlaylist", and they are for
    // "MediaSegment"; however, not for "MediaPlaylist", even though the documentation indicates
    // that at the playlist level they indicate unknown tags before the first media segment).
    for tag in &media_playlist.unknown_tags {
        if tag.tag.as_str() == "X-SERVER-CONTROL" {
            if let Some(rest) = &tag.rest {
                if rest.contains("CAN-SKIP-UNTIL") {
                    // This will get complicated here. In this scenario I would need to custom parse
                    // the server control tag, extract the CAN-SKIP-UNTIL value, and then update the
                    // skip until we are working with here. Since I know this isn't the case for the
                    // bench, I'll avoid going through this complexity, but the implementation is
                    // therefore incomplete.
                    todo!()
                } else {
                    existing_server_control = Some(ExtTag {
                        tag: "X-SERVER-CONTROL".to_string(),
                        rest: Some(format!("{rest},CAN-SKIP-UNTIL={skip_until}")),
                    });
                }
            } else {
                existing_server_control = Some(ExtTag {
                    tag: "X-SERVER-CONTROL".to_string(),
                    rest: Some(format!("CAN-SKIP-UNTIL={skip_until}")),
                });
            }
            break;
        }
    }
    // m3u8-rs doesn't support the concept of EXT-X-SKIP. Furthermore, any tag that can appear
    // within the segments section (e.g. EXT-X-DATERANGE) is tied to a segment, which means I can't
    // include them without having segments. This is problematic because I need to leave the skipped
    // over EXT-X-DATERANGE in the playlist, so I have to write these to intermediate data, and then
    // stitch it together after writing. I'll start with the server control tag and then later add
    // the daterange tags.
    let mut intermediate_lines = if let Some(server_control) = existing_server_control {
        format!("{server_control}\n").as_bytes().to_vec()
    } else {
        format!("#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL={skip_until:?}\n")
            .as_bytes()
            .to_vec()
    };
    // This logic for deciding how many segments to skip is similar to the above logic using our
    // library.
    let total_segment_count = media_playlist.segments.len();
    let mut drain_end = 0;
    let mut backwards_segment_duration = 0.0;
    let mut segment_count = 0usize;
    for (index, segment) in media_playlist.segments.iter().enumerate().rev() {
        if backwards_segment_duration >= skip_until {
            drain_end = index + 1;
            break;
        } else {
            segment_count += 1;
            backwards_segment_duration += segment.duration;
        }
    }
    for segment in media_playlist.segments.drain(..drain_end) {
        // Strangely, m3u8-rs only considers the possibility of having one EXT-X-DATERANGE tag per
        // segment. This means that in our example here, we lose a bunch of daterange tags, because
        // there are plenty of occurrences of multiple daterange before a segment tag (most times
        // an ad break ends a new chapter or program start marker will also be inserted at the same
        // time). This means that actually, it seems quite impossible to implement this delta update
        // properly (and in general, to use this library to parse a playlist like we have), so this
        // example is purely a test of speed and not accuracy.
        if let Some(daterange) = segment.daterange {
            let mut tag = b"#EXT-X-DATERANGE:".to_vec();
            daterange.write_attributes_to(&mut tag)?;
            tag.write_all(b"\n")?;
            intermediate_lines.write_all(&tag)?;
        }
    }
    intermediate_lines.write_all(
        format!(
            "#EXT-X-SKIP:SKIPPED-SEGMENTS={}\n",
            total_segment_count - segment_count
        )
        .as_bytes(),
    )?;
    let mut temporary_output = Vec::new();
    media_playlist.write_to(&mut temporary_output)?;
    // Here I take a fairly dumb approach to finding the first media segment tag by looping through
    // the bytes until I find the name of a media segment tag.
    let mut last_tag_token_index = 0;
    let mut insertion_index = None;
    for (index, byte) in temporary_output.iter().enumerate() {
        match byte {
            b':' | b'\n' if temporary_output[last_tag_token_index..index].starts_with(b"#EXT-") => {
                if byte == &b'\n' && temporary_output[last_tag_token_index..index].contains(&b':') {
                    continue; // We already checked this tag
                }
                let end_index = if temporary_output[index - 1] == b'\r' {
                    index - 1
                } else {
                    index
                };
                // To avoid re-writing a bunch of matching code, I'm just using our library code for
                // determining tag name and tag type. We would have to write it otherwise and it
                // would probably just be the same (nothing fancy there).
                if let Some(tag_name) =
                    std::str::from_utf8(&temporary_output[(last_tag_token_index + 4)..end_index])
                        .ok()
                        .and_then(|s| TagName::try_from(s).ok())
                {
                    match tag_name.tag_type() {
                        TagType::MediaSegment => {
                            insertion_index = Some(last_tag_token_index);
                            break;
                        }
                        _ => (),
                    }
                }
            }
            b'#' => last_tag_token_index = index,
            _ => (),
        }
    }
    if let Some(index) = insertion_index {
        let rest_of_lines = temporary_output.split_off(index);
        output.write_all(&temporary_output)?;
        output.write_all(&intermediate_lines)?;
        output.write_all(&rest_of_lines)?;
    } else {
        output.write_all(&temporary_output)?;
    }
    Ok(())
}

// m3u8-rs re-orders many tags, because when writing, it allocates new strings for each tag and has
// a pre-defined order of output. As mentioned in the code above, it also has a bug where it only
// considers that a segment tag can have at most one EXT-X-DATERANGE before it. So, I write
// "expected" output, but this is just to memorialize what I think the output should be given what
// we can do, so that if we try and change implementation later on we have a base of what to expect.
const KIND_OF_EXPECTED_M3U8_RS_OUTPUT: &str = r#"#EXTM3U
#EXT-X-VERSION:9
#EXT-X-TARGETDURATION:4
#EXT-X-MEDIA-SEQUENCE:541647
#EXT-X-SERVER-CONTROL:CAN-SKIP-UNTIL=24.0
#EXT-X-DATERANGE:ID="0x10-2-1654866008",START-DATE="2022-06-10T13:00:09+00:00",PLANNED-DURATION=299.484,SCTE35-OUT="0xFC303100000000000000FFF00506FE00A84094001B021943554549000000027FFF00019B47470E053131313131100100FC61B6AE"
#EXT-X-DATERANGE:ID="0x30-5-1654866308",START-DATE="2022-06-10T13:05:09+00:00",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE02443D39001B021943554549000000057FFF0000F748B60E05313131313130011663D5F99E"
#EXT-X-DATERANGE:ID="0x30-5-1654866308",START-DATE="2022-06-10T13:05:09+00:00",END-DATE="2022-06-10T13:08:09+00:00",SCTE35-IN="0xFC303100000000000000FFF00506FE033B964B001B021943554549000000057FFF0000F748B60E05313131313131011629C34189"
#EXT-X-DATERANGE:ID="0x30-5-1654866788",START-DATE="2022-06-10T13:13:09+00:00",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE04D792F0001B021943554549000000057FFF0000F748B60E053131313131300215F66424AB"
#EXT-X-DATERANGE:ID="0x30-5-1654866788",START-DATE="2022-06-10T13:13:09+00:00",END-DATE="2022-06-10T13:16:09+00:00",SCTE35-IN="0xFC303100000000000000FFF00506FE05CEEC02001B021943554549000000057FFF0000F748B60E0531313131313102154673565E"
#EXT-X-DATERANGE:ID="0x30-5-1654867269",START-DATE="2022-06-10T13:21:09+00:00",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE076AE8A7001B021943554549000000057FFF0000F748B60E05313131313130031472973E3C"
#EXT-X-DATERANGE:ID="0x30-5-1654867269",START-DATE="2022-06-10T13:21:09+00:00",END-DATE="2022-06-10T13:24:09+00:00",SCTE35-IN="0xFC303100000000000000FFF00506FE086241B9001B021943554549000000057FFF0000F748B60E05313131313131031406A15535"
#EXT-X-DATERANGE:ID="0x30-5-1654867749",START-DATE="2022-06-10T13:29:09+00:00",PLANNED-DURATION=180.067,SCTE35-OUT="0xFC303100000000000000FFF00506FE09FE3E5E001B021943554549000000057FFF0000F748B60E0531313131313004135B147454"
#EXT-X-SKIP:SKIPPED-SEGMENTS=9204
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:26.820Z
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=77.611,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=77.611,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXTINF:3.904,
1652717346750item-01item_Segment-550851.mp4
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:30.724Z
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=81.515,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=81.515,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXTINF:3.904,
1652717346750item-01item_Segment-550852.mp4
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:34.628Z
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=85.419,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=85.419,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXTINF:3.903,
1652717346750item-01item_Segment-550853.mp4
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:38.531Z
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=89.322,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=89.322,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXTINF:3.904,
1652717346750item-01item_Segment-550854.mp4
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:42.435Z
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=93.226,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=93.226,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXTINF:3.904,
1652717346750item-01item_Segment-550855.mp4
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:46.339Z
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=97.130,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=97.130,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXTINF:3.904,
1652717346750item-01item_Segment-550856.mp4
#EXT-X-PROGRAM-DATE-TIME:2022-06-10T13:30:50.243Z
#EXT-X-SCTE35:TYPE=0x22,ELAPSED=101.034,UPID="0x0E:0x3131313131",ID="1",DURATION=180.067,BLACKOUT=NO,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAAAX//AAD3SLYOBTExMTExIgQTSTKHkA=="
#EXT-X-SCTE35:TYPE=0x30,ELAPSED=101.034,UPID="0x0E:0x3131313131",ID="5",DURATION=180.067,BLACKOUT=NO,CUE-OUT=CONT,CUE="/DAxAAAAAAAAAP/wBQb+Cf4+XgAbAhlDVUVJAAAABX//AAD3SLYOBTExMTExMAQTWxR0VA=="
#EXTINF:3.904,
1652717346750item-01item_Segment-550857.mp4
"#;
