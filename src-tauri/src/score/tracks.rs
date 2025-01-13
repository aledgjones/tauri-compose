use super::{
    components::duration::Tick,
    entries::{Entries, Entry},
};
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Tracks {
    pub by_id: FxHashMap<UniqueId, Track>,
}

impl Tracks {
    pub fn new() -> Tracks {
        Tracks {
            by_id: FxHashMap::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: UniqueId,
    pub entries: Entries,
}

impl Track {
    pub fn new(id: UniqueId) -> Track {
        Track {
            id,
            entries: Entries::new(),
        }
    }

    /// Insert and entry into the track
    pub fn insert(&mut self, entry: Entry) {
        let tick = self.entries.by_tick.entry(entry.tick()).or_default();
        tick.push(entry.id());
        self.entries.by_id.insert(entry.id(), entry);
    }

    /// Move an entry to a new tick
    pub fn shift(&mut self, id: UniqueId, new_tick: Tick) {
        let entry = match self.entries.by_id.get_mut(&id) {
            Some(entry) => entry,
            None => return,
        };

        let old_tick = entry.tick();

        // move the entry tp the new tick only if it has actually moved
        if old_tick != new_tick {
            entry.set_tick(new_tick);
            // move the entry key to the new tick
            if let Some(keys) = self.entries.by_tick.get_mut(&old_tick) {
                keys.retain(|item_id| *item_id != id);
            };
            let tick = self.entries.by_tick.entry(new_tick).or_default();
            tick.push(id);
        }
    }

    /// remove an entry and return the removed entry
    pub fn remove(&mut self, id: UniqueId) -> Option<Entry> {
        let entry = self.entries.by_id.get(&id)?;

        if let Some(ids) = self.entries.by_tick.get_mut(&entry.tick()) {
            ids.retain(|item_id| *item_id != id);
        };

        self.entries.by_id.remove(&id)
    }
}
