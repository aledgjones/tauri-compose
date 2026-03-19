use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::{Notation, NotationByTrack, NotationTrack};
use crate::score::components::duration::Tick;
use crate::score::flows::Flow;
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

pub type DotOffsets = FxHashSet<i8>;
pub type Dots = FxHashMap<Tick, DotOffsets>;
pub type DotsByTrack = FxHashMap<UniqueId, Dots>;

/// If there are not enough gaps to put dots, we draw as many dots as possible
fn best_effort_fallback(entry: &Notation, tone_offsets: &ToneVerticalOffsets) -> DotOffsets {
    let mut output: DotOffsets = FxHashSet::default();

    // put a dot at each space with a tone
    for tone in &entry.tones {
        let offset = tone_offsets[&tone.id];
        // is space
        if offset % 2 != 0 {
            output.insert(offset);
        }
    }

    for tone in &entry.tones {
        let offset = tone_offsets[&tone.id];
        // is space
        if offset % 2 == 0 {
            match output.get(&(offset - 1)) {
                Some(_) => {
                    output.insert(offset + 1);
                }
                None => {
                    output.insert(offset - 1);
                }
            }
        }
    }

    output
}

fn is_close_enough(slot: i8, entry: &Notation, tone_offsets: &ToneVerticalOffsets) -> bool {
    for tone in &entry.tones {
        let offset = tone_offsets[&tone.id];
        if slot >= offset - 2 && slot <= offset + 2 {
            return true;
        }
    }

    false
}

/// try and write all dots that are needed, we fallback to the best effort apporach
/// if the dots get too far away from the note heads
fn get_dots_at_tick(entry: &Notation, tone_offsets: &ToneVerticalOffsets) -> DotOffsets {
    let mut output: DotOffsets = FxHashSet::default();

    // put a dot at each space with a tone
    for tone in &entry.tones {
        let offset = tone_offsets[&tone.id];
        // is space
        if offset % 2 != 0 {
            output.insert(offset);
        }
    }

    for tone in &entry.tones {
        let offset = tone_offsets[&tone.id];
        // is line
        if offset % 2 == 0 {
            let mut n = 0;
            let mut direction = -1;
            let mut slot = offset + (1 + n * 2) * direction;

            // find the nearest available slot
            while output.contains(&slot) {
                if direction == 1 {
                    n += 1;
                }
                direction *= -1;
                slot = offset + (1 + n * 2) * direction;
            }

            // check if the slot is close enough
            if is_close_enough(slot, entry, tone_offsets) {
                output.insert(slot);
            } else {
                // if you can't fit the dot close enough to the end
                // of the cluster try moving it to the other end
                direction *= -1;
                // find last space we can fit dot in
                loop {
                    slot += 2 * direction;
                    if !output.contains(&slot) {
                        break;
                    }
                }
                if is_close_enough(slot, entry, tone_offsets) {
                    output.insert(slot);
                } else {
                    return best_effort_fallback(entry, tone_offsets);
                }
            }
        }
    }

    output
}

pub(crate) fn get_dots_in_track(
    notation: &NotationTrack,
    tone_offsets: &ToneVerticalOffsets,
) -> Dots {
    let mut output: Dots = FxHashMap::default();

    for (tick, entry) in &notation.track {
        if !entry.is_rest() && entry.is_dotted() {
            let dots: DotOffsets = get_dots_at_tick(entry, tone_offsets);
            output.insert(*tick, dots);
        }
    }

    output
}

