mod draw;
mod parse;
mod render_plan;

pub use render_plan::{prepare_flow_render_plan, FlowRenderPlan, FlowRenderPreparationDiagnostics};

use crate::instructions::parse::get_horizontal_spacing::HorizontalSpacing;
use crate::renderer::converter::{Converter, Space};
use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::RendererContext;
use crate::renderer::{DrawInstruction, HitTarget};
use crate::score::components::duration::{Direction, Ticks};
use crate::score::components::measurements::Point;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::entries::clef::Clef;
use crate::score::entries::tempo::TEMPO_STAVE_OFFSET;
use crate::AppContext;
use draw::draw_accidentals::draw_accidental;
use draw::draw_articulations::draw_articulations;
use draw::draw_barline::draw_barline;
use draw::draw_beam::draw_beam;
use draw::draw_braces::draw_braces;
use draw::draw_brackets::draw_brackets;
use draw::draw_clef::{draw_clef, ClefDrawContext};
use draw::draw_dot::draw_dots;
use draw::draw_flag::draw_flag;
use draw::draw_key_signature::{draw_key_signature, KeySignatureDrawContext};
use draw::draw_ledger_lines::draw_ledger_lines;
use draw::draw_names::draw_names;
use draw::draw_notehead::draw_notehead;
use draw::draw_rest::draw_rest;
use draw::draw_staves::draw_staves;
use draw::draw_stem::draw_stem;
use draw::draw_sub_brackets::draw_sub_brackets;
use draw::draw_systemic_barline::draw_systemic_barline;
use draw::draw_tempo::draw_tempo;
use draw::draw_ties::draw_ties;
use draw::draw_time_signature::draw_time_signature;
use parse::get_accidentals::get_accidentals;
use parse::get_articulations::get_articulations;
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
use skia_safe::Color;
use std::time::{Duration, Instant};
use tauri::Emitter;

const MIN_INTER_FLOW_GAP_PX: f32 = 20.0;

#[derive(Clone, Copy, Default)]
struct RenderPhaseTimings {
    vertical: Duration,
    layout: Duration,
    structure: Duration,
    notation: Duration,
    elements: Duration,
    spacing: Duration,
    stems: Duration,
    setup: Duration,
    barlines: Duration,
    stave: Duration,
}

impl RenderPhaseTimings {
    fn add_assign(&mut self, other: Self) {
        self.vertical += other.vertical;
        self.layout += other.layout;
        self.structure += other.structure;
        self.notation += other.notation;
        self.elements += other.elements;
        self.spacing += other.spacing;
        self.stems += other.stems;
        self.setup += other.setup;
        self.barlines += other.barlines;
        self.stave += other.stave;
    }
}

#[derive(Clone)]
struct FlowRenderSegment {
    width: Space,
    height: Space,
    instructions: Vec<DrawInstruction>,
    hit_targets: Vec<HitTarget>,
    timings: RenderPhaseTimings,
}

struct ComposedFlowSurface {
    width: Space,
    height: Space,
    instructions: Vec<DrawInstruction>,
    hit_targets: Vec<HitTarget>,
}

fn inter_flow_gap_spaces(converter: &Converter) -> Space {
    converter.px_to_spaces(MIN_INTER_FLOW_GAP_PX)
}

fn make_segment_renderer(converter: &Converter) -> RendererContext {
    RendererContext {
        backend: None,
        converter: converter.clone(),
        px_per_mm: converter.px_per_mm,
        instructions: Vec::new(),
        width: 0.0,
        height: 0.0,
        hit_targets: Vec::new(),
        bounds: (0.0, 0.0, 0.0, 0.0),
        pixel_density: 1.0,
    }
}

fn compose_render_segments(
    segments: Vec<FlowRenderSegment>,
    converter: &Converter,
) -> ComposedFlowSurface {
    let gap = if segments.len() > 1 {
        inter_flow_gap_spaces(converter)
    } else {
        0.0
    };

    let mut width: Space = 0.0;
    let mut height: Space = 0.0;
    let mut cursor_x: Space = 0.0;
    let mut instructions = Vec::new();
    let mut hit_targets = Vec::new();
    let segment_count = segments.len();

    for (index, segment) in segments.into_iter().enumerate() {
        let x_offset_px = converter.spaces_to_px(cursor_x);

        instructions.extend(
            segment
                .instructions
                .into_iter()
                .map(|instruction| instruction.offset(x_offset_px, 0.0)),
        );
        hit_targets.extend(
            segment
                .hit_targets
                .into_iter()
                .map(|target| target.offset(cursor_x, 0.0)),
        );

        cursor_x += segment.width;
        if index + 1 < segment_count {
            cursor_x += gap;
        }

        width = f32::max(width, cursor_x);
        height = f32::max(height, segment.height);
    }

    ComposedFlowSurface {
        width,
        height,
        instructions,
        hit_targets,
    }
}

