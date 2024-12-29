mod ui;

use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use ui::{ui_set_active_layout_id, ui_set_tab, Ui};

#[derive(Serialize, Clone)]
struct AppStateInner {
    ui: Ui,
}

impl AppStateInner {
    fn new() -> AppState {
        Mutex::new(Self { ui: Ui::new() })
    }

    fn emit(&self, app: &tauri::AppHandle) {
        app.emit("solocompose://state", self.clone()).unwrap();
    }
}

type AppState = Mutex<AppStateInner>;

#[tauri::command]
fn initialize(app: tauri::AppHandle, state: State<'_, AppState>) {
    let state = state.lock().unwrap();
    state.emit(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppStateInner::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize,
            ui_set_tab,
            ui_set_active_layout_id
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
