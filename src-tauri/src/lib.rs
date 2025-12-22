mod render;
mod renderer;
mod sampler;
mod score;
mod ui;
mod utils;

use render::render;
use sampler::Sampler;
use score::config::{
    score_config_auto_count_styles_section_set, score_config_auto_count_styles_solo_set,
};
use score::engraves::{
    score_engraves_bracket_single_staves_set, score_engraves_bracket_style_set,
    score_engraves_bracketing_approach_set,
    score_engraves_systemic_barline_single_instrument_system_set,
};
use score::entries::barline::{score_entries_barline_clear, score_entries_barline_create};
use score::entries::clef::{score_entries_clef_clear, score_entries_clef_create};
use score::entries::key_signature::{
    score_entries_key_signature_clear, score_entries_key_signature_create,
};
use score::entries::time_signature::{
    score_entries_time_signature_clear, score_entries_time_signature_create,
};
use score::entries::tone::{
    score_entries_tone_create, score_entries_tone_duration_remove, score_entries_tone_duration_set,
};
use score::files::{load_project_from_file, save_project_to_file};
use score::flows::{
    score_flows_assign_player, score_flows_create, score_flows_move, score_flows_remove,
    score_flows_rename, score_flows_unassign_player,
};
use score::instruments::defs::{
    score_instruments_get_all_definitions, score_instruments_get_patches,
};
use score::instruments::{
    score_instruments_create, score_instruments_move, score_instruments_remove,
    score_instruments_set_mute, score_instruments_set_solo, score_instruments_set_volume,
};
use score::layouts::{
    score_layouts_assign_flow, score_layouts_assign_player, score_layouts_create,
    score_layouts_move, score_layouts_remove, score_layouts_rename, score_layouts_unassign_flow,
    score_layouts_unassign_player,
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
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager, State};
use ui::sequencer::{
    ui_sequencer_coords_set, ui_sequencer_instrument_set, ui_sequencer_snap_set,
    ui_sequencer_tool_set, ui_sequencer_track_set,
};
use ui::setup::{
    ui_setup_expanded_insert, ui_setup_expanded_remove, ui_setup_selection_clear,
    ui_setup_selection_set,
};
use ui::write::{ui_write_bottom_panel_minimized_toggle, ui_write_bottom_panel_tab_set};
use ui::{ui_layout_id_set, ui_tab_set, Ui};
use utils::UniqueId;

#[derive(Serialize, Deserialize, Clone)]
struct AppStateGeneral {
    ui: Ui,
    score: Score,
    next_id: UniqueId,
}

#[derive(Serialize, Deserialize, Clone)]
struct AppStateInner {
    ui: Ui,
    score: Score,
    sampler: Sampler,
    next_id: UniqueId,
}

impl AppStateInner {
    fn new() -> AppState {
        Mutex::new(Self {
            ui: Ui::new(),
            score: Score::new(),
            sampler: Sampler::default(),
            next_id: 1,
        })
    }

    fn emit(&self, app: &tauri::AppHandle) {
        let general_state = AppStateGeneral {
            ui: self.ui.clone(),
            score: self.score.clone(),
            next_id: self.next_id,
        };
        app.emit("soloc://state/general", general_state).unwrap();
    }

    fn emit_sampler(&self, app: &tauri::AppHandle) {
        app.emit("soloc://state/sampler", &self.sampler).unwrap();
    }

