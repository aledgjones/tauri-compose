use crate::instructions::FlowRenderRecalculationRequest;
use crate::migrations::{MigrationKind, MIGRATIONS};
use crate::playback::sampler::sync_sampler_with_score;
use crate::playback::transport::{
    stop_transport_playback, sync_transport_to_app_state, TransportContext,
};
use crate::score::Score;
use crate::soloc::*;
use crate::ui::sequencer::Tool;
use crate::ui::Ui;
use crate::utils::UniqueId;
use crate::{AppContextState, AudioContextState, RendererContextState, SamplerContextState};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Db;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::menu::{MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Emitter, Manager, State};

const MAX_RECENT_FILES: usize = 10;
const CURRENT_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_TEMP_FILE_ATTEMPTS: usize = 16;

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    ui: Ui,
    score: Score,
    next_id: UniqueId,
}

#[derive(Clone, Copy)]
struct ChunkEntry {
    type_code: u32,
    flags: u16,
    offset: u64,
    length: u64,
}

#[derive(Debug)]
struct ParsedSolocContainer<'a> {
    app_version: String,
    project_chunk_flags: u16,
    project_payload: &'a [u8],
}

fn get_recent_db(app: &AppHandle) -> Result<Db, String> {
    let db_path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("recent_files");
    sled::open(&db_path).map_err(|e| e.to_string())
}

fn get_project_app_version(data: &Value) -> Option<&str> {
    data.get("score")
        .and_then(|s| s.get("meta"))
        .and_then(|m| m.get("app_version"))
        .and_then(|v| v.as_str())
}

fn set_project_app_version(data: &mut Value, app_version: &str) {
    if let Some(score) = data.get_mut("score") {
        if let Some(meta) = score.get_mut("meta") {
            meta["app_version"] = Value::String(app_version.to_string());
        }
    }
}

fn migrate_project_data(mut data: Value, file_version: &str) -> Result<Value, String> {
    let file_version =
        Version::parse(file_version).map_err(|e| format!("Failed to parse version: {}", e))?;

    for migration in MIGRATIONS.iter() {
        match migration.kind() {
            MigrationKind::Shipped(target_version) => {
                let patch_version = Version::parse(target_version)
                    .map_err(|e| format!("Failed to parse migration version: {}", e))?;

                if file_version < patch_version {
                    data = (migration.migrate())(data)?;
                }
            }
            MigrationKind::PendingNext => {
                data = (migration.migrate())(data)?;
            }
        }
    }

    set_project_app_version(&mut data, CURRENT_APP_VERSION);

    Ok(data)
}

fn validate_project_version_match(
    header_app_version: &str,
    project_data: &Value,
) -> Result<(), String> {
    let project_app_version = get_project_app_version(project_data)
        .ok_or("Invalid .soloc version mismatch between header and project metadata".to_string())?;

    if project_app_version != header_app_version {
        return Err(
            "Invalid .soloc version mismatch between header and project metadata".to_string(),
        );
    }

    Ok(())
}

fn build_project_file(state: &crate::AppContext) -> ProjectFile {
    let mut score = state.score.clone();
    score.meta.app_version = CURRENT_APP_VERSION.to_string();

    ProjectFile {
        ui: state.ui.clone(),
        score,
        next_id: state.next_id,
    }
}

fn parse_chunk_entry(file_data: &[u8], offset: usize) -> ChunkEntry {
    let type_code = u32::from_le_bytes(file_data[offset..offset + 4].try_into().unwrap());
    let flags = u16::from_le_bytes(file_data[offset + 4..offset + 6].try_into().unwrap());
    let chunk_offset = u64::from_le_bytes(file_data[offset + 6..offset + 14].try_into().unwrap());
    let length = u64::from_le_bytes(file_data[offset + 14..offset + 22].try_into().unwrap());

    ChunkEntry {
        type_code,
        flags,
        offset: chunk_offset,
        length,
    }
}

