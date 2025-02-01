use super::key_signature::{KeySignature, KeySignatureMode};
use super::Entry;
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::{Tick, Ticks};
use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::components::velocity::Velocity;
use crate::utils::UniqueId;
use crate::AppState;
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

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    track_id: UniqueId,
    tick: Tick,
    duration: Ticks,
    pitch_int: u8,
    accidental: Option<Accidental>,
    velocity: u8,
    articulation: Articulation,
) -> UniqueId {
    let mut state = state.lock().unwrap();

    let tone_id = state.generate_id();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();

    let pitch = match accidental {
        Some(accidental) => Pitch::new(pitch_int, accidental),
        None => {
            // default to C Major if no explicit key
            let key_signature = match track.get_key_signature_before_tick_inclusive(tick) {
                Some(key_signature) => key_signature,
                None => &KeySignature::new(0, 0, KeySignatureMode::Major, 0),
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

    state.emit(&app);

    tone_id
}
