use super::components::duration::{Tick, Ticks};
use super::entries::tone::Tone;
use super::entries::{Entries, Entry};
use crate::utils::UniqueId;
use rustc_hash::{FxHashMap, FxHashSet};
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
    fn stateful_entry_kind(entry: &Entry) -> Option<u8> {
        match entry {
            Entry::Clef(_) => Some(0),
            Entry::KeySignature(_) => Some(1),
            Entry::PlayingTechnique(_) => Some(2),
            Entry::Tempo(_) => Some(3),
            Entry::TimeSignature(_) => Some(4),
            Entry::Barline(_) | Entry::Tone(_) | Entry::Tuplet(_) => None,
        }
    }

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

    fn validate_splice_boundary(&self, tick: Tick) -> Result<(), String> {
        for entry in self.entries.by_id.values() {
            let Entry::Tuplet(tuplet) = entry else {
                continue;
            };
            let end = tuplet.tick + tuplet.consumed_ticks();
            if tick > tuplet.tick && tick < end {
                return Err(format!(
                    "Tick {tick} cuts through tuplet {} on track {}",
                    tuplet.id, self.id
                ));
            }
        }
        Ok(())
    }

    pub fn insert_time(
        &mut self,
        tick: Tick,
        duration: Ticks,
        mut generate_id: impl FnMut() -> UniqueId,
    ) -> Result<(), String> {
        if duration == 0 {
            return Err("Inserted duration must be greater than zero".to_string());
        }
        tick.checked_add(duration)
            .ok_or_else(|| "Inserted time exceeds the tick range".to_string())?;
        self.validate_splice_boundary(tick)?;

        let entries = self.entries.by_id.values().cloned().collect::<Vec<_>>();
        for entry in entries {
            match entry {
                Entry::Tone(tone) if tone.tick < tick && tone.tick + tone.duration > tick => {
                    let tail_duration = tone.tick + tone.duration - tick;
                    self.set_tone_duration(tone.id, tick - tone.tick, None)?;
                    self.insert_entry(Entry::Tone(Tone::new(
                        generate_id(),
                        tick + duration,
                        tail_duration,
                        tone.pitch,
                        tone.velocity,
                        tone.articulations,
                    )));
                }
                entry if entry.tick() >= tick => {
                    let shifted_tick = entry
                        .tick()
                        .checked_add(duration)
                        .ok_or_else(|| "Shifted entry exceeds the tick range".to_string())?;
                    self.shift_entry(entry.id(), shifted_tick);
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn delete_time(&mut self, start: Tick, duration: Ticks) -> Result<(), String> {
        if duration == 0 {
            return Err("Deleted duration must be greater than zero".to_string());
        }
        let end = start
            .checked_add(duration)
            .ok_or_else(|| "Deleted time exceeds the tick range".to_string())?;
        self.validate_splice_boundary(start)?;
        self.validate_splice_boundary(end)?;

        let entries = self.entries.by_id.values().cloned().collect::<Vec<_>>();
        let stateful_kinds_at_end = entries
            .iter()
            .filter(|entry| entry.tick() == end)
            .filter_map(Self::stateful_entry_kind)
            .collect::<FxHashSet<_>>();
        let mut latest_stateful = FxHashMap::<u8, (Tick, UniqueId)>::default();
        for entry in &entries {
            if entry.tick() < start || entry.tick() >= end {
                continue;
            }
            let Some(kind) = Self::stateful_entry_kind(entry) else {
                continue;
            };
            if stateful_kinds_at_end.contains(&kind) {
                continue;
            }
            let candidate = (entry.tick(), entry.id());
            if latest_stateful
                .get(&kind)
                .is_none_or(|current| candidate.0 >= current.0)
            {
                latest_stateful.insert(kind, candidate);
            }
        }
        let preserved_ids = latest_stateful
            .values()
            .map(|(_, id)| *id)
            .collect::<FxHashSet<_>>();

        for entry in entries {
            match entry {
                Entry::Tone(tone) => {
                    let tone_end = tone.tick + tone.duration;
                    if tone_end <= start {
                        continue;
                    }
                    if tone.tick >= end {
                        self.shift_entry(tone.id, tone.tick - duration);
                    } else if tone.tick >= start && tone_end <= end {
                        self.remove_entry(tone.id);
                    } else if tone.tick < start && tone_end <= end {
                        self.set_tone_duration(tone.id, start - tone.tick, None)?;
                    } else if tone.tick >= start && tone.tick < end && tone_end > end {
                        self.set_tone_position_and_duration(
                            tone.id,
                            start,
                            tone_end - end,
                            None,
                            None,
                        )?;
                    } else if tone.tick < start && tone_end > end {
                        self.set_tone_duration(tone.id, tone.duration - duration, None)?;
                    }
                }
                entry if entry.tick() >= start && entry.tick() < end => {
                    if preserved_ids.contains(&entry.id()) {
                        self.shift_entry(entry.id(), start);
                    } else {
                        self.remove_entry(entry.id());
                    }
                }
                entry if entry.tick() >= end => {
                    self.shift_entry(entry.id(), entry.tick() - duration);
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::{
        articulation::Articulations,
        duration::NoteDuration,
        pitch::{Accidental, Pitch},
        velocity::Velocity,
    };
    use crate::score::entries::{
        playing_technique::{PlayingTechnique, PlayingTechniqueKind},
        tone::Tone,
        tuplet::{Tuplet, TupletEntries, TupletEntry},
        Entry,
    };

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

    fn make_tuplet(id: UniqueId, tick: Tick) -> Entry {
        let mut entries = TupletEntries::new();
        entries.insert_entry(TupletEntry::Tone(Tone::new(
            id + 100,
            16,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        Entry::Tuplet(Box::new(Tuplet {
            id,
            tick,
            note_count: 3,
            in_space_of: 2,
            base_note_value: NoteDuration::Eighth,
            groupings: vec![3],
            entries,
        }))
    }

    #[test]
    fn insert_time_splits_crossing_tone_and_leaves_an_empty_gap() {
        let mut track = Track::new(1);
        track.insert_entry(make_tone(5, 64));

        track.insert_time(80, 32, || 6).unwrap();

        let Entry::Tone(head) = &track.entries.by_id[&5] else {
            panic!("expected head tone");
        };
        let Entry::Tone(tail) = &track.entries.by_id[&6] else {
            panic!("expected tail tone");
        };
        assert_eq!((head.tick, head.duration), (64, 16));
        assert_eq!((tail.tick, tail.duration), (112, 16));
        assert_eq!(track.entries.by_tick.get(&112), Some(&vec![6]));
    }

    #[test]
    fn insert_time_shifts_entries_and_whole_tuplets_at_boundary() {
        let mut track = Track::new(1);
        track.insert_entry(make_tone(5, 96));
        track.insert_entry(make_tuplet(20, 128));

        track.insert_time(96, 32, || 99).unwrap();

        assert_eq!(track.entries.by_id[&5].tick(), 128);
        assert_eq!(track.entries.by_id[&20].tick(), 160);
    }

    #[test]
    fn insert_time_rejects_boundary_inside_tuplet_without_mutating() {
        let mut track = Track::new(1);
        track.insert_entry(make_tuplet(20, 64));
        let original = track.clone();

        let error = track.insert_time(72, 32, || 99).unwrap_err();

        assert!(error.contains("cuts through tuplet 20"));
        assert_eq!(
            track.entries.by_id[&20].tick(),
            original.entries.by_id[&20].tick()
        );
    }

    #[test]
    fn delete_time_reconciles_all_root_tone_overlap_shapes() {
        let mut track = Track::new(1);
        let mut before = make_tone(1, 0);
        let mut clipped_head = make_tone(2, 48);
        let mut spanning = make_tone(3, 48);
        let mut removed = make_tone(4, 72);
        let mut clipped_tail = make_tone(5, 80);
        let mut shifted = make_tone(6, 128);
        if let Entry::Tone(tone) = &mut before {
            tone.duration = 32;
        }
        if let Entry::Tone(tone) = &mut clipped_head {
            tone.duration = 32;
        }
        if let Entry::Tone(tone) = &mut spanning {
            tone.duration = 96;
        }
        if let Entry::Tone(tone) = &mut removed {
            tone.duration = 16;
        }
        if let Entry::Tone(tone) = &mut clipped_tail {
            tone.duration = 64;
        }
        if let Entry::Tone(tone) = &mut shifted {
            tone.duration = 16;
        }
        for entry in [
            before,
            clipped_head,
            spanning,
            removed,
            clipped_tail,
            shifted,
        ] {
            track.insert_entry(entry);
        }

        track.delete_time(64, 32).unwrap();

        let tone = |id| match &track.entries.by_id[&id] {
            Entry::Tone(tone) => (tone.tick, tone.duration),
            _ => panic!("expected tone"),
        };
        assert_eq!(tone(1), (0, 32));
        assert_eq!(tone(2), (48, 16));
        assert_eq!(tone(3), (48, 64));
        assert!(!track.entries.by_id.contains_key(&4));
        assert_eq!(tone(5), (64, 48));
        assert_eq!(tone(6), (96, 16));
    }

    #[test]
    fn delete_time_removes_and_shifts_whole_tuplets() {
        let mut track = Track::new(1);
        track.insert_entry(make_tuplet(20, 64));
        track.insert_entry(make_tuplet(30, 128));

        track.delete_time(64, 64).unwrap();

        assert!(!track.entries.by_id.contains_key(&20));
        assert_eq!(track.entries.by_id[&30].tick(), 64);
    }

    #[test]
    fn delete_time_rejects_boundary_inside_tuplet_without_mutating() {
        let mut track = Track::new(1);
        track.insert_entry(make_tuplet(20, 64));

        let error = track.delete_time(72, 16).unwrap_err();

        assert!(error.contains("cuts through tuplet 20"));
        assert_eq!(track.entries.by_id[&20].tick(), 64);
    }

    #[test]
    fn delete_time_preserves_latest_stateful_entry_at_splice_point() {
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};

        let mut track = Track::new(1);
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            10,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            11,
            32,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));

        track.delete_time(0, 64).unwrap();

        assert!(!track.entries.by_id.contains_key(&10));
        assert_eq!(track.entries.by_id[&11].tick(), 0);
    }

    #[test]
    fn delete_time_prefers_stateful_entry_at_end_boundary() {
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};

        let mut track = Track::new(1);
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            10,
            64,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            11,
            128,
            5,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));

        track.delete_time(64, 64).unwrap();

        assert!(!track.entries.by_id.contains_key(&10));
        assert_eq!(track.entries.by_id[&11].tick(), 64);
    }

    fn make_playing_technique(id: UniqueId, tick: Tick) -> Entry {
        Entry::PlayingTechnique(PlayingTechnique::new(id, tick, PlayingTechniqueKind::Pizz))
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

    #[test]
    fn insert_playing_technique_adds_to_both_indices() {
        let mut track = Track::new(1);
        track.insert_entry(make_playing_technique(30, 80));

        assert!(track.entries.by_id.contains_key(&30));
        let ids = track.entries.by_tick.get(&80).unwrap();
        assert!(ids.contains(&30));
    }

    #[test]
    fn shift_playing_technique_moves_to_new_tick() {
        let mut track = Track::new(1);
        track.insert_entry(make_playing_technique(31, 80));

        track.shift_entry(31, 144);

        let at_80 = track.entries.by_tick.get(&80).unwrap();
        assert!(!at_80.contains(&31));
        let at_144 = track.entries.by_tick.get(&144).unwrap();
        assert!(at_144.contains(&31));
        assert_eq!(track.entries.by_id.get(&31).unwrap().tick(), 144);
    }

    #[test]
    fn remove_playing_technique_removes_from_both_indices() {
        let mut track = Track::new(1);
        track.insert_entry(make_playing_technique(32, 160));

        let removed = track.remove_entry(32);

        assert!(removed.is_some());
        assert!(!track.entries.by_id.contains_key(&32));
        let at_160 = track.entries.by_tick.get(&160).unwrap();
        assert!(!at_160.contains(&32));
    }

    #[test]
    fn insert_tuplet_adds_to_both_indices() {
        let mut track = Track::new(1);
        track.insert_entry(make_tuplet(20, 96));

        assert!(track.entries.by_id.contains_key(&20));
        let ids = track.entries.by_tick.get(&96).unwrap();
        assert!(ids.contains(&20));
    }

    #[test]
    fn shift_tuplet_moves_to_new_tick() {
        let mut track = Track::new(1);
        track.insert_entry(make_tuplet(21, 96));

        track.shift_entry(21, 160);

        let at_96 = track.entries.by_tick.get(&96).unwrap();
        assert!(!at_96.contains(&21));
        let at_160 = track.entries.by_tick.get(&160).unwrap();
        assert!(at_160.contains(&21));
        assert_eq!(track.entries.by_id.get(&21).unwrap().tick(), 160);
    }

    #[test]
    fn remove_tuplet_removes_from_both_indices() {
        let mut track = Track::new(1);
        track.insert_entry(make_tuplet(22, 192));

        let removed = track.remove_entry(22);

        assert!(removed.is_some());
        assert!(!track.entries.by_id.contains_key(&22));
        let at_192 = track.entries.by_tick.get(&192).unwrap();
        assert!(!at_192.contains(&22));
    }
}
