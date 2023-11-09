//! MIDI event dumping.

use midly::Smf;

pub fn dump(smf: &Smf) {
    for (track_i, track) in smf.tracks.iter().enumerate() {
        if track_i != 0 {
            println!();
        }
        println!("## Track {track_i}\n");
        for ev in track {
            println!("{:+}\t{:?}", ev.delta, ev.kind)
        }
    }
}
