mod draw;
mod parse;
mod render_plan;

pub use render_plan::{
    prepare_flow_render_plan, FlowRenderPlan, FlowRenderPreparationDiagnostics,
    FlowRenderRecalculationRequest,
};

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
use crate::score::entries::tuplet::Tuplet;
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
use draw::draw_ties::{draw_tie_between_contexts, draw_ties, get_tie_directions};
use draw::draw_time_signature::draw_time_signature;
use draw::draw_tuplet::draw_tuplet;
use parse::get_accidentals::get_accidentals_with_anchored_tracks;
use parse::get_accidentals::Accidentals;
use parse::get_articulations::get_articulations;
use parse::get_articulations::Articulations as TrackArticulations;
use parse::get_barlines::get_barlines;
use parse::get_beams::{get_beams, Beams};
use parse::get_dots::{get_dots, Dots};
use parse::get_horizontal_spacing::{
    get_horizontal_spacing_with_anchored_layouts, stretch_spacing_to_width_using_local_spacing,
    AnchoredTrackLayout, Slot,
};
use parse::get_key_signatures::get_key_signatures;
use parse::get_max_brackets_width::get_max_brackets_width;
use parse::get_max_instrument_name_width::get_max_instrument_name_width;
use parse::get_measures::{get_measures, Measures};
use parse::get_shunts::{get_note_shunts, Shunts};
use parse::get_stem_directions::{get_stem_directions, StemDirections};
use parse::get_stem_lengths::{get_stem_lengths, get_stem_lengths_in_track, StemLengths};
use parse::get_tone_offsets::get_tone_offsets;
use parse::get_tone_offsets::ToneVerticalOffsets;
use parse::get_vertical_spacing::get_vertical_spacing;
use parse::get_vertical_spans::get_vertical_spans;
use parse::get_written_durations::{
    get_written_durations, AnchoredNotationTrack, Notation, NotationTrack,
};
use rustc_hash::FxHashMap;
use serde::Serialize;
use skia_safe::Color;
use std::cmp::Ordering;
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
pub(crate) struct FlowRenderSegment {
    width: Space,
    height: Space,
    instructions: Vec<DrawInstruction>,
    hit_targets: Vec<HitTarget>,
    timings: RenderPhaseTimings,
}

struct ComposedFlowSurface {
    width: Space,
    height: Space,
    page_offsets: Vec<Space>,
    page_widths: Vec<Space>,
    inter_flow_gap: Space,
    page_backgrounds: Vec<DrawInstruction>,
    instructions: Vec<DrawInstruction>,
    hit_targets: Vec<HitTarget>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectedTrackKind {
    Host,
    Anchored(crate::utils::UniqueId),
}

struct ProjectedTrackTieContext<'a> {
    kind: ProjectedTrackKind,
    origin: Point,
    notation: &'a NotationTrack,
    horizontal_spacing: &'a HorizontalSpacing,
    shunts: &'a Shunts,
    stem_directions: &'a StemDirections,
    host_start_tick: u64,
    host_end_tick: u64,
}

struct AnchoredTrackRenderContext<'a> {
    tuplet: &'a Tuplet,
    anchored_track: &'a AnchoredNotationTrack,
    tuplet_id: crate::utils::UniqueId,
    host_left: Space,
    host_right: Space,
    origin: Point,
    measures: Measures,
    horizontal_spacing: HorizontalSpacing,
    beams: Beams,
    stem_directions: StemDirections,
    stem_lengths: StemLengths,
    shunts: Shunts,
    accidentals: Accidentals,
    dots: Dots,
    articulations: TrackArticulations,
}

struct TieFragment<'a> {
    tone_id: crate::utils::UniqueId,
    order_key: f32,
    context: &'a ProjectedTrackTieContext<'a>,
    entry: &'a Notation,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct RenderSurfacePageDiagnostics {
    page_index: usize,
    x_px: f32,
    width_px: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct RenderSurfaceDiagnostics {
    surface_width_px: f32,
    surface_height_px: f32,
    viewport_width_px: f32,
    viewport_height_px: f32,
    page_count: usize,
    inter_flow_gap_px: f32,
    page_boxes: Vec<RenderSurfacePageDiagnostics>,
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
        flow_segment_cache: None,
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
    let mut page_offsets = Vec::new();
    let mut page_widths = Vec::new();
    let mut page_backgrounds = Vec::new();
    let mut instructions = Vec::new();
    let mut hit_targets = Vec::new();
    let segment_count = segments.len();

    for (index, segment) in segments.into_iter().enumerate() {
        page_offsets.push(cursor_x);
        page_widths.push(segment.width);
        let x_offset_px = converter.spaces_to_px(cursor_x);

        page_backgrounds.push(DrawInstruction::Rectangle {
            x: x_offset_px,
            y: 0.0,
            width: converter.spaces_to_px(segment.width),
            height: converter.spaces_to_px(segment.height),
            color: Color::WHITE,
        });

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
        page_offsets,
        page_widths,
        inter_flow_gap: gap,
        page_backgrounds,
        instructions,
        hit_targets,
    }
}

struct AnchoredTrackHostLayout {
    left: Space,
    right: Space,
    stave_top: Space,
}

