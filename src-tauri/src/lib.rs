pub mod automation;
mod files;
mod instructions;
mod migrations;
mod playback;
mod preferences;
mod renderer;
mod score;
pub mod soloc;
mod ui;
mod utils;

use crate::playback::audio::{
    audio_device_sample_rate_set, audio_device_set, audio_devices_refresh, AudioContext,
};
use crate::playback::sampler::{
    sampler_audition, sampler_load_plugin, sampler_rescan_plugins, sampler_sync_with_instruments,
    SamplerContext,
};
use crate::playback::transport::{
    stop_transport_playback, sync_transport_to_app_state, transport_get_state, transport_pause,
    transport_play, transport_seek, TransportContext, TransportSharedState,
};
use crate::renderer::backends::metal::MetalRenderer;
use files::{
    add_recent_file, clear_recent_files, get_current_file_path, get_recent_files,
    load_project_from_file, refresh_menus, save_project_to_file, set_current_file_path,
    update_recent_files_menu,
};
use instructions::{
    generate_render_instructions, prepare_flow_render_plan, resolve_render_plan_recalculation,
    FlowRenderPreparationDiagnostics, FlowRenderRecalculationRequest,
};
use preferences::{preferences_audition_set, Preferences};
use renderer::hit_test::renderer_hit_test;
use renderer::RendererContext;
use score::config::{
    score_config_auto_count_styles_section_set, score_config_auto_count_styles_solo_set,
};
use score::engraves::{
    score_engraves_base_note_space_set, score_engraves_bracket_single_staves_set,
    score_engraves_bracket_style_set, score_engraves_bracketing_approach_set,
    score_engraves_frame_padding_bottom_set, score_engraves_frame_padding_top_set,
    score_engraves_minimum_note_space_set, score_engraves_minimum_tie_space_set,
    score_engraves_note_space_ratio_set, score_engraves_page_padding_bottom_set,
    score_engraves_page_padding_left_set, score_engraves_page_padding_right_set,
    score_engraves_page_padding_top_set, score_engraves_space_set,
    score_engraves_systemic_barline_single_instrument_system_set,
};
use score::entries::barline::{score_entries_barline_clear, score_entries_barline_create};
use score::entries::clef::{
    score_entries_clef_clear, score_entries_clef_create, score_entries_clef_update,
};
use score::entries::key_signature::{
    score_entries_key_signature_clear, score_entries_key_signature_create,
    score_entries_key_signature_update,
};
use score::entries::tempo::{
    score_entries_tempo_clear, score_entries_tempo_create, score_entries_tempo_update,
};
use score::entries::time_signature::{
    score_entries_time_signature_clear, score_entries_time_signature_create,
    score_entries_time_signature_update,
};
use score::entries::tone::{
    score_entries_tone_accidental_set, score_entries_tone_articulation_set,
    score_entries_tone_create, score_entries_tone_duration_remove, score_entries_tone_duration_set,
    score_entries_tone_pitch_set, score_entries_tone_pitch_shift_by_octaves,
    score_entries_tone_pitch_shift_by_steps, score_entries_tone_set_position_and_duration,
    score_entries_tone_shift, score_entries_tone_slice, score_entries_tone_unify_articulations,
};
use score::entries::tuplet::{score_entries_tuplet_create, score_entries_tuplet_remove};
use score::flows::{
    score_flows_assign_player, score_flows_create, score_flows_move, score_flows_remove,
    score_flows_rename, score_flows_unassign_player,
};
use score::instruments::defs::score_instruments_get_all_definitions;
use score::instruments::{
    score_instruments_assign_plugin, score_instruments_create, score_instruments_move,
    score_instruments_remove, score_instruments_set_mute, score_instruments_set_pan,
    score_instruments_set_solo, score_instruments_set_volume,
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
use std::time::Instant;
use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, State};
use ui::renderer::{
    ui_renderer_bounds_set, ui_renderer_debug_toggle, ui_renderer_pan_set, ui_renderer_zoom_set,
};
use ui::screens::sequence::{
    ui_sequence_bottom_panel_minimized_toggle, ui_sequence_bottom_panel_tab_set,
    ui_sequence_show_active_only_set,
};
use ui::screens::setup::{
    ui_setup_expanded_insert, ui_setup_expanded_remove, ui_setup_selection_clear,
    ui_setup_selection_set,
};
use ui::screens::write::{ui_write_bottom_panel_minimized_toggle, ui_write_bottom_panel_tab_set};
use ui::sequencer::{
    ui_sequencer_coords_set, ui_sequencer_instrument_set, ui_sequencer_selection_clear,
    ui_sequencer_selection_set_clef, ui_sequencer_selection_set_key_signature,
    ui_sequencer_selection_set_rest, ui_sequencer_selection_set_tempo,
    ui_sequencer_selection_set_time_signature, ui_sequencer_selection_set_tone,
    ui_sequencer_selection_set_tuplet, ui_sequencer_snap_set, ui_sequencer_tool_set,
    ui_sequencer_track_set,
};
use ui::{ui_flow_id_set, ui_get_selection_coordinates, ui_layout_id_set, ui_tab_set, Ui};
use utils::UniqueId;

