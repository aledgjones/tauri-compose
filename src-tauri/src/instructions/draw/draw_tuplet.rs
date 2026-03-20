use crate::instructions::draw::draw_articulations::tuplet_bracket_clearance;
use crate::instructions::parse::get_articulations::Articulations as TrackArticulations;
use crate::instructions::parse::get_beams::Beams;
use crate::instructions::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::instructions::parse::get_stem_directions::StemDirections;
use crate::instructions::parse::get_stem_lengths::StemLengths;
use crate::instructions::parse::get_written_durations::AnchoredNotationTrack;
use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::duration::{Direction, Tick};
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::tuplet::Tuplet;
use crate::ui::sequencer::Selection;
use crate::utils::UniqueId;
use skia_safe::Color;

const TUPLET_NUMBER_FONT_SIZE: f32 = 1.5;
const TUPLET_BRACKET_NUMBER_GAP: f32 = -0.25;
const TUPLET_BEAM_NUMBER_GAP: f32 = 0.5;
const TUPLET_SIDE_GAP: f32 = 1.35;
const TUPLET_BRACKET_HOOK_HEIGHT: f32 = 0.45;
const TUPLET_BRACKET_GAP_PADDING: f32 = 0.18;
const TUPLET_LINE_WIDTH: f32 = 0.1;
const TUPLET_MIN_HIT_HEIGHT: f32 = 1.2;
const TUPLET_STAVE_OFFSET: f32 = 1.5;

#[derive(Clone, Copy)]
enum TupletSide {
    Above,
    Below,
}

enum TupletLayout {
    Bracketed {
        side: TupletSide,
        anchor_y: f32,
        number_y: f32,
    },
    NumberOnly {
        side: TupletSide,
        number_y: f32,
    },
}

fn is_number_only_tuplet(track: &AnchoredNotationTrack, beams: &Beams) -> bool {
    !beams.is_empty()
        && track
            .notation
            .track
            .values()
            .all(|notation| !notation.is_rest() && notation.has_beam(beams))
}

fn resolve_stem_direction(
    track: &AnchoredNotationTrack,
    stem_directions: &StemDirections,
) -> Direction {
    let mut up_count = 0;
    let mut down_count = 0;
    let mut first_direction = None;

    for (tick, notation) in &track.notation.track {
        if notation.is_rest() {
            continue;
        }

        let Some(direction) = stem_directions.get(tick) else {
            continue;
        };

        if first_direction.is_none() {
            first_direction = Some(*direction);
        }

        match direction {
            Direction::Up => up_count += 1,
            Direction::Down => down_count += 1,
        }
    }

    match up_count.cmp(&down_count) {
        std::cmp::Ordering::Greater => Direction::Up,
        std::cmp::Ordering::Less => Direction::Down,
        std::cmp::Ordering::Equal => first_direction.unwrap_or(Direction::Down),
    }
}

fn notehead_side_anchor_y(
    track: &AnchoredNotationTrack,
    stem_direction: &Direction,
    stave_top: f32,
) -> Option<f32> {
    track
        .notation
        .track
        .values()
        .filter(|notation| !notation.is_rest())
        .filter_map(|notation| notation.info.as_ref())
        .map(|info| match stem_direction {
            Direction::Down => stave_top + (info.highest as f32 / 2.0),
            Direction::Up => stave_top + (info.lowest as f32 / 2.0),
        })
        .reduce(|left, right| match stem_direction {
            Direction::Down => left.min(right),
            Direction::Up => left.max(right),
        })
}

fn beam_side_anchor_y(
    beams: &Beams,
    stem_lengths: &StemLengths,
    host_left: f32,
    center_x: f32,
    stave_top: f32,
) -> Option<f32> {
    let beam = beams.iter().max_by_key(|beam| beam.stop - beam.start)?;
    let start = stem_lengths.get(&beam.start)?;
    let stop = stem_lengths
        .get(&beam.stop)
        .or_else(|| stem_lengths.get(&beam.start))?;
    let start_x = host_left + start.tail.x;
    let stop_x = host_left + stop.tail.x;
    let tail_y = if (stop_x - start_x).abs() < f32::EPSILON {
        (start.tail.y + stop.tail.y) / 2.0
    } else {
        let clamped_center_x = center_x.clamp(start_x.min(stop_x), start_x.max(stop_x));
        let t = (clamped_center_x - start_x) / (stop_x - start_x);
        start.tail.y + ((stop.tail.y - start.tail.y) * t)
    };

    Some(stave_top + tail_y)
}

