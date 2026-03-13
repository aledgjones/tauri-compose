use crate::instructions::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::instructions::parse::get_shunts::{Shunt, Shunts};
use crate::instructions::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::instructions::parse::get_written_durations::Notation;
use crate::renderer::RendererContext;
use crate::score::components::duration::Direction;
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
    let direction_modifier = -(tie_direction.to_modifier() as f32);

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
    offset: i8,
) -> f32 {
    let x = horizontal_spacing.get(start.tick, Slot::NoteSpacing).x;
    let metrics = start.glyph_metrics();
    let glyph_width = metrics.total_width();

    let after_pre = x + 0.2;
    let after_note = x + glyph_width + 0.2;
    let after_post = x + (glyph_width * 2.0) + 0.2;

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
    offset: i8,
) -> f32 {
    let x = horizontal_spacing.get(stop.tick, Slot::NoteSpacing).x;
    let metrics = stop.glyph_metrics();
    let glyph_width = metrics.total_width();

    let before_pre = x - glyph_width - 0.2;
    let before_note = x - 0.2;
    let before_post = x + glyph_width - 0.2;

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
                offset,
            ),
            x + stop_x(
                stop,
                horizontal_spacing,
                shunts,
                tie_direction,
                stem_direction,
                offset,
            ),
        ),
        false => {
            let metrics = start.glyph_metrics();
            let glyph_width = metrics.total_width();

            let start_x = x
                + horizontal_spacing.get(start.tick, Slot::NoteSpacing).x
                + (glyph_width / 2.0)
                + 0.1;

            let stop_x =
                x + horizontal_spacing.get(stop.tick, Slot::NoteSpacing).x + (glyph_width / 2.0)
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
    renderer: &mut RendererContext,
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
                thickness: 0.25,
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
    notation: &crate::instructions::parse::get_written_durations::NotationTrack,
    stem_directions: &crate::instructions::parse::get_stem_directions::StemDirections,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    tone_offsets: &ToneVerticalOffsets,
    renderer: &mut RendererContext,
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
                    renderer,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::draw_tie;
    use crate::instructions::parse::get_horizontal_spacing::HorizontalSpacing;
    use crate::instructions::parse::get_shunts::Shunts;
    use crate::instructions::parse::get_written_durations::Notation;
    use crate::renderer::{test_helpers::make_test_renderer, DrawInstruction};
    use crate::score::components::articulation::Articulations;
    use crate::score::components::duration::Direction;
    use crate::score::components::measurements::Point;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tone::Tone;
    use rustc_hash::FxHashMap;

    fn empty_shunts() -> Shunts {
        Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        }
    }

    fn make_single_tone_notation(tick: u64, duration: u64, tone_id: u64) -> Notation {
        let mut n = Notation::new(tick, duration);
        let tone = Tone::new(
            tone_id,
            tick,
            duration,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        n.tones.push(tone);
        n
    }

    #[test]
    fn draw_tie_single_tone_produces_curve() {
        let tone_id = 1u64;
        let start = make_single_tone_notation(0, 32, tone_id);
        let stop = make_single_tone_notation(32, 32, tone_id);
        let mut tone_offsets = FxHashMap::default();
        tone_offsets.insert(tone_id, 0i8);
        let spacing = HorizontalSpacing::new(64);
        let shunts = empty_shunts();
        let origin = Point { x: 0.0, y: 0.0 };
        let mut r = make_test_renderer();
        draw_tie(
            &origin,
            &tone_id,
            &start,
            &stop,
            &Direction::Up,
            &Direction::Up,
            &spacing,
            &shunts,
            &tone_offsets,
            &mut r,
        );
        assert_eq!(
            r.instructions
                .iter()
                .filter(|i| matches!(i, DrawInstruction::Curve { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn draw_tie_down_direction_produces_curve() {
        let tone_id = 2u64;
        let start = make_single_tone_notation(0, 32, tone_id);
        let stop = make_single_tone_notation(32, 32, tone_id);
        let mut tone_offsets = FxHashMap::default();
        tone_offsets.insert(tone_id, 1i8); // odd offset (space between lines)
        let spacing = HorizontalSpacing::new(64);
        let shunts = empty_shunts();
        let origin = Point { x: 0.0, y: 0.0 };
        let mut r = make_test_renderer();
        draw_tie(
            &origin,
            &tone_id,
            &start,
            &stop,
            &Direction::Down,
            &Direction::Down,
            &spacing,
            &shunts,
            &tone_offsets,
            &mut r,
        );
        assert_eq!(
            r.instructions
                .iter()
                .filter(|i| matches!(i, DrawInstruction::Curve { .. }))
                .count(),
            1
        );
    }
}
