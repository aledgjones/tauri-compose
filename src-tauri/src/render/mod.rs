mod debug;
mod draw;
mod parse;

use crate::renderer::converter::{Converter, Space};
use crate::renderer::fonts::Fonts;
use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::entries::clef::Clef;
use crate::utils::UniqueId;
use crate::AppState;
use draw::draw_barline::draw_barline;
use draw::draw_braces::draw_braces;
use draw::draw_brackets::draw_brackets;
use draw::draw_key_signature::draw_key_signature;
use draw::draw_names::draw_names;
use draw::draw_staves::draw_staves;
use draw::draw_sub_brackets::draw_sub_brackets;
use draw::draw_systemic_barline::draw_systemic_barline;
use parse::get_barlines::get_barlines;
use parse::get_horizontal_spacing::{get_horizontal_spacing, Slot};
use parse::get_key_signatures::get_key_signatures;
use parse::get_max_brackets_width::get_max_brackets_width;
use parse::get_max_instrument_name_width::get_max_instrument_name_width;
use parse::get_measures::get_measures;
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

    let measures = get_measures(flow, &state.score.tracks);
    let barlines = get_barlines(flow, &state.score.tracks, &measures);
    let key_signatures = get_key_signatures(flow, &state.score.tracks);
    let tone_offsets = get_tone_offsets(flow, &instruments, &state.score.tracks);

    let written_durations =
        get_written_durations(flow, &instruments, &state.score.tracks, &measures);

    let horizontal_spacing = get_horizontal_spacing(flow, &barlines, &key_signatures);

    let stave_left =
        frame_padding.left + max_name_width + engrave.instrument_name_spacing + max_bracket_width;
    let height = frame_padding.top + vertical_spacing.height + frame_padding.bottom;
    let width = stave_left + horizontal_spacing.width() + frame_padding.right;

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
        horizontal_spacing.width(),
        flow,
        &instruments,
        &vertical_spacing,
        &mut renderer,
    );

    let slot_count = Slot::count();
    let mut x: Space = 0.0;

    for tick in 0..flow.length {
        let barline_def = barlines.get(&tick);

        for slot_index in 0..slot_count {
            let slot = Slot::from_usize(slot_index);
            let width = horizontal_spacing.get(tick, slot);

            // BARLINES

            if slot == Slot::StartRepeat {
                if let Some(def) = barline_def {
                    if def.start_repeat {
                        draw_barline(
                            &Point {
                                x: stave_left + x,
                                y: frame_padding.top,
                            },
                            &BarlineDrawType::StartRepeat,
                            &vertical_spacing,
                            &vertical_spans,
                            &mut renderer,
                        );
                    }
                }
            }

            if slot == Slot::EndRepeat {
                if let Some(def) = barline_def {
                    if def.end_repeat {
                        draw_barline(
                            &Point {
                                x: stave_left + x,
                                y: frame_padding.top,
                            },
                            &BarlineDrawType::EndRepeat,
                            &vertical_spacing,
                            &vertical_spans,
                            &mut renderer,
                        );
                    }
                }
            }

            if slot == Slot::Barline {
                if let Some(def) = barline_def {
                    if let Some(draw_type) = def.draw_type {
                        draw_barline(
                            &Point {
                                x: stave_left + x,
                                y: frame_padding.top,
                            },
                            &draw_type,
                            &vertical_spacing,
                            &vertical_spans,
                            &mut renderer,
                        );
                    }
                }
            }

            x += width;
        }
    }

    for instrument in &instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &state.score.tracks.by_id[&stave.master_id];
            let stave_spacing = &vertical_spacing.staves[stave_id];
            let stave_top = frame_padding.top + stave_spacing.y;

            let mut clef = Clef::default();

            let mut x: Space = 0.0;

            for tick in 0..flow.length {
                if let Some(found) = stave_master.get_clef_at_tick(&tick) {
                    clef = found.clone();
                }

                for slot_index in 0..slot_count {
                    let slot = Slot::from_usize(slot_index);
                    let width = horizontal_spacing.get(tick, slot);

                    // KEY SIGNATURES
                    if slot == Slot::KeySignature {
                        if let Some(def) = key_signatures.get(&tick) {
                            draw_key_signature(
                                &Point {
                                    x: stave_left + x,
                                    y: stave_top,
                                },
                                &clef,
                                &def.key_signature,
                                def.negate,
                                &mut renderer,
                            );
                        };
                    };

                    x += width;
                }
            }
        }
    }

    Response::new(renderer.data())
}
