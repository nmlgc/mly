//! Loop detection.

use midly::{Smf, Timing, TrackEvent};

use crate::time::MidiTimeDisplay;

#[derive(Clone, Copy, Default)]
struct Loop {
    start: usize,
    len: usize,
}

impl Loop {
    fn print(&self, prefix: &str, timing: &Timing, track: &[TrackEvent]) {
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
        let mut time = MidiTimeDisplay::new(timing, track);
        for (ev_i, ev) in track.iter().enumerate() {
            time.time = time.time + ev;
            if ev_i == start {
                println!("Loop start: event {ev_i:>event_width$} / {time}");
            } else if ev_i == end_1 {
                println!("  Loop end: event {ev_i:>event_width$} / {time}");
                return;
            }
        }
    }
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
        return Some(new);
    }
    None
}

pub fn find(smf: &Smf) -> Result<(), String> {
    if smf.tracks.len() != 1 {
        return Err(format!(
            "only implemented for single-track sequences (sequence has {} tracks)",
            smf.tracks.len()
        ));
    }

    let track = &smf.tracks[0];
    let note_loop = (0..track.len()).fold(Loop::default(), |longest, cursor| {
        find_loop_ending_at(cursor, 0, longest.len, track).unwrap_or(longest)
    });
    note_loop.print("Best loop:", &smf.header.timing, track);
    Ok(())
}
