use super::components::duration::{NoteDuration, Ticks};
use super::entries::{
    clef::Clef,
    time_signature::{TimeSignature, TimeSignatureDrawType},
    Entry,
};
use super::instruments::defs::get_instrument_definition;
use super::staves::{Stave, Staves};
use super::tracks::Track;
use crate::{utils::UniqueId, AppContext, AppContextState, RendererContextState};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct Flows {
    pub order: Vec<UniqueId>,
    pub by_id: FxHashMap<UniqueId, Flow>,
}

impl Flows {
    pub fn new() -> Flows {
        Flows {
            order: Vec::new(),
            by_id: FxHashMap::default(),
        }
    }

    pub fn rename(&mut self, flow_id: UniqueId, title: String) {
        let flow = self.by_id.get_mut(&flow_id).unwrap();
        flow.title = title;
    }

    pub fn reorder(&mut self, from_index: usize, to_index: usize) {
        let moved = self.order.remove(from_index);
        self.order.insert(to_index, moved);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Flow {
    pub id: UniqueId,
    pub title: String,
    pub player_ids: FxHashSet<UniqueId>, // purely for inclusion lookup -- order comes from score.players.order
    pub length: Ticks,                   // number of subdivision ticks in the flow
    pub master_id: UniqueId,
    pub staves: Staves,
}

impl Flow {
    pub fn new(id: UniqueId, master_id: UniqueId) -> Flow {
        Flow {
            id,
            title: String::from(""),
            player_ids: FxHashSet::default(),
            length: 32 * 4 * 16, // 16 * 4/4
            master_id,
            staves: Staves::new(),
        }
    }
}

impl AppContext {
    pub fn assign_player_to_flow(&mut self, flow_id: UniqueId, player_id: UniqueId) {
        let flow = self.score.flows.by_id.get_mut(&flow_id).unwrap();

        flow.player_ids.insert(player_id);

        let player = &self.score.players.by_id[&player_id];
        let instruments = player.instrument_ids.clone();
        for instrument_id in instruments {
            self.assign_instrument_to_flow(flow_id, instrument_id)
        }
    }

    pub fn assign_instrument_to_flow(&mut self, flow_id: UniqueId, instrument_id: UniqueId) {
        let instrument = self.score.instruments.by_id[&instrument_id].clone();
        let def = get_instrument_definition(&instrument.definition_id).unwrap();

        for (i, stave_id) in instrument.stave_ids.iter().enumerate() {
            let stave_def = &def.staves[i];

            // Create master track
            let master_id = self.generate_id();
            let mut master = Track::new(master_id);

            // Create and insert clef entry
            let clef_id = self.generate_id();
            master.insert_entry(Entry::Clef(Clef::new(
                clef_id,
                0,
                stave_def.clef.pitch,
                stave_def.clef.offset,
                stave_def.clef.draw_as,
            )));

            // Create track
            let track_id = self.generate_id();
            let track = Track::new(track_id);

            // Create stave
            let mut stave = Stave::new(*stave_id, stave_def, master_id);
            stave.track_ids.push(track_id);

            // Insert tracks into score
            self.score.tracks.by_id.insert(track_id, track);
            self.score.tracks.by_id.insert(master_id, master);

            // Insert stave into flow
            let flow = self.score.flows.by_id.get_mut(&flow_id).unwrap();
            flow.staves.by_id.insert(*stave_id, stave);
        }
    }

    pub fn unassign_player_from_flow(&mut self, flow_id: UniqueId, player_id: UniqueId) {
        let flow = self.score.flows.by_id.get_mut(&flow_id).unwrap();

        flow.player_ids.remove(&player_id);

        for instrument_id in self.score.players.by_id[&player_id].instrument_ids.clone() {
            self.unassign_instrument_from_flow(flow_id, instrument_id);
        }
    }

    pub fn unassign_instrument_from_flow(&mut self, flow_id: UniqueId, instrument_id: UniqueId) {
        let flow = self.score.flows.by_id.get_mut(&flow_id).unwrap();

        for stave_id in &self.score.instruments.by_id[&instrument_id].stave_ids {
            if let Some(stave) = flow.staves.by_id.get(stave_id) {
                self.score.tracks.by_id.remove(&stave.master_id);
                for track_id in &stave.track_ids {
                    self.score.tracks.by_id.remove(track_id);
                }
                flow.staves.by_id.remove(stave_id);
            };
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow_id = state.generate_id();
    let master_id = state.generate_id();
    let time_signature_id = state.generate_id();

    let mut master = Track::new(master_id);
    let flow = Flow::new(flow_id, master_id);

    master.insert_entry(Entry::TimeSignature(TimeSignature::new(
        time_signature_id,
        0,
        4,
        NoteDuration::Quarter,
        TimeSignatureDrawType::Regular,
        None,
    )));

    state.score.tracks.by_id.insert(master_id, master);
    state.score.flows.order.push(flow_id);
    state.score.flows.by_id.insert(flow_id, flow);

    // add all the player keys into the new flow
    for player_id in state.score.players.order.clone() {
        state.assign_player_to_flow(flow_id, player_id);
    }

    // add flow to all layouts
    for layout_id in state.score.layouts.order.clone() {
        state.assign_flow_to_layout(layout_id, flow_id);
    }

    state.emit(&app, &mut renderer, true);

    flow_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_rename(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    title: String,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.score.flows.rename(flow_id, title);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_move(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    from_index: usize,
    to_index: usize,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.score.flows.reorder(from_index, to_index);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_assign_player(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.assign_player_to_flow(flow_id, player_id);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_unassign_player(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.unassign_player_from_flow(flow_id, player_id);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    for player_id in state.score.players.order.clone() {
        state.unassign_player_from_flow(flow_id, player_id);
    }

    let flow = state.score.flows.by_id.remove(&flow_id).unwrap();

    state.score.flows.order.retain(|id| *id != flow_id);
    state.score.tracks.by_id.remove(&flow.master_id);

    state.ui.screens.setup.selection = None;

    if state.ui.flow_id == Some(flow_id) {
        state.ui.flow_id = None;
    }

    for layout_id in state.score.layouts.order.clone() {
        state.unassign_flow_from_layout(layout_id, flow_id);
    }

    state.emit(&app, &mut renderer, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::Preferences;
    use crate::score::instruments::{Instrument, Instruments};
    use crate::score::layouts::Layouts;
    use crate::score::players::{Player, PlayerType, Players};
    use crate::score::Score;
    use crate::ui::Ui;
    use crate::AppContext;

    fn make_flows(order: &[UniqueId]) -> Flows {
        let mut flows = Flows::new();

        for flow_id in order {
            flows.order.push(*flow_id);
            flows
                .by_id
                .insert(*flow_id, Flow::new(*flow_id, flow_id + 100));
        }

        flows
    }

    fn make_instrument(id: UniqueId, definition_id: &str, stave_ids: Vec<UniqueId>) -> Instrument {
        let definition_id = definition_id.to_string();
        let def = get_instrument_definition(&definition_id).unwrap();
        let instrument_type = def.instrument_type;
        let long_name = def.long_name.to_string();
        let short_name = def.short_name.to_string();

        Instrument {
            id,
            definition_id,
            instrument_type,
            long_name,
            short_name,
            stave_ids,
            count: None,
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    fn make_app_context(players: Vec<Player>, instruments: Vec<Instrument>) -> AppContext {
        let mut score = Score::new();
        let flow_id = 50;
        let master_id = 60;

        score.players = Players {
            order: players.iter().map(|player| player.id).collect(),
            by_id: players
                .into_iter()
                .map(|player| (player.id, player))
                .collect(),
        };
        score.instruments = Instruments {
            by_id: instruments
                .into_iter()
                .map(|instrument| (instrument.id, instrument))
                .collect(),
        };
        score.layouts = Layouts::new();
        score.flows.order.push(flow_id);
        score
            .flows
            .by_id
            .insert(flow_id, Flow::new(flow_id, master_id));

        AppContext {
            ui: Ui::new(),
            score,
            preferences: serde_json::from_str::<Preferences>(r#"{"audition":true}"#).unwrap(),
            next_id: 1,
            current_file_path: None,
        }
    }

    #[test]
    fn flows_new_is_empty() {
        let f = Flows::new();
        assert!(f.order.is_empty());
        assert!(f.by_id.is_empty());
    }

    #[test]
    fn rename_updates_flow_title() {
        let mut flows = make_flows(&[1]);

        flows.rename(1, String::from("Intro"));

        assert_eq!(flows.by_id[&1].title, "Intro");
    }

    #[test]
    fn reorder_moves_flow_id_within_order() {
        let mut flows = make_flows(&[10, 20, 30]);

        flows.reorder(0, 2);

        assert_eq!(flows.order, vec![20, 30, 10]);
    }

    #[test]
    fn assign_instrument_to_flow_creates_staves_and_tracks() {
        let instrument = make_instrument(10, "strings.violin", vec![100]);
        let mut app = make_app_context(vec![], vec![instrument]);

        app.assign_instrument_to_flow(50, 10);

        let flow = &app.score.flows.by_id[&50];
        let stave = &flow.staves.by_id[&100];

        assert_eq!(flow.staves.by_id.len(), 1);
        assert_eq!(stave.track_ids.len(), 1);
        assert!(app.score.tracks.by_id.contains_key(&stave.master_id));
        assert!(app.score.tracks.by_id.contains_key(&stave.track_ids[0]));
    }

    #[test]
    fn assign_player_to_flow_wires_all_player_instruments() {
        let players = vec![Player {
            id: 1,
            player_type: PlayerType::Section,
            instrument_ids: vec![10, 11],
        }];
        let instruments = vec![
            make_instrument(10, "strings.violin", vec![100]),
            make_instrument(11, "woodwinds.flute", vec![101]),
        ];
        let mut app = make_app_context(players, instruments);

        app.assign_player_to_flow(50, 1);

        let flow = &app.score.flows.by_id[&50];

        assert!(flow.player_ids.contains(&1));
        assert!(flow.staves.by_id.contains_key(&100));
        assert!(flow.staves.by_id.contains_key(&101));
        assert_eq!(app.score.tracks.by_id.len(), 4);
    }

    #[test]
    fn unassign_instrument_from_flow_removes_stave_and_tracks() {
        let instrument = make_instrument(10, "strings.violin", vec![100]);
        let mut app = make_app_context(vec![], vec![instrument]);

        app.assign_instrument_to_flow(50, 10);
        let stave = app.score.flows.by_id[&50].staves.by_id[&100].clone();

        app.unassign_instrument_from_flow(50, 10);

        assert!(!app.score.flows.by_id[&50].staves.by_id.contains_key(&100));
        assert!(!app.score.tracks.by_id.contains_key(&stave.master_id));
        assert!(!app.score.tracks.by_id.contains_key(&stave.track_ids[0]));
    }

    #[test]
    fn unassign_player_from_flow_removes_player_and_all_staves() {
        let players = vec![Player {
            id: 1,
            player_type: PlayerType::Section,
            instrument_ids: vec![10, 11],
        }];
        let instruments = vec![
            make_instrument(10, "strings.violin", vec![100]),
            make_instrument(11, "woodwinds.flute", vec![101]),
        ];
        let mut app = make_app_context(players, instruments);

        app.assign_player_to_flow(50, 1);
        app.unassign_player_from_flow(50, 1);

        let flow = &app.score.flows.by_id[&50];
        assert!(!flow.player_ids.contains(&1));
        assert!(flow.staves.by_id.is_empty());
        assert!(app.score.tracks.by_id.is_empty());
    }

    #[test]
    fn flow_new_stores_id() {
        let f = Flow::new(42, 100);
        assert_eq!(f.id, 42);
    }

    #[test]
    fn flow_new_stores_master_id() {
        let f = Flow::new(42, 100);
        assert_eq!(f.master_id, 100);
    }

    #[test]
    fn flow_new_has_empty_player_ids() {
        let f = Flow::new(1, 2);
        assert!(f.player_ids.is_empty());
    }

    #[test]
    fn flow_new_has_positive_default_length() {
        let f = Flow::new(1, 2);
        assert!(f.length > 0);
    }

    #[test]
    fn flow_new_has_empty_title() {
        let f = Flow::new(1, 2);
        assert_eq!(f.title, "");
    }

    #[test]
    fn flow_new_has_empty_staves() {
        let f = Flow::new(1, 2);
        assert!(f.staves.by_id.is_empty());
    }

    #[test]
    fn flows_order_can_push() {
        let mut flows = Flows::new();
        let f = Flow::new(10, 20);
        flows.by_id.insert(10, f);
        flows.order.push(10);
        assert_eq!(flows.order.len(), 1);
        assert!(flows.by_id.contains_key(&10));
    }

    #[test]
    fn flow_player_ids_can_insert() {
        let mut f = Flow::new(1, 2);
        f.player_ids.insert(99);
        assert!(f.player_ids.contains(&99));
    }

    #[test]
    fn flow_player_ids_can_remove() {
        let mut f = Flow::new(1, 2);
        f.player_ids.insert(99);
        f.player_ids.remove(&99);
        assert!(!f.player_ids.contains(&99));
    }

    #[test]
    fn flow_title_can_be_set() {
        let mut f = Flow::new(1, 2);
        f.title = String::from("Movement I");
        assert_eq!(f.title, "Movement I");
    }

    #[test]
    fn flow_length_can_be_modified() {
        let mut f = Flow::new(1, 2);
        f.length = 256;
        assert_eq!(f.length, 256);
    }
}
