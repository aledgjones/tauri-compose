use super::Entry;
use crate::score::components::duration::Tick;
use crate::score::components::measurements::BoundingBox;
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::AppState;
use crate::RenderState;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use tauri::State;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BarlineDrawType {
    Single,
    Double,
    Triple,
    Dashed,
    Final,
    StartRepeat,
    EndRepeat,
    EndStartRepeat,
}

impl BarlineDrawType {
    pub fn metrics(&self) -> BoundingBox {
        match self {
            BarlineDrawType::Single => BoundingBox {
                width: 0.0,
                height: 4.0,
                right_padding: 1.0,
            },
            BarlineDrawType::Double => BoundingBox {
                width: 0.5,
                height: 4.0,
                right_padding: 1.0,
            },
            BarlineDrawType::Triple => BoundingBox {
                width: 1.0,
                height: 4.0,
                right_padding: 1.0,
            },
            BarlineDrawType::Dashed => BoundingBox {
                width: 0.0,
                height: 4.0,
                right_padding: 1.0,
            },
            BarlineDrawType::Final => BoundingBox {
                width: 1.0,
                height: 4.0,
                right_padding: 0.0,
            },
            BarlineDrawType::StartRepeat => BoundingBox {
                width: 2.0,
                height: 4.0,
                right_padding: 1.0,
            },
            BarlineDrawType::EndRepeat => BoundingBox {
                width: 2.0,
                height: 4.0,
                right_padding: 1.0,
            },
            BarlineDrawType::EndStartRepeat => BoundingBox {
                width: 3.5,
                height: 4.0,
                right_padding: 1.0,
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Barline {
    pub id: UniqueId,
    pub tick: Tick,
    pub barline_draw_type: BarlineDrawType,
}

impl Barline {
    pub fn new(id: UniqueId, tick: Tick, barline_draw_type: BarlineDrawType) -> Self {
        Self {
            id,
            tick,
            barline_draw_type,
        }
    }
}

impl Track {
    /// Returns the barline entry at a given tick if it exists
    pub fn get_barline_at_tick(&self, at: &Tick) -> Option<&Barline> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::Barline(barline)) = self.entries.by_id.get(key) {
                return Some(barline);
            }
        }

        None
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_barline_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    tick: Tick,
    barline_draw_type: BarlineDrawType,
) -> Option<UniqueId> {
    if tick == 0 && barline_draw_type != BarlineDrawType::StartRepeat {
        return None;
    }

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let barline_id = state.generate_id();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    // remove old barline if defined
    if let Some(old) = master.get_barline_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    // insert the new barline
    let new = Barline::new(barline_id, tick, barline_draw_type);
    master.insert_entry(Entry::Barline(new));

    state.emit(&app, &mut renderer, true);

    Some(barline_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_barline_clear(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    tick: Tick,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    if let Some(old) = master.get_barline_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    state.emit(&app, &mut renderer, true);
}
