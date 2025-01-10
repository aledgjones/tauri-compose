use crate::{utils::UniqueId, AppState};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlayerType {
    Solo = 0,
    Section,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: UniqueId,
    pub player_type: PlayerType,
    pub instruments: Vec<UniqueId>,
}

impl Player {
    pub fn new(id: UniqueId, player_type: PlayerType) -> Self {
        Player {
            id,
            player_type,
            instruments: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Players {
    pub order: Vec<UniqueId>,
    pub by_id: FxHashMap<UniqueId, Player>,
}

impl Players {
    pub fn new() -> Self {
        Players {
            order: Vec::new(),
            by_id: FxHashMap::default(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_player(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    player_type: PlayerType,
) -> UniqueId {
    let mut state = state.lock().unwrap();

    let id = state.generate_id();

    let player = Player::new(id, player_type);

    state.score.players.order.push(id);
    state.score.players.by_id.insert(id, player);

    // let flows = self.file.score.flows.order.clone();
    // for flow_key in flows {
    //     self.assign_player_to_flow(&flow_key, &key.clone());
    // }

    state.emit(&app);

    id
}

#[tauri::command(rename_all = "snake_case")]
pub fn reorder_players(app: tauri::AppHandle, state: State<'_, AppState>, from: usize, to: usize) {
    let mut state = state.lock().unwrap();

    let moved = state.score.players.order.remove(from);
    state.score.players.order.insert(to, moved);

    state.score.update_counts();
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn assign_instrument_to_player(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    player_id: UniqueId,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    let player = state.score.players.by_id.get_mut(&player_id).unwrap();
    player.instruments.push(instrument_id);

    // let flows = self.file.score.flows.order.clone();
    // for flow_key in flows {
    //     let flow = &self.file.score.flows.by_key[&flow_key];
    //     if flow.players.contains(player_key) {
    //         self.assign_instrument_to_flow(&flow_key, instrument_key);
    //     }
    // }

    state.score.update_counts();
    state.emit(&app);
}
