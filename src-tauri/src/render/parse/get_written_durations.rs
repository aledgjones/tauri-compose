use crate::score::components::duration::Tick;
use crate::score::components::duration::Ticks;
use crate::score::entries::tone::Tone;
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
    pub fn is_rest(&self) -> bool {
        self.tones.is_empty()
    }
}

pub struct NotationTrack {
    pub length: u64,
    pub track: FxHashMap<Tick, Notation>,
}

impl NotationTrack {
    pub fn new(length: Ticks) -> Self {
        let mut track = FxHashMap::default();
        track.insert(
            0,
            Notation {
                tick: 0,
                tones: Vec::new(),
                duration: length,
                ties: FxHashSet::default(),
            },
        );
        Self { length, track }
    }

    pub fn get_previous_notation(&self, at: Tick) -> Option<(Tick, &Notation)> {
        for tick in (0..at + 1).rev() {
            match self.track.get(&tick) {
                Some(notation) => return Some((tick, notation)),
                None => continue,
            }
        }

        None
    }

    pub fn insert(&mut self, tick: Tick, notation: Notation) {
        self.track.insert(tick, notation);
    }

    pub fn split(&mut self, split_at: Tick) {
        if let Some((event_at, notation)) = self.get_previous_notation(split_at) {
            // only split if:
            // 1. split index not already the start of an event.
            // 2. split index is not the end of an event (ie; end of flow);
            if event_at != split_at && split_at != event_at + notation.duration {
                let tones = notation.tones.clone();
                let ties = notation.ties.clone();
                let duration = notation.duration;

                self.insert(
                    event_at,
                    Notation {
                        tick: event_at,
                        tones: tones.clone(),
                        duration: split_at - event_at,
                        ties: tones.iter().map(|tone| tone.id).collect(),
                    },
                );

                self.insert(
                    split_at,
                    Notation {
                        tick: split_at,
                        tones,
                        duration: event_at + duration - split_at,
                        ties,
                    },
                );
            }
        }
    }

    pub fn split_at_tone_events(&mut self, track: &Track) {
        for tick in 0..self.length {
            let tones = track.get_tones_at_tick(&tick);
            if !tones.is_empty() {
                self.split(tick);

                for tone in tones {
                    self.split(tick + tone.duration);
                    let mut is_split = false;
                    for inner_tick in (tick..tick + tone.duration).rev() {
                        if let Some(notation) = self.track.get_mut(&inner_tick) {
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

    pub fn split_at_measures(&mut self, barlines: &Bars) {
        for tick in barlines.keys() {
            self.split(*tick);
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
                notation.split_at_tone_events(track);
                notation.split_at_measures(bars);
                entries.insert(track.id, notation);
            }
        }
    }

    entries
}
