use crate::parse::measure_vertical_spacing::VerticalSpacing;
use crate::renderer::converter::Space;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::text::Font;
use crate::score::engraves::Engrave;
use crate::score::instruments::Instrument;

pub fn draw_names(
    instruments: &Vec<&Instrument>,
    origin: &Point,
    vertical_spacing: &VerticalSpacing,
    engrave: &Engrave,
    renderer: &mut Renderer,
) {
    for instrument in instruments {
        let spacing = &vertical_spacing.instruments[&instrument.id];
        let top: Space = origin.y + spacing.y + (spacing.height / 2.0);

        renderer.text(
            &instrument.name(),
            &Font {
                family: engrave.instrument_name.family,
                align: engrave.instrument_name.align,
                justify: engrave.instrument_name.justify,
                size: engrave.instrument_name.size,
            },
            Point {
                x: origin.x,
                y: top,
            },
        );
    }
}
