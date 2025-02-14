use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::components::{duration::Tick, measurements::BoundingBox};
use crate::score::flows::Flow;
use crate::score::tracks::{Track, Tracks};
use crate::utils::UniqueId;
use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

use super::Entry;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ClefDrawType {
    Hidden,
    G,
    F,
    C,
    Percussion,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Clef {
    pub id: UniqueId,
    pub tick: Tick,
    pub draw_as: ClefDrawType,
    pub pitch: Pitch, // the pitch that the clef represents (treble => G, Alto => C etc.)
    pub offset: i8,   // visual offset from middle stave line
}

impl Clef {
    pub fn new(id: UniqueId, tick: Tick, pitch: Pitch, offset: i8, draw_as: ClefDrawType) -> Self {
        Self {
            id,
            tick,
            draw_as,
            pitch,
            offset,
        }
    }

    pub fn metrics(&self, is_full_size: bool) -> BoundingBox {
        match self.draw_as {
            ClefDrawType::Hidden => BoundingBox {
                width: 0.0,
                height: 4.0,
            },
            _ => BoundingBox {
                width: if is_full_size { 3.8 } else { 2.75 },
                height: if is_full_size { 4.0 } else { 2.5 },
            },
        }
    }

    pub fn glyph(&self) -> Option<String> {
        match self.draw_as {
            ClefDrawType::Hidden => None,
            ClefDrawType::G => Some(String::from("\u{E050}")),
            ClefDrawType::F => Some(String::from("\u{E062}")),
            ClefDrawType::C => Some(String::from("\u{E05C}")),
            ClefDrawType::Percussion => Some(String::from("\u{E069}")),
        }
    }
}

impl Default for Clef {
    fn default() -> Self {
        Clef::new(
            0,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::C,
        )
    }
}

impl Track {
    /// Returns the time signature entry at a given tick if it exists
    pub fn get_clef_at_tick(&self, at: &Tick) -> Option<&Clef> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::Clef(time_signature)) = self.entries.by_id.get(key) {
                return Some(time_signature);
            }
        }

        None
    }
}

impl Flow {
    pub fn has_clef_at_tick(&self, tick: &Tick, tracks: &Tracks) -> bool {
        self.staves.by_id.values().any(|stave| {
            let track = &tracks.by_id[&stave.master_id];
            track.get_clef_at_tick(tick).is_some()
        })
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_clef_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    tick: Tick,
    pitch: u8,
    offset: i8,
    clef_draw_type: ClefDrawType,
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let clef_id = state.generate_id();

    let flow = &state.score.flows.by_id[&flow_id];
    let stave = &flow.staves.by_id[&stave_id];
    let master_id = stave.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    // remove old clef if defined
    if let Some(old) = master.get_clef_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    // insert the new clef
    let new = Clef::new(
        clef_id,
        tick,
        Pitch::new(pitch, Accidental::Natural),
        offset,
        clef_draw_type,
    );
    master.insert_entry(Entry::Clef(new));

    state.emit(&app);

    clef_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_clef_clear(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    tick: Tick,
) {
    if tick == 0 {
        return;
    }

    let mut state = state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let stave = &flow.staves.by_id[&stave_id];
    let master_id = stave.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    if let Some(old) = master.get_clef_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    state.emit(&app);
}
