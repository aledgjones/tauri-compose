use super::clef::{Clef, ClefDrawType};
use super::Entry;
use crate::score::components::measurements::BoundingBox;
use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::{components::duration::Tick, tracks::Track};
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;
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
    pub hidden: bool,
    pub mode: KeySignatureMode,
    pub offset: i8, // + == sharps | - == flats
}

impl KeySignature {
    pub fn new(id: UniqueId, tick: Tick, mode: KeySignatureMode, offset: i8) -> Self {
        Self {
            id,
            tick,
            hidden: false,
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
        if self.hidden {
            return BoundingBox {
                width: 0.0,
                height: 4.0,
                right_padding: 0.0,
            };
        }

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

fn clear_key_signature_at_tick(master: &mut Track, tick: Tick) -> bool {
    let Some(found) = master.get_key_signature_at_tick(&tick).cloned() else {
        return false;
    };

    if tick == 0 {
        if let Some(Entry::KeySignature(key_signature)) = master.entries.by_id.get_mut(&found.id) {
            key_signature.mode = KeySignatureMode::Major;
            key_signature.offset = 0;
            key_signature.hidden = false;
            return true;
        }

        return false;
    }

    master.remove_entry(found.id).is_some()
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_key_signature_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    tick: Tick,
    mode: KeySignatureMode,
    offset: i8,
    hidden: bool,
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
    let mut new = KeySignature::new(key_signature_id, tick, mode, offset);
    new.hidden = hidden;
    master.insert_entry(Entry::KeySignature(new));

    state.emit_affected_flows(&app, &mut renderer, &[flow_id]);

    key_signature_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_key_signature_clear(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    tick: Tick,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    clear_key_signature_at_tick(master, tick);
    state.ui.sequencer.selection = None;

    state.emit_affected_flows(&app, &mut renderer, &[flow_id]);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_key_signature_update(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    key_signature_id: UniqueId,
    hidden: bool,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let master_id = state
        .score
        .flows
        .by_id
        .get(&flow_id)
        .map(|flow| flow.master_id)
        .ok_or_else(|| String::from("Flow not found"))?;

    let entry = state
        .score
        .tracks
        .by_id
        .get_mut(&master_id)
        .ok_or_else(|| String::from("Key signature track not found"))?
        .entries
        .by_id
        .get_mut(&key_signature_id)
        .ok_or_else(|| String::from("Key signature not found"))?;

    let Entry::KeySignature(key_signature) = entry else {
        return Err(String::from("Selected entry is not a key signature"));
    };

    key_signature.hidden = hidden;

    state.emit_affected_flows(&app, &mut renderer, &[flow_id]);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_signature_new_stores_fields() {
        let ks = KeySignature::new(10, 64, KeySignatureMode::Major, 2);
        assert_eq!(ks.id, 10);
        assert_eq!(ks.tick, 64);
        assert_eq!(ks.offset, 2);
    }

    #[test]
    fn accidentals_one_sharp_has_one_pitch() {
        // G major = 1 sharp
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);
        assert_eq!(ks.accidentals().len(), 1);
    }

    #[test]
    fn accidentals_two_flats_has_two_pitches() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -2);
        assert_eq!(ks.accidentals().len(), 2);
    }

    #[test]
    fn accidentals_c_major_is_empty() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 0);
        assert!(ks.accidentals().is_empty());
    }

    #[test]
    fn metrics_width_equals_abs_offset() {
        let ks_sharp = KeySignature::new(0, 0, KeySignatureMode::Major, 3);
        let bb = ks_sharp.metrics();
        assert!((bb.width - 3.0).abs() < f32::EPSILON);
        assert!((bb.right_padding - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn metrics_zero_offset_has_zero_padding() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 0);
        let bb = ks.metrics();
        assert!((bb.right_padding - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn glyph_positive_offset_returns_sharp_glyph() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 2);
        assert_eq!(ks.glyph(), "\u{E262}");
    }

    #[test]
    fn glyph_negative_offset_returns_flat_glyph() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -2);
        assert_eq!(ks.glyph(), "\u{E260}");
    }

    #[test]
    fn default_key_signature_is_c_major() {
        let ks = KeySignature::default();
        assert_eq!(ks.offset, 0);
    }

