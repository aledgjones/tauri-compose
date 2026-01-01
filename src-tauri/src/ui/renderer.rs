use crate::{AppState, RenderState};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct Renderer {
    pub zoom: i32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub bounds: RendererBounds,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RendererBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            zoom: 100,
            pan_x: 50.0,
            pan_y: 50.0,
            bounds: RendererBounds {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_zoom_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    zoom: i32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.renderer.zoom = zoom;
    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_pan_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    x: f32,
    y: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.renderer.pan_x = x;
    state.ui.renderer.pan_y = y;
    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_bounds_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    bounds: RendererBounds,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.renderer.bounds = bounds;
    state.emit(&app, &mut renderer, false);
}
