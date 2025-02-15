use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::clef::Clef;

pub fn draw_clef(origin: &Point, clef: &Clef, is_full_size: bool, renderer: &mut Renderer) {
    if let Some(glyph) = clef.glyph() {
        renderer.text(
            &glyph,
            &Font {
                size: if is_full_size { 4.0 } else { 2.5 },
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Start,
            },
            &Point {
                x: origin.x,
                y: origin.y + (0.5 * clef.offset as f32),
            },
        );
    }
}
