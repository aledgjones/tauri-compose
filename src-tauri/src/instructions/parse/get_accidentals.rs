use super::get_barlines::Barlines;
use super::get_shunts::{Shunt, Shunts, ShuntsByTrack};
use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::{Notation, NotationByTrack, NotationTrack};
use crate::score::components::duration::Tick;
use crate::score::components::pitch::Accidental;
use crate::score::entries::{key_signature::KeySignature, tone::Tone};
use crate::score::flows::Flow;
use crate::score::tracks::{Track, Tracks};
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

type SlotsByTick = FxHashMap<Tick, u8>;

pub struct AccidentalNotation {
    pub tone_id: UniqueId,
    pub slot: u8,
}

pub struct Accidentals {
    pub by_id: FxHashMap<(Tick, UniqueId), AccidentalNotation>,
    pub slots_by_tick: SlotsByTick,
}

impl Accidentals {
    pub fn new() -> Self {
        Self {
            by_id: FxHashMap::default(),
            slots_by_tick: FxHashMap::default(),
        }
    }
}

impl Default for Accidentals {
    fn default() -> Self {
        Self::new()
    }
}

pub type AccidentalsByTrack = FxHashMap<UniqueId, Accidentals>;

pub fn fits_in_slot(
    tick: Tick,
    slot: u8,
    accidental_offset: i8,
    slots: &[AccidentalNotation],
    tone_offsets: &ToneVerticalOffsets,
    shunts: &Shunts,
) -> bool {
    if slot == 1 {
        for offset in (accidental_offset - 4)..(accidental_offset + 4) {
            if let Some(Shunt::Pre) = shunts.by_offset.get(&(tick, offset)) {
                return false;
            }
        }
    }

    // avoid other accidentals
    for entry in slots {
        if entry.slot == slot {
            let entry_offset = tone_offsets[&entry.tone_id];
            let diff = (entry_offset - accidental_offset).abs();
            if diff < 6 {
                return false;
            }
        };
    }
    true
}

pub fn find_slot(
    tick: Tick,
    tone_id: UniqueId,
    slots: &[AccidentalNotation],
    tone_offsets: &ToneVerticalOffsets,
    shunts: &Shunts,
) -> u8 {
    let accidental_offset = tone_offsets[&tone_id];

    let mut slot: u8 = 1;
    loop {
        if fits_in_slot(tick, slot, accidental_offset, slots, tone_offsets, shunts) {
            return slot;
        } else {
            slot += 1;
        }
    }
}

pub fn sort_accidentals_alternate(accidentals: &[UniqueId]) -> Vec<UniqueId> {
    let mut output: Vec<UniqueId> = Vec::new();

    let mut i = 0;
    let mut j = accidentals.len() - 1;
    while i < j {
        output.push(accidentals[j]);
        output.push(accidentals[i]);
        j -= 1;
        i += 1;
    }

    if !accidentals.len().is_multiple_of(2) {
        output.push(accidentals[i]);
    }

    output
}

pub fn is_accidental_needed(
    tone: &Tone,
    tones: &Vec<Tone>,
    previous_tones: &FxHashSet<UniqueId>,
    altered_base_pitches: &FxHashMap<u8, Accidental>,
    key_signature: &KeySignature,
) -> bool {
    // ignore tied notes (the tones would have already been seen)
    if previous_tones.contains(&tone.id) {
        return false;
    }

    // if the tone shares the same note letter as another tone reuiring a different
    // accideantal, we draw all these accidentals.
    for other_tone in tones {
        if tone.id != other_tone.id
            && tone.pitch.base() % 12 == other_tone.pitch.base() % 12
            && tone.pitch.accidental != other_tone.pitch.accidental
        {
            return true;
        }
    }

    // altered accidentals
    if let Some(&accidental) = altered_base_pitches.get(&tone.pitch.base()) {
        return tone.pitch.accidental != accidental;
    }

    // key accidentals
    for pitch in key_signature.accidentals() {
        // not octave aware
        if tone.pitch.base() % 12 == pitch.base() % 12 {
            return tone.pitch.accidental != pitch.accidental;
        }
    }

    // finally if it's just an accidental not covered by the above cases
    if tone.pitch.accidental != Accidental::Natural {
        return true;
    }

    false
}

