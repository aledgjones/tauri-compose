use crate::{AppContextState, RendererContextState};
use serde::{Deserialize, Serialize};
use sled::Db;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Preferences {
    pub audition: bool,
    #[serde(skip)]
    db: Option<Db>,
}

impl Preferences {
    pub fn new(app: &AppHandle) -> Self {
        let db_path = app.path().app_config_dir().unwrap().join("preferences");
        let db = sled::open(&db_path).ok();

        let mut prefs = Self { audition: true, db };

        if let Ok(Some(audition)) = prefs.load("audition") {
            prefs.audition = audition;
        }

        prefs
    }

    pub fn load<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, String> {
        if let Some(ref db) = self.db {
            if let Ok(Some(value)) = db.get(key.as_bytes()) {
                let deserialized = serde_json::from_slice(&value).map_err(|e| e.to_string())?;
                Ok(Some(deserialized))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn persist<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        if let Some(ref db) = self.db {
            let value_bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
            db.insert(key.as_bytes(), value_bytes)
                .map_err(|e| e.to_string())?;
            db.flush().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

// App-wide preferences update and emit
#[tauri::command(rename_all = "snake_case")]
pub fn preferences_audition_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    audition: bool,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.preferences.audition = audition;
    state.preferences.persist("audition", &audition)?;

    state.emit(&app, &mut renderer, false);

    Ok(())
}
