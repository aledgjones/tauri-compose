use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::{Notation, NotationByTrack, NotationTrack};
use crate::score::components::duration::{Tick, Ticks};
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

fn get_dots_in_track(
    notation: &NotationTrack,
    tone_offsets: &ToneVerticalOffsets,
    subdivisions: Ticks,
) -> Dots {
    let mut output: Dots = FxHashMap::default();

    for (tick, entry) in &notation.track {
        if !entry.is_rest() && entry.is_dotted(subdivisions) {
            let dots: DotOffsets = get_dots_at_tick(entry, tone_offsets);
            output.insert(*tick, dots);
        }
    }

    output
}

pub fn get_dots(
    flow: &Flow,
    notation_by_track: &NotationByTrack,
    tone_offsets: &ToneVerticalOffsets,
) -> DotsByTrack {
    let results: Vec<(UniqueId, _)> = notation_by_track.par_iter().map(|(track_id, notation)| {
        let dots: Dots = get_dots_in_track(notation, tone_offsets, flow.subdivisions);
        (*track_id, dots)
    }).collect();

    let mut output: DotsByTrack = FxHashMap::default();
    for (track_id, dots) in results {
        output.insert(track_id, dots);
    }

    output
}
