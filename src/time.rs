//! Timekeeping in various units.

use std::cmp::max;

use midly::{num::u28, TrackEvent};

#[derive(Clone)]
pub struct UnitWidths {
    pub delta: usize,
}

pub struct MidiTimeDisplay {
    widths: UnitWidths,
}

impl MidiTimeDisplay {
    fn with_limits(delta_max: u28) -> Self {
        MidiTimeDisplay {
            widths: UnitWidths {
                delta: ((max(delta_max.as_int(), 1).ilog10() + 1) + 1) as usize,
            },
        }
    }

    pub fn new(track: &[TrackEvent]) -> Self {
        let delta_max = track.iter().fold(0.into(), |acc, ev| (max(acc, ev.delta)));
        Self::with_limits(delta_max)
    }

    pub fn widths(&self) -> UnitWidths {
        self.widths.clone()
    }
}
