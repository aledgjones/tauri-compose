use crate::renderer::converter::Space;
use crate::score::engraves::Engrave;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;

pub struct VerticalPosition {
    pub y: Space,
    pub height: Space,
}

pub struct VerticalSpacing {
    pub height: Space,
    pub instruments: FxHashMap<UniqueId, VerticalPosition>,
    pub staves: FxHashMap<UniqueId, VerticalPosition>,
}

impl VerticalSpacing {
    fn new() -> Self {
        Self {
            height: 0.0,
            instruments: FxHashMap::default(),
            staves: FxHashMap::default(),
        }
    }
}

pub fn get_vertical_spacing(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    engrave: &Engrave,
) -> VerticalSpacing {
    let mut output = VerticalSpacing::new();

    for (i, instrument) in instruments.iter().enumerate() {
        if i > 0 {
            output.height += engrave.instrument_spacing;
        }

        let mut instrument_entry = VerticalPosition {
            y: output.height,
            height: 0.0,
        };

        for (ii, stave_id) in instrument.stave_ids.iter().enumerate() {
            let stave = &flow.staves.by_id[stave_id];

            if ii > 0 {
                output.height += engrave.stave_spacing;
                instrument_entry.height += engrave.stave_spacing;
            }

            let stave_entry = VerticalPosition {
                // items are positioned around the center of the stave (hence / 2);
                y: output.height + ((stave.lines.len() - 1) / 2) as Space,
                height: (stave.lines.len() - 1) as Space,
            };

            output.height += stave_entry.height;
            instrument_entry.height += stave_entry.height;

            output.staves.insert(*stave_id, stave_entry);
        }

        output.instruments.insert(instrument.id, instrument_entry);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_spacing_new_has_zero_height() {
        let vs = VerticalSpacing::new();
        assert!((vs.height - 0.0).abs() < f32::EPSILON);
        assert!(vs.instruments.is_empty());
        assert!(vs.staves.is_empty());
    }

    #[test]
    fn get_vertical_spacing_with_no_instruments_returns_zero_height() {
        use crate::score::engraves::Engraves;
        use crate::score::flows::Flow;
        use crate::score::layouts::LayoutType;

        let flow = Flow::new(1, 100);
        let instruments: Vec<&crate::score::instruments::Instrument> = vec![];
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];

        let vs = get_vertical_spacing(&flow, &instruments, engrave);
        assert!((vs.height - 0.0).abs() < f32::EPSILON);
        assert!(vs.instruments.is_empty());
    }
}
