use super::get_accidentals::{Accidentals, AccidentalsByTrack, AnchoredAccidentalsByTrack};
use super::get_articulations::{get_articulations_in_track, Articulations};
use super::get_barlines::Barlines;
use super::get_beams::{get_beams_in_track, Beams, BeamsByTrack};
use super::get_dots::{get_dots_in_track, Dots};
use super::get_measures::{measures_from_single_time_signature, Measures};
use super::get_shunts::{get_note_shunts_in_track, Shunts, ShuntsByTrack};
use super::get_stem_directions::{get_tuplet_stem_directions_in_track, StemDirections};
use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::{AnchoredNotationTrack, NotationByTrack, NotationTrack};
use crate::renderer::converter::Space;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::components::measurements::BoundingBox;
use crate::score::engraves::Engrave;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::entries::tuplet::Tuplet;
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;

const EMPTY_MEASURE_BASE_WIDTH_MULTIPLIER: f32 = 2.0;
const EMPTY_MEASURE_WIDTH_EXPONENT: f32 = 0.3;
const EMPTY_MEASURE_WIDTH_BLEND: f32 = 0.55;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Slot {
    PaddingStart = 0,
    EndRepeat,
    Clef,
    Barline,
    KeySignature,
    TimeSignature,
    StartRepeat,
    BarPadding,
    Accidentals, // only used if there is no previous space to cut into.
    PreNote,     // only used if there is no previous space to cut into.
    NoteSpacing,
}

impl Slot {
    pub fn count() -> usize {
        Slot::NoteSpacing as usize + 1
    }
}

#[derive(Clone)]
pub struct HorizontalSpacingSpan {
    pub x: Space,
    pub width: Space,
}

impl HorizontalSpacingSpan {
    pub fn new() -> Self {
        Self { width: 0.0, x: 0.0 }
    }
}

#[derive(Clone)]
pub struct HorizontalSpacing {
    pub widths: Vec<HorizontalSpacingSpan>,
    pub width: Space,
}

impl HorizontalSpacing {
    pub fn new(length: Ticks) -> HorizontalSpacing {
        HorizontalSpacing {
            widths: vec![HorizontalSpacingSpan::new(); (length as usize + 1) * Slot::count()],
            width: 0.0,
        }
    }

    pub fn get(&self, tick: Tick, slot: Slot) -> HorizontalSpacingSpan {
        let index = (tick as usize * Slot::count()) + slot as usize;

        self.widths[index].clone()
    }

    pub fn set_width(&mut self, tick: Tick, slot: Slot, width: Space) {
        let index = (tick as usize * Slot::count()) + slot as usize;
        self.widths[index].width = width;
    }

    pub fn add_width(&mut self, tick: Tick, slot: Slot, width: Space) {
        let index = (tick as usize * Slot::count()) + slot as usize;
        self.widths[index].width += width;
    }
}

fn empty_measure_width(time_signature: &TimeSignature, engrave: &Engrave) -> Space {
    let normalized_ticks =
        time_signature.ticks_per_bar() as f32 / NoteDuration::Whole.to_ticks() as f32;
    let compressed_width = normalized_ticks.powf(EMPTY_MEASURE_WIDTH_EXPONENT);
    let blended_width = 1.0 + ((compressed_width - 1.0) * EMPTY_MEASURE_WIDTH_BLEND);

    engrave.base_note_space * EMPTY_MEASURE_BASE_WIDTH_MULTIPLIER * blended_width
}

fn recompute_spacing_positions(spacing: &mut HorizontalSpacing, include_final_barline: bool) {
    let mut x: Space = 0.0;
    for span in &mut spacing.widths {
        span.x = x;
        x += span.width;
    }

    spacing.width = if include_final_barline {
        x + BarlineDrawType::Final.metrics().total_width()
    } else {
        x
    };
}

#[allow(clippy::too_many_arguments)]
fn collect_notation_spacing_updates(
    notation: &NotationTrack,
    anchored_tracks: &[AnchoredNotationTrack],
    anchored_lead_ins: &FxHashMap<UniqueId, InitialNoteLeadIn>,
    barlines: &Barlines,
    shunts: &Shunts,
    beams: &Beams,
    accidentals: &Accidentals,
    engrave: &Engrave,
) -> Vec<(Tick, Slot, Space)> {
    let mut updates: Vec<(Tick, Slot, Space)> = Vec::new();

    for (tick, entry) in &notation.track {
        let mut spacing = entry.metrics(shunts, engrave, beams).width;

        if *tick == 0 || barlines.contains_key(tick) {
            let metrics = entry.glyph_metrics();
            let glyph_width = metrics.total_width();
            let has_pre_shunt = entry.has_pre_shunt(shunts);

            if has_pre_shunt {
                updates.push((*tick, Slot::PreNote, glyph_width));
            }

            if let Some(slots) = accidentals.slots_by_tick.get(tick) {
                let width = if has_pre_shunt {
                    ((*slots as f32 - 1.0) * 1.1) + 0.25
                } else {
                    (*slots as f32 * 1.1) + 0.25
                };
                updates.push((*tick, Slot::Accidentals, width));
            };
        }

        if let Some(next_visible) =
            get_next_visible_notation_at_or_after(notation, anchored_tracks, *tick + 1)
        {
            let next_preamble = match next_visible {
                NextVisibleNotation::Host {
                    tick: next_tick,
                    entry,
                } => {
                    if barlines.contains_key(&next_tick) {
                        None
                    } else {
                        Some(host_notation_preamble(
                            next_tick,
                            entry,
                            shunts,
                            accidentals,
                        ))
                    }
                }
                NextVisibleNotation::Anchored { track } => {
                    if barlines.contains_key(&track.host_span.start_tick) {
                        None
                    } else {
                        anchored_lead_ins
                            .get(&track.owner_id)
                            .copied()
                            .and_then(lead_in_preamble)
                    }
                }
            };

            if let Some(next_preamble) = next_preamble {
                spacing = required_spacing_with_next_notation(
                    entry,
                    spacing,
                    shunts,
                    engrave,
                    beams,
                    next_preamble.accidentals_width,
                    next_preamble.glyph_width,
                    next_preamble.has_pre_shunt,
                );
            }
        };

        let note_spacing_per_tick = spacing / entry.duration as f32;
        let end = tick + entry.duration;
        for i in *tick..end {
            updates.push((i, Slot::NoteSpacing, note_spacing_per_tick));
        }
    }

    updates
}

fn accidental_spacing_width(slot_count: u8, next_has_pre_shunt: bool) -> Space {
    if slot_count == 0 {
        0.0
    } else if next_has_pre_shunt {
        ((slot_count as f32 - 1.0) * 1.1) + 0.25
    } else {
        (slot_count as f32 * 1.1) + 0.25
    }
}

#[derive(Clone, Copy)]
struct NextNotationPreamble {
    accidentals_width: Space,
    glyph_width: Space,
    has_pre_shunt: bool,
}

enum NextVisibleNotation<'a> {
    Host {
        tick: Tick,
        entry: &'a super::get_written_durations::Notation,
    },
    Anchored {
        track: &'a AnchoredNotationTrack,
    },
}

fn get_next_notation_at_or_after(
    notation: &NotationTrack,
    tick: Tick,
) -> Option<(Tick, &super::get_written_durations::Notation)> {
    for current_tick in tick..notation.length {
        if let Some(entry) = notation.track.get(&current_tick) {
            return Some((current_tick, entry));
        }
    }

    None
}

fn get_next_anchored_track_at_or_after(
    anchored_tracks: &[AnchoredNotationTrack],
    tick: Tick,
) -> Option<&AnchoredNotationTrack> {
    anchored_tracks
        .iter()
        .filter(|track| track.host_span.start_tick >= tick)
        .min_by_key(|track| track.host_span.start_tick)
}

fn get_next_visible_notation_at_or_after<'a>(
    notation: &'a NotationTrack,
    anchored_tracks: &'a [AnchoredNotationTrack],
    tick: Tick,
) -> Option<NextVisibleNotation<'a>> {
    let next_host = get_next_notation_at_or_after(notation, tick);
    let next_anchored = get_next_anchored_track_at_or_after(anchored_tracks, tick);

    match (next_host, next_anchored) {
        (Some((host_tick, entry)), Some(anchored_track)) => {
            if anchored_track.host_span.start_tick < host_tick {
                Some(NextVisibleNotation::Anchored {
                    track: anchored_track,
                })
            } else {
                Some(NextVisibleNotation::Host {
                    tick: host_tick,
                    entry,
                })
            }
        }
        (Some((host_tick, entry)), None) => Some(NextVisibleNotation::Host {
            tick: host_tick,
            entry,
        }),
        (None, Some(anchored_track)) => Some(NextVisibleNotation::Anchored {
            track: anchored_track,
        }),
        (None, None) => None,
    }
}

