use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::NotationTrack;
use crate::score::entries::tone::Tone;

#[derive(Clone, Debug)]
pub struct ToneOffsetInfo {
    highest: i8,
    lowest: i8,
    furthest: i8,
}

impl ToneOffsetInfo {
    pub fn new(tones: &[Tone], tone_offsets: &ToneVerticalOffsets) -> Option<ToneOffsetInfo> {
        if tones.is_empty() {
            return None;
        }

        let highest = tone_offsets[&tones.last().unwrap().id];
        let lowest = tone_offsets[&tones.first().unwrap().id];

        let furthest = if lowest.abs() > highest.abs() {
            lowest
        } else {
            highest
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
