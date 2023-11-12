//! Timekeeping in various units.

use std::cmp::max;

use midly::{num::u28, TrackEvent};

#[derive(Clone, Copy)]
pub struct MidiTime {
    pulse: u64,
}

impl std::ops::Add<&TrackEvent<'_>> for MidiTime {
    type Output = Self;

    fn add(self, ev: &TrackEvent<'_>) -> Self {
        MidiTime {
            pulse: self.pulse + ev.delta.as_int() as u64,
        }
    }
}

#[derive(Clone)]
pub struct UnitWidths {
    pub delta: usize,
    pub pulse: usize,
}

/// Provides formatting for `MidiTime`.
pub struct MidiTimeDisplay {
    pub time: MidiTime,
    widths: UnitWidths,
}

pub struct MidiTimeDisplayPulse<'a>(&'a MidiTimeDisplay);

impl MidiTimeDisplay {
    fn with_limits(start: MidiTime, end: &MidiTime, delta_max: u28) -> Self {
        MidiTimeDisplay {
            time: start,
            widths: UnitWidths {
                delta: ((max(delta_max.as_int(), 1).ilog10() + 1) + 1) as usize,
                pulse: (max(end.pulse, 1).ilog10() + 1) as usize,
            },
        }
    }

    pub fn new(track: &[TrackEvent]) -> Self {
        let time_init = MidiTime { pulse: 0 };
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
}

impl<'a> std::fmt::Display for MidiTimeDisplayPulse<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pulse = self.0.time.pulse;
        let pulse_width = self.0.widths.pulse;
        write!(f, "{pulse:>pulse_width$}")
    }
}
