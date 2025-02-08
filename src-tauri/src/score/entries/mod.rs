pub mod barline;
pub mod clef;
pub mod key_signature;
pub mod time_signature;
pub mod tone;

use super::components::duration::Tick;
use crate::utils::UniqueId;
use barline::Barline;
use clef::Clef;
use key_signature::KeySignature;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use time_signature::TimeSignature;
use tone::Tone;

#[derive(Clone, Serialize, Deserialize)]
pub struct Entries {
    pub by_tick: FxHashMap<Tick, Vec<UniqueId>>,
    pub by_id: FxHashMap<UniqueId, Entry>,
}

impl Entries {
    pub fn new() -> Entries {
        Entries {
            by_tick: FxHashMap::default(),
            by_id: FxHashMap::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    Barline(Barline),
    Clef(Clef),
    KeySignature(KeySignature),
    TimeSignature(TimeSignature),
    Tone(Tone),
}

impl Entry {
    /// Get the entries key without having to manually unwrap the entry
    pub fn id(&self) -> UniqueId {
        match self {
            Entry::Barline(barline) => barline.id,
            Entry::Clef(clef) => clef.id,
            Entry::KeySignature(key_signature) => key_signature.id,
            Entry::TimeSignature(time_signature) => time_signature.id,
            Entry::Tone(tone) => tone.id,
        }
    }

    /// Get the entries tick without having to manually unwrap the entry
    pub fn tick(&self) -> Tick {
        match self {
            Entry::Barline(barline) => barline.tick,
            Entry::Clef(clef) => clef.tick,
            Entry::KeySignature(key_signature) => key_signature.tick,
            Entry::TimeSignature(time_signature) => time_signature.tick,
            Entry::Tone(tone) => tone.tick,
        }
    }

    /// Updates the entries tick value
    ///
    /// This shouldn't be called directly. It will most likely be called via the
    /// Track interface.
    pub fn set_tick(&mut self, tick: Tick) {
        match self {
            Entry::Barline(barline) => barline.tick = tick,
            Entry::Clef(clef) => clef.tick = tick,
            Entry::KeySignature(key_signature) => key_signature.tick = tick,
            Entry::TimeSignature(time_signature) => time_signature.tick = tick,
            Entry::Tone(tone) => tone.tick = tick,
        }
    }
}
