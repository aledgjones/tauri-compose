use crate::score::components::duration::NoteDuration;
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum Selection {
    Tone {
        flow_id: UniqueId,
        instrument_id: UniqueId,
        stave_id: UniqueId,
        track_id: UniqueId,
        tone_id: UniqueId,
    },
    Rest {
        flow_id: UniqueId,
        instrument_id: UniqueId,
        stave_id: UniqueId,
        track_id: UniqueId,
        tick: u64,
    },
    Tempo {
        flow_id: UniqueId,
        tempo_id: UniqueId,
        tick: u64,
    },
    TimeSignature {
        flow_id: UniqueId,
        time_signature_id: UniqueId,
        tick: u64,
    },
    KeySignature {
        flow_id: UniqueId,
        key_signature_id: UniqueId,
        tick: u64,
    },
    Clef {
        flow_id: UniqueId,
        stave_id: UniqueId,
        clef_id: UniqueId,
        tick: u64,
    },
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
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
pub fn ui_sequencer_selection_set_tone(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_id: UniqueId,
    track_id: UniqueId,
    tone_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = Some(Selection::Tone {
        flow_id,
        instrument_id,
        stave_id,
        track_id,
        tone_id,
    });

    state.emit(&app, &mut renderer, true);
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_rest(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_id: UniqueId,
    track_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = Some(Selection::Rest {
        flow_id,
        instrument_id,
        stave_id,
        track_id,
        tick,
    });

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_tempo(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    tempo_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = Some(Selection::Tempo {
        flow_id,
        tempo_id,
        tick,
    });

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_time_signature(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    time_signature_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = Some(Selection::TimeSignature {
        flow_id,
        time_signature_id,
        tick,
    });

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_key_signature(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    key_signature_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = Some(Selection::KeySignature {
        flow_id,
        key_signature_id,
        tick,
    });

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_clef(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    clef_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = Some(Selection::Clef {
        flow_id,
        stave_id,
        clef_id,
        tick,
    });

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_clear(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.sequencer.selection = None;

    state.emit(&app, &mut renderer, true);
}

#[cfg(test)]
mod tests {
    use super::Selection;

    #[test]
    fn tempo_selection_serializes_with_expected_shape() {
        let selection = Selection::Tempo {
            flow_id: 11,
            tempo_id: 12,
            tick: 96,
        };

        let json = serde_json::to_value(selection).unwrap();

        assert_eq!(json["type"], "Tempo");
        assert_eq!(json["flow_id"], 11);
        assert_eq!(json["tempo_id"], 12);
        assert_eq!(json["tick"], 96);
    }

    #[test]
    fn time_signature_selection_serializes_with_expected_shape() {
        let selection = Selection::TimeSignature {
            flow_id: 11,
            time_signature_id: 13,
            tick: 96,
        };

        let json = serde_json::to_value(selection).unwrap();

        assert_eq!(json["type"], "TimeSignature");
        assert_eq!(json["flow_id"], 11);
        assert_eq!(json["time_signature_id"], 13);
        assert_eq!(json["tick"], 96);
    }

    #[test]
    fn key_signature_selection_serializes_with_expected_shape() {
        let selection = Selection::KeySignature {
            flow_id: 11,
            key_signature_id: 14,
            tick: 96,
        };

        let json = serde_json::to_value(selection).unwrap();

        assert_eq!(json["type"], "KeySignature");
        assert_eq!(json["flow_id"], 11);
        assert_eq!(json["key_signature_id"], 14);
        assert_eq!(json["tick"], 96);
    }

    #[test]
    fn clef_selection_serializes_with_expected_shape() {
        let selection = Selection::Clef {
            flow_id: 11,
            stave_id: 4,
            clef_id: 15,
            tick: 96,
        };

        let json = serde_json::to_value(selection).unwrap();

        assert_eq!(json["type"], "Clef");
        assert_eq!(json["flow_id"], 11);
        assert_eq!(json["stave_id"], 4);
        assert_eq!(json["clef_id"], 15);
        assert_eq!(json["tick"], 96);
    }
}
