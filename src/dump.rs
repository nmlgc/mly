//! MIDI event dumping.

use std::cmp::max;

use midly::Smf;

use crate::time::MidiTimeDisplay;

pub fn dump(smf: &Smf) {
    let delta_header = "Delta";
    let pulse_header = "Pulse";
    let beat_header = "Beat";

    for (track_i, track) in smf.tracks.iter().enumerate() {
        let mut time = MidiTimeDisplay::new(&smf.header.timing, track);
        let widths = time.widths();
        let delta_width = max(delta_header.chars().count(), widths.delta);
        let pulse_width = max(pulse_header.chars().count(), widths.pulse);
        let beat_width = max(beat_header.chars().count(), widths.beat);
        if track_i != 0 {
            println!();
        }
        println!("## Track {track_i}\n");
        println!(
            "{:>delta_width$}\t{:>pulse_width$}\t{:>beat_width$}\tEvent",
            delta_header, pulse_header, beat_header,
        );
        for ev in track {
            time.time = time.time + ev;
            println!(
                "{:>+delta_width$}\t{:>pulse_width$}\t{:>beat_width$}\t{:?}",
                ev.delta,
                time.display_pulse(),
                time.display_beat(),
                ev.kind,
            );
        }
    }
}
