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
pub fn score_entries_tone_slice(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
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
        original_tone.articulation,
    );

    track.insert_entry(Entry::Tone(new_tone));

    state.emit(&app, &mut renderer, true);

    Some(new_tone_id)
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)]
pub fn score_entries_tone_pitch_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    pitch_int: u8,
    accidental: Option<Accidental>,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();

    // Get the tick first to avoid borrowing conflicts
    let tick = if let Some(Entry::Tone(tone)) = track.entries.by_id.get(&tone_id) {
        tone.tick
    } else {
        return; // Tone not found
    };

    // Create the pitch based on whether an accidental was provided
    let pitch = match accidental {
        Some(accidental) => Pitch::new(pitch_int, accidental),
        None => {
            let key_signature = match track.get_key_signature_before_tick_inclusive(tick) {
                Some(key_sig) => key_sig.clone(),
                None => KeySignature::default(),
            };
            Pitch::from_int(pitch_int, &key_signature)
        }
    };

    // Now update the tone with the new pitch
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    if let Entry::Tone(tone) = entry {
        tone.pitch = pitch;
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

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_set_position_and_duration(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
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
pub fn score_entries_shift(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
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
pub fn score_entries_tone_pitch_shift_by_steps(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    track_id: UniqueId,
    tone_id: UniqueId,
    steps: i8,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = match state.score.flows.by_id.get(&flow_id) {
        Some(flow) => flow,
        None => return,
    };

    let master_track = state.score.tracks.by_id.get(&flow.master_id).unwrap();
    let track = state.score.tracks.by_id.get(&track_id).unwrap();

    let tone = track.entries.by_id.get(&tone_id);
    let tone = match tone {
        Some(Entry::Tone(tone)) => tone,
        _ => return,
    };

    let key_signature = match master_track.get_key_signature_before_tick_inclusive(tone.tick) {
        Some(key_sig) => key_sig.clone(),
        None => KeySignature::default(),
    };

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    if let Entry::Tone(tone) = entry {
        tone.pitch = tone.pitch.shift_by_steps(steps, &key_signature);
    }

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_pitch_shift_by_octaves(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    octaves: i8,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    if let Entry::Tone(tone) = entry {
        tone.pitch = tone.pitch.shift_by_octaves(octaves);
    }

    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tone_accidental_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    track_id: UniqueId,
    tone_id: UniqueId,
    accidental: Accidental,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let track = state.score.tracks.by_id.get_mut(&track_id).unwrap();
    let entry = track.entries.by_id.get_mut(&tone_id).unwrap();
    if let Entry::Tone(tone) = entry {
        let base_pitch = tone.pitch.base();
        let new_pitch_int = (base_pitch as i8 + accidental as i8) as u8;
        tone.pitch = Pitch::new(new_pitch_int, accidental);
    }

    state.emit(&app, &mut renderer, true);
}
