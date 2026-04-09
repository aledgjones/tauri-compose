pub mod barline;
pub mod clef;
pub mod key_signature;
pub mod playing_technique;
pub mod tempo;
pub mod time_signature;
pub mod tone;
pub mod tuplet;

use super::components::duration::Tick;
use crate::utils::UniqueId;
use barline::Barline;
use clef::Clef;
use key_signature::KeySignature;
use playing_technique::PlayingTechnique;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tempo::Tempo;
use time_signature::TimeSignature;
use tone::Tone;
use tuplet::Tuplet;

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct Entries {
    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
    pub by_tick: FxHashMap<Tick, Vec<UniqueId>>,
    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
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
    PlayingTechnique(PlayingTechnique),
    Tempo(Tempo),
    TimeSignature(TimeSignature),
    Tone(Tone),
    Tuplet(Box<Tuplet>),
}

impl Entry {
    /// Get the entries key without having to manually unwrap the entry
    pub fn id(&self) -> UniqueId {
        match self {
            Entry::Barline(barline) => barline.id,
            Entry::Clef(clef) => clef.id,
            Entry::KeySignature(key_signature) => key_signature.id,
            Entry::PlayingTechnique(playing_technique) => playing_technique.id,
            Entry::Tempo(tempo) => tempo.id,
            Entry::TimeSignature(time_signature) => time_signature.id,
            Entry::Tone(tone) => tone.id,
            Entry::Tuplet(tuplet) => tuplet.id,
        }
    }

