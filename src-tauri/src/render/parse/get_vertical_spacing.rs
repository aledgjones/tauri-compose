use crate::renderer::converter::Space;
use crate::score::engraves::Engrave;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct VerticalPosition {
    pub y: Space,
    pub height: Space,
}

#[derive(Debug)]
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
