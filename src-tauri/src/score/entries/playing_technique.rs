use crate::renderer::converter::Space;
use crate::renderer::text_measurer::TextMeasurer;
use crate::score::components::duration::Tick;
use crate::score::components::measurements::BoundingBox;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

use super::Entry;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlayingTechniqueKind {
    Pizz,
    Arco,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayingTechnique {
    pub id: UniqueId,
    pub tick: Tick,
    pub hidden: bool,
    pub kind: PlayingTechniqueKind,
}

impl PlayingTechnique {
    pub fn new(id: UniqueId, tick: Tick, kind: PlayingTechniqueKind) -> Self {
        Self {
            id,
            tick,
            hidden: false,
            kind,
        }
    }

    pub fn text_font() -> Font {
        Font {
            size: 1.7,
            family: FontFamily::LibreBaskerville,
            justify: Justify::Start,
            align: Align::Bottom,
        }
    }

    pub fn display_text(&self) -> &'static str {
        match self.kind {
            PlayingTechniqueKind::Pizz => "pizz",
            PlayingTechniqueKind::Arco => "arco",
        }
    }

    fn text_width(&self, text_measurer: &TextMeasurer<'_>) -> Space {
        if self.hidden {
            return 0.0;
        }

        text_measurer
            .measure(self.display_text(), &Self::text_font())
            .width
    }

    fn text_height(&self, text_measurer: &TextMeasurer<'_>) -> Space {
        if self.hidden {
            return 0.0;
        }

        text_measurer
            .measure(self.display_text(), &Self::text_font())
            .height
    }

    pub fn metrics(&self, text_measurer: &TextMeasurer<'_>) -> BoundingBox {
        if self.hidden {
            return BoundingBox {
                width: 0.0,
                height: 0.0,
                right_padding: 0.0,
            };
        }

        BoundingBox {
            width: self.text_width(text_measurer),
            height: self.text_height(text_measurer),
            right_padding: 0.0,
        }
    }
}

impl Track {
    pub fn get_playing_technique_at_tick(
        &self,
        at: &Tick,
        kind: PlayingTechniqueKind,
    ) -> Option<&PlayingTechnique> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::PlayingTechnique(playing_technique)) = self.entries.by_id.get(key) {
                if playing_technique.kind == kind {
                    return Some(playing_technique);
                }
            }
        }

        None
    }
}

