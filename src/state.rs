//! Per-channel state tracking.

use midly::{
    num::{u24, u7},
    MetaMessage, MidiMessage, PitchBend, TrackEvent, TrackEventKind,
};

#[derive(Copy, Clone, PartialEq)]
pub struct MidiStateOnChannel {
    pub cc: [u7; 128],
    program: u7,
    bend: PitchBend,
}

#[derive(Clone, PartialEq)]
pub struct MidiState {
    pub ch: [MidiStateOnChannel; 16],
    tempo: u24,
}

impl MidiState {
    pub fn new() -> Self {
        Self {
            ch: [MidiStateOnChannel {
                cc: [0.into(); 128],
                program: 0.into(),
                bend: PitchBend::mid_raw_value(),
            }; 16],
            tempo: 0.into(),
        }
    }

    pub fn update(&mut self, ev: &TrackEvent) {
        if let TrackEventKind::Midi { channel, message } = ev.kind {
            match message {
                MidiMessage::Controller { controller, value } => {
                    self.ch[channel.as_int() as usize].cc[controller.as_int() as usize] = value;
                }
                MidiMessage::ProgramChange { program } => {
                    self.ch[channel.as_int() as usize].program = program;
                }
                MidiMessage::PitchBend { bend } => {
                    self.ch[channel.as_int() as usize].bend = bend;
                }
                _ => {}
            }
        }
        if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = ev.kind {
            self.tempo = tempo;
        }
    }
}
