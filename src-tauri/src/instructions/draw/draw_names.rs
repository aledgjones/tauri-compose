use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::renderer::converter::Space;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::Font;
use crate::score::engraves::Engrave;
use crate::score::instruments::Instrument;
use skia_safe::Color;

pub fn draw_names(
    origin: &Point,
    instruments: &Vec<&Instrument>,
    vertical_spacing: &VerticalSpacing,
    engrave: &Engrave,
    renderer: &mut RendererContext,
) {
    for instrument in instruments {
        let spacing = &vertical_spacing.instruments[&instrument.id];
        let top: Space = origin.y + spacing.y + (spacing.height / 2.0);

        renderer.text(
            &instrument.name(),
            &Font {
                family: engrave.instrument_name_font.family,
                align: engrave.instrument_name_font.align,
                justify: engrave.instrument_name_font.justify,
                size: engrave.instrument_name_font.size,
            },
            &Point {
                x: origin.x,
                y: top,
            },
            Color::BLACK,
        );
    }
}
