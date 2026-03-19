use super::get_barlines::Barlines;
use super::get_beams::get_beams_in_track;
use super::get_measures::measures_from_single_time_signature;
use super::get_shunts::{get_note_shunts_in_track, Shunt, Shunts, ShuntsByTrack};
use super::get_stem_directions::get_tuplet_stem_directions_in_track;
use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::{
    AnchoredNotationTrack, Notation, NotationByTrack, NotationTrack,
};
use crate::score::components::duration::Tick;
use crate::score::components::pitch::Accidental;
use crate::score::entries::Entry;
use crate::score::entries::{key_signature::KeySignature, tone::Tone};
use crate::score::flows::Flow;
use crate::score::tracks::{Track, Tracks};
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

type SlotsByTick = FxHashMap<Tick, u8>;

#[derive(Clone)]
pub struct AccidentalNotation {
    pub tone_id: UniqueId,
    pub slot: u8,
}

#[derive(Clone)]
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
pub type AnchoredAccidentals = FxHashMap<UniqueId, Accidentals>;
pub type AnchoredAccidentalsByTrack = FxHashMap<UniqueId, AnchoredAccidentals>;

#[allow(clippy::too_many_arguments)]
fn apply_entry_accidentals(
    tick: Tick,
    entry: &Notation,
    output: &mut Accidentals,
    previous_tones: &mut FxHashSet<UniqueId>,
    altered_base_pitches: &mut FxHashMap<u8, Accidental>,
    key_signature: &KeySignature,
    tone_offsets: &ToneVerticalOffsets,
    shunts: &Shunts,
) {
    let accidentals =
        get_tones_needing_accidentals(entry, previous_tones, altered_base_pitches, key_signature);

    if accidentals.is_empty() {
        return;
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

fn apply_track_accidentals(
    notation: &NotationTrack,
    output: &mut Accidentals,
    previous_tones: &mut FxHashSet<UniqueId>,
    altered_base_pitches: &mut FxHashMap<u8, Accidental>,
    key_signature: &KeySignature,
    tone_offsets: &ToneVerticalOffsets,
    shunts: &Shunts,
) {
    for tick in 0..notation.length {
        let Some(entry) = notation.track.get(&tick) else {
            continue;
        };

        apply_entry_accidentals(
            tick,
            entry,
            output,
            previous_tones,
            altered_base_pitches,
            key_signature,
            tone_offsets,
            shunts,
        );
    }
}

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

fn get_accidentals_in_track_with_anchored_tracks(
    notation: &NotationTrack,
    flow_master: &Track,
    track_master: &Track,
    barlines: &Barlines,
    tone_offsets: &ToneVerticalOffsets,
    shunts: &Shunts,
) -> (Accidentals, AnchoredAccidentals) {
    let mut output = Accidentals::new();
    let mut anchored_output = AnchoredAccidentals::default();

    let mut key_signature = &KeySignature::default();
    let mut altered_base_pitches: FxHashMap<u8, Accidental> = FxHashMap::default();
    let mut previous_tones: FxHashSet<UniqueId> = FxHashSet::default();

    let mut anchored_tracks_by_start: FxHashMap<Tick, Vec<&AnchoredNotationTrack>> =
        FxHashMap::default();
    let mut anchored_shunts_by_owner: FxHashMap<UniqueId, Shunts> = FxHashMap::default();

    for anchored_track in &notation.anchored_tracks {
        let Some(Entry::Tuplet(tuplet)) = track_master.entries.by_id.get(&anchored_track.owner_id)
        else {
            continue;
        };

        let measures = measures_from_single_time_signature(tuplet.local_time_signature());
        let beams = get_beams_in_track(&anchored_track.notation, &measures);
        let stem_directions = get_tuplet_stem_directions_in_track(&anchored_track.notation, &beams);
        let shunts =
            get_note_shunts_in_track(&anchored_track.notation, tone_offsets, &stem_directions);

        anchored_tracks_by_start
            .entry(anchored_track.host_span.start_tick)
            .or_default()
            .push(anchored_track);
        anchored_shunts_by_owner.insert(anchored_track.owner_id, shunts);
    }

    for tick in 0..notation.length {
        let found = match flow_master.get_key_signature_at_tick(&tick) {
            Some(entry) => {
                key_signature = entry;
                true
            }
            None => false,
        };

        if found || barlines.contains_key(&tick) {
            altered_base_pitches.clear();
        }

        if let Some(entry) = notation.track.get(&tick) {
            apply_entry_accidentals(
                tick,
                entry,
                &mut output,
                &mut previous_tones,
                &mut altered_base_pitches,
                key_signature,
                tone_offsets,
                shunts,
            );
        }

        if let Some(anchored_tracks) = anchored_tracks_by_start.get(&tick) {
            for anchored_track in anchored_tracks {
                let Some(anchored_shunts) = anchored_shunts_by_owner.get(&anchored_track.owner_id)
                else {
                    continue;
                };

                let anchored_accidentals = anchored_output
                    .entry(anchored_track.owner_id)
                    .or_insert_with(Accidentals::new);
                apply_track_accidentals(
                    &anchored_track.notation,
                    anchored_accidentals,
                    &mut previous_tones,
                    &mut altered_base_pitches,
                    key_signature,
                    tone_offsets,
                    anchored_shunts,
                );
            }
        }
    }

    (output, anchored_output)
}

pub fn get_accidentals_with_anchored_tracks(
    flow: &Flow,
    tracks: &Tracks,
    notation_by_track: &NotationByTrack,
    bars: &Barlines,
    tone_offsets: &ToneVerticalOffsets,
    shunts_by_track: &ShuntsByTrack,
) -> (AccidentalsByTrack, AnchoredAccidentalsByTrack) {
    let flow_master = &tracks.by_id[&flow.master_id];

    let results: Vec<(UniqueId, Accidentals, AnchoredAccidentals)> = notation_by_track
        .par_iter()
        .map(|(track_id, notation)| {
            let track_master = &tracks.by_id[track_id];
            let shunts = &shunts_by_track[track_id];
            let (accidentals, anchored_accidentals) = get_accidentals_in_track_with_anchored_tracks(
                notation,
                flow_master,
                track_master,
                bars,
                tone_offsets,
                shunts,
            );
            (*track_id, accidentals, anchored_accidentals)
        })
        .collect();

    let mut output: AccidentalsByTrack = FxHashMap::default();
    let mut anchored_output: AnchoredAccidentalsByTrack = FxHashMap::default();
    for (track_id, accidentals, anchored_accidentals) in results {
        output.insert(track_id, accidentals);
        if !anchored_accidentals.is_empty() {
            anchored_output.insert(track_id, anchored_accidentals);
        }
    }

    (output, anchored_output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_tone_offset_info::get_tone_offset_info;
    use crate::instructions::parse::get_written_durations::HostSpan;
    use crate::score::components::duration::NoteDuration;
    use crate::score::components::pitch::Pitch;
    use crate::score::entries::key_signature::KeySignatureMode;
    use crate::score::entries::tuplet::Tuplet;
    use crate::score::entries::Entry;
    use crate::score::flows::Flow;
    use crate::score::tracks::Tracks;

    fn get_accidentals_in_track_with_seeded_key_signature(
        notation: &NotationTrack,
        initial_key_signature: &KeySignature,
        barlines: &Barlines,
        tone_offsets: &ToneVerticalOffsets,
        shunts: &Shunts,
    ) -> Accidentals {
        let mut output = Accidentals::new();
        let mut altered_base_pitches: FxHashMap<u8, Accidental> = FxHashMap::default();
        let mut previous_tones: FxHashSet<UniqueId> = FxHashSet::default();

        for tick in 0..notation.length {
            if barlines.contains_key(&tick) {
                altered_base_pitches.clear();
            }

            let Some(entry) = notation.track.get(&tick) else {
                continue;
            };

            apply_entry_accidentals(
                tick,
                entry,
                &mut output,
                &mut previous_tones,
                &mut altered_base_pitches,
                initial_key_signature,
                tone_offsets,
                shunts,
            );
        }

        output
    }

    fn get_accidentals_in_track(
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
            let found = match master.get_key_signature_at_tick(&tick) {
                Some(entry) => {
                    key_signature = entry;
                    true
                }
                None => false,
            };

            if found || barlines.contains_key(&tick) {
                altered_base_pitches.clear();
            }

            let Some(entry) = notation.track.get(&tick) else {
                continue;
            };

            apply_entry_accidentals(
                tick,
                entry,
                &mut output,
                &mut previous_tones,
                &mut altered_base_pitches,
                key_signature,
                tone_offsets,
                shunts,
            );
        }

        output
    }

    fn get_accidentals_in_track_with_initial_key_signature(
        notation: &NotationTrack,
        initial_key_signature: &KeySignature,
        barlines: &Barlines,
        tone_offsets: &ToneVerticalOffsets,
        shunts: &Shunts,
    ) -> Accidentals {
        get_accidentals_in_track_with_seeded_key_signature(
            notation,
            initial_key_signature,
            barlines,
            tone_offsets,
            shunts,
        )
    }

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

    fn make_sharp_tone(id: u64, tick: Tick) -> Tone {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::velocity::Velocity;

        Tone::new(
            id,
            tick,
            32,
            Pitch::new(61, Accidental::Sharp),
            Velocity::new(100),
            Articulations::new(),
        )
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
    fn get_accidentals_in_track_does_not_repeat_accidental_on_split_tied_fragment() {
        let mut notation = NotationTrack::new(64);
        let mut first = Notation::new(0, 32);
        first.tones.push(make_sharp_tone(1, 0));
        first.ties.insert(1);

        let mut second = Notation::new(32, 16);
        second.tones.push(make_sharp_tone(1, 32));

        notation.track.insert(0, first);
        notation.track.insert(32, second);

        let master = Track::new(99);
        let barlines = Barlines::default();
        let mut tone_offsets = ToneVerticalOffsets::default();
        tone_offsets.insert(1, 0);
        let shunts = Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        };

        let result =
            get_accidentals_in_track(&notation, &master, &barlines, &tone_offsets, &shunts);

        assert!(result.by_id.contains_key(&(0, 1)));
        assert!(!result.by_id.contains_key(&(32, 1)));
    }

    #[test]
    fn seeded_key_signature_requires_cancellation_inside_local_track() {
        let mut notation = NotationTrack::new(32);
        let mut entry = Notation::new(0, 32);
        entry.tones.push(make_test_tone(1, 65, Accidental::Natural));
        notation.track.insert(0, entry);

        let key_sig = KeySignature::new(10, 0, KeySignatureMode::Major, 1);
        let mut tone_offsets = ToneVerticalOffsets::default();
        tone_offsets.insert(1, 0);
        let shunts = Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        };

        let result = get_accidentals_in_track_with_initial_key_signature(
            &notation,
            &key_sig,
            &Barlines::default(),
            &tone_offsets,
            &shunts,
        );

        assert!(result.by_id.contains_key(&(0, 1)));
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

    fn make_test_notation_entry(
        tone_id: u64,
        tick: Tick,
        duration: Tick,
        pitch_int: u8,
        accidental: Accidental,
    ) -> Notation {
        let mut entry = Notation::new(tick, duration);
        entry
            .tones
            .push(make_test_tone(tone_id, pitch_int, accidental));
        entry
    }

    fn make_empty_shunts_for_track(track_id: UniqueId) -> ShuntsByTrack {
        let mut shunts_by_track = ShuntsByTrack::default();
        shunts_by_track.insert(track_id, make_empty_shunts());
        shunts_by_track
    }

    fn build_tuplet_accidental_test_context(
        anchored_entry: Notation,
        host_entry: Option<Notation>,
    ) -> (
        Flow,
        Tracks,
        NotationByTrack,
        ToneVerticalOffsets,
        ShuntsByTrack,
    ) {
        let flow_id = 10;
        let flow_master_id = 11;
        let track_id = 12;
        let tuplet_id = 13;

        let mut flow = Flow::new(flow_id, flow_master_id);
        flow.length = 64;

        let mut flow_master = Track::new(flow_master_id);
        flow_master.insert_entry(Entry::KeySignature(KeySignature::new(
            20,
            0,
            KeySignatureMode::Major,
            1,
        )));

        let mut track_master = Track::new(track_id);
        track_master.insert_entry(Entry::Tuplet(Box::new(Tuplet::new(
            tuplet_id,
            0,
            3,
            2,
            NoteDuration::Eighth,
            vec![3],
        ))));

        let mut tone_offsets = ToneVerticalOffsets::default();
        tone_offsets.insert(anchored_entry.tones[0].id, 0);
        if let Some(entry) = host_entry.as_ref() {
            tone_offsets.insert(entry.tones[0].id, 0);
        }

        let mut local_notation = NotationTrack::new(48);
        local_notation
            .track
            .insert(anchored_entry.tick, anchored_entry.clone());
        get_tone_offset_info(&mut local_notation, &tone_offsets);

        let mut notation = NotationTrack::new(64);
        notation.anchored_tracks.push(AnchoredNotationTrack {
            owner_id: tuplet_id,
            host_span: HostSpan {
                start_tick: 0,
                end_tick: 32,
            },
            notation: local_notation,
        });
        if let Some(entry) = host_entry {
            notation.track.insert(entry.tick, entry);
        }
        get_tone_offset_info(&mut notation, &tone_offsets);

        let mut notation_by_track = NotationByTrack::default();
        notation_by_track.insert(track_id, notation);

        let mut tracks = Tracks::new();
        tracks.by_id.insert(flow_master_id, flow_master);
        tracks.by_id.insert(track_id, track_master);

        (
            flow,
            tracks,
            notation_by_track,
            tone_offsets,
            make_empty_shunts_for_track(track_id),
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

    #[test]
    fn anchored_track_uses_flow_key_signature_when_accidental_is_in_key() {
        let anchored_entry = make_test_notation_entry(1, 0, 16, 66, Accidental::Sharp);
        let (flow, tracks, notation_by_track, tone_offsets, shunts_by_track) =
            build_tuplet_accidental_test_context(anchored_entry, None);

        let (_host, anchored) = get_accidentals_with_anchored_tracks(
            &flow,
            &tracks,
            &notation_by_track,
            &Barlines::default(),
            &tone_offsets,
            &shunts_by_track,
        );

        let anchored_track = &anchored[&12][&13];
        assert!(anchored_track.by_id.is_empty());
    }

    #[test]
    fn anchored_track_accidental_updates_later_host_accidentals_in_same_bar() {
        let anchored_entry = make_test_notation_entry(1, 0, 16, 65, Accidental::Natural);
        let host_entry = make_test_notation_entry(2, 32, 16, 66, Accidental::Sharp);
        let (flow, tracks, notation_by_track, tone_offsets, shunts_by_track) =
            build_tuplet_accidental_test_context(anchored_entry, Some(host_entry));

        let (host, anchored) = get_accidentals_with_anchored_tracks(
            &flow,
            &tracks,
            &notation_by_track,
            &Barlines::default(),
            &tone_offsets,
            &shunts_by_track,
        );

        assert!(anchored[&12][&13].by_id.contains_key(&(0, 1)));
        assert!(host[&12].by_id.contains_key(&(32, 2)));
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
