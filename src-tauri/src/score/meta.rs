use crate::{utils::now, AppState};
use serde::{Deserialize, Serialize};
use tauri::State;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    app_version: String,
    title: String,
    subtitle: String,
    composer: String,
    arranger: String,
    lyricist: String,
    copyright: String,
    created: u128,
}

impl Meta {
    pub fn new() -> Self {
        Meta {
            app_version: String::from(VERSION),
            title: String::from(""),
            subtitle: String::from(""),
            composer: String::from(""),
            arranger: String::from(""),
            lyricist: String::from(""),
            copyright: String::from(""),
            created: now(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_score_meta_title(app: tauri::AppHandle, state: State<'_, AppState>, title: String) {
    let mut state = state.lock().unwrap();
    state.score.meta.title = title;
    state.emit(&app);
}
