//! Loop detection.

use std::collections::HashSet;

use midly::{MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use rayon::prelude::*;

use crate::{event, state::MidiState, time::MidiTimeDisplay};

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
    in_recording_space: bool,
) -> Option<Loop> {
    #[derive(PartialEq, Eq, Hash)]
    struct CCOnChannel {
        ch: usize,
        cc: usize,
    }

    let cursor_ev = &track[cursor];
    let mut state_before = MidiState::new();
    for ev in track.iter().take(earliest_start) {
        state_before.update(ev);
    }
    for start in earliest_start..(cursor - min_len) {
        let start_ev = &track[start];
        state_before.update(start_ev);

        // SMF Type 1 sequences can only ever support pulse-based looping. Not looping at arbitrary
        // events within a pulse is also better for playback integrity in general.
        if start_ev.delta == 0 || cursor_ev.delta == 0 {
            continue;
        }

        // Program changes can be expensive operations on some MIDI devices. Let's not start a loop
        // on the same pulse.
        if let TrackEventKind::Midi {
            channel: _,
            message: MidiMessage::ProgramChange { program: _ },
        } = start_ev.kind
        {
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

        // Identical state at both points?
        let mut state_past = state_before.clone();
        let mut redundant_ccs = HashSet::new(); // Defer the overrides to the end of the loop
        let mut played_a_note = [false; 16];

        // Let's better assume that the polyphony is equal to the maximum allowed note number in a
        // MIDI file… (see https://youtu.be/4uDfG1BbxmQ)
        let mut notes_active_on = [0_u64; 16];
        for ev in track.iter().skip(start).take(len) {
            state_past.update(ev);

            if let Some(note) = event::note(ev) {
                let ch = note.channel.as_int() as usize;
                if note.is_on() {
                    // If a channel hasn't played a note between the start of the loop and a
                    // controller change, we can ignore that controller for the state comparison.
                    played_a_note[ch] = true;
                    notes_active_on[ch] += 1;
                } else if notes_active_on[ch] > 0 {
                    // Nicer than saturating_sub() in defending against mismatched Note Off events.
                    notes_active_on[ch] -= 1;
                }
            }

            if let Some(cc) = event::controller(ev) {
                let ch = cc.channel.as_int() as usize;
                let cc = cc.controller.as_int() as usize;
                if !played_a_note[ch] {
                    redundant_ccs.insert(CCOnChannel { ch, cc });
                }
            }
        }

        // In recording space, any active notes at the loop boundaries must have identical channel
        // state.
        if in_recording_space
            && notes_active_on
                .iter()
                .enumerate()
                .any(|(ch, active)| *active > 0 && state_before.ch[ch] != state_past.ch[ch])
        {
            continue;
        }

        for CCOnChannel { ch, cc } in redundant_ccs {
            state_past.ch[ch].cc[cc] = state_before.ch[ch].cc[cc];
        }

        if state_before != state_past {
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
            "only implemented for single-track sequences (sequence has {} tracks); try flattening with the `smf0` command",
            smf.tracks.len()
        ));
    }

    let track = &smf.tracks[0];

    let note_loop = (0..track.len())
        .into_par_iter()
        .fold_with(Loop::default(), |longest, cursor| {
            find_loop_ending_at(cursor, 0, longest.len, track, false).unwrap_or(longest)
        })
        .reduce(Loop::default, |a, b| if a.better_than(&b) { a } else { b });

    note_loop.print("Best loop in note space:", &smf.header.timing, track, None);

    if note_loop.len != 0 && opts.samplerate.is_some() {
        let start = note_loop.start;
        let recording_loop = ((start + note_loop.len)..track.len())
            .find_map(|cursor| find_loop_ending_at(cursor, start, 0, track, true))
            .unwrap_or_default();

        print!("\nBest loop in recording space: ");
        recording_loop.print("", &smf.header.timing, track, opts.samplerate);
    }

    Ok(())
}
