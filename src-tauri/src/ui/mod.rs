use crate::AppState;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
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
    layout: LayoutId,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            tab: Tab::Setup,
            layout: "full-score".to_owned(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_set_tab(app: tauri::AppHandle, state: State<'_, AppState>, tab: Tab) {
    let mut state = state.lock().unwrap();
    state.ui.tab = tab;
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_set_active_layout_id(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_id: LayoutId,
) {
    let mut state = state.lock().unwrap();
    state.ui.layout = layout_id;
    state.emit(&app);
}
