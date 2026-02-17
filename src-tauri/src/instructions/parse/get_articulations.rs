use super::get_written_durations::{NotationByTrack, NotationTrack};
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::Tick;
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

pub type ArticulationSet = FxHashSet<Articulation>;
pub type Articulations = FxHashMap<Tick, ArticulationSet>;
pub type ArticulationsByTrack = FxHashMap<UniqueId, Articulations>;

fn get_articulations_in_track(notation: &NotationTrack) -> Articulations {
    let mut output: Articulations = FxHashMap::default();

    for (tick, entry) in &notation.track {
        if entry.is_rest() {
            continue;
        }

        let mut tick_set: ArticulationSet = FxHashSet::default();
        for tone in &entry.tones {
            for a in tone.articulations.iter() {
                tick_set.insert(*a);
            }
        }

        if !tick_set.is_empty() {
            output.insert(*tick, tick_set);
        }
    }

    output
}

pub fn get_articulations(notation_by_track: &NotationByTrack) -> ArticulationsByTrack {
    let results: Vec<(UniqueId, _)> = notation_by_track
        .par_iter()
        .map(|(track_id, notation)| {
            let articulations: Articulations = get_articulations_in_track(notation);
            (*track_id, articulations)
        })
        .collect();

    let mut output: ArticulationsByTrack = FxHashMap::default();
    for (track_id, articulations) in results {
        output.insert(track_id, articulations);
    }

    output
}
