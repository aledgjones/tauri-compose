use crate::renderer::converter::Space;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::pitch::Accidental;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::clef::Clef;
use crate::score::entries::key_signature::KeySignature;
use skia_safe::Color;

pub fn draw_key_signature(
    origin: &Point,
    clef: &Clef,
    key_signature: &KeySignature,
    negate: bool,
    renderer: &mut RendererContext,
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
                Color::BLACK,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::entries::clef::{Clef, ClefDrawType};
    use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};

    fn make_g_clef() -> Clef {
        Clef { draw_as: ClefDrawType::G, offset: 2, ..Default::default() }
    }

    #[test]
    fn c_major_zero_offset_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 0);

        draw_key_signature(&Point { x: 0.0, y: 0.0 }, &clef, &ks, false, &mut renderer);

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn one_sharp_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);

        draw_key_signature(&Point { x: 0.0, y: 0.0 }, &clef, &ks, false, &mut renderer);

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(renderer.instructions[0], DrawInstruction::Text { .. }));
    }

    #[test]
    fn three_flats_produces_three_text_instructions() {
        let mut renderer = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -3);

        draw_key_signature(&Point { x: 0.0, y: 0.0 }, &clef, &ks, false, &mut renderer);

        assert_eq!(renderer.instructions.len(), 3);
    }

    #[test]
    fn negate_true_uses_natural_glyph() {
        let mut renderer_sharp = make_test_renderer();
        let mut renderer_natural = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);

        draw_key_signature(&Point { x: 0.0, y: 0.0 }, &clef, &ks, false, &mut renderer_sharp);
        draw_key_signature(&Point { x: 0.0, y: 0.0 }, &clef, &ks, true, &mut renderer_natural);

        if let (DrawInstruction::Text { text: sharp_text, .. }, DrawInstruction::Text { text: nat_text, .. }) =
            (&renderer_sharp.instructions[0], &renderer_natural.instructions[0])
        {
            assert_ne!(sharp_text, nat_text);
            let natural_glyph = crate::score::components::pitch::Accidental::Natural.to_glyph();
            assert_eq!(*nat_text, natural_glyph);
        } else {
            panic!("expected Text instructions");
        }
    }

    #[test]
    fn hidden_clef_produces_no_instructions_even_with_key_sig() {
        let mut renderer = make_test_renderer();
        let clef = Clef { draw_as: ClefDrawType::Hidden, ..Default::default() };
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 3);

        draw_key_signature(&Point { x: 0.0, y: 0.0 }, &clef, &ks, false, &mut renderer);

        assert!(renderer.instructions.is_empty());
    }
}
