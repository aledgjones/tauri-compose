use super::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::instructions::parse::get_vertical_spans::VerticalSpans;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use skia_safe::Color;

pub fn draw_sub_brackets(
    origin: &Point,
    vertical_spans: &VerticalSpans,
    vertical_spacing: &VerticalSpacing,
    renderer: &mut RendererContext,
) {
    for bracket in &vertical_spans.sub_brackets {
        let top = &vertical_spacing.instruments[&bracket.start];
        let bottom = &vertical_spacing.instruments[&bracket.stop];

        let top = origin.y + top.y;
        let bottom = origin.y + bottom.y + bottom.height;

        renderer.line(
            STAVE_LINE_WIDTH,
            Color::BLACK,
            vec![
                Point {
                    x: origin.x,
                    y: top,
                },
                Point {
                    x: origin.x - 1.5,
                    y: top,
                },
                Point {
                    x: origin.x - 1.5,
                    y: bottom,
                },
                Point {
                    x: origin.x,
                    y: bottom,
                },
            ],
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_vertical_spacing::{VerticalPosition, VerticalSpacing};
    use crate::instructions::parse::get_vertical_spans::{VerticalSpan, VerticalSpans};
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::utils::UniqueId;
    use rustc_hash::FxHashMap;

    fn make_spans_with_sub_bracket(start: UniqueId, stop: UniqueId) -> VerticalSpans {
        VerticalSpans {
            brackets: vec![],
            sub_brackets: vec![VerticalSpan { start, stop }],
            braces: vec![],
            barlines: vec![],
        }
    }

    fn make_spacing(start: UniqueId, stop: UniqueId) -> VerticalSpacing {
        let mut instruments = FxHashMap::default();
        instruments.insert(
            start,
            VerticalPosition {
                y: 0.0,
                height: 4.0,
            },
        );
        instruments.insert(
            stop,
            VerticalPosition {
                y: 8.0,
                height: 4.0,
            },
        );
        VerticalSpacing {
            height: 12.0,
            staves: FxHashMap::default(),
            instruments,
        }
    }

    #[test]
    fn one_sub_bracket_produces_one_line_instruction() {
        let mut renderer = make_test_renderer();
        let spans = make_spans_with_sub_bracket(1, 2);
        let spacing = make_spacing(1, 2);

        draw_sub_brackets(&Point { x: 5.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Line { .. }
        ));
    }

    #[test]
    fn no_sub_brackets_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let spans = VerticalSpans {
            brackets: vec![],
            sub_brackets: vec![],
            braces: vec![],
            barlines: vec![],
        };
        let spacing = make_spacing(1, 2);

        draw_sub_brackets(&Point { x: 0.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn sub_bracket_line_has_four_points() {
        let mut renderer = make_test_renderer();
        let spans = make_spans_with_sub_bracket(1, 2);
        let spacing = make_spacing(1, 2);

        draw_sub_brackets(&Point { x: 0.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        if let DrawInstruction::Line { points, .. } = &renderer.instructions[0] {
            assert_eq!(points.len(), 4);
        } else {
            panic!("expected Line instruction");
        }
    }

    #[test]
    fn two_sub_brackets_produce_two_line_instructions() {
        let mut renderer = make_test_renderer();
        let spans = VerticalSpans {
            brackets: vec![],
            sub_brackets: vec![
                VerticalSpan { start: 1, stop: 2 },
                VerticalSpan { start: 3, stop: 4 },
            ],
            braces: vec![],
            barlines: vec![],
        };
        let mut instruments = FxHashMap::default();
        instruments.insert(
            1u64,
            VerticalPosition {
                y: 0.0,
                height: 4.0,
            },
        );
        instruments.insert(
            2u64,
            VerticalPosition {
                y: 4.0,
                height: 4.0,
            },
        );
        instruments.insert(
            3u64,
            VerticalPosition {
                y: 10.0,
                height: 4.0,
            },
        );
        instruments.insert(
            4u64,
            VerticalPosition {
                y: 14.0,
                height: 4.0,
            },
        );
        let spacing = VerticalSpacing {
            height: 18.0,
            staves: FxHashMap::default(),
            instruments,
        };

        draw_sub_brackets(&Point { x: 0.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        assert_eq!(renderer.instructions.len(), 2);
    }
}
