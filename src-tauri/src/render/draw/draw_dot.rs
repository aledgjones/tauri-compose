use crate::render::parse::get_dots::DotOffsets;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use skia_safe::Color;

pub fn draw_dots(
    origin: &Point,
    tick_dots: &DotOffsets,
    glyph_width: f32,
    has_post_shunt: bool,
    renderer: &mut Renderer,
) {
    let x = if has_post_shunt {
        origin.x + (glyph_width * 2.0) + 0.5
    } else {
        origin.x + glyph_width + 0.5
    };

    for dot_offset in tick_dots {
        renderer.circle(
            0.2,
            Color::BLACK,
            &Point {
                x,
                y: origin.y + (*dot_offset as f32 / 2.0),
            },
        );
    }
}
