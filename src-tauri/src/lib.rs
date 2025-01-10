mod renderer;
mod score;
mod ui;
mod utils;

use renderer::render;
use score::config::{
    set_score_config_auto_count_styles_section, set_score_config_auto_count_styles_solo,
};
use score::instruments::create_instrument;
use score::instruments::defs::get_all_instrument_definitions;
use score::meta::{
    set_score_meta_arranger, set_score_meta_artist, set_score_meta_composer,
    set_score_meta_composer_dates, set_score_meta_composition_year, set_score_meta_copyist,
    set_score_meta_copyright, set_score_meta_dedication, set_score_meta_editor,
    set_score_meta_lyricist, set_score_meta_other_information, set_score_meta_publisher,
    set_score_meta_subtitle, set_score_meta_title, set_score_meta_work_number,
};
use score::players::{assign_instrument_to_player, create_player, reorder_players};
use score::Score;
use serde::Serialize;
use std::sync::Mutex;
use tauri::http::Response;
use tauri::{Emitter, Manager, State};
use ui::setup::{
    clear_ui_setup_selection, insert_ui_setup_expanded, remove_ui_setup_expanded,
    set_ui_setup_selection,
};
use ui::{set_ui_layout_id, set_ui_tab, Ui};
use utils::UniqueId;

#[derive(Serialize, Clone)]
struct AppStateInner {
    ui: Ui,
    score: Score,
    next_id: UniqueId,
}

impl AppStateInner {
    fn new() -> AppState {
        Mutex::new(Self {
            ui: Ui::new(),
            score: Score::new(),
            next_id: 0,
        })
    }

    fn emit(&self, app: &tauri::AppHandle) {
        app.emit("solocompose://state", self.clone()).unwrap();
    }

    pub fn generate_id(&mut self) -> UniqueId {
        let id = self.next_id;
        self.next_id += 1;
        id
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
        .register_asynchronous_uri_scheme_protocol(
            "solocompose",
            move |_ctx, _request, responder| {
                let bytes = render();
                let response = Response::builder()
                    .header("Content-Type", "image/webp")
                    .header("Cache-Control", "no-store")
                    .body(bytes);

                match response {
                    Ok(response) => responder.respond(response),
                    Err(_) => {
                        responder.respond(Response::builder().status(500).body(vec![]).unwrap())
                    }
                }
            },
        )
        .invoke_handler(tauri::generate_handler![
            initialize,
            assign_instrument_to_player,
            create_player,
            create_instrument,
            clear_ui_setup_selection,
            get_all_instrument_definitions,
            insert_ui_setup_expanded,
            remove_ui_setup_expanded,
            reorder_players,
            set_score_config_auto_count_styles_solo,
            set_score_config_auto_count_styles_section,
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
            set_score_meta_other_information,
            set_ui_tab,
            set_ui_layout_id,
            set_ui_setup_selection,
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
