pub mod renderer;
pub mod sequence;
pub mod sequencer;
pub mod setup;
pub mod write;

use crate::RenderState;
use crate::{utils::UniqueId, AppState};
use renderer::Renderer;
use sequence::Sequence;
use sequencer::Sequencer;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use setup::Setup;
use tauri::State;
use write::Write;

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum Tab {
    Setup = 0,
    Write = 1,
    Engrave = 2,
    Play = 3,
    Print = 4,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ui {
    tab: Tab,
    pub layout_id: UniqueId,
    pub setup: Setup,
    pub write: Write,
    pub sequence: Sequence,
    pub sequencer: Sequencer,
    pub renderer: Renderer,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            tab: Tab::Setup,
            layout_id: 0,
            setup: Setup::new(),
            write: Write::new(),
            sequence: Sequence::new(),
            sequencer: Sequencer::new(),
            renderer: Renderer::new(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_tab_set(app: tauri::AppHandle, state: State<'_, AppState>, tab: Tab) {
    let mut state = state.lock().unwrap();
    state.ui.tab = tab;
    state.emit(&app, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_layout_id_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    layout_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.layout_id = layout_id;
    state.emit(&app, Some(&mut renderer));
}
