use super::instruments::defs::StaveDefinition;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Staves {
    pub by_id: FxHashMap<UniqueId, Stave>,
}

impl Staves {
    pub fn new() -> Staves {
        Staves {
            by_id: FxHashMap::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Stave {
    pub id: UniqueId,
    pub lines: Vec<u8>,
    pub master_id: UniqueId,
    pub track_ids: Vec<UniqueId>,
}

impl Stave {
    pub fn new(id: UniqueId, stave_def: &StaveDefinition, master_id: UniqueId) -> Stave {
        Stave {
            id,
            lines: stave_def.lines.clone(),
            master_id,
            track_ids: Vec::new(),
        }
    }
}
