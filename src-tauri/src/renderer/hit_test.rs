use crate::utils::UniqueId;
use crate::RenderState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Clone)]
pub struct HitTestResult {
    pub tone_id: UniqueId,
    pub track_id: UniqueId,
    pub flow_id: UniqueId,
}

#[tauri::command(rename_all = "snake_case")]
pub fn renderer_hit_test(
    renderer_state: State<'_, RenderState>,
    x: f32,
    y: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> Option<HitTestResult> {
    let renderer = renderer_state.lock().unwrap();

    let zoom_scale = zoom / 100.0;

    let render_x = (x - pan_x) / zoom_scale;
    let render_y = (y - pan_y) / zoom_scale;

    let render_x_spaces = renderer.converter.px_to_spaces(render_x);
    let render_y_spaces = renderer.converter.px_to_spaces(render_y);

    for target in &renderer.hit_targets {
        if render_x_spaces >= target.x
            && render_x_spaces <= target.x + target.width
            && render_y_spaces >= target.y
            && render_y_spaces <= target.y + target.height
        {
            return Some(HitTestResult {
                tone_id: target.tone_id,
                track_id: target.track_id,
                flow_id: target.flow_id,
            });
        }
    }

    None
}
