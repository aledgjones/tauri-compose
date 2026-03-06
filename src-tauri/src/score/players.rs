use super::layouts::LayoutType;
use crate::playback::sampler::sync_sampler_with_score;
use crate::utils::UniqueId;
use crate::{
    AppContext, AppContextState, AudioContextState, RendererContextState, SamplerContextState,
};
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

    pub fn reorder(&mut self, from_index: usize, to_index: usize) {
        let moved = self.order.remove(from_index);
        self.order.insert(to_index, moved);
    }
}

impl AppContext {
    pub fn create_player(&mut self, player_type: PlayerType) -> UniqueId {
        let player_id = self.generate_id();
        let player = Player::new(player_id, player_type);

        self.score.players.order.push(player_id);
        self.score.players.by_id.insert(player_id, player);

        for flow_id in self.score.flows.order.clone() {
            self.assign_player_to_flow(flow_id, player_id);
        }

        for layout_id in self.score.layouts.order.clone() {
            let layout = &self.score.layouts.by_id[&layout_id];
            if layout.layout_type == LayoutType::FullScore {
                self.assign_player_to_layout(layout_id, player_id);
            }
        }

        player_id
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    player_type: PlayerType,
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let player_id = state.create_player(player_type);

    state.emit(&app, &mut renderer, true);

    player_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_move(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    from_index: usize,
    to_index: usize,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.score.players.reorder(from_index, to_index);

    state.score.update_counts();
    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_assign_instrument(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    player_id: UniqueId,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let player = state.score.players.by_id.get_mut(&player_id).unwrap();
    player.instrument_ids.push(instrument_id);

    for flow_id in state.score.flows.order.clone() {
        let flow = &state.score.flows.by_id[&flow_id];
        if flow.player_ids.contains(&player_id) {
            state.assign_instrument_to_flow(flow_id, instrument_id);
        }
    }

    state.score.update_counts();
    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_unassign_instrument(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    player_id: UniqueId,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let player = state.score.players.by_id.get_mut(&player_id).unwrap();
    player.instrument_ids.retain(|id| *id != instrument_id);

    for flow_id in state.score.flows.order.clone() {
        let flow = &state.score.flows.by_id[&flow_id];
        if flow.player_ids.contains(&player_id) {
            state.unassign_instrument_from_flow(flow_id, instrument_id);
        }
    }

    state.score.update_counts();

    sync_sampler_with_score(
        &app,
        &state.score,
        sampler_state.inner(),
        audio_state.inner(),
        "after player instrument unassignment",
    );

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_players_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

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
    state.ui.screens.setup.expanded.remove(&player_id);
    state.ui.screens.setup.selection = None;

    // remove player from layouts and remove any empty parts
    for layout_id in state.score.layouts.order.clone() {
        let layout = state.score.layouts.by_id.get_mut(&layout_id).unwrap();
        layout.player_ids.remove(&player_id);

        if layout.layout_type == LayoutType::Part && layout.player_ids.is_empty() {
            state.score.layouts.order.retain(|id| *id != layout_id);
            state.score.layouts.by_id.remove(&layout_id);
        }
    }

    state.score.update_counts();

    sync_sampler_with_score(
        &app,
        &state.score,
        sampler_state.inner(),
        audio_state.inner(),
        "after player removal",
    );

    state.emit(&app, &mut renderer, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::Preferences;
    use crate::score::flows::{Flow, Flows};
    use crate::score::layouts::{Layout, LayoutType, Layouts};
    use crate::score::Score;
    use crate::ui::Ui;

    fn make_players(order: &[UniqueId]) -> Players {
        let mut players = Players::new();

        for player_id in order {
            players.order.push(*player_id);
            players
                .by_id
                .insert(*player_id, Player::new(*player_id, PlayerType::Solo));
        }

        players
    }

    fn make_app_context() -> AppContext {
        let mut score = Score::new();
        let flow_id = 50;
        let part_layout_id = 60;

        score.flows = Flows::new();
        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, Flow::new(flow_id, 100));
        score.layouts = Layouts::new();
        score.layouts.order.push(part_layout_id);
        score.layouts.by_id.insert(
            part_layout_id,
            Layout::new(part_layout_id, LayoutType::Part),
        );

        AppContext {
            ui: Ui::new(),
            score,
            preferences: serde_json::from_str::<Preferences>(r#"{"audition":true}"#).unwrap(),
            next_id: 1,
            current_file_path: None,
        }
    }

    #[test]
    fn players_new_is_empty() {
        let p = Players::new();
        assert!(p.order.is_empty());
        assert!(p.by_id.is_empty());
    }

    #[test]
    fn player_new_stores_id() {
        let p = Player::new(5, PlayerType::Solo);
        assert_eq!(p.id, 5);
    }

    #[test]
    fn player_new_solo_type() {
        let p = Player::new(1, PlayerType::Solo);
        assert!(matches!(p.player_type, PlayerType::Solo));
    }

    #[test]
    fn player_new_section_type() {
        let p = Player::new(2, PlayerType::Section);
        assert!(matches!(p.player_type, PlayerType::Section));
    }

    #[test]
    fn player_new_has_empty_instrument_ids() {
        let p = Player::new(1, PlayerType::Solo);
        assert!(p.instrument_ids.is_empty());
    }

    #[test]
    fn player_instrument_ids_can_push() {
        let mut p = Player::new(1, PlayerType::Solo);
        p.instrument_ids.push(10);
        assert_eq!(p.instrument_ids.len(), 1);
        assert_eq!(p.instrument_ids[0], 10);
    }

    #[test]
    fn players_by_id_can_insert() {
        let mut ps = Players::new();
        let player = Player::new(7, PlayerType::Solo);
        ps.by_id.insert(7, player);
        ps.order.push(7);
        assert_eq!(ps.by_id.len(), 1);
        assert_eq!(ps.order.len(), 1);
    }

    #[test]
    fn reorder_moves_player_within_order() {
        let mut players = make_players(&[1, 2, 3]);

        players.reorder(2, 0);

        assert_eq!(players.order, vec![3, 1, 2]);
    }

    #[test]
    fn create_player_wires_new_player_into_flows_and_full_score_layouts() {
        let mut app = make_app_context();

        let player_id = app.create_player(PlayerType::Section);

        assert_eq!(player_id, 1);
        assert_eq!(app.score.players.order, vec![player_id]);
        assert!(app.score.players.by_id.contains_key(&player_id));
        assert!(app.score.flows.by_id[&50].player_ids.contains(&player_id));
        assert!(app.score.layouts.by_id[&0].player_ids.contains(&player_id));
        assert!(!app.score.layouts.by_id[&60].player_ids.contains(&player_id));
    }

    #[test]
    fn player_type_solo_and_section_are_different() {
        assert!(PlayerType::Solo != PlayerType::Section);
    }

    #[test]
    fn player_type_solo_equality() {
        assert!(matches!(PlayerType::Solo, PlayerType::Solo));
    }

    #[test]
    fn player_type_section_equality() {
        assert!(matches!(PlayerType::Section, PlayerType::Section));
    }
}
