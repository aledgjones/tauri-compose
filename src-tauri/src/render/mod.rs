mod draw;
mod parse;

use crate::renderer::converter::Converter;
use crate::renderer::fonts::Fonts;
use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::layouts::LayoutType;
use crate::utils::UniqueId;
use crate::AppState;
use draw::draw_brackets::draw_brackets;
use draw::draw_staves::draw_staves;
use draw::draw_sub_brackets::draw_sub_brackets;
use draw::draw_systemic_barline::draw_systemic_barline;
use draw::{draw_braces::draw_braces, draw_names::draw_names};
use parse::get_max_brackets_width::get_max_brackets_width;
use parse::get_max_instrument_name_width::get_max_instrument_name_width;
use parse::get_vertical_spacing::get_vertical_spacing;
use parse::get_vertical_spans::get_vertical_spans;
use tauri::{ipc::Response, State};

#[tauri::command(rename_all = "snake_case")]
pub async fn render(
    state: State<'_, AppState>,
    flow_id: UniqueId,
    px_per_mm: f32,
) -> Result<Response, ()> {
    // ** TODO: get these dynamically
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

    let vertical_spacing = get_vertical_spacing(flow, &instruments, engrave);
    let vertical_spans = get_vertical_spans(&instruments, engrave);

    let frame_padding = engrave.frame_padding.to_spaces(&engrave.space);
    let max_name_width = get_max_instrument_name_width(&instruments, engrave, &text_measurer);
    let max_bracket_width = get_max_brackets_width(&vertical_spacing, &vertical_spans, engrave);

    let stave_left =
        frame_padding.left + max_name_width + engrave.instrument_name_spacing + max_bracket_width;
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

    draw_braces(
        &Point {
            x: stave_left,
            y: frame_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        &mut renderer,
    );

    draw_brackets(
        &Point {
            x: stave_left,
            y: frame_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        engrave,
        &mut renderer,
    );

    draw_sub_brackets(
        &Point {
            x: stave_left,
            y: frame_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
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
