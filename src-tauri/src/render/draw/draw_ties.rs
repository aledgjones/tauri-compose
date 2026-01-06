use crate::render::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::render::parse::get_shunts::{Shunt, Shunts};
use crate::render::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::render::parse::get_written_durations::Notation;
use crate::renderer::Renderer;
use crate::score::components::duration::{Direction, Ticks};
use crate::score::components::measurements::CurvePoint;
use crate::score::components::measurements::Point;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use skia_safe::Color;

type TieDirections = FxHashMap<UniqueId, Direction>;

pub fn get_tie_directions(
    entry: &Notation,
    stem_direction: &Direction,
    tone_offsets: &ToneVerticalOffsets,
) -> TieDirections {
    let mut output: TieDirections = FxHashMap::default();

    let count = entry.ties.len();

    let middle = match stem_direction {
        Direction::Up => (count as f32 / 2.0).ceil() as usize,
        Direction::Down => (count as f32 / 2.0).floor() as usize,
    };

    let mut sorted_tones = entry.tones.clone();
    sorted_tones.sort_by_key(|tone| tone_offsets[&tone.id]);
    sorted_tones.reverse();
    let sorted = sorted_tones
        .iter()
        .filter(|tone| entry.ties.contains(&tone.id));

    for (i, tone) in sorted.enumerate() {
        if i < middle {
            output.insert(tone.id, Direction::Down);
        } else {
            output.insert(tone.id, Direction::Up);
        }
    }

    output
}

fn tie_points_y(
    y: f32,
    start: &Notation,
    tie_direction: &Direction,
    width: f32,
    offset: i8,
) -> [f32; 3] {
    let is_on_line = offset % 2 == 0;
    let is_wide = width > 10.0;
    let direction_modifier = tie_direction.to_modifier() as f32 * -1.0;

    let (ends_tweak, middle_tweak) = match start.is_chord() {
        true => {
            let ends_tweak = 0.25 * direction_modifier;
            let middle_tweak = match is_wide {
                true => match is_on_line {
                    true => 0.75 * direction_modifier,
                    false => 0.5 * direction_modifier,
                },
                false => 0.5 * direction_modifier,
            };
            (ends_tweak, middle_tweak)
        }
        false => {
            let ends_tweak = 0.74 * direction_modifier;
            let middle_tweak = match is_wide {
                true => match is_on_line {
                    true => 0.75 * direction_modifier,
                    false => 0.5 * direction_modifier,
                },
                false => 0.5 * direction_modifier,
            };
            (ends_tweak, middle_tweak)
        }
    };

    let ends = y + (offset as f32 / 2.0) - ends_tweak;
    let middle = ends - middle_tweak;

    [ends, middle, ends]
}

fn start_x(
    start: &Notation,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    tie_direction: &Direction,
    stem_direction: &Direction,
    subdivisions: Ticks,
    offset: i8,
) -> f32 {
    let x = horizontal_spacing.get(start.tick, Slot::NoteSpacing).x;

    let after_pre = x + 0.2;
    let after_note = x + start.glyph_metrics(subdivisions).width + 0.2;
    let after_post = x + (start.glyph_metrics(subdivisions).width * 2.0) + 0.2;

    let shunt = shunts.by_offset.get(&(start.tick, offset)).unwrap();
    let next_shunt = shunts
        .by_offset
        .get(&(start.tick, offset + tie_direction.to_modifier()));

    match stem_direction {
        Direction::Up => {
            if let Shunt::Post = shunt {
                return after_post;
            }

            if let Some(Shunt::Post) = next_shunt {
                return after_post;
            }
        }
        Direction::Down => {
            if shunt == &Shunt::Pre && next_shunt.is_none() {
                return after_pre;
            }
        }
    };

    after_note
}

