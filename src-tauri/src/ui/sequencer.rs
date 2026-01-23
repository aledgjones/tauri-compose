use crate::score::components::duration::NoteDuration;
use crate::utils::UniqueId;
use crate::AppState;
use crate::RenderState;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Serialize_repr, Deserialize_repr, Clone, PartialEq)]
#[repr(u8)]
pub enum SequencerSelectionType {
    Tone = 0,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Selection {
    pub id: UniqueId,
    pub selection_type: SequencerSelectionType,
    pub track_id: UniqueId,
    pub flow_id: UniqueId,
    pub instrument_id: UniqueId,
}

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
    pub selection: Option<Selection>,
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
            selection: None,
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_instrument_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.instrument = Some(id);

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_track_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.track = Some(id);

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_tool_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    tool: Tool,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.tool = tool;

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_snap_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    snap: NoteDuration,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.snap = snap;

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_coords_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    x: u16,
    y: u16,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.x = x;
    state.ui.sequencer.y = y;

    state.emit(&app, &mut renderer, false);
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    selection_type: SequencerSelectionType,
    id: UniqueId,
    track_id: UniqueId,
    flow_id: UniqueId,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = Some(Selection {
        id,
        selection_type,
        track_id,
        flow_id,
        instrument_id,
    });

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_clear(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = None;

    state.emit(&app, &mut renderer, true);
}
