use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
use skia_safe::Color;

pub fn draw_time_signature(
    origin: &Point,
    time_signature: &TimeSignature,
    renderer: &mut RendererContext,
) {
    let metrics = time_signature.metrics();
    let offset = metrics.width * 0.5;

    match time_signature.draw_type {
        TimeSignatureDrawType::Hidden => (),
        TimeSignatureDrawType::CommonTime => renderer.text(
            "\u{E08A}",
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point {
                y: origin.y,
                x: origin.x + offset,
            },
            Color::BLACK,
        ),
        TimeSignatureDrawType::SplitCommonTime => renderer.text(
            "\u{E08B}",
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point {
                y: origin.y,
                x: origin.x + offset,
            },
            Color::BLACK,
        ),
        TimeSignatureDrawType::Open => renderer.text(
            "\u{E09C}",
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point {
                y: origin.y,
                x: origin.x + offset,
            },
            Color::BLACK,
        ),
        TimeSignatureDrawType::Regular => {
            renderer.text(
                &time_signature.beats_to_glyph(),
                &Font {
                    size: 4.0,
                    family: FontFamily::Bravura,
                    align: Align::Bottom,
                    justify: Justify::Middle,
                },
                &Point {
                    y: origin.y - 1.0,
                    x: origin.x + offset,
                },
                Color::BLACK,
            );
            renderer.text(
                &time_signature.beat_type_to_glyph(),
                &Font {
                    size: 4.0,
                    family: FontFamily::Bravura,
                    align: Align::Bottom,
                    justify: Justify::Middle,
                },
                &Point {
                    y: origin.y + 1.0,
                    x: origin.x + offset,
                },
                Color::BLACK,
            );
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::components::duration::NoteDuration;

    #[test]
    fn hidden_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let ts = TimeSignature::new(0, 0, 4, NoteDuration::Quarter, TimeSignatureDrawType::Hidden, None);
        draw_time_signature(&Point { x: 0.0, y: 0.0 }, &ts, &mut renderer);
        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn common_time_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let ts = TimeSignature::new(0, 0, 4, NoteDuration::Quarter, TimeSignatureDrawType::CommonTime, None);
        draw_time_signature(&Point { x: 0.0, y: 0.0 }, &ts, &mut renderer);
        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(renderer.instructions[0], DrawInstruction::Text { .. }));
    }

    #[test]
    fn split_common_time_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let ts = TimeSignature::new(0, 0, 2, NoteDuration::Half, TimeSignatureDrawType::SplitCommonTime, None);
        draw_time_signature(&Point { x: 0.0, y: 0.0 }, &ts, &mut renderer);
        assert_eq!(renderer.instructions.len(), 1);
    }

    #[test]
    fn open_time_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let ts = TimeSignature::new(0, 0, 4, NoteDuration::Quarter, TimeSignatureDrawType::Open, None);
        draw_time_signature(&Point { x: 0.0, y: 0.0 }, &ts, &mut renderer);
        assert_eq!(renderer.instructions.len(), 1);
    }

    #[test]
    fn regular_four_four_produces_two_text_instructions() {
        let mut renderer = make_test_renderer();
        let ts = TimeSignature::new(0, 0, 4, NoteDuration::Quarter, TimeSignatureDrawType::Regular, None);
        draw_time_signature(&Point { x: 0.0, y: 0.0 }, &ts, &mut renderer);
        assert_eq!(renderer.instructions.len(), 2);
        for instr in &renderer.instructions {
            assert!(matches!(instr, DrawInstruction::Text { .. }));
        }
    }

    #[test]
    fn regular_beats_and_beat_type_are_at_different_y_positions() {
        let mut renderer = make_test_renderer();
        let ts = TimeSignature::new(0, 0, 6, NoteDuration::Eighth, TimeSignatureDrawType::Regular, None);
        draw_time_signature(&Point { x: 0.0, y: 2.0 }, &ts, &mut renderer);
        if let (DrawInstruction::Text { point: top_pt, .. }, DrawInstruction::Text { point: bot_pt, .. }) =
            (&renderer.instructions[0], &renderer.instructions[1])
        {
            // beats at origin.y - 1, beat_type at origin.y + 1
            assert!(top_pt.1 < bot_pt.1);
        } else {
            panic!("expected two Text instructions");
        }
    }
}
