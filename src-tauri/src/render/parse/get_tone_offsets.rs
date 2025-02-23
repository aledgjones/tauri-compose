use std::cmp::max_by_key;

use crate::score::components::duration::Tick;
use crate::score::components::pitch::Pitch;
use crate::score::entries::clef::Clef;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;

pub struct ToneOffsetInfo {
    highest: i8,
    lowest: i8,
    furthest: i8,
}

impl ToneOffsetInfo {
    pub fn new(highest: i8, lowest: i8, furthest: i8) -> Self {
        Self {
            highest,
            lowest,
            furthest,
        }
    }
}

pub struct ToneVerticalOffsets {
    pub by_tone_id: FxHashMap<UniqueId, i8>,
    pub info_by_tick: FxHashMap<Tick, ToneOffsetInfo>,
}

impl ToneVerticalOffsets {
    pub fn new() -> Self {
        Self {
            by_tone_id: FxHashMap::default(),
            info_by_tick: FxHashMap::default(),
        }
    }
}

pub fn get_tone_offsets(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
) -> ToneVerticalOffsets {
    let mut output: ToneVerticalOffsets = ToneVerticalOffsets::new();

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let master = &tracks.by_id[&stave.master_id];

            for track_id in &stave.track_ids {
                let track = &tracks.by_id[track_id];
                let mut clef = &Clef::default();

                for tick in 0..flow.length {
                    if let Some(found) = master.get_clef_at_tick(&tick) {
                        clef = found;
                    };

                    let mut highest: i8 = 0;
                    let mut lowest: i8 = 0;

                    for tone in track.get_tones_at_tick(&tick) {
                        let offset = Pitch::steps_between(&tone.pitch, &clef.pitch) + clef.offset;

                        if offset < highest {
                            highest = offset;
                        }

                        if offset > lowest {
                            lowest = offset;
                        }

                        output.by_tone_id.insert(tone.id, offset);
                    }

                    let furthest = max_by_key(lowest, highest, |value: &i8| i8::abs(*value));

                    output
                        .info_by_tick
                        .insert(tick, ToneOffsetInfo::new(highest, lowest, furthest));
                }
            }
        }
    }

    output
}
