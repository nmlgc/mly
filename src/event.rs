//! Helpful event inspection wrappers.

use midly::{
    num::{u4, u7},
    MidiMessage, TrackEvent, TrackEventKind,
};

pub struct Note {
    pub channel: u4,
    pub key: u7,
    pub vel: u7,
}

impl Note {
    pub fn is_on(&self) -> bool {
        self.vel > 0
    }
}

pub fn note(ev: &TrackEvent) -> Option<Note> {
    if let TrackEventKind::Midi {
        channel,
        message: MidiMessage::NoteOn { vel, key },
    } = ev.kind
    {
        return Some(Note { channel, key, vel });
    }
    None
}

pub fn note_on(ev: &TrackEvent) -> Option<Note> {
    note(ev).filter(|n| n.is_on())
}
