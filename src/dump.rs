//! MIDI event dumping.

use std::{cmp::max, fmt::Debug};

use midly::{MetaMessage, Smf, TrackEventKind};

use crate::time::MidiTimeDisplay;

struct Hex<'a>(&'a [u8]);

impl<'a> std::fmt::Display for Hex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            write!(f, "{}{:02X}", if i != 0 { " " } else { "" }, byte)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct KindDisplay<'a>(&'a TrackEventKind<'a>);

impl<'a> std::fmt::Display for KindDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            TrackEventKind::SysEx(b) => write!(f, "SysEx({})", Hex(b)),
            TrackEventKind::Meta(meta) => match meta {
                MetaMessage::Text(b) => write!(f, "Meta(Text({}))", Hex(b)),
                MetaMessage::Copyright(b) => write!(f, "Meta(Copyright({}))", Hex(b)),
                MetaMessage::TrackName(b) => write!(f, "Meta(TrackName({}))", Hex(b)),
                MetaMessage::Marker(b) => write!(f, "Meta(Marker({}))", Hex(b)),
                MetaMessage::InstrumentName(b) => write!(f, "Meta(InstrumentName({}))", Hex(b)),
                MetaMessage::Lyric(b) => write!(f, "Meta(Lyric({}))", Hex(b)),
                MetaMessage::CuePoint(b) => write!(f, "Meta(CuePoint({}))", Hex(b)),
                MetaMessage::ProgramName(b) => write!(f, "Meta(ProgramName({}))", Hex(b)),
                MetaMessage::DeviceName(b) => write!(f, "Meta(DeviceName({}))", Hex(b)),
                MetaMessage::SequencerSpecific(b) => {
                    write!(f, "Meta(SequencerSpecific({}))", Hex(b))
                }
                _ => self.0.fmt(f),
            },
            _ => self.0.fmt(f),
        }
    }
}

pub fn dump(smf: &Smf) {
    let delta_header = "Delta";
    let pulse_header = "Pulse";
    let beat_header = "Beat";

    for (track_i, track) in smf.tracks.iter().enumerate() {
        let mut time = MidiTimeDisplay::new(&smf.header.timing, track, None);
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
                "{:>+delta_width$}\t{:>pulse_width$}\t{:>beat_width$}\t{:}",
                ev.delta,
                time.display_pulse(),
                time.display_beat(),
                KindDisplay(&ev.kind),
            );
        }
    }
}
