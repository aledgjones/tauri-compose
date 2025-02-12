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
    let mut output: KeySignatures = FxHashMap::default();

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
