use crate::{
    instructions::FlowRenderRecalculationRequest, utils::UniqueId, AppContext, AppContextState,
    RendererContextState,
};
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

impl AppContext {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_new_has_one_entry() {
        let layouts = Layouts::new();
        assert_eq!(layouts.order.len(), 1);
        assert_eq!(layouts.by_id.len(), 1);
    }

    #[test]
    fn layouts_new_default_is_full_score() {
        let layouts = Layouts::new();
        let layout = &layouts.by_id[&layouts.order[0]];
        assert!(layout.layout_type == LayoutType::FullScore);
    }

    #[test]
    fn layout_new_stores_id() {
        let layout = Layout::new(42, LayoutType::FullScore);
        assert_eq!(layout.id, 42);
    }

    #[test]
    fn layout_new_empty_title() {
        let layout = Layout::new(1, LayoutType::Part);
        assert!(layout.title.is_empty());
    }

    #[test]
    fn layout_new_empty_player_ids() {
        let layout = Layout::new(1, LayoutType::FullScore);
        assert!(layout.player_ids.is_empty());
    }

    #[test]
    fn layout_new_empty_flow_ids() {
        let layout = Layout::new(1, LayoutType::FullScore);
        assert!(layout.flow_ids.is_empty());
    }

    #[test]
    fn layout_type_full_score_and_part_are_different() {
        assert!(LayoutType::FullScore != LayoutType::Part);
    }

    #[test]
    fn layout_player_ids_can_insert() {
        let mut layout = Layout::new(1, LayoutType::FullScore);
        layout.player_ids.insert(99);
        assert!(layout.player_ids.contains(&99));
    }

    #[test]
    fn layout_player_ids_can_remove() {
        let mut layout = Layout::new(1, LayoutType::FullScore);
        layout.player_ids.insert(99);
        layout.player_ids.remove(&99);
        assert!(layout.player_ids.is_empty());
    }

    #[test]
    fn layout_flow_ids_can_insert() {
        let mut layout = Layout::new(1, LayoutType::FullScore);
        layout.flow_ids.insert(42);
        assert!(layout.flow_ids.contains(&42));
    }

    #[test]
    fn layout_flow_ids_can_remove() {
        let mut layout = Layout::new(1, LayoutType::FullScore);
        layout.flow_ids.insert(42);
        layout.flow_ids.remove(&42);
        assert!(layout.flow_ids.is_empty());
    }

    #[test]
    fn layout_title_can_be_set() {
        let mut layout = Layout::new(1, LayoutType::Part);
        layout.title = String::from("Violin I");
        assert_eq!(layout.title, "Violin I");
    }

    #[test]
    fn layouts_new_order_is_single_zero() {
        let layouts = Layouts::new();
        assert_eq!(layouts.order, vec![0]);
    }

    #[test]
    fn layout_type_part_stays_part() {
        let layout = Layout::new(5, LayoutType::Part);
        assert!(layout.layout_type == LayoutType::Part);
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

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

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );

    layout_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_rename(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_id: UniqueId,
    title: String,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let layout = state.score.layouts.by_id.get_mut(&layout_id).unwrap();
    layout.title = title;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_assign_player(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.assign_player_to_layout(layout_id, player_id);

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_unassign_player(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.unassign_player_from_layout(layout_id, player_id);

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_assign_flow(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.assign_flow_to_layout(layout_id, player_id);

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_unassign_flow(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.unassign_flow_from_layout(layout_id, player_id);

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_move(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    from_index: usize,
    to_index: usize,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let moved = state.score.layouts.order.remove(from_index);
    state.score.layouts.order.insert(to_index, moved);

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_layouts_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.score.layouts.order.retain(|id| *id != layout_id);
    state.score.layouts.by_id.remove(&layout_id);

    state.ui.screens.setup.expanded.remove(&layout_id);
    state.ui.screens.setup.selection = None;

    if state.ui.layout_id == Some(layout_id) {
        state.ui.layout_id = None;
    }

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}
