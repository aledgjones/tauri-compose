use super::components::duration::{NoteDuration, Ticks};
use super::entries::{
    clef::Clef,
    time_signature::{TimeSignature, TimeSignatureDrawType},
    Entry,
};
use super::instruments::defs::get_instrument_definition;
use super::staves::{Stave, Staves};
use super::tracks::Track;
use crate::{utils::UniqueId, AppState, AppStateInner, RenderState};
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

impl AppStateInner {
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
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
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
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    title: String,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = state.score.flows.by_id.get_mut(&flow_id).unwrap();
    flow.title = title;

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_move(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    from_index: usize,
    to_index: usize,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let moved = state.score.flows.order.remove(from_index);
    state.score.flows.order.insert(to_index, moved);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_assign_player(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
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
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
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
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
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

    state.ui.setup.selection = None;

    for layout_id in state.score.layouts.order.clone() {
        state.unassign_flow_from_layout(layout_id, flow_id);
    }

    state.emit(&app, &mut renderer, true);
}
