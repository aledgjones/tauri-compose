use super::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::engraves::Engrave;
use skia_safe::Color;

pub fn draw_systemic_barline(
    origin: &Point,
    vertical_spacing: &VerticalSpacing,
    engrave: &Engrave,
    renderer: &mut RendererContext,
) {
    let is_multi_stave = vertical_spacing.staves.len() > 1;

    if is_multi_stave || engrave.systemic_barline_single_instrument_system {
        let tweak_for_stave_line_width = STAVE_LINE_WIDTH / 2.0;

        renderer.line(
            STAVE_LINE_WIDTH,
            Color::BLACK,
            vec![
                Point {
                    x: origin.x,
                    y: origin.y - tweak_for_stave_line_width,
                },
                Point {
                    x: origin.x,
                    y: origin.y + vertical_spacing.height + tweak_for_stave_line_width,
                },
            ],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_vertical_spacing::{VerticalPosition, VerticalSpacing};
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::engraves::Engraves;
    use crate::score::layouts::LayoutType;
    use rustc_hash::FxHashMap;

    fn make_engrave() -> crate::score::engraves::Engrave {
        Engraves::new().by_layout_type.remove(&LayoutType::FullScore).unwrap()
    }

    fn make_spacing_multi() -> VerticalSpacing {
        let mut staves = FxHashMap::default();
        staves.insert(1u64, VerticalPosition { y: 0.0, height: 4.0 });
        staves.insert(2u64, VerticalPosition { y: 8.0, height: 4.0 });
        VerticalSpacing { height: 12.0, instruments: FxHashMap::default(), staves }
    }

    fn make_spacing_single() -> VerticalSpacing {
        let mut staves = FxHashMap::default();
        staves.insert(1u64, VerticalPosition { y: 0.0, height: 4.0 });
        VerticalSpacing { height: 4.0, instruments: FxHashMap::default(), staves }
    }

    #[test]
    fn multi_stave_always_draws_barline() {
        let mut renderer = make_test_renderer();
        let spacing = make_spacing_multi();
        let mut engrave = make_engrave();
        engrave.systemic_barline_single_instrument_system = false;

        draw_systemic_barline(&Point { x: 0.0, y: 0.0 }, &spacing, &engrave, &mut renderer);

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(renderer.instructions[0], DrawInstruction::Line { .. }));
    }

    #[test]
    fn single_stave_with_flag_true_draws_barline() {
        let mut renderer = make_test_renderer();
        let spacing = make_spacing_single();
        let mut engrave = make_engrave();
        engrave.systemic_barline_single_instrument_system = true;

        draw_systemic_barline(&Point { x: 0.0, y: 0.0 }, &spacing, &engrave, &mut renderer);

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(renderer.instructions[0], DrawInstruction::Line { .. }));
    }

    #[test]
    fn single_stave_with_flag_false_draws_nothing() {
        let mut renderer = make_test_renderer();
        let spacing = make_spacing_single();
        let mut engrave = make_engrave();
        engrave.systemic_barline_single_instrument_system = false;

        draw_systemic_barline(&Point { x: 0.0, y: 0.0 }, &spacing, &engrave, &mut renderer);

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn barline_spans_full_height_of_vertical_spacing() {
        let mut renderer = make_test_renderer();
        let spacing = make_spacing_multi();
        let engrave = make_engrave();
        let origin = Point { x: 0.0, y: 2.0 };

        draw_systemic_barline(&origin, &spacing, &engrave, &mut renderer);

        if let DrawInstruction::Line { points, .. } = &renderer.instructions[0] {
            let tweak = STAVE_LINE_WIDTH / 2.0;
            assert!((points[0].1 - (origin.y - tweak)).abs() < 1e-5);
            assert!((points[1].1 - (origin.y + spacing.height + tweak)).abs() < 1e-5);
        } else {
            panic!("expected Line instruction");
        }
    }
}
