pub mod defs;

use super::{config::AutoCountStyle, Score};
use crate::instructions::FlowRenderRecalculationRequest;
use crate::playback::sampler::sync_sampler_with_score;
use crate::playback::transport::TransportSharedState;
use crate::score::players::PlayerType;
use crate::utils::UniqueId;
use crate::{AppContextState, AudioContextState, RendererContextState, SamplerContextState};
use defs::{get_instrument_definition, InstrumentType};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlaybackAssignmentMode {
    Unassigned = 0,
    Direct = 1,
    Targeted = 2,
    SharedInstance = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlaybackAssignmentTargetMode {
    PluginPart = 0,
    MidiChannel = 1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackAssignmentTarget {
    pub mode: PlaybackAssignmentTargetMode,
    pub index: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackAssignmentIntent {
    pub mode: PlaybackAssignmentMode,
    pub plugin_id: Option<String>,
    pub target: Option<PlaybackAssignmentTarget>,
    pub shared_instance_id: Option<UniqueId>,
}

impl Default for PlaybackAssignmentIntent {
    fn default() -> Self {
        Self::unassigned()
    }
}

impl PlaybackAssignmentIntent {
    pub fn unassigned() -> Self {
        Self {
            mode: PlaybackAssignmentMode::Unassigned,
            plugin_id: None,
            target: None,
            shared_instance_id: None,
        }
    }

    pub fn direct(plugin_id: String) -> Self {
        Self {
            mode: PlaybackAssignmentMode::Direct,
            plugin_id: Some(plugin_id),
            target: None,
            shared_instance_id: None,
        }
    }

    pub fn effective_plugin_id(&self, available_plugin_ids: &FxHashSet<String>) -> Option<&str> {
        let plugin_id = self.plugin_id.as_deref()?;

        match self.mode {
            PlaybackAssignmentMode::Direct if available_plugin_ids.contains(plugin_id) => {
                Some(plugin_id)
            }
            PlaybackAssignmentMode::Unassigned
            | PlaybackAssignmentMode::Direct
            | PlaybackAssignmentMode::Targeted
            | PlaybackAssignmentMode::SharedInstance => None,
        }
    }

    pub fn resolved_state(
        &self,
        available_plugin_ids: &FxHashSet<String>,
    ) -> ResolvedPlaybackAssignment {
        let effective_plugin_id = self
            .effective_plugin_id(available_plugin_ids)
            .map(String::from);
        let status = match self.mode {
            PlaybackAssignmentMode::Unassigned => PlaybackAssignmentStatus::Unassigned,
            PlaybackAssignmentMode::Direct if effective_plugin_id.is_some() => {
                PlaybackAssignmentStatus::Ready
            }
            PlaybackAssignmentMode::Direct
            | PlaybackAssignmentMode::Targeted
            | PlaybackAssignmentMode::SharedInstance => PlaybackAssignmentStatus::NeedsAttention,
        };

        ResolvedPlaybackAssignment {
            status,
            mode: self.mode,
            effective_plugin_id,
            target: self.target.clone(),
            shared_instance_id: self.shared_instance_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlaybackAssignmentStatus {
    Unassigned = 0,
    Ready = 1,
    NeedsAttention = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedPlaybackAssignment {
    pub status: PlaybackAssignmentStatus,
    pub mode: PlaybackAssignmentMode,
    pub effective_plugin_id: Option<String>,
    pub target: Option<PlaybackAssignmentTarget>,
    pub shared_instance_id: Option<UniqueId>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub id: UniqueId,
    pub definition_id: String,
    pub instrument_type: InstrumentType,
    pub long_name: String,
    pub short_name: String,
    pub stave_ids: Vec<UniqueId>,
    pub count: Option<String>,
    pub assignment_intent: PlaybackAssignmentIntent,
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

    pub fn effective_plugin_id(&self, available_plugin_ids: &FxHashSet<String>) -> Option<&str> {
        self.assignment_intent
            .effective_plugin_id(available_plugin_ids)
    }

    pub fn resolved_playback_assignment(
        &self,
        available_plugin_ids: &FxHashSet<String>,
    ) -> ResolvedPlaybackAssignment {
        self.assignment_intent.resolved_state(available_plugin_ids)
    }

    pub fn set_direct_plugin_assignment(&mut self, plugin_id: Option<String>) {
        self.assignment_intent = plugin_id.map_or_else(
            PlaybackAssignmentIntent::unassigned,
            PlaybackAssignmentIntent::direct,
        );
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
    pub fn assigned_plugin_ids(&self, available_plugin_ids: &FxHashSet<String>) -> Vec<String> {
        self.instruments
            .by_id
            .values()
            .filter_map(|instrument| {
                instrument
                    .effective_plugin_id(available_plugin_ids)
                    .map(String::from)
            })
            .collect()
    }

    pub fn resolved_playback_assignment_by_instrument_id(
        &self,
        available_plugin_ids: &FxHashSet<String>,
    ) -> FxHashMap<UniqueId, ResolvedPlaybackAssignment> {
        self.instruments
            .by_id
            .iter()
            .map(|(instrument_id, instrument)| {
                (
                    *instrument_id,
                    instrument.resolved_playback_assignment(available_plugin_ids),
                )
            })
            .collect()
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
        assignment_intent: PlaybackAssignmentIntent::default(),
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
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    instrument_id: UniqueId,
) {
    let mut state = state.lock().unwrap();

    state.score.instruments.by_id.remove(&instrument_id);

    sync_sampler_with_score(
        &app,
        &state.score,
        sampler_state.inner(),
        audio_state.inner(),
        "after instrument removal",
    );

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
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
        instrument.set_direct_plugin_assignment(plugin_id);
    }

    sync_sampler_with_score(
        &app,
        &state.score,
        sampler_state.inner(),
        audio_state.inner(),
        "after plugin assignment",
    );

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_general_state(&app, &mut renderer, None);
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

    let clamped = state
        .score
        .instruments
        .by_id
        .get_mut(&instrument_id)
        .map(|instrument| instrument.set_volume(volume))
        .unwrap_or_else(|| volume.min(127));

    // Update live mix params for in-flight playback
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.volume = clamped;
        }
    }

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

    let clamped = state
        .score
        .instruments
        .by_id
        .get_mut(&instrument_id)
        .map(|instrument| instrument.set_pan(pan))
        .unwrap_or_else(|| pan.min(127));

    // Update live mix params for in-flight playback
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.pan = clamped;
        }
    }

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
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.mute = mute;
        }
    }

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
    if let Ok(mut mix) = transport_shared.mix_params.write() {
        if let Some(p) = mix.get_mut(&instrument_id) {
            p.solo = solo;
        }
    }

    let mut renderer = renderer_state.lock().unwrap();
    state.emit_general_state(&app, &mut renderer, None);
}

#[cfg(test)]
mod tests {
    use super::*;
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
            assignment_intent: PlaybackAssignmentIntent::default(),
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
    fn instrument_assignment_intent_defaults_to_unassigned() {
        let i = make_instrument(1, "woodwind.flute", "Flute");
        assert_eq!(i.assignment_intent, PlaybackAssignmentIntent::unassigned());
    }

    #[test]
    fn effective_plugin_id_reads_direct_assignment() {
        let mut i = make_instrument(1, "woodwind.flute", "Flute");
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("plugin-a"));

        i.set_direct_plugin_assignment(Some(String::from("plugin-a")));

        assert_eq!(
            i.effective_plugin_id(&available_plugin_ids),
            Some("plugin-a")
        );
    }

    #[test]
    fn targeted_assignment_resolves_to_attention_needed() {
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("plugin-a"));

        let i = Instrument {
            assignment_intent: PlaybackAssignmentIntent {
                mode: PlaybackAssignmentMode::Targeted,
                plugin_id: Some(String::from("plugin-a")),
                target: Some(PlaybackAssignmentTarget {
                    mode: PlaybackAssignmentTargetMode::PluginPart,
                    index: 2,
                }),
                shared_instance_id: None,
            },
            ..make_instrument(1, "woodwind.flute", "Flute")
        };

        assert_eq!(
            i.resolved_playback_assignment(&available_plugin_ids).status,
            PlaybackAssignmentStatus::NeedsAttention
        );
        assert_eq!(i.effective_plugin_id(&available_plugin_ids), None);
    }

    #[test]
    fn direct_assignment_without_available_inventory_resolves_to_attention_needed() {
        let i = Instrument {
            assignment_intent: PlaybackAssignmentIntent::direct(String::from("plugin-a")),
            ..make_instrument(1, "woodwind.flute", "Flute")
        };

        let available_plugin_ids = FxHashSet::default();
        let resolved = i.resolved_playback_assignment(&available_plugin_ids);

        assert_eq!(resolved.status, PlaybackAssignmentStatus::NeedsAttention);
        assert_eq!(resolved.effective_plugin_id, None);
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
    fn assigned_plugin_ids_collects_only_assigned_plugins() {
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

        score
            .instruments
            .by_id
            .get_mut(&10)
            .unwrap()
            .set_direct_plugin_assignment(Some(String::from("a")));
        score
            .instruments
            .by_id
            .get_mut(&12)
            .unwrap()
            .set_direct_plugin_assignment(Some(String::from("b")));

        assert_eq!(
            score.assigned_plugin_ids(&available_plugin_ids),
            vec![String::from("a"), String::from("b")]
        );
    }

    #[test]
    fn resolved_playback_assignment_map_uses_backend_truth() {
        let mut available_plugin_ids = FxHashSet::default();
        available_plugin_ids.insert(String::from("plugin-a"));

        let mut score = make_score(
            vec![make_player(1, PlayerType::Section, vec![10, 11])],
            vec![
                make_instrument(10, "strings.violin", "Violin"),
                make_instrument(11, "woodwind.flute", "Flute"),
            ],
        );

        score
            .instruments
            .by_id
            .get_mut(&10)
            .unwrap()
            .set_direct_plugin_assignment(Some(String::from("plugin-a")));

        let by_instrument =
            score.resolved_playback_assignment_by_instrument_id(&available_plugin_ids);

        assert_eq!(
            by_instrument.get(&10).unwrap().status,
            PlaybackAssignmentStatus::Ready
        );
        assert_eq!(
            by_instrument.get(&11).unwrap().status,
            PlaybackAssignmentStatus::Unassigned
        );
    }
}
