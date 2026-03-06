use super::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_stem_lengths::StemDef;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use skia_safe::Color;

pub fn draw_stem(origin: &Point, def: &StemDef, renderer: &mut RendererContext) {
    renderer.line(
        STAVE_LINE_WIDTH,
        Color::BLACK,
        vec![
            Point {
                x: origin.x + def.head.x,
                y: origin.y + def.head.y,
            },
            Point {
                x: origin.x + def.tail.x,
                y: origin.y + def.tail.y,
            },
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;

    fn make_stem_def() -> StemDef {
        StemDef {
            head: Point { x: 0.5, y: 0.0 },
            tail: Point { x: 0.5, y: -3.5 },
        }
    }

    #[test]
    fn draw_stem_produces_one_line_instruction() {
        let mut renderer = make_test_renderer();
        let origin = Point { x: 10.0, y: 5.0 };
        let def = make_stem_def();

        draw_stem(&origin, &def, &mut renderer);

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(renderer.instructions[0], DrawInstruction::Line { .. }));
    }

    #[test]
    fn draw_stem_line_connects_head_to_tail_relative_to_origin() {
        let mut renderer = make_test_renderer();
        let origin = Point { x: 10.0, y: 5.0 };
        let def = make_stem_def();

        draw_stem(&origin, &def, &mut renderer);

        if let DrawInstruction::Line { points, .. } = &renderer.instructions[0] {
            // With Converter(1.0, 1.0): spaces == px
            let (hx, hy) = points[0];
            let (tx, ty) = points[1];
            assert!((hx - (origin.x + def.head.x)).abs() < 1e-5);
            assert!((hy - (origin.y + def.head.y)).abs() < 1e-5);
            assert!((tx - (origin.x + def.tail.x)).abs() < 1e-5);
            assert!((ty - (origin.y + def.tail.y)).abs() < 1e-5);
        } else {
            panic!("expected DrawInstruction::Line");
        }
    }

    #[test]
    fn draw_stem_uses_black_color() {
        let mut renderer = make_test_renderer();
        draw_stem(&Point { x: 0.0, y: 0.0 }, &make_stem_def(), &mut renderer);
        if let DrawInstruction::Line { color, .. } = &renderer.instructions[0] {
            assert_eq!(*color, skia_safe::Color::BLACK);
        } else {
            panic!("expected DrawInstruction::Line");
        }
    }
}
