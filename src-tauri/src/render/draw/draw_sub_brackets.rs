use super::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::render::parse::get_vertical_spans::VerticalSpans;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use skia_safe::Color;

pub fn draw_sub_brackets(
    origin: &Point,
    vertical_spans: &VerticalSpans,
    vertical_spacing: &VerticalSpacing,
    renderer: &mut Renderer,
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
