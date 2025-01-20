mod draw_names;
mod draw_staves;
mod draw_systemic_barline;

use crate::renderer::converter::Converter;
use crate::renderer::fonts::Fonts;
use crate::renderer::text_measurer::TextMeasurer;
use crate::score::components::measurements::Point;
use crate::score::layouts::LayoutType;
use crate::utils::UniqueId;
use crate::AppState;
use crate::{parse::measure_instrument_names::measure_instrument_names, renderer::Renderer};
use draw_names::draw_names;
use draw_staves::draw_staves;
use draw_systemic_barline::draw_systemic_barline;
use tauri::{ipc::Response, State};

#[tauri::command(rename_all = "snake_case")]
pub async fn render(state: State<'_, AppState>, flow_id: UniqueId) -> Result<Response, ()> {
    // ** TODO: get these dynamically
    let px_per_mm: f32 = 3.77;
    let layout_type = LayoutType::FullScore;
    let stave_width = 50.0;
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
    let stave_left = frame_padding.left + max_name_width + engrave.instrument_name_spacing;

    let height = frame_padding.top + vertical_spacing.height + frame_padding.bottom;
    let width = stave_left + stave_width + frame_padding.right;

    let mut renderer = Renderer::new(width, height, fonts, converter);

    draw_names(
        &Point {
            x: frame_padding.left + max_name_width,
            y: frame_padding.top,
        },
        &instruments,
        &vertical_spacing,
        engrave,
        &mut renderer,
    );

    draw_systemic_barline(
        &Point {
            x: stave_left,
            y: frame_padding.top,
        },
        &vertical_spacing,
        engrave,
        &mut renderer,
    );

    draw_staves(
        &Point {
            x: stave_left,
            y: frame_padding.top,
        },
        stave_width,
        flow,
        &instruments,
        &vertical_spacing,
        &mut renderer,
    );

    Ok(Response::new(renderer.data()))
}
