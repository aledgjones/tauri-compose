use super::key_signature::KeySignature;
use super::{tuplet::TupletEntry, Entry};
use crate::score::components::articulation::{Articulation, Articulations};
use crate::score::components::duration::{Tick, Ticks};
use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::components::velocity::Velocity;
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;

use serde::{Deserialize, Serialize};
use tauri::State;

/// These represent the audiable tones of the music.
/// They are never directly drawn in the score.
#[derive(Clone, Serialize, Deserialize)]
pub struct Tone {
    pub id: UniqueId,
    pub tick: Tick,
    pub duration: Ticks,
    pub pitch: Pitch,
    pub velocity: Velocity,
    pub articulations: Articulations,
}

impl Tone {
    pub fn new(
        id: UniqueId,
        tick: Tick,
        duration: Ticks,
        pitch: Pitch,
        velocity: Velocity,
        articulations: Articulations,
    ) -> Self {
        Self {
            id,
            tick,
            duration,
            pitch,
            velocity,
            articulations,
        }
    }
}

impl Track {
    /// Returns tone entry at a given tick if it exists
    pub fn get_tones_at_tick(&self, tick: &Tick) -> Vec<&Tone> {
        let mut output = Vec::new();

        let entry_ids = match self.entries.by_tick.get(tick) {
            Some(entries) => entries,
            None => return output,
        };

        for entry_id in entry_ids {
            if let Some(Entry::Tone(tone)) = self.entries.by_id.get(entry_id) {
                output.push(tone);
            }
        }

        output
    }

    /// Return the first non-empty articulation *set* present at `tick`, if any.
    pub fn first_articulations_at_tick(&self, tick: &Tick) -> Option<Articulations> {
        if let Some(tone_ids) = self.entries.by_tick.get(tick) {
            for tone_id in tone_ids {
                if let Some(Entry::Tone(tone)) = self.entries.by_id.get(tone_id) {
                    if !tone.articulations.is_empty() {
                        return Some(tone.articulations.clone());
                    }
                }
            }
        }
        None
    }

    /// Replace the articulations set for all Tone entries at `tick`.
    pub fn apply_articulations_to_tick(&mut self, tick: Tick, articulations: &Articulations) {
        if let Some(tone_ids) = self.entries.by_tick.get(&tick) {
            for tone_id in tone_ids {
                if let Some(Entry::Tone(tone)) = self.entries.by_id.get_mut(tone_id) {
                    tone.articulations = articulations.clone();
                }
            }
        }
    }

    /// Resolve a pitch from a MIDI integer + optional explicit accidental.
    /// If no accidental is provided, infers from the key signature at `tick`.
    pub fn resolve_pitch(
        &self,
        tick: Tick,
        pitch_int: u8,
        accidental: Option<Accidental>,
    ) -> Pitch {
        match accidental {
            Some(acc) => Pitch::new(pitch_int, acc),
            None => {
                let key_sig = match self.get_key_signature_before_tick_inclusive(tick) {
                    Some(ks) => ks.clone(),
                    None => KeySignature::default(),
                };
                Pitch::from_int(pitch_int, &key_sig)
            }
        }
    }

    fn get_tone_mut(&mut self, tone_id: UniqueId, tuplet_id: Option<UniqueId>) -> Option<&mut Tone> {
        match tuplet_id {
            Some(tuplet_id) => match self.entries.by_id.get_mut(&tuplet_id)? {
                Entry::Tuplet(tuplet) => match tuplet.entries.by_id.get_mut(&tone_id)? {
                    TupletEntry::Tone(tone) => Some(tone),
                },
                _ => None,
            },
            None => match self.entries.by_id.get_mut(&tone_id)? {
                Entry::Tone(tone) => Some(tone),
                _ => None,
            },
        }
    }

    fn resolve_tone_pitch_tick(
        &self,
        tone_id: UniqueId,
        tuplet_id: Option<UniqueId>,
    ) -> Option<Tick> {
        match tuplet_id {
            Some(tuplet_id) => match self.entries.by_id.get(&tuplet_id)? {
                Entry::Tuplet(tuplet) => match tuplet.entries.by_id.get(&tone_id)? {
                    TupletEntry::Tone(tone) => {
                        self.resolve_tuplet_pitch_tick(tone.tick, tuplet_id).ok()
                    }
                },
                _ => None,
            },
            None => match self.entries.by_id.get(&tone_id)? {
                Entry::Tone(tone) => Some(tone.tick),
                _ => None,
            },
        }
    }

    /// Set the duration of the tone identified by `tone_id`.
    pub fn set_tone_duration(&mut self, tone_id: UniqueId, duration: Ticks) {
        if let Some(Entry::Tone(tone)) = self.entries.by_id.get_mut(&tone_id) {
            tone.duration = duration;
        }
    }

    /// Set the pitch of the tone identified by `tone_id`, resolving via key signature if needed.
    pub fn set_tone_pitch(
        &mut self,
        tone_id: UniqueId,
        pitch_int: u8,
        accidental: Option<Accidental>,
        tuplet_id: Option<UniqueId>,
    ) {
        let Some(tick) = self.resolve_tone_pitch_tick(tone_id, tuplet_id) else {
            return;
        };
        let pitch = self.resolve_pitch(tick, pitch_int, accidental);
        if let Some(tone) = self.get_tone_mut(tone_id, tuplet_id) {
            tone.pitch = pitch;
        }
    }

