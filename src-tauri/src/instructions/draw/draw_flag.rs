use super::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_stem_lengths::StemDef;
use crate::instructions::parse::get_written_durations::Notation;
use crate::renderer::RendererContext;
use crate::score::components::duration::Direction;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use skia_safe::Color;

pub fn draw_flag(
    position: &Point,
    notation: &Notation,
    stem_direction: &Direction,
    stem_length: &StemDef,
    renderer: &mut RendererContext,
) {
    let glyph = notation.flag_glyph(stem_direction);
    let left = position.x + stem_length.tail.x - (STAVE_LINE_WIDTH / 2.0);
    let top = position.y + stem_length.tail.y;

    renderer.text(
        &glyph,
        &Font {
            family: FontFamily::Bravura,
            size: 4.0,
            justify: Justify::Start,
            align: Align::Bottom,
        },
        &Point { x: left, y: top },
        Color::BLACK,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_stem_lengths::StemDef;
    use crate::instructions::parse::get_written_durations::Notation;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::components::duration::Direction;

    fn make_eighth_notation() -> Notation {
        // 16 ticks = eighth note
        Notation::new(0, 16)
    }

    fn make_stem() -> StemDef {
        StemDef {
            head: Point { x: 0.5, y: 0.0 },
            tail: Point { x: 0.5, y: -3.5 },
        }
    }

    #[test]
    fn draw_flag_up_stem_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let notation = make_eighth_notation();
        let stem = make_stem();
        let origin = Point { x: 10.0, y: 5.0 };

        draw_flag(&origin, &notation, &Direction::Up, &stem, &mut renderer);

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
    }

    #[test]
    fn draw_flag_down_stem_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let notation = make_eighth_notation();
        let stem = make_stem();

        draw_flag(
            &Point { x: 0.0, y: 0.0 },
            &notation,
            &Direction::Down,
            &stem,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
    }

    #[test]
    fn draw_flag_up_and_down_use_different_glyphs() {
        let notation = make_eighth_notation();
        let stem = make_stem();
        let origin = Point { x: 0.0, y: 0.0 };

        let mut r_up = make_test_renderer();
        draw_flag(&origin, &notation, &Direction::Up, &stem, &mut r_up);

        let mut r_down = make_test_renderer();
        draw_flag(&origin, &notation, &Direction::Down, &stem, &mut r_down);

        if let (
            DrawInstruction::Text { text: up_text, .. },
            DrawInstruction::Text {
                text: down_text, ..
            },
        ) = (&r_up.instructions[0], &r_down.instructions[0])
        {
            assert_ne!(up_text, down_text);
        } else {
            panic!("expected Text instructions");
        }
    }

    #[test]
    fn draw_flag_x_position_is_tail_x_plus_origin_x() {
        let mut renderer = make_test_renderer();
        let notation = make_eighth_notation();
        let stem = make_stem();
        let origin = Point { x: 10.0, y: 5.0 };

        draw_flag(&origin, &notation, &Direction::Up, &stem, &mut renderer);

        if let DrawInstruction::Text { point, .. } = &renderer.instructions[0] {
            // left = origin.x + stem.tail.x - (STAVE_LINE_WIDTH / 2.0)
            let expected_x = origin.x + stem.tail.x - (STAVE_LINE_WIDTH / 2.0);
            assert!((point.0 - expected_x).abs() < 1e-5);
        } else {
            panic!("expected Text instruction");
        }
    }
}
