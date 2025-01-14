mod renderer;
mod score;
mod ui;
mod utils;

use renderer::render;
use score::config::{
    score_config_auto_count_styles_section_set, score_config_auto_count_styles_solo_set,
};
use score::flows::{
    score_flows_assign_player, score_flows_create, score_flows_move, score_flows_remove,
    score_flows_rename, score_flows_unassign_player,
};
use score::instruments::defs::score_instruments_get_all_definitions;
use score::instruments::{
    score_instruments_create, score_instruments_move, score_instruments_remove,
};
use score::meta::{
    score_meta_arranger_set, score_meta_artist_set, score_meta_composer_dates_set,
    score_meta_composer_set, score_meta_composition_year_set, score_meta_copyist_set,
    score_meta_copyright_set, score_meta_dedication_set, score_meta_editor_set,
    score_meta_lyricist_set, score_meta_other_information_set, score_meta_publisher_set,
    score_meta_subtitle_set, score_meta_title_set, score_meta_work_number_set,
};
use score::players::{
    score_players_assign_instrument, score_players_create, score_players_move,
    score_players_remove, score_players_unassign_instrument,
};
use score::Score;
use serde::Serialize;
use std::sync::Mutex;
use tauri::http::Response;
use tauri::{Emitter, Manager, State};
use ui::setup::{
    ui_setup_expanded_insert, ui_setup_expanded_remove, ui_setup_selection_clear,
    ui_setup_selection_set,
};
use ui::{ui_layout_id_set, ui_tab_set, Ui};
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
            score_config_auto_count_styles_solo_set,
            score_config_auto_count_styles_section_set,
            score_flows_create,
            score_flows_rename,
            score_flows_move,
            score_flows_assign_player,
            score_flows_unassign_player,
            score_flows_remove,
            score_instruments_create,
            score_instruments_get_all_definitions,
            score_instruments_move,
            score_instruments_remove,
            score_meta_arranger_set,
            score_meta_copyist_set,
            score_meta_artist_set,
            score_meta_composer_set,
            score_meta_composer_dates_set,
            score_meta_composition_year_set,
            score_meta_copyright_set,
            score_meta_dedication_set,
            score_meta_editor_set,
            score_meta_lyricist_set,
            score_meta_other_information_set,
            score_meta_publisher_set,
            score_meta_subtitle_set,
            score_meta_title_set,
            score_meta_work_number_set,
            score_players_assign_instrument,
            score_players_unassign_instrument,
            score_players_create,
            score_players_move,
            score_players_remove,
            ui_layout_id_set,
            ui_setup_expanded_insert,
            ui_setup_expanded_remove,
            ui_setup_selection_clear,
            ui_setup_selection_set,
            ui_tab_set,
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
