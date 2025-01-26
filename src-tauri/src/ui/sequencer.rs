use crate::{score::components::duration::NoteDuration, utils::UniqueId, AppState};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum Tool {
    Select,
    Write,
    Slice,
    Erase,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sequencer {
    pub x: i16,
    pub y: i16,
    pub tool: Tool,
    pub snap: NoteDuration,
    pub flow: Option<UniqueId>,
    pub instrument: Option<UniqueId>,
    pub track: Option<UniqueId>,
}

impl Sequencer {
    pub fn new() -> Self {
        Sequencer {
            x: 0,
            y: 1008,
            tool: Tool::Select,
            snap: NoteDuration::Eighth,
            flow: None,
            instrument: None,
            track: None,
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_instrument_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    state.ui.sequencer.instrument = Some(id);

    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_track_set(app: tauri::AppHandle, state: State<'_, AppState>, id: UniqueId) {
    let mut state = state.lock().unwrap();
    state.ui.sequencer.track = Some(id);

    state.emit(&app);
}

// #[wasm_bindgen]
// impl Core {
//     pub fn ui_sequence_get_x(&self) -> i16 {
//         self.file.ui.sequencer.x
//     }

//     pub fn ui_sequence_get_y(&self) -> i16 {
//         self.file.ui.sequencer.y
//     }

//     pub fn ui_sequence_set_coords(&mut self, x: i16, y: i16) {
//         self.file.ui.sequencer.x = x;
//         self.file.ui.sequencer.y = y;
//         self.emit();
//     }

//     pub fn ui_sequence_get_tool(&self) -> Tool {
//         self.file.ui.sequencer.tool
//     }

//     pub fn ui_sequence_set_tool(&mut self, tool: Tool) {
//         self.file.ui.sequencer.tool = tool;
//         self.emit();
//     }

//     pub fn ui_sequence_get_snap(&self) -> NoteDuration {
//         self.file.ui.sequencer.snap
//     }

//     pub fn ui_sequence_set_snap(&mut self, duration: NoteDuration) {
//         self.file.ui.sequencer.snap = duration;

//         for flow_key in self.file.score.flows.order.clone() {
//             self.calculate_flow_ticks(&flow_key);
//         }

//         self.emit();
//     }

//     pub fn ui_sequence_get_flow(&self) -> Option<String> {
//         match &self.file.ui.sequencer.flow {
//             None => self.file.score.flows.order.first().cloned(),
//             Some(selection) => match self.file.score.flows.by_key.contains_key(selection) {
//                 true => Some(selection.clone()),
//                 false => self.file.score.flows.order.first().cloned(),
//             },
//         }
//     }

//     pub fn ui_sequence_set_flow(&mut self, flow_key: String) {
//         self.file.ui.sequencer.flow = Some(flow_key);
//         self.emit();
//     }

//     pub fn ui_sequence_get_instrument(&self) -> Option<String> {
//         match &self.file.ui.sequencer.instrument {
//             None => self.get_first_instrument(),
//             Some(selection) => match self.file.score.instruments.contains_key(selection) {
//                 true => Some(selection.clone()),
//                 false => self.get_first_instrument(),
//             },
//         }
//     }

//     pub fn ui_sequence_set_instrument(&mut self, instrument_key: String) {
//         self.file.ui.sequencer.instrument = Some(instrument_key);
//         self.file.ui.sequencer.track = None;
//         self.emit();
//     }

//     pub fn ui_sequence_get_track(&self) -> Option<String> {
//         match self.ui_sequence_get_flow() {
//             None => None,
//             Some(flow_key) => match self.ui_sequence_get_instrument() {
//                 None => None,
//                 Some(instrument_key) => {
//                     let flow = &self.file.score.flows.by_key[&flow_key];
//                     let instrument = &self.file.score.instruments[&instrument_key];
//                     let first_stave_key = instrument.staves.first().unwrap();
//                     match flow.staves.get(first_stave_key) {
//                         None => None,
//                         Some(first_stave) => match &self.file.ui.sequencer.track {
//                             None => first_stave.tracks.first().cloned(),
//                             Some(selection) => {
//                                 for stave_key in &instrument.staves {
//                                     let stave = flow.staves.get(stave_key).unwrap();
//                                     if stave.tracks.contains(selection) {
//                                         return Some(selection.clone());
//                                     };
//                                 }

//                                 first_stave.tracks.first().cloned()
//                             }
//                         },
//                     }
//                 }
//             },
//         }
//     }

//     pub fn ui_sequence_set_track(&mut self, track_key: String) {
//         self.file.ui.sequencer.track = Some(track_key);
//         self.emit();
//     }
// }
