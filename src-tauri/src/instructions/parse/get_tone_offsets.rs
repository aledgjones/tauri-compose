use crate::score::components::pitch::Pitch;
use crate::score::entries::clef::Clef;
use crate::score::entries::tuplet::TupletEntry;
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub type ToneVerticalOffsets = FxHashMap<UniqueId, i8>;

pub fn get_tone_offsets(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
) -> ToneVerticalOffsets {
    let mut output = ToneVerticalOffsets::default();

    let results: Vec<(UniqueId, i8)> = instruments
        .par_iter()
        .flat_map(|instrument| {
            instrument
                .stave_ids
                .par_iter()
                .flat_map(|stave_id| {
                    let stave = &flow.staves.by_id[stave_id];
                    let master = &tracks.by_id[&stave.master_id];
                    stave
                        .track_ids
                        .par_iter()
                        .flat_map(|track_id| {
                            let track = &tracks.by_id[track_id];
                            let mut clef = &Clef::default();
                            let mut offsets = Vec::new();

                            for tick in 0..flow.length {
                                if let Some(found) = master.get_clef_at_tick(&tick) {
                                    clef = found;
                                };

                                for tone in track.get_tones_at_tick(&tick) {
                                    let offset = Pitch::steps_between(&tone.pitch, &clef.pitch)
                                        + clef.offset;
                                    offsets.push((tone.id, offset));
                                }
                            }

                            for entry in track.entries.by_id.values() {
                                let Entry::Tuplet(tuplet) = entry else {
                                    continue;
                                };

                                let tuplet_clef = master
                                    .get_clef_before_tick_inclusive(tuplet.tick)
                                    .unwrap_or(clef);

                                for child_entry in tuplet.entries.by_id.values() {
                                    let TupletEntry::Tone(tone) = child_entry;
                                    let offset =
                                        Pitch::steps_between(&tone.pitch, &tuplet_clef.pitch)
                                            + tuplet_clef.offset;
                                    offsets.push((tone.id, offset));
                                }
                            }

                            offsets
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for (tone_id, offset) in results {
        output.insert(tone_id, offset);
    }

    output
}
