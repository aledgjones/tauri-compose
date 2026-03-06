use crate::instructions::parse::get_dots::DotOffsets;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use skia_safe::Color;

pub fn draw_dots(
    origin: &Point,
    tick_dots: &DotOffsets,
    glyph_width: f32,
    has_post_shunt: bool,
    has_up_stem_flag: bool,
    renderer: &mut RendererContext,
) {
    let x = if has_post_shunt {
        origin.x + (glyph_width * 2.0) + 0.5
    } else if has_up_stem_flag {
        origin.x + (glyph_width * 2.0)
    } else {
        origin.x + glyph_width + 0.5
    };

    for dot_offset in tick_dots {
        renderer.circle(
            0.2,
            Color::BLACK,
            &Point {
                x,
                y: origin.y + (*dot_offset as f32 / 2.0),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use rustc_hash::FxHashSet;

    #[test]
    fn no_dots_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let dots: DotOffsets = FxHashSet::default();
        draw_dots(
            &Point { x: 0.0, y: 0.0 },
            &dots,
            1.175,
            false,
            false,
            &mut renderer,
        );
        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn one_dot_produces_one_circle_instruction() {
        let mut renderer = make_test_renderer();
        let mut dots: DotOffsets = FxHashSet::default();
        dots.insert(0); // offset 0 = above middle line
        draw_dots(
            &Point { x: 0.0, y: 0.0 },
            &dots,
            1.175,
            false,
            false,
            &mut renderer,
        );
        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Circle { .. }
        ));
    }

    #[test]
    fn two_dots_produce_two_circle_instructions() {
        let mut renderer = make_test_renderer();
        let mut dots: DotOffsets = FxHashSet::default();
        dots.insert(0);
        dots.insert(1);
        draw_dots(
            &Point { x: 0.0, y: 0.0 },
            &dots,
            1.175,
            false,
            false,
            &mut renderer,
        );
        assert_eq!(renderer.instructions.len(), 2);
    }

    #[test]
    fn post_shunt_shifts_x_by_double_width() {
        let mut renderer_normal = make_test_renderer();
        let mut renderer_post = make_test_renderer();
        let mut dots: DotOffsets = FxHashSet::default();
        dots.insert(0);
        let glyph_width = 1.175;
        let origin = Point { x: 5.0, y: 0.0 };

        draw_dots(
            &origin,
            &dots,
            glyph_width,
            false,
            false,
            &mut renderer_normal,
        );
        draw_dots(&origin, &dots, glyph_width, true, false, &mut renderer_post);

        if let (
            DrawInstruction::Circle {
                origin: o_normal, ..
            },
            DrawInstruction::Circle { origin: o_post, .. },
        ) = (
            &renderer_normal.instructions[0],
            &renderer_post.instructions[0],
        ) {
            // post_shunt x = origin.x + glyph_width * 2 + 0.5
            // normal x   = origin.x + glyph_width + 0.5
            assert!(o_post.0 > o_normal.0);
        } else {
            panic!("expected Circle instructions");
        }
    }

    #[test]
    fn up_stem_flag_shifts_x_like_post_shunt_without_extra_0_5() {
        let mut renderer_flag = make_test_renderer();
        let mut renderer_normal = make_test_renderer();
        let mut dots: DotOffsets = FxHashSet::default();
        dots.insert(0);
        let glyph_width = 1.175;
        let origin = Point { x: 5.0, y: 0.0 };

        draw_dots(&origin, &dots, glyph_width, false, true, &mut renderer_flag);
        draw_dots(
            &origin,
            &dots,
            glyph_width,
            false,
            false,
            &mut renderer_normal,
        );

        if let (
            DrawInstruction::Circle { origin: o_flag, .. },
            DrawInstruction::Circle {
                origin: o_normal, ..
            },
        ) = (
            &renderer_flag.instructions[0],
            &renderer_normal.instructions[0],
        ) {
            // flag x = origin.x + glyph_width * 2 (no +0.5)
            // normal x = origin.x + glyph_width + 0.5
            assert!((o_flag.0 - (origin.x + glyph_width * 2.0)).abs() < 1e-5);
            assert!((o_normal.0 - (origin.x + glyph_width + 0.5)).abs() < 1e-5);
        } else {
            panic!("expected Circle instructions");
        }
    }
}
