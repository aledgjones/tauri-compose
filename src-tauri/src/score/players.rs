use super::layouts::LayoutType;
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
    pub instrument_ids: Vec<UniqueId>,
}

impl Player {
    pub fn new(id: UniqueId, player_type: PlayerType) -> Self {
        Player {
            id,
            player_type,
            instrument_ids: Vec::new(),
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
pub fn score_players_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    player_type: PlayerType,
) -> UniqueId {
    let mut state = state.lock().unwrap();

    let player_id = state.generate_id();
    let player = Player::new(player_id, player_type);

    state.score.players.order.push(player_id);
    state.score.players.by_id.insert(player_id, player);

    for flow_id in state.score.flows.order.clone() {
        state.assign_player_to_flow(flow_id, player_id);
    }

    for layout_id in state.score.layouts.order.clone() {
        let layout = &state.score.layouts.by_id[&layout_id];
        if layout.layout_type == LayoutType::FullScore {
            state.assign_player_to_layout(layout_id, player_id);
        }
    }

    state.emit(&app);

    player_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_move(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    from_index: usize,
    to_index: usize,
) {
    let mut state = state.lock().unwrap();

    let moved = state.score.players.order.remove(from_index);
    state.score.players.order.insert(to_index, moved);

    state.score.update_counts();
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_assign_instrument(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    player_id: UniqueId,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    let player = state.score.players.by_id.get_mut(&player_id).unwrap();
    player.instrument_ids.push(instrument_id);

    for flow_id in state.score.flows.order.clone() {
        let flow = &state.score.flows.by_id[&flow_id];
        if flow.player_ids.contains(&player_id) {
            state.assign_instrument_to_flow(flow_id, instrument_id);
        }
    }

    state.score.update_counts();
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_unassign_instrument(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    player_id: UniqueId,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    let player = state.score.players.by_id.get_mut(&player_id).unwrap();
    player.instrument_ids.retain(|id| *id != instrument_id);

    for flow_id in state.score.flows.order.clone() {
        let flow = &state.score.flows.by_id[&flow_id];
        if flow.player_ids.contains(&player_id) {
            state.unassign_instrument_from_flow(flow_id, instrument_id);
        }
    }

    state.score.update_counts();
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_remove(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    // remove player & instruments from flows
    for flow_id in state.score.flows.order.clone() {
        state.unassign_player_from_flow(flow_id, player_id);
    }

    // cleanup player instruments
    let player = state.score.players.by_id.get(&player_id).unwrap();
    for instrument_id in player.instrument_ids.clone() {
        state.score.instruments.by_id.remove(&instrument_id);
    }

    // cleanup player
    state.score.players.order.retain(|id| *id != player_id);
    state.score.players.by_id.remove(&player_id);

    // cleanup ui state
    state.ui.setup.expanded.remove(&player_id);
    state.ui.setup.selection = None;

    state.score.update_counts();
    state.emit(&app);
}
