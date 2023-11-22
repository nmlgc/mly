//! Loop detection.

use midly::{Smf, Timing, TrackEvent};
use rayon::prelude::*;

use crate::{event, time::MidiTimeDisplay};

#[derive(Clone, Copy, Default)]
struct Loop {
    start: usize,
    len: usize,
}

impl Loop {
    fn better_than(&self, other: &Loop) -> bool {
        (self.len > other.len) || ((self.len == other.len) && (self.start < other.start))
    }

    fn print(&self, prefix: &str, timing: &Timing, track: &[TrackEvent], samplerate: Option<u32>) {
        if self.len == 0 {
            println!("No loop found.");
            return;
        };
        let start = self.start;
        let len = self.len;
        let end_1 = start + len;
        let end_2 = end_1 + len;
        println!(
            "{prefix} {len} events (between event #[{start}, {end_1}[ and [{end_1}, {end_2}[)"
        );

        let event_width = (track.len().ilog10() + 1) as usize;
        let mut first_note_seen = false;
        let mut time = MidiTimeDisplay::new(timing, track, samplerate);
        for (ev_i, ev) in track.iter().enumerate() {
            time.time = time.time + ev;
            if !first_note_seen && event::note_on(ev).is_some() {
                println!("First note: event {ev_i:>event_width$} / {time}");
                first_note_seen = true;
            }
            if ev_i == start {
                println!("Loop start: event {ev_i:>event_width$} / {time}");
            } else if ev_i == end_1 {
                println!("  Loop end: event {ev_i:>event_width$} / {time}");
                return;
            }
        }
    }
}

fn loop_contains_itself(track: &[TrackEvent], found_loop: &Loop) -> bool {
    let track_at_loop_start = track.iter().skip(found_loop.start);
    for factor in (2..(found_loop.len / 2) + 1).filter(|&x| found_loop.len % x == 0) {
        let section_len = found_loop.len / factor;
        let section_is_repeated = (1..factor).all(|section_i| {
            let a = track_at_loop_start.clone().take(section_len);
            let b = track_at_loop_start
                .clone()
                .skip(section_i * section_len)
                .take(section_len);
            a.eq(b)
        });
        if section_is_repeated {
            return true;
        }
    }
    false
}

fn find_loop_ending_at(
    cursor: usize,
    earliest_start: usize,
    min_len: usize,
    track: &[TrackEvent],
) -> Option<Loop> {
    let cursor_ev = &track[cursor];
    for start in earliest_start..(cursor - min_len) {
        let start_ev = &track[start];

        // SMF Type 1 sequences can only ever support pulse-based looping. Not looping at arbitrary
        // events within a pulse is also better for playback integrity in general.
        if start_ev.delta == 0 || cursor_ev.delta == 0 {
            continue;
        }

        if start_ev != cursor_ev {
            continue;
        }

        let len = cursor - start;
        let before_cursor = track.iter().skip(start).take(len);
        let past_cursor = track.iter().skip(cursor).take(len);
        if before_cursor.ne(past_cursor) {
            continue;
        }

        let new = Loop { start, len };
        if loop_contains_itself(track, &new) {
            continue;
        }

        return Some(new);
    }
    None
}

pub struct Options {
    pub samplerate: Option<u32>,
}

pub fn find(smf: &Smf, opts: Options) -> Result<(), String> {
    if smf.tracks.len() != 1 {
        return Err(format!(
            "only implemented for single-track sequences (sequence has {} tracks)",
            smf.tracks.len()
        ));
    }

    let track = &smf.tracks[0];

    let note_loop = (0..track.len())
        .into_par_iter()
        .fold_with(Loop::default(), |longest, cursor| {
            find_loop_ending_at(cursor, 0, longest.len, track).unwrap_or(longest)
        })
        .reduce(Loop::default, |a, b| if a.better_than(&b) { a } else { b });

    note_loop.print("Best loop:", &smf.header.timing, track, opts.samplerate);
    Ok(())
}
