use super::get_beams::{Beam, Beams, BeamsByTrack};
use super::get_horizontal_spacing::{HorizontalSpacing, Slot};
use super::get_stem_directions::{StemDirections, StemDirectionsByTrack};
use super::get_written_durations::{Notation, NotationByTrack, NotationTrack};
use crate::instructions::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::renderer::converter::Space;
use crate::score::components::duration::{Direction, NoteDuration, Tick};
use crate::score::components::measurements::Point;
use crate::score::engraves::Engrave;
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub enum BeamSlant {
    Up = -1,
    None = 0,
    Down = 1,
}

#[derive(Clone)]
pub struct StemDef {
    pub head: Point,
    pub tail: Point,
}
pub type StemLengths = FxHashMap<Tick, StemDef>;
pub type StemLengthsByTrack = FxHashMap<UniqueId, StemLengths>;

fn adjust_to_beam(
    beam: &Beam,
    high: &StemDef,
    low: &StemDef,
    maximum_beam_slant: f32,
    output: &mut StemLengths,
) {
    let diff = low.tail.y - high.tail.y;

    let low_y = if diff > maximum_beam_slant {
        high.tail.y + maximum_beam_slant
    } else {
        low.tail.y
    };

    let tan_angle = (low_y - high.tail.y) / (low.tail.x - high.tail.x);

    let mut adjustment = 0.0;

    for tick in beam.ticks.keys() {
        let original = output.get_mut(tick).unwrap();
        let opp = tan_angle * (low.tail.x - original.tail.x);
        let beam_dictated_tail = low_y - opp;

        if beam_dictated_tail.abs() < original.tail.y.abs() {
            let offset = original.tail.y - beam_dictated_tail;
            if offset > adjustment {
                adjustment = offset;
            }
        }

        original.tail.y = beam_dictated_tail;
    }

    for tick in beam.ticks.keys() {
        output.get_mut(tick).unwrap().tail.y += adjustment;
    }
}

fn get_beam_slant(beam: &Beam, notation: &NotationTrack, stem_direction: &Direction) -> BeamSlant {
    let start = &notation.track[&beam.start];
    let stop = &notation.track[&beam.stop];

    let start_guide = start.get_beam_guide_note(stem_direction);
    let stop_guide = stop.get_beam_guide_note(stem_direction);

    if start_guide == stop_guide {
        return BeamSlant::None;
    }

    // check the melodic shape, we flatten in certain conditions
    for tick in beam.ticks.keys() {
        if tick == &beam.start || tick == &beam.stop {
            continue;
        }

        let entry = &notation.track[tick];
        let guide = entry.get_beam_guide_note(stem_direction);

        match stem_direction {
            Direction::Up => {
                if guide < start_guide && guide < stop_guide {
                    return BeamSlant::None;
                }
            }
            Direction::Down => {
                if guide > start_guide && guide > stop_guide {
                    return BeamSlant::None;
                }
            }
        }
    }

    if start_guide < stop_guide {
        BeamSlant::Down
    } else {
        BeamSlant::Up
    }
}

fn get_furthest_tail(beam: &Beam, stem_lengths: &StemLengths, stem_direction: &Direction) -> f32 {
    let mut furthest = 0.0;
    beam.ticks.iter().for_each(|(tick, _)| {
        let def = &stem_lengths[tick];
        match stem_direction {
            Direction::Up => {
                if def.tail.y < furthest {
                    furthest = def.tail.y;
                }
            }
            Direction::Down => {
                if def.tail.y > furthest {
                    furthest = def.tail.y;
                }
            }
        }
    });
    furthest
}

fn get_natural_stem_length(
    x: Space,
    entry: &Notation,
    stem_direction: &Direction,
    beams: &Beams,
) -> StemDef {
    let info = entry.info.clone().unwrap();

    // Base stem length
    let base_stem_length = 3.25;
    let beam_count = entry.get_beam_count();
    let is_flagged = entry.is_flagged(beams);
    let is_leading_note_on_line = match stem_direction {
        Direction::Up => info.lowest % 2 == 0,
        Direction::Down => info.highest % 2 == 0,
    };

    let mut beam_extension: f32 = 0.0;

    if beam_count > 0 && !is_flagged {
        if is_leading_note_on_line {
            beam_extension = if beam_count == 2 {
                -0.25
            } else if beam_count == 3 {
                0.5
            } else {
                0.0
            }
        } else {
            beam_extension = if beam_count > 2 { 1.0 } else { 0.0 }
        }
    };

    let total_stem_length = base_stem_length + beam_extension;

    let (x, head, tail) = match stem_direction {
        Direction::Up => {
            let head = (info.lowest as f32 - 0.5) / 2.0;
            let mut tail = (info.highest as f32 - 0.5) / 2.0 - total_stem_length;
            if tail > 0.0 {
                tail = 0.0
            }

            let metrics = entry.glyph_metrics();
            let glyph_width = metrics.total_width();
            let x = x + glyph_width;

            (x - (STAVE_LINE_WIDTH / 2.0), head, tail)
        }
        Direction::Down => {
            let head = (info.highest as f32 + 0.5) / 2.0;
            let mut tail = (info.lowest as f32 + 0.5) / 2.0 + total_stem_length;
            if tail < 0.0 {
                tail = 0.0
            };

            (x + (STAVE_LINE_WIDTH / 2.0), head, tail)
        }
    };

    StemDef {
        head: Point { x, y: head },
        tail: Point { x, y: tail },
    }
}

