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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Write {
    pub bottom_panel_minimized: bool,
    pub bottom_panel_tab: BottomPanelTab,
}

impl Write {
    pub fn new() -> Self {
        Write {
            bottom_panel_minimized: false,
            bottom_panel_tab: BottomPanelTab::Sequencer,
        }
    }
}

impl Default for Write {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_write_bottom_panel_minimized_toggle(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.write.bottom_panel_minimized = !state.ui.screens.write.bottom_panel_minimized;

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_write_bottom_panel_tab_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    tab: BottomPanelTab,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.write.bottom_panel_tab = tab;

    state.emit(&app, &mut renderer, false);
}