/// returns the *unordered* tones needing accidentals
pub fn get_tones_needing_accidentals(
    entry: &Notation,
    previous_tones: &mut FxHashSet<UniqueId>,
    altered_base_pitches: &mut FxHashMap<u8, Accidental>,
    key_signature: &KeySignature,
) -> Vec<UniqueId> {
    let mut output: Vec<UniqueId> = Vec::new();

    for tone in &entry.tones {
        if is_accidental_needed(
            tone,
            &entry.tones,
            previous_tones,
            altered_base_pitches,
            key_signature,
        ) {
            output.push(tone.id);
            altered_base_pitches.insert(tone.pitch.base(), tone.pitch.accidental);
        }
        previous_tones.insert(tone.id);
    }

    output
}

pub fn get_accidentals_in_track(
    notation: &NotationTrack,
    master: &Track,
    barlines: &Barlines,
    tone_offsets: &ToneVerticalOffsets,
    shunts: &Shunts,
) -> Accidentals {
    let mut output = Accidentals::new();

    let mut key_signature = &KeySignature::default();
    let mut altered_base_pitches: FxHashMap<u8, Accidental> = FxHashMap::default();
    let mut previous_tones: FxHashSet<UniqueId> = FxHashSet::default();

    for tick in 0..notation.length {
        // look for a key signature at this tick
        let found = match master.get_key_signature_at_tick(&tick) {
            Some(entry) => {
                key_signature = entry;
                true
            }
            None => false,
        };

        // clear alterations at barlines and key changes
        if found || barlines.contains_key(&tick) {
            altered_base_pitches.clear();
        }

        let entry = match notation.track.get(&tick) {
            Some(entry) => entry,
            None => {
                continue;
            }
        };

        let accidentals = get_tones_needing_accidentals(
            entry,
            &mut previous_tones,
            &mut altered_base_pitches,
            key_signature,
        );

        if accidentals.is_empty() {
            continue;
        }

        let mut slots: Vec<AccidentalNotation> = Vec::new();
        for tone_id in sort_accidentals_alternate(&accidentals) {
            let slot = find_slot(tick, tone_id, &slots, tone_offsets, shunts);
            slots.push(AccidentalNotation { tone_id, slot });
        }

        let mut max_slot = 0;
        for entry in slots {
            if entry.slot > max_slot {
                max_slot = entry.slot;
            }
            output.by_id.insert((tick, entry.tone_id), entry);
        }
        output.slots_by_tick.insert(tick, max_slot);
    }

    output
}

