use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::NotationTrack;
use crate::score::entries::tone::Tone;

#[derive(Clone)]
pub struct ToneOffsetInfo {
    pub highest: i8,
    pub lowest: i8,
    pub furthest: i8,
}

impl ToneOffsetInfo {
    pub fn new(tones: &[Tone], tone_offsets: &ToneVerticalOffsets) -> Option<ToneOffsetInfo> {
        if tones.is_empty() {
            return None;
        }

        let highest = tone_offsets[&tones.last().unwrap().id];
        let lowest = tone_offsets[&tones.first().unwrap().id];

        // if lowest explicitly less than highest, use highest -- falback to lowest when equal or explixitly greater.
        let furthest = if lowest.abs() < highest.abs() {
            highest
        } else {
            lowest
        };

        Some(ToneOffsetInfo {
            highest,
            lowest,
            furthest,
        })
    }
}

pub fn get_tone_offset_info(notation: &mut NotationTrack, tone_offsets: &ToneVerticalOffsets) {
    for entry in notation.track.values_mut() {
        entry.sort_tones_by_offset(tone_offsets);
        entry.info = ToneOffsetInfo::new(&entry.tones, tone_offsets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::{
        articulation::Articulations,
        pitch::{Accidental, Pitch},
        velocity::Velocity,
    };
    use crate::score::entries::tone::Tone;
    use rustc_hash::FxHashMap;

    fn make_tone(id: u64, offset: i8) -> (Tone, (u64, i8)) {
        let tone = Tone::new(
            id,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        (tone, (id, offset))
    }

    #[test]
    fn tone_offset_info_new_returns_none_on_empty_tones() {
        let offsets: ToneVerticalOffsets = FxHashMap::default();
        assert!(ToneOffsetInfo::new(&[], &offsets).is_none());
    }

    #[test]
    fn tone_offset_info_single_tone_highest_equals_lowest() {
        let (tone, (id, offset)) = make_tone(1, 3);
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(id, offset);
        let info = ToneOffsetInfo::new(&[tone], &offsets).unwrap();
        assert_eq!(info.highest, 3);
        assert_eq!(info.lowest, 3);
    }

    #[test]
    fn tone_offset_info_furthest_is_abs_highest_when_abs_higher_is_larger() {
        // highest=5 (abs 5), lowest=2 (abs 2) → furthest = highest
        let (tone1, (id1, _)) = make_tone(1, 2);
        let (tone2, (id2, _)) = make_tone(2, 5);
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(id1, 2);
        offsets.insert(id2, 5);
        // tones[first] = lowest, tones[last] = highest
        let info = ToneOffsetInfo::new(&[tone1, tone2], &offsets).unwrap();
        // furthest = highest(5) because abs(2) < abs(5)
        assert_eq!(info.furthest, 5);
    }

    #[test]
    fn tone_offset_info_furthest_is_lowest_when_abs_lower_is_larger() {
        // lowest=-6 (abs 6), highest=2 (abs 2) → furthest = lowest
        let (tone1, (id1, _)) = make_tone(1, -6);
        let (tone2, (id2, _)) = make_tone(2, 2);
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(id1, -6);
        offsets.insert(id2, 2);
        let info = ToneOffsetInfo::new(&[tone1, tone2], &offsets).unwrap();
        // furthest = lowest(-6) because abs(-6) >= abs(2)
        assert_eq!(info.furthest, -6);
    }

    #[test]
    fn tone_offset_info_furthest_equals_lowest_when_equal_abs() {
        // lowest=-3 (abs 3), highest=3 (abs 3) → tie → furthest = lowest
        let (tone1, (id1, _)) = make_tone(1, -3);
        let (tone2, (id2, _)) = make_tone(2, 3);
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(id1, -3);
        offsets.insert(id2, 3);
        let info = ToneOffsetInfo::new(&[tone1, tone2], &offsets).unwrap();
        // abs(-3) >= abs(3) → furthest = lowest = -3
        assert_eq!(info.furthest, -3);
    }
}
