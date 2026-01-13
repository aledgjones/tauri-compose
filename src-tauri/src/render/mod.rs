mod draw;
mod parse;

use std::time::Instant;
use tauri::Emitter;

use crate::renderer::converter::Converter;
use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::entries::clef::Clef;
use crate::ui::sequencer::SelectionType;
use crate::utils::UniqueId;
use crate::AppStateInner;
use draw::draw_accidentals::draw_accidental;
use draw::draw_barline::draw_barline;
use draw::draw_beam::draw_beam;
use draw::draw_braces::draw_braces;
use draw::draw_brackets::draw_brackets;
use draw::draw_clef::draw_clef;
use draw::draw_dot::draw_dots;
use draw::draw_flag::draw_flag;
use draw::draw_key_signature::draw_key_signature;
use draw::draw_ledger_lines::draw_ledger_line;
use draw::draw_names::draw_names;
use draw::draw_notehead::draw_notehead;
use draw::draw_rest::draw_rest;
use draw::draw_staves::draw_staves;
use draw::draw_stem::draw_stem;
use draw::draw_sub_brackets::draw_sub_brackets;
use draw::draw_systemic_barline::draw_systemic_barline;
use draw::draw_ties::draw_ties;
use draw::draw_time_signature::draw_time_signature;
use parse::get_accidentals::get_accidentals;
use parse::get_barlines::get_barlines;
use parse::get_beams::get_beams;
use parse::get_dots::get_dots;
use parse::get_horizontal_spacing::{get_horizontal_spacing, Slot};
use parse::get_key_signatures::get_key_signatures;
use parse::get_max_brackets_width::get_max_brackets_width;
use parse::get_max_instrument_name_width::get_max_instrument_name_width;
use parse::get_measures::get_measures;
use parse::get_shunts::get_note_shunts;
use parse::get_stem_directions::get_stem_directions;
use parse::get_stem_lengths::get_stem_lengths;
use parse::get_tone_offsets::get_tone_offsets;
use parse::get_vertical_spacing::get_vertical_spacing;
use parse::get_vertical_spans::get_vertical_spans;
use parse::get_written_durations::get_written_durations;

