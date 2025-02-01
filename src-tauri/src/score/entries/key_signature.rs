use crate::score::{components::duration::Tick, tracks::Track};
use crate::utils::UniqueId;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::Entry;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum KeySignatureMode {
    Major,
    Minor,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeySignature {
    pub id: UniqueId,
    pub tick: Tick,
    pub mode: KeySignatureMode,
    pub offset: i8, // + == sharps | - == flats
}

impl KeySignature {
    pub fn new(id: UniqueId, tick: Tick, mode: KeySignatureMode, offset: i8) -> Self {
        Self {
            id,
            tick,
            mode,
            offset,
        }
    }
}

impl Track {
    /// Returns the time signature entry at a given tick if it exists
    pub fn get_key_signature_at_tick(&self, at: &Tick) -> Option<&KeySignature> {
        let entry_keys = match self.entries.by_tick.get(at) {
            Some(entries) => entries,
            None => return None,
        };

        for key in entry_keys {
            if let Some(Entry::KeySignature(key_signature)) = self.entries.by_id.get(key) {
                return Some(key_signature);
            }
        }

        None
    }

    /// Returns the time signature entry *before* a given tick if it exists
    pub fn get_key_signature_before_tick_inclusive(&self, tick: Tick) -> Option<&KeySignature> {
        for i in (0..tick + 1).rev() {
            if let Some(key_signature) = self.get_key_signature_at_tick(&i) {
                return Some(key_signature);
            };
        }

        None
    }
}
