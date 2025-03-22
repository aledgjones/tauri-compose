mod debug;
mod draw;
mod parse;

use crate::renderer::converter::Converter;
use crate::renderer::fonts::Fonts;
use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::entries::clef::Clef;
use crate::utils::UniqueId;
use crate::AppState;
use debug::debug_tone_drawing::debug_tone_drawing;
use draw::draw_accidentals::draw_accidental;
use draw::draw_barline::draw_barline;
use draw::draw_braces::draw_braces;
use draw::draw_brackets::draw_brackets;
use draw::draw_clef::draw_clef;
use draw::draw_key_signature::draw_key_signature;
use draw::draw_names::draw_names;
use draw::draw_rest::draw_rest;
use draw::draw_staves::draw_staves;
use draw::draw_sub_brackets::draw_sub_brackets;
use draw::draw_systemic_barline::draw_systemic_barline;
use draw::draw_time_signature::draw_time_signature;
use parse::get_accidentals::get_accidentals;
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
    let flow_master = &state.score.tracks.by_id[&flow.master_id];
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

    let notation_by_track = get_written_durations(
        flow,
        &instruments,
        &state.score.tracks,
        &measures,
        &tone_offsets,
    );

    let accidentals_by_track = get_accidentals(
        flow,
        &state.score.tracks,
        &notation_by_track,
        &barlines,
        &tone_offsets,
    );

    let horizontal_spacing = get_horizontal_spacing(
        flow,
        &barlines,
        &key_signatures,
        &instruments,
        &state.score.tracks,
        &accidentals_by_track,
    );

    let stave_left =
        frame_padding.left + max_name_width + engrave.instrument_name_spacing + max_bracket_width;
    let height = frame_padding.top + vertical_spacing.height + frame_padding.bottom;
    let width = stave_left + horizontal_spacing.width + frame_padding.right;

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
        horizontal_spacing.width,
        flow,
        &instruments,
        &vertical_spacing,
        &mut renderer,
    );

    for (tick, def) in barlines {
        if def.start_repeat {
            let span = horizontal_spacing.get(tick, Slot::StartRepeat);
            draw_barline(
                &Point {
                    x: stave_left + span.x,
                    y: frame_padding.top,
                },
                &BarlineDrawType::StartRepeat,
                &vertical_spacing,
                &vertical_spans,
                &mut renderer,
            );
        }

        if def.end_repeat {
            let span = horizontal_spacing.get(tick, Slot::EndRepeat);
            draw_barline(
                &Point {
                    x: stave_left + span.x,
                    y: frame_padding.top,
                },
                &BarlineDrawType::EndRepeat,
                &vertical_spacing,
                &vertical_spans,
                &mut renderer,
            );
        }

        if let Some(draw_type) = def.draw_type {
            let span = horizontal_spacing.get(tick, Slot::Barline);
            draw_barline(
                &Point {
                    x: stave_left + span.x,
                    y: frame_padding.top,
                },
                &draw_type,
                &vertical_spacing,
                &vertical_spans,
                &mut renderer,
            );
        }
    }

    draw_barline(
        &Point {
            x: stave_left + horizontal_spacing.width - BarlineDrawType::Final.metrics().width,
            y: frame_padding.top,
        },
        &BarlineDrawType::Final,
        &vertical_spacing,
        &vertical_spans,
        &mut renderer,
    );

    for instrument in &instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &state.score.tracks.by_id[&stave.master_id];
            let stave_spacing = &vertical_spacing.staves[stave_id];
            let stave_top = frame_padding.top + stave_spacing.y;

            let mut clef = Clef::default();

            for tick in 0..flow.length {
                let time_signature = flow_master.get_time_signature_at_tick(&tick);
                let should_draw_clef = stave_master.get_clef_at_tick(&tick).is_some_and(|found| {
                    clef = found.clone();
                    true
                });

                // CLEF
                if should_draw_clef {
                    let span = horizontal_spacing.get(tick, Slot::Clef);
                    draw_clef(
                        &Point {
                            x: stave_left + span.x,
                            y: stave_top,
                        },
                        &clef,
                        tick == 0,
                        &mut renderer,
                    );
                }

                // KEY SIGNATURES
                if let Some(def) = key_signatures.get(&tick) {
                    let span = horizontal_spacing.get(tick, Slot::KeySignature);
                    draw_key_signature(
                        &Point {
                            x: stave_left + span.x,
                            y: stave_top,
                        },
                        &clef,
                        &def.key_signature,
                        def.negate,
                        &mut renderer,
                    );
                };

                // TIME SIGNATURE
                if let Some(def) = time_signature {
                    let span = horizontal_spacing.get(tick, Slot::TimeSignature);
                    draw_time_signature(
                        &Point {
                            x: stave_left + span.x,
                            y: stave_top,
                        },
                        def,
                        &mut renderer,
                    )
                }

                for track_id in &stave.track_ids {
                    let notation = &notation_by_track[track_id];
                    let accidentals = &accidentals_by_track[track_id];

                    if let Some(entry) = notation.track.get(&tick) {
                        // REST
                        if entry.is_rest() {
                            let is_full_bar = measures.contains_key(&tick)
                                && (measures.contains_key(&(tick + entry.duration))
                                    || tick + entry.duration == flow.length);

                            draw_rest(
                                &Point {
                                    x: stave_left,
                                    y: stave_top,
                                },
                                tick,
                                entry,
                                flow.subdivisions,
                                is_full_bar,
                                &horizontal_spacing,
                                &mut renderer,
                            );
                        }

                        // TONES
                        for tone in &entry.tones {
                            if let Some(accidental) = accidentals.by_id.get(&(tick, tone.id)) {
                                draw_accidental(
                                    &Point {
                                        x: stave_left,
                                        y: stave_top,
                                    },
                                    tick,
                                    accidental,
                                    tone,
                                    &tone_offsets,
                                    &horizontal_spacing,
                                    &mut renderer,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    debug_tone_drawing(
        &Point {
            x: stave_left,
            y: frame_padding.top,
        },
        flow,
        &instruments,
        &notation_by_track,
        &vertical_spacing,
        &horizontal_spacing,
        &tone_offsets,
        &mut renderer,
    );

    Response::new(renderer.data())
}
