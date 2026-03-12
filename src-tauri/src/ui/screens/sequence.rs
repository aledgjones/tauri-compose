use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

use crate::AppContextState;
use crate::RendererContextState;

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum BottomPanelTab {
    Properties,
    Sequencer,
    Mixer,
    Instruments,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub bottom_panel_minimized: bool,
    pub bottom_panel_tab: BottomPanelTab,
    pub show_active_only: bool,
}

impl Sequence {
    pub fn new() -> Self {
        Sequence {
            bottom_panel_minimized: false,
            bottom_panel_tab: BottomPanelTab::Sequencer,
            show_active_only: true,
        }
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequence_bottom_panel_minimized_toggle(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.sequence.bottom_panel_minimized =
        !state.ui.screens.sequence.bottom_panel_minimized;

    state.emit_general_state(&app, &mut renderer, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequence_bottom_panel_tab_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    tab: BottomPanelTab,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.sequence.bottom_panel_tab = tab;

    state.emit_general_state(&app, &mut renderer, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequence_show_active_only_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    show_active_only: bool,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.sequence.show_active_only = show_active_only;

    state.emit_general_state(&app, &mut renderer, None);
}