    #[test]
    fn glyph_zero_offset_returns_natural_glyph() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 0);
        assert_eq!(ks.glyph(), "\u{E261}");
    }

    #[test]
    fn accidentals_all_seven_sharps() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 7);
        assert_eq!(ks.accidentals().len(), 7);
    }

    #[test]
    fn clear_key_signature_at_tick_removes_existing_entry() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::KeySignature(KeySignature::new(
            11,
            64,
            KeySignatureMode::Major,
            2,
        )));

        assert!(clear_key_signature_at_tick(&mut track, 64));
        assert!(track.get_key_signature_at_tick(&64).is_none());
    }

    #[test]
    fn clear_key_signature_at_tick_restores_opening_signature_to_c_major() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::KeySignature(KeySignature::new(
            11,
            0,
            KeySignatureMode::Minor,
            -3,
        )));

        assert!(clear_key_signature_at_tick(&mut track, 0));

        let restored = track.get_key_signature_at_tick(&0).unwrap();
        assert_eq!(restored.id, 11);
        assert!(matches!(restored.mode, KeySignatureMode::Major));
        assert_eq!(restored.offset, 0);
        assert!(!restored.hidden);
    }

    #[test]
    fn accidentals_all_seven_flats() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -7);
        assert_eq!(ks.accidentals().len(), 7);
    }

    #[test]
    fn metrics_flats_has_padded_width() {
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -3);
        let bb = ks.metrics();
        assert!((bb.width - 3.0).abs() < f32::EPSILON);
        assert!((bb.right_padding - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pattern_g_clef_sharps_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(67, Accidental::Natural),
            2,
            ClefDrawType::G,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_g_clef_flats_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(67, Accidental::Natural),
            2,
            ClefDrawType::G,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_c_major_returns_none() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 0);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(67, Accidental::Natural),
            2,
            ClefDrawType::G,
        );
        assert!(ks.pattern(&clef).is_none());
    }

    #[test]
    fn pattern_percussion_clef_returns_none() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 2);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::Percussion,
        );
        assert!(ks.pattern(&clef).is_none());
    }

    #[test]
    fn pattern_f_clef_offset_minus_two_sharps_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(53, Accidental::Natural),
            -2,
            ClefDrawType::F,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_f_clef_offset_minus_two_flats_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(53, Accidental::Natural),
            -2,
            ClefDrawType::F,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_c_clef_offset_zero_sharps_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::C,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_c_clef_offset_zero_flats_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::C,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_c_clef_offset_minus_two_sharps_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(60, Accidental::Natural),
            -2,
            ClefDrawType::C,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_c_clef_offset_minus_two_flats_returns_some() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(60, Accidental::Natural),
            -2,
            ClefDrawType::C,
        );
        assert!(ks.pattern(&clef).is_some());
    }

    #[test]
    fn pattern_unknown_clef_offset_returns_none() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};
        // G clef with offset != 2 → None
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, 1);
        let clef = Clef::new(
            0,
            0,
            Pitch::new(67, Accidental::Natural),
            0,
            ClefDrawType::G,
        );
        assert!(ks.pattern(&clef).is_none());
    }

    #[test]
    fn get_key_signature_at_tick_returns_none_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        assert!(track.get_key_signature_at_tick(&0).is_none());
    }

    #[test]
    fn get_key_signature_at_tick_returns_when_present() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ks = KeySignature::new(5, 64, KeySignatureMode::Major, 2);
        track.insert_entry(Entry::KeySignature(ks));
        let found = track.get_key_signature_at_tick(&64).unwrap();
        assert_eq!(found.id, 5);
    }

    #[test]
    fn get_key_signature_before_tick_inclusive_returns_ks_at_same_tick() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ks = KeySignature::new(5, 64, KeySignatureMode::Major, 2);
        track.insert_entry(Entry::KeySignature(ks));
        let found = track.get_key_signature_before_tick_inclusive(64);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 5);
    }

    #[test]
    fn get_key_signature_before_tick_inclusive_returns_earlier_ks() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ks = KeySignature::new(5, 0, KeySignatureMode::Major, 2);
        track.insert_entry(Entry::KeySignature(ks));
        let found = track.get_key_signature_before_tick_inclusive(64);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 5);
    }

    #[test]
    fn get_key_signature_before_tick_inclusive_returns_none_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        assert!(track.get_key_signature_before_tick_inclusive(64).is_none());
    }

    #[test]
    fn get_key_signature_before_tick_excludes_same_tick() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ks = KeySignature::new(5, 64, KeySignatureMode::Major, 2);
        track.insert_entry(Entry::KeySignature(ks));
        // before_tick(64) checks 0..64, ks is at 64 → not found
        let found = track.get_key_signature_before_tick(64);
        assert!(found.is_none());
    }

    #[test]
    fn get_key_signature_before_tick_returns_earlier_ks() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ks = KeySignature::new(5, 0, KeySignatureMode::Major, 2);
        track.insert_entry(Entry::KeySignature(ks));
        let found = track.get_key_signature_before_tick(64);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 5);
    }

    #[test]
    fn get_key_signature_before_tick_returns_none_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        assert!(track.get_key_signature_before_tick(64).is_none());
    }
}
