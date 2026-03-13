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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staves_new_is_empty() {
        let s = Staves::new();
        assert!(s.by_id.is_empty());
    }

    #[test]
    fn stave_direct_construction_stores_id() {
        let s = Stave {
            id: 7,
            lines: vec![1, 1, 1, 1, 1],
            master_id: 99,
            track_ids: Vec::new(),
        };
        assert_eq!(s.id, 7);
        assert_eq!(s.master_id, 99);
    }

    #[test]
    fn stave_direct_construction_stores_lines() {
        let lines = vec![1u8, 1, 1, 1, 1];
        let s = Stave {
            id: 1,
            lines: lines.clone(),
            master_id: 2,
            track_ids: Vec::new(),
        };
        assert_eq!(s.lines, lines);
    }

    #[test]
    fn stave_direct_construction_has_empty_track_ids() {
        let s = Stave {
            id: 1,
            lines: vec![1, 1, 1, 1, 1],
            master_id: 2,
            track_ids: Vec::new(),
        };
        assert!(s.track_ids.is_empty());
    }

    #[test]
    fn stave_new_from_definition_copies_lines() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::entries::clef::{Clef, ClefDrawType};

        let def = StaveDefinition {
            lines: vec![1, 1, 1],
            clef: Clef::new(
                1,
                0,
                Pitch::new(67, Accidental::Natural),
                2,
                ClefDrawType::G,
            ),
        };
        let s = Stave::new(5, &def, 10);
        assert_eq!(s.lines, vec![1u8, 1, 1]);
        assert_eq!(s.id, 5);
        assert_eq!(s.master_id, 10);
    }
}
