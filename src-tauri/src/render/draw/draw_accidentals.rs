use crate::render::parse::get_accidentals::AccidentalNotation;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::tone::Tone;
use crate::utils::UniqueId;
use skia_safe::Color;

pub fn draw_accidental(
    origin: &Point,
    accidental: &AccidentalNotation,
    tone: &Tone,
    selection: Option<UniqueId>,
    renderer: &mut Renderer,
) {
    let glyph = tone.pitch.accidental.to_glyph();
    let left = origin.x - ((accidental.slot as f32 - 1.0) * 1.1) - 0.25;

    let color = if selection == Some(tone.id) {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };

    renderer.text(
        &glyph,
        &Font {
            size: 4.0,
            family: FontFamily::Bravura,
            align: Align::Bottom,
            justify: Justify::End,
        },
        &Point {
            x: left,
            y: origin.y,
        },
        color,
    );
}
