pub mod renderer;
pub mod screens;
pub mod sequencer;

use crate::instructions::FlowRenderRecalculationRequest;
use crate::ui::screens::sequence::Sequence;
use crate::ui::screens::setup::Setup;
use crate::ui::screens::write::Write;
use crate::{
    playback::transport::{sync_transport_to_app_state, TransportSharedState},
    RendererContextState,
};
use crate::{utils::UniqueId, AppContextState};
use renderer::Renderer;
use sequencer::Sequencer;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

fn resolve_selection_coordinates(
    selection: Option<&sequencer::Selection>,
    ui_renderer: &renderer::Renderer,
    renderer: &crate::renderer::RendererContext,
) -> Option<(f32, f32)> {
    let selection = selection?;

    for target in &renderer.hit_targets {
        if target.entry == *selection {
            let zoom_scale = ui_renderer.zoom as f32 / 100.0;

            let popover_x = renderer.converter.spaces_to_px(target.popover_x);
            let x = popover_x * zoom_scale + ui_renderer.pan_x;

            let popover_y = renderer.converter.spaces_to_px(target.popover_y);
            let y = popover_y * zoom_scale + ui_renderer.pan_y;

            return Some((x, y));
        }
    }

    None
}

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
    state.emit_general_state(&app, &mut renderer, None);
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

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_flow_id_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.flow_id = Some(flow_id);
    sync_transport_to_app_state(transport_shared.inner(), &state, 0);

    state.emit_general_state(&app, &mut renderer, None);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_get_selection_coordinates(
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) -> Result<Option<(f32, f32)>, String> {
    let state = state.lock().unwrap();
    let renderer = renderer_state.lock().unwrap();

    Ok(resolve_selection_coordinates(
        state.ui.sequencer.selection.as_ref(),
        &state.ui.renderer,
        &renderer,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{HitTarget, RendererContext};
    use crate::ui::sequencer::Selection;

    fn make_renderer_context(hit_targets: Vec<HitTarget>) -> RendererContext {
        let mut renderer = RendererContext::new().into_inner().unwrap();
        renderer.hit_targets = hit_targets;
        renderer
    }

    #[test]
    fn resolves_later_flow_selection_coordinates_after_zoom_and_pan() {
        let selection = Selection::Tempo {
            flow_id: 20,
            tempo_id: 9,
            tick: 128,
        };
        let ui_renderer = renderer::Renderer {
            zoom: 200,
            pan_x: 30.0,
            pan_y: -10.0,
            debug: false,
        };
        let renderer = make_renderer_context(vec![HitTarget {
            entry: selection.clone(),
            x: 25.0,
            y: 12.0,
            width: 1.0,
            height: 1.0,
            popover_x: 25.5,
            popover_y: 10.25,
        }]);

        let coordinates = resolve_selection_coordinates(Some(&selection), &ui_renderer, &renderer);

        assert_eq!(coordinates, Some((81.0, 10.5)));
    }

    #[test]
    fn resolves_boundary_adjacent_selection_against_the_exact_matching_hit_target() {
        let selected_target = Selection::Rest {
            flow_id: 20,
            instrument_id: 2,
            stave_id: 3,
            track_id: 4,
            tick: 64,
        };
        let ui_renderer = renderer::Renderer {
            zoom: 150,
            pan_x: -5.0,
            pan_y: 8.0,
            debug: false,
        };
        let renderer = make_renderer_context(vec![
            HitTarget {
                entry: Selection::Rest {
                    flow_id: 10,
                    instrument_id: 2,
                    stave_id: 3,
                    track_id: 4,
                    tick: 64,
                },
                x: 24.8,
                y: 6.0,
                width: 1.0,
                height: 1.0,
                popover_x: 24.9,
                popover_y: 6.25,
            },
            HitTarget {
                entry: selected_target.clone(),
                x: 25.0,
                y: 6.0,
                width: 1.0,
                height: 1.0,
                popover_x: 25.1,
                popover_y: 6.75,
            },
        ]);

        let coordinates =
            resolve_selection_coordinates(Some(&selected_target), &ui_renderer, &renderer);

        assert_eq!(coordinates, Some((32.65, 18.125)));
    }
}
