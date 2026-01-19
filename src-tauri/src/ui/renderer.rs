use crate::{AppState, RenderState, NATIVE_RENDERER};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

#[derive(Clone, Serialize, Deserialize)]
pub struct Renderer {
    pub zoom: i32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub debug_rendering: bool,
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
            pan_x: 100.0,
            pan_y: 100.0,
            debug_rendering: false,
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_zoom_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    zoom: i32,
) -> (i32, f32, f32) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    // Clamp zoom between 25% and 800%
    let clamped_zoom = zoom.clamp(25, 800);

    let old_zoom = state.ui.renderer.zoom as f32 / 100.0;
    let new_zoom = clamped_zoom as f32 / 100.0;

    // Calculate zoom around center of bounds
    let (new_pan_x, new_pan_y) = if let Ok(native_renderer_lock) = NATIVE_RENDERER.lock() {
        if let Some(native) = native_renderer_lock.as_ref() {
            let bounds = native.get_bounds();
            let center_x = bounds.2 / 2.0; // width / 2
            let center_y = bounds.3 / 2.0; // height / 2

            // Calculate world position at center before zoom
            let world_x = (center_x - state.ui.renderer.pan_x) / old_zoom;
            let world_y = (center_y - state.ui.renderer.pan_y) / old_zoom;

            // Calculate new pan to keep same world position at center
            let new_pan_x = center_x - world_x * new_zoom;
            let new_pan_y = center_y - world_y * new_zoom;

            // Constrain the new pan values
            native.constrain_pan(
                new_pan_x,
                new_pan_y,
                renderer.width,
                renderer.height,
                new_zoom,
                &renderer.converter,
            )
        } else {
            (state.ui.renderer.pan_x, state.ui.renderer.pan_y)
        }
    } else {
        (state.ui.renderer.pan_x, state.ui.renderer.pan_y)
    };

    state.ui.renderer.zoom = clamped_zoom;
    state.ui.renderer.pan_x = new_pan_x;
    state.ui.renderer.pan_y = new_pan_y;

    state.emit(&app, &mut renderer, false);

    (clamped_zoom, new_pan_x, new_pan_y)
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_pan_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    x: f32,
    y: f32,
) -> (f32, f32) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    // Get the clamped values from the native renderer
    let (clamped_x, clamped_y) = if let Ok(native_renderer_lock) = NATIVE_RENDERER.lock() {
        if let Some(native) = native_renderer_lock.as_ref() {
            native.constrain_pan(
                x,
                y,
                renderer.width,
                renderer.height,
                state.ui.renderer.zoom as f32 / 100.0,
                &renderer.converter,
            )
        } else {
            (x, y)
        }
    } else {
        (x, y)
    };

    state.ui.renderer.pan_x = clamped_x;
    state.ui.renderer.pan_y = clamped_y;

    state.render(&app, &mut renderer, false);

    // Return clamped values
    (clamped_x, clamped_y)
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_bounds_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    px_per_mm: f32,
    bounds: RendererBounds,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    renderer.px_per_mm = px_per_mm;

    // Directly update native renderer bounds if it exists
    if let Some(window) = app.get_webview_window("main") {
        let scale = window.scale_factor().unwrap_or(1.0);
        if let Ok(mut guard) = crate::NATIVE_RENDERER.lock() {
            if let Some(ref mut native_renderer) = *guard {
                let _ = native_renderer.update_bounds(
                    bounds.x,
                    bounds.y,
                    bounds.width,
                    bounds.height,
                    scale,
                );
            }
        }
    }

    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn ui_renderer_debug_rendering_toggle(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui.renderer.debug_rendering = !state.ui.renderer.debug_rendering;

    state.emit(&app, &mut renderer, true);
}