fn build_flow_render_segment(
    state: &AppContext,
    layout_id: crate::utils::UniqueId,
    flow_id: crate::utils::UniqueId,
    converter: &Converter,
) -> FlowRenderSegment {
    let flow = &state.score.flows.by_id[&flow_id];
    let flow_master = &state.score.tracks.by_id[&flow.master_id];
    let layout = &state.score.layouts.by_id[&layout_id];
    let instruments = state.get_instruments(layout, flow);
    let engrave = &state.score.engraves.by_layout_type[&layout.layout_type];

    let text_measurer = TextMeasurer::new(converter);
    let selection = state.ui.sequencer.selection.as_ref();
    let mut segment_renderer = make_segment_renderer(converter);

    let mut timings = RenderPhaseTimings::default();

    let vertical_start = Instant::now();
    let vertical_spacing = get_vertical_spacing(flow, &instruments, engrave);
    let vertical_spans = get_vertical_spans(&instruments, engrave);
    timings.vertical = vertical_start.elapsed();

    let layout_start = Instant::now();
    let page_padding = engrave.page_padding.to_spaces(&engrave.space);
    let max_name_width = get_max_instrument_name_width(&instruments, engrave, &text_measurer);
    let max_bracket_width = get_max_brackets_width(&vertical_spacing, &vertical_spans, engrave);
    timings.layout = layout_start.elapsed();

    let structure_start = Instant::now();
    let measures = get_measures(flow, &state.score.tracks);
    let barlines = get_barlines(flow, &state.score.tracks, &measures);
    let key_signatures = get_key_signatures(flow, &state.score.tracks);
    let tone_offsets = get_tone_offsets(flow, &instruments, &state.score.tracks);
    timings.structure = structure_start.elapsed();

    let notation_start = Instant::now();
    let notation_by_track = get_written_durations(
        flow,
        &instruments,
        &state.score.tracks,
        &measures,
        &tone_offsets,
    );
    timings.notation = notation_start.elapsed();

    let elements_start = Instant::now();
    let beams_by_track = get_beams(flow, &notation_by_track, &measures);
    let stem_directions_by_track = get_stem_directions(&notation_by_track, &beams_by_track);
    let shunts_by_track =
        get_note_shunts(&notation_by_track, &tone_offsets, &stem_directions_by_track);
    let dots_by_track = get_dots(flow, &notation_by_track, &tone_offsets);
    let articulations_by_track = get_articulations(&notation_by_track);
    let accidentals_by_track = get_accidentals(
        flow,
        &state.score.tracks,
        &notation_by_track,
        &barlines,
        &tone_offsets,
        &shunts_by_track,
    );
    timings.elements = elements_start.elapsed();

    let spacing_start = Instant::now();
    let horizontal_spacing = get_horizontal_spacing(
        flow,
        &instruments,
        &state.score.tracks,
        &measures,
        &notation_by_track,
        &barlines,
        &shunts_by_track,
        &beams_by_track,
        &accidentals_by_track,
        engrave,
    );
    timings.spacing = spacing_start.elapsed();

    let stems_start = Instant::now();
    let stem_lengths_by_track = get_stem_lengths(
        &notation_by_track,
        &stem_directions_by_track,
        &beams_by_track,
        engrave,
        &horizontal_spacing,
    );
    timings.stems = stems_start.elapsed();

    let setup_start = Instant::now();
    let stave_left =
        page_padding.left + max_name_width + engrave.instrument_name_spacing + max_bracket_width;
    let height = page_padding.top + vertical_spacing.height + page_padding.bottom;
    let width = stave_left + horizontal_spacing.width + page_padding.right;

    segment_renderer.width = width;
    segment_renderer.height = height;

    draw_names(
        &Point {
            x: page_padding.left + max_name_width,
            y: page_padding.top,
        },
        &instruments,
        &vertical_spacing,
        engrave,
        state.ui.renderer.debug,
        &mut segment_renderer,
    );

    draw_braces(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        &mut segment_renderer,
    );

    draw_brackets(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        engrave,
        &mut segment_renderer,
    );

    draw_sub_brackets(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spans,
        &vertical_spacing,
        &mut segment_renderer,
    );

    draw_systemic_barline(
        &Point {
            x: stave_left,
            y: page_padding.top,
        },
        &vertical_spacing,
        engrave,
        &mut segment_renderer,
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
        &mut segment_renderer,
    );
    timings.setup = setup_start.elapsed();

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
                &mut segment_renderer,
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
                &mut segment_renderer,
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
                &mut segment_renderer,
            );
        }
    }

    let metrics = BarlineDrawType::Final.metrics();

    draw_barline(
        &Point {
            x: stave_left + horizontal_spacing.width - metrics.total_width(),
            y: page_padding.top,
        },
        &BarlineDrawType::Final,
        &vertical_spacing,
        &vertical_spans,
        &mut segment_renderer,
    );
    timings.barlines = barlines_start.elapsed();

    if let Some(first_instrument) = instruments.first() {
        if let Some(first_stave_id) = first_instrument.stave_ids.first() {
            let top_stave = &vertical_spacing.staves[first_stave_id];

            for tick in 0..flow.length {
                if let Some(tempo) = flow_master.get_tempo_at_tick(&tick) {
                    let _previous_tempo = flow_master.get_tempo_before_tick(tick);
                    let span = if flow_master.get_time_signature_at_tick(&tick).is_some() {
                        horizontal_spacing.get(tick, Slot::TimeSignature)
                    } else {
                        horizontal_spacing.get(tick, Slot::Accidentals)
                    };
                    draw_tempo(
                        &Point {
                            x: stave_left + span.x,
                            y: page_padding.top + top_stave.y - TEMPO_STAVE_OFFSET,
                        },
                        tempo,
                        &text_measurer,
                        selection,
                        flow_id,
                        page_padding.top + top_stave.y,
                        &mut segment_renderer,
                    );
                }
            }
        }
    }

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

                if should_draw_clef {
                    let span = horizontal_spacing.get(tick, Slot::Clef);
                    draw_clef(
                        &Point {
                            x: stave_left + span.x,
                            y: stave_top,
                        },
                        &clef,
                        tick == 0,
                        &ClefDrawContext {
                            flow_id,
                            stave_id: *stave_id,
                            stave_top,
                            is_selected: matches!(selection, Some(crate::ui::sequencer::Selection::Clef { flow_id: selected_flow_id, stave_id: selected_stave_id, clef_id, .. }) if *selected_flow_id == flow_id && *selected_stave_id == *stave_id && *clef_id == clef.id),
                        },
                        &mut segment_renderer,
                    );
                }

                if let Some(def) = key_signatures.get(&tick) {
                    let span = horizontal_spacing.get(tick, Slot::KeySignature);
                    let selection_key_signature = flow_master
                        .get_key_signature_at_tick(&tick)
                        .unwrap_or(&def.key_signature);
                    draw_key_signature(
                        &Point {
                            x: stave_left + span.x,
                            y: stave_top,
                        },
                        &clef,
                        &def.key_signature,
                        def.negate,
                        &KeySignatureDrawContext {
                            flow_id,
                            key_signature_id: selection_key_signature.id,
                            tick,
                            stave_top,
                            is_selected: matches!(selection, Some(crate::ui::sequencer::Selection::KeySignature { flow_id: selected_flow_id, key_signature_id, .. }) if *selected_flow_id == flow_id && *key_signature_id == selection_key_signature.id),
                        },
                        &mut segment_renderer,
                    );
                };

                if let Some(def) = time_signature {
                    let span = horizontal_spacing.get(tick, Slot::TimeSignature);
                    draw_time_signature(
                        &Point {
                            x: stave_left + span.x,
                            y: stave_top,
                        },
                        def,
                        selection,
                        flow_id,
                        def.id,
                        stave_top,
                        &mut segment_renderer,
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
                    &mut segment_renderer,
                );

                for beam in beams {
                    draw_beam(
                        &Point {
                            x: stave_left,
                            y: stave_top,
                        },
                        beam,
                        stem_lengths,
                        stem_directions,
                        &mut segment_renderer,
                    );
                }

                for (tick, notation) in &notation.track {
                    let glyph = notation.glyph();
                    let metrics = notation.glyph_metrics();
                    let glyph_width = metrics.total_width();
                    let has_post_shunt = notation.has_post_shunt(shunts);

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
                            state.ui.sequencer.selection.as_ref(),
                            *track_id,
                            *stave_id,
                            flow_id,
                            instrument.id,
                            stave_top,
                            &mut segment_renderer,
                        );

                        continue;
                    }

                    let x = horizontal_spacing.get(*tick, Slot::NoteSpacing).x;
                    let stem_length = stem_lengths.get(tick);
                    let stem_direction = stem_directions.get(tick);

                    if let Some(def) = stem_length {
                        draw_stem(
                            &Point {
                                x: stave_left,
                                y: stave_top,
                            },
                            def,
                            &mut segment_renderer,
                        );
                    }

                    let mut has_up_stem_flag = false;
                    if notation.is_flagged(beams) {
                        if let (Some(stem_dir), Some(stem_len)) = (stem_direction, stem_length) {
                            has_up_stem_flag = *stem_dir == Direction::Up;

                            draw_flag(
                                &Point {
                                    x: stave_left,
                                    y: stave_top,
                                },
                                notation,
                                stem_dir,
                                stem_len,
                                &mut segment_renderer,
                            );
                        }
                    }

                    if let Some(tick_dots) = dots.get(tick) {
                        draw_dots(
                            &Point {
                                x: stave_left + x,
                                y: stave_top,
                            },
                            tick_dots,
                            glyph_width,
                            has_post_shunt,
                            has_up_stem_flag,
                            &mut segment_renderer,
                        );
                    }

                    if !notation.tones.is_empty() {
                        draw_ledger_lines(
                            &Point {
                                x: stave_left + x,
                                y: stave_top,
                            },
                            tick,
                            notation,
                            glyph_width,
                            shunts,
                            accidentals,
                            &tone_offsets,
                            &mut segment_renderer,
                        );
                    }

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
                            state.ui.sequencer.selection.as_ref(),
                            *track_id,
                            *stave_id,
                            flow_id,
                            instrument.id,
                            stave_top,
                            &mut segment_renderer,
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
                                state.ui.sequencer.selection.as_ref(),
                                &mut segment_renderer,
                            );
                        }
                    }

                    if let Some(track_artics) = articulations_by_track.get(track_id) {
                        if let Some(articulation_set) = track_artics.get(tick) {
                            draw_articulations(
                                &Point {
                                    x: stave_left + x,
                                    y: stave_top,
                                },
                                articulation_set,
                                notation,
                                glyph_width,
                                stem_directions.get(tick),
                                stem_length,
                                &mut segment_renderer,
                            );
                        }
                    }
                }
            }
        }
    }

    if state.ui.renderer.debug {
        draw_debug_layer(
            &horizontal_spacing,
            stave_left,
            page_padding.top,
            vertical_spacing.height,
            flow.length,
            &mut segment_renderer,
        );
    }

    timings.stave = stave_start.elapsed();

    FlowRenderSegment {
        width,
        height,
        instructions: segment_renderer.instructions,
        hit_targets: segment_renderer.hit_targets,
        timings,
    }
}

