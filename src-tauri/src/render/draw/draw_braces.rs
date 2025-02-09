use super::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::render::parse::get_vertical_spans::VerticalSpans;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};

pub fn draw_braces(
    origin: &Point,
    vertical_spans: &VerticalSpans,
    vertical_spacing: &VerticalSpacing,
    renderer: &mut Renderer,
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
        );
    }
}