    /// Shift the pitch of `tone_id` by a number of octaves. Returns new MIDI int if found.
    pub fn shift_tone_pitch_by_octaves(
        &mut self,
        tone_id: UniqueId,
        octaves: i8,
        tuplet_id: Option<UniqueId>,
    ) -> Option<u8> {
        if let Some(tone) = self.get_tone_mut(tone_id, tuplet_id) {
            tone.pitch = tone.pitch.shift_by_octaves(octaves);
            Some(tone.pitch.int)
        } else {
            None
        }
    }

    /// Set the accidental on `tone_id`, recomputing pitch from base + accidental offset. Returns new MIDI int if found.
    pub fn set_tone_accidental(
        &mut self,
        tone_id: UniqueId,
        accidental: Accidental,
        tuplet_id: Option<UniqueId>,
    ) -> Option<u8> {
        if let Some(tone) = self.get_tone_mut(tone_id, tuplet_id) {
            let base_pitch = tone.pitch.base();
            let new_pitch_int = (base_pitch as i8 + accidental as i8) as u8;
            tone.pitch = Pitch::new(new_pitch_int, accidental);
            Some(tone.pitch.int)
        } else {
            None
        }
    }

    pub fn remove_tone(&mut self, tone_id: UniqueId, tuplet_id: Option<UniqueId>) -> bool {
        match tuplet_id {
            Some(tuplet_id) => match self.entries.by_id.get_mut(&tuplet_id) {
                Some(Entry::Tuplet(tuplet)) => tuplet.entries.remove_entry(tone_id).is_some(),
                _ => false,
            },
            None => self.remove_entry(tone_id).is_some(),
        }
    }

    fn resolve_tuplet_pitch_tick(
        &self,
        local_tick: Tick,
        tuplet_id: UniqueId,
    ) -> Result<Tick, String> {
        let track_id = self.id;
        let tuplet = self
            .entries
            .by_id
            .get(&tuplet_id)
            .ok_or_else(|| format!("Tuplet {tuplet_id} not found on track {track_id}"))?;

        let Entry::Tuplet(tuplet) = tuplet else {
            return Err(format!("Tuplet {tuplet_id} not found on track {track_id}"));
        };

        let local_length = tuplet.local_length_ticks();

        if local_length == 0 {
            return Ok(tuplet.tick);
        }

        Ok(tuplet.tick + ((local_tick * tuplet.consumed_ticks()) / local_length))
    }

    /// Create a tone with pitch resolution and articulation inheritance, then insert it.
    #[allow(clippy::too_many_arguments)]
    pub fn create_tone(
        &mut self,
        tone_id: UniqueId,
        tick: Tick,
        duration: Ticks,
        pitch_int: u8,
        accidental: Option<Accidental>,
        velocity: u8,
        articulations: Articulations,
        tuplet_id: Option<UniqueId>,
    ) -> Result<UniqueId, String> {
        let track_id = self.id;
        let pitch_tick = match tuplet_id {
            Some(tuplet_id) => self.resolve_tuplet_pitch_tick(tick, tuplet_id)?,
            None => tick,
        };
        let pitch = self.resolve_pitch(pitch_tick, pitch_int, accidental);

        match tuplet_id {
            Some(tuplet_id) => {
                let tuplet = self
                    .entries
                    .by_id
                    .get_mut(&tuplet_id)
                    .ok_or_else(|| format!("Tuplet {tuplet_id} not found on track {track_id}"))?;

                let Entry::Tuplet(tuplet) = tuplet else {
                    return Err(format!("Tuplet {tuplet_id} not found on track {track_id}"));
                };

                let effective_articulations = if !articulations.is_empty() {
                    articulations.clone()
                } else {
                    tuplet
                        .entries
                        .first_articulations_at_tick(&tick)
                        .unwrap_or_default()
                };

                tuplet.entries.insert_entry(TupletEntry::Tone(Tone::new(
                    tone_id,
                    tick,
                    duration,
                    pitch,
                    Velocity::new(velocity),
                    effective_articulations,
                )));

                if !articulations.is_empty() {
                    tuplet.entries.apply_articulations_to_tick(tick, &articulations);
                }

                Ok(tone_id)
            }
            None => {
                let effective_articulations = if !articulations.is_empty() {
                    articulations.clone()
                } else {
                    self.first_articulations_at_tick(&tick).unwrap_or_default()
                };

                self.insert_entry(Entry::Tone(Tone::new(
                    tone_id,
                    tick,
                    duration,
                    pitch,
                    Velocity::new(velocity),
                    effective_articulations,
                )));

                if !articulations.is_empty() {
                    self.apply_articulations_to_tick(tick, &articulations);
                }

                Ok(tone_id)
            }
        }
    }

