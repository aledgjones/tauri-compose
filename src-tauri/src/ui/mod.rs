pub mod sequencer;
pub mod setup;

use crate::{utils::UniqueId, AppState};
use sequencer::Sequencer;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use setup::Setup;
use tauri::State;

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum Tab {
    Setup = 0,
    Write = 1,
    Engrave = 2,
    Play = 3,
    Print = 4,
}

#[derive(Serialize, Clone)]
pub struct Ui {
    tab: Tab,
    layout_id: UniqueId,
    pub setup: Setup,
    pub sequencer: Sequencer,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            tab: Tab::Setup,
            layout_id: 0,
            setup: Setup::new(),
            sequencer: Sequencer::new(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_tab_set(app: tauri::AppHandle, state: State<'_, AppState>, tab: Tab) {
    let mut state = state.lock().unwrap();
    state.ui.tab = tab;
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_layout_id_set(app: tauri::AppHandle, state: State<'_, AppState>, layout_id: UniqueId) {
    let mut state = state.lock().unwrap();
    state.ui.layout_id = layout_id;
    state.emit(&app);
}