#[derive(Serialize, Deserialize, Clone)]
struct AppStateGeneral {
    ui: Ui,
    score: Score,
    preferences: Preferences,
    current_file_path: Option<String>,
}

#[derive(Clone)]
struct AppContext {
    ui: Ui,
    score: Score,
    preferences: Preferences,
    next_id: UniqueId,
    current_file_path: Option<String>,
}

impl AppContext {
    fn new(app_handle: &AppHandle) -> AppContextState {
        Mutex::new(Self {
            ui: Ui::new(),
            score: Score::new(),
            preferences: Preferences::new(app_handle),
            next_id: 1,
            current_file_path: None,
        })
    }

    pub(crate) fn flow_id_for_track(&self, track_id: UniqueId) -> Option<UniqueId> {
        self.score.flows.by_id.values().find_map(|flow| {
            if flow.master_id == track_id {
                return Some(flow.id);
            }

            flow.staves.by_id.values().find_map(|stave| {
                if stave.master_id == track_id || stave.track_ids.contains(&track_id) {
                    Some(flow.id)
                } else {
                    None
                }
            })
        })
    }

    fn render_with_recalculation(
        &mut self,
        app: &tauri::AppHandle,
        renderer: &mut RendererContext,
        recalculation_request: Option<FlowRenderRecalculationRequest>,
    ) {
        if let Some(recalculation_request) = recalculation_request {
            let layout_id = self
                .ui
                .layout_id
                .or_else(|| self.score.layouts.order.first().copied());
            let flow_id = self
                .ui
                .flow_id
                .or_else(|| self.score.flows.order.first().copied());

            if let (Some(layout_id), Some(flow_id)) = (layout_id, flow_id) {
                let preparation_start = Instant::now();
                if let Some(render_plan) =
                    prepare_flow_render_plan(&self.score, layout_id, flow_id, recalculation_request)
                {
                    let render_plan =
                        resolve_render_plan_recalculation(self, renderer, &render_plan);
                    let preparation_duration = preparation_start.elapsed();
                    let diagnostics = FlowRenderPreparationDiagnostics::from(&render_plan);

                    let _ = app.emit(
                        "render-timing-flow-preparation",
                        preparation_duration.as_micros() as u64,
                    );
                    let _ = app.emit("render-timing-flow-preparation-details", diagnostics);

                    generate_render_instructions(app, self, renderer, &render_plan);
                } else {
                    let _ = app.emit("render-timing-flow-preparation", 0_u64);
                    let _ = app.emit(
                        "render-timing-flow-preparation-details",
                        Option::<FlowRenderPreparationDiagnostics>::None,
                    );
                    let _ = app.emit("render-timing-surface-details", Option::<()>::None);
                    renderer.clear();
                }
            } else {
                let _ = app.emit("render-timing-flow-preparation", 0_u64);
                let _ = app.emit(
                    "render-timing-flow-preparation-details",
                    Option::<FlowRenderPreparationDiagnostics>::None,
                );
                let _ = app.emit("render-timing-surface-details", Option::<()>::None);
                renderer.clear();
            }
        }

        if renderer.backend.is_none() {
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(backend) = MetalRenderer::new(&window) {
                    renderer.set_backend(backend);
                }
            }
        }

