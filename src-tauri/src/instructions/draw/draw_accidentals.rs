use crate::instructions::parse::get_accidentals::AccidentalNotation;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::tone::Tone;
use crate::ui::sequencer::Selection;
use skia_safe::Color;

fn get_accidental_slot_offset(slot: u8, glyph_width: f32, has_pre_shunt: bool) -> f32 {
    let mut offset = 0.0;

    for i in 1..slot {
        if i == 1 && has_pre_shunt {
            offset += glyph_width
        } else {
            offset += 1.1
        }
    }

    offset + 0.25
}

pub fn draw_accidental(
    origin: &Point,
    accidental: &AccidentalNotation,
    tone: &Tone,
    glyph_width: f32,
    has_pre_shunt: bool,
    selection: Option<&Selection>,
    renderer: &mut RendererContext,
) {
    let glyph = tone.pitch.accidental.to_glyph();
    let slot_offset = get_accidental_slot_offset(accidental.slot, glyph_width, has_pre_shunt);
    let left = origin.x - slot_offset;

    let color = if matches!(selection, Some(Selection::Tone { tone_id, .. }) if *tone_id == tone.id)
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_one_no_pre_shunt_returns_base_offset() {
        // slot=1: loop 1..1 runs 0 iterations, so offset = 0 + 0.25
        let result = get_accidental_slot_offset(1, 1.0, false);
        assert!((result - 0.25).abs() < 1e-6);
    }

    #[test]
    fn slot_one_with_pre_shunt_returns_base_offset() {
        // slot=1: same loop, pre_shunt not applied at i=1 when loop doesn't run
        let result = get_accidental_slot_offset(1, 1.0, true);
        assert!((result - 0.25).abs() < 1e-6);
    }

    #[test]
    fn slot_two_no_pre_shunt_adds_1_1() {
        // slot=2: loop runs i=1, no pre_shunt → offset += 1.1 → 1.1 + 0.25 = 1.35
        let result = get_accidental_slot_offset(2, 1.0, false);
        assert!((result - 1.35).abs() < 1e-5);
    }

    #[test]
    fn slot_two_with_pre_shunt_adds_glyph_width() {
        // slot=2: loop runs i=1, has_pre_shunt=true → offset += glyph_width(2.0) → 2.0 + 0.25 = 2.25
        let result = get_accidental_slot_offset(2, 2.0, true);
        assert!((result - 2.25).abs() < 1e-5);
    }

    #[test]
    fn slot_three_no_pre_shunt_adds_twice() {
        // slot=3: loop i=1 → 1.1, i=2 → 1.1, total = 2.2 + 0.25 = 2.45
        let result = get_accidental_slot_offset(3, 1.0, false);
        assert!((result - 2.45).abs() < 1e-5);
    }

    #[test]
    fn slot_three_with_pre_shunt_adds_glyph_then_regular() {
        // slot=3: i=1 has_pre_shunt → +1.5, i=2 no shunt → +1.1 → 2.6 + 0.25 = 2.85
        let result = get_accidental_slot_offset(3, 1.5, true);
        assert!((result - 2.85).abs() < 1e-5);
    }

    // --- draw_accidental tests ---

    fn make_sharp_tone(id: u64) -> Tone {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        Tone::new(
            id,
            0,
            32,
            Pitch::new(61, Accidental::Sharp),
            Velocity::new(64),
            Articulations::default(),
        )
    }

    #[test]
    fn draw_accidental_produces_one_text_instruction() {
        use crate::renderer::test_helpers::make_test_renderer;
        use crate::renderer::DrawInstruction;
        let mut renderer = make_test_renderer();
        let tone = make_sharp_tone(1);
        let an = AccidentalNotation {
            tone_id: 1,
            slot: 1,
        };

        draw_accidental(
            &Point { x: 5.0, y: 0.0 },
            &an,
            &tone,
            1.175,
            false,
            None,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
    }

    #[test]
    fn draw_accidental_uses_black_when_not_selected() {
        use crate::renderer::test_helpers::make_test_renderer;
        use crate::renderer::DrawInstruction;
        use skia_safe::Color;
        let mut renderer = make_test_renderer();
        let tone = make_sharp_tone(1);
        let an = AccidentalNotation {
            tone_id: 1,
            slot: 1,
        };

        draw_accidental(
            &Point { x: 5.0, y: 0.0 },
            &an,
            &tone,
            1.175,
            false,
            None,
            &mut renderer,
        );

        if let DrawInstruction::Text { color, .. } = &renderer.instructions[0] {
            assert_eq!(*color, Color::BLACK);
        } else {
            panic!("expected Text instruction");
        }
    }

    #[test]
    fn draw_accidental_uses_blue_when_tone_is_selected() {
        use crate::renderer::test_helpers::make_test_renderer;
        use crate::renderer::DrawInstruction;
        use crate::ui::sequencer::Selection;
        use skia_safe::Color;
        let mut renderer = make_test_renderer();
        let tone = make_sharp_tone(42);
        let an = AccidentalNotation {
            tone_id: 42,
            slot: 1,
        };
        let selection = Selection::Tone {
            flow_id: 1,
            instrument_id: 1,
            stave_id: 1,
            track_id: 1,
            tone_id: 42,
            tuplet_id: None,
        };

        draw_accidental(
            &Point { x: 5.0, y: 0.0 },
            &an,
            &tone,
            1.175,
            false,
            Some(&selection),
            &mut renderer,
        );

        if let DrawInstruction::Text { color, .. } = &renderer.instructions[0] {
            assert_eq!(*color, Color::from_rgb(14, 165, 233));
        } else {
            panic!("expected Text instruction");
        }
    }

    #[test]
    fn draw_accidental_different_tone_id_selection_uses_black() {
        use crate::renderer::test_helpers::make_test_renderer;
        use crate::renderer::DrawInstruction;
        use crate::ui::sequencer::Selection;
        use skia_safe::Color;
        let mut renderer = make_test_renderer();
        let tone = make_sharp_tone(1);
        let an = AccidentalNotation {
            tone_id: 1,
            slot: 1,
        };
        // Selection points at a DIFFERENT tone
        let selection = Selection::Tone {
            flow_id: 1,
            instrument_id: 1,
            stave_id: 1,
            track_id: 1,
            tone_id: 999,
            tuplet_id: None,
        };

        draw_accidental(
            &Point { x: 0.0, y: 0.0 },
            &an,
            &tone,
            1.175,
            false,
            Some(&selection),
            &mut renderer,
        );

        if let DrawInstruction::Text { color, .. } = &renderer.instructions[0] {
            assert_eq!(*color, Color::BLACK);
        } else {
            panic!("expected Text instruction");
        }
    }
}