fn host_notation_preamble(
    tick: Tick,
    entry: &super::get_written_durations::Notation,
    shunts: &Shunts,
    accidentals: &Accidentals,
) -> NextNotationPreamble {
    let glyph_width = entry.glyph_metrics().total_width();
    let has_pre_shunt = entry.has_pre_shunt(shunts);
    let accidentals_width = accidental_spacing_width(
        accidentals.slots_by_tick.get(&tick).copied().unwrap_or(0),
        has_pre_shunt,
    );

    NextNotationPreamble {
        accidentals_width,
        glyph_width,
        has_pre_shunt,
    }
}

fn lead_in_preamble(lead_in: InitialNoteLeadIn) -> Option<NextNotationPreamble> {
    if lead_in.total_width <= 0.0 {
        return None;
    }

    Some(NextNotationPreamble {
        accidentals_width: lead_in.accidentals_width,
        glyph_width: lead_in.pre_note_width,
        has_pre_shunt: lead_in.pre_note_width > 0.0,
    })
}

#[allow(clippy::too_many_arguments)]
fn required_spacing_with_next_notation(
    entry: &super::get_written_durations::Notation,
    base_spacing: Space,
    shunts: &Shunts,
    engrave: &Engrave,
    beams: &Beams,
    next_accidentals_width: Space,
    next_glyph_width: Space,
    next_has_pre_shunt: bool,
) -> Space {
    let min = entry.min_spacing(shunts, engrave, beams);
    let pre_shunt_width = if next_has_pre_shunt {
        next_glyph_width
    } else {
        0.0
    };
    let ledger_line_width = match &entry.info {
        Some(info) if info.highest < -5 || info.lowest > 5 => 0.4,
        _ => 0.0,
    };

    let avoid_collisions_spacing =
        next_accidentals_width + pre_shunt_width + min.width + ledger_line_width + 0.2;
    let min_natural_spacing = min.total_width() + pre_shunt_width;

    base_spacing
        .max(min_natural_spacing)
        .max(avoid_collisions_spacing)
}

fn apply_spacing_updates(spacing: &mut HorizontalSpacing, updates: Vec<(Tick, Slot, Space)>) {
    for (tick, slot, width) in updates {
        let current = spacing.get(tick, slot);
        if width > current.width {
            spacing.set_width(tick, slot, width);
        }
    }
}

fn apply_note_spacing_to_range(
    spacing: &mut HorizontalSpacing,
    start_tick: Tick,
    duration: Tick,
    required_spacing: Space,
) {
    if duration == 0 {
        return;
    }

    let note_spacing_per_tick = required_spacing / duration as f32;
    let end_tick = start_tick + duration;
    for tick in start_tick..end_tick {
        let current = spacing.get(tick, Slot::NoteSpacing).width;
        if note_spacing_per_tick > current {
            spacing.set_width(tick, Slot::NoteSpacing, note_spacing_per_tick);
        }
    }
}

#[derive(Clone)]
pub struct AnchoredTrackLayout {
    pub measures: Measures,
    pub horizontal_spacing: HorizontalSpacing,
    pub lead_in: InitialNoteLeadIn,
    pub beams: Beams,
    pub stem_directions: StemDirections,
    pub shunts: Shunts,
    pub accidentals: Accidentals,
    pub dots: Dots,
    pub articulations: Articulations,
}

pub type AnchoredTrackLayouts = FxHashMap<UniqueId, AnchoredTrackLayout>;
pub type AnchoredTrackLayoutsByTrack = FxHashMap<UniqueId, AnchoredTrackLayouts>;

