use crate::score::components::duration::Tick;
use crate::score::entries::key_signature::KeySignature;
use crate::score::flows::Flow;
use crate::score::tracks::Tracks;
use rustc_hash::FxHashMap;

pub struct KeySignatureDrawDef {
    pub negate: bool,
    pub key_signature: KeySignature,
}

impl KeySignatureDrawDef {
    pub fn new(negate: bool, key_signature: KeySignature) -> Self {
        Self {
            negate,
            key_signature,
        }
    }
}

pub type KeySignatures = FxHashMap<Tick, KeySignatureDrawDef>;

pub fn get_key_signatures(flow: &Flow, tracks: &Tracks) -> KeySignatures {
    let mut output = KeySignatures::default();

    let master = &tracks.by_id[&flow.master_id];

    let mut key_signature = KeySignature::default();

    for tick in 0..flow.length {
        if let Some(found) = master.get_key_signature_at_tick(&tick) {
            if found.offset == 0 {
                output.insert(tick, KeySignatureDrawDef::new(true, key_signature.clone()));
            } else {
                output.insert(tick, KeySignatureDrawDef::new(false, found.clone()));
            }

            key_signature = found.clone();
        };
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::flows::Flow;
    use crate::score::tracks::{Track, Tracks};

    #[test]
    fn key_signature_draw_def_new_stores_fields() {
        use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
        let ks = KeySignature::new(1, 0, KeySignatureMode::Major, 2);
        let def = KeySignatureDrawDef::new(false, ks);
        assert!(!def.negate);
    }

    #[test]
    fn key_signature_draw_def_negate_true() {
        use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
        let ks = KeySignature::new(1, 0, KeySignatureMode::Major, 0);
        let def = KeySignatureDrawDef::new(true, ks);
        assert!(def.negate);
    }

    #[test]
    fn get_key_signatures_empty_flow_returns_empty() {
        let mut flow = Flow::new(1, 100);
        flow.length = 0; // no ticks to iterate
        let mut tracks = Tracks::new();
        tracks.by_id.insert(100, Track::new(100));
        let result = get_key_signatures(&flow, &tracks);
        assert!(result.is_empty());
    }

    #[test]
    fn get_key_signatures_no_entries_returns_empty() {
        let mut flow = Flow::new(1, 100);
        flow.length = 128; // one whole note worth of ticks
        let mut tracks = Tracks::new();
        tracks.by_id.insert(100, Track::new(100));
        // no key signature entries in master track
        let result = get_key_signatures(&flow, &tracks);
        assert!(result.is_empty());
    }
}
