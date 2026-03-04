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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::{
        get_tone_offset_info::ToneOffsetInfo, get_written_durations::Notation,
    };

    fn notation_with_info(furthest: i8) -> Notation {
        let mut n = Notation::new(0, 32);
        n.info = Some(ToneOffsetInfo {
            highest: furthest,
            lowest: furthest,
            furthest,
        });
        n
    }

    #[test]
    fn stem_direction_up_when_furthest_is_zero() {
        let n = notation_with_info(0);
        assert!(n.get_stem_direction() == Direction::Up);
    }

    #[test]
    fn stem_direction_up_when_furthest_positive() {
        let n = notation_with_info(3);
        assert!(n.get_stem_direction() == Direction::Up);
    }

    #[test]
    fn stem_direction_down_when_furthest_negative() {
        let n = notation_with_info(-2);
        assert!(n.get_stem_direction() == Direction::Down);
    }

    // --- get_span_stem_direction majority vote ---

    #[test]
    fn span_direction_favours_majority_up() {
        use crate::instructions::parse::get_beams::{Beam, BeamSegments, FractionalBeams};
        use rustc_hash::FxHashMap;

        let mut beam_ticks = FxHashMap::default();
        beam_ticks.insert(0u64, 1u8);
        beam_ticks.insert(32u64, 1u8);

        let beam = Beam {
            ticks: beam_ticks,
            start: 0,
            stop: 32,
            beam_segments: BeamSegments::default(),
            fractional_beams: FractionalBeams::default(),
        };

        let mut track = FxHashMap::default();
        track.insert(0u64, notation_with_info(3)); // furthest=3 → Up
        track.insert(32u64, notation_with_info(2)); // furthest=2 → Up

        let notation_track = NotationTrack { length: 64, track };
        let direction = get_span_stem_direction(&beam, &notation_track);
        assert!(direction == Direction::Up);
    }
}
