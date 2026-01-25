use crate::score::components::duration::Direction;
use crate::score::components::duration::Tick;
use crate::utils::UniqueId;

use super::get_beams::Beam;
use super::get_beams::Beams;
use super::get_beams::BeamsByTrack;
use super::get_written_durations::Notation;
use super::get_written_durations::NotationByTrack;
use super::get_written_durations::NotationTrack;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::cmp::Ordering;

pub type StemDirections = FxHashMap<Tick, Direction>;
pub type StemDirectionsByTrack = FxHashMap<UniqueId, StemDirections>;

pub fn get_span_stem_direction(span: &Beam, notation: &NotationTrack) -> Direction {
    let mut up_count = 0;
    let mut down_count = 0;
    let mut furthests: Vec<i8> = Vec::new();

    for tick in span.ticks.keys() {
        let entry = &notation.track[tick];
        let direction = entry.get_stem_direction();

        match direction {
            Direction::Up => up_count += 1,
            Direction::Down => down_count += 1,
        }

        let info = entry.info.clone().unwrap();
        furthests.push(info.furthest);
    }

    match up_count.cmp(&down_count) {
        Ordering::Greater => Direction::Up,
        Ordering::Less => Direction::Down,
        Ordering::Equal => {
            let mut max: i8 = 0;
            for furthest in furthests {
                if furthest.abs() > max.abs() {
                    max = furthest;
                }
            }

            if max > 0 {
                Direction::Up
            } else {
                Direction::Down
            }
        }
    }
}

pub fn get_stem_directions_in_track(notation: &NotationTrack, beams: &Beams) -> StemDirections {
    let mut output = FxHashMap::default();

    // natural stem directions
    for (at, entry) in &notation.track {
        if !entry.is_rest() {
            output.insert(*at, entry.get_stem_direction());
        };
    }

    // stem spans
    for span in beams {
        let direction = get_span_stem_direction(span, notation);
        for tick in span.ticks.keys() {
            output.insert(*tick, direction);
        }
    }

    output
}

pub fn get_stem_directions(
    tracks: &NotationByTrack,
    beams_by_track: &BeamsByTrack,
) -> StemDirectionsByTrack {
    let results: Vec<(UniqueId, _)> = tracks
        .par_iter()
        .map(|(track_id, track)| {
            let beams = &beams_by_track[track_id];
            let stem_directions = get_stem_directions_in_track(track, beams);
            (*track_id, stem_directions)
        })
        .collect();

    let mut output = FxHashMap::default();
    for (track_id, stem_directions) in results {
        output.insert(track_id, stem_directions);
    }

    output
}

impl Notation {
    pub fn get_stem_direction(&self) -> Direction {
        let info = &self.info.clone().unwrap();

        if info.furthest >= 0 {
            Direction::Up
        } else {
            Direction::Down
        }
    }
}
