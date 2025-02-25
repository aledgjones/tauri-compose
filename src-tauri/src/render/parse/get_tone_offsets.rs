use crate::score::components::pitch::Pitch;
use crate::score::entries::clef::Clef;
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
    let mut output = ToneVerticalOffsets::default();

    for instrument in instruments {
        for &stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[&stave_id];
            let master = &tracks.by_id[&stave.master_id];

            for &track_id in &stave.track_ids {
                let track = &tracks.by_id[&track_id];
                let mut clef = &Clef::default();

                for tick in 0..flow.length {
                    if let Some(found) = master.get_clef_at_tick(&tick) {
                        clef = found;
                    };

                    for tone in track.get_tones_at_tick(&tick) {
                        let offset = Pitch::steps_between(&tone.pitch, &clef.pitch) + clef.offset;
                        output.insert(tone.id, offset);
                    }
                }
            }
        }
    }

    output
}
