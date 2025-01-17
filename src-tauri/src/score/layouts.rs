use crate::{utils::UniqueId, AppState, AppStateInner};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LayoutType {
    FullScore,
    Part,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Layouts {
    pub order: Vec<UniqueId>,
    pub by_id: FxHashMap<UniqueId, Layout>,
}

impl Layouts {
    pub fn new() -> Layouts {
        let layout = Layout::new(0, LayoutType::FullScore);

        let mut layouts = Layouts {
            order: Vec::new(),
            by_id: FxHashMap::default(),
        };

        layouts.order.push(0);
        layouts.by_id.insert(0, layout);

        layouts
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Layout {
    pub id: UniqueId,
    pub layout_type: LayoutType,
    pub title: String,
    pub player_ids: FxHashSet<UniqueId>, // purely for inclusion lookup -- order comes from score.players.order
    pub flow_ids: FxHashSet<UniqueId>, // purely for inclusion lookup -- order comes from score.flows.order
}

impl Layout {
    pub fn new(id: UniqueId, layout_type: LayoutType) -> Layout {
        Layout {
            id,
            layout_type,
            title: String::from(""),
            player_ids: FxHashSet::default(),
            flow_ids: FxHashSet::default(),
        }
    }
}

impl AppStateInner {
    pub fn assign_player_to_layout(&mut self, layout_id: UniqueId, player_id: UniqueId) {
        let layout = self.score.layouts.by_id.get_mut(&layout_id).unwrap();
        layout.player_ids.insert(player_id);
    }

    pub fn unassign_player_from_layout(&mut self, layout_id: UniqueId, player_id: UniqueId) {
        let layout = self.score.layouts.by_id.get_mut(&layout_id).unwrap();
        layout.player_ids.remove(&player_id);
    }

    pub fn assign_flow_to_layout(&mut self, layout_id: UniqueId, flow_id: UniqueId) {
        let layout = self.score.layouts.by_id.get_mut(&layout_id).unwrap();
        layout.flow_ids.insert(flow_id);
    }

    pub fn unassign_flow_from_layout(&mut self, layout_id: UniqueId, flow_id: UniqueId) {
        let layout = self.score.layouts.by_id.get_mut(&layout_id).unwrap();
        layout.flow_ids.remove(&flow_id);
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_type: LayoutType,
) -> UniqueId {
    let mut state = state.lock().unwrap();

    let layout_id = state.generate_id();
    let layout = Layout::new(layout_id, layout_type);

    state.score.layouts.order.push(layout_id);
    state.score.layouts.by_id.insert(layout_id, layout);

    for flow_id in state.score.flows.order.clone() {
        state.assign_flow_to_layout(layout_id, flow_id);
    }

    if layout_type == LayoutType::FullScore {
        for player_id in state.score.players.order.clone() {
            state.assign_player_to_layout(layout_id, player_id);
        }
    }

    state.emit(&app);

    layout_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_assign_player(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.assign_player_to_layout(layout_id, player_id);

    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_unassign_player(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.unassign_player_from_layout(layout_id, player_id);

    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_assign_flow(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.assign_flow_to_layout(layout_id, player_id);

    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_unassign_flow(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.unassign_flow_from_layout(layout_id, player_id);

    state.emit(&app);
}
