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
