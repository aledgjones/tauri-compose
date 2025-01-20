use skia_safe::Point;
use tauri::{ipc::Response, State};

use crate::renderer::Renderer;
use crate::score::components::measurements::PaddingSpaces;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::components::units::Converter;
use crate::score::layouts::LayoutType;
use crate::utils::UniqueId;
use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn render(state: State<'_, AppState>, flow_id: UniqueId) -> Result<Response, ()> {
    // ** TODO: pass these into the function
    let mm: f32 = 3.77;
    let layout_type = LayoutType::FullScore;
    // **

    let state = state.lock().unwrap();

    let engrave = &state.score.engraves.by_layout_type[&layout_type];
    let converter = Converter::new(mm, engrave.space);

    let vertical_spacing = state.measure_vertical_spacing(flow_id, engrave);

    let frame_padding = engrave.frame_padding.to_spaces(&converter);

    let height =
        converter.spaces_to_px(frame_padding.top + vertical_spacing.height + frame_padding.bottom);
    let width = converter.spaces_to_px(frame_padding.left + 50.0 + frame_padding.right);

    let mut renderer = Renderer::new(width as i32, height as i32);

    renderer.text(
        "Violin I",
        &Font {
            size: 18.0,
            family: FontFamily::LibreBaskerville,
            justify: Justify::End,
            align: Align::Middle,
            padding: PaddingSpaces {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        },
        Point { x: 100.0, y: 120.0 },
    );

    renderer.line(vec![
        Point { x: 100.0, y: 110.0 },
        Point { x: 100.0, y: 130.0 },
    ]);

    renderer.line(vec![
        Point { x: 90.0, y: 120.0 },
        Point { x: 110.0, y: 120.0 },
    ]);

    renderer.text(
        "Violin II",
        &Font {
            size: 18.0,
            family: FontFamily::LibreBaskerville,
            justify: Justify::End,
            align: Align::Middle,
            padding: PaddingSpaces {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        },
        Point { x: 100.0, y: 180.0 },
    );

    renderer.line(vec![
        Point { x: 100.0, y: 170.0 },
        Point { x: 100.0, y: 190.0 },
    ]);

    renderer.line(vec![
        Point { x: 90.0, y: 180.0 },
        Point { x: 110.0, y: 180.0 },
    ]);

    renderer.text(
        "Viola",
        &Font {
            size: 18.0,
            family: FontFamily::LibreBaskerville,
            justify: Justify::End,
            align: Align::Middle,
            padding: PaddingSpaces {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        },
        Point { x: 100.0, y: 240.0 },
    );

    renderer.line(vec![
        Point { x: 100.0, y: 230.0 },
        Point { x: 100.0, y: 250.0 },
    ]);

    renderer.line(vec![
        Point { x: 90.0, y: 240.0 },
        Point { x: 110.0, y: 240.0 },
    ]);

    Ok(Response::new(renderer.data()))
}