    pub fn generate_id(&mut self) -> UniqueId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

type AppState = Mutex<AppStateInner>;

const APP_NAME: &str = "Solo Compose";

#[tauri::command]
fn initialize(app: tauri::AppHandle, state: State<'_, AppState>) {
    let state = state.lock().unwrap();
    state.emit(&app);
    state.emit_sampler(&app);
}

#[tauri::command(rename_all = "snake_case")]
fn reset(app: tauri::AppHandle, state: State<'_, AppState>) {
    let mut state = state.lock().unwrap();
    state.ui = Ui::new();
    state.score = Score::new();
    state.sampler = Sampler::default();
    state.next_id = 1;
    state.emit(&app);
    state.emit_sampler(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init();
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .setup(|app| {
            let separator = PredefinedMenuItem::separator(app)?;

            // App menu
            let about = PredefinedMenuItem::about(
                app,
                Some(&format!("About {}", APP_NAME)),
                Some(AboutMetadata {
                    name: Some(APP_NAME.to_string()),
                    version: Some(app.package_info().version.to_string()),
                    authors: Some(vec!["Aled Jones <aledjones.viola@gmail.com>".to_string()]),
                    copyright: Some("© 2025".to_string()),
                    ..Default::default()
                }),
            )?;
            let services = PredefinedMenuItem::services(app, None)?;
            let hide = PredefinedMenuItem::hide(app, Some(&format!("Hide {}", APP_NAME)))?;
            let hide_others = PredefinedMenuItem::hide_others(app, None)?;
            let show_all = PredefinedMenuItem::show_all(app, None)?;
            let quit = PredefinedMenuItem::quit(app, None)?;

            let app_menu = Submenu::with_items(
                app,
                APP_NAME,
                true,
                &[
                    &about,
                    &separator,
                    &services,
                    &separator,
                    &hide,
                    &hide_others,
                    &show_all,
                    &separator,
                    &quit,
                ],
            )?;

            // File Menu
            let new = MenuItem::with_id(app, "new", "New", true, None::<&str>)?;
            let open = MenuItem::with_id(app, "open", "Open...", true, None::<&str>)?;
            let save_as = MenuItem::with_id(app, "save_as", "Save As...", true, None::<&str>)?;

            let file_menu =
                Submenu::with_items(app, "File", true, &[&new, &open, &separator, &save_as])?;

            // Create main menu with app menu first, then File menu
            let menu = Menu::with_items(app, &[&app_menu, &file_menu])?;

            // Set the menu
            app.set_menu(menu)?;

            // Handle menu events
            app.on_menu_event(move |app, event| {
                match event.id().as_ref() {
                    "new" => {
                        // Trigger new project action
                        let _ = app.emit("soloc://menu/new", ());
                    }
                    "save_as" => {
                        // Trigger save project action
                        let _ = app.emit("soloc://menu/save-as", ());
                    }
                    "open" => {
                        // Trigger open project action
                        let _ = app.emit("soloc://menu/open", ());
                    }
                    _ => {}
                }
            });

            app.manage(AppStateInner::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize,
            reset,
            render,
            save_project_to_file,
            load_project_from_file,
            score_config_auto_count_styles_solo_set,
            score_config_auto_count_styles_section_set,
            score_engraves_bracketing_approach_set,
            score_engraves_bracket_single_staves_set,
            score_engraves_bracket_style_set,
            score_engraves_systemic_barline_single_instrument_system_set,
            score_entries_barline_create,
            score_entries_barline_clear,
            score_entries_clef_create,
            score_entries_clef_clear,
            score_entries_key_signature_create,
            score_entries_key_signature_clear,
            score_entries_time_signature_create,
            score_entries_time_signature_clear,
            score_entries_tone_create,
            score_entries_tone_duration_set,
            score_entries_tone_duration_remove,
            score_flows_create,
            score_flows_rename,
            score_flows_move,
            score_flows_assign_player,
            score_flows_unassign_player,
            score_flows_remove,
            score_instruments_create,
            score_instruments_get_all_definitions,
            score_instruments_get_patches,
            score_instruments_move,
            score_instruments_remove,
            score_instruments_set_mute,
            score_instruments_set_solo,
            score_instruments_set_volume,
            score_layouts_create,
            score_layouts_rename,
            score_layouts_assign_flow,
            score_layouts_unassign_flow,
            score_layouts_assign_player,
            score_layouts_unassign_player,
            score_layouts_move,
            score_layouts_remove,
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
            ui_sequencer_coords_set,
            ui_sequencer_instrument_set,
            ui_sequencer_snap_set,
            ui_sequencer_tool_set,
            ui_sequencer_track_set,
            ui_setup_expanded_insert,
            ui_setup_expanded_remove,
            ui_setup_selection_clear,
            ui_setup_selection_set,
            ui_tab_set,
            ui_write_bottom_panel_minimized_toggle,
            ui_write_bottom_panel_tab_set
        ])
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
