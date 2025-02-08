use crate::score::components::duration::Tick;
use crate::score::components::duration::Ticks;
use crate::score::entries::tone::Tone;
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Track;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;

use super::get_bars::Bars;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

pub type NotationByTrack = FxHashMap<UniqueId, NotationTrack>;

pub struct Notation {
    pub tick: Tick,
    pub tones: Vec<Tone>,
    pub duration: Ticks,
    pub ties: FxHashSet<UniqueId>,
}

impl Notation {
    pub fn new(tick: Tick, duration: Ticks) -> Notation {
        Notation {
            tick,
            tones: Vec::new(),
            duration,
            ties: FxHashSet::default(),
        }
    }

    pub fn is_rest(&self) -> bool {
        self.tones.is_empty()
    }
}

pub struct NotationTrack {
    pub length: u64,
    pub track: FxHashMap<Tick, Notation>,
}

impl NotationTrack {
    pub fn new(length: Ticks) -> NotationTrack {
        let track = FxHashMap::default();

        NotationTrack { length, track }
    }

    pub fn split_at_events(&mut self, track: &Track, barlines: &Bars) {
        let mut split_points: FxHashSet<Tick> = FxHashSet::default();

        // barline events
        for tick in barlines.keys() {
            split_points.insert(*tick);
        }

        // tone events
        for entry in track.entries.by_id.values() {
            match entry {
                Entry::Tone(tone) => {
                    split_points.insert(tone.tick);
                    split_points.insert(tone.tick + tone.duration);
                }
                Entry::Barline(barline) => {
                    split_points.insert(barline.tick);
                }
                _ => {}
            }
        }

        let mut sorted_split_points: Vec<Tick> = split_points.iter().cloned().collect();
        sorted_split_points.sort_unstable();

        let mut prev_split_point = self.length;

        for &split_point in sorted_split_points.iter().rev() {
            let notation = Notation::new(split_point, prev_split_point - split_point);
            self.track.insert(split_point, notation);
            prev_split_point = split_point;
        }

        for entry in track.entries.by_id.values() {
            let mut is_split = false;

            if let Entry::Tone(tone) = entry {
                for tick in (tone.tick..tone.tick + tone.duration).rev() {
                    if let Some(notation) = self.track.get_mut(&tick) {
                        notation.tones.push(tone.clone());
                        if is_split {
                            notation.ties.insert(tone.id);
                        }
                        is_split = true
                    }
                }
            }
        }
    }
}

pub fn get_written_durations(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
    bars: &Bars,
) -> NotationByTrack {
    let mut entries = NotationByTrack::default();

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];

            for track_id in &stave.track_ids {
                let track = &tracks.by_id[track_id];
                let mut notation = NotationTrack::new(flow.length);
                notation.split_at_events(track, bars);
                entries.insert(track.id, notation);
            }
        }
    }

    entries
}
