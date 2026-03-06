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

    #[test]
    fn get_vertical_spacing_with_one_instrument_has_positive_height() {
        use crate::score::engraves::Engraves;
        use crate::score::flows::Flow;
        use crate::score::instruments::{defs::InstrumentType, Instrument};
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;

        let stave_id = 10u64;
        let instrument = Instrument {
            id: 50,
            definition_id: String::from("test"),
            instrument_type: InstrumentType::Melodic,
            long_name: String::from("Piano"),
            short_name: String::from("Pno."),
            stave_ids: vec![stave_id],
            count: None,
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        };

        let mut flow = Flow::new(1, 100);
        // Insert a 5-line stave into the flow
        flow.staves.by_id.insert(
            stave_id,
            Stave {
                id: stave_id,
                lines: vec![1, 1, 1, 1, 1],
                master_id: 100,
                track_ids: Vec::new(),
            },
        );

        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];

        let vs = get_vertical_spacing(&flow, &vec![&instrument], engrave);
        assert!(vs.height > 0.0);
        assert!(vs.instruments.contains_key(&50));
        assert!(vs.staves.contains_key(&stave_id));
    }

    #[test]
    fn get_vertical_spacing_two_instruments_adds_spacing() {
        use crate::score::engraves::Engraves;
        use crate::score::flows::Flow;
        use crate::score::instruments::{defs::InstrumentType, Instrument};
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;

        let stave_id_1 = 10u64;
        let stave_id_2 = 11u64;

        let mk_instrument = |id: u64, stave_id: u64| Instrument {
            id,
            definition_id: String::from("test"),
            instrument_type: InstrumentType::Melodic,
            long_name: String::new(),
            short_name: String::new(),
            stave_ids: vec![stave_id],
            count: None,
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        };
        let inst1 = mk_instrument(50, stave_id_1);
        let inst2 = mk_instrument(51, stave_id_2);

        let mut flow = Flow::new(1, 100);
        for sid in [stave_id_1, stave_id_2] {
            flow.staves.by_id.insert(
                sid,
                Stave {
                    id: sid,
                    lines: vec![1, 1, 1, 1, 1],
                    master_id: 100,
                    track_ids: Vec::new(),
                },
            );
        }

        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];

        let vs_one = get_vertical_spacing(&flow, &vec![&inst1], engrave);
        let vs_two = get_vertical_spacing(&flow, &vec![&inst1, &inst2], engrave);
        // Two instruments should have more height than one
        assert!(vs_two.height > vs_one.height);
    }
}
