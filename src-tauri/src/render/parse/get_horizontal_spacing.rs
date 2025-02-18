use super::get_barlines::Barlines;
use super::get_key_signatures::KeySignatures;
use crate::renderer::converter::Space;
use crate::score::components::duration::{Tick, Ticks};
use crate::score::entries::barline::BarlineDrawType;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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

pub fn get_horizontal_spacing(
    flow: &Flow,
    barlines: &Barlines,
    key_signatures: &KeySignatures,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
) -> HorizontalSpacing {
    let mut spacings = HorizontalSpacing::new(flow.length);
    let flow_master = &tracks.by_id[&flow.master_id];

    for tick in 0..flow.length {
        // TEMP: make this dynamically spaced ------------
        spacings.set_width(tick, Slot::NoteSpacing, 0.02);

        if tick == 0 {
            spacings.set_width(tick, Slot::PaddingStart, 0.5);
        }

        // BARLINES
        if let Some(def) = barlines.get(&tick) {
            if def.start_repeat {
                let metrics = BarlineDrawType::StartRepeat.metrics();
                spacings.set_width(tick, Slot::StartRepeat, metrics.width);
            }

            if let Some(draw_type) = def.draw_type {
                let metrics = draw_type.metrics();
                spacings.set_width(tick, Slot::Barline, metrics.width);
            }

            if def.end_repeat {
                let metrics = BarlineDrawType::EndRepeat.metrics();
                spacings.set_width(tick, Slot::EndRepeat, metrics.width);
            }
        }

        // KEY SIGNATURES
        if let Some(def) = key_signatures.get(&tick) {
            spacings.set_width(tick, Slot::KeySignature, def.key_signature.metrics().width);
        };

        // TIME SIGNATURE
        if let Some(time_signature) = flow_master.get_time_signature_at_tick(&tick) {
            spacings.set_width(tick, Slot::TimeSignature, time_signature.metrics().width);
        }
    }

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &tracks.by_id[&stave.master_id];

            for tick in 0..flow.length {
                if let Some(clef) = stave_master.get_clef_at_tick(&tick) {
                    spacings.set_width(tick, Slot::Clef, clef.metrics(tick == 0).width);
                }
            }
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