impl Track {
    pub fn get_playing_techniques_at_tick(&self, at: &Tick) -> Vec<&PlayingTechnique> {
        let Some(entry_keys) = self.entries.by_tick.get(at) else {
            return Vec::new();
        };

        entry_keys
            .iter()
            .filter_map(|key| match self.entries.by_id.get(key) {
                Some(Entry::PlayingTechnique(playing_technique)) => Some(playing_technique),
                _ => None,
            })
            .collect()
    }
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_playing_technique_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    tick: Tick,
    kind: PlayingTechniqueKind,
    hidden: bool,
) -> Result<UniqueId, String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let master_id = {
        let flow = state
            .score
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| String::from("Flow not found"))?;
        let stave = flow
            .staves
            .by_id
            .get(&stave_id)
            .ok_or_else(|| String::from("Stave not found"))?;
        stave.master_id
    };
    let playing_technique_id = state.generate_id();
    let master = state
        .score
        .tracks
        .by_id
        .get_mut(&master_id)
        .ok_or_else(|| String::from("Playing-technique track not found"))?;

    let mut playing_technique = PlayingTechnique::new(playing_technique_id, tick, kind);
    playing_technique.hidden = hidden;

    if let Some(existing_id) = master
        .get_playing_technique_at_tick(&tick, kind)
        .map(|playing_technique| playing_technique.id)
    {
        master.remove_entry(existing_id);
    }

    master.insert_entry(Entry::PlayingTechnique(playing_technique));

    state.emit_affected_flows(&app, &mut renderer, &[flow_id]);

    Ok(playing_technique_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_playing_technique_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    playing_technique_id: UniqueId,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let master_id = {
        let flow = state
            .score
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| String::from("Flow not found"))?;
        let stave = flow
            .staves
            .by_id
            .get(&stave_id)
            .ok_or_else(|| String::from("Stave not found"))?;
        stave.master_id
    };
    let master = state
        .score
        .tracks
        .by_id
        .get_mut(&master_id)
        .ok_or_else(|| String::from("Playing-technique track not found"))?;

    let entry = master
        .entries
        .by_id
        .get(&playing_technique_id)
        .ok_or_else(|| String::from("Playing technique not found"))?;

    if !matches!(entry, Entry::PlayingTechnique(_)) {
        return Err(String::from("Selected entry is not a playing technique"));
    }

    master.remove_entry(playing_technique_id);
    state.ui.sequencer.selection = None;
    state.emit_affected_flows(&app, &mut renderer, &[flow_id]);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_playing_technique_update(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    playing_technique_id: UniqueId,
    hidden: bool,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    let master_id = {
        let flow = state
            .score
            .flows
            .by_id
            .get(&flow_id)
            .ok_or_else(|| String::from("Flow not found"))?;
        let stave = flow
            .staves
            .by_id
            .get(&stave_id)
            .ok_or_else(|| String::from("Stave not found"))?;
        stave.master_id
    };

    let entry = state
        .score
        .tracks
        .by_id
        .get_mut(&master_id)
        .ok_or_else(|| String::from("Playing-technique track not found"))?
        .entries
        .by_id
        .get_mut(&playing_technique_id)
        .ok_or_else(|| String::from("Playing technique not found"))?;

    let Entry::PlayingTechnique(playing_technique) = entry else {
        return Err(String::from("Selected entry is not a playing technique"));
    };

    playing_technique.hidden = hidden;

    state.emit_affected_flows(&app, &mut renderer, &[flow_id]);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_playing_technique(
        id: UniqueId,
        tick: Tick,
        kind: PlayingTechniqueKind,
    ) -> PlayingTechnique {
        PlayingTechnique::new(id, tick, kind)
    }

    #[test]
    fn playing_technique_new_stores_fields() {
        let playing_technique = make_playing_technique(10, 64, PlayingTechniqueKind::Pizz);

        assert_eq!(playing_technique.id, 10);
        assert_eq!(playing_technique.tick, 64);
        assert_eq!(playing_technique.kind, PlayingTechniqueKind::Pizz);
        assert!(!playing_technique.hidden);
    }

    fn insert_playing_technique_like_create(
        track: &mut Track,
        playing_technique: PlayingTechnique,
    ) {
        if let Some(existing_id) = track
            .get_playing_technique_at_tick(&playing_technique.tick, playing_technique.kind)
            .map(|playing_technique| playing_technique.id)
        {
            track.remove_entry(existing_id);
        }

        track.insert_entry(Entry::PlayingTechnique(playing_technique));
    }

    fn update_playing_technique_hidden(
        track: &mut Track,
        playing_technique_id: UniqueId,
        hidden: bool,
    ) -> Result<(), String> {
        let entry = track
            .entries
            .by_id
            .get_mut(&playing_technique_id)
            .ok_or_else(|| String::from("Playing technique not found"))?;

        let Entry::PlayingTechnique(playing_technique) = entry else {
            return Err(String::from("Selected entry is not a playing technique"));
        };

        playing_technique.hidden = hidden;

        Ok(())
    }

    #[test]
    fn insert_playing_technique_keeps_distinct_same_tick_kinds() {
        let mut track = Track::new(1);

        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(10, 64, PlayingTechniqueKind::Pizz),
        );
        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(11, 64, PlayingTechniqueKind::Arco),
        );

        let at_tick = track.get_playing_techniques_at_tick(&64);
        assert_eq!(at_tick.len(), 2);
        assert!(at_tick
            .iter()
            .any(|playing_technique| playing_technique.id == 10));
        assert!(at_tick
            .iter()
            .any(|playing_technique| playing_technique.id == 11));
    }

    #[test]
    fn insert_playing_technique_replaces_exact_duplicate_kind_at_same_tick() {
        let mut track = Track::new(1);

        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(10, 64, PlayingTechniqueKind::Pizz),
        );
        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(11, 64, PlayingTechniqueKind::Pizz),
        );

        let ids_at_tick = track.entries.by_tick.get(&64).unwrap();
        assert_eq!(ids_at_tick, &vec![11]);
        assert!(!track.entries.by_id.contains_key(&10));
        assert!(track.entries.by_id.contains_key(&11));
    }

    #[test]
    fn remove_playing_technique_removes_only_targeted_entry() {
        let mut track = Track::new(1);

        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(10, 64, PlayingTechniqueKind::Pizz),
        );
        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(11, 64, PlayingTechniqueKind::Arco),
        );
        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(12, 96, PlayingTechniqueKind::Pizz),
        );

        let removed = track.remove_entry(10);

        assert!(removed.is_some());
        let at_tick = track.get_playing_techniques_at_tick(&64);
        assert_eq!(at_tick.len(), 1);
        assert_eq!(at_tick[0].id, 11);
        assert_eq!(track.get_playing_techniques_at_tick(&96).len(), 1);
        assert!(track.entries.by_id.contains_key(&12));
    }

    #[test]
    fn update_playing_technique_updates_hidden_without_changing_kind() {
        let mut track = Track::new(1);

        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(10, 64, PlayingTechniqueKind::Pizz),
        );
        insert_playing_technique_like_create(
            &mut track,
            make_playing_technique(11, 64, PlayingTechniqueKind::Arco),
        );

        update_playing_technique_hidden(&mut track, 10, true).unwrap();

        let ids_at_tick = track.entries.by_tick.get(&64).unwrap();
        assert_eq!(ids_at_tick, &vec![10, 11]);
        assert!(track.entries.by_id.contains_key(&11));

        let Entry::PlayingTechnique(playing_technique) = track.entries.by_id.get(&10).unwrap()
        else {
            panic!("expected playing-technique entry");
        };
        assert!(playing_technique.hidden);
        assert_eq!(playing_technique.kind, PlayingTechniqueKind::Pizz);
    }
}
