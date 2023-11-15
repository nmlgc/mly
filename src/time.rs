//! Timekeeping in various units.

use std::cmp::max;

use midly::{num::u28, Timing, TrackEvent};

#[derive(Clone, Copy)]
pub struct MidiTime {
    pulse: u64,
    ppqn: u16,
}

impl MidiTime {
    pub fn new(timing: &Timing) -> Self {
        MidiTime {
            pulse: 0,
            ppqn: match timing {
                Timing::Metrical(ppqn) => ppqn.as_int(),
                Timing::Timecode(_, _) => unimplemented!("ticks/second not supported"),
            },
        }
    }
}

impl std::ops::Add<&TrackEvent<'_>> for MidiTime {
    type Output = Self;

    fn add(self, ev: &TrackEvent<'_>) -> Self {
        MidiTime {
            pulse: self.pulse + ev.delta.as_int() as u64,
            ppqn: self.ppqn,
        }
    }
}

#[derive(Clone)]
pub struct UnitWidths {
    pub delta: usize,
    pub pulse: usize,
    pub beat: usize,
    pub beat_qn: usize,
    pub beat_pulse: usize,
}

/// Provides formatting for `MidiTime`.
pub struct MidiTimeDisplay {
    pub time: MidiTime,
    widths: UnitWidths,
}

pub struct MidiTimeDisplayPulse<'a>(&'a MidiTimeDisplay);
pub struct MidiTimeDisplayBeat<'a>(&'a MidiTimeDisplay);

impl MidiTimeDisplay {
    fn with_limits(start: MidiTime, end: &MidiTime, delta_max: u28) -> Self {
        let beat_qn_width = max(end.pulse / (end.ppqn as u64), 1).ilog10() + 1;
        let beat_pulse_width = max(end.ppqn, 1).ilog10() + 1;

        MidiTimeDisplay {
            time: start,
            widths: UnitWidths {
                delta: ((max(delta_max.as_int(), 1).ilog10() + 1) + 1) as usize,
                pulse: (max(end.pulse, 1).ilog10() + 1) as usize,
                beat: (beat_qn_width + 1 + beat_pulse_width) as usize,
                beat_qn: beat_qn_width as usize,
                beat_pulse: beat_pulse_width as usize,
            },
        }
    }

    pub fn new(timing: &Timing, track: &[TrackEvent]) -> Self {
        let time_init = MidiTime::new(timing);
        let (delta_max, end) = track.iter().fold((0.into(), time_init), |acc, ev| {
            (max(acc.0, ev.delta), (acc.1 + ev))
        });
        Self::with_limits(time_init, &end, delta_max)
    }

    pub fn widths(&self) -> UnitWidths {
        self.widths.clone()
    }

    pub fn display_pulse(&self) -> MidiTimeDisplayPulse {
        MidiTimeDisplayPulse(self)
    }
    pub fn display_beat(&self) -> MidiTimeDisplayBeat {
        MidiTimeDisplayBeat(self)
    }
}

impl<'a> std::fmt::Display for MidiTimeDisplayPulse<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pulse = self.0.time.pulse;
        let pulse_width = self.0.widths.pulse;
        write!(f, "{pulse:>pulse_width$}")
    }
}

impl<'a> std::fmt::Display for MidiTimeDisplayBeat<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let qn = self.0.time.pulse / (self.0.time.ppqn as u64);
        let pulse = self.0.time.pulse % (self.0.time.ppqn as u64);
        let qn_width = self.0.widths.beat_qn;
        let pulse_width = self.0.widths.beat_pulse;
        write!(f, "{qn:>qn_width$}:{pulse:>0pulse_width$}")
    }
}

impl std::fmt::Display for MidiTimeDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pulse = self.display_pulse();
        let beat = self.display_beat();
        write!(f, "pulse {pulse} / beat {beat}")
    }
}
