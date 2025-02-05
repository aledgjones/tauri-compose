use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::entries::clef::{Clef, ClefDrawType};
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;

pub type ToneVerticalOffsets = FxHashMap<UniqueId, i8>;

pub fn get_tone_offsets(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
) -> ToneVerticalOffsets {
    let mut output: ToneVerticalOffsets = FxHashMap::default();

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let master = &tracks.by_id[&stave.master_id];

            for track_id in &stave.track_ids {
                let track = &tracks.by_id[track_id];
                let mut clef = &Clef::new(
                    0,
                    0,
                    Pitch::new(60, Accidental::Natural),
                    0,
                    ClefDrawType::C,
                );

                for tick in 0..flow.length {
                    if let Some(found) = master.get_clef_at_tick(&tick) {
                        clef = found;
                    };

                    for tone in track.get_tones_at_tick(&tick) {
                        output.insert(
                            tone.id,
                            Pitch::steps_between(&tone.pitch, &clef.pitch) + clef.offset,
                        );
                    }
                }
            }
        }
    }

    output
}
