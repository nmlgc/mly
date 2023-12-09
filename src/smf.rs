//! Conversions between the various SMF formats.

use std::io;

use midly::{num::u28, MetaMessage, Smf, TrackEvent, TrackEventKind};

pub fn smf0(smf: &Smf) -> Result<(), io::Error> {
    if smf.tracks.len() <= 1 {
        return smf.write_std(io::stdout());
    }

    let mut smf0 = Smf {
        header: smf.header,
        tracks: vec![Vec::with_capacity(
            smf.tracks.iter().fold(0, |acc, track| acc + track.len()),
        )],
    };
    smf0.header.format = midly::Format::SingleTrack;

    struct TrackMerge<'a> {
        track: &'a [TrackEvent<'a>],
        i: usize,
        len: usize,
        pulse_of_i: u64,
    }

    let mut merge_tracks = smf
        .tracks
        .iter()
        .filter_map(|track| {
            track.first().map(|ev| TrackMerge {
                track,
                i: 0,
                len: track.len(),
                pulse_of_i: ev.delta.as_int().into(),
            })
        })
        .collect::<Vec<_>>();

    let mut pulse: u64 = 0;
    let mut delta_last = u28::new(0);
    while let Some((merge_i, merge)) = merge_tracks
        .iter_mut()
        .enumerate()
        .min_by(|a, b| a.1.pulse_of_i.cmp(&b.1.pulse_of_i))
    {
        let kind = merge.track[merge.i].kind;
        delta_last = u28::new((merge.pulse_of_i - pulse) as u32);
        if !matches!(kind, TrackEventKind::Meta(MetaMessage::EndOfTrack)) {
            smf0.tracks[0].push(TrackEvent {
                delta: delta_last,
                kind,
            });
        }
        merge.i += 1;
        if merge.i >= merge.len {
            merge_tracks.remove(merge_i);
            continue;
        }
        pulse = merge.pulse_of_i;
        merge.pulse_of_i += merge.track[merge.i].delta.as_int() as u64;
    }
    smf0.tracks[0].push(TrackEvent {
        delta: delta_last,
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    smf0.write_std(io::stdout())
}