pub fn get_stem_lengths_in_track(
    notation: &NotationTrack,
    stem_directions: &StemDirections,
    beams: &Beams,
    engrave: &Engrave,
    horizontal_spacing: &HorizontalSpacing,
) -> StemLengths {
    let mut output: StemLengths = FxHashMap::default();

    // get natural lengths of stems
    for (tick, entry) in &notation.track {
        if !entry.is_rest() {
            // Only notes half or shorter get stems
            if let Some(duration) = entry.base_to_note_duration() {
                if duration != NoteDuration::Whole {
                    let x = horizontal_spacing.get(*tick, Slot::NoteSpacing).x;
                    let stem_direction = &stem_directions[tick];
                    output.insert(
                        *tick,
                        get_natural_stem_length(x, entry, stem_direction, beams),
                    );
                }
            }
        }
    }

    // for each beam adjust stems to meet beam angle
    for beam in beams {
        let stem_direction = &stem_directions[&beam.start];
        let slant = get_beam_slant(beam, notation, stem_direction);

        // clone these to release them from the borrow checker
        let first = output[&beam.start].clone();
        let last = output[&beam.stop].clone();

        match slant {
            BeamSlant::Up => match stem_direction {
                Direction::Up => {
                    adjust_to_beam(beam, &last, &first, engrave.maximum_beam_slant, &mut output);
                }
                Direction::Down => {
                    adjust_to_beam(beam, &last, &first, engrave.maximum_beam_slant, &mut output);
                }
            },
            BeamSlant::None => {
                let furthest = get_furthest_tail(beam, &output, stem_direction);
                beam.ticks.iter().for_each(|(tick, _)| {
                    let original = output.get_mut(tick).unwrap();
                    original.tail.y = furthest;
                });
            }
            BeamSlant::Down => match stem_direction {
                Direction::Up => {
                    adjust_to_beam(beam, &first, &last, engrave.maximum_beam_slant, &mut output);
                }
                Direction::Down => {
                    adjust_to_beam(beam, &first, &last, engrave.maximum_beam_slant, &mut output);
                }
            },
        }
    }

    output
}

