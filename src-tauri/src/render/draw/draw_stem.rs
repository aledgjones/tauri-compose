use skia_safe::Color;

use super::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_stem_lengths::StemDef;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;

pub fn draw_stem(origin: &Point, def: &StemDef, renderer: &mut Renderer) {
    renderer.line(
        STAVE_LINE_WIDTH,
        Color::BLACK,
        vec![
            Point {
                x: origin.x + def.head.x,
                y: origin.y + def.head.y,
            },
            Point {
                x: origin.x + def.tail.x,
                y: origin.y + def.tail.y,
            },
        ],
    );
}
