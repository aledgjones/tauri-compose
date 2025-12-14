use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::flows::Flow;
use super::draw_staves::STAVE_LINE_WIDTH;

pub fn draw_flag(
    position: &Point,
    notation: &crate::render::parse::get_written_durations::Notation,
    stem_direction: &crate::score::components::duration::Direction,
    stem_length: &crate::render::parse::get_stem_lengths::StemDef,
    flow: &Flow,
    renderer: &mut Renderer,
) {
    let glyph = notation.flag_glyph(stem_direction, flow.subdivisions);
    let left = position.x + stem_length.tail.x - (STAVE_LINE_WIDTH / 2.0);
    let top = position.y + stem_length.tail.y;

    renderer.text(
        &glyph,
        &Font {
            family: FontFamily::Bravura,
            size: 4.0,
            justify: Justify::Start,
            align: Align::Middle,
        },
        &Point {
            x: left,
            y: top,
        },
    );
}