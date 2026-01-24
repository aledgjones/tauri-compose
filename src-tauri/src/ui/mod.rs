pub mod renderer;
pub mod screens;
pub mod sequencer;

use crate::ui::screens::sequence::Sequence;
use crate::ui::screens::setup::Setup;
use crate::ui::screens::write::Write;
use crate::RendererContextState;
use crate::{utils::UniqueId, AppContextState};
use renderer::Renderer;
use sequencer::Sequencer;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.flow_id = Some(flow_id);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_get_selection_coordinates(
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) -> Result<Option<(f32, f32)>, String> {
    let state = state.lock().unwrap();
    let renderer = renderer_state.lock().unwrap();

    if let Some(selection) = &state.ui.sequencer.selection {
        // Find the hit target that matches this selection
        for target in &renderer.hit_targets {
            if target.entry == *selection {
                let zoom_scale = state.ui.renderer.zoom as f32 / 100.0;

                let popover_x = renderer.converter.spaces_to_px(target.popover_x);
                let x = popover_x * zoom_scale + state.ui.renderer.pan_x;

                let popover_y = renderer.converter.spaces_to_px(target.popover_y);
                let y = popover_y * zoom_scale + state.ui.renderer.pan_y;

                return Ok(Some((x, y)));
            }
        }

        Ok(None)
    } else {
        Ok(None)
    }
}
