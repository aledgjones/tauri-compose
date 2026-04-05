pub mod defs;

use super::{
    config::AutoCountStyle,
    playback::{
        InstrumentPlaybackRoute, PlaybackRackSlot, PlaybackRouteAddress, PlaybackRouteAddressMode,
        PlaybackRouteChoice, PlaybackRouteTarget, PlaybackRouteTargetMode,
    },
    Score,
};
use crate::instructions::FlowRenderRecalculationRequest;
use crate::playback::engine::sync_slot_mix_states;
use crate::playback::preparation::{
    effective_playback_mix_by_slot_id, effective_playback_mix_for_instrument,
};
use crate::playback::transport::{
    DeferredPlaybackRefreshKind, InstrumentMixParams, TransportSharedState,
};
use crate::score::players::PlayerType;
use crate::utils::UniqueId;
use crate::{AppContextState, RendererContextState};
use defs::{get_instrument_definition, InstrumentType};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tauri::State;

#[cfg(test)]
use rustc_hash::FxHashSet;

#[cfg(test)]
use super::playback::ResolvedPlaybackRoute;

#[cfg(test)]
use crate::playback::{
    plugin_inventory::{PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry},
    runtime_activation::PlaybackActivationRuntime,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub id: UniqueId,
    pub definition_id: String,
    pub instrument_type: InstrumentType,
    pub long_name: String,
    pub short_name: String,
    pub stave_ids: Vec<UniqueId>,
    pub count: Option<String>,
    pub playback_route: Option<InstrumentPlaybackRoute>,
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

    pub fn set_volume(&mut self, volume: u8) -> u8 {
        let clamped = volume.min(127);
        self.volume = clamped;
        clamped
    }

    pub fn set_pan(&mut self, pan: u8) -> u8 {
        let clamped = pan.min(127);
        self.pan = clamped;
        clamped
    }

    pub fn set_mute(&mut self, mute: bool) {
        self.mute = mute;
    }

    pub fn set_solo(&mut self, solo: bool) {
        self.solo = solo;
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
    fn default_playback_route_choices_for_slot(
        slot: &PlaybackRackSlot,
    ) -> Vec<PlaybackRouteChoice> {
        if slot.target.is_none() {
            return Vec::new();
        }

        vec![PlaybackRouteChoice {
            key: String::from("plugin-root"),
            label: String::from("Plugin output"),
            target: PlaybackRouteTarget::plugin(),
            address: PlaybackRouteAddress::root(),
        }]
    }

    pub fn playback_route_choices_for_slot(
        &self,
        slot_id: UniqueId,
    ) -> Result<Vec<PlaybackRouteChoice>, String> {
        let slot = self
            .playback
            .rack
            .by_id
            .get(&slot_id)
            .ok_or_else(|| format!("Playback slot {} does not exist", slot_id))?;

        Ok(Self::default_playback_route_choices_for_slot(slot))
    }

    pub fn playback_route_choices_by_slot_id(
        &self,
    ) -> FxHashMap<UniqueId, Vec<PlaybackRouteChoice>> {
        self.playback
            .rack
            .by_id
            .iter()
            .map(|(slot_id, slot)| {
                (
                    *slot_id,
                    Self::default_playback_route_choices_for_slot(slot),
                )
            })
            .collect()
    }

    fn route_choice_matches(choice: &PlaybackRouteChoice, route: &InstrumentPlaybackRoute) -> bool {
        choice.target == route.target && choice.address == route.address
    }

    fn route_is_direct_plugin_output(route: &InstrumentPlaybackRoute) -> bool {
        route.target == PlaybackRouteTarget::plugin()
            && route.address == PlaybackRouteAddress::root()
    }

    fn format_route_value(target: &PlaybackRouteTarget, address: &PlaybackRouteAddress) -> String {
        let target_label = match target.mode {
            PlaybackRouteTargetMode::Plugin => String::from("Plugin output"),
            PlaybackRouteTargetMode::PluginPart => format!(
                "Plugin part {}",
                target.index.map(|index| index + 1).unwrap_or(1)
            ),
            PlaybackRouteTargetMode::MidiChannel => format!(
                "MIDI channel {}",
                target.index.map(|index| index + 1).unwrap_or(1)
            ),
        };

        let address_label = match address.mode {
            PlaybackRouteAddressMode::Root => String::from("Root"),
            PlaybackRouteAddressMode::Part => {
                format!("Part {}", address.index.map(|index| index + 1).unwrap_or(1))
            }
            PlaybackRouteAddressMode::MidiChannel => format!(
                "MIDI channel {}",
                address.index.map(|index| index + 1).unwrap_or(1)
            ),
        };

        format!("{} via {}", target_label, address_label)
    }

    pub fn create_playback_slot(&mut self, slot_id: UniqueId) -> Result<(), String> {
        if self.playback.rack.by_id.contains_key(&slot_id) {
            return Err(format!("Playback slot {} already exists", slot_id));
        }

        self.playback.rack.order.push(slot_id);
        self.playback
            .rack
            .by_id
            .insert(slot_id, PlaybackRackSlot::new(slot_id));

        Ok(())
    }

    pub fn remove_playback_slot(&mut self, slot_id: UniqueId) -> Result<(), String> {
        if !self.playback.rack.by_id.contains_key(&slot_id) {
            return Err(format!("Playback slot {} does not exist", slot_id));
        }

        for instrument in self.instruments.by_id.values_mut() {
            if instrument
                .playback_route
                .as_ref()
                .is_some_and(|route| route.slot_id == slot_id)
            {
                instrument.playback_route = None;
            }
        }

        self.playback
            .rack
            .order
            .retain(|current_slot_id| *current_slot_id != slot_id);
        self.playback.rack.by_id.remove(&slot_id);

        Ok(())
    }

    pub fn assign_plugin_to_slot(
        &mut self,
        slot_id: UniqueId,
        plugin_id: String,
    ) -> Result<(), String> {
        let slot = self
            .playback
            .rack
            .by_id
            .get_mut(&slot_id)
            .ok_or_else(|| format!("Playback slot {} does not exist", slot_id))?;

        slot.target = Some(super::playback::PlaybackSlotTarget {
            plugin_id,
            plugin_state: None,
        });

        Ok(())
    }

    pub fn unassign_plugin_from_slot(&mut self, slot_id: UniqueId) -> Result<(), String> {
        let slot = self
            .playback
            .rack
            .by_id
            .get_mut(&slot_id)
            .ok_or_else(|| format!("Playback slot {} does not exist", slot_id))?;

        slot.target = None;

        Ok(())
    }

    #[cfg(test)]
    pub fn assign_instrument_route_to_slot(
        &mut self,
        instrument_id: UniqueId,
        slot_id: UniqueId,
    ) -> Result<(), String> {
        if !self.playback.rack.by_id.contains_key(&slot_id) {
            return Err(format!("Playback slot {} does not exist", slot_id));
        }

        let instrument = self
            .instruments
            .by_id
            .get_mut(&instrument_id)
            .ok_or_else(|| format!("Instrument {} does not exist", instrument_id))?;

        instrument.playback_route = Some(InstrumentPlaybackRoute::direct(slot_id));

        Ok(())
    }

    pub fn assign_instrument_route(
        &mut self,
        instrument_id: UniqueId,
        route: InstrumentPlaybackRoute,
    ) -> Result<(), String> {
        if let Some(slot) = self.playback.rack.by_id.get(&route.slot_id) {
            if slot.target.is_none() {
                if !Self::route_is_direct_plugin_output(&route) {
                    return Err(format!(
                        "Requested route {} is not supported for empty playback slot {}",
                        Self::format_route_value(&route.target, &route.address),
                        route.slot_id
                    ));
                }

                let instrument = self
                    .instruments
                    .by_id
                    .get_mut(&instrument_id)
                    .ok_or_else(|| format!("Instrument {} does not exist", instrument_id))?;

                instrument.playback_route = Some(route);

                return Ok(());
            }
        }

        let choices = self.playback_route_choices_for_slot(route.slot_id)?;

        if choices.is_empty() {
            return Err(format!(
                "Playback slot {} does not have a safely routable target yet",
                route.slot_id
            ));
        }

        if !choices
            .iter()
            .any(|choice| Self::route_choice_matches(choice, &route))
        {
            return Err(format!(
                "Requested route {} is not supported for playback slot {}",
                Self::format_route_value(&route.target, &route.address),
                route.slot_id
            ));
        }

        let instrument = self
            .instruments
            .by_id
            .get_mut(&instrument_id)
            .ok_or_else(|| format!("Instrument {} does not exist", instrument_id))?;

        instrument.playback_route = Some(route);

        Ok(())
    }

    pub fn unassign_instrument_route(&mut self, instrument_id: UniqueId) -> Result<(), String> {
        let instrument = self
            .instruments
            .by_id
            .get_mut(&instrument_id)
            .ok_or_else(|| format!("Instrument {} does not exist", instrument_id))?;

        instrument.playback_route = None;

        Ok(())
    }

    #[cfg(test)]
    fn playback_test_inventory(available_plugin_ids: &FxHashSet<String>) -> PluginInventoryContext {
        PluginInventoryContext::with_entries_for_tests(
            available_plugin_ids
                .iter()
                .map(|plugin_id| PluginInventoryEntry {
                    id: plugin_id.clone(),
                    name: plugin_id.clone(),
                    bundle_path: format!("/Library/Audio/Plug-Ins/VST3/{plugin_id}.vst3"),
                    binary_path: Some(format!(
                        "/Library/Audio/Plug-Ins/VST3/{plugin_id}.vst3/Contents/MacOS/{plugin_id}"
                    )),
                    source_root: String::from("/Library/Audio/Plug-Ins/VST3"),
                    availability: PluginInventoryAvailability::Available,
                    validation_failure: None,
                    editor_host_validation: None,
                })
                .collect(),
        )
    }

    #[cfg(test)]
    pub fn instrument_effective_plugin_id(
        &self,
        instrument_id: UniqueId,
        available_plugin_ids: &FxHashSet<String>,
    ) -> Option<&str> {
        let instrument = self.instruments.by_id.get(&instrument_id)?;
        let route = instrument.playback_route.as_ref()?;
        let slot = self.playback.rack.by_id.get(&route.slot_id)?;
        let plugin_id = slot.target.as_ref()?.plugin_id.as_str();

        available_plugin_ids
            .contains(plugin_id)
            .then_some(plugin_id)
    }

    #[cfg(test)]
    pub fn routed_plugin_ids(&self, available_plugin_ids: &FxHashSet<String>) -> Vec<String> {
        self.instruments
            .by_id
            .keys()
            .filter_map(|instrument_id| {
                self.instrument_effective_plugin_id(*instrument_id, available_plugin_ids)
                    .map(String::from)
            })
            .collect()
    }

    #[cfg(test)]
    pub fn resolved_playback_route_by_instrument_id(
        &self,
        available_plugin_ids: &FxHashSet<String>,
    ) -> FxHashMap<UniqueId, ResolvedPlaybackRoute> {
        let inventory = Self::playback_test_inventory(available_plugin_ids);

        PlaybackActivationRuntime::from_score_and_inventory(self, &inventory)
            .resolved_playback_route_by_instrument_id(self)
    }

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

fn sync_live_playback_mix_state(
    score: &Score,
    playback_runtime: &crate::playback::runtime_activation::PlaybackActivationRuntime,
    transport_shared: &TransportSharedState,
) {
    let resolved_playback_route_by_instrument_id =
        playback_runtime.resolved_playback_route_by_instrument_id(score);

    if let Ok(mut mix) = transport_shared.mix_params.write() {
        mix.clear();
        for (instrument_id, instrument) in &score.instruments.by_id {
            let supported_mix_controls = resolved_playback_route_by_instrument_id
                .get(instrument_id)
                .map(|route| route.supported_mix_controls)
                .unwrap_or_default();
            let effective_mix = effective_playback_mix_for_instrument(
                score,
                &resolved_playback_route_by_instrument_id,
                *instrument_id,
            );

            mix.insert(
                *instrument_id,
                InstrumentMixParams {
                    volume: effective_mix.volume,
                    pan: effective_mix.pan,
                    mute: effective_mix.muted,
                    solo: supported_mix_controls.solo && instrument.solo,
                },
            );
        }
    }

    sync_slot_mix_states(effective_playback_mix_by_slot_id(score, playback_runtime));
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
        playback_route: None,
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
    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );

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
    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.score.instruments.by_id.remove(&instrument_id);

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_after_playback_edit(
        &app,
        &mut renderer,
        DeferredPlaybackRefreshKind::PlaybackTopology,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_playback_slots_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
) -> Result<UniqueId, String> {
    let mut state = state.lock().unwrap();
    let slot_id = state.generate_id();

    state.score.create_playback_slot(slot_id)?;

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_after_playback_edit(
        &app,
        &mut renderer,
        DeferredPlaybackRefreshKind::PlaybackTopology,
        None,
    );

    Ok(slot_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_playback_slots_remove(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    slot_id: UniqueId,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    state.score.remove_playback_slot(slot_id)?;

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_after_playback_edit(
        &app,
        &mut renderer,
        DeferredPlaybackRefreshKind::PlaybackTopology,
        None,
    );

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_playback_slots_assign_plugin(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    slot_id: UniqueId,
    plugin_id: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    state.score.assign_plugin_to_slot(slot_id, plugin_id)?;

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_after_playback_edit(
        &app,
        &mut renderer,
        DeferredPlaybackRefreshKind::PlaybackTopology,
        None,
    );

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_playback_slots_unassign_plugin(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    slot_id: UniqueId,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    state.score.unassign_plugin_from_slot(slot_id)?;

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_after_playback_edit(
        &app,
        &mut renderer,
        DeferredPlaybackRefreshKind::PlaybackTopology,
        None,
    );

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_assign_slot_route(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    instrument_id: UniqueId,
    route: InstrumentPlaybackRoute,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    state.score.assign_instrument_route(instrument_id, route)?;

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_after_playback_edit(
        &app,
        &mut renderer,
        DeferredPlaybackRefreshKind::PlaybackTopology,
        None,
    );

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_unassign_route(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    instrument_id: UniqueId,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    state.score.unassign_instrument_route(instrument_id)?;

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_after_playback_edit(
        &app,
        &mut renderer,
        DeferredPlaybackRefreshKind::PlaybackTopology,
        None,
    );

    Ok(())
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
    let mut state = state.lock().unwrap();

    state
        .score
        .instruments
        .by_id
        .get_mut(&instrument_id)
        .map(|instrument| instrument.set_volume(volume))
        .unwrap_or_else(|| volume.min(127));

    // Update live mix params for in-flight playback
    sync_live_playback_mix_state(
        &state.score,
        &state.playback_activation_runtime,
        transport_shared.inner(),
    );

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_general_state(&app, &mut renderer, None);
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
    let mut state = state.lock().unwrap();

    state
        .score
        .instruments
        .by_id
        .get_mut(&instrument_id)
        .map(|instrument| instrument.set_pan(pan))
        .unwrap_or_else(|| pan.min(127));

    // Update live mix params for in-flight playback
    sync_live_playback_mix_state(
        &state.score,
        &state.playback_activation_runtime,
        transport_shared.inner(),
    );

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_general_state(&app, &mut renderer, None);
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
        instrument.set_mute(mute);
    }

    // Update live mix params for in-flight playback
    sync_live_playback_mix_state(
        &state.score,
        &state.playback_activation_runtime,
        transport_shared.inner(),
    );

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_general_state(&app, &mut renderer, None);
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
        instrument.set_solo(solo);
    }

    // Update live mix params for in-flight playback
    sync_live_playback_mix_state(
        &state.score,
        &state.playback_activation_runtime,
        transport_shared.inner(),
    );

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_general_state(&app, &mut renderer, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::playback::{
        PlaybackRouteAddress, PlaybackRouteChoice, PlaybackRouteTarget, ResolvedPlaybackRouteStatus,
    };
    use crate::score::players::{Player, PlayerType, Players};
    use crate::score::Score;
    use defs::InstrumentType;

    fn make_instrument(id: UniqueId, definition_id: &str, long_name: &str) -> Instrument {
        Instrument {
            id,
            definition_id: definition_id.to_string(),
            instrument_type: InstrumentType::Melodic,
            long_name: long_name.to_string(),
            short_name: String::from(""),
            stave_ids: vec![],
            count: None,
            playback_route: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    fn make_player(id: UniqueId, player_type: PlayerType, instrument_ids: Vec<UniqueId>) -> Player {
        Player {
            id,
            player_type,
            instrument_ids,
        }
    }

    fn make_score(players: Vec<Player>, instruments: Vec<Instrument>) -> Score {
        let mut score = Score::new();

        score.players = Players {
            order: players.iter().map(|player| player.id).collect(),
            by_id: players
                .into_iter()
                .map(|player| (player.id, player))
                .collect(),
        };
        score.instruments.by_id = instruments
            .into_iter()
            .map(|instrument| (instrument.id, instrument))
            .collect();

        score
    }

    fn add_slot(score: &mut Score, slot_id: UniqueId, plugin_id: &str) {
        score.playback.rack.order.push(slot_id);
        score.playback.rack.by_id.insert(
            slot_id,
            PlaybackRackSlot {
                id: slot_id,
                target: Some(crate::score::playback::PlaybackSlotTarget {
                    plugin_id: String::from(plugin_id),
                    plugin_state: None,
                }),
            },
        );
    }

    fn add_empty_slot(score: &mut Score, slot_id: UniqueId) {
        score.playback.rack.order.push(slot_id);
        score
            .playback
            .rack
            .by_id
            .insert(slot_id, PlaybackRackSlot::new(slot_id));
    }

    #[test]
    fn instruments_new_is_empty() {
        let inst = Instruments::new();
        assert!(inst.by_id.is_empty());
    }

    #[test]
    fn family_extracts_first_segment() {
        let i = make_instrument(1, "brass.horn.f", "Horn");
        assert_eq!(i.family(), "brass");
    }

    #[test]
    fn family_single_segment_returns_full() {
        let i = make_instrument(1, "brass", "Brass");
        assert_eq!(i.family(), "brass");
    }

    #[test]
    fn name_without_count_is_long_name() {
        let i = make_instrument(1, "brass.horn.f", "Horn in F");
        assert_eq!(i.name(), "Horn in F");
    }

    #[test]
    fn name_with_count_appends_count() {
        let mut i = make_instrument(1, "brass.horn.f", "Horn in F");
        i.count = Some(String::from("II"));
        assert_eq!(i.name(), "Horn in F II");
    }

    #[test]
    fn name_with_arabic_count() {
        let mut i = make_instrument(1, "strings.violin", "Violin");
        i.count = Some(String::from("1"));
        assert_eq!(i.name(), "Violin 1");
    }

    #[test]
    fn instrument_mute_default_false() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert!(!i.mute);
    }

    #[test]
    fn instrument_solo_default_false() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert!(!i.solo);
    }

    #[test]
    fn instrument_volume_default_100() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert_eq!(i.volume, 100);
    }

    #[test]
    fn instrument_pan_default_64() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert_eq!(i.pan, 64);
    }

    #[test]
    fn instrument_playback_route_defaults_to_unrouted() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert_eq!(i.playback_route, None);
    }

    #[test]
    fn effective_plugin_id_reads_direct_route() {
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("plugin-a"));

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        assert_eq!(
            score.instrument_effective_plugin_id(10, &available_plugin_ids),
            Some("plugin-a")
        );
    }

    #[test]
    fn playback_route_choices_expose_direct_plugin_target_for_prepared_slot() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_slot(&mut score, 100, "plugin-a");

        assert_eq!(
            score.playback_route_choices_for_slot(100).unwrap(),
            vec![PlaybackRouteChoice {
                key: String::from("plugin-root"),
                label: String::from("Plugin output"),
                target: PlaybackRouteTarget::plugin(),
                address: PlaybackRouteAddress::root(),
            }]
        );
    }

    #[test]
    fn playback_route_choices_for_empty_slot_are_empty() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_empty_slot(&mut score, 100);

        assert!(score
            .playback_route_choices_for_slot(100)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn assign_instrument_route_persists_explicit_direct_target_and_address() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_slot(&mut score, 100, "plugin-a");

        score
            .assign_instrument_route(
                10,
                InstrumentPlaybackRoute {
                    slot_id: 100,
                    target: PlaybackRouteTarget::plugin(),
                    address: PlaybackRouteAddress::root(),
                    expression_map_id: None,
                },
            )
            .expect("route assignment should succeed");

        assert_eq!(
            score.instruments.by_id[&10].playback_route,
            Some(InstrumentPlaybackRoute {
                slot_id: 100,
                target: PlaybackRouteTarget::plugin(),
                address: PlaybackRouteAddress::root(),
                expression_map_id: None,
            })
        );
    }

    #[test]
    fn assign_instrument_route_rejects_unsupported_target_request() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_slot(&mut score, 100, "plugin-a");

        let error = score
            .assign_instrument_route(
                10,
                InstrumentPlaybackRoute {
                    slot_id: 100,
                    target: PlaybackRouteTarget {
                        mode: super::super::playback::PlaybackRouteTargetMode::PluginPart,
                        index: Some(0),
                    },
                    address: PlaybackRouteAddress {
                        mode: super::super::playback::PlaybackRouteAddressMode::Part,
                        index: Some(0),
                    },
                    expression_map_id: None,
                },
            )
            .expect_err("unsupported target should fail");

        assert!(error.contains("Requested route Plugin part 1 via Part 1 is not supported"));
    }

    #[test]
    fn assign_instrument_route_accepts_existing_empty_slot() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_empty_slot(&mut score, 100);

        score
            .assign_instrument_route(10, InstrumentPlaybackRoute::direct(100))
            .expect("empty slot selection should persist");

        assert_eq!(
            score.instruments.by_id[&10].playback_route,
            Some(InstrumentPlaybackRoute::direct(100))
        );
    }

    #[test]
    fn assign_instrument_route_rejects_non_root_target_for_empty_slot() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_empty_slot(&mut score, 100);

        let error = score
            .assign_instrument_route(
                10,
                InstrumentPlaybackRoute {
                    slot_id: 100,
                    target: PlaybackRouteTarget {
                        mode: super::super::playback::PlaybackRouteTargetMode::PluginPart,
                        index: Some(0),
                    },
                    address: PlaybackRouteAddress {
                        mode: super::super::playback::PlaybackRouteAddressMode::Part,
                        index: Some(0),
                    },
                    expression_map_id: None,
                },
            )
            .expect_err("empty slots should reject non-root targets");

        assert!(error.contains(
            "Requested route Plugin part 1 via Part 1 is not supported for empty playback slot 100"
        ));
    }

    #[test]
    fn plugin_part_route_without_available_slot_normalizes_to_unrouted() {
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("plugin-a"));

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute {
                slot_id: 400,
                target: PlaybackRouteTarget {
                    mode: super::super::playback::PlaybackRouteTargetMode::PluginPart,
                    index: Some(2),
                },
                address: PlaybackRouteAddress {
                    mode: super::super::playback::PlaybackRouteAddressMode::Part,
                    index: Some(2),
                },
                expression_map_id: None,
            });

        let resolved = score.resolved_playback_route_by_instrument_id(&available_plugin_ids);

        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::Unrouted
        );
        assert_eq!(resolved.get(&10).unwrap().slot_id, None);
        assert_eq!(resolved.get(&10).unwrap().slot_target, None);
        assert_eq!(resolved.get(&10).unwrap().effective_plugin_id, None);
    }

    #[test]
    fn direct_route_without_available_inventory_resolves_to_attention_needed() {
        let available_plugin_ids = FxHashSet::default();

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        add_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let resolved = score.resolved_playback_route_by_instrument_id(&available_plugin_ids);

        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
        assert_eq!(resolved.get(&10).unwrap().effective_plugin_id, None);
    }

    #[test]
    fn instrument_count_default_none() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert!(i.count.is_none());
    }

    #[test]
    fn instrument_stave_ids_default_empty() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert!(i.stave_ids.is_empty());
    }

    #[test]
    fn instruments_by_id_can_insert() {
        let mut instruments = Instruments::new();
        let i = make_instrument(1, "woodwind.flute", "Flute");
        instruments.by_id.insert(i.id, i);
        assert_eq!(instruments.by_id.len(), 1);
    }

    #[test]
    fn family_with_empty_definition_id() {
        let i = make_instrument(1, "", "Unknown");
        // split('.'). next() on empty string returns Some("") which is "unknown"-no-match
        // but actually empty string split gives Some("")
        // The code returns Some(part) where part = "" → returns ""
        assert_eq!(i.family(), "");
    }

    #[test]
    fn name_with_set_count_contains_space() {
        let mut i = make_instrument(1, "strings.violin", "Violin");
        i.count = Some(String::from("III"));
        let name = i.name();
        assert!(name.contains(' '));
        assert!(name.ends_with("III"));
    }

    #[test]
    fn set_volume_clamps_to_midi_max() {
        let mut instrument = make_instrument(1, "woodwind.flute", "Flute");

        let clamped = instrument.set_volume(200);

        assert_eq!(clamped, 127);
        assert_eq!(instrument.volume, 127);
    }

    #[test]
    fn set_pan_clamps_to_midi_max() {
        let mut instrument = make_instrument(1, "woodwind.flute", "Flute");

        let clamped = instrument.set_pan(200);

        assert_eq!(clamped, 127);
        assert_eq!(instrument.pan, 127);
    }

    #[test]
    fn set_mute_updates_flag() {
        let mut instrument = make_instrument(1, "woodwind.flute", "Flute");

        instrument.set_mute(true);

        assert!(instrument.mute);
    }

    #[test]
    fn set_solo_updates_flag() {
        let mut instrument = make_instrument(1, "woodwind.flute", "Flute");

        instrument.set_solo(true);

        assert!(instrument.solo);
    }

    #[test]
    fn update_counts_numbers_duplicate_instruments_on_same_player() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10, 11])],
            vec![
                make_instrument(10, "strings.violin", "Violin"),
                make_instrument(11, "strings.violin", "Violin"),
            ],
        );

        score.update_counts();

        assert_eq!(score.instruments.by_id[&10].count.as_deref(), Some("I"));
        assert_eq!(score.instruments.by_id[&11].count.as_deref(), Some("II"));
    }

    #[test]
    fn update_counts_numbers_duplicate_instruments_across_players() {
        let mut score = make_score(
            vec![
                make_player(1, PlayerType::Section, vec![10]),
                make_player(2, PlayerType::Section, vec![11]),
            ],
            vec![
                make_instrument(10, "strings.violin", "Violin"),
                make_instrument(11, "strings.violin", "Violin"),
            ],
        );

        score.update_counts();

        assert_eq!(score.instruments.by_id[&10].count.as_deref(), Some("I"));
        assert_eq!(score.instruments.by_id[&11].count.as_deref(), Some("II"));
    }

    #[test]
    fn update_counts_leaves_single_instrument_unlabelled() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "woodwind.flute", "Flute")],
        );

        score.update_counts();

        assert!(score.instruments.by_id[&10].count.is_none());
    }

    #[test]
    fn update_counts_tracks_each_instrument_type_independently() {
        let mut score = make_score(
            vec![
                make_player(1, PlayerType::Section, vec![10, 12]),
                make_player(2, PlayerType::Section, vec![11, 13]),
            ],
            vec![
                make_instrument(10, "strings.violin", "Violin"),
                make_instrument(11, "strings.violin", "Violin"),
                make_instrument(12, "woodwind.flute", "Flute"),
                make_instrument(13, "woodwind.flute", "Flute"),
            ],
        );

        score.update_counts();

        assert_eq!(score.instruments.by_id[&10].count.as_deref(), Some("I"));
        assert_eq!(score.instruments.by_id[&11].count.as_deref(), Some("II"));
        assert_eq!(score.instruments.by_id[&12].count.as_deref(), Some("I"));
        assert_eq!(score.instruments.by_id[&13].count.as_deref(), Some("II"));
    }

    #[test]
    fn routed_plugin_ids_collect_only_routed_plugins() {
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("a"));
        available_plugin_ids.insert(String::from("b"));

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10, 11, 12])],
            vec![
                make_instrument(10, "strings.violin", "Violin"),
                make_instrument(11, "woodwind.flute", "Flute"),
                make_instrument(12, "brass.horn.f", "Horn"),
            ],
        );

        add_slot(&mut score, 100, "a");
        add_slot(&mut score, 101, "b");
        score
            .assign_instrument_route_to_slot(10, 100)
            .expect("route assignment should succeed");
        score
            .assign_instrument_route_to_slot(12, 101)
            .expect("route assignment should succeed");

        assert_eq!(
            score.routed_plugin_ids(&available_plugin_ids),
            vec![String::from("a"), String::from("b")]
        );
    }

    #[test]
    fn routed_instrument_with_unassigned_slot_resolves_to_attention_needed() {
        let available_plugin_ids = FxHashSet::default();

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "strings.violin", "Violin")],
        );

        add_empty_slot(&mut score, 100);
        score
            .assign_instrument_route_to_slot(10, 100)
            .expect("route assignment should succeed");

        let by_instrument = score.resolved_playback_route_by_instrument_id(&available_plugin_ids);

        assert_eq!(
            by_instrument.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
        assert_eq!(by_instrument.get(&10).unwrap().slot_id, Some(100));
        assert_eq!(by_instrument.get(&10).unwrap().slot_target, None);
        assert_eq!(by_instrument.get(&10).unwrap().effective_plugin_id, None);
    }

    #[test]
    fn removing_instrument_does_not_remove_slot() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "strings.violin", "Violin")],
        );

        add_slot(&mut score, 100, "plugin-a");
        score
            .assign_instrument_route_to_slot(10, 100)
            .expect("route assignment should succeed");

        score.instruments.by_id.remove(&10);

        assert!(score.playback.rack.by_id.contains_key(&100));
    }

    #[test]
    fn create_playback_slot_creates_empty_slot() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "strings.violin", "Violin")],
        );

        score
            .create_playback_slot(100)
            .expect("slot creation should succeed");

        assert_eq!(score.playback.rack.order, vec![100]);
        assert_eq!(score.playback.rack.by_id[&100].target, None);
    }

    #[test]
    fn removing_referenced_slot_clears_routes() {
        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "strings.violin", "Violin")],
        );

        add_slot(&mut score, 100, "plugin-a");
        score
            .assign_instrument_route_to_slot(10, 100)
            .expect("route assignment should succeed");

        score
            .remove_playback_slot(100)
            .expect("removing a referenced slot should succeed");

        assert!(!score.playback.rack.by_id.contains_key(&100));
        assert!(score.playback.rack.order.is_empty());
        assert_eq!(score.instruments.by_id[&10].playback_route, None);
    }

    #[test]
    fn removing_referenced_slot_resolves_instrument_as_unrouted() {
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("plugin-a"));

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10])],
            vec![make_instrument(10, "strings.violin", "Violin")],
        );

        add_slot(&mut score, 100, "plugin-a");
        score
            .assign_instrument_route_to_slot(10, 100)
            .expect("route assignment should succeed");
        score
            .remove_playback_slot(100)
            .expect("removing a referenced slot should succeed");

        let by_instrument = score.resolved_playback_route_by_instrument_id(&available_plugin_ids);

        assert_eq!(
            by_instrument.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::Unrouted
        );
        assert_eq!(by_instrument.get(&10).unwrap().slot_id, None);
    }

    #[test]
    fn resolved_playback_route_map_uses_backend_truth() {
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("plugin-a"));

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10, 11])],
            vec![
                make_instrument(10, "strings.violin", "Violin"),
                make_instrument(11, "woodwind.flute", "Flute"),
            ],
        );

        add_slot(&mut score, 100, "plugin-a");
        score
            .assign_instrument_route_to_slot(10, 100)
            .expect("route assignment should succeed");

        let by_instrument = score.resolved_playback_route_by_instrument_id(&available_plugin_ids);

        assert_eq!(
            by_instrument.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::Ready
        );
        assert_eq!(
            by_instrument.get(&11).unwrap().status,
            ResolvedPlaybackRouteStatus::Unrouted
        );
    }
}
