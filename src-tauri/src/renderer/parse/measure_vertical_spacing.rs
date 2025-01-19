use rustc_hash::FxHashMap;

use crate::score::components::units::Space;
use crate::score::engraves::Engrave;
use crate::utils::UniqueId;
use crate::AppStateInner;

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

impl AppStateInner {
    pub fn measure_vertical_spacing(
        &self,
        flow_id: UniqueId,
        engrave: &Engrave,
    ) -> VerticalSpacing {
        let mut output = VerticalSpacing::new();

        let flow = &self.score.flows.by_id[&flow_id];
        for player_id in &self.score.players.order {
            if flow.player_ids.contains(player_id) {
                let player = &self.score.players.by_id[player_id];
                for (i, instrument_id) in player.instrument_ids.iter().enumerate() {
                    let instrument = &self.score.instruments.by_id[instrument_id];

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
            }
        }
        output
    }
}
