use super::clef::{Clef, ClefDrawType};
use super::Entry;
use crate::score::components::measurements::BoundingBox;
use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::{components::duration::Tick, tracks::Track};
use crate::utils::UniqueId;
use crate::AppState;
use crate::RenderState;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::Ordering;
use tauri::State;

const SHARPS: [Pitch; 7] = [
    Pitch {
        int: 66,
        accidental: Accidental::Sharp,
    },
    Pitch {
        int: 61,
        accidental: Accidental::Sharp,
    },
    Pitch {
        int: 68,
        accidental: Accidental::Sharp,
    },
    Pitch {
        int: 63,
        accidental: Accidental::Sharp,
    },
    Pitch {
        int: 70,
        accidental: Accidental::Sharp,
    },
    Pitch {
        int: 65,
        accidental: Accidental::Sharp,
    },
    Pitch {
        int: 72,
        accidental: Accidental::Sharp,
    },
];

const FLATS: [Pitch; 7] = [
    Pitch {
        int: 70,
        accidental: Accidental::Flat,
    },
    Pitch {
        int: 63,
        accidental: Accidental::Flat,
    },
    Pitch {
        int: 68,
        accidental: Accidental::Flat,
    },
    Pitch {
        int: 61,
        accidental: Accidental::Flat,
    },
    Pitch {
        int: 66,
        accidental: Accidental::Flat,
    },
    Pitch {
        int: 71,
        accidental: Accidental::Flat,
    },
    Pitch {
        int: 64,
        accidental: Accidental::Flat,
    },
];

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

    pub fn accidentals(&self) -> FxHashSet<&Pitch> {
        let mut output = FxHashSet::default();

        if self.offset > 0 {
            for i in 0..self.offset.abs() {
                output.insert(&SHARPS[i as usize]);
            }
        }

        if self.offset < 0 {
            for i in 0..self.offset.abs() {
                output.insert(&FLATS[i as usize]);
            }
        }

        output
    }

    pub fn metrics(&self) -> BoundingBox {
        let width = self.offset.abs() as f32;
        let right_padding = if width > 0.0 { 1.0 } else { 0.0 };

        BoundingBox {
            width,
            height: 4.0,
            right_padding,
        }
    }

    pub fn pattern(&self, clef: &Clef) -> Option<[i8; 7]> {
        match clef.draw_as {
            ClefDrawType::C => match clef.offset {
                -2 => match self.offset.cmp(&0) {
                    Ordering::Greater => Some([2, -2, 1, -3, 0, -4, -1]),
                    Ordering::Less => Some([-1, -4, 0, -3, 1, -2, 2]),
                    Ordering::Equal => None,
                },
                0 => match self.offset.cmp(&0) {
                    Ordering::Greater => Some([-3, 0, -4, -1, 2, -2, 1]),
                    Ordering::Less => Some([1, -2, 2, -1, 3, 0, 4]),
                    Ordering::Equal => None,
                },
                _ => None,
            },
            ClefDrawType::F => match clef.offset {
                -2 => match self.offset.cmp(&0) {
                    Ordering::Greater => Some([-2, 1, -3, 0, 3, -1, 2]),
                    Ordering::Less => Some([2, -1, 3, 0, 4, 1, 5]),
                    Ordering::Equal => None,
                },
                _ => None,
            },
            ClefDrawType::G => match clef.offset {
                2 => match self.offset.cmp(&0) {
                    Ordering::Greater => Some([-4, -1, -5, -2, 1, -3, 0]),
                    Ordering::Less => Some([0, -3, 1, -2, 2, -1, 3]),
                    Ordering::Equal => None,
                },
                _ => None,
            },
            ClefDrawType::Hidden => None,
            ClefDrawType::Percussion => None,
        }
    }

    pub fn glyph(&self) -> String {
        let accidental = match self.offset.cmp(&0) {
            Ordering::Greater => Accidental::Sharp,
            Ordering::Less => Accidental::Flat,
            Ordering::Equal => Accidental::Natural,
        };
        accidental.to_glyph()
    }
}

impl Default for KeySignature {
    fn default() -> Self {
        KeySignature::new(0, 0, KeySignatureMode::Major, 0)
    }
}

impl Track {
    /// Returns the key signature entry at a given tick if it exists
    pub fn get_key_signature_at_tick(&self, at: &Tick) -> Option<&KeySignature> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::KeySignature(key_signature)) = self.entries.by_id.get(key) {
                return Some(key_signature);
            }
        }

        None
    }

    /// Returns the key signature entry *at* or *before* a given tick if it exists
    pub fn get_key_signature_before_tick_inclusive(&self, tick: Tick) -> Option<&KeySignature> {
        for i in (0..tick + 1).rev() {
            if let Some(key_signature) = self.get_key_signature_at_tick(&i) {
                return Some(key_signature);
            };
        }

        None
    }

    /// Returns the key signature entry *before* a given tick if it exists
    pub fn get_key_signature_before_tick(&self, tick: Tick) -> Option<&KeySignature> {
        for i in (0..tick).rev() {
            if let Some(key_signature) = self.get_key_signature_at_tick(&i) {
                return Some(key_signature);
            };
        }

        None
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_key_signature_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    tick: Tick,
    mode: KeySignatureMode,
    offset: i8,
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let key_signature_id = state.generate_id();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    // remove old key signature if defined
    if let Some(old) = master.get_key_signature_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    // insert the new key signature
    let new = KeySignature::new(key_signature_id, tick, mode, offset);
    master.insert_entry(Entry::KeySignature(new));

    state.emit(&app, &mut renderer, true);

    key_signature_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_key_signature_clear(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    tick: Tick,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    if let Some(old) = master.get_key_signature_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    state.emit(&app, &mut renderer, true);
}
