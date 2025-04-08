use crate::render::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_stem_lengths::StemDef;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use skia_safe::Color;

pub fn draw_beam(origin: &Point, start: &StemDef, stop: &StemDef, renderer: &mut Renderer) {
    let stem_tweek = STAVE_LINE_WIDTH / 2.0;

    renderer.shape(
        Color::BLACK,
        vec![
            Point {
                x: origin.x + start.tail.x - stem_tweek,
                y: origin.y + start.tail.y,
            },
            Point {
                x: origin.x + stop.tail.x + stem_tweek,
                y: origin.y + stop.tail.y,
            },
            Point {
                x: origin.x + stop.tail.x + stem_tweek,
                y: origin.y + stop.tail.y + 0.5,
            },
            Point {
                x: origin.x + start.tail.x - stem_tweek,
                y: origin.y + start.tail.y + 0.5,
            },
        ],
    );
}
