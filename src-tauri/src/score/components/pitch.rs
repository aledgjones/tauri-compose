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

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum Accidental {
    DoubleSharp = 2,
    Sharp = 1,
    Natural = 0,
    Flat = -1,
    DoubleFlat = -2,
}

impl Accidental {
    pub fn to_glyph(self) -> String {
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
    pub fn base(&self) -> u8 {
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

    pub fn shift_by_steps(&self, steps: i8, key_signature: &KeySignature) -> Pitch {
        let mut moved = 0;
        let mut current = self.base();

        while moved < steps.abs() {
            let normalised = current % 12;

            if steps > 0 {
                let one_step = match normalised {
                    4 => 1,
                    11 => 1,
                    _ => 2,
                };

                current += one_step;
            } else {
                let one_step = match normalised {
                    0 => 1,
                    5 => 1,
                    _ => 2,
                };

                current -= one_step;
            }

            moved += 1;
        }

        // check if the base pitch is altered by the key signature to ensure
        // we shift the tone smoothly up the scale.
        for entry in key_signature.accidentals() {
            if current % 12 == entry.base() % 12 {
                // current is the base pitch so set the accidental and offset the base pitch.
                let new_pitch_int = current as i8 + entry.accidental as i8;
                return Pitch::new(new_pitch_int as u8, entry.accidental);
            }
        }

        Pitch::new(current, Accidental::Natural)
    }

    pub fn shift_by_octaves(&self, octaves: i8) -> Pitch {
        let new_int = (self.int as i8) + (octaves * 12);
        Pitch::new(new_int as u8, self.accidental)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};

    fn c_major() -> KeySignature {
        KeySignature::new(0, 0, KeySignatureMode::Major, 0)
    }

    fn g_major() -> KeySignature {
        // 1 sharp (F#)
        KeySignature::new(0, 0, KeySignatureMode::Major, 1)
    }

    // --- Accidental::to_glyph ---

    #[test]
    fn to_glyph_double_sharp() {
        assert_eq!(Accidental::DoubleSharp.to_glyph(), "\u{E263}");
    }

    #[test]
    fn to_glyph_sharp() {
        assert_eq!(Accidental::Sharp.to_glyph(), "\u{E262}");
    }

    #[test]
    fn to_glyph_natural() {
        assert_eq!(Accidental::Natural.to_glyph(), "\u{E261}");
    }

    #[test]
    fn to_glyph_flat() {
        assert_eq!(Accidental::Flat.to_glyph(), "\u{E260}");
    }

    #[test]
    fn to_glyph_double_flat() {
        assert_eq!(Accidental::DoubleFlat.to_glyph(), "\u{E264}");
    }

    // --- Pitch::base ---

    #[test]
    fn base_natural_is_unchanged() {
        assert_eq!(Pitch::new(60, Accidental::Natural).base(), 60);
    }

    #[test]
    fn base_sharp_subtracts_one() {
        assert_eq!(Pitch::new(61, Accidental::Sharp).base(), 60);
    }

    #[test]
    fn base_flat_adds_one() {
        assert_eq!(Pitch::new(61, Accidental::Flat).base(), 62);
    }

    #[test]
    fn base_double_sharp_subtracts_two() {
        assert_eq!(Pitch::new(62, Accidental::DoubleSharp).base(), 60);
    }

    #[test]
    fn base_double_flat_adds_two() {
        assert_eq!(Pitch::new(60, Accidental::DoubleFlat).base(), 62);
    }

    // --- Pitch::half_steps ---

    #[test]
    fn half_steps_c_natural_is_zero() {
        // MIDI 60 = C4; base % 12 = 0
        assert_eq!(Pitch::new(60, Accidental::Natural).half_steps(), 0);
    }

    #[test]
    fn half_steps_d_natural_is_two() {
        assert_eq!(Pitch::new(62, Accidental::Natural).half_steps(), 2);
    }

    // --- Pitch::steps ---

    #[test]
    fn steps_c_is_zero() {
        assert_eq!(Pitch::new(60, Accidental::Natural).steps(), 0);
    }

    #[test]
    fn steps_d_is_one() {
        assert_eq!(Pitch::new(62, Accidental::Natural).steps(), 1);
    }

    #[test]
    fn steps_e_is_two() {
        assert_eq!(Pitch::new(64, Accidental::Natural).steps(), 2);
    }

    #[test]
    fn steps_g_is_four() {
        assert_eq!(Pitch::new(67, Accidental::Natural).steps(), 4);
    }

    // --- Pitch::octave ---

    #[test]
    fn octave_c4() {
        // C0 = 12, C4 = 60 => (60 - 12) / 12 = 4
        assert_eq!(Pitch::new(60, Accidental::Natural).octave(), 4);
    }

    #[test]
    fn octave_c5() {
        assert_eq!(Pitch::new(72, Accidental::Natural).octave(), 5);
    }

    // --- Pitch::steps_between ---

    #[test]
    fn steps_between_c4_to_g4_is_four() {
        let c4 = Pitch::new(60, Accidental::Natural);
        let g4 = Pitch::new(67, Accidental::Natural);
        assert_eq!(Pitch::steps_between(&c4, &g4), 4);
    }

    #[test]
    fn steps_between_c4_to_c5_is_seven() {
        let c4 = Pitch::new(60, Accidental::Natural);
        let c5 = Pitch::new(72, Accidental::Natural);
        assert_eq!(Pitch::steps_between(&c4, &c5), 7);
    }

    #[test]
    fn steps_between_is_negative_descending() {
        let c4 = Pitch::new(60, Accidental::Natural);
        let g4 = Pitch::new(67, Accidental::Natural);
        assert_eq!(Pitch::steps_between(&g4, &c4), -4);
    }

    // --- Pitch::shift_by_octaves ---

    #[test]
    fn shift_by_octaves_up_one() {
        let p = Pitch::new(60, Accidental::Natural);
        let shifted = p.shift_by_octaves(1);
        assert_eq!(shifted.int, 72);
        assert!(shifted.accidental == Accidental::Natural);
    }

    #[test]
    fn shift_by_octaves_down_one() {
        let p = Pitch::new(60, Accidental::Natural);
        let shifted = p.shift_by_octaves(-1);
        assert_eq!(shifted.int, 48);
    }

    // --- Pitch::from_int ---

    #[test]
    fn from_int_c_in_c_major_is_natural() {
        let ks = c_major();
        let p = Pitch::from_int(60, &ks);
        assert_eq!(p.int, 60);
        assert!(p.accidental == Accidental::Natural);
    }

    #[test]
    fn from_int_f_sharp_in_g_major_is_sharp() {
        // G major has one sharp: F# = MIDI 66
        let ks = g_major();
        let p = Pitch::from_int(66, &ks);
        assert!(p.accidental == Accidental::Sharp);
    }

    #[test]
    fn shift_by_steps_up_one_from_c4() {
        // C4 (60) up one step → D4 (62)
        let ks = c_major();
        let c4 = Pitch::new(60, Accidental::Natural);
        let d4 = c4.shift_by_steps(1, &ks);
        assert_eq!(d4.int, 62); // D4 MIDI
    }

    #[test]
    fn shift_by_steps_up_zero_returns_same() {
        let ks = c_major();
        let c4 = Pitch::new(60, Accidental::Natural);
        let result = c4.shift_by_steps(0, &ks);
        assert_eq!(result.int, 60);
    }

    #[test]
    fn shift_by_steps_down_one_from_d4() {
        let ks = c_major();
        let d4 = Pitch::new(62, Accidental::Natural);
        let c4 = d4.shift_by_steps(-1, &ks);
        assert_eq!(c4.int, 60);
    }

    #[test]
    fn shift_by_steps_up_seven_is_octave() {
        let ks = c_major();
        let c4 = Pitch::new(60, Accidental::Natural);
        let c5 = c4.shift_by_steps(7, &ks);
        assert_eq!(c5.int, 72); // C5
    }

    #[test]
    fn shift_by_steps_half_step_boundary_e_to_f() {
        // E4 (64) + 1 step → F4 (65), half-step boundary
        let ks = c_major();
        let e4 = Pitch::new(64, Accidental::Natural);
        let f4 = e4.shift_by_steps(1, &ks);
        assert_eq!(f4.int, 65);
    }

    #[test]
    fn from_int_d_in_c_major_is_natural() {
        let ks = c_major();
        let p = Pitch::from_int(62, &ks);
        assert_eq!(p.int, 62);
        assert!(p.accidental == Accidental::Natural);
    }

    #[test]
    fn from_int_b_flat_in_f_major() {
        // F major has one flat: Bb = MIDI 70
        let ks = KeySignature::new(0, 0, KeySignatureMode::Major, -1); // 1 flat = Bb
        let p = Pitch::from_int(70, &ks);
        assert!(p.accidental == Accidental::Flat);
    }
}
