use crate::score::components::duration::Tick;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::flows::Flow;
use crate::score::tracks::Tracks;
use rustc_hash::FxHashMap;

pub type Measures = FxHashMap<Tick, TimeSignature>;

pub fn get_measures(flow: &Flow, tracks: &Tracks) -> Measures {
    let mut output: Measures = FxHashMap::default();
    let mut current_tick = 0;
    let mut current_time_signature = TimeSignature::default();

    let master = &tracks.by_id[&flow.master_id];

    while current_tick < flow.length {
        if let Some(entry) = master.get_time_signature_at_tick(&current_tick) {
            current_time_signature = entry.clone();
        }

        output.insert(current_tick, current_time_signature.clone());

        // Jump to the next barline
        current_tick += current_time_signature.ticks_per_bar(flow.subdivisions);
    }

    output
}
