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

fn get_consensus_stem_direction<'a, I>(entries: I) -> Option<Direction>
where
    I: IntoIterator<Item = &'a Notation>,
{
    let mut up_count = 0;
    let mut down_count = 0;
    let mut furthests: Vec<i8> = Vec::new();

    for entry in entries {
        let direction = entry.get_stem_direction();

        match direction {
            Direction::Up => up_count += 1,
            Direction::Down => down_count += 1,
        }

        let info = entry.info.clone().unwrap();
        furthests.push(info.furthest);
    }

    match up_count.cmp(&down_count) {
        Ordering::Greater => Some(Direction::Up),
        Ordering::Less => Some(Direction::Down),
        Ordering::Equal if furthests.is_empty() => None,
        Ordering::Equal => {
            let mut max: i8 = 0;
            for furthest in furthests {
                if furthest.abs() > max.abs() {
                    max = furthest;
                }
            }

            if max > 0 {
                Some(Direction::Up)
            } else {
                Some(Direction::Down)
            }
        }
    }
}

pub fn get_span_stem_direction(span: &Beam, notation: &NotationTrack) -> Direction {
    get_consensus_stem_direction(span.ticks.keys().map(|tick| &notation.track[tick]))
        .expect("beam spans should contain at least one notation entry")
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

pub fn get_tuplet_stem_directions_in_track(
    notation: &NotationTrack,
    beams: &Beams,
) -> StemDirections {
    let mut output = get_stem_directions_in_track(notation, beams);
    let Some(direction) =
        get_consensus_stem_direction(notation.track.values().filter(|entry| !entry.is_rest()))
    else {
        return output;
    };

    for (tick, entry) in &notation.track {
        if !entry.is_rest() {
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

        let notation_track = NotationTrack {
            length: 64,
            track,
            anchored_tracks: Vec::new(),
        };
        let direction = get_span_stem_direction(&beam, &notation_track);
        assert!(direction == Direction::Up);
    }

    #[test]
    fn span_direction_down_when_majority_down() {
        use crate::instructions::parse::get_beams::{Beam, BeamSegments, FractionalBeams};
        use rustc_hash::FxHashMap;

        let mut beam_ticks = FxHashMap::default();
        beam_ticks.insert(0u64, 1u8);
        beam_ticks.insert(32u64, 1u8);
        beam_ticks.insert(64u64, 1u8);

        let beam = Beam {
            ticks: beam_ticks,
            start: 0,
            stop: 64,
            beam_segments: BeamSegments::default(),
            fractional_beams: FractionalBeams::default(),
        };

        let mut track = FxHashMap::default();
        track.insert(0u64, notation_with_info(-2)); // Down
        track.insert(32u64, notation_with_info(-3)); // Down
        track.insert(64u64, notation_with_info(1)); // Up

        let notation_track = NotationTrack {
            length: 96,
            track,
            anchored_tracks: Vec::new(),
        };
        let direction = get_span_stem_direction(&beam, &notation_track);
        assert!(direction == Direction::Down);
    }

    #[test]
    fn span_direction_tie_broken_by_furthest_negative() {
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

        // Equal up/down: one positive furthest, one negative
        let mut track = FxHashMap::default();
        track.insert(0u64, notation_with_info(1)); // Up
        track.insert(32u64, notation_with_info(-2)); // Down

        let notation_track = NotationTrack {
            length: 64,
            track,
            anchored_tracks: Vec::new(),
        };
        let direction = get_span_stem_direction(&beam, &notation_track);
        // furthest absolute is -2 (abs=2 > abs=1), so max=-2 and max<0 → Down
        assert!(direction == Direction::Down);
    }

    #[test]
    fn get_stem_directions_in_track_rest_not_included() {
        // A rest (no tones) has no stem direction
        let track = NotationTrack::new(128);
        let beams = vec![];
        let result = get_stem_directions_in_track(&track, &beams);
        assert!(result.is_empty());
    }

    #[test]
    fn get_stem_directions_in_track_note_included() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;

        let mut n = notation_with_info(1); // Up
                                           // give it a tone so is_rest() = false
        n.tones.push(Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));

        let mut track_map = FxHashMap::default();
        track_map.insert(0u64, n);

        let notation_track = NotationTrack {
            length: 128,
            track: track_map,
            anchored_tracks: Vec::new(),
        };
        let beams = vec![];
        let result = get_stem_directions_in_track(&notation_track, &beams);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[&0], Direction::Up));
    }

    #[test]
    fn get_stem_directions_in_track_beam_overrides_direction() {
        use crate::instructions::parse::get_beams::{Beam, BeamSegments, FractionalBeams};
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;

        fn make_tone_for(id: u64, tick: u64) -> Tone {
            Tone::new(
                id,
                tick,
                32,
                Pitch::new(60, Accidental::Natural),
                Velocity::new(100),
                Articulations::new(),
            )
        }

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

        let mut n0 = notation_with_info(1);
        n0.tones.push(make_tone_for(1, 0));
        let mut n32 = notation_with_info(-3);
        n32.tones.push(make_tone_for(2, 32));

        let mut track_map = FxHashMap::default();
        track_map.insert(0u64, n0);
        track_map.insert(32u64, n32);

        let notation_track = NotationTrack {
            length: 64,
            track: track_map,
            anchored_tracks: Vec::new(),
        };
        let result = get_stem_directions_in_track(&notation_track, &vec![beam]);
        // Span direction should be Down (furthest = -3 wins)
        assert!(matches!(result[&0], Direction::Down));
        assert!(matches!(result[&32], Direction::Down));
    }

    #[test]
    fn get_stem_directions_in_track_keeps_split_fragments_without_beams() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;

        let mut first = notation_with_info(2);
        first.tones.push(Tone::new(
            1,
            0,
            48,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        let mut second = notation_with_info(2);
        second.tones.push(Tone::new(
            1,
            48,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));

        let mut track_map = FxHashMap::default();
        track_map.insert(0_u64, first);
        track_map.insert(48_u64, second);

        let notation_track = NotationTrack {
            length: 96,
            track: track_map,
            anchored_tracks: Vec::new(),
        };
        let result = get_stem_directions_in_track(&notation_track, &vec![]);

        assert!(matches!(result[&0], Direction::Up));
        assert!(matches!(result[&48], Direction::Up));
    }

    #[test]
    fn get_tuplet_stem_directions_in_track_unifies_direction_across_notes() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;

        let mut first = notation_with_info(1);
        first.tones.push(Tone::new(
            1,
            0,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));

        let mut second = notation_with_info(-3);
        second.tones.push(Tone::new(
            2,
            32,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));

        let mut track_map = FxHashMap::default();
        track_map.insert(0_u64, first);
        track_map.insert(16_u64, Notation::new(16, 16));
        track_map.insert(32_u64, second);

        let notation_track = NotationTrack {
            length: 48,
            track: track_map,
            anchored_tracks: Vec::new(),
        };

        let result = get_tuplet_stem_directions_in_track(&notation_track, &vec![]);

        assert!(matches!(result[&0], Direction::Down));
        assert!(matches!(result[&32], Direction::Down));
        assert!(!result.contains_key(&16));
    }
}
