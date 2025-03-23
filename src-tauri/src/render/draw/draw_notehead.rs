use crate::render::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::render::parse::get_shunts::{Shunt, Shunts};
use crate::render::parse::get_tone_offsets::ToneVerticalOffsets;
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
    vertical_offset: i8,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    renderer: &mut Renderer,
) {
    let start = horizontal_spacing.get(tick, Slot::NoteSpacing);

    let left = match &shunts.by_id[&(tick, tone.id)] {
        Shunt::Pre => origin.x + start.x - glyph_width,
        Shunt::None => origin.x + start.x,
        Shunt::Post => origin.x + start.x + glyph_width,
    };

    let top = origin.y + (vertical_offset as f32 / 2.0);

    renderer.text(
        glyph,
        &Font {
            family: FontFamily::Bravura,
            size: 4.0,
            justify: Justify::Start,
            align: Align::Bottom,
        },
        &Point { x: left, y: top },
    );
}