    /// Slice a tone at `slice_tick`, inserting a tail tone with `new_tone_id`.
    /// Returns `Some(new_tone_id)` on success, `None` if `slice_tick` is outside the tone's range.
    pub fn slice_tone(
        &mut self,
        tone_id: UniqueId,
        new_tone_id: UniqueId,
        slice_tick: Tick,
    ) -> Option<UniqueId> {
        let original = match self.entries.by_id.get(&tone_id) {
            Some(Entry::Tone(tone)) => tone.clone(),
            _ => return None,
        };
        let tone_start = original.tick;
        let tone_end = original.tick + original.duration;
        if slice_tick <= tone_start || slice_tick >= tone_end {
            return None;
        }
        if let Some(Entry::Tone(tone)) = self.entries.by_id.get_mut(&tone_id) {
            tone.duration = slice_tick - tone_start;
        }
        let new_tone = Tone::new(
            new_tone_id,
            slice_tick,
            tone_end - slice_tick,
            original.pitch,
            original.velocity,
            original.articulations.clone(),
        );
        self.insert_entry(Entry::Tone(new_tone));
        Some(new_tone_id)
    }

    /// Move a tone to `tick` and update its `duration`, keeping the `by_tick` index consistent.
    pub fn set_tone_position_and_duration(
        &mut self,
        tone_id: UniqueId,
        tick: Tick,
        duration: Ticks,
    ) {
        self.shift_entry(tone_id, tick);
        self.set_tone_duration(tone_id, duration);
    }

    /// Reconcile articulations at the moved entry's tick:
    /// if the entry has articulations, propagate them; else inherit from existing tones at that tick.
    pub fn unify_articulations_at_entry(&mut self, entry_id: UniqueId) {
        let (moved_articulations, moved_tick) = match self.entries.by_id.get(&entry_id) {
            Some(Entry::Tone(tone)) => (tone.articulations.clone(), tone.tick),
            _ => return,
        };
        if !moved_articulations.is_empty() {
            self.apply_articulations_to_tick(moved_tick, &moved_articulations);
        } else if let Some(found_set) = self.first_articulations_at_tick(&moved_tick) {
            self.apply_articulations_to_tick(moved_tick, &found_set);
        }
    }

    /// Shift the pitch of `tone_id` by diatonic steps using the given key signature.
    /// Returns new MIDI int if found.
    pub fn shift_tone_pitch_by_steps(
        &mut self,
        tone_id: UniqueId,
        steps: i8,
        key_signature: &KeySignature,
        tuplet_id: Option<UniqueId>,
    ) -> Option<u8> {
        if let Some(tone) = self.get_tone_mut(tone_id, tuplet_id) {
            tone.pitch = tone.pitch.shift_by_steps(steps, key_signature);
            Some(tone.pitch.int)
        } else {
            None
        }
    }