pub fn get_dots(
    _flow: &Flow,
    notation_by_track: &NotationByTrack,
    tone_offsets: &ToneVerticalOffsets,
) -> DotsByTrack {
    let results: Vec<(UniqueId, _)> = notation_by_track
        .par_iter()
        .map(|(track_id, notation)| {
            let dots: Dots = get_dots_in_track(notation, tone_offsets);
            (*track_id, dots)
        })
        .collect();

    let mut output: DotsByTrack = FxHashMap::default();
    for (track_id, dots) in results {
        output.insert(track_id, dots);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_tone_offsets::ToneVerticalOffsets;
    use crate::instructions::parse::get_written_durations::{Notation, NotationTrack};
    use crate::score::components::articulation::Articulations;
    use crate::score::components::duration::NoteDuration;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::entries::tone::Tone;
    use rustc_hash::FxHashMap;

    fn make_tone(id: u64, offset_in_map: i8, offsets: &mut ToneVerticalOffsets) -> Tone {
        use crate::score::components::velocity::Velocity;
        offsets.insert(id, offset_in_map);
        Tone::new(
            id,
            0,
            NoteDuration::Quarter.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        )
    }

    fn make_dotted_notation(
        tick: u64,
        duration: u64,
        tones_and_offsets: Vec<(u64, i8)>,
        offsets: &mut ToneVerticalOffsets,
    ) -> Notation {
        let mut n = Notation::new(tick, duration);
        for (id, offset) in tones_and_offsets {
            n.tones.push(make_tone(id, offset, offsets));
        }
        n
    }

    #[test]
    fn get_dots_in_track_empty_track_returns_empty() {
        let track = NotationTrack::new(512);
        let offsets: ToneVerticalOffsets = FxHashMap::default();
        let result = get_dots_in_track(&track, &offsets);
        assert!(result.is_empty());
    }

    #[test]
    fn get_dots_in_track_rest_entry_not_included() {
        let mut track = NotationTrack::new(512);
        // Rest = no tones; a rest that's dotted still is_rest()
        let quarter = NoteDuration::Quarter.to_ticks();
        let dotted = quarter + quarter / 2; // dotted quarter duration
        let entry = Notation::new(0, dotted); // no tones → is_rest() is true
        track.track.insert(0, entry);
        let offsets: ToneVerticalOffsets = FxHashMap::default();
        let result = get_dots_in_track(&track, &offsets);
        assert!(result.is_empty());
    }

    #[test]
    fn get_dots_at_tick_tone_on_space_gets_dot_at_same_offset() {
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        // offset 1 = odd = space
        let entry = make_dotted_notation(
            0,
            NoteDuration::Quarter.to_ticks() + NoteDuration::Eighth.to_ticks(), // dotted quarter
            vec![(1, 1)],
            &mut offsets,
        );
        let dots = get_dots_at_tick(&entry, &offsets);
        assert!(dots.contains(&1));
    }

    #[test]
    fn get_dots_at_tick_tone_on_line_gets_dot_nearby() {
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        // offset 0 = even = line
        let entry = make_dotted_notation(
            0,
            NoteDuration::Quarter.to_ticks() + NoteDuration::Eighth.to_ticks(),
            vec![(1, 0)],
            &mut offsets,
        );
        let dots = get_dots_at_tick(&entry, &offsets);
        // dot should be placed at an odd offset near 0 (either -1 or 1)
        assert_eq!(dots.len(), 1);
        let &slot = dots.iter().next().unwrap();
        assert!(slot % 2 != 0); // must be a space
    }

    #[test]
    fn best_effort_fallback_places_dots_at_space_offsets_for_spaces() {
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        // two tones at spaces
        let entry = make_dotted_notation(0, 192, vec![(1, 1), (2, 3)], &mut offsets);
        let dots = best_effort_fallback(&entry, &offsets);
        assert!(dots.contains(&1));
        assert!(dots.contains(&3));
    }

    #[test]
    fn is_close_enough_returns_true_when_slot_within_range() {
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        let entry = make_dotted_notation(0, 192, vec![(1, 4)], &mut offsets);
        // slot 3 is within range [4-2, 4+2] = [2, 6]
        assert!(is_close_enough(3, &entry, &offsets));
    }

    #[test]
    fn is_close_enough_returns_false_when_slot_far_away() {
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        let entry = make_dotted_notation(0, 192, vec![(1, 4)], &mut offsets);
        // slot 20 is far from 4
        assert!(!is_close_enough(20, &entry, &offsets));
    }

    #[test]
    fn get_dots_in_track_dotted_note_produces_dot() {
        let mut track = NotationTrack::new(512);
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        let quarter = NoteDuration::Quarter.to_ticks();
        let dotted_quarter = quarter + quarter / 2; // 3 * 16 = 48 ticks
                                                    // tone at space offset 1 (odd)
        let entry = make_dotted_notation(0, dotted_quarter, vec![(1, 1)], &mut offsets);
        track.track.insert(0, entry);
        let result = get_dots_in_track(&track, &offsets);
        // should have a dot at tick 0
        assert!(result.contains_key(&0));
        let dots = &result[&0];
        assert!(!dots.is_empty());
    }

    #[test]
    fn get_dots_in_track_non_dotted_note_produces_no_dot() {
        let mut track = NotationTrack::new(512);
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        let quarter = NoteDuration::Quarter.to_ticks();
        let entry = make_dotted_notation(0, quarter, vec![(1, 1)], &mut offsets);
        track.track.insert(0, entry);
        let result = get_dots_in_track(&track, &offsets);
        // non-dotted notes should not produce dot entries
        assert!(!result.contains_key(&0));
    }

    #[test]
    fn get_dots_at_tick_multiple_tones_on_spaces() {
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        let dq = NoteDuration::Quarter.to_ticks() + NoteDuration::Eighth.to_ticks();
        let entry = make_dotted_notation(0, dq, vec![(1, 1), (2, 5)], &mut offsets);
        let dots = get_dots_at_tick(&entry, &offsets);
        // both offsets are odd (spaces) → dots at 1 and 5
        assert!(dots.contains(&1));
        assert!(dots.contains(&5));
    }

    #[test]
    fn best_effort_fallback_handles_line_tones() {
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        // tone at offset 0 (line): best_effort treats it as if it's a space but at offset 0
        // actually best_effort just places dot at offset if odd, else tries even+1
        let entry = make_dotted_notation(0, 192, vec![(1, 0)], &mut offsets);
        let dots = best_effort_fallback(&entry, &offsets);
        // 0 is even, best effort places at 1 (offset + 1)
        assert!(!dots.is_empty());
    }

    #[test]
    fn get_dots_in_track_only_marks_the_dotted_fragment_of_split_three_four_note() {
        let quarter = NoteDuration::Quarter.to_ticks();
        let eighth = NoteDuration::Eighth.to_ticks();
        let dotted_quarter = quarter + eighth;
        let mut track = NotationTrack::new(quarter * 3);
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();

        let dotted = make_dotted_notation(0, dotted_quarter, vec![(1, 1)], &mut offsets);
        let mut tail = Notation::new(dotted_quarter, eighth);
        tail.tones.push(make_tone(1, 1, &mut offsets));
        let mut final_quarter = Notation::new(quarter * 2, quarter);
        final_quarter.tones.push(make_tone(1, 1, &mut offsets));

        track.track.insert(0, dotted);
        track.track.insert(dotted_quarter, tail);
        track.track.insert(quarter * 2, final_quarter);

        let result = get_dots_in_track(&track, &offsets);

        assert!(result.contains_key(&0));
        assert!(!result.contains_key(&dotted_quarter));
        assert!(!result.contains_key(&(quarter * 2)));
    }
}
