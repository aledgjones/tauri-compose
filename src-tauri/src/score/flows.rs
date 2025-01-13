use super::components::duration::{NoteDuration, Ticks};
use super::entries::{
    clef::Clef,
    time_signature::{TimeSignature, TimeSignatureDrawType},
    Entry,
};
use super::instruments::defs::get_instrument_definition;
use super::staves::{Stave, Staves};
use super::tracks::Track;
use crate::{utils::UniqueId, AppState, AppStateInner};
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
    pub subdivisions: Ticks,
    pub master_id: UniqueId,
    pub staves: Staves,
}

impl Flow {
    pub fn new(id: UniqueId, master_id: UniqueId) -> Flow {
        Flow {
            id,
            title: String::from(""),
            player_ids: FxHashSet::default(),
            length: 256 * 4 * 8, // 8 * 4/4
            subdivisions: 256,
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
        let instruments = player.instruments.clone();
        for instrument_id in instruments {
            self.assign_instrument_to_flow(flow_id, instrument_id)
        }
    }

    pub fn assign_instrument_to_flow(&mut self, flow_id: UniqueId, instrument_id: UniqueId) {
        let instrument = self.score.instruments.by_id[&instrument_id].clone();
        let def = get_instrument_definition(&instrument.definition_id).unwrap();

        for (i, stave_id) in instrument.staves.iter().enumerate() {
            let stave_def = &def.staves[i];

            // Create master track
            let master_id = self.generate_id();
            let mut master = Track::new(master_id);

            // Create and insert clef entry
            let clef_id = self.generate_id();
            master.insert(Entry::Clef(Clef::new(
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
            stave.tracks.push(track_id);

            // Insert tracks into score
            self.score.tracks.by_id.insert(track_id, track);
            self.score.tracks.by_id.insert(master_id, master);

            // Insert stave into flow
            let flow = self.score.flows.by_id.get_mut(&flow_id).unwrap();
            flow.staves.by_id.insert(*stave_id, stave);
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_flows_create(app: tauri::AppHandle, state: State<'_, AppState>) -> UniqueId {
    let mut state = state.lock().unwrap();

    let flow_id = state.generate_id();
    let master_id = state.generate_id();
    let time_signature_id = state.generate_id();

    let mut master = Track::new(master_id);
    let flow = Flow::new(flow_id, master_id);

    master.insert(Entry::TimeSignature(TimeSignature::new(
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
    let players = state.score.players.order.clone();
    for player_key in players {
        state.assign_player_to_flow(flow_id, player_key);
    }

    state.emit(&app);

    flow_id
}