    /// Toggle an articulation on `tone_id` and apply the resulting set to all tones at that tick.
    pub fn toggle_articulation_at_tone(
        &mut self,
        tone_id: UniqueId,
        articulation: Articulation,
        tuplet_id: Option<UniqueId>,
    ) {
        match tuplet_id {
            Some(tuplet_id) => {
                let Some(Entry::Tuplet(tuplet)) = self.entries.by_id.get_mut(&tuplet_id) else {
                    return;
                };

                let (tick, resulting_set) = match tuplet.entries.by_id.get(&tone_id) {
                    Some(TupletEntry::Tone(tone)) => {
                        let set = tone.articulations.clone().toggle(articulation);
                        (tone.tick, set)
                    }
                    _ => return,
                };

                tuplet.entries.apply_articulations_to_tick(tick, &resulting_set);
            }
            None => {
                let (tick, resulting_set) = match self.entries.by_id.get(&tone_id) {
                    Some(Entry::Tone(tone)) => {
                        let set = tone.articulations.clone().toggle(articulation);
                        (tone.tick, set)
                    }
                    _ => return,
                };
                self.apply_articulations_to_tick(tick, &resulting_set);
            }
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)]
pub fn score_entries_tone_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tick: Tick,
    duration: Ticks,
    pitch_int: u8,
    accidental: Option<Accidental>,
    velocity: u8,
    articulations: Articulations,
    tuplet_id: Option<UniqueId>,
) -> Result<UniqueId, String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let tone_id = state.generate_id();
    let track = state
        .score
        .tracks
        .by_id
        .get_mut(&track_id)
        .ok_or_else(|| format!("Track {} not found", track_id))?;
    track.create_tone(
        tone_id,
        tick,
        duration,
        pitch_int,
        accidental,
        velocity,
        articulations,
        tuplet_id,
    )?;

    state.emit_affected_track(&app, &mut renderer, track_id);

    Ok(tone_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_duration_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    duration: Ticks,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.set_tone_duration(tone_id, duration);

    state.emit_affected_track(&app, &mut renderer, track_id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_slice(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    slice_tick: Tick,
) -> Option<UniqueId> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let new_tone_id = state.generate_id();
    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let result = track.slice_tone(tone_id, new_tone_id, slice_tick);

    if result.is_some() {
        state.emit_affected_track(&app, &mut renderer, track_id);
    }

    result
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)]
pub fn score_entries_tone_pitch_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    pitch_int: u8,
    accidental: Option<Accidental>,
    tuplet_id: Option<UniqueId>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.set_tone_pitch(tone_id, pitch_int, accidental, tuplet_id);

    state.emit_affected_track(&app, &mut renderer, track_id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_duration_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    tuplet_id: Option<UniqueId>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.remove_tone(tone_id, tuplet_id);

    state.ui.sequencer.selection = None;

    state.emit_affected_track(&app, &mut renderer, track_id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_set_position_and_duration(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    tick: Tick,
    duration: Ticks,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.set_tone_position_and_duration(tone_id, tick, duration);

    state.emit_affected_track(&app, &mut renderer, track_id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_shift(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    entry_id: UniqueId,
    new_tick: Tick,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();

    track.shift_entry(entry_id, new_tick);

    state.emit_affected_track(&app, &mut renderer, track_id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_unify_articulations(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    entry_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.unify_articulations_at_entry(entry_id);

    state.emit_affected_track(&app, &mut renderer, track_id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_pitch_shift_by_steps(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    steps: i8,
    tuplet_id: Option<UniqueId>,
) -> Option<u8> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow_id = state.flow_id_for_track(track_id)?;
    let flow = state.score.flows.by_id.get(&flow_id)?;
    let master_track = state.score.tracks.by_id.get(&flow.master_id).unwrap();
    let track = state.score.tracks.by_id.get(&track_id).unwrap();

    let tone_tick = track.resolve_tone_pitch_tick(tone_id, tuplet_id)?;

    let key_signature = match master_track.get_key_signature_before_tick_inclusive(tone_tick) {
        Some(key_sig) => key_sig.clone(),
        None => KeySignature::default(),
    };

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let new_pitch = track.shift_tone_pitch_by_steps(tone_id, steps, &key_signature, tuplet_id);

    state.emit_affected_flows(&app, &mut renderer, &[flow_id]);

    new_pitch
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_pitch_shift_by_octaves(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    octaves: i8,
    tuplet_id: Option<UniqueId>,
) -> Option<u8> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let new_pitch = track.shift_tone_pitch_by_octaves(tone_id, octaves, tuplet_id);

    state.emit_affected_track(&app, &mut renderer, track_id);
    new_pitch
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_accidental_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    accidental: Accidental,
    tuplet_id: Option<UniqueId>,
) -> Option<u8> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let new_pitch = track.set_tone_accidental(tone_id, accidental, tuplet_id);

    state.emit_affected_track(&app, &mut renderer, track_id);

    new_pitch
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_articulation_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    articulation: Articulation,
    tuplet_id: Option<UniqueId>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.toggle_articulation_at_tone(tone_id, articulation, tuplet_id);

    state.emit_affected_track(&app, &mut renderer, track_id);
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

    #[test]
    fn tone_new_stores_all_fields() {
        let pitch = Pitch::new(60, Accidental::Natural);
        let velocity = Velocity::new(100);
        let arts = Articulations::new();
        let tone = Tone::new(99, 128, 32, pitch, velocity, arts);
        assert_eq!(tone.id, 99);
        assert_eq!(tone.tick, 128);
        assert_eq!(tone.duration, 32);
        assert_eq!(tone.pitch.int, 60);
        assert_eq!(tone.velocity.int(), 100);
        assert!(tone.articulations.is_empty());
    }

    #[test]
    fn tone_new_with_articulation() {
        use crate::score::components::articulation::Articulation;
        let arts = Articulations::new().toggle(Articulation::Staccato);
        let tone = Tone::new(
            1,
            0,
            16,
            Pitch::new(64, Accidental::Natural),
            Velocity::new(80),
            arts,
        );
        assert!(tone.articulations.contains(&Articulation::Staccato));
    }

    // --- Track::get_tones_at_tick ---

    #[test]
    fn get_tones_at_tick_returns_empty_vec_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        let tones = track.get_tones_at_tick(&0);
        assert!(tones.is_empty());
    }

    #[test]
    fn get_tones_at_tick_returns_tone_when_present() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let tone = Tone::new(
            42,
            64,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        track.insert_entry(Entry::Tone(tone));
        let tones = track.get_tones_at_tick(&64);
        assert_eq!(tones.len(), 1);
        assert_eq!(tones[0].id, 42);
    }

    #[test]
    fn get_tones_at_tick_returns_empty_for_wrong_tick() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let tone = Tone::new(
            42,
            64,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        track.insert_entry(Entry::Tone(tone));
        assert!(track.get_tones_at_tick(&0).is_empty());
    }

    #[test]
    fn get_tones_at_tick_returns_multiple_tones() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        for id in 1..=3 {
            let tone = Tone::new(
                id,
                0,
                32,
                Pitch::new(60 + id as u8, Accidental::Natural),
                Velocity::new(80),
                Articulations::new(),
            );
            track.insert_entry(Entry::Tone(tone));
        }
        let tones = track.get_tones_at_tick(&0);
        assert_eq!(tones.len(), 3);
    }

    // --- Track::first_articulations_at_tick ---

    #[test]
    fn first_articulations_at_tick_returns_none_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        assert!(track.first_articulations_at_tick(&0).is_none());
    }

    #[test]
    fn first_articulations_at_tick_returns_none_when_tone_has_empty_arts() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let tone = Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        track.insert_entry(Entry::Tone(tone));
        assert!(track.first_articulations_at_tick(&0).is_none());
    }

    #[test]
    fn first_articulations_at_tick_returns_arts_when_present() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let arts = Articulations::new().toggle(Articulation::Staccato);
        let tone = Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            arts,
        );
        track.insert_entry(Entry::Tone(tone));
        let found = track.first_articulations_at_tick(&0);
        assert!(found.is_some());
        assert!(found.unwrap().contains(&Articulation::Staccato));
    }

    // --- Track::apply_articulations_to_tick ---

    #[test]
    fn apply_articulations_to_tick_updates_all_tones() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        for id in 1..=2u64 {
            let tone = Tone::new(
                id,
                0,
                32,
                Pitch::new(60 + id as u8, Accidental::Natural),
                Velocity::new(80),
                Articulations::new(),
            );
            track.insert_entry(Entry::Tone(tone));
        }
        let new_arts = Articulations::new().toggle(Articulation::Tenuto);
        track.apply_articulations_to_tick(0, &new_arts);
        for tone in track.get_tones_at_tick(&0) {
            assert!(tone.articulations.contains(&Articulation::Tenuto));
        }
    }

    #[test]
    fn apply_articulations_to_tick_noop_on_empty_track() {
        use crate::score::components::articulation::Articulation;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let arts = Articulations::new().toggle(Articulation::Accent);
        // Should not panic on empty
        track.apply_articulations_to_tick(0, &arts);
    }

    // --- helpers ---

    fn make_track_with_tone(
        tone_id: u64,
        tick: u64,
        pitch_int: u8,
        accidental: Accidental,
    ) -> Track {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let tone = Tone::new(
            tone_id,
            tick,
            32,
            Pitch::new(pitch_int, accidental),
            Velocity::new(80),
            Articulations::new(),
        );
        track.insert_entry(Entry::Tone(tone));
        track
    }

    // --- Track::resolve_pitch ---

    #[test]
    fn resolve_pitch_uses_explicit_accidental() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        // C4 = 60, force Sharp → should use Pitch::new(60, Sharp)
        let pitch = track.resolve_pitch(0, 60, Some(Accidental::Sharp));
        assert_eq!(pitch.int, 60);
        assert_eq!(pitch.accidental, Accidental::Sharp);
    }

    #[test]
    fn resolve_pitch_infers_from_key_sig_when_no_accidental() {
        use crate::score::entries::key_signature::{KeySignature as KS, KeySignatureMode};
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        // Insert a G-major key signature (1 sharp) at tick 0
        let key_sig = KS::new(99, 0, KeySignatureMode::Major, 1);
        track.insert_entry(Entry::KeySignature(key_sig));
        // F# (pitch_int=66) in G-major should be inferred as F#
        let pitch = track.resolve_pitch(0, 66, None);
        assert_eq!(pitch.accidental, Accidental::Sharp);
    }

    #[test]
    fn resolve_pitch_defaults_to_natural_with_no_key_sig() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        // No key sig on track — should use C-major default (all natural)
        let pitch = track.resolve_pitch(0, 60, None);
        assert_eq!(pitch.accidental, Accidental::Natural);
    }

    // --- Track::set_tone_duration ---

    #[test]
    fn set_tone_duration_updates_duration() {
        use crate::score::entries::Entry;
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        track.set_tone_duration(1, 64);
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.duration, 64);
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn set_tone_duration_noop_for_missing_tone() {
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        // Should not panic when tone_id doesn't exist
        track.set_tone_duration(999, 64);
    }

    // --- Track::set_tone_pitch ---

    #[test]
    fn set_tone_pitch_with_explicit_accidental() {
        use crate::score::entries::Entry;
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        track.set_tone_pitch(1, 62, Some(Accidental::Sharp), None);
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.pitch.int, 62);
            assert_eq!(tone.pitch.accidental, Accidental::Sharp);
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn set_tone_pitch_infers_accidental_from_key_sig() {
        use crate::score::entries::key_signature::{KeySignature as KS, KeySignatureMode};
        use crate::score::entries::Entry;
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        // Insert G-major (1#) key signature
        let key_sig = KS::new(99, 0, KeySignatureMode::Major, 1);
        track.insert_entry(Entry::KeySignature(key_sig));
        // Set pitch to F# (66) without explicit accidental — should become F#
        track.set_tone_pitch(1, 66, None, None);
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.pitch.accidental, Accidental::Sharp);
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn set_tone_pitch_updates_tuplet_owned_tone_without_changing_identity_or_timing() {
        use crate::score::entries::tuplet::TupletEntry;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(
                30,
                16,
                80,
                67,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();

        track.set_tone_pitch(30, 69, None, Some(20));

        assert!(!track.entries.by_id.contains_key(&30));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(tone)) = tuplet.entries.by_id.get(&30) else {
            panic!("expected tuplet-owned tone");
        };

        assert_eq!(tone.id, 30);
        assert_eq!(tone.tick, 16);
        assert_eq!(tone.duration, 80);
        assert_eq!(tone.pitch.int, 69);
    }

    #[test]
    fn set_tone_pitch_infers_accidental_from_key_sig_for_tuplet_owned_tone() {
        use crate::score::entries::key_signature::{KeySignature as KS, KeySignatureMode};
        use crate::score::entries::tuplet::TupletEntry;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::KeySignature(KS::new(
            99,
            48,
            KeySignatureMode::Major,
            1,
        )));
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(30, 16, 16, 66, None, 80, Articulations::new(), Some(20))
            .unwrap();

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(initial_tone)) = tuplet.entries.by_id.get(&30) else {
            panic!("expected tuplet-owned tone");
        };
        assert_eq!(initial_tone.pitch.accidental, Accidental::Sharp);

        track.set_tone_pitch(30, 66, None, Some(20));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(tone)) = tuplet.entries.by_id.get(&30) else {
            panic!("expected tuplet-owned tone");
        };

        assert_eq!(tone.pitch.accidental, Accidental::Sharp);
    }

    #[test]
    fn set_tone_pitch_preserves_host_owned_boundary_crossing_tone() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(Tone::new(
            1,
            48,
            96,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        )));
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        track.set_tone_pitch(1, 62, None, None);

        let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) else {
            panic!("expected host-owned tone");
        };

        assert_eq!(tone.id, 1);
        assert_eq!(tone.tick, 48);
        assert_eq!(tone.duration, 96);
        assert_eq!(tone.pitch.int, 62);
    }

    // --- Track::shift_tone_pitch_by_octaves ---

    #[test]
    fn shift_tone_pitch_by_octaves_up_one() {
        use crate::score::entries::Entry;
        // C4 = 60; up 1 octave = 72
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.shift_tone_pitch_by_octaves(1, 1, None);
        assert_eq!(result, Some(72));
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.pitch.int, 72);
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn shift_tone_pitch_by_octaves_down_one() {
        // C5 = 72; down 1 octave = 60
        let mut track = make_track_with_tone(1, 0, 72, Accidental::Natural);
        let result = track.shift_tone_pitch_by_octaves(1, -1, None);
        assert_eq!(result, Some(60));
    }

    #[test]
    fn shift_tone_pitch_by_octaves_returns_none_for_missing_tone() {
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.shift_tone_pitch_by_octaves(999, 1, None);
        assert_eq!(result, None);
    }

    #[test]
    fn shift_tone_pitch_by_octaves_updates_tuplet_owned_tone() {
        use crate::score::entries::tuplet::TupletEntry;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(
                30,
                16,
                16,
                60,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();

        let result = track.shift_tone_pitch_by_octaves(30, 1, Some(20));

        assert_eq!(result, Some(72));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(tone)) = tuplet.entries.by_id.get(&30) else {
            panic!("expected tuplet-owned tone");
        };

        assert_eq!(tone.pitch.int, 72);
        assert_eq!(tone.tick, 16);
        assert_eq!(tone.duration, 16);
    }

    // --- Track::set_tone_accidental ---

    #[test]
    fn set_tone_accidental_updates_pitch() {
        use crate::score::entries::Entry;
        // C natural (int=60, accidental=Natural, base=60) + Sharp(+1) = 61
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.set_tone_accidental(1, Accidental::Sharp, None);
        assert_eq!(result, Some(61));
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.pitch.accidental, Accidental::Sharp);
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn set_tone_accidental_returns_none_for_missing_tone() {
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.set_tone_accidental(999, Accidental::Sharp, None);
        assert_eq!(result, None);
    }

    #[test]
    fn set_tone_accidental_updates_tuplet_owned_tone() {
        use crate::score::entries::tuplet::TupletEntry;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(
                30,
                16,
                16,
                60,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();

        let result = track.set_tone_accidental(30, Accidental::Sharp, Some(20));

        assert_eq!(result, Some(61));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(tone)) = tuplet.entries.by_id.get(&30) else {
            panic!("expected tuplet-owned tone");
        };

        assert_eq!(tone.pitch.int, 61);
        assert_eq!(tone.pitch.accidental, Accidental::Sharp);
    }

    #[test]
    fn remove_tone_removes_tuplet_owned_tone() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(
                30,
                16,
                16,
                60,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();

        assert!(track.remove_tone(30, Some(20)));
        assert!(!track.entries.by_id.contains_key(&30));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };

        assert!(!tuplet.entries.by_id.contains_key(&30));
        assert!(tuplet.entries.by_tick.get(&16).is_none());
    }

    #[test]
    fn remove_tone_keeps_tuplet_container_and_remaining_indexes_stable() {
        use crate::score::entries::Entry;
        use crate::score::entries::tuplet::TupletEntry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(
                30,
                16,
                16,
                60,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();
        track
            .create_tone(
                31,
                32,
                16,
                64,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();

        assert!(track.remove_tone(30, Some(20)));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };

        assert!(!tuplet.entries.by_id.contains_key(&30));
        assert!(tuplet.entries.by_tick.get(&16).is_none());
        assert_eq!(tuplet.entries.by_tick.get(&32), Some(&vec![31]));

        let Some(TupletEntry::Tone(remaining_tone)) = tuplet.entries.by_id.get(&31) else {
            panic!("expected remaining tuplet-owned tone");
        };

        assert_eq!(remaining_tone.tick, 32);
        assert_eq!(remaining_tone.duration, 16);
    }

    // --- Track::create_tone ---

    #[test]
    fn create_tone_with_explicit_accidental() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        track
            .create_tone(
                1,
                0,
                32,
                62,
                Some(Accidental::Sharp),
                80,
                Articulations::new(),
                None,
            )
            .unwrap();
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.pitch.int, 62);
            assert_eq!(tone.pitch.accidental, Accidental::Sharp);
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn create_tone_inherits_articulations() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        // Insert an existing tone with Staccato at tick 0
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        if let Some(Entry::Tone(t)) = track.entries.by_id.get_mut(&1) {
            t.articulations = Articulations::new().toggle(Articulation::Staccato);
        }
        // Create a new tone at same tick with empty articulations → should inherit
        track
            .create_tone(
                2,
                0,
                32,
                64,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                None,
            )
            .unwrap();
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&2) {
            assert!(tone.articulations.contains(&Articulation::Staccato));
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn create_tone_applies_provided_articulations_to_all_at_tick() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let arts = Articulations::new().toggle(Articulation::Tenuto);
        track.create_tone(2, 0, 32, 64, Some(Accidental::Natural), 80, arts, None)
            .unwrap();
        // Both tones at tick 0 should have Tenuto
        if let Some(Entry::Tone(t)) = track.entries.by_id.get(&1) {
            assert!(t.articulations.contains(&Articulation::Tenuto));
        } else {
            panic!("tone 1 not found");
        }
    }

    #[test]
    fn create_tone_without_tuplet_context_still_inserts_into_track_entries() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);

        let result = track.create_tone(
            10,
            32,
            16,
            64,
            Some(Accidental::Natural),
            90,
            Articulations::new(),
            None,
        );

        assert_eq!(result, Ok(10));
        assert!(matches!(track.entries.by_id.get(&10), Some(Entry::Tone(_))));
    }

    #[test]
    fn create_tone_with_tuplet_context_inserts_into_tuplet_entries() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        let result = track.create_tone(
            30,
            16,
            16,
            67,
            Some(Accidental::Natural),
            80,
            Articulations::new(),
            Some(20),
        );

        assert_eq!(result, Ok(30));
        assert!(!track.entries.by_id.contains_key(&30));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };

        assert!(tuplet.entries.by_tick.get(&16).is_some_and(|ids| ids == &vec![30]));
        assert!(matches!(tuplet.entries.by_id.get(&30), Some(crate::score::entries::tuplet::TupletEntry::Tone(_))));
    }

    #[test]
    fn create_tone_allows_multiple_tuplet_tones_at_same_local_tick() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        assert_eq!(
            track.create_tone(
                30,
                16,
                16,
                67,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            ),
            Ok(30)
        );
        assert_eq!(
            track.create_tone(
                31,
                16,
                16,
                71,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            ),
            Ok(31)
        );

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };

        assert_eq!(tuplet.entries.by_tick.get(&16), Some(&vec![30, 31]));
    }

    #[test]
    fn create_tone_rejects_invalid_tuplet_context() {
        use crate::score::tracks::Track;

        let mut track = Track::new(1);

        let result = track.create_tone(
            30,
            16,
            16,
            67,
            Some(Accidental::Natural),
            80,
            Articulations::new(),
            Some(999),
        );

        assert_eq!(result, Err("Tuplet 999 not found on track 1".to_string()));
        assert!(!track.entries.by_id.contains_key(&30));
        assert!(!track.entries.by_tick.contains_key(&16));
    }

    #[test]
    fn create_tone_inside_tuplet_resolves_pitch_from_projected_host_tick() {
        use crate::score::components::pitch::Accidental;
        use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::KeySignature(KeySignature::new(
            10,
            48,
            KeySignatureMode::Major,
            1,
        )));
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();

        let result = track.create_tone(30, 16, 16, 66, None, 80, Articulations::new(), Some(20));

        assert_eq!(result, Ok(30));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(crate::score::entries::tuplet::TupletEntry::Tone(tone)) = tuplet.entries.by_id.get(&30) else {
            panic!("expected tuplet tone");
        };

        assert_eq!(tone.pitch.accidental, Accidental::Sharp);
    }

    // --- Track::slice_tone ---

    #[test]
    fn slice_tone_returns_some_and_creates_remainder() {
        use crate::score::entries::Entry;
        // make_track_with_tone gives duration=32; slice in the middle at tick 16
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.slice_tone(1, 99, 16);
        assert_eq!(result, Some(99));
        // Original shortened to 16
        if let Some(Entry::Tone(orig)) = track.entries.by_id.get(&1) {
            assert_eq!(orig.duration, 16);
        }
        // Remainder at tick 16 with duration 16
        if let Some(Entry::Tone(tail)) = track.entries.by_id.get(&99) {
            assert_eq!(tail.tick, 16);
            assert_eq!(tail.duration, 16);
        } else {
            panic!("tail tone not found");
        }
    }

    #[test]
    fn slice_tone_returns_none_at_start() {
        let mut track = make_track_with_tone(1, 0, 64, Accidental::Natural);
        assert_eq!(track.slice_tone(1, 99, 0), None);
    }

    #[test]
    fn slice_tone_returns_none_at_end() {
        let mut track = make_track_with_tone(1, 0, 64, Accidental::Natural);
        assert_eq!(track.slice_tone(1, 99, 64), None);
    }

    #[test]
    fn slice_tone_returns_none_outside_range() {
        let mut track = make_track_with_tone(1, 0, 64, Accidental::Natural);
        assert_eq!(track.slice_tone(1, 99, 100), None);
    }

    // --- Track::set_tone_position_and_duration ---

    #[test]
    fn set_tone_position_and_duration_updates_tick_and_duration() {
        use crate::score::entries::Entry;
        let mut track = make_track_with_tone(1, 0, 32, Accidental::Natural);
        track.set_tone_position_and_duration(1, 64, 48);
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.tick, 64);
            assert_eq!(tone.duration, 48);
        } else {
            panic!("tone not found");
        }
        // by_tick index updated: tone should be at tick 64, not 0
        assert!(!track
            .entries
            .by_tick
            .get(&0)
            .map(|v| v.contains(&1))
            .unwrap_or(false));
        assert!(track
            .entries
            .by_tick
            .get(&64)
            .map(|v| v.contains(&1))
            .unwrap_or(false));
    }

    // --- Track::unify_articulations_at_entry ---

    #[test]
    fn unify_articulations_propagates_from_entry_with_articulations() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let arts = Articulations::new().toggle(Articulation::Accent);
        // Two tones at tick 0: one with Accent, one without
        let tone1 = Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            arts.clone(),
        );
        let tone2 = Tone::new(
            2,
            0,
            32,
            Pitch::new(64, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        track.insert_entry(Entry::Tone(tone1));
        track.insert_entry(Entry::Tone(tone2));
        track.unify_articulations_at_entry(1);
        // Both tones should now have Accent
        if let Some(Entry::Tone(t)) = track.entries.by_id.get(&2) {
            assert!(t.articulations.contains(&Articulation::Accent));
        } else {
            panic!("tone 2 not found");
        }
    }

    // --- Track::shift_tone_pitch_by_steps ---

    #[test]
    fn shift_tone_pitch_by_steps_up_one() {
        // C natural (60) up 1 step = D (62)
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let key_sig = KeySignature::default();
        let result = track.shift_tone_pitch_by_steps(1, 1, &key_sig, None);
        assert_eq!(result, Some(62));
    }

    #[test]
    fn shift_tone_pitch_by_steps_returns_none_for_missing_tone() {
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let key_sig = KeySignature::default();
        assert_eq!(track.shift_tone_pitch_by_steps(999, 1, &key_sig, None), None);
    }

    #[test]
    fn shift_tone_pitch_by_steps_updates_tuplet_owned_tone() {
        use crate::score::entries::tuplet::TupletEntry;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(
                30,
                16,
                16,
                60,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();

        let result = track.shift_tone_pitch_by_steps(30, 1, &KeySignature::default(), Some(20));

        assert_eq!(result, Some(62));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(tone)) = tuplet.entries.by_id.get(&30) else {
            panic!("expected tuplet-owned tone");
        };

        assert_eq!(tone.pitch.int, 62);
        assert_eq!(tone.tick, 16);
        assert_eq!(tone.duration, 16);
    }

    // --- Track::toggle_articulation_at_tone ---

    #[test]
    fn toggle_articulation_adds_when_absent() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        track.toggle_articulation_at_tone(1, Articulation::Staccato, None);
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert!(tone.articulations.contains(&Articulation::Staccato));
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn toggle_articulation_removes_when_present() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let arts = Articulations::new().toggle(Articulation::Tenuto);
        let mut track = Track::new(1);
        let tone = Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            arts,
        );
        track.insert_entry(Entry::Tone(tone));
        track.toggle_articulation_at_tone(1, Articulation::Tenuto, None);
        if let Some(Entry::Tone(t)) = track.entries.by_id.get(&1) {
            assert!(!t.articulations.contains(&Articulation::Tenuto));
        } else {
            panic!("tone not found");
        }
    }

    #[test]
    fn toggle_articulation_propagates_to_all_tones_at_tick() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let tone1 = Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        let tone2 = Tone::new(
            2,
            0,
            32,
            Pitch::new(64, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        );
        track.insert_entry(Entry::Tone(tone1));
        track.insert_entry(Entry::Tone(tone2));
        track.toggle_articulation_at_tone(1, Articulation::Staccato, None);
        if let Some(Entry::Tone(t)) = track.entries.by_id.get(&2) {
            assert!(t.articulations.contains(&Articulation::Staccato));
        } else {
            panic!("tone 2 not found");
        }
    }

    #[test]
    fn toggle_articulation_propagates_within_tuplet_tick() {
        use crate::score::components::articulation::Articulation;
        use crate::score::entries::tuplet::TupletEntry;
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track
            .create_tuplet(20, 64, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(
                30,
                16,
                16,
                60,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();
        track
            .create_tone(
                31,
                16,
                16,
                64,
                Some(Accidental::Natural),
                80,
                Articulations::new(),
                Some(20),
            )
            .unwrap();

        track.toggle_articulation_at_tone(30, Articulation::Staccato, Some(20));

        let Some(Entry::Tuplet(tuplet)) = track.entries.by_id.get(&20) else {
            panic!("expected tuplet entry");
        };
        let Some(TupletEntry::Tone(tone)) = tuplet.entries.by_id.get(&31) else {
            panic!("expected tuplet-owned tone");
        };

        assert!(tone.articulations.contains(&Articulation::Staccato));
    }
}
