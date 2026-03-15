use crate::score::components::duration::NoteDuration;
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;
use crate::{
    instructions::FlowRenderRecalculationRequest,
    playback::transport::{sync_transport_to_app_state, TransportSharedState},
    ui::Ui,
};
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
        tuplet_id: Option<UniqueId>,
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
    Tuplet {
        flow_id: UniqueId,
        instrument_id: UniqueId,
        stave_id: UniqueId,
        track_id: UniqueId,
        tuplet_id: UniqueId,
        tick: u64,
    },
}

impl Selection {
    fn flow_id(&self) -> UniqueId {
        match self {
            Selection::Tone { flow_id, .. }
            | Selection::Rest { flow_id, .. }
            | Selection::Tempo { flow_id, .. }
            | Selection::TimeSignature { flow_id, .. }
            | Selection::KeySignature { flow_id, .. }
            | Selection::Clef { flow_id, .. }
            | Selection::Tuplet { flow_id, .. } => *flow_id,
        }
    }
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

fn apply_selection(ui: &mut Ui, selection: Selection) {
    ui.flow_id = Some(selection.flow_id());
    ui.sequencer.selection = Some(selection);
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

    state.emit_general_state(&app, &mut renderer, None);
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

    state.emit_general_state(&app, &mut renderer, None);
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

    state.emit_general_state(&app, &mut renderer, None);
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

    state.emit_general_state(&app, &mut renderer, None);
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

    state.emit_general_state(&app, &mut renderer, None);
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_tone(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_id: UniqueId,
    track_id: UniqueId,
    tone_id: UniqueId,
    tuplet_id: Option<UniqueId>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        Some(flow_id),
    );

    apply_selection(
        &mut state.ui,
        Selection::Tone {
            flow_id,
            instrument_id,
            stave_id,
            track_id,
            tone_id,
            tuplet_id,
        },
    );
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_rest(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_id: UniqueId,
    track_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        Some(flow_id),
    );

    apply_selection(
        &mut state.ui,
        Selection::Rest {
            flow_id,
            instrument_id,
            stave_id,
            track_id,
            tick,
        },
    );
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_tempo(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    tempo_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        Some(flow_id),
    );

    apply_selection(
        &mut state.ui,
        Selection::Tempo {
            flow_id,
            tempo_id,
            tick,
        },
    );
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_time_signature(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    time_signature_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        Some(flow_id),
    );

    apply_selection(
        &mut state.ui,
        Selection::TimeSignature {
            flow_id,
            time_signature_id,
            tick,
        },
    );
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_key_signature(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    key_signature_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        Some(flow_id),
    );

    apply_selection(
        &mut state.ui,
        Selection::KeySignature {
            flow_id,
            key_signature_id,
            tick,
        },
    );
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_clef(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    clef_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        Some(flow_id),
    );

    apply_selection(
        &mut state.ui,
        Selection::Clef {
            flow_id,
            stave_id,
            clef_id,
            tick,
        },
    );
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_set_tuplet(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_id: UniqueId,
    track_id: UniqueId,
    tuplet_id: UniqueId,
    tick: u64,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        Some(flow_id),
    );

    apply_selection(
        &mut state.ui,
        Selection::Tuplet {
            flow_id,
            instrument_id,
            stave_id,
            track_id,
            tuplet_id,
            tick,
        },
    );
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequencer_selection_clear(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let recalculation_request = FlowRenderRecalculationRequest::for_local_update(
        state
            .ui
            .sequencer
            .selection
            .as_ref()
            .map(Selection::flow_id),
        None,
    );

    state.ui.sequencer.selection = None;

    state.emit_general_state(&app, &mut renderer, recalculation_request);
}

#[cfg(test)]
mod tests {
    use super::{apply_selection, Selection};
    use crate::ui::Ui;

    #[test]
    fn apply_selection_updates_the_shared_active_flow() {
        let mut ui = Ui::new();

        apply_selection(
            &mut ui,
            Selection::Clef {
                flow_id: 11,
                stave_id: 12,
                clef_id: 13,
                tick: 96,
            },
        );

        assert_eq!(ui.flow_id, Some(11));
        assert_eq!(
            ui.sequencer.selection,
            Some(Selection::Clef {
                flow_id: 11,
                stave_id: 12,
                clef_id: 13,
                tick: 96,
            })
        );
    }

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

    #[test]
    fn tuplet_selection_serializes_with_expected_shape() {
        let selection = Selection::Tuplet {
            flow_id: 11,
            instrument_id: 12,
            stave_id: 13,
            track_id: 14,
            tuplet_id: 15,
            tick: 96,
        };

        let json = serde_json::to_value(selection).unwrap();

        assert_eq!(json["type"], "Tuplet");
        assert_eq!(json["flow_id"], 11);
        assert_eq!(json["instrument_id"], 12);
        assert_eq!(json["stave_id"], 13);
        assert_eq!(json["track_id"], 14);
        assert_eq!(json["tuplet_id"], 15);
        assert_eq!(json["tick"], 96);
    }
}
