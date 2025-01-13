pub mod clef;
pub mod time_signature;

use super::components::duration::Tick;
use crate::utils::UniqueId;
use clef::Clef;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use time_signature::TimeSignature;

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
pub enum Entry {
    Clef(Clef),
    TimeSignature(TimeSignature),
}

impl Entry {
    /// Get the entries key without having to manually unwrap the entry
    pub fn id(&self) -> UniqueId {
        match self {
            Entry::Clef(clef) => clef.id,
            Entry::TimeSignature(time_signature) => time_signature.id,
        }
    }

    /// Get the entries tick without having to manually unwrap the entry
    pub fn tick(&self) -> Tick {
        match self {
            Entry::Clef(clef) => clef.tick,
            Entry::TimeSignature(time_signature) => time_signature.tick,
        }
    }

    /// Updates the entries tick value
    ///
    /// This shouldn't be called directly. It will most likely be called via the
    /// Track interface.
    pub fn set_tick(&mut self, tick: Tick) {
        match self {
            Entry::Clef(clef) => clef.tick = tick,
            Entry::TimeSignature(time_signature) => time_signature.tick = tick,
        }
    }
}
