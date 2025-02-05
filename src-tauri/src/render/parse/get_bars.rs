use rustc_hash::FxHashMap;

use crate::score::components::duration::Tick;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::flows::Flow;
use crate::score::tracks::Track;

pub type Bars = FxHashMap<Tick, TimeSignature>;

pub fn get_bars(flow: &Flow, master: &Track) -> Bars {
    let mut output: Bars = FxHashMap::default();
    let mut current_tick = 0;
    let mut current_time_signature = TimeSignature::default();

    while current_tick < flow.length {
        if let Some(entry) = master.get_time_signature_at_tick(&current_tick) {
            current_time_signature = entry.clone();
        }

        output.insert(current_tick, current_time_signature.clone());

        // Jump to the next barline
        current_tick += current_time_signature.bar_length_in_ticks(flow.subdivisions);
    }

    output
}