    /// Get the entries tick without having to manually unwrap the entry
    pub fn tick(&self) -> Tick {
        match self {
            Entry::Barline(barline) => barline.tick,
            Entry::Clef(clef) => clef.tick,
            Entry::KeySignature(key_signature) => key_signature.tick,
            Entry::PlayingTechnique(playing_technique) => playing_technique.tick,
            Entry::Tempo(tempo) => tempo.tick,
            Entry::TimeSignature(time_signature) => time_signature.tick,
            Entry::Tone(tone) => tone.tick,
            Entry::Tuplet(tuplet) => tuplet.tick,
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
            Entry::PlayingTechnique(playing_technique) => playing_technique.tick = tick,
            Entry::Tempo(tempo) => tempo.tick = tick,
            Entry::TimeSignature(time_signature) => time_signature.tick = tick,
            Entry::Tone(tone) => tone.tick = tick,
            Entry::Tuplet(tuplet) => tuplet.tick = tick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::{
        articulation::Articulations,
        duration::NoteDuration,
        pitch::{Accidental, Pitch},
        velocity::Velocity,
    };
    use barline::{Barline, BarlineDrawType};
    use clef::{Clef, ClefDrawType};
    use key_signature::{KeySignature, KeySignatureMode};
    use playing_technique::{PlayingTechnique, PlayingTechniqueKind};
    use tempo::Tempo;
    use time_signature::{TimeSignature, TimeSignatureDrawType};
    use tone::Tone;
    use tuplet::{Tuplet, TupletEntries, TupletEntry};

    fn make_barline(id: UniqueId, tick: Tick) -> Entry {
        Entry::Barline(Barline {
            id,
            tick,
            barline_draw_type: BarlineDrawType::Single,
        })
    }

    fn make_clef(id: UniqueId, tick: Tick) -> Entry {
        Entry::Clef(Clef::new(
            id,
            tick,
            Pitch::new(67, Accidental::Natural),
            0,
            ClefDrawType::G,
        ))
    }

    fn make_key_signature(id: UniqueId, tick: Tick) -> Entry {
        Entry::KeySignature(KeySignature::new(id, tick, KeySignatureMode::Major, 0))
    }

    fn make_time_signature(id: UniqueId, tick: Tick) -> Entry {
        Entry::TimeSignature(TimeSignature::new(
            id,
            tick,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        ))
    }

    fn make_playing_technique(id: UniqueId, tick: Tick) -> Entry {
        Entry::PlayingTechnique(PlayingTechnique::new(id, tick, PlayingTechniqueKind::Pizz))
    }

    fn make_tempo(id: UniqueId, tick: Tick) -> Entry {
        Entry::Tempo(Tempo::new(
            id,
            tick,
            String::from("Allegro"),
            NoteDuration::Quarter,
            0,
            120,
            false,
            true,
            true,
            false,
        ))
    }

    fn make_tone(id: UniqueId, tick: Tick) -> Entry {
        Entry::Tone(Tone::new(
            id,
            tick,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        ))
    }

    fn make_tuplet(id: UniqueId, tick: Tick) -> Entry {
        let mut entries = TupletEntries::new();
        entries.insert_entry(TupletEntry::Tone(Tone::new(
            id + 100,
            16,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        Entry::Tuplet(Box::new(Tuplet {
            id,
            tick,
            note_count: 3,
            in_space_of: 2,
            base_note_value: NoteDuration::Eighth,
            groupings: vec![3],
            entries,
        }))
    }

    // --- Entries::new() ---

    #[test]
    fn entries_new_is_empty() {
        let entries = Entries::new();
        assert!(entries.by_tick.is_empty());
        assert!(entries.by_id.is_empty());
    }

    // --- Entry::id() ---

    #[test]
    fn barline_id_returns_correct_value() {
        let entry = make_barline(1, 0);
        assert_eq!(entry.id(), 1);
    }

    #[test]
    fn clef_id_returns_correct_value() {
        let entry = make_clef(2, 0);
        assert_eq!(entry.id(), 2);
    }

    #[test]
    fn key_signature_id_returns_correct_value() {
        let entry = make_key_signature(3, 0);
        assert_eq!(entry.id(), 3);
    }

    #[test]
    fn playing_technique_id_returns_correct_value() {
        let entry = make_playing_technique(8, 0);
        assert_eq!(entry.id(), 8);
    }

    #[test]
    fn time_signature_id_returns_correct_value() {
        let entry = make_time_signature(4, 0);
        assert_eq!(entry.id(), 4);
    }

    #[test]
    fn tone_id_returns_correct_value() {
        let entry = make_tone(5, 0);
        assert_eq!(entry.id(), 5);
    }

    #[test]
    fn tempo_id_returns_correct_value() {
        let entry = make_tempo(6, 0);
        assert_eq!(entry.id(), 6);
    }

    #[test]
    fn tuplet_id_returns_correct_value() {
        let entry = make_tuplet(7, 0);
        assert_eq!(entry.id(), 7);
    }

    // --- Entry::tick() ---

    #[test]
    fn barline_tick_returns_correct_value() {
        let entry = make_barline(1, 42);
        assert_eq!(entry.tick(), 42);
    }

    #[test]
    fn clef_tick_returns_correct_value() {
        let entry = make_clef(2, 128);
        assert_eq!(entry.tick(), 128);
    }

    #[test]
    fn key_signature_tick_returns_correct_value() {
        let entry = make_key_signature(3, 256);
        assert_eq!(entry.tick(), 256);
    }

    #[test]
    fn playing_technique_tick_returns_correct_value() {
        let entry = make_playing_technique(8, 320);
        assert_eq!(entry.tick(), 320);
    }

    #[test]
    fn time_signature_tick_returns_correct_value() {
        let entry = make_time_signature(4, 512);
        assert_eq!(entry.tick(), 512);
    }

    #[test]
    fn tone_tick_returns_correct_value() {
        let entry = make_tone(5, 64);
        assert_eq!(entry.tick(), 64);
    }

    #[test]
    fn tempo_tick_returns_correct_value() {
        let entry = make_tempo(6, 96);
        assert_eq!(entry.tick(), 96);
    }

    #[test]
    fn tuplet_tick_returns_correct_value() {
        let entry = make_tuplet(7, 144);
        assert_eq!(entry.tick(), 144);
    }

    // --- Entry::set_tick() ---

    #[test]
    fn set_tick_updates_barline() {
        let mut entry = make_barline(10, 0);
        entry.set_tick(99);
        assert_eq!(entry.tick(), 99);
    }

    #[test]
    fn set_tick_updates_clef() {
        let mut entry = make_clef(11, 0);
        entry.set_tick(200);
        assert_eq!(entry.tick(), 200);
    }

    #[test]
    fn set_tick_updates_key_signature() {
        let mut entry = make_key_signature(12, 0);
        entry.set_tick(50);
        assert_eq!(entry.tick(), 50);
    }

    #[test]
    fn set_tick_updates_playing_technique() {
        let mut entry = make_playing_technique(17, 0);
        entry.set_tick(140);
        assert_eq!(entry.tick(), 140);
    }

    #[test]
    fn set_tick_updates_time_signature() {
        let mut entry = make_time_signature(13, 0);
        entry.set_tick(128);
        assert_eq!(entry.tick(), 128);
    }

    #[test]
    fn set_tick_updates_tone() {
        let mut entry = make_tone(14, 0);
        entry.set_tick(32);
        assert_eq!(entry.tick(), 32);
    }

    #[test]
    fn set_tick_updates_tempo() {
        let mut entry = make_tempo(15, 0);
        entry.set_tick(48);
        assert_eq!(entry.tick(), 48);
    }

    #[test]
    fn set_tick_updates_tuplet() {
        let mut entry = make_tuplet(16, 0);
        entry.set_tick(72);
        assert_eq!(entry.tick(), 72);
    }

    #[test]
    fn serde_round_trip_preserves_supported_tuplet_child_entries() {
        let entry = make_tuplet(17, 64);
        let value = serde_json::to_value(&entry).unwrap();
        let restored: Entry = serde_json::from_value(value).unwrap();

        let Entry::Tuplet(tuplet) = restored else {
            panic!("expected tuplet entry");
        };

        assert_eq!(tuplet.entries.len(), 1);
        assert!(matches!(
            tuplet.entries.get(&117),
            Some(TupletEntry::Tone(_))
        ));
    }

    #[test]
    fn serde_rejects_nested_tuplet_child_entries() {
        let result = serde_json::from_value::<Entry>(serde_json::json!({
            "type": "Tuplet",
            "id": 17,
            "tick": 64,
            "note_count": 3,
            "in_space_of": 2,
            "base_note_value": 8,
            "groupings": [3],
            "entries": {
                "by_tick": {
                    "16": [118]
                },
                "by_id": {
                    "118": {
                        "type": "Tuplet",
                        "id": 118,
                        "tick": 16,
                        "note_count": 3,
                        "in_space_of": 2,
                        "base_note_value": 8,
                        "groupings": [3],
                        "entries": {
                            "by_tick": {},
                            "by_id": {}
                        }
                    }
                }
            }
        }));

        let error = match result {
            Ok(_) => panic!("expected nested tuplet child entries to be rejected"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("Tuplet"));
    }
}
