//! MIDI event dumping.

use midly::Smf;

use crate::time::MidiTimeDisplay;

pub fn dump(smf: &Smf) {
    for (track_i, track) in smf.tracks.iter().enumerate() {
        let time = MidiTimeDisplay::new(track);
        let widths = time.widths();
        let delta_width = widths.delta;
        if track_i != 0 {
            println!();
        }
        println!("## Track {track_i}\n");
        for ev in track {
            println!("{:>+delta_width$}\t{:?}", ev.delta, ev.kind);
        }
    }
}
