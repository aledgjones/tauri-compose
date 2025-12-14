use crate::score::components::duration::{Direction, Tick};
use crate::score::entries::tone::Tone;
use crate::utils::UniqueId;

use super::get_stem_directions::StemDirectionsByTrack;
use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::{Notation, NotationByTrack};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub type Clusters = Vec<Cluster>;
pub type Cluster = Vec<Tone>;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Shunt {
    Pre,
    None,
    Post,
}

pub struct Shunts {
    pub by_id: FxHashMap<(Tick, UniqueId), Shunt>,
    pub by_offset: FxHashMap<(Tick, i8), Shunt>,
}

pub type ShuntsByTrack = FxHashMap<UniqueId, Shunts>;

pub fn note_shunts_in_chord(
    tick: &Tick,
    entry: &Notation,
    tone_offsets: &ToneVerticalOffsets,
    stem_direction: &Direction,
    shunts: &mut Shunts,
) {
    let clusters = entry.get_clusters(tone_offsets);

    for cluster in clusters {
        let is_odd_length = cluster.len() % 2 != 0;
        let first_note_shunted = stem_direction == &Direction::Up || is_odd_length;

        for (i, tone) in cluster.iter().enumerate() {
            // alternate between shunted/not shunted based on the position
            // of the first tone in the cluster
            let shunted = match first_note_shunted {
                true => i % 2 > 0,
                false => i % 2 == 0,
            };

            let shunt = match shunted {
                true => match stem_direction {
                    Direction::Up => Shunt::Post,
                    Direction::Down => Shunt::Pre,
                },
                false => Shunt::None,
            };

            let offset = tone_offsets[&tone.id];

            shunts.by_id.insert((*tick, tone.id), shunt.clone());
            shunts.by_offset.insert((*tick, offset), shunt.clone());
        }
    }
}

pub fn get_note_shunts(
    notation_by_track: &NotationByTrack,
    tone_offsets: &ToneVerticalOffsets,
    stem_directions_by_track: &StemDirectionsByTrack,
) -> ShuntsByTrack {
    let results: Vec<(UniqueId, _)> = notation_by_track.par_iter().map(|(track_id, notation)| {
        let mut shunts = Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        };
        let stem_directions = &stem_directions_by_track[track_id];
        for (tick, entry) in &notation.track {
            if !entry.is_rest() {
                let stem_direction = &stem_directions[tick];
                note_shunts_in_chord(tick, entry, tone_offsets, stem_direction, &mut shunts);
            }
        }
        (*track_id, shunts)
    }).collect();

    let mut output: ShuntsByTrack = FxHashMap::default();
    for (track_id, shunts) in results {
        output.insert(track_id, shunts);
    }

    output
}

impl Notation {
    pub fn get_clusters(&self, tone_offsets: &ToneVerticalOffsets) -> Clusters {
        let mut clusters: Clusters = Vec::new();
        let mut cluster: Cluster = Vec::new();
        let mut previous_tone = self.tones.first().unwrap();

        for current_tone in &self.tones[1..] {
            let current_offset = tone_offsets[&current_tone.id];
            let previous_offset = tone_offsets[&previous_tone.id];

            cluster.push(previous_tone.clone());

            // if not a cluster
            if previous_offset - current_offset > 1 {
                clusters.push(cluster);
                cluster = Vec::new();
            }

            previous_tone = current_tone;
        }

        if !self.tones.is_empty() {
            cluster.push(previous_tone.clone());
            clusters.push(cluster);
        }

        clusters
    }
}
