use crate::AppState;
use serde_json;
use std::fs;
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

    // Serialize to JSON
    let json_string = serde_json::to_string_pretty(&project_data)
        .map_err(|e| format!("Failed to serialize project data: {}", e))?;

    // Write to file
    fs::write(&file_path, json_string).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn load_project_from_file(
    app: AppHandle,
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    // Read the file
    let json_string =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse the JSON
    let project_data: serde_json::Value = serde_json::from_str(&json_string)
        .map_err(|e| format!("Failed to parse project data: {}", e))?;

    let mut state = state.lock().unwrap();

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
    state.emit(&app);
    state.emit_sampler(&app);

    Ok(())
}
