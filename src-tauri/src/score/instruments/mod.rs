pub mod defs;

use super::{config::AutoCountStyle, Score};
use crate::playback::transport::TransportSharedState;
use crate::score::players::PlayerType;
use crate::utils::UniqueId;
use crate::{AppContextState, AudioContextState, RendererContextState, SamplerContextState};
use defs::{get_instrument_definition, InstrumentType};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub id: UniqueId,
    pub definition_id: String,
    pub instrument_type: InstrumentType,
    pub long_name: String,
    pub short_name: String,
    pub stave_ids: Vec<UniqueId>,
    pub count: Option<String>,
    pub assigned_plugin_id: Option<String>,
    pub volume: u8,
    pub pan: u8,
    pub mute: bool,
    pub solo: bool,
}

impl Instrument {
    pub fn family(&self) -> &str {
        let mut parts = self.definition_id.split('.');
        match parts.next() {
            None => "unknown",
            Some(part) => part,
        }
    }

    pub fn name(&self) -> String {
        match &self.count {
            Some(count) => format!("{} {}", self.long_name, count),
            None => self.long_name.clone(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Instruments {
    pub by_id: FxHashMap<UniqueId, Instrument>,
}

impl Instruments {
    pub fn new() -> Self {
        Instruments {
            by_id: FxHashMap::default(),
        }
    }
}

impl Score {
    pub fn update_counts(&mut self) {
        let mut instruments_by_type: FxHashMap<PlayerType, FxHashMap<String, Vec<UniqueId>>> =
            FxHashMap::default();

        for player_id in &self.players.order {
            let player = &self.players.by_id[player_id];

            for instrument_id in &player.instrument_ids {
                let instrument = self.instruments.by_id.get_mut(instrument_id).unwrap();

                // Reset the current count
                instrument.count = None;

                // Group instruments by player type and long name
                instruments_by_type
                    .entry(player.player_type)
                    .or_default()
                    .entry(instrument.long_name.clone())
                    .or_default()
                    .push(*instrument_id);
            }
        }

        // Assign counts to instruments for each player type
        for (player_type, instruments) in instruments_by_type {
            let count_style = match player_type {
                PlayerType::Solo => &self.config.auto_count_styles.solo,
                PlayerType::Section => &self.config.auto_count_styles.section,
            };

            for instrument_ids in instruments.values() {
                if instrument_ids.len() > 1 {
                    for (i, instrument_id) in instrument_ids.iter().enumerate() {
                        let instrument = self.instruments.by_id.get_mut(instrument_id).unwrap();
                        let count = (i + 1) as i32;

                        // Format the count based on the style
                        let styled_count = match count_style {
                            AutoCountStyle::Roman => {
                                roman::to(count).expect("Failed to convert to Roman")
                            }
                            AutoCountStyle::Arabic => count.to_string(),
                        };

                        instrument.count = Some(styled_count);
                    }
                }
            }
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    definition_id: String,
) -> UniqueId {
    let mut state = state.lock().unwrap();

    let instrument_id = state.generate_id();

    let def = get_instrument_definition(&definition_id).unwrap();
    let stave_ids = def
        .staves
        .iter()
        .map(|_| state.generate_id())
        .collect::<Vec<UniqueId>>();

    let instrument = Instrument {
        id: instrument_id,
        definition_id: definition_id.clone(),
        instrument_type: def.instrument_type,
        long_name: String::from(def.long_name),
        short_name: String::from(def.short_name),
        stave_ids,
        count: None,
        assigned_plugin_id: None,
        volume: 100,
        pan: 64,
        mute: false,
        solo: false,
    };

    state
        .score
        .instruments
        .by_id
        .insert(instrument_id, instrument);

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, true);

    instrument_id
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_move(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    from_index: usize,
    to_index: usize,
    from_player_id: UniqueId,
    to_player_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    let from_player = state.score.players.by_id.get_mut(&from_player_id).unwrap();
    let instrument_id = from_player.instrument_ids.remove(from_index);

    if from_player_id == to_player_id {
        from_player.instrument_ids.insert(to_index, instrument_id);
    } else {
        for flow_id in state.score.flows.order.clone() {
            let flow = &state.score.flows.by_id[&flow_id];
            let is_from_player_in_flow = flow.player_ids.contains(&from_player_id);
            let is_to_player_in_flow = flow.player_ids.contains(&to_player_id);

            // the players are either both in the flow or both not in the flow.
            // either way we don't have to do anything.
            if is_from_player_in_flow == is_to_player_in_flow {
                continue;
            }

            if is_from_player_in_flow {
                // moving instrument from player in flow -> player not in flow
                state.assign_player_to_flow(flow_id, to_player_id);
            } else {
                // moving instrument from player not in flow -> player in flow
                state.assign_instrument_to_flow(flow_id, instrument_id);
            }
        }

        // only assign to the new player after the flows loop to cover the situation where:
        // the instrument is already in the flow but the other instruments the to player has aren't
        // we can simply use the assign_player_to_flow helper without the new instrument (as it's already assigned)
        let to_player = state.score.players.by_id.get_mut(&to_player_id).unwrap();
        to_player.instrument_ids.insert(to_index, instrument_id);
    }

    state.score.update_counts();

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.score.instruments.by_id.remove(&instrument_id);

    // Sync sampler to unload plugins that are no longer needed
    let assigned_plugin_ids: Vec<String> = state
        .score
        .instruments
        .by_id
        .values()
        .filter_map(|i| i.assigned_plugin_id.clone())
        .collect();

    let mut sampler = sampler_state.lock().unwrap();
    let audio = audio_state.lock().unwrap();
    let sample_rate = audio.get_sample_rate();
    drop(audio);

    if let Err(e) = sampler.sync_with_instruments(&assigned_plugin_ids, sample_rate) {
        eprintln!("Failed to sync sampler: {}", e);
    }
    sampler.emit(&app);

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, true);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_assign_plugin(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    instrument_id: UniqueId,
    plugin_id: Option<String>,
) {
    let mut state = state.lock().unwrap();

    if let Some(instrument) = state.score.instruments.by_id.get_mut(&instrument_id) {
        instrument.assigned_plugin_id = plugin_id;
    }

    // Collect all assigned plugin IDs and sync the sampler
    let assigned_plugin_ids: Vec<String> = state
        .score
        .instruments
        .by_id
        .values()
        .filter_map(|i| i.assigned_plugin_id.clone())
        .collect();

    let mut sampler = sampler_state.lock().unwrap();
    let audio = audio_state.lock().unwrap();
    let sample_rate = audio.get_sample_rate();
    drop(audio); // Release audio lock before potentially slow operation

    if let Err(e) = sampler.sync_with_instruments(&assigned_plugin_ids, sample_rate) {
        eprintln!("Failed to sync sampler: {}", e);
    }
    sampler.emit(&app);

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_set_volume(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    instrument_id: UniqueId,
    volume: u8,
) {
    let clamped = volume.min(127);
    let mut state = state.lock().unwrap();

    if let Some(instrument) = state.score.instruments.by_id.get_mut(&instrument_id) {
        instrument.volume = clamped;
    }

    // Update live mix params for in-flight playback
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.volume = clamped;
        }
    }

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_set_pan(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    instrument_id: UniqueId,
    pan: u8,
) {
    let clamped = pan.min(127);
    let mut state = state.lock().unwrap();

    if let Some(instrument) = state.score.instruments.by_id.get_mut(&instrument_id) {
        instrument.pan = clamped;
    }

    // Update live mix params for in-flight playback
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.pan = clamped;
        }
    }

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_set_mute(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    instrument_id: UniqueId,
    mute: bool,
) {
    let mut state = state.lock().unwrap();

    if let Some(instrument) = state.score.instruments.by_id.get_mut(&instrument_id) {
        instrument.mute = mute;
    }

    // Update live mix params for in-flight playback
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.mute = mute;
        }
    }

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, false);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_set_solo(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    instrument_id: UniqueId,
    solo: bool,
) {
    let mut state = state.lock().unwrap();

    if let Some(instrument) = state.score.instruments.by_id.get_mut(&instrument_id) {
        instrument.solo = solo;
    }

    // Update live mix params for in-flight playback
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.solo = solo;
        }
    }

    let mut renderer = renderer_state.lock().unwrap();
    state.emit(&app, &mut renderer, false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use defs::InstrumentType;

    fn make_instrument(definition_id: &str, long_name: &str) -> Instrument {
        Instrument {
            id: 1,
            definition_id: definition_id.to_string(),
            instrument_type: InstrumentType::Melodic,
            long_name: long_name.to_string(),
            short_name: String::from(""),
            stave_ids: vec![],
            count: None,
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    #[test]
    fn instruments_new_is_empty() {
        let inst = Instruments::new();
        assert!(inst.by_id.is_empty());
    }

    #[test]
    fn family_extracts_first_segment() {
        let i = make_instrument("brass.horn.f", "Horn");
        assert_eq!(i.family(), "brass");
    }

    #[test]
    fn family_single_segment_returns_full() {
        let i = make_instrument("brass", "Brass");
        assert_eq!(i.family(), "brass");
    }

    #[test]
    fn name_without_count_is_long_name() {
        let i = make_instrument("brass.horn.f", "Horn in F");
        assert_eq!(i.name(), "Horn in F");
    }

    #[test]
    fn name_with_count_appends_count() {
        let mut i = make_instrument("brass.horn.f", "Horn in F");
        i.count = Some(String::from("II"));
        assert_eq!(i.name(), "Horn in F II");
    }

    #[test]
    fn name_with_arabic_count() {
        let mut i = make_instrument("strings.violin", "Violin");
        i.count = Some(String::from("1"));
        assert_eq!(i.name(), "Violin 1");
    }

    #[test]
    fn instrument_mute_default_false() {
        let i = make_instrument("woodwind.flute", "Flute");
        assert!(!i.mute);
    }

    #[test]
    fn instrument_solo_default_false() {
        let i = make_instrument("woodwind.flute", "Flute");
        assert!(!i.solo);
    }

    #[test]
    fn instrument_volume_default_100() {
        let i = make_instrument("woodwind.flute", "Flute");
        assert_eq!(i.volume, 100);
    }

    #[test]
    fn instrument_pan_default_64() {
        let i = make_instrument("woodwind.flute", "Flute");
        assert_eq!(i.pan, 64);
    }

    #[test]
    fn instrument_assigned_plugin_id_default_none() {
        let i = make_instrument("woodwind.flute", "Flute");
        assert!(i.assigned_plugin_id.is_none());
    }

    #[test]
    fn instrument_count_default_none() {
        let i = make_instrument("woodwind.flute", "Flute");
        assert!(i.count.is_none());
    }

    #[test]
    fn instrument_stave_ids_default_empty() {
        let i = make_instrument("woodwind.flute", "Flute");
        assert!(i.stave_ids.is_empty());
    }

    #[test]
    fn instruments_by_id_can_insert() {
        let mut instruments = Instruments::new();
        let i = make_instrument("woodwind.flute", "Flute");
        instruments.by_id.insert(i.id, i);
        assert_eq!(instruments.by_id.len(), 1);
    }

    #[test]
    fn family_with_empty_definition_id() {
        let i = make_instrument("", "Unknown");
        // split('.'). next() on empty string returns Some("") which is "unknown"-no-match
        // but actually empty string split gives Some("")
        // The code returns Some(part) where part = "" → returns ""
        assert_eq!(i.family(), "");
    }

    #[test]
    fn name_with_set_count_contains_space() {
        let mut i = make_instrument("strings.violin", "Violin");
        i.count = Some(String::from("III"));
        let name = i.name();
        assert!(name.contains(' '));
        assert!(name.ends_with("III"));
    }
}
