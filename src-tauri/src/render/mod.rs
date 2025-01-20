mod draw_names;
mod draw_staves;

use crate::renderer::converter::Converter;
use crate::renderer::fonts::Fonts;
use crate::renderer::text_measurer::TextMeasurer;
use crate::score::components::measurements::Point;
use crate::score::layouts::LayoutType;
use crate::utils::UniqueId;
use crate::AppState;
use crate::{parse::measure_instrument_names::measure_instrument_names, renderer::Renderer};
use draw_names::draw_names;
use draw_staves::{draw_staves, STAVE_LINE_WIDTH};
use tauri::{ipc::Response, State};

#[tauri::command(rename_all = "snake_case")]
pub async fn render(state: State<'_, AppState>, flow_id: UniqueId) -> Result<Response, ()> {
    // ** TODO: pass these into the function
    let px_per_mm: f32 = 3.77;
    let layout_type = LayoutType::FullScore;
    // **

    let state = state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let instruments = state.get_instruments(flow);
    let engrave = &state.score.engraves.by_layout_type[&layout_type];

    let fonts = Fonts::new();
    let converter = Converter::new(px_per_mm, engrave.space);
    let text_measurer = TextMeasurer::new(&fonts, &converter);

    let vertical_spacing = state.measure_vertical_spacing(flow, &instruments, engrave);
    let frame_padding = engrave.frame_padding.to_spaces(&engrave.space);
    let max_name_width = measure_instrument_names(&instruments, engrave, &text_measurer);

    let height = frame_padding.top + vertical_spacing.height + frame_padding.bottom;
    let width = frame_padding.left + max_name_width + 50.0 + frame_padding.right;

    let mut renderer = Renderer::new(width, height, fonts, converter);

    draw_names(
        &instruments,
        &Point {
            x: frame_padding.left + max_name_width,
            y: frame_padding.top,
        },
        &vertical_spacing,
        engrave,
        &mut renderer,
    );

    draw_staves(
        flow,
        &instruments,
        &Point {
            x: frame_padding.left + max_name_width,
            y: frame_padding.top,
        },
        50.0,
        &vertical_spacing,
        &mut renderer,
    );

    renderer.line(
        STAVE_LINE_WIDTH,
        vec![
            Point {
                x: frame_padding.left + max_name_width,
                y: frame_padding.top,
            },
            Point {
                x: frame_padding.left + max_name_width,
                y: frame_padding.top + vertical_spacing.height,
            },
        ],
    );

    Ok(Response::new(renderer.data()))
}
