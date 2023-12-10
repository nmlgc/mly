//! MIDI sequence manipulation.

use std::{error::Error, io};

use midly::{MetaMessage, Smf, TrackEvent, TrackEventKind};

use crate::time;

fn ends_with_end_of_track_event(track: &[TrackEvent]) -> bool {
    track
        .last()
        .is_some_and(|ev| matches!(ev.kind, TrackEventKind::Meta(MetaMessage::EndOfTrack)))
}

fn end_of_track_index(track: &[TrackEvent]) -> usize {
    if ends_with_end_of_track_event(track) {
        track.len() - 1
    } else {
        track.len()
    }
}

fn find_event_at_or_after(pulse: u64, track: &[TrackEvent]) -> Option<usize> {
    let mut pulse_cur: u64 = 0;
    track.iter().position(|ev| {
        pulse_cur += ev.delta.as_int() as u64;
        pulse_cur >= pulse
    })
}

pub fn cut(smf: &mut Smf, range: (u64, Option<u64>)) -> Result<(), Box<dyn Error>> {
    time::validate_pulse_range(smf, range)?;

    for (track_i, track) in &mut smf.tracks.iter_mut().enumerate() {
        let Some(start) = find_event_at_or_after(range.0, track) else {
            continue;
        };
        let end = range
            .1
            .and_then(|p| find_event_at_or_after(p, track))
            .unwrap_or(end_of_track_index(track));
        eprintln!("Track #{track_i}: Removing events #[{start}, {end}[",);
        let start_delta = track[start].delta;
        track.drain(start..end);

        if !ends_with_end_of_track_event(track) {
            track.push(TrackEvent {
                delta: start_delta,
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            })
        } else {
            track[start].delta = start_delta;
        }
    }
    Ok(smf.write_std(io::stdout())?)
}