pub fn generate_render_instructions(
    app: &tauri::AppHandle,
    state: &AppStateInner,
    renderer: &mut Renderer,
    layout_id: UniqueId,
    flow_id: UniqueId,
) {
    let total_start = Instant::now();

    let selection = state.ui.sequencer.selection.as_ref().and_then(|selection| {
        if selection.selection_type == SelectionType::Tone {
            Some(selection.id)
        } else {
            None
        }
    });

    let flow = &state.score.flows.by_id[&flow_id];
    let flow_master = &state.score.tracks.by_id[&flow.master_id];
    let layout = &state.score.layouts.by_id[&layout_id];
    let instruments = state.get_instruments(layout, flow);
    let engrave = &state.score.engraves.by_layout_type[&layout.layout_type];

    let converter = Converter::new(renderer.px_per_mm, engrave.space);
    let text_measurer = TextMeasurer::new(&converter);

    // Phase 1: Vertical spacing and spans
    let vertical_start = Instant::now();
    let vertical_spacing = get_vertical_spacing(flow, &instruments, engrave);
    let vertical_spans = get_vertical_spans(&instruments, engrave);
    let vertical_duration = vertical_start.elapsed();

    // Phase 2: Basic layout calculations
    let layout_start = Instant::now();
    let page_padding = engrave.page_padding.to_spaces(&engrave.space);
    let max_name_width = get_max_instrument_name_width(&instruments, engrave, &text_measurer);
    let max_bracket_width = get_max_brackets_width(&vertical_spacing, &vertical_spans, engrave);
    let layout_duration = layout_start.elapsed();

    // Phase 3: Score structure parsing
    let structure_start = Instant::now();
    let measures = get_measures(flow, &state.score.tracks);
    let barlines = get_barlines(flow, &state.score.tracks, &measures);
    let key_signatures = get_key_signatures(flow, &state.score.tracks);
    let tone_offsets = get_tone_offsets(flow, &instruments, &state.score.tracks);
    let structure_duration = structure_start.elapsed();

    // Phase 4: Notation analysis
    let notation_start = Instant::now();
    let notation_by_track = get_written_durations(
        flow,
        &instruments,
        &state.score.tracks,
        &measures,
        &tone_offsets,
    );
    let notation_duration = notation_start.elapsed();

    // Phase 5: Musical elements analysis
    let elements_start = Instant::now();
    let beams_by_track = get_beams(flow, &notation_by_track, &measures);
    let stem_directions_by_track = get_stem_directions(&notation_by_track, &beams_by_track);
    let shunts_by_track =
        get_note_shunts(&notation_by_track, &tone_offsets, &stem_directions_by_track);
    let dots_by_track = get_dots(flow, &notation_by_track, &tone_offsets);
    let accidentals_by_track = get_accidentals(
        flow,
        &state.score.tracks,
        &notation_by_track,
        &barlines,
        &tone_offsets,
        &shunts_by_track,
    );
    let elements_duration = elements_start.elapsed();

    // Phase 6: Horizontal spacing
    let spacing_start = Instant::now();
    let horizontal_spacing = get_horizontal_spacing(
        flow,
        &instruments,
        &state.score.tracks,
        &notation_by_track,
        &barlines,
        &shunts_by_track,
        &beams_by_track,
        &accidentals_by_track,
        engrave,
    );
    let spacing_duration = spacing_start.elapsed();

    // Phase 7: Stem lengths
    let stems_start = Instant::now();
    let stem_lengths_by_track = get_stem_lengths(
        &notation_by_track,
        &stem_directions_by_track,
        &beams_by_track,
        engrave,
        &horizontal_spacing,
    );
    let stems_duration = stems_start.elapsed();

    // Phase 8: Setup and basic drawing
    let setup_start = Instant::now();
    let stave_left =
        page_padding.left + max_name_width + engrave.instrument_name_spacing + max_bracket_width;
    let height = page_padding.top + vertical_spacing.height + page_padding.bottom;
    let width = stave_left + horizontal_spacing.width + page_padding.right;

    renderer.clear_instructions();
    renderer.set_converter(converter);
    renderer.set_dimensions(width, height);

    // Draw white background for the score page
    renderer.rectangle(0.0, 0.0, width, height, skia_safe::Color::WHITE);

    draw_names(
        &Point {
            x: page_padding.left + max_name_width,
            y: page_padding.top,
        },
        &instruments,
        &vertical_spacing,
        engrave,
        renderer,
    );

    draw_braces(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        renderer,
    );

    draw_brackets(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        engrave,
        renderer,
    );

    draw_sub_brackets(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        renderer,
    );

    draw_systemic_barline(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spacing,
        engrave,
        renderer,
    );

    draw_staves(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        horizontal_spacing.width,
        flow,
        &instruments,
        &vertical_spacing,
        renderer,
    );
    let setup_duration = setup_start.elapsed();

    // Phase 9: Barlines
    let barlines_start = Instant::now();
    for (tick, def) in barlines {
        if def.start_repeat {
            let span = horizontal_spacing.get(tick, Slot::StartRepeat);
            draw_barline(
                &Point {
                    x: stave_left + span.x,
                    y: page_padding.top,
                },
                &BarlineDrawType::StartRepeat,
                &vertical_spacing,
                &vertical_spans,
                renderer,
            );
        }

        if def.end_repeat {
            let span = horizontal_spacing.get(tick, Slot::EndRepeat);
            draw_barline(
                &Point {
                    x: stave_left + span.x,
                    y: page_padding.top,
                },
                &BarlineDrawType::EndRepeat,
                &vertical_spacing,
                &vertical_spans,
                renderer,
            );
        }

        if let Some(draw_type) = def.draw_type {
            let span = horizontal_spacing.get(tick, Slot::Barline);
            draw_barline(
                &Point {
                    x: stave_left + span.x,
                    y: page_padding.top,
                },
                &draw_type,
                &vertical_spacing,
                &vertical_spans,
                renderer,
            );
        }
    }

    draw_barline(
        &Point {
            x: stave_left + horizontal_spacing.width - BarlineDrawType::Final.metrics().width,
            y: page_padding.top,
        },
        &BarlineDrawType::Final,
        &vertical_spacing,
        &vertical_spans,
        renderer,
    );
    let barlines_duration = barlines_start.elapsed();

    // Phase 10: Stave contents (clefs, key signatures, time signatures, notes)
    let stave_start = Instant::now();
    for instrument in &instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &state.score.tracks.by_id[&stave.master_id];
            let stave_spacing = &vertical_spacing.staves[stave_id];
            let stave_top = page_padding.top + stave_spacing.y;

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
                        renderer,
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
                        renderer,
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
                        renderer,
                    )
                }
            }

            for track_id in &stave.track_ids {
                let notation = &notation_by_track[track_id];
                let accidentals = &accidentals_by_track[track_id];
                let shunts = &shunts_by_track[track_id];
                let stem_lengths = &stem_lengths_by_track[track_id];
                let stem_directions = &stem_directions_by_track[track_id];
                let beams = &beams_by_track[track_id];
                let dots = &dots_by_track[track_id];

                // TIES
                draw_ties(
                    &Point {
                        x: stave_left,
                        y: stave_top,
                    },
                    notation,
                    stem_directions,
                    &horizontal_spacing,
                    shunts,
                    &tone_offsets,
                    renderer,
                );

                // BEAMS
                for beam in beams {
                    draw_beam(
                        &Point {
                            x: stave_left,
                            y: stave_top,
                        },
                        beam,
                        stem_lengths,
                        stem_directions,
                        renderer,
                    );
                }

                for (tick, notation) in &notation.track {
                    let glyph = notation.glyph();
                    let glyph_width = notation.glyph_metrics().width;
                    let has_post_shunt = notation.has_post_shunt(shunts);

                    // RESTS
                    if notation.is_rest() {
                        let is_full_bar = measures.contains_key(tick)
                            && (measures.contains_key(&(tick + notation.duration))
                                || tick + notation.duration == flow.length);

                        draw_rest(
                            &Point {
                                x: stave_left,
                                y: stave_top,
                            },
                            *tick,
                            glyph,
                            notation,
                            is_full_bar,
                            &horizontal_spacing,
                            renderer,
                        );

                        continue;
                    }

                    let x = horizontal_spacing.get(*tick, Slot::NoteSpacing).x;

                    // Get stem data once for reuse
                    let stem_length = stem_lengths.get(tick);
                    let stem_direction = stem_directions.get(tick);

                    // STEMS
                    if let Some(def) = stem_length {
                        draw_stem(
                            &Point {
                                x: stave_left,
                                y: stave_top,
                            },
                            def,
                            renderer,
                        );
                    }

                    // FLAGS
                    if notation.is_flagged(beams) {
                        if let (Some(stem_dir), Some(stem_len)) = (stem_direction, stem_length) {
                            draw_flag(
                                &Point {
                                    x: stave_left,
                                    y: stave_top,
                                },
                                notation,
                                stem_dir,
                                stem_len,
                                flow,
                                renderer,
                            );
                        }
                    }

                    // DOTS
                    if let Some(tick_dots) = dots.get(tick) {
                        draw_dots(
                            &Point {
                                x: stave_left + x,
                                y: stave_top,
                            },
                            tick_dots,
                            glyph_width,
                            has_post_shunt,
                            renderer,
                        );
                    }

                    // LEDGER LINES
                    if !notation.tones.is_empty() {
                        draw_ledger_line(
                            &Point {
                                x: stave_left + x,
                                y: stave_top,
                            },
                            tick,
                            notation,
                            glyph_width,
                            shunts,
                            renderer,
                        );
                    }

                    // TONES
                    for tone in &notation.tones {
                        let vertical_offset = tone_offsets[&tone.id];

                        draw_notehead(
                            &Point {
                                x: stave_left + x,
                                y: stave_top + (vertical_offset as f32 / 2.0),
                            },
                            *tick,
                            tone,
                            glyph,
                            glyph_width,
                            shunts,
                            selection,
                            renderer,
                        );

                        if let Some(accidental) = accidentals.by_id.get(&(*tick, tone.id)) {
                            let has_pre_shunt = notation.has_pre_shunt(shunts);

                            draw_accidental(
                                &Point {
                                    x: stave_left + x,
                                    y: stave_top + (vertical_offset as f32 / 2.0),
                                },
                                accidental,
                                tone,
                                glyph_width,
                                has_pre_shunt,
                                selection,
                                renderer,
                            );
                        }
                    }
                }
            }
        }
    }

    // Debug: draw slot visualization
    if state.ui.renderer.debug_rendering {
        draw_debug_slots(
            &horizontal_spacing,
            stave_left,
            page_padding.top,
            vertical_spacing.height,
            flow.length,
            renderer,
        );
    }

    let stave_duration = stave_start.elapsed();

    let total_duration = total_start.elapsed();

    // Emit timing events for each phase
    let _ = app.emit(
        "render-timing-vertical",
        vertical_duration.as_millis() as u64,
    );
    let _ = app.emit("render-timing-layout", layout_duration.as_millis() as u64);
    let _ = app.emit(
        "render-timing-structure",
        structure_duration.as_millis() as u64,
    );
    let _ = app.emit(
        "render-timing-notation",
        notation_duration.as_millis() as u64,
    );
    let _ = app.emit(
        "render-timing-elements",
        elements_duration.as_millis() as u64,
    );
    let _ = app.emit("render-timing-spacing", spacing_duration.as_millis() as u64);
    let _ = app.emit("render-timing-stems", stems_duration.as_millis() as u64);
    let _ = app.emit("render-timing-setup", setup_duration.as_millis() as u64);
    let _ = app.emit(
        "render-timing-barlines",
        barlines_duration.as_millis() as u64,
    );
    let _ = app.emit("render-timing-stave", stave_duration.as_millis() as u64);
    let _ = app.emit("render-timing-total", total_duration.as_millis() as u64);
}

