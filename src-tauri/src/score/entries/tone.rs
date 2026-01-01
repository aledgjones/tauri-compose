use super::key_signature::KeySignature;
use super::Entry;
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::{Tick, Ticks};
use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::components::velocity::Velocity;
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::AppState;
use crate::RenderState;
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
    pub articulation: Articulation,
}

impl Tone {
    pub fn new(
        id: UniqueId,
        tick: Tick,
        duration: Ticks,
        pitch: Pitch,
        velocity: Velocity,
        articulation: Articulation,
    ) -> Self {
        Self {
            id,
            tick,
            duration,
            pitch,
            velocity,
            articulation,
        }
    }
}

impl Track {
    /// Returns the time signature entry at a given tick if it exists
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
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)]
pub fn score_entries_tone_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    track_id: UniqueId,
    tick: Tick,
    duration: Ticks,
    pitch_int: u8,
    accidental: Option<Accidental>,
    velocity: u8,
    articulation: Articulation,
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

    track.insert_entry(Entry::Tone(Tone::new(
        tone_id,
        tick,
        duration,
        pitch,
        Velocity::new(velocity),
        articulation,
    )));

    state.emit(&app, &mut renderer, true);

    tone_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_duration_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    duration: Ticks,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    if let Entry::Tone(tone) = entry {
        tone.duration = duration;
    }

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_duration_remove(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    track_id: UniqueId,
    tone_id: UniqueId,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    track.remove_entry(tone_id);

    state.emit(&app, &mut renderer, true);
}
