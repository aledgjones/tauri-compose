use super::key_signature::KeySignature;
use super::Entry;
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
    pub fn resolve_pitch(&self, tick: Tick, pitch_int: u8, accidental: Option<Accidental>) -> Pitch {
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

    /// Set the duration of the tone identified by `tone_id`.
    pub fn set_tone_duration(&mut self, tone_id: UniqueId, duration: Ticks) {
        if let Some(Entry::Tone(tone)) = self.entries.by_id.get_mut(&tone_id) {
            tone.duration = duration;
        }
    }

    /// Set the pitch of the tone identified by `tone_id`, resolving via key signature if needed.
    pub fn set_tone_pitch(&mut self, tone_id: UniqueId, pitch_int: u8, accidental: Option<Accidental>) {
        let tick = if let Some(Entry::Tone(tone)) = self.entries.by_id.get(&tone_id) {
            tone.tick
        } else {
            return;
        };
        let pitch = self.resolve_pitch(tick, pitch_int, accidental);
        if let Some(Entry::Tone(tone)) = self.entries.by_id.get_mut(&tone_id) {
            tone.pitch = pitch;
        }
    }

    /// Shift the pitch of `tone_id` by a number of octaves. Returns new MIDI int if found.
    pub fn shift_tone_pitch_by_octaves(&mut self, tone_id: UniqueId, octaves: i8) -> Option<u8> {
        if let Some(Entry::Tone(tone)) = self.entries.by_id.get_mut(&tone_id) {
            tone.pitch = tone.pitch.shift_by_octaves(octaves);
            Some(tone.pitch.int)
        } else {
            None
        }
    }

    /// Set the accidental on `tone_id`, recomputing pitch from base + accidental offset. Returns new MIDI int if found.
    pub fn set_tone_accidental(&mut self, tone_id: UniqueId, accidental: Accidental) -> Option<u8> {
        if let Some(Entry::Tone(tone)) = self.entries.by_id.get_mut(&tone_id) {
            let base_pitch = tone.pitch.base();
            let new_pitch_int = (base_pitch as i8 + accidental as i8) as u8;
            tone.pitch = Pitch::new(new_pitch_int, accidental);
            Some(tone.pitch.int)
        } else {
            None
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
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let tone_id = state.generate_id();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();

    let pitch = match accidental {
        Some(accidental) => Pitch::new(pitch_int, accidental),
        None => {
            let key_signature = match track.get_key_signature_before_tick_inclusive(tick) {
                Some(key_signature) => key_signature,
                None => &KeySignature::default(),
            };
            Pitch::from_int(pitch_int, key_signature)
        }
    };

    // Determine the effective articulation set for the new tone.
    // - If caller provided a non-empty set => use it and apply it to all tones at the tick.
    // - If caller provided an empty set => inherit the first-existing tone's set at that tick
    //   (if any); otherwise remain empty. Do NOT modify existing tones when caller's set is empty.
    let effective_articulations: Articulations = if !articulations.is_empty() {
        articulations.clone()
    } else {
        track.first_articulations_at_tick(&tick).unwrap_or_default()
    };

    track.insert_entry(Entry::Tone(Tone::new(
        tone_id,
        tick,
        duration,
        pitch,
        Velocity::new(velocity),
        effective_articulations.clone(),
    )));

    // If the caller supplied a non-empty set, apply it to all tones at the tick.
    if !articulations.is_empty() {
        track.apply_articulations_to_tick(tick, &articulations);
    }

    state.emit(&app, &mut renderer, true);

    tone_id
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

    state.emit(&app, &mut renderer, true);
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
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    let original_tone = match entry {
        Entry::Tone(tone) => tone.clone(),
        _ => return None,
    };

    let tone_start = original_tone.tick;
    let tone_end = original_tone.tick + original_tone.duration;

    if slice_tick <= tone_start || slice_tick >= tone_end {
        return None;
    }

    // Shorten the original tone
    if let Entry::Tone(tone) = entry {
        tone.duration = slice_tick - tone_start;
    }

    // Create new tone for the remainder
    let new_duration = tone_end - slice_tick;
    let new_tone = Tone::new(
        new_tone_id,
        slice_tick,
        new_duration,
        original_tone.pitch,
        original_tone.velocity,
        original_tone.articulations.clone(),
    );

    track.insert_entry(Entry::Tone(new_tone));

    state.emit(&app, &mut renderer, true);

    Some(new_tone_id)
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
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.set_tone_pitch(tone_id, pitch_int, accidental);

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_duration_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    track_id: UniqueId,
    tone_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.remove_entry(tone_id);

    state.emit(&app, &mut renderer, true);
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
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    if let Entry::Tone(tone) = entry {
        // Remove from old tick index
        if let Some(ids) = track.entries.by_tick.get_mut(&tone.tick) {
            ids.retain(|id| *id != tone_id);
        }

        // Update tone properties
        tone.tick = tick;
        tone.duration = duration;

        // Add to new tick index
        let tick_vec = track.entries.by_tick.entry(tick).or_default();
        tick_vec.push(tone_id);
    }

    state.emit(&app, &mut renderer, true);
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

    state.emit(&app, &mut renderer, true);
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

    let (moved_tone_articulations, moved_tick) = match track.entries.by_id.get(&entry_id) {
        Some(Entry::Tone(tone)) => (tone.articulations.clone(), tone.tick),
        _ => return,
    };

    if !moved_tone_articulations.is_empty() {
        track.apply_articulations_to_tick(moved_tick, &moved_tone_articulations);
    } else if let Some(found_set) = track.first_articulations_at_tick(&moved_tick) {
        track.apply_articulations_to_tick(moved_tick, &found_set);
    }

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_pitch_shift_by_steps(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    track_id: UniqueId,
    tone_id: UniqueId,
    steps: i8,
) -> Option<u8> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = state.score.flows.by_id.get(&flow_id)?;

    let master_track = state.score.tracks.by_id.get(&flow.master_id).unwrap();
    let track = state.score.tracks.by_id.get(&track_id).unwrap();

    let tone = track.entries.by_id.get(&tone_id);
    let tone = match tone {
        Some(Entry::Tone(tone)) => tone,
        _ => return None,
    };

    let key_signature = match master_track.get_key_signature_before_tick_inclusive(tone.tick) {
        Some(key_sig) => key_sig.clone(),
        None => KeySignature::default(),
    };

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    let new_pitch = if let Entry::Tone(tone) = entry {
        tone.pitch = tone.pitch.shift_by_steps(steps, &key_signature);
        Some(tone.pitch.int)
    } else {
        None
    };

    state.emit(&app, &mut renderer, true);

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
) -> Option<u8> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let new_pitch = track.shift_tone_pitch_by_octaves(tone_id, octaves);

    state.emit(&app, &mut renderer, true);
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
) -> Option<u8> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let new_pitch = track.set_tone_accidental(tone_id, accidental);

    state.emit(&app, &mut renderer, true);

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
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();

    // Use the selected tone as the guide: compute the resulting articulation set once,
    // then apply that set uniformly to all tones at the tick.
    let (tick, selected_set) = if let Entry::Tone(tone) = entry {
        (tone.tick, tone.articulations.clone())
    } else {
        return;
    };

    let resulting_set = selected_set.toggle(articulation);

    track.apply_articulations_to_tick(tick, &resulting_set);

    state.emit(&app, &mut renderer, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::{
        articulation::Articulations,
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

    fn make_track_with_tone(tone_id: u64, tick: u64, pitch_int: u8, accidental: Accidental) -> Track {
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
        track.set_tone_pitch(1, 62, Some(Accidental::Sharp));
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
        track.set_tone_pitch(1, 66, None);
        if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&1) {
            assert_eq!(tone.pitch.accidental, Accidental::Sharp);
        } else {
            panic!("tone not found");
        }
    }

    // --- Track::shift_tone_pitch_by_octaves ---

    #[test]
    fn shift_tone_pitch_by_octaves_up_one() {
        use crate::score::entries::Entry;
        // C4 = 60; up 1 octave = 72
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.shift_tone_pitch_by_octaves(1, 1);
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
        let result = track.shift_tone_pitch_by_octaves(1, -1);
        assert_eq!(result, Some(60));
    }

    #[test]
    fn shift_tone_pitch_by_octaves_returns_none_for_missing_tone() {
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.shift_tone_pitch_by_octaves(999, 1);
        assert_eq!(result, None);
    }

    // --- Track::set_tone_accidental ---

    #[test]
    fn set_tone_accidental_updates_pitch() {
        use crate::score::entries::Entry;
        // C natural (int=60, accidental=Natural, base=60) + Sharp(+1) = 61
        let mut track = make_track_with_tone(1, 0, 60, Accidental::Natural);
        let result = track.set_tone_accidental(1, Accidental::Sharp);
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
        let result = track.set_tone_accidental(999, Accidental::Sharp);
        assert_eq!(result, None);
    }
}
