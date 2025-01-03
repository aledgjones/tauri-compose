mod renderer;
mod score;
mod ui;
mod utils;

use renderer::render;
use score::meta::{
    set_score_meta_arranger, set_score_meta_artist, set_score_meta_composer,
    set_score_meta_composer_dates, set_score_meta_composition_year, set_score_meta_copyist,
    set_score_meta_copyright, set_score_meta_dedication, set_score_meta_editor,
    set_score_meta_lyricist, set_score_meta_other_information, set_score_meta_publisher,
    set_score_meta_subtitle, set_score_meta_title, set_score_meta_work_number,
};
use score::Score;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use ui::{set_ui_layout_id, set_ui_tab, Ui};

#[derive(Serialize, Clone)]
struct AppStateInner {
    ui: Ui,
    score: Score,
}

impl AppStateInner {
    fn new() -> AppState {
        Mutex::new(Self {
            ui: Ui::new(),
            score: Score::new(),
        })
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
            render,
            set_ui_tab,
            set_ui_layout_id,
            set_score_meta_title,
            set_score_meta_subtitle,
            set_score_meta_dedication,
            set_score_meta_composer,
            set_score_meta_arranger,
            set_score_meta_lyricist,
            set_score_meta_artist,
            set_score_meta_copyist,
            set_score_meta_publisher,
            set_score_meta_editor,
            set_score_meta_copyright,
            set_score_meta_work_number,
            set_score_meta_composition_year,
            set_score_meta_composer_dates,
            set_score_meta_other_information
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
