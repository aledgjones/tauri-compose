use crate::renderer::converter::Space;
use crate::renderer::text_measurer::TextMeasurer;
use crate::score::engraves::Engrave;
use crate::score::instruments::Instrument;

pub fn measure_instrument_names(
    instruments: &Vec<&Instrument>,
    engrave: &Engrave,
    text_measurer: &TextMeasurer,
) -> Space {
    let mut max: Space = 0.0;

    for instrument in instruments {
        let text = instrument.name();
        let bounds = text_measurer.measure(&text, &engrave.instrument_name_font);
        if bounds.width > max {
            max = bounds.width;
        }
    }

    max
}