pub fn get_stem_lengths(
    tracks: &NotationByTrack,
    stem_directions_by_track: &StemDirectionsByTrack,
    beams_by_track: &BeamsByTrack,
    engrave: &Engrave,
    horizontal_spacing: &HorizontalSpacing,
) -> StemLengthsByTrack {
    let results: Vec<(UniqueId, _)> = tracks
        .par_iter()
        .map(|(track_id, track)| {
            let stem_directions = &stem_directions_by_track[track_id];
            let beams = &beams_by_track[track_id];
            let stem_lengths = get_stem_lengths_in_track(
                track,
                stem_directions,
                beams,
                engrave,
                horizontal_spacing,
            );
            (*track_id, stem_lengths)
        })
        .collect();

    let mut output: StemLengthsByTrack = FxHashMap::default();
    for (track_id, stem_lengths) in results {
        output.insert(track_id, stem_lengths);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_horizontal_spacing::HorizontalSpacing;
    use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
    use crate::instructions::parse::get_written_durations::{Notation, NotationTrack};
    use crate::score::components::articulation::Articulations;
    use crate::score::components::duration::{Direction, NoteDuration};
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::engraves::Engraves;
    use crate::score::entries::tone::Tone;
    use crate::score::layouts::LayoutType;
    use rustc_hash::FxHashMap;

    fn make_note_with_info(tick: u64, duration: u64, id: u64, highest: i8, lowest: i8) -> Notation {
        let mut n = Notation::new(tick, duration);
        n.tones.push(Tone::new(
            id,
            tick,
            duration,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        n.info = Some(ToneOffsetInfo {
            highest,
            lowest,
            furthest: highest,
        });
        n
    }

    #[test]
    fn get_stem_lengths_in_track_empty_track_returns_empty() {
        let track = NotationTrack::new(512);
        let stem_directions: StemDirections = FxHashMap::default();
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.is_empty());
    }

    #[test]
    fn get_stem_lengths_in_track_rest_is_excluded() {
        let mut track = NotationTrack::new(512);
        // A rest (no tones) should not produce a stem
        let r = Notation::new(0, NoteDuration::Quarter.to_ticks());
        track.track.insert(0, r);
        let stem_directions: StemDirections = FxHashMap::default();
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.is_empty());
    }

    #[test]
    fn get_stem_lengths_in_track_whole_note_excluded() {
        // Whole notes do not get stems
        let mut track = NotationTrack::new(512);
        let whole = NoteDuration::Whole.to_ticks();
        let note = make_note_with_info(0, whole, 1, 2, 2);
        track.track.insert(0, note);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Up);
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.is_empty());
    }

    #[test]
    fn get_stem_lengths_in_track_quarter_note_up_produces_stem() {
        let mut track = NotationTrack::new(512);
        let q = NoteDuration::Quarter.to_ticks();
        let note = make_note_with_info(0, q, 1, 4, 4);
        track.track.insert(0, note);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Up);
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.contains_key(&0));
        let stem = &result[&0];
        // Stem up: tail should be above (more negative y) than head
        assert!(stem.tail.y <= stem.head.y);
    }

    #[test]
    fn get_stem_lengths_in_track_quarter_note_down_produces_stem() {
        let mut track = NotationTrack::new(512);
        let q = NoteDuration::Quarter.to_ticks();
        let note = make_note_with_info(0, q, 1, 4, 4);
        track.track.insert(0, note);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Down);
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.contains_key(&0));
        let stem = &result[&0];
        // Stem down: tail should be below (more positive y) than head
        assert!(stem.tail.y >= stem.head.y);
    }

    #[test]
    fn beam_slant_returns_none_for_same_start_stop_guide() {
        // start and stop both have guide note at same offset → BeamSlant::None
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let note1 = make_note_with_info(0, eighth, 1, 4, 4);
        let note2 = make_note_with_info(eighth, eighth, 2, 4, 4);
        track.track.insert(0, note1);
        track.track.insert(eighth, note2);

        let mut ticks = FxHashMap::default();
        ticks.insert(0u64, 1u8);
        ticks.insert(eighth, 1u8);

        let beam = Beam {
            ticks,
            start: 0,
            stop: eighth,
            beam_segments: FxHashMap::default(),
            fractional_beams: FxHashMap::default(),
        };

        let slant = get_beam_slant(&beam, &track, &Direction::Up);
        assert!(matches!(slant, BeamSlant::None));
    }

    #[test]
    fn beam_slant_returns_up_when_start_higher_than_stop() {
        // start guide note higher (lower offset = higher physically) than stop
        // Direction::Up uses highest offset; higher = larger number is LOWER physically
        // Start at offset 6 (lower), stop at offset 2 (higher): start_guide=6 > stop_guide=2
        // → 6 > 2 → BeamSlant::Up
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let note1 = make_note_with_info(0, eighth, 1, 6, 6); // highest=6
        let note2 = make_note_with_info(eighth, eighth, 2, 2, 2); // highest=2
        track.track.insert(0, note1);
        track.track.insert(eighth, note2);

        let mut ticks = FxHashMap::default();
        ticks.insert(0u64, 1u8);
        ticks.insert(eighth, 1u8);

        let beam = Beam {
            ticks,
            start: 0,
            stop: eighth,
            beam_segments: FxHashMap::default(),
            fractional_beams: FxHashMap::default(),
        };

        let slant = get_beam_slant(&beam, &track, &Direction::Up);
        assert!(matches!(slant, BeamSlant::Up));
    }

    #[test]
    fn beam_slant_returns_down_when_stop_higher_than_start() {
        // Direction::Up uses highest; stop_guide > start_guide → BeamSlant::Down
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let note1 = make_note_with_info(0, eighth, 1, 2, 2); // highest=2
        let note2 = make_note_with_info(eighth, eighth, 2, 6, 6); // highest=6
        track.track.insert(0, note1);
        track.track.insert(eighth, note2);

        let mut ticks = FxHashMap::default();
        ticks.insert(0u64, 1u8);
        ticks.insert(eighth, 1u8);

        let beam = Beam {
            ticks,
            start: 0,
            stop: eighth,
            beam_segments: FxHashMap::default(),
            fractional_beams: FxHashMap::default(),
        };

        let slant = get_beam_slant(&beam, &track, &Direction::Up);
        assert!(matches!(slant, BeamSlant::Down));
    }

    #[test]
    fn beam_slant_none_when_inner_note_breaks_slope_upward() {
        // start=2, stop=6 (slant would be Down), but middle note at 0 < both → flattened to None
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let note1 = make_note_with_info(0, eighth, 1, 2, 2);
        let note2 = make_note_with_info(eighth, eighth, 2, 0, 0); // below both start and stop
        let note3 = make_note_with_info(eighth * 2, eighth, 3, 6, 6);
        track.track.insert(0, note1);
        track.track.insert(eighth, note2);
        track.track.insert(eighth * 2, note3);

        let mut ticks = FxHashMap::default();
        ticks.insert(0u64, 1u8);
        ticks.insert(eighth, 1u8);
        ticks.insert(eighth * 2, 1u8);

        let beam = Beam {
            ticks,
            start: 0,
            stop: eighth * 2,
            beam_segments: FxHashMap::default(),
            fractional_beams: FxHashMap::default(),
        };

        let slant = get_beam_slant(&beam, &track, &Direction::Up);
        assert!(matches!(slant, BeamSlant::None));
    }

    #[test]
    fn get_stem_lengths_in_track_half_note_has_stem() {
        let mut track = NotationTrack::new(512);
        let half = NoteDuration::Half.to_ticks();
        let note = make_note_with_info(0, half, 1, 4, 4);
        track.track.insert(0, note);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Up);
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.contains_key(&0));
    }

    #[test]
    fn get_stem_lengths_in_track_eighth_note_up_has_stem() {
        let mut track = NotationTrack::new(512);
        let eighth = NoteDuration::Eighth.to_ticks();
        let note = make_note_with_info(0, eighth, 1, 4, 4);
        track.track.insert(0, note);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Up);
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.contains_key(&0));
        let stem = &result[&0];
        assert!(stem.tail.y <= stem.head.y);
    }

    #[test]
    fn get_stem_lengths_in_track_two_beamed_eighths_both_have_stems() {
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let note1 = make_note_with_info(0, eighth, 1, 4, 4);
        let note2 = make_note_with_info(eighth, eighth, 2, 4, 4);
        track.track.insert(0, note1);
        track.track.insert(eighth, note2);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Up);
        stem_directions.insert(eighth, Direction::Up);

        // Build a beam for these two notes
        let mut ticks = FxHashMap::default();
        ticks.insert(0u64, 1u8);
        ticks.insert(eighth, 1u8);
        let beam = Beam {
            ticks,
            start: 0,
            stop: eighth,
            beam_segments: FxHashMap::default(),
            fractional_beams: FxHashMap::default(),
        };
        let beams: Beams = vec![beam];

        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.contains_key(&0));
        assert!(result.contains_key(&eighth));
    }

    #[test]
    fn get_stem_lengths_in_track_note_on_line_up_direction() {
        // offset 0 is even (on staff line), direction Up
        let mut track = NotationTrack::new(512);
        let q = NoteDuration::Quarter.to_ticks();
        let note = make_note_with_info(0, q, 1, 0, 0); // highest=0 (on center line)
        track.track.insert(0, note);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Down);
        let beams: Beams = Vec::new();
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        assert!(result.contains_key(&0));
    }

    #[test]
    fn get_furthest_tail_with_two_note_beam_up() {
        // Two notes beamed stem-up: furthest tail should be negative (above staff)
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let note1 = make_note_with_info(0, eighth, 1, 4, 4);
        let note2 = make_note_with_info(eighth, eighth, 2, 4, 4);
        track.track.insert(0, note1);
        track.track.insert(eighth, note2);

        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(0, Direction::Up);
        stem_directions.insert(eighth, Direction::Up);

        let mut ticks = FxHashMap::default();
        ticks.insert(0u64, 1u8);
        ticks.insert(eighth, 1u8);
        let beam = Beam {
            ticks,
            start: 0,
            stop: eighth,
            beam_segments: FxHashMap::default(),
            fractional_beams: FxHashMap::default(),
        };
        let beams: Beams = vec![beam];

        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let hs = HorizontalSpacing::new(512);
        let result = get_stem_lengths_in_track(&track, &stem_directions, &beams, engrave, &hs);
        // Both stems should be present
        assert_eq!(result.len(), 2);
        // Direction::Up → tails should be above heads (negative y)
        for stem in result.values() {
            assert!(stem.tail.y <= stem.head.y);
        }
    }
}
