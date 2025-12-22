use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

use crate::AppState;

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum BottomPanelTab {
    Properties,
    Sequencer,
    Mixer,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub bottom_panel_minimized: bool,
    pub bottom_panel_tab: BottomPanelTab,
}

impl Sequence {
    pub fn new() -> Self {
        Sequence {
            bottom_panel_minimized: false,
            bottom_panel_tab: BottomPanelTab::Sequencer,
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
    state: State<'_, AppState>,
) {
    let mut state = state.lock().unwrap();
    state.ui.sequence.bottom_panel_minimized = !state.ui.sequence.bottom_panel_minimized;

    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_sequence_bottom_panel_tab_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tab: BottomPanelTab,
) {
    let mut state = state.lock().unwrap();
    state.ui.sequence.bottom_panel_tab = tab;

    state.emit(&app);
}
