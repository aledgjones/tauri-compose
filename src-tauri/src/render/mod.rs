mod debug;
mod draw;
mod parse;

use crate::renderer::converter::Converter;
use crate::renderer::fonts::Fonts;
use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::utils::UniqueId;
use crate::AppState;
use debug::debug_tone_drawing::debug_tone_drawing;
use draw::draw_brackets::draw_brackets;
use draw::draw_staves::draw_staves;
use draw::draw_sub_brackets::draw_sub_brackets;
use draw::draw_systemic_barline::draw_systemic_barline;
use draw::{draw_braces::draw_braces, draw_names::draw_names};
use parse::get_bars::get_bars;
use parse::get_max_brackets_width::get_max_brackets_width;
use parse::get_max_instrument_name_width::get_max_instrument_name_width;
use parse::get_tone_offsets::get_tone_offsets;
use parse::get_vertical_spacing::get_vertical_spacing;
use parse::get_vertical_spans::get_vertical_spans;
use parse::get_written_durations::get_written_durations;
use tauri::{ipc::Response, State};

#[tauri::command(rename_all = "snake_case")]
pub fn render(
    state: State<'_, AppState>,
    layout_id: UniqueId,
    flow_id: UniqueId,
    px_per_mm: f32,
) -> Response {
    let state = state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let layout = &state.score.layouts.by_id[&layout_id];
    let instruments = state.get_instruments(layout, flow);
    let engrave = &state.score.engraves.by_layout_type[&layout.layout_type];

    let fonts = Fonts::new();
    let converter = Converter::new(px_per_mm, engrave.space);
    let text_measurer = TextMeasurer::new(&fonts, &converter);

    let vertical_spacing = get_vertical_spacing(flow, &instruments, engrave);
    let vertical_spans = get_vertical_spans(&instruments, engrave);

    let frame_padding = engrave.frame_padding.to_spaces(&engrave.space);
    let max_name_width = get_max_instrument_name_width(&instruments, engrave, &text_measurer);
    let max_bracket_width = get_max_brackets_width(&vertical_spacing, &vertical_spans, engrave);

    let bars = get_bars(flow, &state.score.tracks);
    let tone_offsets = get_tone_offsets(flow, &instruments, &state.score.tracks);
    let written_durations = get_written_durations(flow, &instruments, &state.score.tracks, &bars);

    // ** TODO: this is just fixed to 1 tick = 0.01 space. Update.
    let stave_width: f32 = flow.length as f32 * 0.02;
    // **

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

    debug_tone_drawing(
        &Point {
            x: stave_left,
            y: frame_padding.top,
        },
        flow,
        &instruments,
        &written_durations,
        &vertical_spacing,
        &tone_offsets,
        &mut renderer,
    );

    Response::new(renderer.data())
}
