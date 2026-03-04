use super::components::duration::Tick;
use super::entries::{Entries, Entry};
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
    pub fn insert_entry(&mut self, entry: Entry) {
        let tick = self.entries.by_tick.entry(entry.tick()).or_default();
        tick.push(entry.id());
        self.entries.by_id.insert(entry.id(), entry);
    }

    /// Move an entry to a new tick
    pub fn shift_entry(&mut self, entry_id: UniqueId, new_tick: Tick) {
        let entry = match self.entries.by_id.get_mut(&entry_id) {
            Some(entry) => entry,
            None => return,
        };

        let old_tick = entry.tick();

        // move the entry to the new tick only if it has actually moved
        if old_tick != new_tick {
            entry.set_tick(new_tick);
            // move the entry key to the new tick
            if let Some(keys) = self.entries.by_tick.get_mut(&old_tick) {
                keys.retain(|item_id| *item_id != entry_id);
            };
            let tick = self.entries.by_tick.entry(new_tick).or_default();
            tick.push(entry_id);
        }
    }

    /// remove an entry and return the removed entry
    pub fn remove_entry(&mut self, entry_id: UniqueId) -> Option<Entry> {
        let entry = self.entries.by_id.get(&entry_id)?;

        if let Some(ids) = self.entries.by_tick.get_mut(&entry.tick()) {
            ids.retain(|item_id| *item_id != entry_id);
        };

        self.entries.by_id.remove(&entry_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::{
        articulation::Articulations,
        pitch::{Accidental, Pitch},
        velocity::Velocity,
    };
    use crate::score::entries::{tone::Tone, Entry};

    fn make_tone(id: UniqueId, tick: Tick) -> Entry {
        Entry::Tone(Tone::new(
            id,
            tick,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        ))
    }

    #[test]
    fn insert_entry_adds_to_both_indices() {
        let mut track = Track::new(1);
        track.insert_entry(make_tone(5, 64));
        assert!(track.entries.by_id.contains_key(&5));
        let ids = track.entries.by_tick.get(&64).unwrap();
        assert!(ids.contains(&5));
    }

    #[test]
    fn insert_multiple_entries_at_same_tick() {
        let mut track = Track::new(1);
        track.insert_entry(make_tone(1, 64));
        track.insert_entry(make_tone(2, 64));
        let ids = track.entries.by_tick.get(&64).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn shift_entry_moves_to_new_tick() {
        let mut track = Track::new(1);
        track.insert_entry(make_tone(5, 64));
        track.shift_entry(5, 128);
        let at_64 = track.entries.by_tick.get(&64).unwrap();
        assert!(!at_64.contains(&5));
        let at_128 = track.entries.by_tick.get(&128).unwrap();
        assert!(at_128.contains(&5));
        assert!(track.entries.by_id.contains_key(&5));
    }

    #[test]
    fn shift_entry_noop_when_same_tick() {
        let mut track = Track::new(1);
        track.insert_entry(make_tone(5, 64));
        track.shift_entry(5, 64);
        let ids = track.entries.by_tick.get(&64).unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&5));
    }

    #[test]
    fn remove_entry_removes_from_both_indices() {
        let mut track = Track::new(1);
        track.insert_entry(make_tone(5, 64));
        let removed = track.remove_entry(5);
        assert!(removed.is_some());
        assert!(!track.entries.by_id.contains_key(&5));
        let at_64 = track.entries.by_tick.get(&64).unwrap();
        assert!(!at_64.contains(&5));
    }

    #[test]
    fn remove_entry_returns_none_for_missing_id() {
        let mut track = Track::new(1);
        let removed = track.remove_entry(999);
        assert!(removed.is_none());
    }
}
