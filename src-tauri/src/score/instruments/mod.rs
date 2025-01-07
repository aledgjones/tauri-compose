pub mod defs;

use super::{config::AutoCountStyle, Score};
use crate::score::players::PlayerType;
use crate::utils::UniqueId;
use defs::InstrumentType;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub id: UniqueId,
    pub key: String,
    pub instrument_type: InstrumentType,
    pub long_name: String,
    pub short_name: String,
    pub staves: Vec<UniqueId>,
    pub count: Option<String>,
    pub volume: u8, // 0-127
    pub solo: bool,
    pub mute: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Instruments {
    pub by_id: FxHashMap<UniqueId, Instrument>,
}

impl Instruments {
    pub fn new() -> Self {
        Instruments {
            by_id: FxHashMap::default(),
        }
    }
}

impl Score {
    pub fn update_counts(&mut self) {
        let mut instruments_by_type: FxHashMap<PlayerType, FxHashMap<String, Vec<UniqueId>>> =
            FxHashMap::default();

        for player_id in &self.players.order {
            let player = &self.players.by_id[player_id];

            for instrument_id in &player.instruments {
                let instrument = self.instruments.by_id.get_mut(instrument_id).unwrap();

                // Reset the current count
                instrument.count = None;

                // Group instruments by player type and long name
                instruments_by_type
                    .entry(player.player_type)
                    .or_default()
                    .entry(instrument.long_name.clone())
                    .or_default()
                    .push(*instrument_id);
            }
        }

        // Assign counts to instruments for each player type
        for (player_type, instruments) in instruments_by_type {
            let count_style = match player_type {
                PlayerType::Solo => &self.config.auto_count_styles.solo,
                PlayerType::Section => &self.config.auto_count_styles.section,
            };

            for instrument_ids in instruments.values() {
                if instrument_ids.len() > 1 {
                    for (i, instrument_id) in instrument_ids.iter().enumerate() {
                        let instrument = self.instruments.by_id.get_mut(instrument_id).unwrap();
                        let count = (i + 1) as i32;

                        // Format the count based on the style
                        let styled_count = match count_style {
                            AutoCountStyle::Roman => {
                                roman::to(count).expect("Failed to convert to Roman")
                            }
                            AutoCountStyle::Arabic => count.to_string(),
                        };

                        instrument.count = Some(styled_count);
                    }
                }
            }
        }
    }
}