fn default_anchor_y(stave_top: f32, stem_direction: &Direction, number_only: bool) -> f32 {
    match (number_only, stem_direction) {
        (true, Direction::Up) => stave_top - 3.5,
        (true, Direction::Down) => stave_top + 3.5,
        (false, Direction::Up) => stave_top + 2.0,
        (false, Direction::Down) => stave_top - 1.45,
    }
}

fn bracket_side(stem_direction: &Direction) -> TupletSide {
    match stem_direction {
        Direction::Down => TupletSide::Above,
        Direction::Up => TupletSide::Below,
    }
}

fn number_only_side(stem_direction: &Direction) -> TupletSide {
    match stem_direction {
        Direction::Up => TupletSide::Above,
        Direction::Down => TupletSide::Below,
    }
}

fn clamp_anchor_y_clear_of_stave(anchor_y: f32, side: TupletSide, stave_top: f32) -> f32 {
    match side {
        TupletSide::Above => anchor_y.min(stave_top - TUPLET_STAVE_OFFSET - TUPLET_SIDE_GAP),
        TupletSide::Below => anchor_y.max(stave_top + TUPLET_STAVE_OFFSET + TUPLET_SIDE_GAP),
    }
}

fn clamp_number_y_clear_of_stave(number_y: f32, side: TupletSide, stave_top: f32) -> f32 {
    match side {
        TupletSide::Above => number_y.min(stave_top - TUPLET_STAVE_OFFSET - TUPLET_SIDE_GAP + 0.1),
        TupletSide::Below => number_y.max(stave_top + TUPLET_STAVE_OFFSET + TUPLET_SIDE_GAP + 0.25),
    }
}

fn bracket_number_y_from_anchor(anchor_y: f32, side: TupletSide) -> f32 {
    match side {
        TupletSide::Above => anchor_y - TUPLET_BRACKET_NUMBER_GAP - 0.1,
        TupletSide::Below => anchor_y + TUPLET_BRACKET_NUMBER_GAP + 0.5,
    }
}

fn number_only_y_from_anchor(anchor_y: f32, side: TupletSide) -> f32 {
    match side {
        TupletSide::Above => anchor_y - TUPLET_BEAM_NUMBER_GAP - 0.2,
        TupletSide::Below => anchor_y + TUPLET_BEAM_NUMBER_GAP + 0.5,
    }
}

fn resolve_bracketed_layout(
    anchored_track: &AnchoredNotationTrack,
    articulations: &TrackArticulations,
    stem_direction: &Direction,
    stave_top: f32,
) -> TupletLayout {
    let side = bracket_side(stem_direction);
    let articulation_offset = articulations
        .values()
        .map(tuplet_bracket_clearance)
        .fold(0.0, f32::max);
    let anchor_y = notehead_side_anchor_y(anchored_track, stem_direction, stave_top)
        .map(|notehead_y| match side {
            TupletSide::Above => notehead_y - TUPLET_SIDE_GAP - articulation_offset,
            TupletSide::Below => notehead_y + TUPLET_SIDE_GAP + articulation_offset,
        })
        .unwrap_or_else(|| default_anchor_y(stave_top, stem_direction, false));
    let anchor_y = clamp_anchor_y_clear_of_stave(anchor_y, side, stave_top);
    let number_y = bracket_number_y_from_anchor(anchor_y, side);

    TupletLayout::Bracketed {
        side,
        anchor_y,
        number_y,
    }
}