        if let Ok((constrained_x, constrained_y)) = renderer.render(
            app,
            self.ui.renderer.pan_x,
            self.ui.renderer.pan_y,
            self.ui.renderer.zoom as f32 / 100.0,
        ) {
            self.ui.renderer.pan_x = constrained_x;
            self.ui.renderer.pan_y = constrained_y;
        }
    }

    pub(crate) fn render_only(&mut self, app: &tauri::AppHandle, renderer: &mut RendererContext) {
        self.render_with_recalculation(app, renderer, None);
    }

    pub(crate) fn emit_general_state(
        &mut self,
        app: &tauri::AppHandle,
        renderer: &mut RendererContext,
        recalculation_request: Option<FlowRenderRecalculationRequest>,
    ) {
        self.render_with_recalculation(app, renderer, recalculation_request);

        let general_state = AppStateGeneral {
            ui: self.ui.clone(),
            score: self.score.clone(),
            current_file_path: self.current_file_path.clone(),
            preferences: self.preferences.clone(),
        };

        if let Err(e) = app.emit("soloc://state/general", &general_state) {
            eprintln!("Failed to emit general state: {:?}", e);
        }
    }

    pub(crate) fn emit_affected_flows(
        &mut self,
        app: &tauri::AppHandle,
        renderer: &mut RendererContext,
        flow_ids: &[UniqueId],
    ) {
        self.emit_general_state(
            app,
            renderer,
            FlowRenderRecalculationRequest::for_flow_ids(flow_ids),
        );
    }

    pub(crate) fn emit_affected_track(
        &mut self,
        app: &tauri::AppHandle,
        renderer: &mut RendererContext,
        track_id: UniqueId,
    ) {
        if let Some(flow_id) = self.flow_id_for_track(track_id) {
            self.emit_affected_flows(app, renderer, &[flow_id]);
        } else {
            self.emit_general_state(
                app,
                renderer,
                Some(FlowRenderRecalculationRequest::full_visible_flows()),
            );
        }
    }

    pub fn generate_id(&mut self) -> UniqueId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub(crate) type AppContextState = Mutex<AppContext>;
pub(crate) type AudioContextState = Mutex<AudioContext>;
pub(crate) type RendererContextState = Mutex<RendererContext>;
pub(crate) type SamplerContextState = Mutex<SamplerContext>;

const APP_NAME: &str = "Solo Compose";

