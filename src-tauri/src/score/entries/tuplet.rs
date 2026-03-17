use super::{tone::Tone, Entry};
use crate::score::components::articulation::Articulations;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::{AppContextState, RendererContextState};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TupletEntry {
    Tone(Tone),
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct TupletEntries {
    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
    pub by_tick: FxHashMap<Tick, Vec<UniqueId>>,
    #[serde_as(as = "FxHashMap<DisplayFromStr, _>")]
    pub by_id: FxHashMap<UniqueId, TupletEntry>,
}

#[allow(dead_code)]
impl TupletEntries {
    pub fn new() -> Self {
        Self {
            by_tick: FxHashMap::default(),
            by_id: FxHashMap::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn get(&self, entry_id: &UniqueId) -> Option<&TupletEntry> {
        self.by_id.get(entry_id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn insert_entry(&mut self, entry: TupletEntry) {
        let (entry_id, tick) = match &entry {
            TupletEntry::Tone(tone) => (tone.id, tone.tick),
        };

        let tick_entries = self.by_tick.entry(tick).or_default();
        tick_entries.push(entry_id);
        self.by_id.insert(entry_id, entry);
    }

    pub fn remove_entry(&mut self, entry_id: UniqueId) -> Option<TupletEntry> {
        let entry = self.by_id.get(&entry_id)?;

        let tick = match entry {
            TupletEntry::Tone(tone) => tone.tick,
        };

        if let Some(ids) = self.by_tick.get_mut(&tick) {
            ids.retain(|item_id| *item_id != entry_id);
            if ids.is_empty() {
                self.by_tick.remove(&tick);
            }
        }

        self.by_id.remove(&entry_id)
    }

    pub fn first_articulations_at_tick(
        &self,
        tick: &Tick,
    ) -> Option<Articulations> {
        if let Some(tone_ids) = self.by_tick.get(tick) {
            for tone_id in tone_ids {
                if let Some(TupletEntry::Tone(tone)) = self.by_id.get(tone_id) {
                    if !tone.articulations.is_empty() {
                        return Some(tone.articulations.clone());
                    }
                }
            }
        }

        None
    }

    pub fn apply_articulations_to_tick(
        &mut self,
        tick: Tick,
        articulations: &Articulations,
    ) {
        if let Some(tone_ids) = self.by_tick.get(&tick).cloned() {
            for tone_id in tone_ids {
                if let Some(TupletEntry::Tone(tone)) = self.by_id.get_mut(&tone_id) {
                    tone.articulations = articulations.clone();
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tuplet {
    pub id: UniqueId,
    pub tick: Tick,
    pub note_count: u8,
    pub in_space_of: u8,
    pub base_note_value: NoteDuration,
    pub groupings: Vec<u8>,
    pub entries: TupletEntries,
}

#[allow(dead_code)]
impl Tuplet {
    pub fn new(
        id: UniqueId,
        tick: Tick,
        note_count: u8,
        in_space_of: u8,
        base_note_value: NoteDuration,
        groupings: Vec<u8>,
    ) -> Self {
        Self {
            id,
            tick,
            note_count,
            in_space_of,
            base_note_value,
            groupings,
            entries: TupletEntries::new(),
        }
    }

    pub fn consumed_ticks(&self) -> Ticks {
        u64::from(self.in_space_of) * self.base_note_value.to_ticks()
    }

    pub fn local_length_ticks(&self) -> Ticks {
        u64::from(self.note_count) * self.base_note_value.to_ticks()
    }
}

impl Track {
    fn has_overlapping_tuplet_span(&self, start_tick: Tick, end_tick: Tick) -> bool {
        self.entries.by_id.values().any(|entry| {
            let Entry::Tuplet(tuplet) = entry else {
                return false;
            };

            let tuplet_start = tuplet.tick;
            let tuplet_end = tuplet.tick + tuplet.consumed_ticks();

            start_tick < tuplet_end && end_tick > tuplet_start
        })
    }

    fn tone_ids_starting_in_span(&self, start_tick: Tick, end_tick: Tick) -> Vec<UniqueId> {
        self.entries
            .by_id
            .values()
            .filter_map(|entry| match entry {
                Entry::Tone(tone) if tone.tick >= start_tick && tone.tick < end_tick => {
                    Some(tone.id)
                }
                _ => None,
            })
            .collect()
    }

    pub fn create_tuplet(
        &mut self,
        tuplet_id: UniqueId,
        tick: Tick,
        note_count: u8,
        in_space_of: u8,
        base_note_value: NoteDuration,
        groupings: Vec<u8>,
    ) -> Result<UniqueId, String> {
        let consumed_ticks = u64::from(in_space_of) * base_note_value.to_ticks();
        let end_tick = tick + consumed_ticks;

        if self.has_overlapping_tuplet_span(tick, end_tick) {
            return Err(format!(
                "Tuplet creation at tick {tick} overlaps an existing tuplet on track {}",
                self.id
            ));
        }

        let tone_ids_to_move = self.tone_ids_starting_in_span(tick, end_tick);
        let mut tuplet = Tuplet::new(
            tuplet_id,
            tick,
            note_count,
            in_space_of,
            base_note_value,
            groupings,
        );

        for tone_id in tone_ids_to_move {
            let Some(Entry::Tone(mut tone)) = self.remove_entry(tone_id) else {
                continue;
            };

            tone.tick -= tick;
            tuplet.entries.insert_entry(TupletEntry::Tone(tone));
        }

        self.insert_entry(Entry::Tuplet(Box::new(tuplet)));

        Ok(tuplet_id)
    }

    pub fn remove_tuplet(&mut self, tuplet_id: UniqueId) -> Result<(), String> {
        match self.remove_entry(tuplet_id) {
            Some(Entry::Tuplet(_)) => Ok(()),
            Some(_) | None => Err(format!("Tuplet {tuplet_id} not found on track {}", self.id)),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)]
pub fn score_entries_tuplet_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tick: Tick,
    note_count: u8,
    in_space_of: u8,
    base_note_value: NoteDuration,
    groupings: Vec<u8>,
) -> Result<UniqueId, String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let tuplet_id = state.generate_id();
    let track = state
        .score
        .tracks
        .by_id
        .get_mut(&track_id)
        .ok_or_else(|| format!("Track {track_id} not found"))?;

    track.create_tuplet(
        tuplet_id,
        tick,
        note_count,
        in_space_of,
        base_note_value,
        groupings,
    )?;

    state.emit_affected_track(&app, &mut renderer, track_id);

    Ok(tuplet_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tuplet_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tuplet_id: UniqueId,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state
        .score
        .tracks
        .by_id
        .get_mut(&track_id)
        .ok_or_else(|| format!("Track {track_id} not found"))?;

    track.remove_tuplet(tuplet_id)?;

    state.ui.sequencer.selection = None;
    state.emit_affected_track(&app, &mut renderer, track_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::{
        articulation::Articulations,
        pitch::{Accidental, Pitch},
        velocity::Velocity,
    };
    use crate::score::entries::tone::Tone;

    #[test]
    fn triplet_ratio_derives_expected_tick_spans() {
        let tuplet = Tuplet::new(1, 64, 3, 2, NoteDuration::Eighth, vec![3]);

        assert_eq!(tuplet.local_length_ticks(), 48);
        assert_eq!(tuplet.consumed_ticks(), 32);
    }

    #[test]
    fn create_tuplet_inserts_entry_with_expected_ratio_metadata() {
        let mut track = Track::new(9);
        track.insert_entry(Entry::Tone(Tone::new(
            22,
            16,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        track
            .create_tuplet(
            99,
            64,
            3,
            2,
            NoteDuration::Eighth,
            vec![3],
            )
            .unwrap();

        let Entry::Tuplet(tuplet) = track.entries.by_id.get(&99).unwrap() else {
            panic!("expected tuplet entry");
        };

        assert_eq!(tuplet.tick, 64);
        assert_eq!(tuplet.note_count, 3);
        assert_eq!(tuplet.in_space_of, 2);
        assert!(matches!(tuplet.base_note_value, NoteDuration::Eighth));
        assert_eq!(tuplet.groupings, vec![3]);
        assert!(tuplet.entries.is_empty());
        assert!(matches!(track.entries.by_id.get(&22), Some(Entry::Tone(_))));
    }

    #[test]
    fn create_tuplet_preserves_provided_groupings() {
        let mut track = Track::new(9);

        track
            .create_tuplet(
            99,
            64,
            3,
            2,
            NoteDuration::Eighth,
            vec![2],
            )
            .unwrap();

        let Entry::Tuplet(tuplet) = track.entries.by_id.get(&99).unwrap() else {
            panic!("expected tuplet entry");
        };

        assert_eq!(tuplet.groupings, vec![2]);
    }

    #[test]
    fn create_tuplet_moves_tones_starting_inside_consumed_span_into_child_entries() {
        let mut track = Track::new(9);
        track.insert_entry(Entry::Tone(Tone::new(
            22,
            32,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));
        track.insert_entry(Entry::Tone(Tone::new(
            23,
            48,
            16,
            Pitch::new(62, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        track
            .create_tuplet(99, 32, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        let Entry::Tuplet(tuplet) = track.entries.by_id.get(&99).unwrap() else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(first)) = tuplet.entries.by_id.get(&22) else {
            panic!("expected first carried tone");
        };
        let Some(TupletEntry::Tone(second)) = tuplet.entries.by_id.get(&23) else {
            panic!("expected second carried tone");
        };

        assert_eq!(first.tick, 0);
        assert_eq!(first.duration, 16);
        assert_eq!(second.tick, 16);
        assert_eq!(second.duration, 16);
        assert_eq!(tuplet.entries.len(), 2);
        assert!(!track.entries.by_id.contains_key(&22));
        assert!(!track.entries.by_id.contains_key(&23));
    }

    #[test]
    fn create_tuplet_leaves_crossing_host_owned_tones_on_track() {
        let mut track = Track::new(9);
        track.insert_entry(Entry::Tone(Tone::new(
            21,
            16,
            32,
            Pitch::new(58, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));
        track.insert_entry(Entry::Tone(Tone::new(
            22,
            48,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        track
            .create_tuplet(99, 32, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        let Entry::Tuplet(tuplet) = track.entries.by_id.get(&99).unwrap() else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(carried)) = tuplet.entries.by_id.get(&22) else {
            panic!("expected carried tone");
        };

        assert_eq!(carried.tick, 16);
        assert_eq!(carried.duration, 32);
        assert!(matches!(track.entries.by_id.get(&21), Some(Entry::Tone(_))));
        assert_eq!(track.entries.by_id.len(), 2);
        assert_eq!(tuplet.entries.len(), 1);
    }

    #[test]
    fn tuplet_child_entries_store_tones_only() {
        let mut child_entries = TupletEntries::new();
        let tone = Tone::new(
            11,
            16,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        );

        child_entries.insert_entry(TupletEntry::Tone(tone));

        assert_eq!(child_entries.by_id.len(), 1);
        assert!(matches!(child_entries.by_id.get(&11), Some(TupletEntry::Tone(_))));
    }

    #[test]
    fn remove_entry_updates_shared_tick_index_without_collapsing_other_members() {
        let mut child_entries = TupletEntries::new();
        let first_tone = Tone::new(
            11,
            16,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        );
        let second_tone = Tone::new(
            12,
            16,
            16,
            Pitch::new(64, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        );
        let third_tone = Tone::new(
            13,
            32,
            16,
            Pitch::new(67, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        );

        child_entries.insert_entry(TupletEntry::Tone(first_tone));
        child_entries.insert_entry(TupletEntry::Tone(second_tone));
        child_entries.insert_entry(TupletEntry::Tone(third_tone));

        let removed = child_entries.remove_entry(11);

        assert!(matches!(removed, Some(TupletEntry::Tone(_))));
        assert_eq!(child_entries.by_tick.get(&16), Some(&vec![12]));
        assert_eq!(child_entries.by_tick.get(&32), Some(&vec![13]));
        assert!(!child_entries.by_id.contains_key(&11));
        assert!(child_entries.by_id.contains_key(&12));
        assert!(child_entries.by_id.contains_key(&13));
    }

    #[test]
    fn create_tuplet_rejects_overlapping_existing_tuplet_spans() {
        let mut track = Track::new(9);

        assert_eq!(
            track.create_tuplet(99, 64, 3, 2, NoteDuration::Eighth, vec![3]),
            Ok(99)
        );

        let result = track.create_tuplet(100, 80, 3, 2, NoteDuration::Eighth, vec![3]);

        assert_eq!(
            result,
            Err("Tuplet creation at tick 80 overlaps an existing tuplet on track 9".to_string())
        );
        assert!(!track.entries.by_id.contains_key(&100));
    }

    #[test]
    fn create_tuplet_rejection_leaves_existing_host_ownership_unchanged() {
        let mut track = Track::new(9);
        track.insert_entry(Entry::Tone(Tone::new(
            21,
            80,
            16,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));
        track
            .create_tuplet(99, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        let result = track.create_tuplet(100, 80, 3, 2, NoteDuration::Eighth, vec![3]);

        assert_eq!(
            result,
            Err("Tuplet creation at tick 80 overlaps an existing tuplet on track 9".to_string())
        );
        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&99) else {
            panic!("expected original tuplet entry");
        };
        assert!(matches!(tuplet.entries.by_id.get(&21), Some(TupletEntry::Tone(_))));
        assert!(!track.entries.by_id.contains_key(&100));
    }

    #[test]
    fn remove_tuplet_removes_container_and_child_entries_together() {
        let mut track = Track::new(9);
        track
            .create_tuplet(99, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        let Entry::Tuplet(tuplet) = track.entries.by_id.get_mut(&99).unwrap() else {
            panic!("expected tuplet entry");
        };
        tuplet.entries.insert_entry(TupletEntry::Tone(Tone::new(
            120,
            16,
            24,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        assert_eq!(track.remove_tuplet(99), Ok(()));
        assert!(!track.entries.by_id.contains_key(&99));
        assert!(track
            .entries
            .by_tick
            .get(&64)
            .is_none_or(|ids| !ids.contains(&99)));
        assert!(!track.entries.by_id.contains_key(&120));
    }

    #[test]
    fn remove_tuplet_keeps_other_track_entries_intact() {
        let mut track = Track::new(9);
        track.insert_entry(Entry::Tone(Tone::new(
            21,
            16,
            16,
            Pitch::new(58, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));
        track
            .create_tuplet(99, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        assert_eq!(track.remove_tuplet(99), Ok(()));
        assert!(matches!(track.entries.by_id.get(&21), Some(Entry::Tone(_))));
        assert_eq!(track.entries.by_tick.get(&16), Some(&vec![21]));
    }
}
