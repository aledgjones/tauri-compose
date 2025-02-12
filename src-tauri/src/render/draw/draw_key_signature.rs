use crate::renderer::converter::Space;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::pitch::Accidental;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::clef::Clef;
use crate::score::entries::key_signature::KeySignature;

pub fn draw_key_signature(
    origin: &Point,
    clef: &Clef,
    key_signature: &KeySignature,
    negate: bool,
    renderer: &mut Renderer,
) {
    let glyph = if negate {
        Accidental::Natural.to_glyph()
    } else {
        key_signature.glyph()
    };

    if let Some(pattern) = key_signature.pattern(clef) {
        for i in 0..key_signature.offset.abs() {
            renderer.text(
                &glyph,
                &Font {
                    family: FontFamily::Bravura,
                    justify: Justify::Start,
                    align: Align::Bottom,
                    size: 4.0,
                },
                &Point {
                    x: origin.x + (i as Space),
                    y: origin.y + 0.5 * pattern[i as usize] as Space,
                },
            );
        }
    }
}
