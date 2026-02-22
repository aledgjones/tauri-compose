use super::get_accidentals::AccidentalsByTrack;
use super::get_barlines::Barlines;
use super::get_beams::BeamsByTrack;
use super::get_shunts::ShuntsByTrack;
use super::get_written_durations::NotationByTrack;
use crate::renderer::converter::Space;
use crate::score::components::duration::{Tick, Ticks};
use crate::score::components::measurements::BoundingBox;
use crate::score::engraves::Engrave;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;
use rayon::prelude::*;

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
}

#[allow(clippy::too_many_arguments)]
pub fn get_horizontal_spacing(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
    notations_by_track: &NotationByTrack,
    barlines: &Barlines,
    shunts_by_track: &ShuntsByTrack,
    beams_by_track: &BeamsByTrack,
    accidentals_by_track: &AccidentalsByTrack,
    engrave: &Engrave,
) -> HorizontalSpacing {
    let mut spacings = HorizontalSpacing::new(flow.length);
    let flow_master = &tracks.by_id[&flow.master_id];

    spacings.set_width(0, Slot::PaddingStart, 0.75);
    spacings.set_width(0, Slot::BarPadding, 0.5);

    for entry in flow_master.entries.by_id.values() {
        match entry {
            Entry::KeySignature(key_signature) => {
                let metrics = if key_signature.offset == 0 {
                    // find width needed to cancel the previous key signature
                    match flow_master.get_key_signature_before_tick(key_signature.tick) {
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
                spacings.set_width(
                    key_signature.tick,
                    Slot::KeySignature,
                    metrics.total_width(),
                );
            }
            Entry::TimeSignature(time_signature) => {
                let metrics = time_signature.metrics();
                spacings.set_width(
                    time_signature.tick,
                    Slot::TimeSignature,
                    metrics.total_width(),
                );
            }
            _ => (),
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
            let has_key_signature = flow_master.get_key_signature_at_tick(tick).is_some();

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

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &tracks.by_id[&stave.master_id];
            for entry in stave_master.entries.by_id.values() {
                if let Entry::Clef(clef) = entry {
                    let metrics = clef.metrics(clef.tick == 0);
                    spacings.set_width(clef.tick, Slot::Clef, metrics.total_width());
                }
            }
        }
    }

    let spacing_updates: Vec<(Tick, Slot, Space)> = notations_by_track
        .par_iter()
        .flat_map(|(track_key, notation)| {
            let shunts = &shunts_by_track[track_key];
            let accidentals = &accidentals_by_track[track_key];
            let beams = &beams_by_track[track_key];
            let mut updates: Vec<(Tick, Slot, Space)> = Vec::new();

            for (tick, entry) in &notation.track {
                let mut spacing = entry.metrics(shunts, engrave, beams).width;

                // if the tick is at the start of a barline or the start of
                // the piece we add explicit space for accidentals and pre-shunts
                if *tick == 0 || barlines.contains_key(tick) {
                    let metrics = entry.glyph_metrics();
                    let glyph_width = metrics.total_width();
                    let has_pre_shunt = entry.has_pre_shunt(shunts);

                    if has_pre_shunt {
                        updates.push((*tick, Slot::PreNote, glyph_width));
                    }

                    // start of bars has no previous spacing to extend so we use the accidentals slot
                    if let Some(slots) = accidentals.slots_by_tick.get(tick) {
                        let width = if has_pre_shunt {
                            ((*slots as f32 - 1.0) * 1.1) + 0.25
                        } else {
                            (*slots as f32 * 1.1) + 0.25
                        };
                        updates.push((*tick, Slot::Accidentals, width));
                    };
                }

                // extend spacing to accomodate accidentals + pre shunts that are part of the next note (if needed)
                if let Some((next_tick, next_entry)) = notation.get_next_notation(*tick) {
                    if !barlines.contains_key(&next_tick) {
                        let min = entry.min_spacing(shunts, engrave, beams);
                        let next_metrics = next_entry.glyph_metrics();
                        let next_glyph_width = next_metrics.total_width();
                        let next_has_pre_shunt = next_entry.has_pre_shunt(shunts);

                        let accidentals_width = match accidentals.slots_by_tick.get(&next_tick) {
                            Some(slots) if *slots > 0 => {
                                if next_has_pre_shunt {
                                    ((*slots as f32 - 1.0) * 1.1) + 0.25
                                } else {
                                    (*slots as f32 * 1.1) + 0.25
                                }
                            }
                            _ => 0.0,
                        };

                        let pre_shunt_width = if next_has_pre_shunt {
                            next_glyph_width
                        } else {
                            0.0
                        };

                        let ledger_line_width = match &entry.info {
                            Some(info) if info.highest < -5 || info.lowest > 5 => 0.4,
                            _ => 0.0,
                        };

                        let avoid_collisions_spacing = accidentals_width
                            + pre_shunt_width
                            + min.width
                            + ledger_line_width
                            + 0.2;

                        let min_natural_spacing = min.total_width() + pre_shunt_width;
                        if min_natural_spacing > spacing {
                            spacing = min_natural_spacing;
                        }

                        if avoid_collisions_spacing > spacing {
                            spacing = avoid_collisions_spacing;
                        }
                    }
                };

                let note_spacing_per_tick = spacing / entry.duration as f32;
                let end = tick + entry.duration;
                for i in *tick..end {
                    updates.push((i, Slot::NoteSpacing, note_spacing_per_tick));
                }
            }

            updates
        })
        .collect();

    // Apply all updates sequentially
    for (tick, slot, width) in spacing_updates {
        let current = spacings.get(tick, slot);
        if width > current.width {
            spacings.set_width(tick, slot, width);
        }
    }

    let mut x: Space = 0.0;
    for span in &mut spacings.widths {
        span.x = x;
        x += span.width;
    }

    let metrics = BarlineDrawType::Final.metrics();
    spacings.width = x + metrics.total_width();

    spacings
}
