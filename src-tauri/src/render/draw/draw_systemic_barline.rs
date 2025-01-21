use super::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::engraves::Engrave;

pub fn draw_systemic_barline(
    origin: &Point,
    vertical_spacing: &VerticalSpacing,
    engrave: &Engrave,
    renderer: &mut Renderer,
) {
    let is_multi_stave = vertical_spacing.staves.len() > 1;

    if is_multi_stave || engrave.systemic_barline_single_instrument_system {
        let tweak_for_stave_line_width = STAVE_LINE_WIDTH / 2.0;

        renderer.line(
            STAVE_LINE_WIDTH,
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
