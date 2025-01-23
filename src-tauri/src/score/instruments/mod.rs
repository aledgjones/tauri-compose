pub mod defs;

use super::{config::AutoCountStyle, Score};
use crate::utils::UniqueId;
use crate::{score::players::PlayerType, AppState};
use defs::{get_instrument_definition, InstrumentType};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub id: UniqueId,
    pub definition_id: String,
    pub instrument_type: InstrumentType,
    pub long_name: String,
    pub short_name: String,
    pub stave_ids: Vec<UniqueId>,
    pub count: Option<String>,
    pub volume: u8, // 0-127
    pub solo: bool,
    pub mute: bool,
}

impl Instrument {
    pub fn family(&self) -> &str {
        let mut parts = self.definition_id.split('.');
        match parts.next() {
            None => "unknown",
            Some(part) => part,
        }
    }

    pub fn name(&self) -> String {
        match &self.count {
            Some(count) => format!("{} {}", self.long_name, count),
            None => self.long_name.clone(),
        }
    }
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

            for instrument_id in &player.instrument_ids {
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

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    definition_id: String,
) -> UniqueId {
    let mut state = state.lock().unwrap();

    let instrument_id = state.generate_id();

    let def = get_instrument_definition(&definition_id).unwrap();
    let stave_ids = def
        .staves
        .iter()
        .map(|_| state.generate_id())
        .collect::<Vec<UniqueId>>();

    let instrument = Instrument {
        id: instrument_id,
        definition_id: definition_id.clone(),
        instrument_type: def.instrument_type,
        long_name: String::from(def.long_name),
        short_name: String::from(def.short_name),
        stave_ids,
        count: None,
        volume: 80,
        solo: false,
        mute: false,
    };

    state
        .score
        .instruments
        .by_id
        .insert(instrument_id, instrument);

    state.emit(&app);

    instrument_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_move(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    from_index: usize,
    to_index: usize,
    from_player_id: UniqueId,
    to_player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    let from_player = state.score.players.by_id.get_mut(&from_player_id).unwrap();
    let instrument_id = from_player.instrument_ids.remove(from_index);

    if from_player_id == to_player_id {
        from_player.instrument_ids.insert(to_index, instrument_id);
    } else {
        for flow_id in state.score.flows.order.clone() {
            let flow = &state.score.flows.by_id[&flow_id];
            let is_from_player_in_flow = flow.player_ids.contains(&from_player_id);
            let is_to_player_in_flow = flow.player_ids.contains(&to_player_id);

            // the players are either both in the flow or both not in the flow.
            // either way we don't have to do anything.
            if is_from_player_in_flow == is_to_player_in_flow {
                continue;
            }

            if is_from_player_in_flow {
                // moving instrument from player in flow -> player not in flow
                state.assign_player_to_flow(flow_id, to_player_id);
            } else {
                // moving instrument from player not in flow -> player in flow
                state.assign_instrument_to_flow(flow_id, instrument_id);
            }
        }

        // only assign to the new player after the flows loop to cover the situation where:
        // the instrument is already in the flow but the other instruments the to player has aren't
        // we can simply use the assign_player_to_flow helper without the new instrument (as it's already assigned)
        let to_player = state.score.players.by_id.get_mut(&to_player_id).unwrap();
        to_player.instrument_ids.insert(to_index, instrument_id);
    }

    state.score.update_counts();
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_remove(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.score.instruments.by_id.remove(&instrument_id);

    state.emit(&app);
}
