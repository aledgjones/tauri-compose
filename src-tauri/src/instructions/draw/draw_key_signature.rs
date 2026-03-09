use crate::renderer::converter::Space;
use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::measurements::Point;
use crate::score::components::pitch::Accidental;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::clef::Clef;
use crate::score::entries::key_signature::KeySignature;
use crate::utils::UniqueId;
use skia_safe::Color;

pub struct KeySignatureDrawContext {
    pub flow_id: UniqueId,
    pub key_signature_id: UniqueId,
    pub tick: u64,
    pub stave_top: f32,
    pub is_selected: bool,
}

pub fn draw_key_signature(
    origin: &Point,
    clef: &Clef,
    key_signature: &KeySignature,
    negate: bool,
    context: &KeySignatureDrawContext,
    renderer: &mut RendererContext,
) {
    let glyph = if negate {
        Accidental::Natural.to_glyph()
    } else {
        key_signature.glyph()
    };
    let color = if context.is_selected {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
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
                color,
            );
        }

        if key_signature.offset != 0 {
            let metrics = key_signature.metrics();

            renderer.hit_targets.push(HitTarget {
                entry: crate::ui::sequencer::Selection::KeySignature {
                    flow_id: context.flow_id,
                    key_signature_id: context.key_signature_id,
                    tick: context.tick,
                },
                x: origin.x,
                y: origin.y - 3.0,
                width: metrics.width.max(1.0),
                height: metrics.height.max(1.0),
                popover_x: origin.x,
                popover_y: context.stave_top - 3.0,
            });
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
        Clef {
            draw_as: ClefDrawType::G,
            offset: 2,
            ..Default::default()
        }
    }

    #[test]
    fn c_major_zero_offset_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 0);

        draw_key_signature(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            &ks,
            false,
            &KeySignatureDrawContext {
                flow_id: 1,
                key_signature_id: 1,
                tick: 0,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer,
        );

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn one_sharp_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);

        draw_key_signature(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            &ks,
            false,
            &KeySignatureDrawContext {
                flow_id: 1,
                key_signature_id: 1,
                tick: 0,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
    }

    #[test]
    fn three_flats_produces_three_text_instructions() {
        let mut renderer = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -3);

        draw_key_signature(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            &ks,
            false,
            &KeySignatureDrawContext {
                flow_id: 1,
                key_signature_id: 1,
                tick: 0,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 3);
    }

    #[test]
    fn negate_true_uses_natural_glyph() {
        let mut renderer_sharp = make_test_renderer();
        let mut renderer_natural = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);

        draw_key_signature(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            &ks,
            false,
            &KeySignatureDrawContext {
                flow_id: 1,
                key_signature_id: 1,
                tick: 0,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer_sharp,
        );
        draw_key_signature(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            &ks,
            true,
            &KeySignatureDrawContext {
                flow_id: 1,
                key_signature_id: 1,
                tick: 0,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer_natural,
        );

        if let (
            DrawInstruction::Text {
                text: sharp_text, ..
            },
            DrawInstruction::Text { text: nat_text, .. },
        ) = (
            &renderer_sharp.instructions[0],
            &renderer_natural.instructions[0],
        ) {
            assert_ne!(sharp_text, nat_text);
            let natural_glyph = crate::score::components::pitch::Accidental::Natural.to_glyph();
            assert_eq!(*nat_text, natural_glyph);
        } else {
            panic!("expected Text instructions");
        }
    }

    #[test]
    fn percussion_clef_produces_no_instructions_even_with_key_sig() {
        let mut renderer = make_test_renderer();
        let clef = Clef {
            draw_as: ClefDrawType::Percussion,
            ..Default::default()
        };
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 3);

        draw_key_signature(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            &ks,
            false,
            &KeySignatureDrawContext {
                flow_id: 1,
                key_signature_id: 1,
                tick: 0,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer,
        );

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn selected_key_signature_uses_highlight_and_emits_hit_target() {
        let mut renderer = make_test_renderer();
        let clef = make_g_clef();
        let ks = KeySignature::new(7, 64, KeySignatureMode::Major, 2);

        draw_key_signature(
            &Point { x: 1.0, y: 4.0 },
            &clef,
            &ks,
            false,
            &KeySignatureDrawContext {
                flow_id: 5,
                key_signature_id: 7,
                tick: 64,
                stave_top: 4.0,
                is_selected: true,
            },
            &mut renderer,
        );

        assert!(!renderer.hit_targets.is_empty());
        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { color, .. } if *color == Color::from_rgb(14, 165, 233))
        }));
    }
}