pub fn build_anchored_track_layout(
    tuplet: &Tuplet,
    anchored_track: &AnchoredNotationTrack,
    accidentals: Accidentals,
    tone_offsets: &ToneVerticalOffsets,
    engrave: &Engrave,
) -> AnchoredTrackLayout {
    let measures = measures_from_single_time_signature(tuplet.local_time_signature());
    let barlines = Barlines::default();
    let beams = get_beams_in_track(&anchored_track.notation, &measures);
    let stem_directions = get_tuplet_stem_directions_in_track(&anchored_track.notation, &beams);
    let shunts = get_note_shunts_in_track(&anchored_track.notation, tone_offsets, &stem_directions);
    let dots = get_dots_in_track(&anchored_track.notation, tone_offsets);
    let articulations = get_articulations_in_track(&anchored_track.notation);
    let mut horizontal_spacing = HorizontalSpacing::new(anchored_track.notation.length.max(1));
    let updates = collect_notation_spacing_updates(
        &anchored_track.notation,
        &[],
        &FxHashMap::default(),
        &barlines,
        &shunts,
        &beams,
        &accidentals,
        engrave,
    );
    apply_spacing_updates(&mut horizontal_spacing, updates);
    recompute_spacing_positions(&mut horizontal_spacing, false);
    let lead_in = take_initial_note_lead_in(&mut horizontal_spacing);

    AnchoredTrackLayout {
        measures,
        horizontal_spacing,
        lead_in,
        beams,
        stem_directions,
        shunts,
        accidentals,
        dots,
        articulations,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AnchoredSpacingParticipant {
    track_id: UniqueId,
    owner_id: UniqueId,
    start_tick: Tick,
    end_tick: Tick,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AnchoredSpacingRegion {
    start_tick: Tick,
    end_tick: Tick,
    members: Vec<AnchoredSpacingParticipant>,
}

#[derive(Clone, Debug, PartialEq)]
struct SharedRhythmGrid {
    resolution: Tick,
    slot_widths: Vec<Space>,
    width: Space,
}

impl SharedRhythmGrid {
    fn from_local_spacings(
        host_tick_count: Tick,
        local_spacings: &[&HorizontalSpacing],
    ) -> Option<Self> {
        if host_tick_count == 0 || local_spacings.is_empty() {
            return None;
        }

        let mut resolution = host_tick_count;
        let mut local_tick_counts = Vec::with_capacity(local_spacings.len());

        for spacing in local_spacings {
            let tick_count = (spacing.widths.len() / Slot::count()).saturating_sub(1) as Tick;
            if tick_count == 0 {
                return None;
            }

            resolution = lcm_tick(resolution, tick_count);
            local_tick_counts.push(tick_count);
        }

        if resolution == 0 {
            return None;
        }

        let mut slot_widths = vec![0.0; resolution as usize];
        for (spacing, local_tick_count) in local_spacings.iter().zip(local_tick_counts.into_iter())
        {
            let slots_per_tick = (resolution / local_tick_count) as usize;

            for tick in 0..local_tick_count {
                let source_width = spacing.get(tick, Slot::NoteSpacing).width.max(0.0);
                let projected_width = source_width / slots_per_tick as f32;
                let start = tick as usize * slots_per_tick;
                let end = start + slots_per_tick;

                for slot_width in &mut slot_widths[start..end] {
                    if projected_width > *slot_width {
                        *slot_width = projected_width;
                    }
                }
            }
        }

        let width = slot_widths.iter().sum::<Space>();
        if width <= 0.0 {
            return None;
        }

        Some(Self {
            resolution,
            slot_widths,
            width,
        })
    }

    fn project_widths(&self, target_tick_count: Tick, target_width: Space) -> Vec<Space> {
        if target_tick_count == 0 || self.resolution == 0 {
            return Vec::new();
        }

        let slots_per_tick = (self.resolution / target_tick_count) as usize;
        let scale = if self.width > 0.0 {
            target_width / self.width
        } else {
            1.0
        };

        let mut widths = Vec::with_capacity(target_tick_count as usize);
        for tick in 0..target_tick_count as usize {
            let start = tick * slots_per_tick;
            let end = start + slots_per_tick;
            widths.push(self.slot_widths[start..end].iter().sum::<Space>() * scale);
        }

        widths
    }

    fn project_widths_in_range(
        &self,
        start_unit: Tick,
        unit_count: Tick,
        target_tick_count: Tick,
    ) -> Vec<Space> {
        if target_tick_count == 0 || unit_count == 0 {
            return Vec::new();
        }

        let units_per_tick = unit_count / target_tick_count;
        let mut widths = Vec::with_capacity(target_tick_count as usize);

        for tick in 0..target_tick_count as usize {
            let start = start_unit as usize + (tick * units_per_tick as usize);
            let end = start + units_per_tick as usize;
            widths.push(self.slot_widths[start..end].iter().sum::<Space>());
        }

        widths
    }
}

fn gcd_tick(mut left: Tick, mut right: Tick) -> Tick {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }

    left.max(1)
}

fn lcm_tick(left: Tick, right: Tick) -> Tick {
    if left == 0 || right == 0 {
        0
    } else {
        left / gcd_tick(left, right) * right
    }
}

fn rewrite_note_spacing_from_widths(spacing: &mut HorizontalSpacing, widths: &[Space]) {
    for (tick, width) in widths.iter().enumerate() {
        spacing.set_width(tick as Tick, Slot::NoteSpacing, *width);
    }

    recompute_spacing_positions(spacing, false);
}

fn collect_anchored_spacing_regions(
    notations_by_track: &NotationByTrack,
    anchored_layouts_by_track: &AnchoredTrackLayoutsByTrack,
) -> Vec<AnchoredSpacingRegion> {
    let mut participants = Vec::new();

    for (track_id, notation) in notations_by_track {
        let Some(layouts) = anchored_layouts_by_track.get(track_id) else {
            continue;
        };

        for anchored_track in &notation.anchored_tracks {
            if !layouts.contains_key(&anchored_track.owner_id) {
                continue;
            }

            participants.push(AnchoredSpacingParticipant {
                track_id: *track_id,
                owner_id: anchored_track.owner_id,
                start_tick: anchored_track.host_span.start_tick,
                end_tick: anchored_track.host_span.end_tick,
            });
        }
    }

    participants.sort_by_key(|participant| (participant.start_tick, participant.end_tick));

    let mut regions = Vec::new();
    for participant in participants {
        let should_merge = regions
            .last()
            .is_some_and(|region: &AnchoredSpacingRegion| participant.start_tick < region.end_tick);

        if should_merge {
            let region = regions.last_mut().expect("checked above");
            region.end_tick = region.end_tick.max(participant.end_tick);
            region.members.push(participant);
        } else {
            regions.push(AnchoredSpacingRegion {
                start_tick: participant.start_tick,
                end_tick: participant.end_tick,
                members: vec![participant],
            });
        }
    }

    regions
}

fn build_shared_region_grid(
    spacing: &HorizontalSpacing,
    region: &AnchoredSpacingRegion,
    anchored_layouts_by_track: &AnchoredTrackLayoutsByTrack,
) -> Option<(SharedRhythmGrid, Tick)> {
    let region_length = region.end_tick.saturating_sub(region.start_tick);
    if region_length == 0 {
        return None;
    }

    let mut units_per_host_tick = 1;
    for participant in &region.members {
        let layout = anchored_layouts_by_track
            .get(&participant.track_id)
            .and_then(|layouts| layouts.get(&participant.owner_id))?;
        let local_tick_count =
            (layout.horizontal_spacing.widths.len() / Slot::count()).saturating_sub(1) as Tick;
        let host_length = participant.end_tick.saturating_sub(participant.start_tick);
        if local_tick_count == 0 || host_length == 0 {
            return None;
        }

        let required_units = local_tick_count / gcd_tick(local_tick_count, host_length);
        units_per_host_tick = lcm_tick(units_per_host_tick, required_units);
    }

    let resolution = region_length * units_per_host_tick;
    if resolution == 0 {
        return None;
    }

    let mut slot_widths = vec![0.0; resolution as usize];

    for host_tick in 0..region_length {
        let width = spacing
            .get(region.start_tick + host_tick, Slot::NoteSpacing)
            .width;
        let projected_width = width / units_per_host_tick as f32;
        let start = host_tick as usize * units_per_host_tick as usize;
        let end = start + units_per_host_tick as usize;
        for slot_width in &mut slot_widths[start..end] {
            if projected_width > *slot_width {
                *slot_width = projected_width;
            }
        }
    }

    for participant in &region.members {
        let layout = anchored_layouts_by_track
            .get(&participant.track_id)
            .and_then(|layouts| layouts.get(&participant.owner_id))?;
        let local_tick_count =
            (layout.horizontal_spacing.widths.len() / Slot::count()).saturating_sub(1) as Tick;
        let host_length = participant.end_tick.saturating_sub(participant.start_tick);
        let units_in_span = host_length * units_per_host_tick;
        let units_per_local_tick = units_in_span / local_tick_count;
        let start_unit = (participant.start_tick - region.start_tick) * units_per_host_tick;

        for local_tick in 0..local_tick_count {
            let width = layout
                .horizontal_spacing
                .get(local_tick, Slot::NoteSpacing)
                .width;
            let projected_width = width / units_per_local_tick as f32;
            let start = (start_unit + (local_tick * units_per_local_tick)) as usize;
            let end = start + units_per_local_tick as usize;
            for slot_width in &mut slot_widths[start..end] {
                if projected_width > *slot_width {
                    *slot_width = projected_width;
                }
            }
        }
    }

    let width = slot_widths.iter().sum::<Space>();
    Some((
        SharedRhythmGrid {
            resolution,
            slot_widths,
            width,
        },
        units_per_host_tick,
    ))
}

fn finalize_shared_anchored_layouts(
    spacing: &mut HorizontalSpacing,
    notations_by_track: &NotationByTrack,
    anchored_layouts_by_track: &mut AnchoredTrackLayoutsByTrack,
) {
    let regions = collect_anchored_spacing_regions(notations_by_track, anchored_layouts_by_track);

    for region in regions {
        let region_length = region.end_tick.saturating_sub(region.start_tick);
        if region_length == 0 {
            continue;
        }

        let Some((shared_grid, units_per_host_tick)) =
            build_shared_region_grid(spacing, &region, anchored_layouts_by_track)
        else {
            continue;
        };

        let host_widths =
            shared_grid.project_widths_in_range(0, shared_grid.resolution, region_length);
        for (offset, width) in host_widths.into_iter().enumerate() {
            spacing.set_width(region.start_tick + offset as Tick, Slot::NoteSpacing, width);
        }

        for participant in region.members {
            let Some(layout) = anchored_layouts_by_track
                .get_mut(&participant.track_id)
                .and_then(|layouts| layouts.get_mut(&participant.owner_id))
            else {
                continue;
            };

            let local_tick_count =
                (layout.horizontal_spacing.widths.len() / Slot::count()).saturating_sub(1) as Tick;
            let start_unit = (participant.start_tick - region.start_tick) * units_per_host_tick;
            let unit_count = (participant.end_tick - participant.start_tick) * units_per_host_tick;
            let widths =
                shared_grid.project_widths_in_range(start_unit, unit_count, local_tick_count);
            rewrite_note_spacing_from_widths(&mut layout.horizontal_spacing, &widths);
        }
    }
}

pub fn stretch_spacing_to_width(spacing: &mut HorizontalSpacing, target_width: Space) {
    let local_tick_count = (spacing.widths.len() / Slot::count()).saturating_sub(1);
    if local_tick_count == 0 || target_width <= spacing.width {
        return;
    }

    let extra_per_tick = (target_width - spacing.width) / local_tick_count as f32;
    for tick in 0..local_tick_count as Tick {
        spacing.add_width(tick, Slot::NoteSpacing, extra_per_tick);
    }

    recompute_spacing_positions(spacing, false);
}

pub fn stretch_spacing_to_width_using_local_spacing(
    spacing: &mut HorizontalSpacing,
    target_width: Space,
) {
    let local_tick_count = (spacing.widths.len() / Slot::count()).saturating_sub(1) as Tick;
    if local_tick_count == 0 {
        return;
    }

    let Some(shared_grid) = SharedRhythmGrid::from_local_spacings(local_tick_count, &[spacing])
    else {
        stretch_spacing_to_width(spacing, target_width);
        return;
    };

    if target_width <= spacing.width {
        return;
    }

    let widths = shared_grid.project_widths(local_tick_count, target_width);
    rewrite_note_spacing_from_widths(spacing, &widths);
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InitialNoteLeadIn {
    pub accidentals_width: Space,
    pub pre_note_width: Space,
    pub total_width: Space,
}

pub fn take_initial_note_lead_in(spacing: &mut HorizontalSpacing) -> InitialNoteLeadIn {
    let lead_in = InitialNoteLeadIn {
        accidentals_width: spacing.get(0, Slot::Accidentals).width,
        pre_note_width: spacing.get(0, Slot::PreNote).width,
        total_width: spacing.get(0, Slot::NoteSpacing).x,
    };

    if lead_in.accidentals_width > 0.0 {
        spacing.set_width(0, Slot::Accidentals, 0.0);
    }

    if lead_in.pre_note_width > 0.0 {
        spacing.set_width(0, Slot::PreNote, 0.0);
    }

    if lead_in.accidentals_width > 0.0 || lead_in.pre_note_width > 0.0 {
        recompute_spacing_positions(spacing, false);
    }

    lead_in
}

fn collect_anchored_track_opening_spacing_updates(
    barlines: &Barlines,
    anchored_track: &AnchoredNotationTrack,
    lead_in: InitialNoteLeadIn,
) -> Vec<(Tick, Slot, Space)> {
    let mut updates = Vec::new();

    if lead_in.total_width <= 0.0 {
        return updates;
    }

    let start_tick = anchored_track.host_span.start_tick;

    if start_tick == 0 || barlines.contains_key(&start_tick) {
        if lead_in.accidentals_width > 0.0 {
            updates.push((start_tick, Slot::Accidentals, lead_in.accidentals_width));
        }

        if lead_in.pre_note_width > 0.0 {
            updates.push((start_tick, Slot::PreNote, lead_in.pre_note_width));
        }

        return updates;
    }

    updates
}

#[allow(clippy::too_many_arguments)]
fn extend_anchored_layout_tail_spacing(
    anchored_layout: &mut AnchoredTrackLayout,
    anchored_track: &AnchoredNotationTrack,
    notation: &NotationTrack,
    anchored_tracks: &[AnchoredNotationTrack],
    anchored_lead_ins: &FxHashMap<UniqueId, InitialNoteLeadIn>,
    host_shunts: &Shunts,
    host_accidentals: &Accidentals,
    barlines: &Barlines,
    engrave: &Engrave,
) -> bool {
    if anchored_track.notation.length == 0 {
        return false;
    }

    let last_search_tick = anchored_track.notation.length.saturating_sub(1);
    let Some((previous_tick, previous_entry)) = anchored_track
        .notation
        .get_previous_notation(last_search_tick)
    else {
        return false;
    };

    let Some(next_visible) = get_next_visible_notation_at_or_after(
        notation,
        anchored_tracks,
        anchored_track.host_span.end_tick,
    ) else {
        return false;
    };

    let next_preamble = match next_visible {
        NextVisibleNotation::Host { tick, entry } => {
            if barlines.contains_key(&tick) {
                return false;
            }

            host_notation_preamble(tick, entry, host_shunts, host_accidentals)
        }
        NextVisibleNotation::Anchored { track } => {
            if barlines.contains_key(&track.host_span.start_tick) {
                return false;
            }

            let Some(lead_in) = anchored_lead_ins.get(&track.owner_id).copied() else {
                return false;
            };
            let Some(next_preamble) = lead_in_preamble(lead_in) else {
                return false;
            };
            next_preamble
        }
    };

    let spacing = required_spacing_with_next_notation(
        previous_entry,
        previous_entry
            .metrics(&anchored_layout.shunts, engrave, &anchored_layout.beams)
            .width,
        &anchored_layout.shunts,
        engrave,
        &anchored_layout.beams,
        next_preamble.accidentals_width,
        next_preamble.glyph_width,
        next_preamble.has_pre_shunt,
    );

    apply_note_spacing_to_range(
        &mut anchored_layout.horizontal_spacing,
        previous_tick,
        previous_entry.duration,
        spacing,
    );
    recompute_spacing_positions(&mut anchored_layout.horizontal_spacing, false);

    true
}

#[allow(clippy::too_many_arguments)]
pub fn get_horizontal_spacing_with_anchored_layouts(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
    measures: &Measures,
    notations_by_track: &NotationByTrack,
    barlines: &Barlines,
    shunts_by_track: &ShuntsByTrack,
    beams_by_track: &BeamsByTrack,
    accidentals_by_track: &AccidentalsByTrack,
    anchored_accidentals_by_track: &AnchoredAccidentalsByTrack,
    tone_offsets: &ToneVerticalOffsets,
    engrave: &Engrave,
) -> (HorizontalSpacing, AnchoredTrackLayoutsByTrack) {
    let mut spacings = HorizontalSpacing::new(flow.length);
    let flow_master = &tracks.by_id[&flow.master_id];
    let mut anchored_layouts_by_track = AnchoredTrackLayoutsByTrack::default();

    spacings.set_width(0, Slot::PaddingStart, 0.75);
    spacings.set_width(0, Slot::BarPadding, 0.5);

    for (tick, time_signature) in measures {
        let measure_stop = (*tick + time_signature.ticks_per_bar()).min(flow.length);
        if measure_stop <= *tick {
            continue;
        }

        let width_per_tick =
            empty_measure_width(time_signature, engrave) / (measure_stop - *tick) as f32;

        for measure_tick in *tick..measure_stop {
            let current = spacings.get(measure_tick, Slot::NoteSpacing).width;
            if width_per_tick > current {
                spacings.set_width(measure_tick, Slot::NoteSpacing, width_per_tick);
            }
        }
    }

    for entry in flow_master.entries.by_id.values() {
        if let Entry::TimeSignature(time_signature) = entry {
            let metrics = time_signature.metrics();
            spacings.set_width(
                time_signature.tick,
                Slot::TimeSignature,
                metrics.total_width(),
            );
        }
    }

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &tracks.by_id[&stave.master_id];

            for entry in flow_master.entries.by_id.values() {
                if let Entry::KeySignature(key_signature) = entry {
                    let render_key_signature = if key_signature.offset == 0 {
                        flow_master.get_key_signature_before_tick(key_signature.tick)
                    } else {
                        Some(key_signature)
                    };

                    let width = match stave_master
                        .get_clef_before_tick_inclusive(key_signature.tick)
                    {
                        Some(clef)
                            if clef.can_render_key_signature()
                                && render_key_signature
                                    .is_some_and(|signature| signature.pattern(clef).is_some()) =>
                        {
                            let metrics = if key_signature.offset == 0 {
                                match flow_master.get_key_signature_before_tick(key_signature.tick)
                                {
                                    Some(previous) => previous.metrics(),
                                    None => BoundingBox {
                                        width: 0.0,
                                        height: 0.0,
                                        right_padding: 0.0,
                                    },
                                }
                            } else {
                                key_signature.metrics()
                            };

                            metrics.total_width()
                        }
                        _ => 0.0,
                    };
                    let current = spacings.get(key_signature.tick, Slot::KeySignature).width;

                    if width > current {
                        spacings.set_width(key_signature.tick, Slot::KeySignature, width);
                    }
                }
            }

            for entry in stave_master.entries.by_id.values() {
                if let Entry::Clef(clef) = entry {
                    let metrics = clef.metrics(clef.tick == 0);
                    spacings.set_width(clef.tick, Slot::Clef, metrics.total_width());
                }
            }
        }
    }

    for (tick, def) in barlines {
        spacings.set_width(*tick, Slot::BarPadding, 0.5);

        if def.end_repeat {
            let metrics = BarlineDrawType::EndRepeat.metrics();
            spacings.set_width(*tick, Slot::EndRepeat, metrics.total_width());
        }

        if let Some(draw_type) = &def.draw_type {
            let metrics = draw_type.metrics();

            let has_time_signature = flow_master.get_time_signature_at_tick(tick).is_some();
            let has_key_signature = spacings.get(*tick, Slot::KeySignature).width > 0.0;

            if has_key_signature {
                spacings.set_width(*tick, Slot::Barline, metrics.width + 0.5);
            } else if has_time_signature {
                spacings.set_width(*tick, Slot::Barline, metrics.width + 0.25);
            } else {
                spacings.set_width(*tick, Slot::Barline, metrics.total_width());
            }
        }

        if def.start_repeat {
            let metrics = BarlineDrawType::StartRepeat.metrics();
            spacings.set_width(*tick, Slot::StartRepeat, metrics.total_width());
        }
    }

    let mut spacing_updates: Vec<(Tick, Slot, Space)> = Vec::new();

    for (track_id, notation) in notations_by_track {
        let shunts = &shunts_by_track[track_id];
        let beams = &beams_by_track[track_id];
        let accidentals = &accidentals_by_track[track_id];
        let mut anchored_layouts = AnchoredTrackLayouts::default();
        let anchored_accidentals = anchored_accidentals_by_track.get(track_id);

        for anchored_track in &notation.anchored_tracks {
            let Some(Entry::Tuplet(tuplet)) = tracks.by_id[track_id]
                .entries
                .by_id
                .get(&anchored_track.owner_id)
            else {
                continue;
            };
            let Some(accidentals) = anchored_accidentals
                .and_then(|by_owner| by_owner.get(&anchored_track.owner_id))
                .cloned()
            else {
                continue;
            };

            let layout = build_anchored_track_layout(
                tuplet,
                anchored_track,
                accidentals,
                tone_offsets,
                engrave,
            );

            spacing_updates.extend(collect_anchored_track_opening_spacing_updates(
                barlines,
                anchored_track,
                layout.lead_in,
            ));

            anchored_layouts.insert(anchored_track.owner_id, layout);
        }

        let anchored_lead_ins = anchored_layouts
            .iter()
            .map(|(owner_id, layout)| (*owner_id, layout.lead_in))
            .collect::<FxHashMap<_, _>>();

        spacing_updates.extend(collect_notation_spacing_updates(
            notation,
            &notation.anchored_tracks,
            &anchored_lead_ins,
            barlines,
            shunts,
            beams,
            accidentals,
            engrave,
        ));

        for anchored_track in &notation.anchored_tracks {
            let Some(layout) = anchored_layouts.get_mut(&anchored_track.owner_id) else {
                continue;
            };

            extend_anchored_layout_tail_spacing(
                layout,
                anchored_track,
                notation,
                &notation.anchored_tracks,
                &anchored_lead_ins,
                shunts,
                accidentals,
                barlines,
                engrave,
            );
        }

        if !anchored_layouts.is_empty() {
            anchored_layouts_by_track.insert(*track_id, anchored_layouts);
        }
    }

    apply_spacing_updates(&mut spacings, spacing_updates);

    finalize_shared_anchored_layouts(
        &mut spacings,
        notations_by_track,
        &mut anchored_layouts_by_track,
    );

    recompute_spacing_positions(&mut spacings, true);

    (spacings, anchored_layouts_by_track)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_measures::get_measures;
    use crate::instructions::parse::get_written_durations::HostSpan;

    #[test]
    fn slot_count_reflects_all_variants() {
        // Slot::NoteSpacing = 10, so count = 11
        assert_eq!(Slot::count(), 11);
    }

    #[test]
    fn horizontal_spacing_span_new_has_zero_values() {
        let span = HorizontalSpacingSpan::new();
        assert_eq!(span.x, 0.0);
        assert_eq!(span.width, 0.0);
    }

    #[test]
    fn horizontal_spacing_new_creates_correct_size() {
        // length=1 → (1+1)*11 = 22 spans
        let hs = HorizontalSpacing::new(1);
        assert_eq!(hs.widths.len(), 22);
    }

    #[test]
    fn horizontal_spacing_get_returns_default_span() {
        let hs = HorizontalSpacing::new(4);
        let span = hs.get(0, Slot::NoteSpacing);
        assert_eq!(span.width, 0.0);
    }

    #[test]
    fn horizontal_spacing_set_width_updates_correct_slot() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(2, Slot::Barline, 3.5);
        let span = hs.get(2, Slot::Barline);
        assert!((span.width - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_spacing_set_does_not_affect_other_slots() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(1, Slot::Clef, 2.0);
        let unaffected = hs.get(1, Slot::Barline);
        assert_eq!(unaffected.width, 0.0);
    }

    #[test]
    fn horizontal_spacing_initial_width_is_zero() {
        let hs = HorizontalSpacing::new(4);
        assert_eq!(hs.width, 0.0);
    }

    #[test]
    fn slot_padding_start_is_zero() {
        assert_eq!(Slot::PaddingStart as usize, 0);
    }

    #[test]
    fn slot_note_spacing_is_last() {
        assert_eq!(Slot::NoteSpacing as usize, Slot::count() - 1);
    }

    #[test]
    fn horizontal_spacing_new_length_zero_still_allocates() {
        // length=0 → (0+1)*11 = 11 spans
        let hs = HorizontalSpacing::new(0);
        assert_eq!(hs.widths.len(), 11);
    }

    #[test]
    fn horizontal_spacing_set_width_at_tick_zero() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(0, Slot::PaddingStart, 5.0);
        let span = hs.get(0, Slot::PaddingStart);
        assert!((span.width - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_spacing_multiple_slots_at_same_tick() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(1, Slot::Clef, 1.5);
        hs.set_width(1, Slot::TimeSignature, 2.5);
        assert!((hs.get(1, Slot::Clef).width - 1.5).abs() < f32::EPSILON);
        assert!((hs.get(1, Slot::TimeSignature).width - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_spacing_span_new_x_and_width_are_floats() {
        let span = HorizontalSpacingSpan::new();
        let _ = span.x + span.width; // just ensure they are f32 compatible
        assert_eq!(span.x, 0.0_f32);
    }

    #[test]
    fn horizontal_spacing_set_overwrite_updates_value() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(0, Slot::KeySignature, 1.0);
        hs.set_width(0, Slot::KeySignature, 4.0);
        let span = hs.get(0, Slot::KeySignature);
        assert!((span.width - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn percussion_clef_yields_zero_key_signature_width_in_flow_pass() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::engraves::Engraves;
        use crate::score::entries::clef::{Clef, ClefDrawType};
        use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::entries::Entry;
        use crate::score::instruments::{defs::InstrumentType, Instrument, PlaybackAssignmentIntent};
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;
        use crate::score::tracks::{Track, Tracks};
        use rustc_hash::{FxHashMap, FxHashSet};

        let mut flow = Flow::new(1, 100);
        flow.player_ids = FxHashSet::default();
        flow.length = 128;
        flow.staves.by_id.insert(
            10,
            Stave {
                id: 10,
                lines: vec![1, 1, 1, 1, 1],
                master_id: 200,
                track_ids: vec![],
            },
        );

        let instrument = Instrument {
            id: 20,
            definition_id: String::from("unpitched-percussion.snare"),
            instrument_type: InstrumentType::Percussive,
            long_name: String::from("Snare"),
            short_name: String::from("Sn."),
            stave_ids: vec![10],
            count: None,
            assignment_intent: PlaybackAssignmentIntent::default(),
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        };

        let mut flow_master = Track::new(100);
        flow_master.insert_entry(Entry::TimeSignature(TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));
        flow_master.insert_entry(Entry::KeySignature(KeySignature::new(
            400,
            64,
            KeySignatureMode::Major,
            -5,
        )));

        let mut stave_master = Track::new(200);
        stave_master.insert_entry(Entry::Clef(Clef::new(
            300,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::Percussion,
        )));

        let mut tracks = Tracks::new();
        tracks.by_id = FxHashMap::default();
        tracks.by_id.insert(100, flow_master);
        tracks.by_id.insert(200, stave_master);

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();

        let spacing = get_horizontal_spacing_with_anchored_layouts(
            &flow,
            &vec![&instrument],
            &tracks,
            &get_measures(&flow, &tracks),
            &NotationByTrack::default(),
            &Barlines::default(),
            &ShuntsByTrack::default(),
            &BeamsByTrack::default(),
            &AccidentalsByTrack::default(),
            &AnchoredAccidentalsByTrack::default(),
            &ToneVerticalOffsets::default(),
            engrave,
        )
        .0;

        assert_eq!(spacing.get(64, Slot::KeySignature).width, 0.0);
    }

    #[test]
    fn cancelling_key_signature_reserves_width_from_previous_signature() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::engraves::Engraves;
        use crate::score::entries::clef::{Clef, ClefDrawType};
        use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::entries::Entry;
        use crate::score::instruments::{defs::InstrumentType, Instrument, PlaybackAssignmentIntent};
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;
        use crate::score::tracks::{Track, Tracks};
        use rustc_hash::{FxHashMap, FxHashSet};

        let mut flow = Flow::new(1, 100);
        flow.player_ids = FxHashSet::default();
        flow.length = 128;
        flow.staves.by_id.insert(
            10,
            Stave {
                id: 10,
                lines: vec![1, 1, 1, 1, 1],
                master_id: 200,
                track_ids: vec![],
            },
        );

        let instrument = Instrument {
            id: 20,
            definition_id: String::from("strings.violin"),
            instrument_type: InstrumentType::Melodic,
            long_name: String::from("Violin"),
            short_name: String::from("Vln."),
            stave_ids: vec![10],
            count: None,
            assignment_intent: PlaybackAssignmentIntent::default(),
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        };

        let mut flow_master = Track::new(100);
        flow_master.insert_entry(Entry::TimeSignature(TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));
        flow_master.insert_entry(Entry::KeySignature(KeySignature::new(
            401,
            32,
            KeySignatureMode::Major,
            3,
        )));
        flow_master.insert_entry(Entry::KeySignature(KeySignature::new(
            402,
            64,
            KeySignatureMode::Major,
            0,
        )));

        let mut stave_master = Track::new(200);
        stave_master.insert_entry(Entry::Clef(Clef::new(
            300,
            0,
            Pitch::new(67, Accidental::Natural),
            2,
            ClefDrawType::G,
        )));

        let mut tracks = Tracks::new();
        tracks.by_id = FxHashMap::default();
        tracks.by_id.insert(100, flow_master);
        tracks.by_id.insert(200, stave_master);

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();

        let spacing = get_horizontal_spacing_with_anchored_layouts(
            &flow,
            &vec![&instrument],
            &tracks,
            &get_measures(&flow, &tracks),
            &NotationByTrack::default(),
            &Barlines::default(),
            &ShuntsByTrack::default(),
            &BeamsByTrack::default(),
            &AccidentalsByTrack::default(),
            &AnchoredAccidentalsByTrack::default(),
            &ToneVerticalOffsets::default(),
            engrave,
        )
        .0;

        assert!(spacing.get(64, Slot::KeySignature).width > 0.0);
    }

    #[test]
    fn empty_measure_width_matches_between_three_four_and_six_eight() {
        use crate::score::components::duration::NoteDuration;
        use crate::score::engraves::Engraves;
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::layouts::LayoutType;

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
        let three_four = TimeSignature::new(
            1,
            0,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        let six_eight = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );

        assert!(
            (empty_measure_width(&three_four, engrave) - empty_measure_width(&six_eight, engrave))
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn empty_measure_width_nine_eight_exceeds_six_eight() {
        use crate::score::components::duration::NoteDuration;
        use crate::score::engraves::Engraves;
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::layouts::LayoutType;

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
        let six_eight = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        let nine_eight = TimeSignature::new(
            1,
            0,
            9,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );

        assert!(
            empty_measure_width(&nine_eight, engrave) > empty_measure_width(&six_eight, engrave)
        );
    }

    #[test]
    fn horizontal_spacing_empty_nine_eight_bar_is_wider_than_empty_six_eight_bar() {
        use crate::score::components::duration::NoteDuration;
        use crate::score::engraves::Engraves;
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::entries::Entry;
        use crate::score::flows::Flow;
        use crate::score::instruments::{defs::InstrumentType, Instrument, PlaybackAssignmentIntent};
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;
        use crate::score::tracks::{Track, Tracks};
        use rustc_hash::{FxHashMap, FxHashSet};

        fn make_empty_spacing(time_signature: TimeSignature) -> HorizontalSpacing {
            let mut flow = Flow::new(1, 100);
            flow.player_ids = FxHashSet::default();
            flow.length = time_signature.ticks_per_bar();
            flow.staves.by_id.insert(
                10,
                Stave {
                    id: 10,
                    lines: vec![1, 1, 1, 1, 1],
                    master_id: 200,
                    track_ids: vec![],
                },
            );

            let instrument = Instrument {
                id: 20,
                definition_id: String::from("strings.violin"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Violin"),
                short_name: String::from("Vln."),
                stave_ids: vec![10],
                count: None,
                assignment_intent: PlaybackAssignmentIntent::default(),
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            };

            let mut flow_master = Track::new(100);
            flow_master.insert_entry(Entry::TimeSignature(time_signature));

            let mut tracks = Tracks::new();
            tracks.by_id = FxHashMap::default();
            tracks.by_id.insert(100, flow_master);
            tracks.by_id.insert(200, Track::new(200));

            let engraves = Engraves::new();
            let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
            let measures = get_measures(&flow, &tracks);

            get_horizontal_spacing_with_anchored_layouts(
                &flow,
                &vec![&instrument],
                &tracks,
                &measures,
                &NotationByTrack::default(),
                &Barlines::default(),
                &ShuntsByTrack::default(),
                &BeamsByTrack::default(),
                &AccidentalsByTrack::default(),
                &AnchoredAccidentalsByTrack::default(),
                &ToneVerticalOffsets::default(),
                engrave,
            )
            .0
        }

        let six_eight_spacing = make_empty_spacing(TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        ));
        let nine_eight_spacing = make_empty_spacing(TimeSignature::new(
            1,
            0,
            9,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        ));

        assert!(nine_eight_spacing.width > six_eight_spacing.width);
    }

    #[test]
    fn shared_rhythm_grid_preserves_single_spacing_widths() {
        let mut spacing = HorizontalSpacing::new(3);
        spacing.set_width(0, Slot::NoteSpacing, 3.0);
        spacing.set_width(1, Slot::NoteSpacing, 1.0);
        spacing.set_width(2, Slot::NoteSpacing, 1.0);

        let grid = SharedRhythmGrid::from_local_spacings(3, &[&spacing])
            .expect("resolved note spacing should produce a shared rhythm grid");

        assert_eq!(grid.resolution, 3);
        assert!((grid.width - 5.0).abs() < 1e-6);
        assert_eq!(grid.project_widths(3, grid.width), vec![3.0, 1.0, 1.0]);
    }

    #[test]
    fn shared_rhythm_grid_ignores_non_note_spacing_slots() {
        let mut spacing = HorizontalSpacing::new(2);
        spacing.set_width(0, Slot::NoteSpacing, 2.0);
        spacing.set_width(1, Slot::NoteSpacing, 1.0);
        spacing.set_width(0, Slot::Accidentals, 5.0);
        spacing.set_width(0, Slot::PreNote, 4.0);

        let grid = SharedRhythmGrid::from_local_spacings(2, &[&spacing])
            .expect("resolved note spacing should produce a shared rhythm grid");

        assert_eq!(grid.resolution, 2);
        assert_eq!(grid.project_widths(2, grid.width), vec![2.0, 1.0]);
    }

    #[test]
    fn stretch_spacing_to_width_using_local_spacing_uses_resolved_proportions() {
        let mut spacing = HorizontalSpacing::new(3);
        spacing.set_width(0, Slot::NoteSpacing, 3.0);
        spacing.set_width(1, Slot::NoteSpacing, 1.0);
        spacing.set_width(2, Slot::NoteSpacing, 1.0);
        recompute_spacing_positions(&mut spacing, false);

        stretch_spacing_to_width_using_local_spacing(&mut spacing, 8.0);

        assert!((spacing.get(0, Slot::NoteSpacing).width - 4.8).abs() < 1e-6);
        assert!((spacing.get(1, Slot::NoteSpacing).width - 1.6).abs() < 1e-6);
        assert!((spacing.get(2, Slot::NoteSpacing).width - 1.6).abs() < 1e-6);
    }

    #[test]
    fn stretch_spacing_to_width_using_local_spacing_preserves_uniform_behavior_for_equal_widths() {
        let mut spacing = HorizontalSpacing::new(3);
        spacing.set_width(0, Slot::NoteSpacing, 1.0);
        spacing.set_width(1, Slot::NoteSpacing, 1.0);
        spacing.set_width(2, Slot::NoteSpacing, 1.0);
        recompute_spacing_positions(&mut spacing, false);

        stretch_spacing_to_width_using_local_spacing(&mut spacing, 6.0);

        assert!((spacing.get(0, Slot::NoteSpacing).width - 2.0).abs() < f32::EPSILON);
        assert!((spacing.get(1, Slot::NoteSpacing).width - 2.0).abs() < f32::EPSILON);
        assert!((spacing.get(2, Slot::NoteSpacing).width - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn build_anchored_track_layout_unifies_tuplet_stem_directions() {
        use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
        use crate::instructions::parse::get_tone_offsets::ToneVerticalOffsets;
        use crate::instructions::parse::get_written_durations::Notation;
        use crate::score::components::articulation::Articulations;
        use crate::score::components::duration::{Direction, NoteDuration};
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::engraves::Engraves;
        use crate::score::entries::tone::Tone;
        use crate::score::layouts::LayoutType;

        let eighth = NoteDuration::Eighth.to_ticks();
        let mut local_track = NotationTrack::new(NoteDuration::Quarter.to_ticks() + eighth);

        let mut first = Notation::new(0, eighth);
        first.tones.push(Tone::new(
            1,
            0,
            eighth,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        ));
        first.info = Some(ToneOffsetInfo {
            highest: 1,
            lowest: 1,
            furthest: 1,
        });

        let mut second = Notation::new(eighth, eighth);
        second.tones.push(Tone::new(
            2,
            eighth,
            eighth,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        ));
        second.info = Some(ToneOffsetInfo {
            highest: -3,
            lowest: -3,
            furthest: -3,
        });

        local_track.track.insert(0, first);
        local_track
            .track
            .insert(eighth / 2, Notation::new(eighth / 2, eighth / 2));
        local_track.track.insert(eighth, second);

        let anchored_track = AnchoredNotationTrack {
            owner_id: 77,
            host_span: HostSpan {
                start_tick: 0,
                end_tick: eighth * 2,
            },
            notation: local_track,
        };

        let tuplet = Tuplet::new(77, 0, 3, 2, NoteDuration::Eighth, vec![3]);
        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
        let mut tone_offsets = ToneVerticalOffsets::default();
        tone_offsets.insert(1, 1);
        tone_offsets.insert(2, -3);

        let layout = build_anchored_track_layout(
            &tuplet,
            &anchored_track,
            Accidentals::new(),
            &tone_offsets,
            engrave,
        );

        assert!(matches!(layout.stem_directions[&0], Direction::Down));
        assert!(matches!(layout.stem_directions[&eighth], Direction::Down));
        assert!(!layout.stem_directions.contains_key(&(eighth / 2)));
    }

    #[test]
    fn shared_rhythm_grid_uses_lcm_resolution_for_different_local_lengths() {
        let mut triplet_spacing = HorizontalSpacing::new(3);
        triplet_spacing.set_width(0, Slot::NoteSpacing, 3.0);
        triplet_spacing.set_width(1, Slot::NoteSpacing, 1.0);
        triplet_spacing.set_width(2, Slot::NoteSpacing, 1.0);

        let mut duplet_spacing = HorizontalSpacing::new(2);
        duplet_spacing.set_width(0, Slot::NoteSpacing, 1.0);
        duplet_spacing.set_width(1, Slot::NoteSpacing, 3.0);

        let grid = SharedRhythmGrid::from_local_spacings(2, &[&triplet_spacing, &duplet_spacing])
            .expect("mixed local lengths should produce a shared rhythm grid");

        assert_eq!(grid.resolution, 6);
        assert!((grid.width - 6.5).abs() < 1e-6);
        assert_eq!(grid.project_widths(2, grid.width), vec![3.5, 3.0]);
        assert_eq!(grid.project_widths(3, grid.width), vec![3.0, 1.5, 2.0]);
    }

    #[test]
    fn finalize_shared_anchored_layouts_aligns_tuplets_across_tracks_with_shared_host_span() {
        let mut host_spacing = HorizontalSpacing::new(2);
        host_spacing.set_width(0, Slot::NoteSpacing, 1.0);
        host_spacing.set_width(1, Slot::NoteSpacing, 1.0);

        let mut triplet_spacing = HorizontalSpacing::new(3);
        triplet_spacing.set_width(0, Slot::NoteSpacing, 3.0);
        triplet_spacing.set_width(1, Slot::NoteSpacing, 1.0);
        triplet_spacing.set_width(2, Slot::NoteSpacing, 1.0);
        recompute_spacing_positions(&mut triplet_spacing, false);

        let mut duplet_spacing = HorizontalSpacing::new(2);
        duplet_spacing.set_width(0, Slot::NoteSpacing, 1.0);
        duplet_spacing.set_width(1, Slot::NoteSpacing, 3.0);
        recompute_spacing_positions(&mut duplet_spacing, false);

        let mut anchored_layouts_by_track = AnchoredTrackLayoutsByTrack::default();
        anchored_layouts_by_track.insert(
            300,
            [(
                401,
                AnchoredTrackLayout {
                    measures: Measures::default(),
                    horizontal_spacing: triplet_spacing,
                    lead_in: InitialNoteLeadIn::default(),
                    beams: Vec::new(),
                    stem_directions: FxHashMap::default(),
                    shunts: Shunts {
                        by_id: FxHashMap::default(),
                        by_offset: FxHashMap::default(),
                    },
                    accidentals: Accidentals::new(),
                    dots: FxHashMap::default(),
                    articulations: FxHashMap::default(),
                },
            )]
            .into_iter()
            .collect(),
        );
        anchored_layouts_by_track.insert(
            301,
            [(
                402,
                AnchoredTrackLayout {
                    measures: Measures::default(),
                    horizontal_spacing: duplet_spacing,
                    lead_in: InitialNoteLeadIn::default(),
                    beams: Vec::new(),
                    stem_directions: FxHashMap::default(),
                    shunts: Shunts {
                        by_id: FxHashMap::default(),
                        by_offset: FxHashMap::default(),
                    },
                    accidentals: Accidentals::new(),
                    dots: FxHashMap::default(),
                    articulations: FxHashMap::default(),
                },
            )]
            .into_iter()
            .collect(),
        );

        let shared_host_span = HostSpan {
            start_tick: 0,
            end_tick: 2,
        };
        let mut notations_by_track = NotationByTrack::default();
        notations_by_track.insert(
            300,
            NotationTrack {
                length: 2,
                track: FxHashMap::default(),
                anchored_tracks: vec![AnchoredNotationTrack {
                    owner_id: 401,
                    host_span: shared_host_span.clone(),
                    notation: NotationTrack::new(3),
                }],
            },
        );
        notations_by_track.insert(
            301,
            NotationTrack {
                length: 2,
                track: FxHashMap::default(),
                anchored_tracks: vec![AnchoredNotationTrack {
                    owner_id: 402,
                    host_span: shared_host_span,
                    notation: NotationTrack::new(2),
                }],
            },
        );

        finalize_shared_anchored_layouts(
            &mut host_spacing,
            &notations_by_track,
            &mut anchored_layouts_by_track,
        );

        assert!((host_spacing.get(0, Slot::NoteSpacing).width - 3.5).abs() < 1e-6);
        assert!((host_spacing.get(1, Slot::NoteSpacing).width - 3.0).abs() < 1e-6);

        let first_layout = &anchored_layouts_by_track[&300][&401].horizontal_spacing;
        assert!((first_layout.get(0, Slot::NoteSpacing).width - 3.0).abs() < 1e-6);
        assert!((first_layout.get(1, Slot::NoteSpacing).width - 1.5).abs() < 1e-6);
        assert!((first_layout.get(2, Slot::NoteSpacing).width - 2.0).abs() < 1e-6);

        let second_layout = &anchored_layouts_by_track[&301][&402].horizontal_spacing;
        assert!((second_layout.get(0, Slot::NoteSpacing).width - 3.5).abs() < 1e-6);
        assert!((second_layout.get(1, Slot::NoteSpacing).width - 3.0).abs() < 1e-6);
    }

    #[test]
    fn finalize_shared_anchored_layouts_merges_partially_overlapping_spans() {
        let mut host_spacing = HorizontalSpacing::new(3);
        host_spacing.set_width(0, Slot::NoteSpacing, 1.0);
        host_spacing.set_width(1, Slot::NoteSpacing, 1.0);
        host_spacing.set_width(2, Slot::NoteSpacing, 1.0);

        let mut triplet_spacing = HorizontalSpacing::new(3);
        triplet_spacing.set_width(0, Slot::NoteSpacing, 3.0);
        triplet_spacing.set_width(1, Slot::NoteSpacing, 1.0);
        triplet_spacing.set_width(2, Slot::NoteSpacing, 1.0);
        recompute_spacing_positions(&mut triplet_spacing, false);

        let mut duplet_spacing = HorizontalSpacing::new(2);
        duplet_spacing.set_width(0, Slot::NoteSpacing, 1.0);
        duplet_spacing.set_width(1, Slot::NoteSpacing, 3.0);
        recompute_spacing_positions(&mut duplet_spacing, false);

        let mut anchored_layouts_by_track = AnchoredTrackLayoutsByTrack::default();
        anchored_layouts_by_track.insert(
            300,
            [(
                401,
                AnchoredTrackLayout {
                    measures: Measures::default(),
                    horizontal_spacing: triplet_spacing,
                    lead_in: InitialNoteLeadIn::default(),
                    beams: Vec::new(),
                    stem_directions: FxHashMap::default(),
                    shunts: Shunts {
                        by_id: FxHashMap::default(),
                        by_offset: FxHashMap::default(),
                    },
                    accidentals: Accidentals::new(),
                    dots: FxHashMap::default(),
                    articulations: FxHashMap::default(),
                },
            )]
            .into_iter()
            .collect(),
        );
        anchored_layouts_by_track.insert(
            301,
            [(
                402,
                AnchoredTrackLayout {
                    measures: Measures::default(),
                    horizontal_spacing: duplet_spacing,
                    lead_in: InitialNoteLeadIn::default(),
                    beams: Vec::new(),
                    stem_directions: FxHashMap::default(),
                    shunts: Shunts {
                        by_id: FxHashMap::default(),
                        by_offset: FxHashMap::default(),
                    },
                    accidentals: Accidentals::new(),
                    dots: FxHashMap::default(),
                    articulations: FxHashMap::default(),
                },
            )]
            .into_iter()
            .collect(),
        );

        let mut notations_by_track = NotationByTrack::default();
        notations_by_track.insert(
            300,
            NotationTrack {
                length: 3,
                track: FxHashMap::default(),
                anchored_tracks: vec![AnchoredNotationTrack {
                    owner_id: 401,
                    host_span: HostSpan {
                        start_tick: 0,
                        end_tick: 2,
                    },
                    notation: NotationTrack::new(3),
                }],
            },
        );
        notations_by_track.insert(
            301,
            NotationTrack {
                length: 3,
                track: FxHashMap::default(),
                anchored_tracks: vec![AnchoredNotationTrack {
                    owner_id: 402,
                    host_span: HostSpan {
                        start_tick: 1,
                        end_tick: 3,
                    },
                    notation: NotationTrack::new(2),
                }],
            },
        );

        finalize_shared_anchored_layouts(
            &mut host_spacing,
            &notations_by_track,
            &mut anchored_layouts_by_track,
        );

        assert!((host_spacing.get(0, Slot::NoteSpacing).width - 3.5).abs() < 1e-6);
        assert!((host_spacing.get(1, Slot::NoteSpacing).width - 1.5).abs() < 1e-6);
        assert!((host_spacing.get(2, Slot::NoteSpacing).width - 3.0).abs() < 1e-6);

        let first_layout = &anchored_layouts_by_track[&300][&401].horizontal_spacing;
        assert!((first_layout.get(0, Slot::NoteSpacing).width - 3.0).abs() < 1e-6);
        assert!((first_layout.get(1, Slot::NoteSpacing).width - 1.0).abs() < 1e-6);
        assert!((first_layout.get(2, Slot::NoteSpacing).width - 1.0).abs() < 1e-6);

        let second_layout = &anchored_layouts_by_track[&301][&402].horizontal_spacing;
        assert!((second_layout.get(0, Slot::NoteSpacing).width - 1.5).abs() < 1e-6);
        assert!((second_layout.get(1, Slot::NoteSpacing).width - 3.0).abs() < 1e-6);
    }

    #[test]
    fn consecutive_tuplets_use_next_tuplet_opening_as_forward_successor() {
        use crate::instructions::parse::get_accidentals::AnchoredAccidentals;
        use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
        use crate::instructions::parse::get_written_durations::{HostSpan, Notation, NotationTrack};
        use crate::score::components::duration::NoteDuration;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::engraves::Engraves;
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::entries::tone::Tone;
        use crate::score::entries::tuplet::Tuplet;
        use crate::score::entries::Entry;
        use crate::score::flows::Flow;
        use crate::score::instruments::{defs::InstrumentType, Instrument, PlaybackAssignmentIntent};
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;
        use crate::score::tracks::{Track, Tracks};
        use rustc_hash::{FxHashMap, FxHashSet};

        fn make_layouts(second_tuplet_has_opening_accidental: bool) -> AnchoredTrackLayoutsByTrack {
            let e = NoteDuration::Eighth.to_ticks();
            let first_tuplet_id = 401;
            let second_tuplet_id = 402;
            let first_tone_id = 501;
            let second_tone_id = 502;

            let mut flow = Flow::new(1, 100);
            flow.player_ids = FxHashSet::default();
            flow.length = e * 4;
            flow.staves.by_id.insert(
                10,
                Stave {
                    id: 10,
                    lines: vec![1, 1, 1, 1, 1],
                    master_id: 200,
                    track_ids: vec![300],
                },
            );

            let instrument = Instrument {
                id: 20,
                definition_id: String::from("strings.violin"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Violin"),
                short_name: String::from("Vln."),
                stave_ids: vec![10],
                count: None,
                assignment_intent: PlaybackAssignmentIntent::default(),
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            };

            let mut flow_master = Track::new(100);
            flow_master.insert_entry(Entry::TimeSignature(TimeSignature::new(
                1,
                0,
                4,
                NoteDuration::Quarter,
                TimeSignatureDrawType::Regular,
                None,
            )));

            let mut track_master = Track::new(300);
            track_master.insert_entry(Entry::Tuplet(Box::new(Tuplet::new(
                first_tuplet_id,
                0,
                3,
                2,
                NoteDuration::Eighth,
                vec![3],
            ))));
            track_master.insert_entry(Entry::Tuplet(Box::new(Tuplet::new(
                second_tuplet_id,
                e * 2,
                3,
                2,
                NoteDuration::Eighth,
                vec![3],
            ))));

            let mut tracks = Tracks::new();
            tracks.by_id = FxHashMap::default();
            tracks.by_id.insert(100, flow_master);
            tracks.by_id.insert(200, Track::new(200));
            tracks.by_id.insert(300, track_master);

            let mut first_local_track = NotationTrack::new(e * 3);
            let mut first_entry = Notation::new(e * 2, e);
            first_entry.tones.push(Tone::new(
                first_tone_id,
                e * 2,
                e,
                Pitch::new(60, Accidental::Natural),
                Velocity::new(90),
                Default::default(),
            ));
            first_entry.info = Some(ToneOffsetInfo {
                highest: 0,
                lowest: 0,
                furthest: 0,
            });
            first_local_track.track.insert(e * 2, first_entry);

            let mut second_local_track = NotationTrack::new(e * 3);
            let mut second_entry = Notation::new(0, e);
            second_entry.tones.push(Tone::new(
                second_tone_id,
                0,
                e,
                Pitch::new(61, Accidental::Sharp),
                Velocity::new(90),
                Default::default(),
            ));
            second_entry.info = Some(ToneOffsetInfo {
                highest: 0,
                lowest: 0,
                furthest: 0,
            });
            second_local_track.track.insert(0, second_entry);

            let notation = NotationTrack {
                length: flow.length,
                track: FxHashMap::default(),
                anchored_tracks: vec![
                    AnchoredNotationTrack {
                        owner_id: first_tuplet_id,
                        host_span: HostSpan {
                            start_tick: 0,
                            end_tick: e * 2,
                        },
                        notation: first_local_track,
                    },
                    AnchoredNotationTrack {
                        owner_id: second_tuplet_id,
                        host_span: HostSpan {
                            start_tick: e * 2,
                            end_tick: e * 4,
                        },
                        notation: second_local_track,
                    },
                ],
            };

            let mut notations_by_track = NotationByTrack::default();
            notations_by_track.insert(300, notation);

            let mut shunts_by_track = ShuntsByTrack::default();
            shunts_by_track.insert(
                300,
                Shunts {
                    by_id: FxHashMap::default(),
                    by_offset: FxHashMap::default(),
                },
            );

            let mut beams_by_track = BeamsByTrack::default();
            beams_by_track.insert(300, Vec::new());

            let mut accidentals_by_track = AccidentalsByTrack::default();
            accidentals_by_track.insert(300, Accidentals::new());

            let mut second_anchored_accidentals = Accidentals::new();
            if second_tuplet_has_opening_accidental {
                second_anchored_accidentals.slots_by_tick.insert(0, 1);
            }

            let mut anchored_accidentals = AnchoredAccidentals::default();
            anchored_accidentals.insert(first_tuplet_id, Accidentals::new());
            anchored_accidentals.insert(second_tuplet_id, second_anchored_accidentals);

            let mut anchored_accidentals_by_track = AnchoredAccidentalsByTrack::default();
            anchored_accidentals_by_track.insert(300, anchored_accidentals);

            let mut tone_offsets = ToneVerticalOffsets::default();
            tone_offsets.insert(first_tone_id, 0);
            tone_offsets.insert(second_tone_id, 0);

            let engraves = Engraves::new();
            let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
            let measures = get_measures(&flow, &tracks);

            get_horizontal_spacing_with_anchored_layouts(
                &flow,
                &vec![&instrument],
                &tracks,
                &measures,
                &notations_by_track,
                &Barlines::default(),
                &shunts_by_track,
                &beams_by_track,
                &accidentals_by_track,
                &anchored_accidentals_by_track,
                &tone_offsets,
                engrave,
            )
            .1
        }

        let base_layouts = make_layouts(false);
        let accidental_layouts = make_layouts(true);

        let e = NoteDuration::Eighth.to_ticks();
        let base_width = base_layouts[&300][&401]
            .horizontal_spacing
            .get(e * 2, Slot::NoteSpacing)
            .width;
        let accidental_width = accidental_layouts[&300][&401]
            .horizontal_spacing
            .get(e * 2, Slot::NoteSpacing)
            .width;

        assert!(
            accidental_width > base_width,
            "expected the first tuplet tail to use the second tuplet opening as its forward spacing successor"
        );
    }
}
