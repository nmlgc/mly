//! MIDI sequence manipulation.

use std::{error::Error, io};

use midly::{num::u28, MetaMessage, Smf, TrackEvent, TrackEventKind};

use crate::{event, time};

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

pub fn filter_note(
    smf: &Smf,
    range: (u64, Option<u64>),
    invert: bool,
) -> Result<(), Box<dyn Error>> {
    let range = time::validate_pulse_range(smf, range)?;

    let mut filtered_smf = Smf {
        header: smf.header,
        tracks: vec![],
    };

    for track in &smf.tracks {
        let mut pulse: u64 = 0;
        let mut delta_carry: u28 = 0.into();
        filtered_smf.tracks.push(
            track
                .iter()
                .filter_map(|ev| {
                    let mut ev = *ev;
                    pulse += ev.delta.as_int() as u64;
                    ev.delta += delta_carry;
                    delta_carry = 0.into();
                    if (range.contains(&pulse) ^ invert) && event::note_on(&ev).is_some() {
                        delta_carry = ev.delta;
                        return None;
                    }
                    Some(ev)
                })
                .collect::<Vec<TrackEvent>>(),
        );
    }

    Ok(filtered_smf.write_std(io::stdout())?)
}

pub fn loop_unfold(smf: &mut Smf, start: u64) -> Result<(), Box<dyn Error>> {
    time::validate_pulse(smf, start)?;

    for (track_i, track) in &mut smf.tracks.iter_mut().enumerate() {
        if !ends_with_end_of_track_event(track) {
            return Err(format!("track #{track_i} does not end with an end-of-track event").into());
        };
        let Some(start) = find_event_at_or_after(start, track) else {
            continue;
        };
        let end = track.len() - 1;
        let len = end - start;
        let range = Vec::from_iter(track.iter().skip(start).take(len).cloned());

        eprintln!(
            "Track #{track_i}: Repeating events #[{start}, {end}[ at the end of the sequence"
        );
        track.pop();
        track.extend(range);
    }
    Ok(smf.write_std(io::stdout())?)
}
