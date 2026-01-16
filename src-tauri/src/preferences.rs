use crate::{AppState, RenderState};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_store::StoreBuilder;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Preferences {
    pub audition: bool,
}

impl Preferences {
    pub async fn new(app: AppHandle) -> Self {
        let store_path = app
            .path()
            .app_config_dir()
            .unwrap()
            .join("preferences.store");

        let store = StoreBuilder::new(&app, store_path).build().unwrap();

        let audition = store
            .get("audition")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Self { audition }
    }
}

// App-wide preferences update and emit
#[tauri::command(rename_all = "snake_case")]
pub async fn preferences_audition_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    audition: bool,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    state.preferences.audition = audition;

    // Persist to store
    let store_path = app
        .path()
        .app_config_dir()
        .unwrap()
        .join("preferences.store");
    let store = StoreBuilder::new(&app, store_path)
        .build()
        .map_err(|e| e.to_string())?;
    store.set("audition", audition);
    store.save().map_err(|e| e.to_string())?;

    // Emit state change
    state.emit(&app, &mut renderer, false);

    Ok(())
}
