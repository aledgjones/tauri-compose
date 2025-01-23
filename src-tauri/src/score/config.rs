use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum AutoCountStyle {
    Arabic = 0,
    Roman = 1,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AutoCountStyles {
    pub solo: AutoCountStyle,
    pub section: AutoCountStyle,
}

impl AutoCountStyles {
    pub fn new() -> AutoCountStyles {
        AutoCountStyles {
            solo: AutoCountStyle::Arabic,
            section: AutoCountStyle::Roman,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub auto_count_styles: AutoCountStyles,
}

impl Config {
    pub fn new() -> Config {
        Config {
            auto_count_styles: AutoCountStyles::new(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_config_auto_count_styles_solo_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    auto_count_style: AutoCountStyle,
) {
    let mut state = state.lock().unwrap();
    state.score.config.auto_count_styles.solo = auto_count_style;
    state.score.update_counts();
    state.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_config_auto_count_styles_section_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    auto_count_style: AutoCountStyle,
) {
    let mut state = state.lock().unwrap();
    state.score.config.auto_count_styles.section = auto_count_style;
    state.score.update_counts();
    state.emit(&app);
}