fn stop_x(
    stop: &Notation,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    tie_direction: &Direction,
    stem_direction: &Direction,
    subdivisions: Ticks,
    offset: i8,
) -> f32 {
    let x = horizontal_spacing.get(stop.tick, Slot::NoteSpacing).x;

    let before_pre = x - stop.glyph_metrics(subdivisions).width - 0.2;
    let before_note = x - 0.2;
    let before_post = x - 0.2;

    let shunt = shunts.by_offset.get(&(stop.tick, offset)).unwrap();
    let next_shunt = shunts
        .by_offset
        .get(&(stop.tick, offset + tie_direction.to_modifier()));

    match stem_direction {
        Direction::Down => {
            if let Shunt::Pre = shunt {
                return before_pre;
            }

            if let Some(Shunt::Pre) = next_shunt {
                return before_pre;
            }
        }
        Direction::Up => {
            if shunt == &Shunt::Post && next_shunt.is_none() {
                return before_post;
            }
        }
    };

    before_note
}

#[allow(clippy::too_many_arguments)]
fn tie_points_x(
    x: f32,
    start: &Notation,
    stop: &Notation,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    tie_direction: &Direction,
    stem_direction: &Direction,
    subdivisions: Ticks,
    offset: i8,
) -> [f32; 3] {
    let (start_x, stop_x) = match start.is_chord() {
        true => (
            x + start_x(
                start,
                horizontal_spacing,
                shunts,
                tie_direction,
                stem_direction,
                subdivisions,
                offset,
            ),
            x + stop_x(
                stop,
                horizontal_spacing,
                shunts,
                tie_direction,
                stem_direction,
                subdivisions,
                offset,
            ),
        ),
        false => {
            let start_x = x
                + horizontal_spacing.get(start.tick, Slot::NoteSpacing).x
                + (start.glyph_metrics(subdivisions).width / 2.0)
                + 0.1;

            let stop_x = x
                + horizontal_spacing.get(stop.tick, Slot::NoteSpacing).x
                + (start.glyph_metrics(subdivisions).width / 2.0)
                - 0.1;

            (start_x, stop_x)
        }
    };

    let middle_x = start_x + ((stop_x - start_x) / 2.0);

    [start_x, middle_x, stop_x]
}

#[allow(clippy::too_many_arguments)]
pub fn draw_tie(
    origin: &Point,
    tone_id: &UniqueId,
    start: &Notation,
    stop: &Notation,
    tie_direction: &Direction,
    stem_direction: &Direction,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    tone_offsets: &ToneVerticalOffsets,
    subdivisions: Ticks,
    renderer: &mut Renderer,
) {
    let offset = *tone_offsets.get(tone_id).unwrap();

    let [start_x, middle_x, stop_x] = tie_points_x(
        origin.x,
        start,
        stop,
        horizontal_spacing,
        shunts,
        tie_direction,
        stem_direction,
        subdivisions,
        offset,
    );
    let [start_y, middle_y, stop_y] =
        tie_points_y(origin.y, start, tie_direction, stop_x - start_x, offset);

    renderer.curve(
        Color::BLACK,
        [
            CurvePoint {
                x: start_x,
                y: start_y,
                thickness: 0.125,
            },
            CurvePoint {
                x: middle_x,
                y: middle_y,
                thickness: 0.2,
            },
            CurvePoint {
                x: stop_x,
                y: stop_y,
                thickness: 0.125,
            },
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ties(
    origin: &Point,
    notation: &crate::render::parse::get_written_durations::NotationTrack,
    stem_directions: &crate::render::parse::get_stem_directions::StemDirections,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    tone_offsets: &ToneVerticalOffsets,
    subdivisions: Ticks,
    renderer: &mut Renderer,
) {
    for (tick, entry) in &notation.track {
        if entry.has_tie() {
            let stem_direction = stem_directions.get(tick).unwrap();
            let tie_directions = get_tie_directions(entry, stem_direction, tone_offsets);

            for (tone_id, tie_direction) in tie_directions {
                draw_tie(
                    origin,
                    &tone_id,
                    entry,
                    notation.track.get(&(tick + entry.duration)).unwrap(),
                    &tie_direction,
                    stem_direction,
                    horizontal_spacing,
                    shunts,
                    tone_offsets,
                    subdivisions,
                    renderer,
                )
            }
        }
    }
}
