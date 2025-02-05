use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

const C0: u8 = 12;

lazy_static! {
    pub static ref ACCIDENTAL_MAP: FxHashMap<(i8, KeySignatureMode), [Accidental; 12]> = {
        let mut map = FxHashMap::default();

        // C# Major
        map.insert(
            (7, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // A# Minor
        map.insert(
            (7, KeySignatureMode::Minor),
            [
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // F# Major
        map.insert(
            (6, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // D# Minor
        map.insert(
            (6, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // B Major
        map.insert(
            (5, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // G# Minor
        map.insert(
            (5, KeySignatureMode::Minor),
            [
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // E Major
        map.insert(
            (4, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // C# Minor
        map.insert(
            (4, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::DoubleSharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // A Major
        map.insert(
            (3, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // F# Minor
        map.insert(
            (3, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // D Major
        map.insert(
            (2, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // B Minor
        map.insert(
            (2, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // G Major
        map.insert(
            (1, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // E Minor
        map.insert(
            (1, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
            ],
        );

        // C Major
        map.insert(
            (0, KeySignatureMode::Major),
            [
                Accidental::Natural, // C
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // A Minor
        map.insert(
            (0, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // F Major
        map.insert(
            (-1, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // D Minor
        map.insert(
            (-1, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // B♭ Major
        map.insert(
            (-2, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // G Minor
        map.insert(
            (-2, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // E♭ Major
        map.insert(
            (-3, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        // C Minor
        map.insert(
            (-3, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Sharp,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // A♭ Major
        map.insert(
            (-4, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        // F Minor
        map.insert(
            (-4, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
            ],
        );

        // D♭ Major
        map.insert(
            (-5, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::DoubleFlat,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        // B♭ Minor
        map.insert(
            (-5, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        // G♭ Major
        map.insert(
            (-6, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::DoubleFlat,
                Accidental::Flat,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::DoubleFlat,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        // E♭ Minor
        map.insert(
            (-6, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        // C♭ Major
        map.insert(
            (-7, KeySignatureMode::Major),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::DoubleFlat,
                Accidental::Flat,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::DoubleFlat,
                Accidental::Flat,
                Accidental::DoubleFlat,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        // A♭ Minor
        map.insert(
            (-7, KeySignatureMode::Minor),
            [
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::Natural,
                Accidental::Flat,
                Accidental::DoubleFlat,
                Accidental::Flat,
                Accidental::Flat,
            ],
        );

        map
    };
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum Accidental {
    DoubleSharp = 2,
    Sharp = 1,
    Natural = 0,
    Flat = -1,
    DoubleFlat = -2,
}

impl Accidental {
    pub fn to_glyph(&self) -> String {
        match self {
            Accidental::DoubleSharp => String::from("\u{E263}"),
            Accidental::Sharp => String::from("\u{E262}"),
            Accidental::Natural => String::from("\u{E261}"),
            Accidental::Flat => String::from("\u{E260}"),
            Accidental::DoubleFlat => String::from("\u{E264}"),
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Pitch {
    pub int: u8, // the midi number
    pub accidental: Accidental,
}

impl Pitch {
    pub fn new(int: u8, accidental: Accidental) -> Self {
        Pitch { int, accidental }
    }

    pub fn from_int(int: u8, key_signature: &KeySignature) -> Self {
        let pitch_class = (int % 12) as usize;

        let accidental = match ACCIDENTAL_MAP.get(&(key_signature.offset, key_signature.mode)) {
            Some(mapping) => mapping[pitch_class],
            None => Accidental::Natural,
        };

        Pitch { int, accidental }
    }

    /// gets the base note pitch (natural) for an accidental note
    /// ie 61 (C#) -> 60 (C), 61 (D flat) -> 62 (D)
    fn base(&self) -> u8 {
        let int = self.int;
        match self.accidental {
            Accidental::DoubleSharp => int - 2,
            Accidental::Sharp => int - 1,
            Accidental::Natural => int,
            Accidental::Flat => int + 1,
            Accidental::DoubleFlat => int + 2,
        }
    }

    /// gets the number of semitones the tone is from C
    pub fn half_steps(&self) -> u8 {
        self.base() % 12
    }

    /// gets the number of steps the tone is from C
    pub fn steps(&self) -> u8 {
        match self.half_steps() {
            0 => 0,
            2 => 1,
            4 => 2,
            5 => 3,
            7 => 4,
            9 => 5,
            11 => 6,
            _ => 0,
        }
    }

    pub fn octave(&self) -> u8 {
        let natural = self.base();
        (natural - C0) / 12
    }

    pub fn steps_between(a: &Pitch, b: &Pitch) -> i8 {
        let octave_offset = (b.octave() as i8 - a.octave() as i8) * 7;
        octave_offset + b.steps() as i8 - a.steps() as i8
    }
}
