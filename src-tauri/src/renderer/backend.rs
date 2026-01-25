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
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> Result<(f32, f32), Box<dyn Error>>;

    fn get_bounds(&self) -> (f32, f32, f32, f32);

    fn constrain_pan(
        &self,
        pan_x: f32,
        pan_y: f32,
        page_width_spaces: f32,
        page_height_spaces: f32,
        zoom: f32,
        converter: &Converter,
    ) -> (f32, f32);

    fn update_bounds(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale: f64,
    ) -> Result<(), Box<dyn Error>>;
}
