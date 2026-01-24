use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

use crate::{utils::UniqueId, AppContextState, RendererContextState};

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum SetupSelectionType {
    Player = 0,
    Flow,
    Layout,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Selection {
    id: UniqueId,
    selection_type: SetupSelectionType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Setup {
    pub selection: Option<Selection>,
    pub expanded: FxHashSet<UniqueId>,
}

impl Setup {
    pub fn new() -> Self {
        Setup {
            selection: None,
            expanded: FxHashSet::default(),
        }
    }
}

impl Default for Setup {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_setup_selection_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    selection_type: SetupSelectionType,
    id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.setup.selection = Some(Selection { id, selection_type });

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_setup_selection_clear(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.setup.selection = None;

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_setup_expanded_insert(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.setup.expanded.insert(id);

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_setup_expanded_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.screens.setup.expanded.remove(&id);

    state.emit(&app, &mut renderer, false);
}
