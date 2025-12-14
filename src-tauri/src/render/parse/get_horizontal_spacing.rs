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
    Accidentals, // only used if there is no previous space to cut into.
    PreNote,     // only used if there is no previous space to cut into.
    NoteSpacing,
}

impl Slot {
    pub fn count() -> usize {
        Slot::NoteSpacing as usize + 1
    }

    pub fn from_usize(value: usize) -> Self {
        match value {
            0 => Slot::PaddingStart,
            1 => Slot::EndRepeat,
            2 => Slot::Clef,
            3 => Slot::Barline,
            4 => Slot::KeySignature,
            5 => Slot::TimeSignature,
            6 => Slot::StartRepeat,
            7 => Slot::Accidentals,
            8 => Slot::PreNote,
            9 => Slot::NoteSpacing,
            _ => Slot::PaddingStart,
        }
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
                        },
                    }
                } else {
                    key_signature.metrics()
                };
                spacings.set_width(key_signature.tick, Slot::KeySignature, metrics.width);
            }
            Entry::TimeSignature(time_signature) => {
                let metrics = time_signature.metrics();
                spacings.set_width(time_signature.tick, Slot::TimeSignature, metrics.width);
            }
            _ => (),
        }
    }

    for (tick, def) in barlines {
        let time = flow_master.get_time_signature_at_tick(tick);
        let key = flow_master.get_key_signature_at_tick(tick);

        if def.end_repeat {
            let metrics = BarlineDrawType::EndRepeat.metrics();
            spacings.set_width(*tick, Slot::EndRepeat, metrics.width);
        }

        if let Some(draw_type) = &def.draw_type {
            let metrics = draw_type.metrics();
            spacings.set_width(*tick, Slot::Barline, metrics.width);
        }

        if def.start_repeat {
            let metrics = BarlineDrawType::StartRepeat.metrics();
            spacings.set_width(*tick, Slot::StartRepeat, metrics.width);
        }
    }

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &tracks.by_id[&stave.master_id];
            for entry in stave_master.entries.by_id.values() {
                if let Entry::Clef(clef) = entry {
                    let metrics = clef.metrics(clef.tick == 0);
                    spacings.set_width(clef.tick, Slot::Clef, metrics.width);
                }
            }
        }
    }

    let spacing_updates: Vec<(Tick, Slot, Space, bool)> = notations_by_track.par_iter().flat_map(|(track_key, notation)| {
        let shunts = &shunts_by_track[track_key];
        let accidentals = &accidentals_by_track[track_key];
        let beams = &beams_by_track[track_key];
        let mut updates = Vec::new();

        for (tick, entry) in &notation.track {
            if entry.has_pre_shunt(shunts) {
                updates.push((*tick, Slot::PreNote, entry.glyph_metrics().width, false)); // false means always set
            }

            let mut spacing = entry
                .metrics(shunts, flow.subdivisions, engrave, beams)
                .width;

            // ACCIDENTALS

            // first beat acccidental sapcing
            if *tick == 0 || barlines.contains_key(tick) {
                // start of bars has no previous spacing to extend so we use the accidentals slot
                if let Some(slots) = accidentals.slots_by_tick.get(tick) {
                    let width = (*slots as f32) * 1.1;
                    updates.push((*tick, Slot::Accidentals, width, true)); // true means set only if larger
                };
            }

            // extend spacing to accomodate accidentals + pre shunts (if needed)
            if let Some((next_tick, next_entry)) = notation.get_next_notation(*tick) {
                if !barlines.contains_key(&next_tick) {
                    // natural minimum for note value
                    let min = entry.min_spacing(shunts, flow.subdivisions, engrave, beams);

                    let post_shunt = match entry.has_post_shunt(shunts) {
                        true => next_entry.glyph_metrics().width,
                        false => 0.0,
                    };

                    let pre_shunt = match next_entry.has_pre_shunt(shunts) {
                        true => next_entry.glyph_metrics().width,
                        false => 0.0,
                    };

                    let accidentals_width = match accidentals.slots_by_tick.get(&next_tick) {
                        Some(slots) => {
                            // slots + 1.5 notehead padding
                            ((*slots as f32 * 1.1) - pre_shunt + post_shunt)
                                + (entry.glyph_metrics().width * 1.5)
                        }
                        None => 0.0,
                    };

                    if min > spacing {
                        spacing = min;
                    }

                    if accidentals_width > spacing {
                        spacing = accidentals_width;
                    }
                }
            };

            let note_spacing_per_tick = spacing / entry.duration as f32;
            let end = tick + entry.duration;
            for i in *tick..end {
                updates.push((i, Slot::NoteSpacing, note_spacing_per_tick, true)); // true means set only if larger
            }
        }

        updates
    }).collect();

    // Apply all updates sequentially
    for (tick, slot, width, check_larger) in spacing_updates {
        if check_larger {
            let current = spacings.get(tick, slot);
            if width > current.width {
                spacings.set_width(tick, slot, width);
            }
        } else {
            spacings.set_width(tick, slot, width);
        }
    }

    let mut x: Space = 0.0;
    for span in &mut spacings.widths {
        span.x = x;
        x += span.width;
    }

    spacings.width = x + BarlineDrawType::Final.metrics().width;

    spacings
}
