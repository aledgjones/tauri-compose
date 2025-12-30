use crate::score::components::duration::NoteDuration;
use crate::utils::UniqueId;
use crate::AppState;
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
    pub x: u16,
    pub y: u16,
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

    state.emit(&app, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_track_set(app: tauri::AppHandle, state: State<'_, AppState>, id: UniqueId) {
    let mut state = state.lock().unwrap();
    state.ui.sequencer.track = Some(id);

    state.emit(&app, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_tool_set(app: tauri::AppHandle, state: State<'_, AppState>, tool: Tool) {
    let mut state = state.lock().unwrap();
    state.ui.sequencer.tool = tool;
    state.emit(&app, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_snap_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    snap: NoteDuration,
) {
    let mut state = state.lock().unwrap();
    state.ui.sequencer.snap = snap;
    state.emit(&app, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_coords_set(app: tauri::AppHandle, state: State<'_, AppState>, x: u16, y: u16) {
    let mut state = state.lock().unwrap();
    state.ui.sequencer.x = x;
    state.ui.sequencer.y = y;
    state.emit(&app, None);
}