fn validate_chunk_bounds(
    entry: ChunkEntry,
    file_len: u64,
    payload_start: u64,
) -> Result<(), String> {
    if entry.flags & SOLOC_RESERVED_CHUNK_FLAGS_MASK != 0 {
        return Err(format!("Unsupported .soloc chunk flags: {}", entry.flags));
    }

    if entry.offset < payload_start {
        return Err("Chunk points outside file bounds".to_string());
    }

    let chunk_end = entry
        .offset
        .checked_add(entry.length)
        .ok_or("Chunk bounds overflow".to_string())?;

    if chunk_end > file_len {
        return Err("Chunk points outside file bounds".to_string());
    }

    Ok(())
}

fn parse_soloc_container(file_data: &[u8]) -> Result<ParsedSolocContainer<'_>, String> {
    let minimum_header_len = SOLOC_MAGIC.len() + 1 + 1 + 2;
    if file_data.len() < minimum_header_len {
        return Err("File too small to be a valid .soloc file".to_string());
    }

    if &file_data[..SOLOC_MAGIC.len()] != SOLOC_MAGIC {
        return Err("Invalid .soloc magic".to_string());
    }

    let container_version = file_data[SOLOC_MAGIC.len()];
    if container_version != SOLOC_CONTAINER_VERSION {
        return Err(format!(
            "Unsupported .soloc container version: {}",
            container_version
        ));
    }

    let app_version_len = file_data[SOLOC_MAGIC.len() + 1] as usize;
    let app_version_start = SOLOC_MAGIC.len() + 2;
    let chunk_count_offset = app_version_start
        .checked_add(app_version_len)
        .ok_or("Truncated .soloc header".to_string())?;
    let header_end = chunk_count_offset
        .checked_add(2)
        .ok_or("Truncated .soloc header".to_string())?;

    if file_data.len() < header_end {
        return Err("Truncated .soloc header".to_string());
    }

    let app_version = std::str::from_utf8(&file_data[app_version_start..chunk_count_offset])
        .map_err(|e| format!("Invalid .soloc app version encoding: {}", e))?
        .to_string();
    Version::parse(&app_version).map_err(|e| format!("Invalid .soloc app version: {}", e))?;
    let chunk_count = u16::from_le_bytes(
        file_data[chunk_count_offset..header_end]
            .try_into()
            .unwrap(),
    ) as usize;
    let directory_len = chunk_count
        .checked_mul(SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE)
        .ok_or("Truncated .soloc chunk directory".to_string())?;
    let directory_end = header_end
        .checked_add(directory_len)
        .ok_or("Truncated .soloc chunk directory".to_string())?;

    if file_data.len() < directory_end {
        return Err("Truncated .soloc chunk directory".to_string());
    }

    let file_len = u64::try_from(file_data.len())
        .map_err(|_| "File too large to be a valid .soloc file".to_string())?;
    let payload_start =
        u64::try_from(directory_end).map_err(|_| "Chunk points outside file bounds".to_string())?;
    let mut project_chunk = None;

    for entry_offset in (header_end..directory_end).step_by(SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE) {
        let entry = parse_chunk_entry(file_data, entry_offset);
        validate_chunk_bounds(entry, file_len, payload_start)?;

        if entry.type_code == SOLOC_PROJECT_JSON_CHUNK_TYPE {
            if project_chunk.is_some() {
                return Err("Duplicate PROJECT_JSON chunk".to_string());
            }

            project_chunk = Some(entry);
        }
    }

    let project_chunk = project_chunk.ok_or("Missing required PROJECT_JSON chunk".to_string())?;
    let project_chunk_end = project_chunk
        .offset
        .checked_add(project_chunk.length)
        .ok_or("Chunk bounds overflow".to_string())?;
    let project_chunk_start = usize::try_from(project_chunk.offset)
        .map_err(|_| "Chunk points outside file bounds".to_string())?;
    let project_chunk_end = usize::try_from(project_chunk_end)
        .map_err(|_| "Chunk points outside file bounds".to_string())?;

    Ok(ParsedSolocContainer {
        app_version,
        project_chunk_flags: project_chunk.flags,
        project_payload: &file_data[project_chunk_start..project_chunk_end],
    })
}

fn build_temp_file_path(destination: &Path, attempt: usize) -> PathBuf {
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project.soloc");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let temp_file_name = format!(
        ".{}.tmp-{}-{}-{}",
        file_name,
        process::id(),
        timestamp,
        attempt
    );

    destination
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(temp_file_name)
}