fn resolve_number_only_layout(
    beams: &Beams,
    stem_lengths: &StemLengths,
    stem_direction: &Direction,
    host_left: f32,
    center_x: f32,
    stave_top: f32,
) -> TupletLayout {
    let side = number_only_side(stem_direction);
    let anchor_y = beam_side_anchor_y(beams, stem_lengths, host_left, center_x, stave_top)
        .unwrap_or_else(|| default_anchor_y(stave_top, stem_direction, true));
    let number_y = number_only_y_from_anchor(anchor_y, side);
    let number_y = clamp_number_y_clear_of_stave(number_y, side, stave_top);

    TupletLayout::NumberOnly { side, number_y }
}

fn tuplet_font() -> Font {
    Font {
        family: FontFamily::LibreBaskerville,
        size: TUPLET_NUMBER_FONT_SIZE,
        justify: Justify::Middle,
        align: Align::Middle,
    }
}

fn tuplet_visual_bounds(
    track: &AnchoredNotationTrack,
    horizontal_spacing: &HorizontalSpacing,
    host_left: f32,
    host_right: f32,
) -> (f32, f32) {
    if track.notation.track.is_empty() {
        return (host_left, host_right);
    }

    let local_length = track.notation.length.max(1);
    let first_tick = track.notation.track.keys().copied().min().unwrap_or(0);
    let left = map_local_tick_to_host_x(
        first_tick,
        local_length,
        horizontal_spacing,
        host_left,
        host_right,
    );

    let (last_tick, last_notation) = track
        .notation
        .track
        .iter()
        .max_by_key(|(tick, _)| *tick)
        .expect("non-empty notation track should have a last entry");
    let last_x = map_local_tick_to_host_x(
        *last_tick,
        local_length,
        horizontal_spacing,
        host_left,
        host_right,
    );
    let right = (last_x + last_notation.glyph_metrics().total_width()).clamp(host_left, host_right);

    (left, right.max(left))
}

