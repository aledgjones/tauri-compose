use crate::render::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_beams::{Beam, FractionalDirection};
use crate::render::parse::get_stem_directions::StemDirections;
use crate::render::parse::get_stem_lengths::{StemDef, StemLengths};
use crate::renderer::Renderer;
use crate::score::components::duration::Direction;
use crate::score::components::measurements::Point;
use skia_safe::Color;

const BEAM_THICKNESS: f32 = 0.5;

fn draw_outer_beam(
    origin: &Point,
    start: &StemDef,
    stop: &StemDef,
    beam_angle: f32,
    draw_to_stem_length_tweak: f32,
    renderer: &mut Renderer,
) {
    let stem_tweek = STAVE_LINE_WIDTH / 2.0;

    // Draw outer beam with calculated angle
    let start_x = start.tail.x - stem_tweek;
    let stop_x = stop.tail.x + stem_tweek;
    let start_y = start.tail.y;
    let stop_y = start_y + beam_angle * (stop_x - start_x);

    renderer.shape(
        Color::BLACK,
        vec![
            Point {
                x: origin.x + start_x,
                y: origin.y + start_y - draw_to_stem_length_tweak,
            },
            Point {
                x: origin.x + stop_x,
                y: origin.y + stop_y - draw_to_stem_length_tweak,
            },
            Point {
                x: origin.x + stop_x,
                y: origin.y + stop_y + BEAM_THICKNESS - draw_to_stem_length_tweak,
            },
            Point {
                x: origin.x + start_x,
                y: origin.y + start_y + BEAM_THICKNESS - draw_to_stem_length_tweak,
            },
        ],
    );
}

fn draw_inner_beams(
    origin: &Point,
    beam: &Beam,
    stem_lengths: &StemLengths,
    stem_directions: &StemDirections,
    beam_angle: f32,
    draw_to_stem_length_tweak: f32,
    renderer: &mut Renderer,
) {
    let stem_tweek = STAVE_LINE_WIDTH / 2.0;

    // Draw inner beams using pre-calculated segments
    for (beam_index, segments) in &beam.beam_segments {
        let vertical_offset = *beam_index as f32 * 0.75; // 0.5 beam + 0.25 gap

        for segment in segments {
            if let (Some(start_stem), Some(end_stem)) = (
                stem_lengths.get(&segment.start_tick),
                stem_lengths.get(&segment.end_tick),
            ) {
                let start_x = start_stem.tail.x - stem_tweek;
                let end_x = end_stem.tail.x + stem_tweek;

                // Get stem direction multiplier (1 for down, -1 for up)
                let modifier = stem_directions[&segment.start_tick].to_modifier() as f32;

                // Apply consistent beam angle to inner beams
                let start_y = start_stem.tail.y + vertical_offset * -modifier;
                let end_y = start_y + beam_angle * (end_x - start_x);

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
    renderer: &mut Renderer,
) {
    let stem_tweek = STAVE_LINE_WIDTH / 2.0;
    let vertical_offset = beam_index as f32 * 0.75; // 0.5 beam + 0.25 gap
    let modifier = stem_direction.to_modifier() as f32;

    // Use the overall beam angle for consistency
    let beam_length = 1.0; // 1 space wide
    let direction_modifier = match direction {
        FractionalDirection::Right => 1.0,
        FractionalDirection::Left => -1.0,
    };

    // Calculate beam start position
    let start_x = current_stem.tail.x - stem_tweek;
    let start_y = current_stem.tail.y + vertical_offset * -modifier;

    // Calculate beam end position (1 space to the right or left, following the beam angle)
    let end_x = start_x + beam_length * direction_modifier;
    let end_y = start_y + beam_angle * beam_length * direction_modifier;

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

pub fn draw_beam(
    origin: &Point,
    beam: &Beam,
    stem_lengths: &StemLengths,
    stem_directions: &StemDirections,
    renderer: &mut Renderer,
) {
    let start = &stem_lengths[&beam.start];
    let stop = &stem_lengths[&beam.stop];

    let draw_to_stem_length_tweak = match stem_directions[&beam.start] {
        Direction::Up => 0.0,
        Direction::Down => BEAM_THICKNESS,
    };

    // Calculate beam angle for consistency across all beams in this group
    let beam_angle = if stop.tail.x != start.tail.x {
        (stop.tail.y - start.tail.y) / (stop.tail.x - start.tail.x)
    } else {
        0.0
    };

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
