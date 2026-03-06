use super::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::instructions::parse::get_vertical_spans::VerticalSpans;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use skia_safe::Color;

pub fn draw_braces(
    origin: &Point,
    vertical_spans: &VerticalSpans,
    vertical_spacing: &VerticalSpacing,
    renderer: &mut RendererContext,
) {
    for brace in &vertical_spans.braces {
        let top = &vertical_spacing.staves[&brace.start];
        let bottom = &vertical_spacing.staves[&brace.stop];
        let height = (bottom.y + bottom.height) - top.y;

        renderer.text(
            "\u{E000}",
            &Font {
                size: height,
                family: FontFamily::Bravura,
                justify: Justify::End,
                align: Align::Middle,
            },
            &Point {
                x: origin.x - 0.25,                        // 1/4 space back from stave start
                y: origin.y + bottom.y + STAVE_LINE_WIDTH, // right the the edges of the stave
            },
            Color::BLACK,
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

    fn make_spans_with_brace(start: UniqueId, stop: UniqueId) -> VerticalSpans {
        VerticalSpans {
            brackets: vec![],
            sub_brackets: vec![],
            braces: vec![VerticalSpan { start, stop }],
            barlines: vec![],
        }
    }

    fn make_spacing(start: UniqueId, stop: UniqueId) -> VerticalSpacing {
        let mut staves = FxHashMap::default();
        staves.insert(start, VerticalPosition { y: 0.0, height: 4.0 });
        staves.insert(stop, VerticalPosition { y: 6.0, height: 4.0 });
        VerticalSpacing { height: 10.0, instruments: FxHashMap::default(), staves }
    }

    #[test]
    fn draw_braces_with_one_brace_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let spans = make_spans_with_brace(1, 2);
        let spacing = make_spacing(1, 2);

        draw_braces(&Point { x: 5.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(renderer.instructions[0], DrawInstruction::Text { .. }));
    }

    #[test]
    fn draw_braces_with_no_braces_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let spans = VerticalSpans { brackets: vec![], sub_brackets: vec![], braces: vec![], barlines: vec![] };
        let spacing = make_spacing(1, 2);

        draw_braces(&Point { x: 0.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn draw_braces_with_two_braces_produces_two_text_instructions() {
        let mut renderer = make_test_renderer();
        let spans = VerticalSpans {
            brackets: vec![],
            sub_brackets: vec![],
            braces: vec![
                VerticalSpan { start: 1, stop: 2 },
                VerticalSpan { start: 3, stop: 4 },
            ],
            barlines: vec![],
        };
        let mut staves = FxHashMap::default();
        staves.insert(1u64, VerticalPosition { y: 0.0, height: 4.0 });
        staves.insert(2u64, VerticalPosition { y: 4.0, height: 4.0 });
        staves.insert(3u64, VerticalPosition { y: 10.0, height: 4.0 });
        staves.insert(4u64, VerticalPosition { y: 14.0, height: 4.0 });
        let spacing = VerticalSpacing { height: 18.0, instruments: FxHashMap::default(), staves };

        draw_braces(&Point { x: 0.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        assert_eq!(renderer.instructions.len(), 2);
    }

    #[test]
    fn draw_braces_uses_bravura_font() {
        let mut renderer = make_test_renderer();
        let spans = make_spans_with_brace(1, 2);
        let spacing = make_spacing(1, 2);

        draw_braces(&Point { x: 0.0, y: 0.0 }, &spans, &spacing, &mut renderer);

        if let DrawInstruction::Text { font_family, .. } = &renderer.instructions[0] {
            use crate::score::components::text::FontFamily;
            assert_eq!(*font_family, FontFamily::Bravura);
        } else {
            panic!("expected Text instruction");
        }
    }
}
