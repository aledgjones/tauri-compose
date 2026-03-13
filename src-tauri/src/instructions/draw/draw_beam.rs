use crate::instructions::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_beams::{Beam, FractionalDirection};
use crate::instructions::parse::get_stem_directions::StemDirections;
use crate::instructions::parse::get_stem_lengths::{StemDef, StemLengths};
use crate::renderer::RendererContext;
use crate::score::components::duration::Direction;
use crate::score::components::measurements::Point;
use skia_safe::Color;

const BEAM_THICKNESS: f32 = 0.5;
const STEM_TWEAK: f32 = STAVE_LINE_WIDTH / 2.0;

/// Calculates the beam angle (rise/run) between two stem tips.
/// Returns 0.0 when the stems have the same x position (avoid division by zero).
pub(crate) fn compute_beam_angle(start: &StemDef, stop: &StemDef) -> f32 {
    if stop.tail.x != start.tail.x {
        (stop.tail.y - start.tail.y) / (stop.tail.x - start.tail.x)
    } else {
        0.0
    }
}

/// Returns the y-axis tweak needed when drawing beam shapes relative to stem tips.
/// Up-stems: 0 (beam sits on top of the tip), Down-stems: BEAM_THICKNESS (shift down).
pub(crate) fn compute_stem_length_tweak(direction: &Direction) -> f32 {
    match direction {
        Direction::Up => 0.0,
        Direction::Down => BEAM_THICKNESS,
    }
}

fn draw_beam_shape(
    origin: &Point,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    draw_to_stem_length_tweak: f32,
    renderer: &mut RendererContext,
) {
    renderer.shape(
        Color::BLACK,
        vec![
            Point {
                x: origin.x + start_x,
                y: origin.y + start_y - draw_to_stem_length_tweak,
            },
            Point {
                x: origin.x + end_x,
                y: origin.y + end_y - draw_to_stem_length_tweak,
            },
            Point {
                x: origin.x + end_x,
                y: origin.y + end_y + BEAM_THICKNESS - draw_to_stem_length_tweak,
            },
            Point {
                x: origin.x + start_x,
                y: origin.y + start_y + BEAM_THICKNESS - draw_to_stem_length_tweak,
            },
        ],
    );
}

fn draw_outer_beam(
    origin: &Point,
    start: &StemDef,
    stop: &StemDef,
    beam_angle: f32,
    draw_to_stem_length_tweak: f32,
    renderer: &mut RendererContext,
) {
    // Draw outer beam with calculated angle
    let start_x = start.tail.x - STEM_TWEAK;
    let stop_x = stop.tail.x + STEM_TWEAK;
    let start_y = start.tail.y;
    let stop_y = start_y + beam_angle * (stop_x - start_x);

    draw_beam_shape(
        origin,
        start_x,
        start_y,
        stop_x,
        stop_y,
        draw_to_stem_length_tweak,
        renderer,
    );
}

