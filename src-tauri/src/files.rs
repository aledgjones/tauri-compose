use crate::migrations::MIGRATIONS;
use crate::ui::sequencer::Tool;
use crate::AppState;
use crate::RenderState;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use semver::Version;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use tauri::menu::{MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

const MAX_RECENT_FILES: usize = 10;

fn migrate_project_data(mut data: Value) -> Result<Value, String> {
    let file_version = data
        .get("score")
        .and_then(|s| s.get("meta"))
        .and_then(|m| m.get("app_version"))
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0");

    let file_version =
        Version::parse(file_version).map_err(|e| format!("Failed to parse version: {}", e))?;

    for (patch, migrate_fn) in MIGRATIONS.iter() {
        let patch_version = Version::parse(patch)
            .map_err(|e| format!("Failed to parse migration version: {}", e))?;

        if file_version < patch_version {
            data = migrate_fn(data)?;
        }
    }

    // update the meta app_version to current version
    if let Some(score) = data.get_mut("score") {
        if let Some(meta) = score.get_mut("meta") {
            meta["app_version"] = Value::String(env!("CARGO_PKG_VERSION").to_string());
        }
    }

    Ok(data)
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_project_to_file(
    app: AppHandle,
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    // Create the project data structure from the current app state
    let project_data = serde_json::json!({
        "ui": &state.ui,
        "score": &state.score,
        "sampler": &state.sampler,
        "next_id": &state.next_id
    });

    // Serialize to minified JSON
    let json_string = serde_json::to_string(&project_data)
        .map_err(|e| format!("Failed to serialize project data: {}", e))?;

    // Compress with gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(json_string.as_bytes())
        .map_err(|e| format!("Failed to compress data: {}", e))?;
    let compressed_data = encoder
        .finish()
        .map_err(|e| format!("Failed to finish compression: {}", e))?;

    // Write compressed data to file
    fs::write(&file_path, compressed_data).map_err(|e| format!("Failed to write file: {}", e))?;

    // Update the current file path
    state.current_file_path = Some(file_path);

    // Emit event to trigger menu refresh
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn load_project_from_file(
    app: AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    file_path: String,
) -> Result<(), String> {
    // Read the file
    let file_data = fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Decompress gzip data
    let mut decoder = GzDecoder::new(&file_data[..]);
    let mut json_string = String::new();
    decoder
        .read_to_string(&mut json_string)
        .map_err(|e| format!("Failed to decompress file: {}", e))?;

    // Parse the JSON
    let project_data: serde_json::Value = serde_json::from_str(&json_string)
        .map_err(|e| format!("Failed to parse project data: {}", e))?;

    // Apply migrations
    let project_data = migrate_project_data(project_data)?;

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    // Load UI state if present
    if let Some(ui_data) = project_data.get("ui") {
        if let Ok(ui) = serde_json::from_value(ui_data.clone()) {
            state.ui = ui;
        }
    }

    // Reset ephemral state
    state.ui.sequencer.tool = Tool::Select;
    state.ui.screens.setup.selection = None;
    state.ui.sequencer.selection = None;

    // Load score state
    if let Some(score_data) = project_data.get("score") {
        if let Ok(score) = serde_json::from_value(score_data.clone()) {
            state.score = score;
        }
    }

    // Load sampler state if present
    if let Some(sampler_data) = project_data.get("sampler") {
        if let Ok(sampler) = serde_json::from_value(sampler_data.clone()) {
            state.sampler = sampler;
        }
    }

    // Load next_id if present
    if let Some(next_id_data) = project_data.get("next_id") {
        if let Ok(next_id) = serde_json::from_value(next_id_data.clone()) {
            state.next_id = next_id;
        }
    }

    // Update the current file path
    state.current_file_path = Some(file_path);

    // Emit the updated state
    state.emit(&app, &mut renderer, true);
    state.emit_sampler(&app);

    // Emit event to trigger menu refresh
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Get the current file path (if any)
#[tauri::command(rename_all = "snake_case")]
pub fn get_current_file_path(state: State<'_, AppState>) -> Option<String> {
    let state = state.lock().unwrap();
    state.current_file_path.clone()
}

/// Set the current file path and update menu state
#[tauri::command(rename_all = "snake_case")]
pub fn set_current_file_path(
    app: AppHandle,
    state: State<'_, AppState>,
    file_path: Option<String>,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let has_file = file_path.is_some();
    state.current_file_path = file_path.clone();

    // Enable/disable the Save menu item based on whether we have a file path
    if let Some(window) = app.get_webview_window("main") {
        if let Some(menu) = window.menu() {
            if let Some(save_item) = menu.get("save") {
                if let Some(item) = save_item.as_menuitem() {
                    item.set_enabled(has_file).ok();
                }
            }
        }
    }

    // Emit event to update UI
    app.emit("soloc://file/path-changed", file_path).ok();

    Ok(())
}

/// Add a file to recent files list
#[tauri::command(rename_all = "snake_case")]
pub fn add_recent_file(app: AppHandle, file_path: String) -> Result<(), String> {
    let store = app
        .store("store.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    // Get existing recent files
    let mut recent_files: Vec<String> = store
        .get("recent_files")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Remove the file if it already exists (we'll add it to the front)
    recent_files.retain(|f| f != &file_path);

    // Add to front
    recent_files.insert(0, file_path);

    // Keep only the last MAX_RECENT_FILES
    recent_files.truncate(MAX_RECENT_FILES);

    // Save to store
    store.set("recent_files", serde_json::json!(recent_files));

    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    // Emit event so frontend knows to update menu
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Refresh the menu UI (called by frontend after window is ready)
#[tauri::command(rename_all = "snake_case")]
pub fn refresh_menus(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    // Try to update recent files menu (don't fail if menu isn't ready)
    let _ = update_recent_files_menu(&app);

    // Update save menu item based on current file path
    let state = state.lock().unwrap();
    let has_file = state.current_file_path.is_some();

    // Access menu from app handle (not window, since it's application-level on macOS)
    if let Some(menu) = app.menu() {
        if let Some(save_item) = menu.get("save") {
            if let Some(item) = save_item.as_menuitem() {
                item.set_enabled(has_file).ok();
            }
        } else {
            // Save item is in the File submenu, need to find it there
            if let Ok(items) = menu.items() {
                for item in items {
                    if let Some(submenu) = item.as_submenu() {
                        if submenu.text().map(|t| t == "File").unwrap_or(false) {
                            if let Some(save_item) = submenu.get("save") {
                                if let Some(item) = save_item.as_menuitem() {
                                    item.set_enabled(has_file).ok();
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get the list of recent files
#[tauri::command(rename_all = "snake_case")]
pub fn get_recent_files(app: AppHandle) -> Result<Vec<String>, String> {
    let store = app
        .store("store.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    let recent_files: Vec<String> = store
        .get("recent_files")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(recent_files)
}

/// Clear all recent files
#[tauri::command(rename_all = "snake_case")]
pub fn clear_recent_files(app: AppHandle) -> Result<(), String> {
    let store = app
        .store("store.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    store.set("recent_files", serde_json::json!([]));

    store
        .save()
        .map_err(|e| format!("Failed to save store: {}", e))?;

    // Emit event so frontend knows to update menu
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Update the Recent Files menu
pub fn update_recent_files_menu(app: &AppHandle) -> Result<(), String> {
    let menu = app.menu().ok_or("Menu not found")?;
    let items = menu.items().map_err(|e| e.to_string())?;

    // Find the File submenu
    let file_menu = items
        .iter()
        .find_map(|item| {
            item.as_submenu().filter(|submenu| {
                submenu.id().as_ref() == "file_menu"
                    || submenu.text().map(|t| t == "File").unwrap_or(false)
            })
        })
        .ok_or("File menu not found")?;

    // Get the Open Recents submenu
    let recent_items = file_menu
        .get("open_recent")
        .ok_or("Open Recents menu not found")?;

    let recents = recent_items.as_submenu_unchecked();

    // Remove all existing items
    let items = recents.items().map_err(|e| e.to_string())?;
    for item in items {
        recents.remove(&item).ok();
    }

    // Get recent files from store
    let store = app
        .store("store.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;

    let recent_files: Vec<String> = store
        .get("recent_files")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Add new items
    if recent_files.is_empty() {
        // Add a disabled "No Recent Files" item
        let no_items = MenuItem::with_id(
            app,
            "no_recent_files",
            "No Recent Files",
            false,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        recents.append(&no_items).map_err(|e| e.to_string())?;
    } else {
        for (i, file_path) in recent_files.iter().enumerate() {
            // Extract just the filename for display
            let file_name = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);

            // Create a unique ID using the index to avoid issues with special characters in paths
            let menu_id = format!("recent_{}", i);

            let item = MenuItem::with_id(app, &menu_id, file_name, true, None::<&str>)
                .map_err(|e| e.to_string())?;

            recents.append(&item).map_err(|e| e.to_string())?;
        }

        // Add separator and "Clear Recent Files" option
        let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
        recents.append(&separator).map_err(|e| e.to_string())?;

        let clear_item = MenuItem::with_id(
            app,
            "clear_recent_files",
            "Clear Recent Files",
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;

        recents.append(&clear_item).map_err(|e| e.to_string())?;
    }

    Ok(())
}