fn map_local_tick_to_host_x(
    local_tick: Tick,
    local_length: Tick,
    horizontal_spacing: &HorizontalSpacing,
    host_left: f32,
    host_right: f32,
) -> f32 {
    if host_right <= host_left || local_length == 0 {
        return host_left;
    }

    let local_x = if local_tick >= local_length {
        horizontal_spacing.width
    } else {
        horizontal_spacing.get(local_tick, Slot::NoteSpacing).x
    };

    (host_left + local_x).clamp(host_left, host_right)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_tuplet(
    tuplet: &Tuplet,
    anchored_track: &AnchoredNotationTrack,
    articulations: &TrackArticulations,
    beams: &Beams,
    stem_directions: &StemDirections,
    stem_lengths: &StemLengths,
    horizontal_spacing: &HorizontalSpacing,
    host_left: f32,
    host_right: f32,
    text_measurer: &TextMeasurer<'_>,
    selection: Option<&Selection>,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_id: UniqueId,
    track_id: UniqueId,
    stave_top: f32,
    renderer: &mut RendererContext,
) {
    if host_right <= host_left {
        return;
    }

    let (content_left, content_right) =
        tuplet_visual_bounds(anchored_track, horizontal_spacing, host_left, host_right);
    let center_x = (content_left + content_right) / 2.0;
    let number_text = tuplet.note_count.to_string();
    let color = if matches!(
        selection,
        Some(Selection::Tuplet {
            flow_id: selected_flow_id,
            track_id: selected_track_id,
            tuplet_id: selected_tuplet_id,
            ..
        }) if *selected_flow_id == flow_id
            && *selected_track_id == track_id
            && *selected_tuplet_id == anchored_track.owner_id
    ) {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };

    let font = tuplet_font();
    let number_width = text_measurer.measure(&number_text, &font).width;
    let bracket_left = content_left.max(host_left);
    let bracket_right = content_right.min(host_right);
    let gap_half_width = (number_width / 2.0) + TUPLET_BRACKET_GAP_PADDING;
    let left_gap_edge = (center_x - gap_half_width).clamp(bracket_left, bracket_right);
    let right_gap_edge = (center_x + gap_half_width).clamp(bracket_left, bracket_right);
    let stem_direction = resolve_stem_direction(anchored_track, stem_directions);
    let layout = if is_number_only_tuplet(anchored_track, beams) {
        resolve_number_only_layout(
            beams,
            stem_lengths,
            &stem_direction,
            host_left,
            center_x,
            stave_top,
        )
    } else {
        resolve_bracketed_layout(anchored_track, articulations, &stem_direction, stave_top)
    };

    let draw_bracket = matches!(layout, TupletLayout::Bracketed { .. });
    let side = match layout {
        TupletLayout::Bracketed { side, .. } | TupletLayout::NumberOnly { side, .. } => side,
    };
    let number_y = match layout {
        TupletLayout::Bracketed { number_y, .. } | TupletLayout::NumberOnly { number_y, .. } => {
            number_y
        }
    };

    renderer.text(
        &number_text,
        &font,
        &Point {
            x: center_x,
            y: number_y,
        },
        color,
    );

    if let TupletLayout::Bracketed { anchor_y, .. } = layout {
        let hook_end_y = match side {
            TupletSide::Above => anchor_y + TUPLET_BRACKET_HOOK_HEIGHT,
            TupletSide::Below => anchor_y - TUPLET_BRACKET_HOOK_HEIGHT,
        };

        if left_gap_edge > bracket_left {
            renderer.line(
                TUPLET_LINE_WIDTH,
                color,
                vec![
                    Point {
                        x: bracket_left - (TUPLET_LINE_WIDTH / 2.0),
                        y: anchor_y,
                    },
                    Point {
                        x: left_gap_edge,
                        y: anchor_y,
                    },
                ],
            );
        }

        if bracket_right > right_gap_edge {
            renderer.line(
                TUPLET_LINE_WIDTH,
                color,
                vec![
                    Point {
                        x: right_gap_edge,
                        y: anchor_y,
                    },
                    Point {
                        x: bracket_right + (TUPLET_LINE_WIDTH / 2.0),
                        y: anchor_y,
                    },
                ],
            );
        }

        renderer.line(
            TUPLET_LINE_WIDTH,
            color,
            vec![
                Point {
                    x: bracket_left,
                    y: anchor_y,
                },
                Point {
                    x: bracket_left,
                    y: hook_end_y,
                },
            ],
        );
        renderer.line(
            TUPLET_LINE_WIDTH,
            color,
            vec![
                Point {
                    x: bracket_right,
                    y: anchor_y,
                },
                Point {
                    x: bracket_right,
                    y: hook_end_y,
                },
            ],
        );
    }

    let (hit_top, hit_bottom) = match layout {
        TupletLayout::Bracketed { anchor_y, .. } => {
            let hook_end_y = match side {
                TupletSide::Above => anchor_y + TUPLET_BRACKET_HOOK_HEIGHT,
                TupletSide::Below => anchor_y - TUPLET_BRACKET_HOOK_HEIGHT,
            };
            (
                (number_y - TUPLET_NUMBER_FONT_SIZE).min(anchor_y.min(hook_end_y)) + 0.6,
                (number_y + 0.25).max(anchor_y.max(hook_end_y)) + 0.25,
            )
        }
        TupletLayout::NumberOnly { .. } => {
            (number_y - TUPLET_NUMBER_FONT_SIZE + 0.6, number_y + 0.5)
        }
    };
    let hit_left = if draw_bracket {
        bracket_left - (TUPLET_LINE_WIDTH / 2.0)
    } else {
        center_x - (number_width / 2.0)
    };
    let hit_right = if draw_bracket {
        bracket_right + (TUPLET_LINE_WIDTH / 2.0)
    } else {
        center_x + (number_width / 2.0)
    };

    renderer.hit_targets.push(HitTarget {
        entry: Selection::Tuplet {
            flow_id,
            instrument_id,
            stave_id,
            track_id,
            tuplet_id: anchored_track.owner_id,
            tick: anchored_track.host_span.start_tick,
        },
        x: hit_left,
        y: hit_top,
        width: (hit_right - hit_left).max(number_width),
        height: (hit_bottom - hit_top).max(TUPLET_MIN_HIT_HEIGHT),
        popover_x: center_x,
        popover_y: if matches!(side, TupletSide::Above) {
            hit_top - 0.5
        } else {
            hit_bottom + 0.5
        },
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_articulations::Articulations as TrackArticulations;
    use crate::instructions::parse::get_beams::{Beam, BeamSegments, BeamTicks, FractionalBeams};
    use crate::instructions::parse::get_horizontal_spacing::{
        stretch_spacing_to_width, stretch_spacing_to_width_using_local_spacing,
    };
    use crate::instructions::parse::get_stem_directions::StemDirections;
    use crate::instructions::parse::get_stem_lengths::{StemDef, StemLengths};
    use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
    use crate::instructions::parse::get_written_durations::{HostSpan, Notation, NotationTrack};
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::components::articulation::{Articulation, Articulations};
    use crate::score::components::duration::{Direction, NoteDuration};
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tone::Tone;
    use rustc_hash::FxHashMap;

    fn make_tone(id: u64, tick: u64, duration: u64) -> Tone {
        Tone::new(
            id,
            tick,
            duration,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )
    }

    fn make_local_track(entries: &[(u64, u64, bool)]) -> NotationTrack {
        let mut track =
            NotationTrack::new(NoteDuration::Quarter.to_ticks() + NoteDuration::Eighth.to_ticks());
        for (index, (tick, duration, is_rest)) in entries.iter().enumerate() {
            let mut notation = Notation::new(*tick, *duration);
            if !is_rest {
                notation
                    .tones
                    .push(make_tone(index as u64 + 1, *tick, *duration));
                notation.info = Some(ToneOffsetInfo {
                    highest: -2,
                    lowest: 2,
                    furthest: 2,
                });
            }
            track.track.insert(*tick, notation);
        }
        track
    }

    fn make_anchored_track(entries: &[(u64, u64, bool)]) -> AnchoredNotationTrack {
        AnchoredNotationTrack {
            owner_id: 55,
            host_span: HostSpan {
                start_tick: 32,
                end_tick: 64,
            },
            notation: make_local_track(entries),
        }
    }

    fn make_triplet() -> Tuplet {
        Tuplet::new(55, 32, 3, 2, NoteDuration::Eighth, vec![3])
    }

    fn make_beam(start: Tick, stop: Tick) -> Beam {
        let mut ticks = BeamTicks::default();
        let mut tick = start;
        while tick <= stop {
            ticks.insert(tick, 1);
            tick += NoteDuration::Eighth.to_ticks();
        }

        Beam {
            ticks,
            start,
            stop,
            beam_segments: BeamSegments::default(),
            fractional_beams: FractionalBeams::default(),
        }
    }

    fn make_uniform_spacing(length: Tick, width: f32) -> HorizontalSpacing {
        let mut spacing = HorizontalSpacing::new(length.max(1));
        let per_tick = width / length.max(1) as f32;
        for tick in 0..length.max(1) {
            spacing.set_width(tick, Slot::NoteSpacing, per_tick);
        }
        stretch_spacing_to_width(&mut spacing, width);
        spacing
    }

    fn make_proportional_spacing(track: &AnchoredNotationTrack, width: f32) -> HorizontalSpacing {
        let mut spacing = HorizontalSpacing::new(track.notation.length.max(1));
        let per_tick = width / track.notation.length.max(1) as f32;
        for tick in 0..track.notation.length.max(1) {
            spacing.set_width(tick, Slot::NoteSpacing, per_tick);
        }
        stretch_spacing_to_width_using_local_spacing(&mut spacing, width);
        spacing
    }

    fn make_stem_directions(direction: Direction, ticks: &[Tick]) -> StemDirections {
        ticks.iter().map(|tick| (*tick, direction)).collect()
    }

    fn make_stem_lengths(values: &[(Tick, f32, f32, f32)]) -> StemLengths {
        let mut lengths = FxHashMap::default();
        for (tick, head_y, tail_x, tail_y) in values {
            lengths.insert(
                *tick,
                StemDef {
                    head: Point {
                        x: *tail_x,
                        y: *head_y,
                    },
                    tail: Point {
                        x: *tail_x,
                        y: *tail_y,
                    },
                },
            );
        }
        lengths
    }

    fn tuplet_number_y(renderer: &RendererContext) -> f32 {
        renderer
            .instructions
            .iter()
            .find_map(|instruction| match instruction {
                DrawInstruction::Text { text, point, .. } if text == "3" => Some(point.1),
                _ => None,
            })
            .expect("tuplet number should be drawn")
    }

    fn bracket_horizontal_extents(renderer: &RendererContext) -> Vec<(f32, f32)> {
        let mut horizontal_segments = renderer
            .instructions
            .iter()
            .filter_map(|instruction| match instruction {
                DrawInstruction::Line { points, .. }
                    if points.len() == 2 && (points[0].1 - points[1].1).abs() < f32::EPSILON =>
                {
                    Some((points[0].0.min(points[1].0), points[0].0.max(points[1].0)))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        horizontal_segments.sort_by(|left, right| left.0.total_cmp(&right.0));
        horizontal_segments
    }

    #[test]
    fn draws_number_only_on_beam_side_for_fully_beamed_triplet() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track =
            make_anchored_track(&[(0, 16, false), (16, 16, false), (32, 16, false)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 12.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let beams = vec![make_beam(0, 32)];
        let stem_directions = make_stem_directions(Direction::Up, &[0, 16, 32]);
        let stem_lengths = make_stem_lengths(&[(0, -1.0, 1.0, -4.0), (32, -1.0, 11.0, -4.0)]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &beams,
            &stem_directions,
            &stem_lengths,
            &horizontal_spacing,
            10.0,
            22.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text == "3")
        }));
        let text_y = renderer
            .instructions
            .iter()
            .find_map(|instruction| match instruction {
                DrawInstruction::Text { text, point, .. } if text == "3" => Some(point.1),
                _ => None,
            })
            .expect("tuplet number should be drawn");
        assert_eq!(
            renderer
                .instructions
                .iter()
                .filter(|instruction| matches!(instruction, DrawInstruction::Line { .. }))
                .count(),
            0
        );
        assert!(text_y < 5.0);
    }

    #[test]
    fn draws_bracket_on_notehead_side_for_rest_bearing_triplet() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let mut anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        anchored_track.notation.track.get_mut(&0).unwrap().info = Some(ToneOffsetInfo {
            highest: -3,
            lowest: 1,
            furthest: -3,
        });
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let stem_directions = make_stem_directions(Direction::Down, &[0]);
        let stem_lengths = StemLengths::default();

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &stem_directions,
            &stem_lengths,
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        assert_eq!(
            renderer
                .instructions
                .iter()
                .filter(|instruction| matches!(instruction, DrawInstruction::Line { .. }))
                .count(),
            4
        );
        assert!(tuplet_number_y(&renderer) < 5.65);
    }

    #[test]
    fn emits_native_tuplet_hit_target() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track = make_anchored_track(&[(0, 16, true), (16, 16, true), (32, 16, true)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 12.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &StemDirections::default(),
            &StemLengths::default(),
            &horizontal_spacing,
            5.0,
            17.0,
            &text_measurer,
            Some(&Selection::Tuplet {
                flow_id: 1,
                instrument_id: 2,
                stave_id: 3,
                track_id: 4,
                tuplet_id: 55,
                tick: 32,
            }),
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        assert_eq!(renderer.hit_targets.len(), 1);
        assert!(matches!(
            renderer.hit_targets[0].entry,
            Selection::Tuplet {
                flow_id: 1,
                instrument_id: 2,
                stave_id: 3,
                track_id: 4,
                tuplet_id: 55,
                tick: 32,
            }
        ));
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { color, .. } if color == Color::from_rgb(14, 165, 233)
        ));
    }

    #[test]
    fn centers_bracket_gap_around_number() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &StemDirections::default(),
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        let horizontal_segments = bracket_horizontal_extents(&renderer);

        let number_center = renderer
            .instructions
            .iter()
            .find_map(|instruction| match instruction {
                DrawInstruction::Text { text, point, .. } if text == "3" => Some(point.0),
                _ => None,
            })
            .expect("tuplet number should be drawn");

        assert_eq!(horizontal_segments.len(), 2);
        assert!(horizontal_segments[0].1 < number_center);
        assert!(horizontal_segments[1].0 > number_center);
    }

    #[test]
    fn aligns_bracket_bounds_to_resolved_local_spacing_instead_of_uniform_ticks() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track =
            make_anchored_track(&[(0, 16, false), (16, 16, false), (32, 32, false)]);
        let horizontal_spacing = make_proportional_spacing(&anchored_track, 20.0);

        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &StemDirections::default(),
            &StemLengths::default(),
            &horizontal_spacing,
            10.0,
            30.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        let tuplet_target = renderer
            .hit_targets
            .first()
            .expect("tuplet hit target should exist");
        assert!(tuplet_target.width > 10.0);
        assert!(tuplet_target.x > 9.5);
        assert!(tuplet_target.width < 20.5);
    }

    #[test]
    fn omits_bracket_for_clean_beamed_triplet() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track =
            make_anchored_track(&[(0, 16, false), (16, 16, false), (32, 16, false)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 12.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let beams = vec![make_beam(0, 32)];
        let stem_directions = make_stem_directions(Direction::Down, &[0, 16, 32]);
        let stem_lengths = make_stem_lengths(&[(0, 1.0, 1.0, 4.0), (32, 1.0, 11.0, 4.0)]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &beams,
            &stem_directions,
            &stem_lengths,
            &horizontal_spacing,
            10.0,
            22.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        assert_eq!(
            renderer
                .instructions
                .iter()
                .filter(|instruction| matches!(instruction, DrawInstruction::Line { .. }))
                .count(),
            0
        );
        assert_eq!(renderer.hit_targets.len(), 1);
    }

    #[test]
    fn clamps_number_only_tuplet_with_same_stave_clearance_rule() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track =
            make_anchored_track(&[(0, 16, false), (16, 16, false), (32, 16, false)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 12.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let beams = vec![make_beam(0, 32)];
        let stem_directions = make_stem_directions(Direction::Up, &[0, 16, 32]);
        let stem_lengths = make_stem_lengths(&[(0, -1.0, 1.0, -1.0), (32, -1.0, 11.0, -1.0)]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &beams,
            &stem_directions,
            &stem_lengths,
            &horizontal_spacing,
            10.0,
            22.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        assert!((tuplet_number_y(&renderer) - 5.25).abs() < 1e-6);
    }

    #[test]
    fn places_beamed_number_below_beam_for_stem_down_tuplet() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track =
            make_anchored_track(&[(0, 16, false), (16, 16, false), (32, 16, false)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 12.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let beams = vec![make_beam(0, 32)];
        let stem_directions = make_stem_directions(Direction::Down, &[0, 16, 32]);
        let stem_lengths = make_stem_lengths(&[(0, 1.0, 1.0, 4.0), (32, 1.0, 11.0, 4.0)]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &beams,
            &stem_directions,
            &stem_lengths,
            &horizontal_spacing,
            10.0,
            22.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        assert!(tuplet_number_y(&renderer) > 12.5);
    }

    #[test]
    fn keeps_bracketed_tuplet_clear_of_stave_when_placed_below() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let mut anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        anchored_track.notation.track.get_mut(&0).unwrap().info = Some(ToneOffsetInfo {
            highest: -2,
            lowest: 1,
            furthest: 1,
        });
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let stem_directions = make_stem_directions(Direction::Up, &[0]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        assert!(tuplet_number_y(&renderer) > 11.0);
        assert!(renderer.hit_targets[0].y > 10.0);
    }

    #[test]
    fn extends_bracket_horizontals_to_outer_edge_of_hooks() {
        let mut renderer = make_test_renderer();
        let triplet = make_triplet();
        let anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &StemDirections::default(),
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut renderer,
        );

        let horizontal_segments = bracket_horizontal_extents(&renderer);
        assert_eq!(horizontal_segments.len(), 2);
        assert!((horizontal_segments[0].0 - 5.95).abs() < 1e-6);
        assert!((horizontal_segments[1].1 - 24.05).abs() < 1e-6);
    }

    #[test]
    fn shifts_bracket_and_number_to_clear_articulations() {
        let triplet = make_triplet();
        let anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let mut baseline_renderer = make_test_renderer();
        let converter = baseline_renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let stem_directions = make_stem_directions(Direction::Down, &[0]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut baseline_renderer,
        );
        let baseline_number_y = tuplet_number_y(&baseline_renderer);

        let mut articulations = TrackArticulations::default();
        articulations.insert(
            0,
            [Articulation::Staccato, Articulation::Marcato]
                .into_iter()
                .collect(),
        );

        let mut articulated_renderer = make_test_renderer();

        draw_tuplet(
            &triplet,
            &anchored_track,
            &articulations,
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut articulated_renderer,
        );

        let articulated_number_y = tuplet_number_y(&articulated_renderer);

        assert!(articulated_number_y < baseline_number_y);
        assert!(((baseline_number_y - articulated_number_y) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gives_staccatissimo_extra_bracket_clearance() {
        let triplet = make_triplet();
        let anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let mut baseline_renderer = make_test_renderer();
        let converter = baseline_renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let stem_directions = make_stem_directions(Direction::Down, &[0]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut baseline_renderer,
        );
        let baseline_number_y = tuplet_number_y(&baseline_renderer);

        let mut articulations = TrackArticulations::default();
        articulations.insert(0, [Articulation::Staccatissimo].into_iter().collect());

        let mut articulated_renderer = make_test_renderer();
        draw_tuplet(
            &triplet,
            &anchored_track,
            &articulations,
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut articulated_renderer,
        );

        let articulated_number_y = tuplet_number_y(&articulated_renderer);

        assert!(articulated_number_y < baseline_number_y);
        assert!(((baseline_number_y - articulated_number_y) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn stacks_combined_articulation_clearance_within_a_note() {
        let triplet = make_triplet();
        let anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let mut baseline_renderer = make_test_renderer();
        let converter = baseline_renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let stem_directions = make_stem_directions(Direction::Down, &[0]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut baseline_renderer,
        );
        let baseline_number_y = tuplet_number_y(&baseline_renderer);

        let mut articulations = TrackArticulations::default();
        articulations.insert(
            0,
            [Articulation::Tenuto, Articulation::Marcato]
                .into_iter()
                .collect(),
        );

        let mut articulated_renderer = make_test_renderer();
        draw_tuplet(
            &triplet,
            &anchored_track,
            &articulations,
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut articulated_renderer,
        );

        let articulated_number_y = tuplet_number_y(&articulated_renderer);

        assert!(articulated_number_y < baseline_number_y);
        assert!(((baseline_number_y - articulated_number_y) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn adds_extra_clearance_for_stacked_staccatissimo() {
        let triplet = make_triplet();
        let anchored_track = make_anchored_track(&[(0, 32, false), (32, 16, true)]);
        let horizontal_spacing = make_uniform_spacing(anchored_track.notation.length, 18.0);
        let mut baseline_renderer = make_test_renderer();
        let converter = baseline_renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let stem_directions = make_stem_directions(Direction::Down, &[0]);

        draw_tuplet(
            &triplet,
            &anchored_track,
            &TrackArticulations::default(),
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut baseline_renderer,
        );
        let baseline_number_y = tuplet_number_y(&baseline_renderer);

        let mut articulations = TrackArticulations::default();
        articulations.insert(
            0,
            [Articulation::Staccatissimo, Articulation::Tenuto]
                .into_iter()
                .collect(),
        );

        let mut articulated_renderer = make_test_renderer();
        draw_tuplet(
            &triplet,
            &anchored_track,
            &articulations,
            &vec![],
            &stem_directions,
            &StemLengths::default(),
            &horizontal_spacing,
            6.0,
            24.0,
            &text_measurer,
            None,
            1,
            2,
            3,
            4,
            8.0,
            &mut articulated_renderer,
        );

        let articulated_number_y = tuplet_number_y(&articulated_renderer);

        assert!(articulated_number_y < baseline_number_y);
        assert!(((baseline_number_y - articulated_number_y) - 1.5).abs() < 1e-6);
    }
}
