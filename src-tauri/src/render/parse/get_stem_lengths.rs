use super::get_beams::{Beam, Beams, BeamsByTrack};
use super::get_horizontal_spacing::{HorizontalSpacing, Slot};
use super::get_stem_directions::{StemDirections, StemDirectionsByTrack};
use super::get_written_durations::{Notation, NotationByTrack, NotationTrack};
use crate::render::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::renderer::converter::Space;
use crate::score::components::duration::{Direction, Tick};
use crate::score::components::measurements::Point;
use crate::score::engraves::Engrave;
use crate::utils::UniqueId;
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

fn get_natural_stem_length(x: Space, entry: &Notation, stem_direction: &Direction) -> StemDef {
    let info = entry.info.clone().unwrap();

    let (x, head, tail) = match stem_direction {
        Direction::Up => {
            let head = (info.lowest as f32 - 0.5) / 2.0;
            let mut tail = (info.highest as f32 - 0.5) / 2.0 - 3.25;
            if tail > 0.0 {
                tail = 0.0
            }

            let x = x + entry.glyph_metrics().width;

            (x - (STAVE_LINE_WIDTH / 2.0), head, tail)
        }
        Direction::Down => {
            let head = (info.highest as f32 + 0.5) / 2.0;
            let mut tail = (info.lowest as f32 + 0.5) / 2.0 + 3.25;
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
            let x = horizontal_spacing.get(*tick, Slot::NoteSpacing).x;
            let stem_direction = &stem_directions[tick];
            output.insert(*tick, get_natural_stem_length(x, entry, stem_direction));
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
    let mut output: StemLengthsByTrack = FxHashMap::default();

    for (track_id, track) in tracks {
        let stem_directions = &stem_directions_by_track[track_id];
        let beams = &beams_by_track[track_id];
        let stem_lengths =
            get_stem_lengths_in_track(track, stem_directions, beams, engrave, horizontal_spacing);
        output.insert(*track_id, stem_lengths);
    }

    output
}
