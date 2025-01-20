use crate::parse::measure_vertical_spacing::VerticalSpacing;
use crate::renderer::converter::Space;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;

pub const STAVE_LINE_WIDTH: Space = 0.125;

pub fn draw_staves(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    origin: &Point,
    width: Space,
    vertical_spacing: &VerticalSpacing,
    renderer: &mut Renderer,
) {
    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_spacing = &vertical_spacing.staves[stave_id];

            for (i, line) in stave.lines.iter().enumerate() {
                if line == &1 {
                    let top =
                        origin.y + stave_spacing.y - (stave_spacing.height / 2.0) + i as Space;

                    renderer.line(
                        STAVE_LINE_WIDTH,
                        vec![
                            Point {
                                x: origin.x,
                                y: top,
                            },
                            Point {
                                x: origin.x + width,
                                y: top,
                            },
                        ],
                    );
                }
            }
        }
    }
}