pub fn generate_render_instructions(
    app: &tauri::AppHandle,
    state: &AppContext,
    renderer: &mut RendererContext,
    render_plan: &FlowRenderPlan,
) {
    let total_start = Instant::now();
    let layout = &state.score.layouts.by_id[&render_plan.layout_id];
    let engrave = &state.score.engraves.by_layout_type[&layout.layout_type];
    let converter = Converter::new(renderer.px_per_mm, engrave.space);

    let mut timings = RenderPhaseTimings::default();
    let composed_flow_ids = render_plan.composed_flow_ids();
    let segments = composed_flow_ids
        .into_iter()
        .map(|flow_id| build_flow_render_segment(state, render_plan.layout_id, flow_id, &converter))
        .inspect(|segment| timings.add_assign(segment.timings))
        .collect::<Vec<_>>();
    let composed_surface = compose_render_segments(segments, &converter);

    renderer.instructions.clear();
    renderer.hit_targets.clear();
    renderer.converter = converter.clone();
    renderer.width = composed_surface.width;
    renderer.height = composed_surface.height;
    renderer.rectangle(
        0.0,
        0.0,
        composed_surface.width,
        composed_surface.height,
        skia_safe::Color::WHITE,
    );
    renderer.instructions.extend(composed_surface.instructions);
    renderer.hit_targets = composed_surface.hit_targets;

    let total_duration = total_start.elapsed();

    let _ = app.emit(
        "render-timing-vertical",
        timings.vertical.as_micros() as u64,
    );
    let _ = app.emit("render-timing-layout", timings.layout.as_micros() as u64);
    let _ = app.emit(
        "render-timing-structure",
        timings.structure.as_micros() as u64,
    );
    let _ = app.emit(
        "render-timing-notation",
        timings.notation.as_micros() as u64,
    );
    let _ = app.emit(
        "render-timing-elements",
        timings.elements.as_micros() as u64,
    );
    let _ = app.emit("render-timing-spacing", timings.spacing.as_micros() as u64);
    let _ = app.emit("render-timing-stems", timings.stems.as_micros() as u64);
    let _ = app.emit("render-timing-setup", timings.setup.as_micros() as u64);
    let _ = app.emit(
        "render-timing-barlines",
        timings.barlines.as_micros() as u64,
    );
    let _ = app.emit("render-timing-stave", timings.stave.as_micros() as u64);
    let _ = app.emit("render-timing-total", total_duration.as_micros() as u64);
}

