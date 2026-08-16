use super::components::duration::{Tick, Ticks};
use super::entries::time_signature::TimeSignature;
use super::Score;
use crate::utils::UniqueId;

impl Score {
    fn flow_bars(&self, flow_id: UniqueId) -> Result<Vec<(Tick, TimeSignature)>, String> {
        let flow = self
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| format!("Flow {flow_id} not found"))?;
        let master = self.tracks.by_id.get(&flow.master_id).ok_or_else(|| {
            format!(
                "Master track {} not found for flow {flow_id}",
                flow.master_id
            )
        })?;
        let mut bars = Vec::new();
        let mut tick = 0;
        let mut time_signature = TimeSignature::default();
        while tick < flow.length {
            if let Some(found) = master.get_time_signature_at_tick(&tick) {
                time_signature = found.clone();
            }
            let duration = time_signature.ticks_per_bar();
            if duration == 0 {
                return Err(format!(
                    "Flow {flow_id} has a zero-length bar at tick {tick}"
                ));
            }
            bars.push((tick, time_signature.clone()));
            tick = tick
                .checked_add(duration)
                .ok_or_else(|| "Bar boundary exceeds the tick range".to_string())?;
        }
        Ok(bars)
    }

    pub(crate) fn bar_range(
        &self,
        flow_id: UniqueId,
        bar_start_tick: Tick,
    ) -> Result<(Tick, Tick), String> {
        let flow = self
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| format!("Flow {flow_id} not found"))?;
        let starts = self
            .flow_bars(flow_id)?
            .into_iter()
            .map(|(tick, _)| tick)
            .collect::<Vec<_>>();
        let index = starts
            .iter()
            .position(|tick| *tick == bar_start_tick)
            .ok_or_else(|| format!("Tick {bar_start_tick} is not a bar start in flow {flow_id}"))?;
        let end = starts.get(index + 1).copied().unwrap_or(flow.length);
        Ok((bar_start_tick, end))
    }

    pub(crate) fn bar_duration(
        &self,
        flow_id: UniqueId,
        bar_start_tick: Tick,
    ) -> Result<Ticks, String> {
        self.flow_bars(flow_id)?
            .into_iter()
            .find(|(tick, _)| *tick == bar_start_tick)
            .map(|(_, time_signature)| time_signature.ticks_per_bar())
            .ok_or_else(|| format!("Tick {bar_start_tick} is not a bar start in flow {flow_id}"))
    }

    pub(crate) fn previous_bar_start(
        &self,
        flow_id: UniqueId,
        before_tick: Tick,
    ) -> Result<Tick, String> {
        self.flow_bars(flow_id)?
            .into_iter()
            .map(|(tick, _)| tick)
            .filter(|tick| *tick < before_tick)
            .max()
            .ok_or_else(|| format!("Flow {flow_id} has no bar before tick {before_tick}"))
    }

    pub(crate) fn insert_bar_after(
        &mut self,
        flow_id: UniqueId,
        bar_start_tick: Tick,
        next_id: &mut UniqueId,
    ) -> Result<Tick, String> {
        let (_, insertion_tick) = self.bar_range(flow_id, bar_start_tick)?;
        let duration = self.bar_duration(flow_id, bar_start_tick)?;
        self.insert_flow_time(flow_id, insertion_tick, duration, next_id)?;
        Ok(insertion_tick)
    }

    pub(crate) fn delete_bar(
        &mut self,
        flow_id: UniqueId,
        bar_start_tick: Tick,
    ) -> Result<Tick, String> {
        let flow_length = self
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| format!("Flow {flow_id} not found"))?
            .length;
        let (start, end) = self.bar_range(flow_id, bar_start_tick)?;
        if start == 0 && end == flow_length {
            return Err("A flow must contain at least one bar".to_string());
        }
        let selection_tick = if end < flow_length {
            start
        } else {
            self.previous_bar_start(flow_id, start)?
        };
        self.delete_flow_time(flow_id, start, end - start)?;
        Ok(selection_tick)
    }

    fn flow_track_ids(&self, flow_id: UniqueId) -> Result<Vec<UniqueId>, String> {
        let flow = self
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| format!("Flow {flow_id} not found"))?;
        let mut track_ids = vec![flow.master_id];
        for stave in flow.staves.by_id.values() {
            track_ids.push(stave.master_id);
            track_ids.extend(stave.track_ids.iter().copied());
        }
        track_ids.sort_unstable();
        track_ids.dedup();
        Ok(track_ids)
    }

    pub(crate) fn insert_flow_time(
        &mut self,
        flow_id: UniqueId,
        tick: Tick,
        duration: Ticks,
        next_id: &mut UniqueId,
    ) -> Result<(), String> {
        if duration == 0 {
            return Err("Inserted duration must be greater than zero".to_string());
        }
        let flow = self
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| format!("Flow {flow_id} not found"))?;
        if tick > flow.length {
            return Err(format!(
                "Insertion tick {tick} exceeds flow {flow_id} length {}",
                flow.length
            ));
        }
        let new_length = flow
            .length
            .checked_add(duration)
            .ok_or_else(|| "Flow length exceeds the tick range".to_string())?;
        let track_ids = self.flow_track_ids(flow_id)?;

        let mut candidate = self.clone();
        let mut candidate_next_id = *next_id;
        for track_id in track_ids {
            let track = candidate
                .tracks
                .by_id
                .get_mut(&track_id)
                .ok_or_else(|| format!("Track {track_id} not found for flow {flow_id}"))?;
            track.insert_time(tick, duration, || {
                let id = candidate_next_id;
                candidate_next_id += 1;
                id
            })?;
        }
        candidate.flows.by_id.get_mut(&flow_id).unwrap().length = new_length;

        *self = candidate;
        *next_id = candidate_next_id;
        Ok(())
    }

    pub(crate) fn delete_flow_time(
        &mut self,
        flow_id: UniqueId,
        start: Tick,
        duration: Ticks,
    ) -> Result<(), String> {
        if duration == 0 {
            return Err("Deleted duration must be greater than zero".to_string());
        }
        let flow = self
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| format!("Flow {flow_id} not found"))?;
        let end = start
            .checked_add(duration)
            .ok_or_else(|| "Deleted time exceeds the tick range".to_string())?;
        if end > flow.length {
            return Err(format!(
                "Deleted range {start}..{end} exceeds flow {flow_id} length {}",
                flow.length
            ));
        }
        let new_length = flow.length - duration;
        let track_ids = self.flow_track_ids(flow_id)?;

        let mut candidate = self.clone();
        for track_id in track_ids {
            let track = candidate
                .tracks
                .by_id
                .get_mut(&track_id)
                .ok_or_else(|| format!("Track {track_id} not found for flow {flow_id}"))?;
            track.delete_time(start, duration)?;
        }
        candidate.flows.by_id.get_mut(&flow_id).unwrap().length = new_length;

        *self = candidate;
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
        time_signature::{TimeSignature, TimeSignatureDrawType},
        tone::Tone,
        tuplet::{Tuplet, TupletEntries},
        Entry,
    };
    use crate::score::flows::Flow;
    use crate::score::staves::Stave;
    use crate::score::tracks::Track;

    fn score_with_flow() -> Score {
        let mut score = Score::new();
        let mut flow = Flow::new(1, 10);
        flow.length = 256;
        flow.staves.by_id.insert(
            20,
            Stave {
                id: 20,
                lines: vec![1, 1, 1, 1, 1],
                master_id: 30,
                track_ids: vec![40],
            },
        );
        score.flows.order.push(1);
        score.flows.by_id.insert(1, flow);
        score.tracks.by_id.insert(10, Track::new(10));
        score.tracks.by_id.insert(30, Track::new(30));
        score.tracks.by_id.insert(40, Track::new(40));
        score
    }

    fn insert_time_signature(score: &mut Score, id: UniqueId, tick: Tick, beats: u8) {
        score
            .tracks
            .by_id
            .get_mut(&10)
            .unwrap()
            .insert_entry(Entry::TimeSignature(TimeSignature::new(
                id,
                tick,
                beats,
                NoteDuration::Quarter,
                TimeSignatureDrawType::Regular,
                None,
            )));
    }

    fn tone(id: UniqueId, tick: Tick, duration: Ticks) -> Entry {
        Entry::Tone(Tone::new(
            id,
            tick,
            duration,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        ))
    }

    #[test]
    fn insert_flow_time_updates_all_owned_tracks_and_length() {
        let mut score = score_with_flow();
        score
            .tracks
            .by_id
            .get_mut(&10)
            .unwrap()
            .insert_entry(tone(50, 128, 16));
        score
            .tracks
            .by_id
            .get_mut(&30)
            .unwrap()
            .insert_entry(tone(51, 128, 16));
        score
            .tracks
            .by_id
            .get_mut(&40)
            .unwrap()
            .insert_entry(tone(52, 128, 16));
        let mut next_id = 100;

        score.insert_flow_time(1, 128, 32, &mut next_id).unwrap();

        assert_eq!(score.flows.by_id[&1].length, 288);
        assert_eq!(score.tracks.by_id[&10].entries.by_id[&50].tick(), 160);
        assert_eq!(score.tracks.by_id[&30].entries.by_id[&51].tick(), 160);
        assert_eq!(score.tracks.by_id[&40].entries.by_id[&52].tick(), 160);
    }

    #[test]
    fn delete_flow_time_updates_all_owned_tracks_and_length() {
        let mut score = score_with_flow();
        score
            .tracks
            .by_id
            .get_mut(&10)
            .unwrap()
            .insert_entry(tone(50, 192, 16));
        score
            .tracks
            .by_id
            .get_mut(&40)
            .unwrap()
            .insert_entry(tone(52, 192, 16));

        score.delete_flow_time(1, 128, 32).unwrap();

        assert_eq!(score.flows.by_id[&1].length, 224);
        assert_eq!(score.tracks.by_id[&10].entries.by_id[&50].tick(), 160);
        assert_eq!(score.tracks.by_id[&40].entries.by_id[&52].tick(), 160);
    }

    #[test]
    fn failed_insert_is_atomic_across_tracks_and_id_allocation() {
        let mut score = score_with_flow();
        score
            .tracks
            .by_id
            .get_mut(&10)
            .unwrap()
            .insert_entry(tone(50, 96, 64));
        score
            .tracks
            .by_id
            .get_mut(&40)
            .unwrap()
            .insert_entry(Entry::Tuplet(Box::new(Tuplet {
                id: 60,
                tick: 128,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![3],
                entries: TupletEntries::new(),
            })));
        let mut next_id = 100;

        let error = score
            .insert_flow_time(1, 136, 32, &mut next_id)
            .unwrap_err();

        assert!(error.contains("cuts through tuplet 60"));
        assert_eq!(score.flows.by_id[&1].length, 256);
        assert_eq!(score.tracks.by_id[&10].entries.by_id[&50].tick(), 96);
        assert!(!score.tracks.by_id[&10].entries.by_id.contains_key(&100));
        assert_eq!(next_id, 100);
    }

    #[test]
    fn rejects_ranges_outside_flow() {
        let mut score = score_with_flow();
        let mut next_id = 100;

        assert!(score.insert_flow_time(1, 257, 1, &mut next_id).is_err());
        assert!(score.delete_flow_time(1, 240, 32).is_err());
        assert_eq!(score.flows.by_id[&1].length, 256);
    }

    #[test]
    fn resolves_bar_ranges_and_duration_across_time_signature_change() {
        let mut score = score_with_flow();
        score.flows.by_id.get_mut(&1).unwrap().length = 352;
        insert_time_signature(&mut score, 70, 0, 4);
        insert_time_signature(&mut score, 71, 256, 3);

        assert_eq!(score.bar_range(1, 128), Ok((128, 256)));
        assert_eq!(score.bar_range(1, 256), Ok((256, 352)));
        assert_eq!(score.bar_duration(1, 256), Ok(96));
        assert!(score.bar_range(1, 64).is_err());
    }

    #[test]
    fn insert_bar_after_uses_selected_bar_time_signature_and_returns_new_tick() {
        let mut score = score_with_flow();
        score.flows.by_id.get_mut(&1).unwrap().length = 352;
        insert_time_signature(&mut score, 70, 0, 4);
        insert_time_signature(&mut score, 71, 256, 3);
        let mut next_id = 100;

        let selected_tick = score.insert_bar_after(1, 256, &mut next_id).unwrap();

        assert_eq!(selected_tick, 352);
        assert_eq!(score.flows.by_id[&1].length, 448);
    }

    #[test]
    fn delete_middle_and_last_bars_return_expected_selection_tick() {
        let mut middle = score_with_flow();
        insert_time_signature(&mut middle, 70, 0, 4);
        assert_eq!(middle.delete_bar(1, 0), Ok(0));
        assert_eq!(middle.flows.by_id[&1].length, 128);

        let mut last = score_with_flow();
        insert_time_signature(&mut last, 70, 0, 4);
        assert_eq!(last.delete_bar(1, 128), Ok(0));
        assert_eq!(last.flows.by_id[&1].length, 128);
    }

    #[test]
    fn delete_bar_rejects_removing_the_only_bar() {
        let mut score = score_with_flow();
        score.flows.by_id.get_mut(&1).unwrap().length = 128;
        insert_time_signature(&mut score, 70, 0, 4);

        let error = score.delete_bar(1, 0).unwrap_err();

        assert_eq!(error, "A flow must contain at least one bar");
        assert_eq!(score.flows.by_id[&1].length, 128);
    }
}