fn build_anchored_track_render_context<'a>(
    tuplet: &'a Tuplet,
    anchored_track: &'a AnchoredNotationTrack,
    layout: &AnchoredTrackLayout,
    engrave: &crate::score::engraves::Engrave,
    host_layout: AnchoredTrackHostLayout,
) -> AnchoredTrackRenderContext<'a> {
    let mut horizontal_spacing = layout.horizontal_spacing.clone();
    stretch_spacing_to_width_using_local_spacing(
        &mut horizontal_spacing,
        host_layout.right - host_layout.left,
    );
    let stem_lengths = get_stem_lengths_in_track(
        &anchored_track.notation,
        &layout.stem_directions,
        &layout.beams,
        engrave,
        &horizontal_spacing,
    );

    AnchoredTrackRenderContext {
        tuplet,
        anchored_track,
        tuplet_id: anchored_track.owner_id,
        host_left: host_layout.left,
        host_right: host_layout.right,
        origin: Point {
            x: host_layout.left,
            y: host_layout.stave_top,
        },
        measures: layout.measures.clone(),
        horizontal_spacing,
        beams: layout.beams.clone(),
        stem_directions: layout.stem_directions.clone(),
        stem_lengths,
        shunts: layout.shunts.clone(),
        accidentals: layout.accidentals.clone(),
        dots: layout.dots.clone(),
        articulations: layout.articulations.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_notation_symbols(
    origin: &Point,
    notation_track: &NotationTrack,
    measures: &Measures,
    horizontal_spacing: &HorizontalSpacing,
    shunts: &Shunts,
    accidentals: &Accidentals,
    stem_lengths: &StemLengths,
    stem_directions: &StemDirections,
    beams: &Beams,
    dots: &Dots,
    articulations: &TrackArticulations,
    tone_offsets: &ToneVerticalOffsets,
    selection: Option<&crate::ui::sequencer::Selection>,
    track_id: crate::utils::UniqueId,
    stave_id: crate::utils::UniqueId,
    flow_id: crate::utils::UniqueId,
    instrument_id: crate::utils::UniqueId,
    tuplet_id: Option<crate::utils::UniqueId>,
    stave_top: Space,
    renderer: &mut RendererContext,
) {
    for beam in beams {
        draw_beam(origin, beam, stem_lengths, stem_directions, renderer);
    }

    for (tick, notation) in &notation_track.track {
        let glyph = notation.glyph();
        let metrics = notation.glyph_metrics();
        let glyph_width = metrics.total_width();
        let has_post_shunt = notation.has_post_shunt(shunts);

        if notation.is_rest() {
            let is_full_bar = measures.contains_key(tick)
                && (measures.contains_key(&(tick + notation.duration))
                    || tick + notation.duration == notation_track.length);

            draw_rest(
                origin,
                *tick,
                glyph,
                notation,
                is_full_bar,
                horizontal_spacing,
                selection,
                track_id,
                stave_id,
                flow_id,
                instrument_id,
                tuplet_id,
                stave_top,
                renderer,
            );

            continue;
        }

        let x = horizontal_spacing.get(*tick, Slot::NoteSpacing).x;
        let stem_length = stem_lengths.get(tick);
        let stem_direction = stem_directions.get(tick);

        if let Some(def) = stem_length {
            draw_stem(origin, def, renderer);
        }

        let mut has_up_stem_flag = false;
        if notation.is_flagged(beams) {
            if let (Some(stem_dir), Some(stem_len)) = (stem_direction, stem_length) {
                has_up_stem_flag = *stem_dir == Direction::Up;

                draw_flag(origin, notation, stem_dir, stem_len, renderer);
            }
        }

        if let Some(tick_dots) = dots.get(tick) {
            draw_dots(
                &Point {
                    x: origin.x + x,
                    y: origin.y,
                },
                tick_dots,
                glyph_width,
                has_post_shunt,
                has_up_stem_flag,
                renderer,
            );
        }

        if !notation.tones.is_empty() {
            draw_ledger_lines(
                &Point {
                    x: origin.x + x,
                    y: origin.y,
                },
                tick,
                notation,
                glyph_width,
                shunts,
                accidentals,
                tone_offsets,
                renderer,
            );
        }

        for tone in &notation.tones {
            let vertical_offset = tone_offsets[&tone.id];

            draw_notehead(
                &Point {
                    x: origin.x + x,
                    y: origin.y + (vertical_offset as f32 / 2.0),
                },
                *tick,
                tone,
                glyph,
                glyph_width,
                shunts,
                selection,
                track_id,
                stave_id,
                flow_id,
                instrument_id,
                tuplet_id,
                stave_top,
                renderer,
            );

            if let Some(accidental) = accidentals.by_id.get(&(*tick, tone.id)) {
                let has_pre_shunt = notation.has_pre_shunt(shunts);

                draw_accidental(
                    &Point {
                        x: origin.x + x,
                        y: origin.y + (vertical_offset as f32 / 2.0),
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

        if let Some(articulation_set) = articulations.get(tick) {
            draw_articulations(
                &Point {
                    x: origin.x + x,
                    y: origin.y,
                },
                articulation_set,
                notation,
                glyph_width,
                stem_direction,
                stem_length,
                renderer,
            );
        }
    }
}

fn fragment_order_key(context: &ProjectedTrackTieContext<'_>, entry: &Notation) -> f32 {
    match context.kind {
        ProjectedTrackKind::Host => entry.tick as f32,
        ProjectedTrackKind::Anchored(_) => {
            let host_span = context
                .host_end_tick
                .saturating_sub(context.host_start_tick)
                .max(1);
            let local_length = context.notation.length.max(1);
            context.host_start_tick as f32
                + (entry.tick as f32 / local_length as f32) * host_span as f32
        }
    }
}

fn draw_cross_context_ties(
    contexts: &[ProjectedTrackTieContext<'_>],
    tone_offsets: &ToneVerticalOffsets,
    renderer: &mut RendererContext,
) {
    let mut by_tone: FxHashMap<crate::utils::UniqueId, Vec<TieFragment<'_>>> = FxHashMap::default();

    for context in contexts {
        for entry in context.notation.track.values() {
            for tone in &entry.tones {
                by_tone.entry(tone.id).or_default().push(TieFragment {
                    tone_id: tone.id,
                    order_key: fragment_order_key(context, entry),
                    context,
                    entry,
                });
            }
        }
    }

    for fragments in by_tone.values_mut() {
        fragments.sort_by(|left, right| {
            left.order_key
                .partial_cmp(&right.order_key)
                .unwrap_or(Ordering::Equal)
        });

        for pair in fragments.windows(2) {
            let [start, stop] = pair else {
                continue;
            };

            if start.context.kind == stop.context.kind {
                continue;
            }

            let Some(stem_direction) = start.context.stem_directions.get(&start.entry.tick) else {
                continue;
            };

            let mut start_for_tie = start.entry.clone();
            start_for_tie.ties.insert(start.tone_id);
            let tie_directions = get_tie_directions(&start_for_tie, stem_direction, tone_offsets);
            let Some(tie_direction) = tie_directions.get(&start.tone_id) else {
                continue;
            };

            draw_tie_between_contexts(
                &start.context.origin,
                &start.tone_id,
                start.entry,
                start.context.horizontal_spacing,
                start.context.shunts,
                &stop.context.origin,
                stop.entry,
                stop.context.horizontal_spacing,
                stop.context.shunts,
                tie_direction,
                stem_direction,
                tone_offsets,
                renderer,
            );
        }
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
    let (accidentals_by_track, anchored_accidentals_by_track) =
        get_accidentals_with_anchored_tracks(
            flow,
            &state.score.tracks,
            &notation_by_track,
            &barlines,
            &tone_offsets,
            &shunts_by_track,
        );
    timings.elements = elements_start.elapsed();

    let spacing_start = Instant::now();
    let (horizontal_spacing, anchored_layouts_by_track) =
        get_horizontal_spacing_with_anchored_layouts(
            flow,
            &instruments,
            &state.score.tracks,
            &measures,
            &notation_by_track,
            &barlines,
            &shunts_by_track,
            &beams_by_track,
            &accidentals_by_track,
            &anchored_accidentals_by_track,
            &tone_offsets,
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
                let track_master = &state.score.tracks.by_id[track_id];
                let notation = &notation_by_track[track_id];
                let accidentals = &accidentals_by_track[track_id];
                let shunts = &shunts_by_track[track_id];
                let stem_lengths = &stem_lengths_by_track[track_id];
                let stem_directions = &stem_directions_by_track[track_id];
                let beams = &beams_by_track[track_id];
                let dots = &dots_by_track[track_id];
                let empty_articulations = TrackArticulations::default();
                let track_articulations = articulations_by_track
                    .get(track_id)
                    .unwrap_or(&empty_articulations);

                let anchored_render_contexts = notation
                    .anchored_tracks
                    .iter()
                    .filter_map(|anchored_track| {
                        let Some(crate::score::entries::Entry::Tuplet(tuplet)) =
                            track_master.entries.by_id.get(&anchored_track.owner_id)
                        else {
                            return None;
                        };

                        let host_start = horizontal_spacing
                            .get(anchored_track.host_span.start_tick, Slot::NoteSpacing);
                        let host_end_tick = anchored_track.host_span.end_tick.saturating_sub(1);
                        let host_end = horizontal_spacing.get(host_end_tick, Slot::NoteSpacing);
                        let host_left = stave_left + host_start.x;
                        let host_right = stave_left + host_end.x + host_end.width;
                        let layout = anchored_layouts_by_track
                            .get(track_id)
                            .and_then(|layouts| layouts.get(&anchored_track.owner_id))?;

                        Some(build_anchored_track_render_context(
                            tuplet,
                            anchored_track,
                            layout,
                            engrave,
                            AnchoredTrackHostLayout {
                                left: host_left,
                                right: host_right,
                                stave_top,
                            },
                        ))
                    })
                    .collect::<Vec<_>>();

                let mut tie_contexts = vec![ProjectedTrackTieContext {
                    kind: ProjectedTrackKind::Host,
                    origin: Point {
                        x: stave_left,
                        y: stave_top,
                    },
                    notation,
                    horizontal_spacing: &horizontal_spacing,
                    shunts,
                    stem_directions,
                    host_start_tick: 0,
                    host_end_tick: notation.length,
                }];

                for anchored in &anchored_render_contexts {
                    tie_contexts.push(ProjectedTrackTieContext {
                        kind: ProjectedTrackKind::Anchored(anchored.anchored_track.owner_id),
                        origin: anchored.origin,
                        notation: &anchored.anchored_track.notation,
                        horizontal_spacing: &anchored.horizontal_spacing,
                        shunts: &anchored.shunts,
                        stem_directions: &anchored.stem_directions,
                        host_start_tick: anchored.anchored_track.host_span.start_tick,
                        host_end_tick: anchored.anchored_track.host_span.end_tick,
                    });
                }

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

                for anchored in &anchored_render_contexts {
                    draw_ties(
                        &anchored.origin,
                        &anchored.anchored_track.notation,
                        &anchored.stem_directions,
                        &anchored.horizontal_spacing,
                        &anchored.shunts,
                        &tone_offsets,
                        &mut segment_renderer,
                    );
                }

                draw_cross_context_ties(&tie_contexts, &tone_offsets, &mut segment_renderer);

                draw_notation_symbols(
                    &Point {
                        x: stave_left,
                        y: stave_top,
                    },
                    notation,
                    &measures,
                    &horizontal_spacing,
                    shunts,
                    accidentals,
                    stem_lengths,
                    stem_directions,
                    beams,
                    dots,
                    track_articulations,
                    &tone_offsets,
                    state.ui.sequencer.selection.as_ref(),
                    *track_id,
                    *stave_id,
                    flow_id,
                    instrument.id,
                    None,
                    stave_top,
                    &mut segment_renderer,
                );

                for anchored in &anchored_render_contexts {
                    draw_notation_symbols(
                        &anchored.origin,
                        &anchored.anchored_track.notation,
                        &anchored.measures,
                        &anchored.horizontal_spacing,
                        &anchored.shunts,
                        &anchored.accidentals,
                        &anchored.stem_lengths,
                        &anchored.stem_directions,
                        &anchored.beams,
                        &anchored.dots,
                        &anchored.articulations,
                        &tone_offsets,
                        state.ui.sequencer.selection.as_ref(),
                        *track_id,
                        *stave_id,
                        flow_id,
                        instrument.id,
                        Some(anchored.tuplet_id),
                        stave_top,
                        &mut segment_renderer,
                    );

                    draw_tuplet(
                        anchored.tuplet,
                        anchored.anchored_track,
                        &anchored.articulations,
                        &anchored.beams,
                        &anchored.stem_directions,
                        &anchored.stem_lengths,
                        &anchored.horizontal_spacing,
                        anchored.host_left,
                        anchored.host_right,
                        &text_measurer,
                        state.ui.sequencer.selection.as_ref(),
                        flow_id,
                        instrument.id,
                        *stave_id,
                        *track_id,
                        stave_top,
                        &mut segment_renderer,
                    );
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

fn build_render_converter(
    state: &AppContext,
    renderer: &RendererContext,
    layout_id: crate::utils::UniqueId,
) -> Converter {
    let layout = &state.score.layouts.by_id[&layout_id];
    let engrave = &state.score.engraves.by_layout_type[&layout.layout_type];
    Converter::new(renderer.px_per_mm, engrave.space)
}

pub(crate) fn resolve_render_plan_recalculation(
    state: &AppContext,
    renderer: &RendererContext,
    render_plan: &FlowRenderPlan,
) -> FlowRenderPlan {
    let converter = build_render_converter(state, renderer, render_plan.layout_id);
    let composed_flow_ids = render_plan.composed_flow_ids();
    let recalculated_flow_count = render_plan.recalculated_flow_count();

    if recalculated_flow_count == 0 {
        return render_plan.clone();
    }

    if recalculated_flow_count < render_plan.composed_flow_count()
        && !renderer.can_reuse_flow_segments(
            render_plan.layout_id,
            &converter,
            state.ui.renderer.debug,
            &composed_flow_ids,
        )
    {
        return render_plan.with_all_prepared_flows_recalculated();
    }

    render_plan.clone()
}

fn collect_flow_render_segments(
    state: &AppContext,
    renderer: &mut RendererContext,
    render_plan: &FlowRenderPlan,
    converter: &Converter,
) -> Vec<(crate::utils::UniqueId, FlowRenderSegment)> {
    let mut segments_by_flow_id = FxHashMap::default();
    let mut ordered_segments = Vec::new();

    for flow_id in render_plan.composed_flow_ids() {
        let segment = if render_plan.should_recalculate_flow(flow_id) {
            build_flow_render_segment(state, render_plan.layout_id, flow_id, converter)
        } else {
            renderer
                .get_cached_flow_segment(flow_id)
                .unwrap_or_else(|| {
                    build_flow_render_segment(state, render_plan.layout_id, flow_id, converter)
                })
        };

        segments_by_flow_id.insert(flow_id, segment.clone());
        ordered_segments.push((flow_id, segment));
    }

    renderer.store_flow_segments(
        render_plan.layout_id,
        converter,
        state.ui.renderer.debug,
        segments_by_flow_id,
    );

    ordered_segments
}

pub fn generate_render_instructions(
    app: &tauri::AppHandle,
    state: &AppContext,
    renderer: &mut RendererContext,
    render_plan: &FlowRenderPlan,
) {
    let total_start = Instant::now();
    let converter = build_render_converter(state, renderer, render_plan.layout_id);

    let mut timings = RenderPhaseTimings::default();
    let segments = collect_flow_render_segments(state, renderer, render_plan, &converter)
        .into_iter()
        .map(|(_, segment)| {
            timings.add_assign(segment.timings);
            segment
        })
        .collect::<Vec<_>>();
    let composed_surface = compose_render_segments(segments, &converter);

    renderer.instructions.clear();
    renderer.hit_targets.clear();
    renderer.converter = converter.clone();
    renderer.width = composed_surface.width;
    renderer.height = composed_surface.height;
    renderer
        .instructions
        .extend(composed_surface.page_backgrounds);
    renderer.instructions.extend(composed_surface.instructions);
    renderer.hit_targets = composed_surface.hit_targets;

    let _ = app.emit(
        "render-timing-surface-details",
        RenderSurfaceDiagnostics {
            surface_width_px: converter.spaces_to_px(renderer.width),
            surface_height_px: converter.spaces_to_px(renderer.height),
            viewport_width_px: renderer.bounds.2,
            viewport_height_px: renderer.bounds.3,
            page_count: composed_surface.page_widths.len(),
            inter_flow_gap_px: converter.spaces_to_px(composed_surface.inter_flow_gap),
            page_boxes: composed_surface
                .page_offsets
                .iter()
                .copied()
                .zip(composed_surface.page_widths.iter().copied())
                .enumerate()
                .map(
                    |(page_index, (offset, width))| RenderSurfacePageDiagnostics {
                        page_index,
                        x_px: converter.spaces_to_px(offset),
                        width_px: converter.spaces_to_px(width),
                    },
                )
                .collect(),
        },
    );

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
        build_anchored_track_render_context, build_flow_render_segment, build_render_converter,
        collect_flow_render_segments, compose_render_segments, inter_flow_gap_spaces,
        resolve_render_plan_recalculation, AnchoredTrackHostLayout, FlowRenderRecalculationRequest,
        FlowRenderSegment, RenderPhaseTimings,
    };
    use crate::instructions::parse::get_accidentals::get_accidentals_with_anchored_tracks;
    use crate::instructions::parse::get_barlines::get_barlines;
    use crate::instructions::parse::get_beams::get_beams;
    use crate::instructions::parse::get_horizontal_spacing::{
        get_horizontal_spacing_with_anchored_layouts, Slot,
    };
    use crate::instructions::parse::get_measures::get_measures;
    use crate::instructions::parse::get_shunts::get_note_shunts;
    use crate::instructions::parse::get_stem_directions::get_stem_directions;
    use crate::instructions::parse::get_tone_offsets::get_tone_offsets;
    use crate::instructions::parse::get_written_durations::get_written_durations;
    use crate::renderer::converter::Converter;
    use crate::renderer::RendererContext;
    use crate::renderer::{DrawInstruction, HitTarget};
    use crate::score::components::articulation::Articulations;
    use crate::score::components::duration::NoteDuration;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tempo::Tempo;
    use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
    use crate::score::entries::tone::Tone;
    use crate::score::entries::tuplet::{Tuplet, TupletEntry};
    use crate::score::entries::Entry;
    use crate::score::flows::Flow;
    use crate::score::instruments::defs::InstrumentType;
    use crate::score::instruments::Instrument;
    use crate::score::players::{Player, PlayerType};
    use crate::score::tracks::Track;
    use crate::score::Score;
    use crate::ui::sequencer::Selection;
    use crate::ui::Ui;
    use crate::AppContext;
    use rustc_hash::FxHashMap;
    use skia_safe::Color;

    fn make_test_app_context(flow_ids: &[u64]) -> AppContext {
        let instrument_id = 1;
        let player_id = 2;
        let stave_id = 3;

        let mut score = Score::new();
        score.players.order.push(player_id);
        score.players.by_id.insert(
            player_id,
            Player {
                id: player_id,
                player_type: PlayerType::Solo,
                instrument_ids: vec![instrument_id],
            },
        );
        score.instruments.by_id.insert(
            instrument_id,
            Instrument {
                id: instrument_id,
                definition_id: String::from("strings.violin"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Violin"),
                short_name: String::from("Vln."),
                stave_ids: vec![stave_id],
                count: None,
                assigned_plugin_id: None,
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            },
        );

        let mut app = AppContext {
            ui: Ui::new(),
            score,
            preferences: serde_json::from_str(r#"{"audition":true}"#).unwrap(),
            next_id: 100,
            current_file_path: None,
        };

        app.assign_player_to_layout(0, player_id);

        for (index, flow_id) in flow_ids.iter().copied().enumerate() {
            let master_id = app.generate_id();
            let time_signature_id = app.generate_id();
            let tempo_id = app.generate_id();

            let mut master = Track::new(master_id);
            master.insert_entry(Entry::TimeSignature(TimeSignature::new(
                time_signature_id,
                0,
                4,
                NoteDuration::Quarter,
                TimeSignatureDrawType::Regular,
                None,
            )));
            master.insert_entry(Entry::Tempo(Tempo::new(
                tempo_id,
                0,
                String::from("Andante"),
                NoteDuration::Quarter,
                0,
                96,
                false,
                true,
                true,
                false,
            )));
            app.score.tracks.by_id.insert(master_id, master);
            app.score.flows.order.push(flow_id);
            app.score
                .flows
                .by_id
                .insert(flow_id, Flow::new(flow_id, master_id));
            app.assign_flow_to_layout(0, flow_id);
            app.assign_player_to_flow(flow_id, player_id);

            let track_id = app.score.flows.by_id[&flow_id].staves.by_id[&stave_id].track_ids[0];
            let tone_id = app.generate_id();
            app.score
                .tracks
                .by_id
                .get_mut(&track_id)
                .unwrap()
                .insert_entry(Entry::Tone(Tone::new(
                    tone_id,
                    (index as u64) * 32,
                    32,
                    Pitch::new(60 + (index as u8 * 2), Accidental::Natural),
                    Velocity::new(90),
                    Articulations::new(),
                )));
        }

        app.ui.layout_id = Some(0);
        app.ui.flow_id = flow_ids.first().copied();
        app
    }

    fn flow_track_id(app: &AppContext, flow_id: u64) -> u64 {
        app.score.flows.by_id[&flow_id].staves.by_id[&3].track_ids[0]
    }

    fn build_flow_spacing(
        app: &AppContext,
        flow_id: u64,
    ) -> crate::instructions::parse::get_horizontal_spacing::HorizontalSpacing {
        let flow = &app.score.flows.by_id[&flow_id];
        let layout = &app.score.layouts.by_id[&0];
        let engrave = &app.score.engraves.by_layout_type[&layout.layout_type];
        let instruments = app.get_instruments(layout, flow);
        let measures = get_measures(flow, &app.score.tracks);
        let barlines = get_barlines(flow, &app.score.tracks, &measures);
        let tone_offsets = get_tone_offsets(flow, &instruments, &app.score.tracks);
        let notation_by_track = get_written_durations(
            flow,
            &instruments,
            &app.score.tracks,
            &measures,
            &tone_offsets,
        );
        let beams_by_track = get_beams(flow, &notation_by_track, &measures);
        let stem_directions_by_track = get_stem_directions(&notation_by_track, &beams_by_track);
        let shunts_by_track =
            get_note_shunts(&notation_by_track, &tone_offsets, &stem_directions_by_track);
        let (accidentals_by_track, anchored_accidentals_by_track) =
            get_accidentals_with_anchored_tracks(
                flow,
                &app.score.tracks,
                &notation_by_track,
                &barlines,
                &tone_offsets,
                &shunts_by_track,
            );

        get_horizontal_spacing_with_anchored_layouts(
            flow,
            &instruments,
            &app.score.tracks,
            &measures,
            &notation_by_track,
            &barlines,
            &shunts_by_track,
            &beams_by_track,
            &accidentals_by_track,
            &anchored_accidentals_by_track,
            &tone_offsets,
            engrave,
        )
        .0
    }

    fn insert_host_tone(app: &mut AppContext, flow_id: u64, tick: u64, duration: u64) -> u64 {
        let track_id = flow_track_id(app, flow_id);
        let tone_id = app.generate_id();
        app.score
            .tracks
            .by_id
            .get_mut(&track_id)
            .unwrap()
            .insert_entry(Entry::Tone(Tone::new(
                tone_id,
                tick,
                duration,
                Pitch::new(64, Accidental::Natural),
                Velocity::new(90),
                Articulations::new(),
            )));
        tone_id
    }

    fn insert_triplet(app: &mut AppContext, flow_id: u64, tick: u64, local_ticks: &[u64]) -> u64 {
        let track_id = flow_track_id(app, flow_id);
        let tuplet_id = app.generate_id();
        let mut tuplet = Tuplet::new(tuplet_id, tick, 3, 2, NoteDuration::Eighth, vec![3]);

        for local_tick in local_ticks {
            let tone_id = app.generate_id();
            tuplet.entries.insert_entry(TupletEntry::Tone(Tone::new(
                tone_id,
                *local_tick,
                NoteDuration::Eighth.to_ticks(),
                Pitch::new(60, Accidental::Natural),
                Velocity::new(90),
                Articulations::new(),
            )));
        }

        app.score
            .tracks
            .by_id
            .get_mut(&track_id)
            .unwrap()
            .insert_entry(Entry::Tuplet(Box::new(tuplet)));

        tuplet_id
    }

    fn set_first_tuplet_tone_pitch(
        app: &mut AppContext,
        flow_id: u64,
        tuplet_id: u64,
        pitch: Pitch,
    ) {
        let track_id = flow_track_id(app, flow_id);
        let Entry::Tuplet(tuplet) = app
            .score
            .tracks
            .by_id
            .get_mut(&track_id)
            .unwrap()
            .entries
            .by_id
            .get_mut(&tuplet_id)
            .unwrap()
        else {
            panic!("expected tuplet entry")
        };

        let Some(TupletEntry::Tone(tone)) = tuplet.entries.by_id.values_mut().next() else {
            panic!("expected first tuplet tone")
        };

        tone.pitch = pitch;
    }

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
        assert_eq!(composed.page_offsets, vec![0.0, 25.0]);
        assert_eq!(composed.page_widths, vec![5.0, 4.0]);
        assert_eq!(composed.inter_flow_gap, 20.0);
        assert_eq!(composed.page_backgrounds.len(), 2);
        assert_eq!(
            composed.page_backgrounds[0],
            DrawInstruction::Rectangle {
                x: 0.0,
                y: 0.0,
                width: 5.0,
                height: 3.0,
                color: Color::WHITE,
            }
        );
        assert_eq!(
            composed.page_backgrounds[1],
            DrawInstruction::Rectangle {
                x: 25.0,
                y: 0.0,
                width: 4.0,
                height: 6.0,
                color: Color::WHITE,
            }
        );

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

    #[test]
    fn compose_render_segments_offsets_all_segments_and_tracks_page_boxes() {
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
            FlowRenderSegment {
                width: 3.0,
                height: 4.0,
                instructions: vec![DrawInstruction::Rectangle {
                    x: 0.0,
                    y: 1.0,
                    width: 3.0,
                    height: 2.0,
                    color: Color::BLACK,
                }],
                hit_targets: vec![HitTarget {
                    entry: Selection::Tempo {
                        flow_id: 30,
                        tempo_id: 3,
                        tick: 64,
                    },
                    x: 0.25,
                    y: 1.25,
                    width: 1.0,
                    height: 1.0,
                    popover_x: 0.25,
                    popover_y: 0.75,
                }],
                timings: RenderPhaseTimings::default(),
            },
        ];

        let composed = compose_render_segments(segments, &converter);

        assert_eq!(composed.width, 52.0);
        assert_eq!(composed.height, 6.0);
        assert_eq!(composed.page_offsets, vec![0.0, 25.0, 49.0]);
        assert_eq!(composed.page_widths, vec![5.0, 4.0, 3.0]);
        assert_eq!(composed.inter_flow_gap, 20.0);
        assert_eq!(composed.page_backgrounds.len(), 3);
        assert_eq!(
            composed.page_backgrounds[2],
            DrawInstruction::Rectangle {
                x: 49.0,
                y: 0.0,
                width: 3.0,
                height: 4.0,
                color: Color::WHITE,
            }
        );
        assert_eq!(
            composed.instructions[2],
            DrawInstruction::Rectangle {
                x: 49.0,
                y: 1.0,
                width: 3.0,
                height: 2.0,
                color: Color::BLACK,
            }
        );
        assert_eq!(composed.hit_targets[2].x, 49.25);
        assert_eq!(composed.hit_targets[2].popover_x, 49.25);
    }

    #[test]
    fn single_flow_composition_preserves_real_rendered_notation_and_hit_targets() {
        let app = make_test_app_context(&[10]);
        let converter = Converter::new(1.0, 1.75);

        let segment = build_flow_render_segment(&app, 0, 10, &converter);
        assert!(segment.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text == "Violin")
        }));
        assert!(!segment.hit_targets.is_empty());

        let original_width = segment.width;
        let original_height = segment.height;
        let original_instructions = segment.instructions.clone();
        let original_hit_targets = segment.hit_targets.clone();

        let composed = compose_render_segments(vec![segment], &converter);

        assert_eq!(composed.width, original_width);
        assert_eq!(composed.height, original_height);
        assert_eq!(composed.page_offsets, vec![0.0]);
        assert_eq!(composed.page_widths, vec![original_width]);
        assert_eq!(composed.inter_flow_gap, 0.0);
        assert_eq!(composed.instructions, original_instructions);
        assert_eq!(composed.hit_targets, original_hit_targets);
        assert_eq!(composed.page_backgrounds.len(), 1);
    }

    #[test]
    fn second_flow_keeps_page_start_chrome_after_composition_offset() {
        let app = make_test_app_context(&[10, 20]);
        let converter = Converter::new(1.0, 1.75);

        let first_segment = build_flow_render_segment(&app, 0, 10, &converter);
        let second_segment = build_flow_render_segment(&app, 0, 20, &converter);
        let second_name_instruction = second_segment
            .instructions
            .iter()
            .find(|instruction| {
                matches!(instruction, DrawInstruction::Text { text, .. } if text == "Violin")
            })
            .cloned()
            .expect("second segment should keep its instrument name as page-start chrome");
        let second_tempo_target = second_segment
            .hit_targets
            .iter()
            .find(|target| matches!(target.entry, Selection::Tempo { flow_id: 20, .. }))
            .cloned()
            .expect("second segment should keep its page-start tempo hit target");

        let second_offset_spaces = first_segment.width + inter_flow_gap_spaces(&converter);
        let second_offset_px = converter.spaces_to_px(second_offset_spaces);

        let composed = compose_render_segments(vec![first_segment, second_segment], &converter);

        assert!(composed
            .instructions
            .contains(&second_name_instruction.offset(second_offset_px, 0.0)));
        assert!(composed
            .hit_targets
            .contains(&second_tempo_target.offset(second_offset_spaces, 0.0)));
    }

    #[test]
    fn second_flow_preserves_its_rendered_notation_when_composed_into_a_shared_surface() {
        let app = make_test_app_context(&[10, 20]);
        let converter = Converter::new(1.0, 1.75);

        let first_segment = build_flow_render_segment(&app, 0, 10, &converter);
        let second_segment = build_flow_render_segment(&app, 0, 20, &converter);
        let second_instructions = second_segment.instructions.clone();
        let second_hit_targets = second_segment.hit_targets.clone();

        let second_offset_spaces = first_segment.width + inter_flow_gap_spaces(&converter);
        let second_offset_px = converter.spaces_to_px(second_offset_spaces);

        let composed = compose_render_segments(vec![first_segment, second_segment], &converter);

        assert!(second_instructions.iter().all(|instruction| composed
            .instructions
            .contains(&instruction.clone().offset(second_offset_px, 0.0))));
        assert!(second_hit_targets.iter().all(|target| composed
            .hit_targets
            .contains(&target.clone().offset(second_offset_spaces, 0.0))));
    }

    #[test]
    fn compose_render_segments_preserves_flow_owned_selection_payloads() {
        let converter = Converter::new(1.0, 1.0);
        let segments = vec![
            FlowRenderSegment {
                width: 5.0,
                height: 3.0,
                instructions: Vec::new(),
                hit_targets: vec![HitTarget {
                    entry: Selection::Tone {
                        flow_id: 10,
                        track_id: 100,
                        stave_id: 1000,
                        instrument_id: 10000,
                        tone_id: 1,
                        tuplet_id: None,
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
                height: 3.0,
                instructions: Vec::new(),
                hit_targets: vec![HitTarget {
                    entry: Selection::Rest {
                        flow_id: 20,
                        track_id: 200,
                        stave_id: 2000,
                        instrument_id: 20000,
                        tick: 32,
                        tuplet_id: None,
                    },
                    x: 0.5,
                    y: 1.0,
                    width: 1.0,
                    height: 1.0,
                    popover_x: 0.5,
                    popover_y: 0.5,
                }],
                timings: RenderPhaseTimings::default(),
            },
            FlowRenderSegment {
                width: 3.0,
                height: 3.0,
                instructions: Vec::new(),
                hit_targets: vec![HitTarget {
                    entry: Selection::Tempo {
                        flow_id: 30,
                        tempo_id: 3,
                        tick: 64,
                    },
                    x: 0.25,
                    y: 1.0,
                    width: 1.0,
                    height: 1.0,
                    popover_x: 0.25,
                    popover_y: 0.5,
                }],
                timings: RenderPhaseTimings::default(),
            },
        ];

        let composed = compose_render_segments(segments, &converter);

        assert_eq!(composed.hit_targets.len(), 3);
        assert_eq!(
            composed.hit_targets[0].entry,
            Selection::Tone {
                flow_id: 10,
                track_id: 100,
                stave_id: 1000,
                instrument_id: 10000,
                tone_id: 1,
                tuplet_id: None,
            }
        );
        assert_eq!(
            composed.hit_targets[1].entry,
            Selection::Rest {
                flow_id: 20,
                track_id: 200,
                stave_id: 2000,
                instrument_id: 20000,
                tick: 32,
                tuplet_id: None,
            }
        );
        assert_eq!(
            composed.hit_targets[2].entry,
            Selection::Tempo {
                flow_id: 30,
                tempo_id: 3,
                tick: 64,
            }
        );
        assert_eq!(composed.hit_targets[0].x, 1.0);
        assert_eq!(composed.hit_targets[1].x, 25.5);
        assert_eq!(composed.hit_targets[2].x, 49.25);
    }

    #[test]
    fn build_flow_render_segment_emits_tuplet_number_and_native_hit_target() {
        let mut app = make_test_app_context(&[10]);
        let tuplet_id = insert_triplet(&mut app, 10, 32, &[0, 16, 32]);
        let converter = Converter::new(1.0, 1.75);

        let segment = build_flow_render_segment(&app, 0, 10, &converter);

        assert!(segment.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text == "3")
        }));
        assert!(segment.hit_targets.iter().any(|target| {
            matches!(
                target.entry,
                Selection::Tuplet {
                    flow_id: 10,
                    track_id,
                    stave_id: 3,
                    instrument_id: 1,
                    tuplet_id: selected_tuplet_id,
                    tick: 32,
                } if track_id == flow_track_id(&app, 10) && selected_tuplet_id == tuplet_id
            )
        }));
    }

    #[test]
    fn build_flow_render_segment_keeps_boundary_tuplet_hit_target_between_surrounding_notation() {
        let mut app = make_test_app_context(&[10]);
        insert_triplet(&mut app, 10, 32, &[]);
        insert_host_tone(&mut app, 10, 64, 32);
        let converter = Converter::new(1.0, 1.75);

        let segment = build_flow_render_segment(&app, 0, 10, &converter);

        let tuplet_target = segment
            .hit_targets
            .iter()
            .find(|target| matches!(target.entry, Selection::Tuplet { flow_id: 10, .. }))
            .expect("tuplet hit target should be emitted");
        let tone_xs = segment
            .hit_targets
            .iter()
            .filter_map(|target| match target.entry {
                Selection::Tone { flow_id: 10, .. } => Some(target.x),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(tone_xs.len() >= 2);
        let left_tone_x = tone_xs.iter().copied().fold(f32::INFINITY, f32::min);
        let right_tone_x = tone_xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        assert!(tuplet_target.x > left_tone_x);
        assert!(tuplet_target.x < right_tone_x);
    }

    #[test]
    fn build_flow_render_segment_emits_tuplet_local_rest_hit_targets_for_empty_triplet() {
        let mut app = make_test_app_context(&[10]);
        let tuplet_id = insert_triplet(&mut app, 10, 32, &[]);
        let converter = Converter::new(1.0, 1.75);

        let segment = build_flow_render_segment(&app, 0, 10, &converter);

        let local_rests = segment
            .hit_targets
            .iter()
            .filter(|target| {
                matches!(
                    target.entry,
                    Selection::Rest {
                        flow_id: 10,
                        tuplet_id: Some(selected_tuplet_id),
                        ..
                    } if selected_tuplet_id == tuplet_id
                )
            })
            .count();

        assert!(local_rests >= 3);
    }

    #[test]
    fn build_flow_render_segment_emits_tuplet_local_notes_and_beam_shapes() {
        let mut app = make_test_app_context(&[10]);
        let tuplet_id = insert_triplet(&mut app, 10, 32, &[0, 16, 32]);
        let converter = Converter::new(1.0, 1.75);

        let segment = build_flow_render_segment(&app, 0, 10, &converter);

        let local_tones = segment
            .hit_targets
            .iter()
            .filter(|target| {
                matches!(
                    target.entry,
                    Selection::Tone {
                        flow_id: 10,
                        tuplet_id: Some(selected_tuplet_id),
                        ..
                    } if selected_tuplet_id == tuplet_id
                )
            })
            .count();
        let beam_shapes = segment
            .instructions
            .iter()
            .filter(|instruction| matches!(instruction, DrawInstruction::Shape { .. }))
            .count();

        assert!(local_tones >= 3);
        assert!(beam_shapes >= 1);
    }

    #[test]
    fn build_flow_render_segment_draws_accidental_for_tuplet_local_tone() {
        let mut app = make_test_app_context(&[10]);
        let tuplet_id = insert_triplet(&mut app, 10, 32, &[0]);
        set_first_tuplet_tone_pitch(&mut app, 10, tuplet_id, Pitch::new(61, Accidental::Sharp));
        let converter = Converter::new(1.0, 1.75);

        let segment = build_flow_render_segment(&app, 0, 10, &converter);
        let sharp = Accidental::Sharp.to_glyph();

        assert!(segment.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text == &sharp)
        }));
    }

    #[test]
    fn build_anchored_track_render_context_keeps_origin_pinned_for_starting_accidental() {
        let mut app = make_test_app_context(&[10]);
        let tuplet_id = insert_triplet(&mut app, 10, 32, &[0]);
        set_first_tuplet_tone_pitch(&mut app, 10, tuplet_id, Pitch::new(61, Accidental::Sharp));
        let flow = &app.score.flows.by_id[&10];
        let layout = &app.score.layouts.by_id[&0];
        let engrave = &app.score.engraves.by_layout_type[&layout.layout_type];
        let instruments = app.get_instruments(layout, flow);
        let measures = get_measures(flow, &app.score.tracks);
        let barlines = get_barlines(flow, &app.score.tracks, &measures);
        let tone_offsets = get_tone_offsets(flow, &instruments, &app.score.tracks);
        let notation_by_track = get_written_durations(
            flow,
            &instruments,
            &app.score.tracks,
            &measures,
            &tone_offsets,
        );
        let beams_by_track = get_beams(flow, &notation_by_track, &measures);
        let stem_directions_by_track = get_stem_directions(&notation_by_track, &beams_by_track);
        let shunts_by_track =
            get_note_shunts(&notation_by_track, &tone_offsets, &stem_directions_by_track);
        let (accidentals_by_track, anchored_accidentals_by_track) =
            get_accidentals_with_anchored_tracks(
                flow,
                &app.score.tracks,
                &notation_by_track,
                &barlines,
                &tone_offsets,
                &shunts_by_track,
            );
        let (horizontal_spacing, anchored_layouts_by_track) =
            get_horizontal_spacing_with_anchored_layouts(
                flow,
                &instruments,
                &app.score.tracks,
                &measures,
                &notation_by_track,
                &barlines,
                &shunts_by_track,
                &beams_by_track,
                &accidentals_by_track,
                &anchored_accidentals_by_track,
                &tone_offsets,
                engrave,
            );

        let track_id = flow_track_id(&app, 10);
        let track_master = &app.score.tracks.by_id[&track_id];
        let notation = &notation_by_track[&track_id];
        let anchored_track = notation
            .anchored_tracks
            .iter()
            .find(|anchored| anchored.owner_id == tuplet_id)
            .expect("tuplet anchored track should exist");

        let host_start =
            horizontal_spacing.get(anchored_track.host_span.start_tick, Slot::NoteSpacing);
        let host_end_tick = anchored_track.host_span.end_tick.saturating_sub(1);
        let host_end = horizontal_spacing.get(host_end_tick, Slot::NoteSpacing);
        let layout = anchored_layouts_by_track[&track_id]
            .get(&tuplet_id)
            .expect("anchored layout should exist");
        let render_context = build_anchored_track_render_context(
            track_master
                .entries
                .by_id
                .get(&tuplet_id)
                .and_then(|entry| match entry {
                    Entry::Tuplet(tuplet) => Some(tuplet.as_ref()),
                    _ => None,
                })
                .expect("tuplet entry should exist"),
            anchored_track,
            layout,
            engrave,
            AnchoredTrackHostLayout {
                left: host_start.x,
                right: host_end.x + host_end.width,
                stave_top: 0.0,
            },
        );

        assert!(
            (render_context.origin.x - host_start.x).abs() < 1e-6,
            "expected the anchored tuplet origin to stay pinned to the host start"
        );
        assert!(
            render_context.horizontal_spacing.get(0, Slot::NoteSpacing).x.abs() < 1e-6,
            "expected the first tuplet notehead to start at the local origin once host spacing owns the lead-in"
        );
    }

    #[test]
    fn build_flow_spacing_keeps_previous_host_spacing_when_natural_space_is_enough() {
        let mut base = make_test_app_context(&[10]);
        insert_triplet(&mut base, 10, 32, &[0]);

        let mut with_accidental = make_test_app_context(&[10]);
        let tuplet_id = insert_triplet(&mut with_accidental, 10, 32, &[0]);
        set_first_tuplet_tone_pitch(
            &mut with_accidental,
            10,
            tuplet_id,
            Pitch::new(61, Accidental::Sharp),
        );

        let base_spacing = build_flow_spacing(&base, 10);
        let accidental_spacing = build_flow_spacing(&with_accidental, 10);

        assert!(
            (accidental_spacing.get(31, Slot::NoteSpacing).width
                - base_spacing.get(31, Slot::NoteSpacing).width)
                .abs()
                < 1e-6,
            "expected the preceding host note spacing to stay unchanged when its natural space already covers the tuplet lead-in"
        );
    }

    #[test]
    fn build_flow_spacing_grows_previous_host_spacing_when_tuples_lead_in_exceeds_natural_space() {
        let mut base = make_test_app_context(&[10]);
        let track_id = flow_track_id(&base, 10);
        let initial_ids = base.score.tracks.by_id[&track_id]
            .entries
            .by_tick
            .get(&0)
            .cloned()
            .unwrap_or_default();
        for entry_id in initial_ids {
            base.score
                .tracks
                .by_id
                .get_mut(&track_id)
                .expect("flow track should exist")
                .remove_entry(entry_id);
        }
        insert_host_tone(&mut base, 10, 16, 16);
        insert_triplet(&mut base, 10, 32, &[0]);

        let mut with_accidental = base.clone();
        let tuplet_id = with_accidental.score.tracks.by_id[&track_id]
            .entries
            .by_tick
            .get(&32)
            .and_then(|ids| ids.first().copied())
            .expect("triplet should be inserted at tick 32");
        set_first_tuplet_tone_pitch(
            &mut with_accidental,
            10,
            tuplet_id,
            Pitch::new(61, Accidental::Sharp),
        );

        let base_spacing = build_flow_spacing(&base, 10);
        let accidental_spacing = build_flow_spacing(&with_accidental, 10);

        assert!(
            accidental_spacing.get(31, Slot::NoteSpacing).width
                > base_spacing.get(31, Slot::NoteSpacing).width,
            "expected the preceding shorter host note spacing to grow when the tuplet lead-in exceeds its natural space"
        );
    }

    #[test]
    fn build_flow_spacing_matches_continuous_next_notation_floor_for_tuplet_start_accidental() {
        let mut continuous = make_test_app_context(&[10]);
        let track_id = flow_track_id(&continuous, 10);
        let initial_ids = continuous.score.tracks.by_id[&track_id]
            .entries
            .by_tick
            .get(&0)
            .cloned()
            .unwrap_or_default();
        for entry_id in initial_ids {
            continuous
                .score
                .tracks
                .by_id
                .get_mut(&track_id)
                .expect("flow track should exist")
                .remove_entry(entry_id);
        }
        insert_host_tone(&mut continuous, 10, 16, 16);
        let next_tone_id = insert_host_tone(&mut continuous, 10, 32, 16);
        if let Some(Entry::Tone(tone)) = continuous
            .score
            .tracks
            .by_id
            .get_mut(&track_id)
            .and_then(|track| track.entries.by_id.get_mut(&next_tone_id))
        {
            tone.pitch = Pitch::new(61, Accidental::Sharp);
        } else {
            panic!("expected host tone at tick 32");
        }

        let mut tuplet_case = make_test_app_context(&[10]);
        let tuplet_track_id = flow_track_id(&tuplet_case, 10);
        let tuplet_initial_ids = tuplet_case.score.tracks.by_id[&tuplet_track_id]
            .entries
            .by_tick
            .get(&0)
            .cloned()
            .unwrap_or_default();
        for entry_id in tuplet_initial_ids {
            tuplet_case
                .score
                .tracks
                .by_id
                .get_mut(&tuplet_track_id)
                .expect("flow track should exist")
                .remove_entry(entry_id);
        }
        insert_host_tone(&mut tuplet_case, 10, 16, 16);
        let tuplet_id = insert_triplet(&mut tuplet_case, 10, 32, &[0]);
        set_first_tuplet_tone_pitch(
            &mut tuplet_case,
            10,
            tuplet_id,
            Pitch::new(61, Accidental::Sharp),
        );

        let continuous_spacing = build_flow_spacing(&continuous, 10);
        let tuplet_spacing = build_flow_spacing(&tuplet_case, 10);

        assert!(
            (continuous_spacing.get(31, Slot::NoteSpacing).width
                - tuplet_spacing.get(31, Slot::NoteSpacing).width)
                .abs()
                < 1e-6,
            "expected tuplet-start accidental spacing to match the continuous next-notation floor"
        );
    }

    #[test]
    fn build_flow_spacing_uses_host_accidental_slot_for_opening_tuplet_accidental() {
        let mut app = make_test_app_context(&[10]);
        let track_id = flow_track_id(&app, 10);
        let initial_ids = app.score.tracks.by_id[&track_id]
            .entries
            .by_tick
            .get(&0)
            .cloned()
            .unwrap_or_default();

        for entry_id in initial_ids {
            app.score
                .tracks
                .by_id
                .get_mut(&track_id)
                .expect("flow track should exist")
                .remove_entry(entry_id);
        }

        let tuplet_id = insert_triplet(&mut app, 10, 0, &[0]);
        set_first_tuplet_tone_pitch(&mut app, 10, tuplet_id, Pitch::new(61, Accidental::Sharp));

        let spacing = build_flow_spacing(&app, 10);

        assert!(
            spacing.get(0, Slot::Accidentals).width > 0.0,
            "expected an opening tuplet accidental to reserve the shared host accidental slot"
        );
    }

    #[test]
    fn build_flow_render_segment_draws_curve_for_host_to_tuplet_tie() {
        let mut app = make_test_app_context(&[10]);
        insert_triplet(&mut app, 10, 32, &[]);
        insert_host_tone(&mut app, 10, 16, 48);
        let converter = Converter::new(1.0, 1.75);

        let segment = build_flow_render_segment(&app, 0, 10, &converter);
        let curve_count = segment
            .instructions
            .iter()
            .filter(|instruction| matches!(instruction, DrawInstruction::Curve { .. }))
            .count();

        assert!(curve_count >= 1);
    }

    #[test]
    fn compose_render_segments_offsets_tuplet_hit_targets_with_shared_surface_gap() {
        let mut app = make_test_app_context(&[10, 20]);
        insert_triplet(&mut app, 20, 64, &[0, 16, 32]);
        let converter = Converter::new(1.0, 1.75);

        let first_segment = build_flow_render_segment(&app, 0, 10, &converter);
        let second_segment = build_flow_render_segment(&app, 0, 20, &converter);
        let tuplet_target = second_segment
            .hit_targets
            .iter()
            .find(|target| matches!(target.entry, Selection::Tuplet { flow_id: 20, .. }))
            .cloned()
            .expect("second segment should keep its tuplet hit target");
        let second_offset_spaces = first_segment.width + inter_flow_gap_spaces(&converter);

        let composed = compose_render_segments(vec![first_segment, second_segment], &converter);

        assert!(composed
            .hit_targets
            .contains(&tuplet_target.offset(second_offset_spaces, 0.0)));
    }

    #[test]
    fn targeted_render_plan_broadens_when_reuse_cache_is_unavailable() {
        let app = make_test_app_context(&[10, 20]);
        let renderer = RendererContext {
            backend: None,
            converter: Converter::new(1.0, 1.75),
            px_per_mm: 1.0,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
            flow_segment_cache: None,
        };
        let plan = crate::instructions::prepare_flow_render_plan(
            &app.score,
            0,
            10,
            FlowRenderRecalculationRequest::Flows(vec![10]),
        )
        .unwrap();

        let resolved = resolve_render_plan_recalculation(&app, &renderer, &plan);

        assert_eq!(resolved.recalculated_flow_ids(), vec![10, 20]);
    }

    #[test]
    fn three_flow_targeted_render_plan_broadens_when_reuse_cache_is_unavailable() {
        let app = make_test_app_context(&[10, 20, 30]);
        let renderer = RendererContext {
            backend: None,
            converter: Converter::new(1.0, 1.75),
            px_per_mm: 1.0,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
            flow_segment_cache: None,
        };
        let plan = crate::instructions::prepare_flow_render_plan(
            &app.score,
            0,
            10,
            FlowRenderRecalculationRequest::Flows(vec![20]),
        )
        .unwrap();

        let resolved = resolve_render_plan_recalculation(&app, &renderer, &plan);

        assert_eq!(resolved.recalculated_flow_ids(), vec![10, 20, 30]);
    }

    #[test]
    fn hidden_target_render_plan_remains_a_noop_when_reuse_cache_is_unavailable() {
        let mut app = make_test_app_context(&[10, 20, 30]);
        let renderer = RendererContext {
            backend: None,
            converter: Converter::new(1.0, 1.75),
            px_per_mm: 1.0,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
            flow_segment_cache: None,
        };
        app.score
            .layouts
            .by_id
            .get_mut(&0)
            .unwrap()
            .flow_ids
            .remove(&30);
        let plan = crate::instructions::prepare_flow_render_plan(
            &app.score,
            0,
            30,
            FlowRenderRecalculationRequest::Flows(vec![30]),
        )
        .unwrap();

        let resolved = resolve_render_plan_recalculation(&app, &renderer, &plan);

        assert!(resolved.recalculated_flow_ids().is_empty());
    }

    #[test]
    fn unaffected_flow_segments_are_reused_during_targeted_recalculation() {
        let mut app = make_test_app_context(&[10, 20]);
        let mut renderer = RendererContext {
            backend: None,
            converter: Converter::new(1.0, 1.75),
            px_per_mm: 1.0,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
            flow_segment_cache: None,
        };
        let converter = build_render_converter(&app, &renderer, 0);
        let initial_flow_10 = build_flow_render_segment(&app, 0, 10, &converter);
        let initial_flow_20 = build_flow_render_segment(&app, 0, 20, &converter);
        let initial_composed = compose_render_segments(
            vec![initial_flow_10.clone(), initial_flow_20.clone()],
            &converter,
        );

        renderer.store_flow_segments(
            0,
            &converter,
            false,
            FxHashMap::from_iter([(10, initial_flow_10.clone()), (20, initial_flow_20.clone())]),
        );

        let flow_20_track_id = app.score.flows.by_id[&20].staves.by_id[&3].track_ids[0];
        let replacement_tone_id = app.generate_id();
        let flow_20_track = app.score.tracks.by_id.get_mut(&flow_20_track_id).unwrap();
        flow_20_track.entries.by_id.clear();
        flow_20_track.entries.by_tick.clear();
        flow_20_track.insert_entry(Entry::Tone(Tone::new(
            replacement_tone_id,
            64,
            32,
            Pitch::new(72, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        let plan = crate::instructions::prepare_flow_render_plan(
            &app.score,
            0,
            10,
            FlowRenderRecalculationRequest::Flows(vec![20]),
        )
        .unwrap();
        let resolved_plan = resolve_render_plan_recalculation(&app, &renderer, &plan);
        let segments =
            collect_flow_render_segments(&app, &mut renderer, &resolved_plan, &converter);
        let recomposed = compose_render_segments(
            segments
                .iter()
                .map(|(_, segment)| segment.clone())
                .collect(),
            &converter,
        );

        assert_eq!(segments[0].0, 10);
        assert_eq!(segments[0].1.instructions, initial_flow_10.instructions);
        assert_eq!(segments[0].1.hit_targets, initial_flow_10.hit_targets);
        assert_eq!(segments[0].1.width, initial_flow_10.width);
        assert_eq!(segments[0].1.height, initial_flow_10.height);
        assert_eq!(segments[1].0, 20);
        assert_ne!(segments[1].1.instructions, initial_flow_20.instructions);
        assert_ne!(segments[1].1.hit_targets, initial_flow_20.hit_targets);
        assert_eq!(recomposed.page_offsets, initial_composed.page_offsets);
        assert_eq!(recomposed.page_widths[0], initial_composed.page_widths[0]);
    }

    #[test]
    fn multi_flow_targeted_render_plan_stays_narrow_when_reuse_cache_is_available() {
        let app = make_test_app_context(&[10, 20, 30]);
        let mut renderer = RendererContext {
            backend: None,
            converter: Converter::new(1.0, 1.75),
            px_per_mm: 1.0,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
            flow_segment_cache: None,
        };
        let converter = build_render_converter(&app, &renderer, 0);

        let cached_segments = FxHashMap::from_iter([
            (10, build_flow_render_segment(&app, 0, 10, &converter)),
            (20, build_flow_render_segment(&app, 0, 20, &converter)),
            (30, build_flow_render_segment(&app, 0, 30, &converter)),
        ]);

        renderer.store_flow_segments(0, &converter, false, cached_segments);
        let plan = crate::instructions::prepare_flow_render_plan(
            &app.score,
            0,
            10,
            FlowRenderRecalculationRequest::Flows(vec![20, 30]),
        )
        .unwrap();

        assert!(renderer.can_reuse_flow_segments(
            plan.layout_id,
            &converter,
            app.ui.renderer.debug,
            &plan.composed_flow_ids(),
        ));
        assert_eq!(plan.recalculated_flow_ids(), vec![20, 30]);
        assert_eq!(
            resolve_render_plan_recalculation(&app, &renderer, &plan).recalculated_flow_ids(),
            vec![20, 30]
        );
    }

    #[test]
    fn single_affected_flow_stays_narrow_in_three_flow_layout_when_reuse_cache_is_available() {
        let app = make_test_app_context(&[10, 20, 30]);
        let mut renderer = RendererContext {
            backend: None,
            converter: Converter::new(1.0, 1.75),
            px_per_mm: 1.0,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
            flow_segment_cache: None,
        };
        let converter = build_render_converter(&app, &renderer, 0);

        let cached_segments = FxHashMap::from_iter([
            (10, build_flow_render_segment(&app, 0, 10, &converter)),
            (20, build_flow_render_segment(&app, 0, 20, &converter)),
            (30, build_flow_render_segment(&app, 0, 30, &converter)),
        ]);

        renderer.store_flow_segments(0, &converter, false, cached_segments);
        let plan = crate::instructions::prepare_flow_render_plan(
            &app.score,
            0,
            10,
            FlowRenderRecalculationRequest::Flows(vec![20]),
        )
        .unwrap();

        assert!(renderer.can_reuse_flow_segments(
            plan.layout_id,
            &converter,
            app.ui.renderer.debug,
            &plan.composed_flow_ids(),
        ));
        assert_eq!(plan.recalculated_flow_ids(), vec![20]);
        assert_eq!(
            resolve_render_plan_recalculation(&app, &renderer, &plan).recalculated_flow_ids(),
            vec![20]
        );
    }
}
