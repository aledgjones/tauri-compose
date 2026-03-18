use crate::score::components::duration::Tick;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::flows::Flow;
use crate::score::tracks::Tracks;
use rustc_hash::FxHashMap;

pub type Measures = FxHashMap<Tick, TimeSignature>;

pub fn measures_from_single_time_signature(time_signature: TimeSignature) -> Measures {
    let mut measures = Measures::default();
    measures.insert(0, time_signature);
    measures
}

pub fn get_measures(flow: &Flow, tracks: &Tracks) -> Measures {
    let mut output = Measures::default();
    let mut current_tick = 0;
    let mut current_time_signature = TimeSignature::default();

    let master = &tracks.by_id[&flow.master_id];

    while current_tick < flow.length {
        if let Some(entry) = master.get_time_signature_at_tick(&current_tick) {
            current_time_signature = entry.clone();
        }

        output.insert(current_tick, current_time_signature.clone());

        // Jump to the next barline
        current_tick += current_time_signature.ticks_per_bar();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::duration::NoteDuration;
    use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
    use crate::score::entries::Entry;
    use crate::score::flows::Flow;
    use crate::score::staves::Staves;
    use crate::score::tracks::{Track, Tracks};
    use rustc_hash::{FxHashMap, FxHashSet};

    fn test_flow(master_id: u64, length: u64) -> Flow {
        Flow {
            id: 1,
            title: String::new(),
            player_ids: FxHashSet::default(),
            length,
            master_id,
            staves: Staves::new(),
        }
    }

    fn build_tracks(master_id: u64, time_sigs: Vec<TimeSignature>) -> Tracks {
        let mut master = Track::new(master_id);
        for ts in time_sigs {
            master.insert_entry(Entry::TimeSignature(ts));
        }
        let mut by_id = FxHashMap::default();
        by_id.insert(master_id, master);
        Tracks { by_id }
    }

    #[test]
    fn single_4_4_time_sig_produces_four_measures() {
        let master_id = 100u64;
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        let flow = test_flow(master_id, 512);
        let tracks = build_tracks(master_id, vec![ts]);

        let measures = get_measures(&flow, &tracks);

        assert_eq!(measures.len(), 4, "expected 4 measures");
        assert!(measures.contains_key(&0), "measure at tick 0");
        assert!(measures.contains_key(&128), "measure at tick 128");
        assert!(measures.contains_key(&256), "measure at tick 256");
        assert!(measures.contains_key(&384), "measure at tick 384");
    }

    #[test]
    fn time_sig_change_produces_correct_boundaries() {
        let master_id = 200u64;
        // 4/4 at tick 0 → ticks_per_bar = 32 * 4 = 128
        let ts_4_4 = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        // 3/4 at tick 256 → ticks_per_bar = 32 * 3 = 96
        let ts_3_4 = TimeSignature::new(
            2,
            256,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        // 2 bars of 4/4 = 256 ticks, 2 bars of 3/4 = 192 ticks → length = 448
        let flow = test_flow(master_id, 448);
        let tracks = build_tracks(master_id, vec![ts_4_4, ts_3_4]);

        let measures = get_measures(&flow, &tracks);

        assert_eq!(measures.len(), 4, "expected 4 measures total");
        assert!(measures.contains_key(&0), "4/4 bar 1 at tick 0");
        assert!(measures.contains_key(&128), "4/4 bar 2 at tick 128");
        assert!(measures.contains_key(&256), "3/4 bar 1 at tick 256");
        assert!(measures.contains_key(&352), "3/4 bar 2 at tick 352");
    }

    #[test]
    fn single_time_signature_helper_starts_at_zero() {
        let time_signature = TimeSignature::new(
            1,
            24,
            5,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            Some(vec![3, 2]),
        );

        let measures = measures_from_single_time_signature(time_signature.clone());

        assert_eq!(measures.len(), 1);
        assert!(measures.get(&0).unwrap().beats == time_signature.beats);
        assert!(measures.get(&0).unwrap().beat_type == time_signature.beat_type);
        assert!(measures.get(&0).unwrap().groupings == time_signature.groupings);
    }
}
