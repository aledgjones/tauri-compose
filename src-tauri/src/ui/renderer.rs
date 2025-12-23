use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct Renderer {
    pub zoom: i32,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer { zoom: 100 }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_zoom_set(app: tauri::AppHandle, state: State<'_, AppState>, zoom: i32) {
    let mut state = state.lock().unwrap();
    state.ui.renderer.zoom = zoom;
    state.emit(&app);
}
