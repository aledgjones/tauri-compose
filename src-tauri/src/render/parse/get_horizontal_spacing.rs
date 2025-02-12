use super::get_barlines::Barlines;
use super::get_key_signatures::KeySignatures;
use crate::renderer::converter::Space;
use crate::score::components::duration::{Tick, Ticks};
use crate::score::entries::barline::BarlineDrawType;
use crate::score::flows::Flow;

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

pub struct HorizontalSpacing {
    pub widths: Vec<Space>,
}

impl HorizontalSpacing {
    pub fn new(length: Ticks) -> HorizontalSpacing {
        HorizontalSpacing {
            widths: vec![0.0; length as usize * Slot::count()],
        }
    }

    pub fn width(&self) -> Space {
        let width: Space = self.widths.iter().sum();

        width + BarlineDrawType::Final.metrics().width
    }

    pub fn get(&self, tick: Tick, slot: Slot) -> Space {
        let index = (tick as usize * Slot::count()) + slot as usize;

        self.widths[index]
    }

    pub fn set(&mut self, tick: Tick, slot: Slot, width: Space) {
        let index = (tick as usize * Slot::count()) + slot as usize;
        self.widths[index] = width;
    }
}

pub fn get_horizontal_spacing(
    flow: &Flow,
    barlines: &Barlines,
    key_signatures: &KeySignatures,
) -> HorizontalSpacing {
    let mut spacings = HorizontalSpacing::new(flow.length);

    for tick in 0..flow.length {
        // TEMP: make this dynamically spaced ------------
        spacings.set(tick, Slot::NoteSpacing, 0.02);

        if tick == 0 {
            spacings.set(tick, Slot::PaddingStart, 1.0);
        }

        // BARLINES
        if let Some(def) = barlines.get(&tick) {
            if def.start_repeat {
                let metrics = BarlineDrawType::StartRepeat.metrics();
                spacings.set(tick, Slot::StartRepeat, metrics.width);
            }

            if let Some(draw_type) = def.draw_type {
                let metrics = draw_type.metrics();
                spacings.set(tick, Slot::Barline, metrics.width);
            }

            if def.end_repeat {
                let metrics = BarlineDrawType::EndRepeat.metrics();
                spacings.set(tick, Slot::EndRepeat, metrics.width);
            }
        }

        // KEY SIGNATURES
        if let Some(def) = key_signatures.get(&tick) {
            spacings.set(tick, Slot::KeySignature, def.key_signature.metrics().width);
        };
    }

    spacings
}
