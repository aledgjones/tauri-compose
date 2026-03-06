use crate::AppContextState;
use crate::RendererContextState;
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    auto_count_style: AutoCountStyle,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.score.config.auto_count_styles.solo = auto_count_style;
    state.score.update_counts();
    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_config_auto_count_styles_section_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    auto_count_style: AutoCountStyle,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.score.config.auto_count_styles.section = auto_count_style;
    state.score.update_counts();
    state.emit(&app, &mut renderer, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_count_styles_new_has_arabic_solo() {
        let s = AutoCountStyles::new();
        assert!(matches!(s.solo, AutoCountStyle::Arabic));
    }

    #[test]
    fn auto_count_styles_new_has_roman_section() {
        let s = AutoCountStyles::new();
        assert!(matches!(s.section, AutoCountStyle::Roman));
    }

    #[test]
    fn config_new_constructs_successfully() {
        let c = Config::new();
        assert!(matches!(c.auto_count_styles.solo, AutoCountStyle::Arabic));
        assert!(matches!(c.auto_count_styles.section, AutoCountStyle::Roman));
    }
}
