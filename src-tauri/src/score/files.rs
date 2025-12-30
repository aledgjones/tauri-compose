use crate::AppState;
use crate::RenderState;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde_json;
use std::fs;
use std::io::{Read, Write};
use tauri::{AppHandle, State};

#[tauri::command(rename_all = "snake_case")]
pub fn save_project_to_file(
    _app: AppHandle,
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    let state = state.lock().unwrap();

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

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    // Load UI state if present
    if let Some(ui_data) = project_data.get("ui") {
        if let Ok(ui) = serde_json::from_value(ui_data.clone()) {
            state.ui = ui;
        }
    }

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

    // Emit the updated state
    state.emit(&app, Some(&mut renderer));
    state.emit_sampler(&app);

    Ok(())
}