fn sync_parent_directory(parent: &Path) -> Result<(), String> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|e| format!("Failed to sync destination directory: {}", e))
}

fn write_file_atomically(destination: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));

    for attempt in 0..MAX_TEMP_FILE_ATTEMPTS {
        let temp_path = build_temp_file_path(destination, attempt);
        let open_result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path);

        let mut temp_file = match open_result {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Failed to create temp file: {}", error)),
        };

        if let Err(error) = temp_file.write_all(contents) {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("Failed to write file: {}", error));
        }

        if let Err(error) = temp_file.sync_all() {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("Failed to sync temp file: {}", error));
        }

        drop(temp_file);

        if let Err(error) = fs::rename(&temp_path, destination) {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("Failed to rename temp file into place: {}", error));
        }

        sync_parent_directory(parent)?;
        return Ok(());
    }

    Err("Failed to create a unique temp file for atomic save".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::{MigrationKind, MIGRATIONS};
    use serde_json::json;

    fn test_app_version() -> &'static str {
        CURRENT_APP_VERSION
    }

    fn build_test_json(app_version: &str) -> Vec<u8> {
        format!(
            "{{\"score\":{{\"meta\":{{\"app_version\":\"{}\"}}}},\"ui\":{{}},\"next_id\":1}}",
            app_version
        )
        .into_bytes()
    }

    fn build_test_container() -> Vec<u8> {
        let app_version = test_app_version();
        let json_payload = build_test_json(app_version);
        let compressed_payload = compress_project_payload(&json_payload).unwrap();
        build_soloc_container(app_version, &compressed_payload).unwrap()
    }

    fn header_end(app_version: &str) -> usize {
        SOLOC_MAGIC.len() + 1 + 1 + app_version.len() + 2
    }

    fn flags_offset(app_version: &str) -> usize {
        header_end(app_version) + 4
    }

    fn type_offset(app_version: &str) -> usize {
        header_end(app_version)
    }

    fn payload_offset_offset(app_version: &str) -> usize {
        type_offset(app_version) + 4 + 2
    }

    fn pending_migration_registered() -> bool {
        MIGRATIONS
            .iter()
            .any(|migration| matches!(migration.kind(), MigrationKind::PendingNext))
    }

    #[test]
    fn parse_soloc_container_round_trips_project_payload() {
        let container = build_test_container();
        let app_version = test_app_version();

        let parsed = parse_soloc_container(&container).unwrap();
        let decompressed =
            decompress_project_payload(parsed.project_payload, parsed.project_chunk_flags).unwrap();

        assert_eq!(parsed.app_version, app_version);
        assert_eq!(
            std::str::from_utf8(&decompressed).unwrap(),
            String::from_utf8(build_test_json(app_version)).unwrap()
        );
    }

    #[test]
    fn parse_soloc_container_rejects_invalid_magic() {
        let mut container = build_test_container();
        container[0] = b'X';

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Invalid .soloc magic");
    }

    #[test]
    fn parse_soloc_container_rejects_missing_project_chunk() {
        let mut container = build_test_container();
        let type_offset = type_offset(test_app_version());
        container[type_offset..type_offset + 4]
            .copy_from_slice(&u32::from_le_bytes(*b"META").to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Missing required PROJECT_JSON chunk");
    }

    #[test]
    fn parse_soloc_container_rejects_unsupported_flags() {
        let mut container = build_test_container();
        let flags_offset = flags_offset(test_app_version());
        container[flags_offset..flags_offset + 2].copy_from_slice(&2_u16.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Unsupported .soloc chunk flags: 2");
    }

    #[test]
    fn parse_soloc_container_rejects_out_of_bounds_chunk() {
        let mut container = build_test_container();
        let payload_offset_offset = payload_offset_offset(test_app_version());
        container[payload_offset_offset..payload_offset_offset + 8]
            .copy_from_slice(&u64::MAX.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Chunk bounds overflow");
    }

    #[test]
    fn parse_soloc_container_rejects_duplicate_project_chunk() {
        let app_version = test_app_version();
        let mut container = build_test_container();
        let header_end = header_end(app_version);
        let original_entry =
            container[header_end..header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE].to_vec();
        let chunk_count_offset = header_end - 2;
        container[chunk_count_offset..header_end].copy_from_slice(&2_u16.to_le_bytes());
        container.splice(
            header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE
                ..header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE,
            original_entry,
        );

        let new_payload_offset =
            u64::try_from(header_end + (SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE * 2)).unwrap();
        let first_offset = header_end + 6;
        let second_offset = header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE + 6;
        container[first_offset..first_offset + 8]
            .copy_from_slice(&new_payload_offset.to_le_bytes());
        container[second_offset..second_offset + 8]
            .copy_from_slice(&new_payload_offset.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Duplicate PROJECT_JSON chunk");
    }

    #[test]
    fn parse_soloc_container_rejects_chunk_pointing_into_header() {
        let mut container = build_test_container();
        let payload_offset_offset = payload_offset_offset(test_app_version());
        container[payload_offset_offset..payload_offset_offset + 8]
            .copy_from_slice(&0_u64.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Chunk points outside file bounds");
    }

    #[test]
    fn validate_project_version_match_rejects_mismatch() {
        let project_data = json!({
            "score": {
                "meta": {
                    "app_version": "0.0.1"
                }
            }
        });

        let error = validate_project_version_match(test_app_version(), &project_data).unwrap_err();

        assert_eq!(
            error,
            "Invalid .soloc version mismatch between header and project metadata"
        );
    }

    #[test]
    fn migrate_project_data_applies_shipped_migrations_for_older_files() {
        let migrated = migrate_project_data(
            json!({
                "score": {
                    "meta": {
                        "app_version": "1.24.0"
                    },
                    "tracks": {
                        "by_id": {
                            "1": {
                                "entries": {
                                    "by_id": {
                                        "10": {
                                            "type": "Tempo",
                                            "text": null
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "ui": {},
                "next_id": 1
            }),
            "1.24.0",
        )
        .unwrap();

        assert_eq!(
            migrated["score"]["tracks"]["by_id"]["1"]["entries"]["by_id"]["10"]["text"],
            ""
        );
        assert_eq!(
            migrated["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
    }

    #[test]
    fn migrate_project_data_applies_pending_migration_in_development_state() {
        if !pending_migration_registered() {
            return;
        }

        let migrated = migrate_project_data(
            json!({
                "score": {
                    "meta": {
                        "app_version": test_app_version()
                    }
                },
                "ui": {},
                "next_id": 1
            }),
            test_app_version(),
        )
        .unwrap();

        assert_eq!(
            migrated["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
    }

    #[test]
    fn migrate_project_data_handles_mixed_shipped_and_pending_migrations() {
        if !pending_migration_registered() {
            return;
        }

        let migrated = migrate_project_data(
            json!({
                "score": {
                    "meta": {
                        "app_version": "1.24.0"
                    },
                    "tracks": {
                        "by_id": {
                            "1": {
                                "entries": {
                                    "by_id": {
                                        "10": {
                                            "type": "Tempo",
                                            "text": null
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "ui": {},
                "next_id": 1
            }),
            "1.24.0",
        )
        .unwrap();

        assert_eq!(
            migrated["score"]["tracks"]["by_id"]["1"]["entries"]["by_id"]["10"]["text"],
            ""
        );
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_project_to_file(
    app: AppHandle,
    state: State<'_, AppContextState>,
    file_path: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    state.score.meta.app_version = CURRENT_APP_VERSION.to_string();

    let project_file = build_project_file(&state);
    let json_data = serde_json::to_vec(&project_file)
        .map_err(|e| format!("Failed to serialize project data: {}", e))?;
    let compressed_data = compress_project_payload(&json_data)?;
    let container = build_soloc_container(CURRENT_APP_VERSION, &compressed_data)?;
    write_file_atomically(Path::new(&file_path), &container)?;

    // Update the current file path
    state.current_file_path = Some(file_path);

    // Emit event to trigger menu refresh
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)] // Tauri command wiring requires separate managed state parameters here.
pub fn load_project_from_file(
    app: AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    transport_state: State<'_, Mutex<TransportContext>>,
    file_path: String,
) -> Result<(), String> {
    let transport_shared = stop_transport_playback(transport_state.inner())?;

    let file_data = fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let parsed_container = parse_soloc_container(&file_data)?;
    let json_data = decompress_project_payload(
        parsed_container.project_payload,
        parsed_container.project_chunk_flags,
    )?;
    let project_data: Value = serde_json::from_slice(&json_data)
        .map_err(|e| format!("Failed to parse project JSON: {}", e))?;
    validate_project_version_match(&parsed_container.app_version, &project_data)?;

    let project_data = migrate_project_data(project_data, &parsed_container.app_version)
        .map_err(|e| format!("Failed to migrate project data: {}", e))?;
    let mut project_file: ProjectFile = serde_json::from_value(project_data)
        .map_err(|e| format!("Failed to deserialize project file: {}", e))?;
    project_file.score.meta.app_version = CURRENT_APP_VERSION.to_string();

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui = project_file.ui;

    // Reset ephemral state
    state.ui.sequencer.tool = Tool::Select;
    state.ui.screens.setup.selection = None;
    state.ui.sequencer.selection = None;

    state.score = project_file.score;
    state.next_id = project_file.next_id;

    // Update the current file path
    state.current_file_path = Some(file_path);
    sync_transport_to_app_state(&transport_shared, &state, 0);

    let score = state.score.clone();

    // Emit the updated state
    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );

    // Drop state locks before potentially slow sampler sync
    drop(state);
    drop(renderer);

    sync_sampler_with_score(
        &app,
        &score,
        sampler_state.inner(),
        audio_state.inner(),
        "after load",
    );

    // Emit event to trigger menu refresh
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Get the current file path (if any)
#[tauri::command(rename_all = "snake_case")]
pub fn get_current_file_path(state: State<'_, AppContextState>) -> Option<String> {
    let state = state.lock().unwrap();
    state.current_file_path.clone()
}

/// Set the current file path and update menu state
#[tauri::command(rename_all = "snake_case")]
pub fn set_current_file_path(
    app: AppHandle,
    state: State<'_, AppContextState>,
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
    let db = get_recent_db(&app)?;

    // Get existing recent files
    let mut recent_files: Vec<String> = db
        .get(b"recent_files")
        .map_err(|e| e.to_string())?
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();

    // Remove the file if it already exists (we'll add it to the front)
    recent_files.retain(|f| f != &file_path);

    // Add to front
    recent_files.insert(0, file_path);

    // Keep only the last MAX_RECENT_FILES
    recent_files.truncate(MAX_RECENT_FILES);

    // Save to DB
    let data = serde_json::to_vec(&recent_files).map_err(|e| e.to_string())?;
    db.insert(b"recent_files", data)
        .map_err(|e| e.to_string())?;
    db.flush().map_err(|e| e.to_string())?;

    // Emit event so frontend knows to update menu
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Refresh the menu UI (called by frontend after window is ready)
#[tauri::command(rename_all = "snake_case")]
pub fn refresh_menus(app: AppHandle, state: State<'_, AppContextState>) -> Result<(), String> {
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
    let db = get_recent_db(&app)?;

    let recent_files: Vec<String> = db
        .get(b"recent_files")
        .map_err(|e| e.to_string())?
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();

    Ok(recent_files)
}

/// Clear all recent files
#[tauri::command(rename_all = "snake_case")]
pub fn clear_recent_files(app: AppHandle) -> Result<(), String> {
    let db = get_recent_db(&app)?;

    let empty: Vec<String> = vec![];
    let data = serde_json::to_vec(&empty).map_err(|e| e.to_string())?;
    db.insert(b"recent_files", data)
        .map_err(|e| e.to_string())?;
    db.flush().map_err(|e| e.to_string())?;

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

    // Get recent files from DB
    let db = get_recent_db(app)?;
    let recent_files: Vec<String> = db
        .get(b"recent_files")
        .map_err(|e| e.to_string())?
        .and_then(|v| serde_json::from_slice(&v).ok())
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
