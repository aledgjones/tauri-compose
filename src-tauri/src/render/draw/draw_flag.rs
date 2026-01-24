use super::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_stem_lengths::StemDef;
use crate::render::parse::get_written_durations::Notation;
use crate::renderer::RendererContext;
use crate::score::components::duration::Direction;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use skia_safe::Color;

pub fn draw_flag(
    position: &Point,
    notation: &Notation,
    stem_direction: &Direction,
    stem_length: &StemDef,
    renderer: &mut RendererContext,
) {
    let glyph = notation.flag_glyph(stem_direction);
    let left = position.x + stem_length.tail.x - (STAVE_LINE_WIDTH / 2.0);
    let top = position.y + stem_length.tail.y;

    renderer.text(
        &glyph,
        &Font {
            family: FontFamily::Bravura,
            size: 4.0,
            justify: Justify::Start,
            align: Align::Bottom,
        },
        &Point { x: left, y: top },
        Color::BLACK,
    );
}