#[tauri::command(rename_all = "snake_case")]
fn get_general_state(state: State<'_, AppContextState>) -> AppStateGeneral {
    let state = state.lock().unwrap();

    AppStateGeneral {
        ui: state.ui.clone(),
        score: state.score.clone(),
        preferences: state.preferences.clone(),
        current_file_path: state.current_file_path.clone(),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn initialize(
    app: AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    audio_state: State<'_, AudioContextState>,
    sampler_state: State<'_, SamplerContextState>,
    transport_shared: State<'_, TransportSharedState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let audio = audio_state.lock().unwrap();
    let sampler = sampler_state.lock().unwrap();

    sync_transport_to_app_state(transport_shared.inner(), &state, 0);
    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
    audio.emit(&app);
    sampler.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
fn reset(
    app: AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    transport_state: State<'_, Mutex<TransportContext>>,
) -> Result<(), String> {
    let transport_shared = stop_transport_playback(transport_state.inner(), audio_state.inner())?;

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui = Ui::new();
    state.score = Score::new();
    state.next_id = 1;
    state.current_file_path = None;

    sync_transport_to_app_state(&transport_shared, &state, 0);

    renderer.clear();

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );

    // Drop state locks before sampler sync
    drop(state);
    drop(renderer);

    // Sync sampler — new score has no instruments, so this unloads everything
    let mut sampler = sampler_state.lock().unwrap();
    let audio = audio_state.lock().unwrap();
    let sample_rate = audio.get_sample_rate();
    drop(audio);

    if let Err(e) = sampler.sync_with_instruments(&[], sample_rate) {
        eprintln!("Failed to sync sampler after reset: {}", e);
    }
    sampler.emit(&app);

    Ok(())
}

pub(crate) fn get_general_state_snapshot(state: State<'_, AppContextState>) -> AppStateGeneral {
    get_general_state(state)
}

pub(crate) fn reset_app_state(
    app: AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    transport_state: State<'_, Mutex<TransportContext>>,
) -> Result<(), String> {
    reset(
        app,
        state,
        renderer_state,
        sampler_state,
        audio_state,
        transport_state,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        let devtools = tauri_plugin_devtools::init();
        builder = builder.plugin(devtools);
    }

    builder
        .setup(|app| {
            let app_handle = app.app_handle();

            app.manage(AppContext::new(app_handle));
            app.manage(RendererContext::new());
            app.manage(AudioContext::new());
            app.manage(SamplerContext::new());

            let (transport_ctx, transport_shared) = TransportContext::new();
            app.manage(transport_ctx);
            app.manage(transport_shared);

            automation::start_server(app.handle().clone());

            // MENU INITIALIZATION
            let separator = PredefinedMenuItem::separator(app)?;

            // App Menu
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
            let quit = PredefinedMenuItem::quit(app, Some(&format!("Quit {}", APP_NAME)))?;
            let preferences = MenuItem::with_id(
                app,
                "preferences",
                "Preferences...",
                true,
                Some("CmdOrCtrl+,"),
            )?;

            let app_menu = Submenu::with_items(
                app,
                APP_NAME,
                true,
                &[
                    &about,
                    &separator,
                    &preferences,
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
            let new = MenuItem::with_id(app, "new", "New", true, Some("CmdOrCtrl+N"))?;
            let open = MenuItem::with_id(app, "open", "Open...", true, Some("CmdOrCtrl+O"))?;
            let open_recent = Submenu::with_id(app, "open_recent", "Open Recent", true)?;
            let save = MenuItem::with_id(app, "save", "Save", false, Some("CmdOrCtrl+S"))?;
            let save_as = MenuItem::with_id(
                app,
                "save_as",
                "Save As...",
                true,
                Some("CmdOrCtrl+Shift+S"),
            )?;

            let file_menu = Submenu::with_id_and_items(
                app,
                "file_menu",
                "File",
                true,
                &[&new, &open, &open_recent, &separator, &save, &save_as],
            )?;

            // Create main menu with app menu first, then File menu
            let menu = Menu::with_items(app, &[&app_menu, &file_menu])?;

            // Set the menu
            app.set_menu(menu)?;

            // Handle menu events
            app.on_menu_event(move |app, event| {
                let event_id = event.id().as_ref();
                match event_id {
                    "new" => {
                        // Trigger new project action
                        let _ = app.emit("soloc://menu/new", ());
                    }
                    "save" => {
                        // Trigger save project action
                        let _ = app.emit("soloc://menu/save", ());
                    }
                    "save_as" => {
                        // Trigger save as project action
                        let _ = app.emit("soloc://menu/save-as", ());
                    }
                    "open" => {
                        // Trigger open project action
                        let _ = app.emit("soloc://menu/open", ());
                    }
                    "preferences" => {
                        // Trigger preferences window
                        let _ = app.emit("soloc://menu/preferences", ());
                    }
                    "clear_recent_files" => {
                        // Clear recent files
                        let _ = clear_recent_files(app.clone());
                    }
                    id if id.starts_with("recent_") => {
                        // Get the index from the menu item ID
                        if let Some(index_str) = id.strip_prefix("recent_") {
                            if let Ok(index) = index_str.parse::<usize>() {
                                // Get the recent files and emit the path at the index
                                if let Ok(recent_files) = get_recent_files(app.clone()) {
                                    if let Some(file_path) = recent_files.get(index) {
                                        let _ =
                                            app.emit("soloc://menu/open-recent", file_path.clone());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            });

            // Initialize the recent files menu
            let _ = update_recent_files_menu(app.app_handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize,
            reset,
            save_project_to_file,
            load_project_from_file,
            get_current_file_path,
            set_current_file_path,
            add_recent_file,
            get_recent_files,
            clear_recent_files,
            refresh_menus,
            audio_device_set,
            audio_device_sample_rate_set,
            audio_devices_refresh,
            sampler_load_plugin,
            sampler_rescan_plugins,
            sampler_sync_with_instruments,
            sampler_audition,
            transport_play,
            transport_pause,
            transport_seek,
            transport_get_state,
            preferences_audition_set,
            score_config_auto_count_styles_solo_set,
            score_config_auto_count_styles_section_set,
            score_engraves_bracketing_approach_set,
            score_engraves_bracket_single_staves_set,
            score_engraves_bracket_style_set,
            score_engraves_systemic_barline_single_instrument_system_set,
            score_engraves_base_note_space_set,
            score_engraves_minimum_note_space_set,
            score_engraves_minimum_tie_space_set,
            score_engraves_note_space_ratio_set,
            score_engraves_space_set,
            score_engraves_frame_padding_top_set,
            score_engraves_frame_padding_bottom_set,
            score_engraves_page_padding_top_set,
            score_engraves_page_padding_right_set,
            score_engraves_page_padding_bottom_set,
            score_engraves_page_padding_left_set,
            score_entries_barline_create,
            score_entries_barline_clear,
            score_entries_clef_create,
            score_entries_clef_clear,
            score_entries_clef_update,
            score_entries_key_signature_create,
            score_entries_key_signature_clear,
            score_entries_key_signature_update,
            score_entries_tempo_create,
            score_entries_tempo_clear,
            score_entries_tempo_update,
            score_entries_time_signature_create,
            score_entries_time_signature_clear,
            score_entries_time_signature_update,
            score_entries_tone_create,
            score_entries_tone_duration_set,
            score_entries_tone_pitch_set,
            score_entries_tone_set_position_and_duration,
            score_entries_tone_pitch_shift_by_octaves,
            score_entries_tone_pitch_shift_by_steps,
            score_entries_tone_accidental_set,
            score_entries_tone_articulation_set,
            score_entries_tone_shift,
            score_entries_tone_unify_articulations,
            score_entries_tone_slice,
            score_entries_tone_duration_remove,
            score_entries_tuplet_create,
            score_entries_tuplet_remove,
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
            score_instruments_assign_plugin,
            score_instruments_set_volume,
            score_instruments_set_pan,
            score_instruments_set_mute,
            score_instruments_set_solo,
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
            renderer_hit_test,
            ui_layout_id_set,
            ui_flow_id_set,
            ui_renderer_bounds_set,
            ui_renderer_debug_toggle,
            ui_renderer_pan_set,
            ui_renderer_zoom_set,
            ui_sequencer_coords_set,
            ui_sequencer_instrument_set,
            ui_sequencer_selection_clear,
            ui_sequencer_selection_set_clef,
            ui_sequencer_selection_set_key_signature,
            ui_sequencer_selection_set_rest,
            ui_sequencer_selection_set_tempo,
            ui_sequencer_selection_set_time_signature,
            ui_sequencer_selection_set_tone,
            ui_sequencer_selection_set_tuplet,
            ui_sequencer_snap_set,
            ui_sequencer_tool_set,
            ui_sequencer_track_set,
            ui_setup_expanded_insert,
            ui_setup_expanded_remove,
            ui_setup_selection_clear,
            ui_setup_selection_set,
            ui_tab_set,
            ui_get_selection_coordinates,
            ui_write_bottom_panel_minimized_toggle,
            ui_write_bottom_panel_tab_set,
            ui_sequence_bottom_panel_minimized_toggle,
            ui_sequence_bottom_panel_tab_set,
            ui_sequence_show_active_only_set,
            get_general_state
        ])
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
