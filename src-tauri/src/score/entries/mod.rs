pub mod barline;
pub mod clef;
pub mod key_signature;
pub mod tempo;
pub mod time_signature;
pub mod tone;

use super::components::duration::Tick;
use crate::utils::UniqueId;
use barline::Barline;
use clef::Clef;
use key_signature::KeySignature;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tempo::Tempo;
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
    Tempo(Tempo),
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
            Entry::Tempo(tempo) => tempo.id,
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
            Entry::Tempo(tempo) => tempo.tick,
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
            Entry::Tempo(tempo) => tempo.tick = tick,
            Entry::TimeSignature(time_signature) => time_signature.tick = tick,
            Entry::Tone(tone) => tone.tick = tick,
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
    use tempo::Tempo;
    use time_signature::{TimeSignature, TimeSignatureDrawType};
    use tone::Tone;

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
            TimeSignatureDrawType::Hidden,
            None,
        ))
    }

    fn make_tempo(id: UniqueId, tick: Tick) -> Entry {
        Entry::Tempo(Tempo::new(
            id,
            tick,
            Some(String::from("Allegro")),
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
}
