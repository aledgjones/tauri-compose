use crate::{utils::UniqueId, AppState};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlayerType {
    Solo = 0,
    Section,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: UniqueId,
    pub player_type: PlayerType,
    pub instruments: Vec<String>,
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

impl Default for Players {
    fn default() -> Self {
        Self::new()
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

// pub fn remove_player(&mut self, player_key: &str) {
//     let flows = self.file.score.flows.order.clone();
//     for flow_key in flows {
//         self.unassign_player_from_flow(&flow_key, player_key);
//     }

//     let player = &self.file.score.players.by_key[player_key];
//     let instruments = player.instruments.clone();
//     for instrument_key in instruments {
//         self.unassign_instrument_from_player(player_key, &instrument_key);
//         self.remove_instrument(&instrument_key);
//     }

//     self.file
//         .score
//         .players
//         .order
//         .retain(|item| item != player_key);
//     self.file.score.players.by_key.remove(player_key);

//     self.calculate_counts();
//     self.calculate_colors();
//     self.emit();
// }

// pub fn reorder_players(&mut self, from: usize, to: usize) {
//     let mover = self.file.score.players.order.remove(from);
//     self.file.score.players.order.insert(to, mover);

//     self.calculate_counts();
//     self.calculate_colors();
//     self.emit();
// }