pub fn get_accidentals(
    flow: &Flow,
    tracks: &Tracks,
    notation_by_track: &NotationByTrack,
    bars: &Barlines,
    tone_offsets: &ToneVerticalOffsets,
    shunts_by_track: &ShuntsByTrack,
) -> AccidentalsByTrack {
    let master = &tracks.by_id[&flow.master_id];

    let results: Vec<(UniqueId, _)> = notation_by_track
        .par_iter()
        .map(|(track_id, notation)| {
            let shunts = &shunts_by_track[track_id];
            let accidentals =
                get_accidentals_in_track(notation, master, bars, tone_offsets, shunts);
            (*track_id, accidentals)
        })
        .collect();

    let mut output: AccidentalsByTrack = FxHashMap::default();
    for (track_id, accidentals) in results {
        output.insert(track_id, accidentals);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::pitch::Pitch;
    use crate::score::entries::key_signature::KeySignatureMode;

    #[test]
    fn accidentals_new_is_empty() {
        let a = Accidentals::new();
        assert!(a.by_id.is_empty());
        assert!(a.slots_by_tick.is_empty());
    }

    #[test]
    fn accidentals_default_is_empty() {
        let a = Accidentals::default();
        assert!(a.by_id.is_empty());
        assert!(a.slots_by_tick.is_empty());
    }

    #[test]
    fn accidental_notation_stores_fields() {
        let an = AccidentalNotation {
            tone_id: 42,
            slot: 3,
        };
        assert_eq!(an.tone_id, 42);
        assert_eq!(an.slot, 3);
    }

    // --- sort_accidentals_alternate ---

    #[test]
    fn sort_accidentals_alternate_even_count() {
        let ids = vec![1u64, 2, 3, 4];
        let result = sort_accidentals_alternate(&ids);
        assert_eq!(result.len(), 4);
        // Must contain all original ids
        for id in &ids {
            assert!(result.contains(id));
        }
    }

    #[test]
    fn sort_accidentals_alternate_odd_count() {
        let ids = vec![1u64, 2, 3];
        let result = sort_accidentals_alternate(&ids);
        assert_eq!(result.len(), 3);
        for id in &ids {
            assert!(result.contains(id));
        }
    }

    #[test]
    fn sort_accidentals_alternate_single_element() {
        let ids = vec![99u64];
        let result = sort_accidentals_alternate(&ids);
        assert_eq!(result, vec![99u64]);
    }

    // --- is_accidental_needed ---

    fn make_test_tone(id: u64, pitch_int: u8, accidental: Accidental) -> Tone {
        use crate::score::components::{articulation::Articulations, velocity::Velocity};
        Tone::new(
            id,
            0,
            32,
            Pitch::new(pitch_int, accidental),
            Velocity::new(80),
            Articulations::new(),
        )
    }

    #[test]
    fn natural_in_c_major_needs_no_accidental() {
        use rustc_hash::{FxHashMap, FxHashSet};
        let tone = make_test_tone(1, 60, Accidental::Natural); // C natural
        let key_sig = KeySignature::default(); // C major, offset=0
        let previous_tones: FxHashSet<u64> = FxHashSet::default();
        let altered: FxHashMap<u8, Accidental> = FxHashMap::default();
        assert!(!is_accidental_needed(
            &tone,
            &vec![tone.clone()],
            &previous_tones,
            &altered,
            &key_sig
        ));
    }

    #[test]
    fn sharp_not_in_key_needs_accidental() {
        use rustc_hash::{FxHashMap, FxHashSet};
        let tone = make_test_tone(1, 61, Accidental::Sharp); // C# not in C major
        let key_sig = KeySignature::default(); // C major
        let previous_tones: FxHashSet<u64> = FxHashSet::default();
        let altered: FxHashMap<u8, Accidental> = FxHashMap::default();
        assert!(is_accidental_needed(
            &tone,
            &vec![tone.clone()],
            &previous_tones,
            &altered,
            &key_sig
        ));
    }

    #[test]
    fn tied_tone_needs_no_accidental() {
        use rustc_hash::{FxHashMap, FxHashSet};
        let tone = make_test_tone(1, 61, Accidental::Sharp);
        let key_sig = KeySignature::default();
        let mut previous_tones: FxHashSet<u64> = FxHashSet::default();
        previous_tones.insert(1); // tone.id in previous
        let altered: FxHashMap<u8, Accidental> = FxHashMap::default();
        assert!(!is_accidental_needed(
            &tone,
            &vec![tone.clone()],
            &previous_tones,
            &altered,
            &key_sig
        ));
    }

    #[test]
    fn altered_pitch_with_different_accidental_needs_accidental() {
        use rustc_hash::{FxHashMap, FxHashSet};
        let tone = make_test_tone(1, 60, Accidental::Natural); // C natural
        let key_sig = KeySignature::default();
        let previous_tones: FxHashSet<u64> = FxHashSet::default();
        let mut altered: FxHashMap<u8, Accidental> = FxHashMap::default();
        // C was previously altered to Sharp
        altered.insert(tone.pitch.base(), Accidental::Sharp);
        assert!(is_accidental_needed(
            &tone,
            &vec![tone.clone()],
            &previous_tones,
            &altered,
            &key_sig
        ));
    }

    #[test]
    fn sharp_in_g_major_key_needs_no_accidental() {
        use rustc_hash::{FxHashMap, FxHashSet};
        // G major has F# (pitch int 66, accidental Sharp)
        let key_sig = KeySignature::new(0, 0, KeySignatureMode::Major, 1);
        let tone = make_test_tone(1, 66, Accidental::Sharp); // F#
        let previous_tones: FxHashSet<u64> = FxHashSet::default();
        let altered: FxHashMap<u8, Accidental> = FxHashMap::default();
        assert!(!is_accidental_needed(
            &tone,
            &vec![tone.clone()],
            &previous_tones,
            &altered,
            &key_sig
        ));
    }

    // --- get_tones_needing_accidentals ---

    #[test]
    fn get_tones_needing_accidentals_sharp_in_c_major_needs_accidental() {
        use crate::instructions::parse::get_written_durations::Notation;
        use rustc_hash::{FxHashMap, FxHashSet};
        let tone = make_test_tone(1, 61, Accidental::Sharp); // C#
        let mut entry = Notation::new(0, 32);
        entry.tones.push(tone.clone());
        let key_sig = KeySignature::default();
        let mut previous: FxHashSet<u64> = FxHashSet::default();
        let mut altered: FxHashMap<u8, Accidental> = FxHashMap::default();
        let result = get_tones_needing_accidentals(&entry, &mut previous, &mut altered, &key_sig);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 1);
    }

    #[test]
    fn get_tones_needing_accidentals_natural_in_c_major_not_needed() {
        use crate::instructions::parse::get_written_durations::Notation;
        use rustc_hash::{FxHashMap, FxHashSet};
        let tone = make_test_tone(1, 60, Accidental::Natural); // C natural
        let mut entry = Notation::new(0, 32);
        entry.tones.push(tone.clone());
        let key_sig = KeySignature::default();
        let mut previous: FxHashSet<u64> = FxHashSet::default();
        let mut altered: FxHashMap<u8, Accidental> = FxHashMap::default();
        let result = get_tones_needing_accidentals(&entry, &mut previous, &mut altered, &key_sig);
        assert!(result.is_empty());
    }

    // --- fits_in_slot ---

    fn make_empty_shunts() -> super::super::get_shunts::Shunts {
        super::super::get_shunts::Shunts {
            by_id: rustc_hash::FxHashMap::default(),
            by_offset: rustc_hash::FxHashMap::default(),
        }
    }

    #[test]
    fn fits_in_slot_returns_true_when_no_conflicts() {
        let slots: Vec<AccidentalNotation> = vec![];
        let mut tone_offsets: ToneVerticalOffsets = rustc_hash::FxHashMap::default();
        tone_offsets.insert(1, 0);
        let shunts = make_empty_shunts();
        // slot 1, offset 0: no existing entries, no shunts → should fit
        assert!(fits_in_slot(0, 1, 0, &slots, &tone_offsets, &shunts));
    }

    #[test]
    fn fits_in_slot_returns_false_when_same_slot_close_offset() {
        // Two accidentals in slot 1, offset 0 and offset 2 → diff=2, less than 6
        let slots: Vec<AccidentalNotation> = vec![AccidentalNotation {
            tone_id: 99,
            slot: 1,
        }];
        let mut tone_offsets: ToneVerticalOffsets = rustc_hash::FxHashMap::default();
        tone_offsets.insert(99, 0); // existing accidental at offset 0
        let shunts = make_empty_shunts();
        // new accidental at offset 2 → diff=2 < 6 → doesn't fit
        assert!(!fits_in_slot(0, 1, 2, &slots, &tone_offsets, &shunts));
    }

    #[test]
    fn fits_in_slot_returns_true_when_different_slot() {
        // Existing entry is in slot 2, new is in slot 1 → no conflict
        let slots: Vec<AccidentalNotation> = vec![AccidentalNotation {
            tone_id: 99,
            slot: 2,
        }];
        let mut tone_offsets: ToneVerticalOffsets = rustc_hash::FxHashMap::default();
        tone_offsets.insert(99, 0);
        let shunts = make_empty_shunts();
        assert!(fits_in_slot(0, 1, 0, &slots, &tone_offsets, &shunts));
    }

    #[test]
    fn fits_in_slot_returns_true_when_far_apart_same_slot() {
        // Two accidentals in slot 1, offsets 0 and 10 → diff=10 >= 6 → fits
        let slots: Vec<AccidentalNotation> = vec![AccidentalNotation {
            tone_id: 99,
            slot: 1,
        }];
        let mut tone_offsets: ToneVerticalOffsets = rustc_hash::FxHashMap::default();
        tone_offsets.insert(99, 0);
        let shunts = make_empty_shunts();
        assert!(fits_in_slot(0, 1, 10, &slots, &tone_offsets, &shunts));
    }

    // --- find_slot ---

    #[test]
    fn find_slot_returns_slot_1_when_no_conflicts() {
        let slots: Vec<AccidentalNotation> = vec![];
        let mut tone_offsets: ToneVerticalOffsets = rustc_hash::FxHashMap::default();
        tone_offsets.insert(1, 0);
        let shunts = make_empty_shunts();
        let slot = find_slot(0, 1, &slots, &tone_offsets, &shunts);
        assert_eq!(slot, 1);
    }

    #[test]
    fn find_slot_advances_to_slot_2_when_slot1_has_conflict() {
        // Existing entry at slot 1 and offset 0; new tone wants offset 2 → slot 1 fails, slot 2 works
        let slots: Vec<AccidentalNotation> = vec![AccidentalNotation {
            tone_id: 99,
            slot: 1,
        }];
        let mut tone_offsets: ToneVerticalOffsets = rustc_hash::FxHashMap::default();
        tone_offsets.insert(99, 0);
        tone_offsets.insert(1, 2); // close to existing
        let shunts = make_empty_shunts();
        let slot = find_slot(0, 1, &slots, &tone_offsets, &shunts);
        assert!(slot >= 2);
    }
}
