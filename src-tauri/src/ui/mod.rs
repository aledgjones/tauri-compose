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
pub struct Screens {
    pub setup: Setup,
    pub write: Write,
    pub sequence: Sequence,
}

impl Screens {
    pub fn new() -> Self {
        Self {
            setup: Setup::new(),
            write: Write::new(),
            sequence: Sequence::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ui {
    tab: Tab,
    pub layout_id: Option<UniqueId>,
    pub flow_id: Option<UniqueId>,
    pub renderer: Renderer,
    pub sequencer: Sequencer,
    pub screens: Screens,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            tab: Tab::Setup,
            layout_id: None,
            flow_id: None,
            renderer: Renderer::new(),
            sequencer: Sequencer::new(),
            screens: Screens::new(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_tab_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    tab: Tab,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.tab = tab;
    state.emit(&app, &mut renderer, false);
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

    state.ui.layout_id = Some(layout_id);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_flow_id_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.flow_id = Some(flow_id);

    state.emit(&app, &mut renderer, true);
}
