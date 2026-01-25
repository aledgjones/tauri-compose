use crate::renderer::converter::Converter;
use crate::renderer::DrawInstruction;
use std::error::Error;
use tauri::AppHandle;

pub trait RenderBackend: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        app: &AppHandle,
        width: f32,
        height: f32,
        instructions: &[DrawInstruction],
        converter: &Converter,
        bounds: &(f32, f32, f32, f32),
        pixel_density: f64,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> Result<(), Box<dyn Error>>;

    fn update_bounds(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale: f64,
    ) -> Result<(), Box<dyn Error>>;
}