fn draw_debug_slots(
    horizontal_spacing: &parse::get_horizontal_spacing::HorizontalSpacing,
    stave_left: crate::renderer::converter::Space,
    stave_top: crate::renderer::converter::Space,
    stave_height: crate::renderer::converter::Space,
    flow_length: crate::score::components::duration::Ticks,
    renderer: &mut Renderer,
) {
    use parse::get_horizontal_spacing::Slot;
    use skia_safe::Color;

    let slot_colors = [
        Color::from_argb(64, 255, 0, 0),     // PaddingStart - Red with alpha
        Color::from_argb(64, 255, 165, 0),   // EndRepeat - Orange
        Color::from_argb(64, 255, 255, 0),   // Clef - Yellow
        Color::from_argb(64, 0, 255, 0),     // Barline - Green
        Color::from_argb(64, 0, 255, 255),   // KeySignature - Cyan
        Color::from_argb(64, 0, 0, 255),     // TimeSignature - Blue
        Color::from_argb(64, 128, 0, 128),   // StartRepeat - Purple
        Color::from_argb(64, 255, 0, 255),   // Accidentals - Magenta
        Color::from_argb(64, 128, 128, 128), // PreNote - Gray
        Color::from_argb(64, 0, 0, 0),       // NoteSpacing - Black
    ];

    let slots = [
        Slot::PaddingStart,
        Slot::EndRepeat,
        Slot::Clef,
        Slot::Barline,
        Slot::KeySignature,
        Slot::TimeSignature,
        Slot::StartRepeat,
        Slot::Accidentals,
        Slot::PreNote,
        Slot::NoteSpacing,
    ];

    // Draw slot bars as vertical translucent overlays
    for tick in 0..=flow_length {
        for (i, &slot) in slots.iter().enumerate() {
            if slot == Slot::NoteSpacing {
                continue;
            }
            let span = horizontal_spacing.get(tick, slot);
            if span.width > 0.0 {
                let color = slot_colors[i];
                renderer.rectangle(
                    stave_left + span.x,
                    stave_top,
                    span.width,
                    stave_height,
                    color,
                );
            }
        }
    }
}
