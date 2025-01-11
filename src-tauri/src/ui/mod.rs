pub mod setup;

use crate::AppState;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use setup::Setup;
use tauri::State;

type LayoutId = String;

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
    layout_id: LayoutId,
    pub setup: Setup,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            tab: Tab::Setup,
            layout_id: "full-score".to_owned(),
            setup: Setup::new(),
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
pub fn ui_layout_id_set(app: tauri::AppHandle, state: State<'_, AppState>, layout_id: LayoutId) {
    let mut state = state.lock().unwrap();
    state.ui.layout_id = layout_id;
    state.emit(&app);
}
