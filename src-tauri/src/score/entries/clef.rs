use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::components::{duration::Tick, measurements::BoundingBox};
use crate::score::flows::Flow;
use crate::score::tracks::{Track, Tracks};
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;
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
                right_padding: 0.0,
            },
            _ => BoundingBox {
                width: if is_full_size { 2.8 } else { 1.75 },
                height: if is_full_size { 4.0 } else { 2.5 },
                right_padding: 1.0,
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

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_clef_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    tick: Tick,
    pitch: u8,
    offset: i8,
    clef_draw_type: ClefDrawType,
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

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

    state.emit(&app, &mut renderer, true);

    clef_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_clef_clear(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    tick: Tick,
) {
    if tick == 0 {
        return;
    }

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let stave = &flow.staves.by_id[&stave_id];
    let master_id = stave.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    if let Some(old) = master.get_clef_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    state.emit(&app, &mut renderer, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::pitch::{Accidental, Pitch};

    fn g_clef() -> Clef {
        Clef::new(
            1,
            0,
            Pitch::new(67, Accidental::Natural),
            2,
            ClefDrawType::G,
        )
    }

    fn hidden_clef() -> Clef {
        Clef::new(
            2,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::Hidden,
        )
    }

    #[test]
    fn clef_new_stores_fields() {
        let c = g_clef();
        assert_eq!(c.id, 1);
        assert_eq!(c.offset, 2);
    }

    #[test]
    fn full_size_g_clef_metrics_width() {
        let bb = g_clef().metrics(true);
        assert!((bb.width - 2.8).abs() < f32::EPSILON);
        assert!((bb.right_padding - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn small_g_clef_metrics_width() {
        let bb = g_clef().metrics(false);
        assert!((bb.width - 1.75).abs() < f32::EPSILON);
        assert!((bb.height - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn hidden_clef_metrics_has_zero_width_and_padding() {
        let bb = hidden_clef().metrics(true);
        assert!((bb.width - 0.0).abs() < f32::EPSILON);
        assert!((bb.right_padding - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn g_clef_glyph_is_some() {
        assert_eq!(g_clef().glyph(), Some(String::from("\u{E050}")));
    }

    #[test]
    fn hidden_clef_glyph_is_none() {
        assert_eq!(hidden_clef().glyph(), None);
    }

    #[test]
    fn default_clef_is_c_at_middle() {
        let c = Clef::default();
        assert_eq!(c.id, 0);
        assert_eq!(c.offset, 0);
        assert!(c.draw_as == ClefDrawType::C);
    }

    #[test]
    fn f_clef_glyph() {
        let c = Clef::new(1, 0, Pitch::new(53, Accidental::Natural), -2, ClefDrawType::F);
        assert_eq!(c.glyph(), Some(String::from("\u{E062}")));
    }

    #[test]
    fn c_clef_glyph() {
        let c = Clef::new(1, 0, Pitch::new(60, Accidental::Natural), 0, ClefDrawType::C);
        assert_eq!(c.glyph(), Some(String::from("\u{E05C}")));
    }

    #[test]
    fn percussion_clef_glyph() {
        let c = Clef::new(
            1,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::Percussion,
        );
        assert_eq!(c.glyph(), Some(String::from("\u{E069}")));
    }

    #[test]
    fn get_clef_at_tick_returns_none_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        assert!(track.get_clef_at_tick(&0).is_none());
    }

    #[test]
    fn get_clef_at_tick_returns_clef_when_present() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let clef = Clef::new(10, 0, Pitch::new(67, Accidental::Natural), 2, ClefDrawType::G);
        track.insert_entry(Entry::Clef(clef));
        let found = track.get_clef_at_tick(&0).unwrap();
        assert_eq!(found.id, 10);
    }

    #[test]
    fn get_clef_at_tick_ignores_different_tick() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let clef = Clef::new(10, 64, Pitch::new(67, Accidental::Natural), 2, ClefDrawType::G);
        track.insert_entry(Entry::Clef(clef));
        assert!(track.get_clef_at_tick(&0).is_none());
    }

    #[test]
    fn small_hidden_clef_also_zero_width() {
        let bb = hidden_clef().metrics(false);
        assert!((bb.width - 0.0).abs() < f32::EPSILON);
        assert!((bb.right_padding - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn g_clef_tick_set_on_new() {
        let c = g_clef();
        assert_eq!(c.tick, 0);
    }
}
