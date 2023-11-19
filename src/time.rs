//! Timekeeping in various units.

use std::{cmp::max, time::Duration};

use midly::{num::u28, MetaMessage, Timing, TrackEvent, TrackEventKind};

#[derive(Clone, Copy)]
pub struct MidiTime {
    pulse: u64,
    realtime: Option<Duration>,
    qn_duration: Option<Duration>, // a.k.a. "tempo"

    ppqn: u16,
    samplerate: Option<u32>,
}

impl MidiTime {
    pub fn new(timing: &Timing, samplerate: Option<u32>) -> Self {
        MidiTime {
            pulse: 0,
            realtime: None,
            qn_duration: None,
            ppqn: match timing {
                Timing::Metrical(ppqn) => ppqn.as_int(),
                Timing::Timecode(_, _) => unimplemented!("ticks/second not supported"),
            },
            samplerate,
        }
    }

    pub fn sample(&self) -> Option<f64> {
        self.samplerate
            .zip(self.realtime)
            .map(|(r, t)| t.as_secs_f64() * r as f64)
    }
}

impl std::ops::Add<&TrackEvent<'_>> for MidiTime {
    type Output = Self;

    fn add(self, ev: &TrackEvent<'_>) -> Self {
        let pulse = self.pulse + ev.delta.as_int() as u64;
        let realtime = self
            .qn_duration
            .filter(|_| self.realtime.is_some() || pulse == 0)
            .map(|q| {
                self.realtime.unwrap_or_default()
                    + q.mul_f64(ev.delta.as_int().into())
                        .div_f64(self.ppqn as f64)
            });
        let qn_duration = if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = ev.kind {
            Some(Duration::from_micros(tempo.as_int().into()))
        } else {
            self.qn_duration
        };
        MidiTime {
            pulse,
            realtime,
            qn_duration,
            ppqn: self.ppqn,
            samplerate: self.samplerate,
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
    pub minutes: usize,
    pub sample: usize,
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
        let minutes_width = max(end.realtime.map_or(1, |d| d.as_secs() / 60), 1).ilog10() + 1;

        MidiTimeDisplay {
            time: start,
            widths: UnitWidths {
                delta: ((max(delta_max.as_int(), 1).ilog10() + 1) + 1) as usize,
                pulse: (max(end.pulse, 1).ilog10() + 1) as usize,
                beat: (beat_qn_width + 1 + beat_pulse_width) as usize,
                beat_qn: beat_qn_width as usize,
                beat_pulse: beat_pulse_width as usize,
                minutes: minutes_width as usize,
                sample: (max(end.sample().unwrap_or(1.0) as u64, 1).ilog10() + 1) as usize + 3,
            },
        }
    }

    pub fn new(timing: &Timing, track: &[TrackEvent], samplerate: Option<u32>) -> Self {
        let time_init = MidiTime::new(timing, samplerate);
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
        write!(f, "pulse {pulse} / beat {beat}")?;
        if let Some(realtime) = self.time.realtime {
            // Duration::subsec_millis() truncates, which is not all too nice. Note that we have to
            // preserve the carry in case we round up from 999 to 1000 milliseconds – I was very
            // fortunate to have this case happen in my tests!
            let total_millis = (realtime.as_micros() as f64 / 1000.0).round() as u128;
            let millis = (total_millis % 1000) as u16;
            let seconds = ((total_millis / 1000) % 60) as u8;
            let minutes = ((total_millis / 1000) / 60) % 60;
            let minutes_width = self.widths.minutes;
            write!(f, " / {minutes:>minutes_width$}:{seconds:02}:{millis:03}m")?;
        }
        if let Some(sample) = self.time.sample() {
            let sample_width = self.widths.sample;
            write!(f, " / sample {sample:>sample_width$.2}")?;
        }
        Ok(())
    }
}