fn draw_inner_beams(
    origin: &Point,
    beam: &Beam,
    stem_lengths: &StemLengths,
    stem_directions: &StemDirections,
    beam_angle: f32,
    draw_to_stem_length_tweak: f32,
    renderer: &mut RendererContext,
) {
    // Draw inner beams using pre-calculated segments
    for (beam_index, segments) in &beam.beam_segments {
        let vertical_offset = *beam_index as f32 * 0.75; // 0.5 beam + 0.25 gap

        for segment in segments {
            if let (Some(start_stem), Some(end_stem)) = (
                stem_lengths.get(&segment.start_tick),
                stem_lengths.get(&segment.end_tick),
            ) {
                let start_x = start_stem.tail.x - STEM_TWEAK;
                let end_x = end_stem.tail.x + STEM_TWEAK;

                // Get stem direction multiplier (1 for down, -1 for up)
                let modifier = stem_directions[&segment.start_tick].to_modifier() as f32;

                // Apply consistent beam angle to inner beams
                let start_y = start_stem.tail.y + vertical_offset * -modifier;
                let end_y = start_y + beam_angle * (end_x - start_x);

                draw_beam_shape(
                    origin,
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    draw_to_stem_length_tweak,
                    renderer,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_fractional_beam(
    origin: &Point,
    current_stem: &StemDef,
    beam_angle: f32,
    beam_index: u8,
    stem_direction: &Direction,
    direction: FractionalDirection,
    draw_to_stem_length_tweak: f32,
    renderer: &mut RendererContext,
) {
    let vertical_offset = beam_index as f32 * 0.75; // 0.5 beam + 0.25 gap
    let modifier = stem_direction.to_modifier() as f32;

    // Use the overall beam angle for consistency
    let beam_length = 1.0; // 1 space wide
    let direction_modifier = match direction {
        FractionalDirection::Right => 1.0,
        FractionalDirection::Left => -1.0,
    };

    // Calculate beam start position
    let start_x = current_stem.tail.x - STEM_TWEAK;
    let start_y = current_stem.tail.y + vertical_offset * -modifier;

    // Calculate beam end position (1 space to the right or left, following the beam angle)
    let end_x = start_x + beam_length * direction_modifier;
    let end_y = start_y + beam_angle * beam_length * direction_modifier;

    draw_beam_shape(
        origin,
        start_x,
        start_y,
        end_x,
        end_y,
        draw_to_stem_length_tweak,
        renderer,
    );
}

pub fn draw_beam(
    origin: &Point,
    beam: &Beam,
    stem_lengths: &StemLengths,
    stem_directions: &StemDirections,
    renderer: &mut RendererContext,
) {
    let start = &stem_lengths[&beam.start];
    let stop = &stem_lengths[&beam.stop];

    let draw_to_stem_length_tweak = compute_stem_length_tweak(&stem_directions[&beam.start]);

    // Calculate beam angle for consistency across all beams in this group
    let beam_angle = compute_beam_angle(start, stop);

    // Draw outer beam
    draw_outer_beam(
        origin,
        start,
        stop,
        beam_angle,
        draw_to_stem_length_tweak,
        renderer,
    );

    // Draw inner beams
    draw_inner_beams(
        origin,
        beam,
        stem_lengths,
        stem_directions,
        beam_angle,
        draw_to_stem_length_tweak,
        renderer,
    );

    // Draw fractional beams using the same beam angle
    for (beam_index, fractional_beams) in &beam.fractional_beams {
        for fractional_beam in fractional_beams {
            if let Some(current_stem) = stem_lengths.get(&fractional_beam.tick) {
                let stem_direction = &stem_directions[&fractional_beam.tick];
                draw_fractional_beam(
                    origin,
                    current_stem,
                    beam_angle,
                    *beam_index,
                    stem_direction,
                    fractional_beam.direction,
                    draw_to_stem_length_tweak,
                    renderer,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_beam_angle, compute_stem_length_tweak, draw_beam};
    use crate::instructions::draw::test_helpers::make_two_note_beam;
    use crate::renderer::{test_helpers::make_test_renderer, DrawInstruction};
    use crate::score::components::duration::Direction;
    use crate::score::components::measurements::Point;

    // ─── pure helper tests ───────────────────────────────────────────────────

    #[test]
    fn beam_angle_flat_stems() {
        let (beam, stems, _) = make_two_note_beam();
        let start = &stems[&beam.start];
        let stop = &stems[&beam.stop];
        let angle = compute_beam_angle(start, stop);
        // both tails at y=-3.5, so angle = 0
        assert!((angle - 0.0).abs() < 1e-5);
    }

    #[test]
    fn beam_angle_slanted_stems() {
        use crate::instructions::draw::test_helpers::make_stem_def;
        use rustc_hash::FxHashMap;
        let mut stems = FxHashMap::default();
        stems.insert(0u64, make_stem_def(0.0, -3.5));
        stems.insert(8u64, make_stem_def(4.0, -4.5));
        let angle = compute_beam_angle(&stems[&0], &stems[&8]);
        // rise = -4.5 - (-3.5) = -1.0, run = 4.0; angle = -0.25
        assert!((angle - (-0.25)).abs() < 1e-5);
    }

    #[test]
    fn beam_angle_zero_division_guard() {
        use crate::instructions::draw::test_helpers::make_stem_def;
        let same = make_stem_def(2.0, -3.5);
        let angle = compute_beam_angle(&same, &same);
        assert_eq!(angle, 0.0);
    }

    #[test]
    fn stem_length_tweak_up_is_zero() {
        assert_eq!(compute_stem_length_tweak(&Direction::Up), 0.0);
    }

    #[test]
    fn stem_length_tweak_down_is_beam_thickness() {
        // BEAM_THICKNESS = 0.5
        assert!((compute_stem_length_tweak(&Direction::Down) - 0.5).abs() < 1e-5);
    }

    // ─── draw_beam integration tests ─────────────────────────────────────────

    #[test]
    fn two_note_beam_produces_at_least_one_shape() {
        let (beam, stem_lengths, stem_directions) = make_two_note_beam();
        let origin = Point { x: 0.0, y: 0.0 };
        let mut r = make_test_renderer();
        draw_beam(&origin, &beam, &stem_lengths, &stem_directions, &mut r);
        let shapes = r
            .instructions
            .iter()
            .filter(|i| matches!(i, DrawInstruction::Shape { .. }))
            .count();
        assert!(
            shapes >= 1,
            "expected at least 1 Shape instruction, got {}",
            shapes
        );
    }

    #[test]
    fn down_stem_beam_produces_shape() {
        use crate::instructions::draw::test_helpers::{make_stem_def, make_two_note_beam};
        use crate::score::components::duration::Direction;
        use rustc_hash::FxHashMap;
        let (beam, mut stem_lengths, _) = make_two_note_beam();
        // Flip stems to down direction (tails below heads)
        stem_lengths.insert(beam.start, make_stem_def(0.0, 3.5));
        stem_lengths.insert(beam.stop, make_stem_def(4.0, 3.5));
        let mut stem_directions = FxHashMap::default();
        stem_directions.insert(beam.start, Direction::Down);
        stem_directions.insert(beam.stop, Direction::Down);
        let origin = Point { x: 0.0, y: 0.0 };
        let mut r = make_test_renderer();
        draw_beam(&origin, &beam, &stem_lengths, &stem_directions, &mut r);
        let shapes = r
            .instructions
            .iter()
            .filter(|i| matches!(i, DrawInstruction::Shape { .. }))
            .count();
        assert!(shapes >= 1);
    }
}
