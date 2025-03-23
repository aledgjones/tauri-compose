use crate::render::parse::get_shunts::{Shunt, Shunts};
use crate::renderer::Renderer;
use crate::score::components::duration::Tick;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::tone::Tone;

pub fn draw_notehead(
    origin: &Point,
    tick: Tick,
    tone: &Tone,
    glyph: &str,
    glyph_width: f32,
    shunts: &Shunts,
    renderer: &mut Renderer,
) {
    let left = match &shunts.by_id[&(tick, tone.id)] {
        Shunt::Pre => origin.x - glyph_width,
        Shunt::None => origin.x,
        Shunt::Post => origin.x + glyph_width,
    };

    renderer.text(
        glyph,
        &Font {
            family: FontFamily::Bravura,
            size: 4.0,
            justify: Justify::Start,
            align: Align::Bottom,
        },
        &Point {
            x: left,
            y: origin.y,
        },
    );
}