fn draw_debug_layer(
    horizontal_spacing: &HorizontalSpacing,
    stave_left: Space,
    stave_top: Space,
    stave_height: Space,
    flow_length: Ticks,
    renderer: &mut RendererContext,
) {
    let slot_colors = [
        Color::from_argb(64, 255, 0, 0),     // PaddingStart - Red with alpha
        Color::from_argb(64, 255, 165, 0),   // EndRepeat - Orange
        Color::from_argb(64, 255, 255, 0),   // Clef - Yellow
        Color::from_argb(64, 0, 255, 0),     // Barline - Green
        Color::from_argb(64, 0, 255, 255),   // KeySignature - Cyan
        Color::from_argb(64, 0, 0, 255),     // TimeSignature - Blue
        Color::from_argb(64, 128, 0, 128),   // StartRepeat - Purple
        Color::from_argb(64, 255, 0, 0),     // BarPadding - Red with alpha
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
        Slot::BarPadding,
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

    // Debug: Draw red boxes around hit targets
    let hit_targets = renderer.hit_targets.clone();
    for target in hit_targets {
        renderer.rectangle(
            target.x,
            target.y,
            target.width,
            target.height,
            Color::from_argb(100, 255, 0, 0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compose_render_segments, inter_flow_gap_spaces, FlowRenderSegment, RenderPhaseTimings,
    };
    use crate::renderer::converter::Converter;
    use crate::renderer::{DrawInstruction, HitTarget};
    use crate::ui::sequencer::Selection;
    use skia_safe::Color;

    #[test]
    fn inter_flow_gap_meets_minimum_css_pixel_requirement() {
        let converter = Converter::new(3.779_527_7, 1.75);

        let gap_spaces = inter_flow_gap_spaces(&converter);

        assert!(converter.spaces_to_px(gap_spaces) >= 20.0);
    }

    #[test]
    fn compose_render_segments_offsets_second_segment_and_extents() {
        let converter = Converter::new(1.0, 1.0);
        let segments = vec![
            FlowRenderSegment {
                width: 5.0,
                height: 3.0,
                instructions: vec![DrawInstruction::Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 5.0,
                    height: 3.0,
                    color: Color::BLACK,
                }],
                hit_targets: vec![HitTarget {
                    entry: Selection::Tempo {
                        flow_id: 10,
                        tempo_id: 1,
                        tick: 0,
                    },
                    x: 1.0,
                    y: 1.0,
                    width: 1.0,
                    height: 1.0,
                    popover_x: 1.0,
                    popover_y: 0.5,
                }],
                timings: RenderPhaseTimings::default(),
            },
            FlowRenderSegment {
                width: 4.0,
                height: 6.0,
                instructions: vec![DrawInstruction::Rectangle {
                    x: 0.0,
                    y: 2.0,
                    width: 4.0,
                    height: 2.0,
                    color: Color::WHITE,
                }],
                hit_targets: vec![HitTarget {
                    entry: Selection::Tempo {
                        flow_id: 20,
                        tempo_id: 2,
                        tick: 32,
                    },
                    x: 0.5,
                    y: 1.5,
                    width: 1.0,
                    height: 1.0,
                    popover_x: 0.5,
                    popover_y: 1.0,
                }],
                timings: RenderPhaseTimings::default(),
            },
        ];

        let composed = compose_render_segments(segments, &converter);

        assert_eq!(composed.width, 29.0);
        assert_eq!(composed.height, 6.0);

        assert_eq!(
            composed.instructions[1],
            DrawInstruction::Rectangle {
                x: 25.0,
                y: 2.0,
                width: 4.0,
                height: 2.0,
                color: Color::WHITE,
            }
        );
        assert_eq!(composed.hit_targets[1].x, 25.5);
        assert_eq!(composed.hit_targets[1].popover_x, 25.5);
    }
}
