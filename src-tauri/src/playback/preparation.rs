use crate::playback::expression_maps::{ExpressionMap, ExpressionMapLibraryState};
use crate::playback::runtime_activation::{
    PlaybackActivationRuntime, SingleInstrumentTrustGateBlocker,
};
use crate::playback::tuplet_projection::{collect_track_playback_events, PlaybackToneEvent};
use crate::score::playback::{
    PlaybackRouteAddress, PlaybackRouteTarget, PlaybackSlotPluginState, ResolvedPlaybackRoute,
    ResolvedPlaybackRouteStatus, SupportedPlaybackMixControls,
};
use crate::score::Score;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectivePlaybackMix {
    pub volume: u8,
    pub pan: u8,
    pub muted: bool,
}

impl Default for EffectivePlaybackMix {
    fn default() -> Self {
        Self {
            volume: 127,
            pan: 64,
            muted: false,
        }
    }
}

impl EffectivePlaybackMix {
    pub fn gain_scalar(self) -> f32 {
        f32::from(self.volume) / 127.0
    }
}

fn supported_solo_is_active(
    score: &Score,
    resolved_playback_route_by_instrument_id: &FxHashMap<UniqueId, ResolvedPlaybackRoute>,
) -> bool {
    score
        .instruments
        .by_id
        .iter()
        .any(|(instrument_id, instrument)| {
            let Some(resolved_route) = resolved_playback_route_by_instrument_id.get(instrument_id)
            else {
                return false;
            };

            resolved_route.status == ResolvedPlaybackRouteStatus::Ready
                && resolved_route.supported_mix_controls.solo
                && instrument.solo
        })
}

pub fn effective_playback_mix_for_instrument(
    score: &Score,
    resolved_playback_route_by_instrument_id: &FxHashMap<UniqueId, ResolvedPlaybackRoute>,
    instrument_id: UniqueId,
) -> EffectivePlaybackMix {
    let Some(instrument) = score.instruments.by_id.get(&instrument_id) else {
        return EffectivePlaybackMix::default();
    };

    let supported_mix_controls = resolved_playback_route_by_instrument_id
        .get(&instrument_id)
        .map(|route| route.supported_mix_controls)
        .unwrap_or_default();
    let supported_solo_active =
        supported_solo_is_active(score, resolved_playback_route_by_instrument_id);
    let instrument_is_supported_solo = supported_mix_controls.solo && instrument.solo;

    EffectivePlaybackMix {
        volume: if supported_mix_controls.volume {
            instrument.volume
        } else {
            127
        },
        pan: if supported_mix_controls.pan {
            instrument.pan
        } else {
            64
        },
        muted: (supported_mix_controls.mute && instrument.mute)
            || (supported_solo_active && !instrument_is_supported_solo),
    }
}

pub fn effective_playback_mix_by_slot_id(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
) -> FxHashMap<UniqueId, EffectivePlaybackMix> {
    let resolved_playback_route_by_instrument_id =
        playback_runtime.resolved_playback_route_by_instrument_id(score);

    score
        .instruments
        .by_id
        .iter()
        .filter_map(|(instrument_id, instrument)| {
            let slot_id = instrument.playback_route.as_ref()?.slot_id;
            let resolved_route = resolved_playback_route_by_instrument_id.get(instrument_id)?;

            if resolved_route.status != ResolvedPlaybackRouteStatus::Ready {
                return None;
            }

            Some((
                slot_id,
                effective_playback_mix_for_instrument(
                    score,
                    &resolved_playback_route_by_instrument_id,
                    *instrument_id,
                ),
            ))
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedPlaybackTarget {
    pub instrument_id: UniqueId,
    pub slot_id: UniqueId,
    pub plugin_id: String,
    pub bundle_path: PathBuf,
    pub plugin_state: Option<PlaybackSlotPluginState>,
    pub expression_map: Option<ExpressionMap>,
    pub target: PlaybackRouteTarget,
    pub address: PlaybackRouteAddress,
    pub supported_mix_controls: SupportedPlaybackMixControls,
    pub effective_mix: EffectivePlaybackMix,
}

pub fn prepare_supported_note_events_for_flow(
    score: &Score,
    flow_id: UniqueId,
    preparation: &PlaybackPreparation,
) -> FxHashMap<UniqueId, Vec<PlaybackToneEvent>> {
    let Some(flow) = score.flows.by_id.get(&flow_id) else {
        return FxHashMap::default();
    };

    let supported_targets_by_instrument_id = preparation
        .supported_targets
        .iter()
        .map(|target| (target.instrument_id, target))
        .collect::<FxHashMap<_, _>>();
    if supported_targets_by_instrument_id.is_empty() {
        return FxHashMap::default();
    }

    let mut note_events_by_instrument_id = FxHashMap::default();

    for player_id in &flow.player_ids {
        let Some(player) = score.players.by_id.get(player_id) else {
            continue;
        };

        for instrument_id in &player.instrument_ids {
            let Some(_prepared_target) = supported_targets_by_instrument_id.get(instrument_id)
            else {
                continue;
            };

            let Some(instrument) = score.instruments.by_id.get(instrument_id) else {
                continue;
            };

            let mut note_events = Vec::new();

            for stave_id in &instrument.stave_ids {
                let Some(stave) = flow.staves.by_id.get(stave_id) else {
                    continue;
                };
                let Some(master_track) = score.tracks.by_id.get(&stave.master_id) else {
                    continue;
                };
                let technique_resolver = master_track.effective_playing_technique_resolver();

                for track_id in &stave.track_ids {
                    let Some(track) = score.tracks.by_id.get(track_id) else {
                        continue;
                    };

                    note_events.extend(collect_track_playback_events(
                        track,
                        Some(&technique_resolver),
                    ));
                }
            }

            note_events_by_instrument_id.insert(*instrument_id, note_events);
        }
    }

    note_events_by_instrument_id
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackPreparation {
    pub sample_rate: u32,
    pub resolved_playback_route_by_instrument_id: FxHashMap<UniqueId, ResolvedPlaybackRoute>,
    pub supported_targets: Vec<PreparedPlaybackTarget>,
    pub blockers: Vec<SingleInstrumentTrustGateBlocker>,
}

impl PlaybackPreparation {
    pub fn supported_target_for_instrument(
        &self,
        instrument_id: UniqueId,
    ) -> Option<&PreparedPlaybackTarget> {
        self.supported_targets
            .iter()
            .find(|target| target.instrument_id == instrument_id)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn blocks_supported_transport_start(&self) -> bool {
        self.supported_targets.is_empty()
            && self.blockers.iter().any(|blocker| {
                !matches!(
                    blocker,
                    SingleInstrumentTrustGateBlocker::NoRoutedInstrument
                )
            })
    }
}

fn prepared_supported_target_for_instrument(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    expression_map_library: &ExpressionMapLibraryState,
    resolved_playback_route_by_instrument_id: &FxHashMap<UniqueId, ResolvedPlaybackRoute>,
    instrument_id: UniqueId,
) -> Option<PreparedPlaybackTarget> {
    let route = score
        .instruments
        .by_id
        .get(&instrument_id)?
        .playback_route
        .as_ref()?;

    if route.target != PlaybackRouteTarget::plugin()
        || route.address != PlaybackRouteAddress::root()
    {
        return None;
    }

    let resolved_route = resolved_playback_route_by_instrument_id.get(&instrument_id)?;
    if resolved_route.status != ResolvedPlaybackRouteStatus::Ready {
        return None;
    }

    let slot_target = playback_runtime.activated_slot_target(route.slot_id)?;
    let expression_map = route
        .expression_map_id
        .as_deref()
        .and_then(|expression_map_id| expression_map_library.by_id.get(expression_map_id))
        .cloned();

    Some(PreparedPlaybackTarget {
        instrument_id,
        slot_id: route.slot_id,
        plugin_id: slot_target.plugin_id.clone(),
        bundle_path: slot_target.bundle_path.clone(),
        plugin_state: slot_target.plugin_state.clone(),
        expression_map,
        target: route.target.clone(),
        address: route.address.clone(),
        supported_mix_controls: resolved_route.supported_mix_controls,
        effective_mix: effective_playback_mix_for_instrument(
            score,
            resolved_playback_route_by_instrument_id,
            instrument_id,
        ),
    })
}

fn supported_transport_targets(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    expression_map_library: &ExpressionMapLibraryState,
    resolved_playback_route_by_instrument_id: &FxHashMap<UniqueId, ResolvedPlaybackRoute>,
) -> Vec<PreparedPlaybackTarget> {
    let mut ready_instrument_ids = score
        .instruments
        .by_id
        .keys()
        .copied()
        .filter(|instrument_id| {
            prepared_supported_target_for_instrument(
                score,
                playback_runtime,
                expression_map_library,
                resolved_playback_route_by_instrument_id,
                *instrument_id,
            )
            .is_some()
        })
        .collect::<Vec<_>>();
    ready_instrument_ids.sort_unstable();

    ready_instrument_ids
        .into_iter()
        .filter_map(|instrument_id| {
            prepared_supported_target_for_instrument(
                score,
                playback_runtime,
                expression_map_library,
                resolved_playback_route_by_instrument_id,
                instrument_id,
            )
        })
        .collect()
}

pub fn prepare_supported_playback_with_expression_maps(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    expression_map_library: &ExpressionMapLibraryState,
    sample_rate: u32,
) -> PlaybackPreparation {
    let resolved_playback_route_by_instrument_id =
        playback_runtime.resolved_playback_route_by_instrument_id(score);
    let trust_gate = playback_runtime.single_instrument_trust_gate(score);
    let supported_targets = supported_transport_targets(
        score,
        playback_runtime,
        expression_map_library,
        &resolved_playback_route_by_instrument_id,
    );

    PlaybackPreparation {
        sample_rate,
        resolved_playback_route_by_instrument_id,
        supported_targets,
        blockers: trust_gate.blockers,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn prepare_supported_playback(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    sample_rate: u32,
) -> PlaybackPreparation {
    prepare_supported_playback_with_expression_maps(
        score,
        playback_runtime,
        &ExpressionMapLibraryState::default(),
        sample_rate,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        effective_playback_mix_for_instrument, prepare_supported_note_events_for_flow,
        prepare_supported_playback, prepare_supported_playback_with_expression_maps,
        EffectivePlaybackMix, PreparedPlaybackTarget,
    };
    use crate::playback::expression_maps::{
        resolve_expression_map_playback, ExpressionMap, ExpressionMapAction,
        ExpressionMapCondition, ExpressionMapEqualityComparator, ExpressionMapLibraryState,
        ExpressionMapRule, ResolvedExpressionMapPlaybackSource,
    };
    use crate::playback::plugin_inventory::{
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
    };
    use crate::playback::runtime_activation::{
        PlaybackActivationRuntime, SingleInstrumentTrustGateBlocker,
    };
    use crate::score::components::articulation::Articulation;
    use crate::score::components::duration::NoteDuration;
    use crate::score::entries::playing_technique::{
        PlayingTechnique, PlayingTechniqueExclusivityGroup, PlayingTechniqueKind,
    };
    use crate::score::entries::Entry;
    use crate::score::flows::Flow;
    use crate::score::instruments::{defs::InstrumentType, Instrument, Instruments};
    use crate::score::playback::{
        InstrumentPlaybackRoute, PlaybackRackSlot, PlaybackSlotPluginState, PlaybackSlotTarget,
        ResolvedPlaybackRouteStatus, SupportedPlaybackMixControls,
    };
    use crate::score::players::{Player, PlayerType, Players};
    use crate::score::staves::Stave;
    use crate::score::tracks::Track;
    use crate::score::Score;
    use crate::utils::UniqueId;
    use rustc_hash::{FxHashMap, FxHashSet};
    use std::path::PathBuf;

    fn available_entry(id: &str, bundle_path: &str) -> PluginInventoryEntry {
        PluginInventoryEntry {
            id: id.to_string(),
            name: id.to_string(),
            bundle_path: bundle_path.to_string(),
            binary_path: Some(format!("{bundle_path}/Contents/MacOS/plugin")),
            source_root: String::from("/Library/Audio/Plug-Ins/VST3"),
            availability: PluginInventoryAvailability::Available,
            validation_failure: None,
            editor_host_validation: None,
        }
    }

    fn make_score() -> Score {
        let mut score = Score::new();
        score.players = Players {
            order: vec![1],
            by_id: FxHashMap::from_iter([(1, Player::new(1, PlayerType::Section))]),
        };
        score.players.by_id.get_mut(&1).unwrap().instrument_ids = vec![10];
        score.instruments = Instruments {
            by_id: FxHashMap::from_iter([(
                10,
                Instrument {
                    id: 10,
                    definition_id: String::from("strings.violin"),
                    instrument_type: InstrumentType::Melodic,
                    long_name: String::from("Violin"),
                    short_name: String::from("Vln."),
                    stave_ids: vec![],
                    count: None,
                    playback_route: None,
                    volume: 100,
                    pan: 64,
                    mute: false,
                    solo: false,
                },
            )]),
        };
        score
    }

    fn assign_slot(score: &mut Score, slot_id: UniqueId, plugin_id: &str) {
        score.playback.rack.order.push(slot_id);
        score.playback.rack.by_id.insert(
            slot_id,
            PlaybackRackSlot {
                id: slot_id,
                target: Some(PlaybackSlotTarget {
                    plugin_id: plugin_id.to_string(),
                    plugin_state: None,
                }),
            },
        );
    }

    fn make_flow_score_with_track_and_master(
        track: Track,
        master_track: Track,
    ) -> (Score, UniqueId) {
        let flow_id = 1;
        let player_id = 2;
        let instrument_id = 10;
        let stave_id = 20;
        let master_id = master_track.id;
        let track_id = track.id;
        let mut score = Score::new();
        let mut flow = Flow::new(flow_id, 999);
        flow.player_ids = FxHashSet::from_iter([player_id]);
        flow.staves.by_id.insert(
            stave_id,
            Stave {
                id: stave_id,
                lines: vec![1, 1, 1, 1, 1],
                master_id,
                track_ids: vec![track_id],
            },
        );

        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, flow);
        score.players.order.push(player_id);
        score
            .players
            .by_id
            .insert(player_id, Player::new(player_id, PlayerType::Section));
        score
            .players
            .by_id
            .get_mut(&player_id)
            .unwrap()
            .instrument_ids = vec![instrument_id];
        score.instruments.by_id.insert(
            instrument_id,
            Instrument {
                id: instrument_id,
                definition_id: String::from("strings.violin"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Violin"),
                short_name: String::from("Vln."),
                stave_ids: vec![stave_id],
                count: None,
                playback_route: Some(InstrumentPlaybackRoute::direct(100)),
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            },
        );
        score.tracks.by_id.insert(master_id, master_track);
        score.tracks.by_id.insert(track_id, track);
        assign_slot(&mut score, 100, "plugin-a");

        (score, flow_id)
    }

    fn insert_playing_technique(
        track: &mut Track,
        id: UniqueId,
        tick: u64,
        kind: PlayingTechniqueKind,
    ) {
        track.insert_entry(Entry::PlayingTechnique(PlayingTechnique::new(
            id, tick, kind,
        )));
    }

    fn expression_map_library_with_staccato_map() -> ExpressionMapLibraryState {
        ExpressionMapLibraryState {
            order: vec![String::from("staccato-map")],
            by_id: FxHashMap::from_iter([(
                String::from("staccato-map"),
                ExpressionMap {
                    id: String::from("staccato-map"),
                    name: String::from("Staccato Map"),
                    creator: String::new(),
                    version: 1,
                    default_actions: vec![ExpressionMapAction::NoOp {
                        id: String::from("default-action"),
                    }],
                    rules: vec![ExpressionMapRule {
                        id: String::from("staccato-rule"),
                        name: String::from("Staccato"),
                        conditions: vec![ExpressionMapCondition::Articulation {
                            id: String::from("staccato-condition"),
                            comparator: crate::playback::expression_maps::ExpressionMapEqualityComparator::Equals,
                            articulation: Some(Articulation::Staccato),
                        }],
                        actions: vec![ExpressionMapAction::KeyswitchNote {
                            id: String::from("staccato-action"),
                            midi_note: 24,
                            velocity: 96,
                        }],
                    }, ExpressionMapRule {
                        id: String::from("fallback-note-length"),
                        name: String::from("Fallback"),
                        conditions: vec![ExpressionMapCondition::WrittenNoteLength {
                            id: String::from("fallback-note-length-condition"),
                            comparator: crate::playback::expression_maps::ExpressionMapNoteLengthComparator::GreaterThan,
                            note_duration: NoteDuration::Eighth,
                        }],
                        actions: vec![ExpressionMapAction::NoteLengthTransform {
                            id: String::from("fallback-note-length-action"),
                            ratio_percent: 55,
                        }],
                    }],
                },
            )]),
        }
    }

    fn expression_map_library_with_pizz_map() -> ExpressionMapLibraryState {
        ExpressionMapLibraryState {
            order: vec![String::from("pizz-map")],
            by_id: FxHashMap::from_iter([(
                String::from("pizz-map"),
                ExpressionMap {
                    id: String::from("pizz-map"),
                    name: String::from("Pizz Map"),
                    creator: String::new(),
                    version: 1,
                    default_actions: vec![ExpressionMapAction::NoOp {
                        id: String::from("default-action"),
                    }],
                    rules: vec![ExpressionMapRule {
                        id: String::from("pizz-rule"),
                        name: String::from("Pizz"),
                        conditions: vec![ExpressionMapCondition::PlayingTechnique {
                            id: String::from("pizz-condition"),
                            comparator: ExpressionMapEqualityComparator::Equals,
                            playing_technique: Some(PlayingTechniqueKind::Pizz),
                        }],
                        actions: vec![ExpressionMapAction::KeyswitchNote {
                            id: String::from("pizz-action"),
                            midi_note: 24,
                            velocity: 96,
                        }],
                    }],
                },
            )]),
        }
    }

    fn expression_map_library_with_bowed_map() -> ExpressionMapLibraryState {
        ExpressionMapLibraryState {
            order: vec![String::from("bowed-map")],
            by_id: FxHashMap::from_iter([(
                String::from("bowed-map"),
                ExpressionMap {
                    id: String::from("bowed-map"),
                    name: String::from("Bowed Map"),
                    creator: String::new(),
                    version: 1,
                    default_actions: vec![ExpressionMapAction::NoOp {
                        id: String::from("default-action"),
                    }],
                    rules: vec![
                        ExpressionMapRule {
                            id: String::from("pizz-rule"),
                            name: String::from("Pizz"),
                            conditions: vec![ExpressionMapCondition::PlayingTechnique {
                                id: String::from("pizz-condition"),
                                comparator: ExpressionMapEqualityComparator::Equals,
                                playing_technique: Some(PlayingTechniqueKind::Pizz),
                            }],
                            actions: vec![ExpressionMapAction::KeyswitchNote {
                                id: String::from("pizz-action"),
                                midi_note: 24,
                                velocity: 96,
                            }],
                        },
                        ExpressionMapRule {
                            id: String::from("arco-rule"),
                            name: String::from("Arco"),
                            conditions: vec![ExpressionMapCondition::PlayingTechnique {
                                id: String::from("arco-condition"),
                                comparator: ExpressionMapEqualityComparator::Equals,
                                playing_technique: Some(PlayingTechniqueKind::Arco),
                            }],
                            actions: vec![ExpressionMapAction::KeyswitchNote {
                                id: String::from("arco-action"),
                                midi_note: 25,
                                velocity: 96,
                            }],
                        },
                    ],
                },
            )]),
        }
    }

    #[test]
    fn prepare_supported_playback_returns_single_ready_target_and_sample_rate() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.playback.rack.by_id.get_mut(&100).unwrap().target = Some(PlaybackSlotTarget {
            plugin_id: String::from("plugin-a"),
            plugin_state: Some(PlaybackSlotPluginState {
                vst3_component_state_base64: String::from("AQID"),
            }),
        });
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 96_000);

        assert_eq!(preparation.sample_rate, 96_000);
        assert!(preparation.blockers.is_empty());
        assert_eq!(preparation.supported_targets.len(), 1);
        assert_eq!(
            preparation.supported_target_for_instrument(10),
            Some(&PreparedPlaybackTarget {
                instrument_id: 10,
                slot_id: 100,
                plugin_id: String::from("plugin-a"),
                bundle_path: PathBuf::from("/Library/Audio/Plug-Ins/VST3/Plugin A.vst3"),
                plugin_state: Some(PlaybackSlotPluginState {
                    vst3_component_state_base64: String::from("AQID"),
                }),
                expression_map: None,
                target: score
                    .instruments
                    .by_id
                    .get(&10)
                    .unwrap()
                    .playback_route
                    .as_ref()
                    .unwrap()
                    .target
                    .clone(),
                address: score
                    .instruments
                    .by_id
                    .get(&10)
                    .unwrap()
                    .playback_route
                    .as_ref()
                    .unwrap()
                    .address
                    .clone(),
                supported_mix_controls: SupportedPlaybackMixControls::direct_plugin_root(),
                effective_mix: EffectivePlaybackMix {
                    volume: 100,
                    pan: 64,
                    muted: false,
                },
            })
        );
    }

    #[test]
    fn prepare_supported_playback_keeps_portable_plugin_state_after_score_round_trip() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.playback.rack.by_id.get_mut(&100).unwrap().target = Some(PlaybackSlotTarget {
            plugin_id: String::from("plugin-a"),
            plugin_state: Some(PlaybackSlotPluginState {
                vst3_component_state_base64: String::from("AQID"),
            }),
        });
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let restored_score: Score = serde_json::from_value(serde_json::to_value(&score).unwrap())
            .expect("score round-trip should preserve portable playback intent");

        let runtime =
            PlaybackActivationRuntime::from_score_and_inventory(&restored_score, &inventory);
        let preparation = prepare_supported_playback(&restored_score, &runtime, 48_000);

        assert_eq!(
            preparation
                .supported_target_for_instrument(10)
                .and_then(|target| target.plugin_state.as_ref()),
            Some(&PlaybackSlotPluginState {
                vst3_component_state_base64: String::from("AQID"),
            })
        );
        assert_eq!(
            preparation
                .resolved_playback_route_by_instrument_id
                .get(&10)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::Ready
        );
    }

    #[test]
    fn prepare_supported_playback_resolves_linked_expression_maps_for_ready_targets() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        score
            .instruments
            .by_id
            .get_mut(&10)
            .unwrap()
            .playback_route
            .as_mut()
            .unwrap()
            .expression_map_id = Some(String::from("staccato-map"));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let expression_maps = expression_map_library_with_staccato_map();

        let preparation = prepare_supported_playback_with_expression_maps(
            &score,
            &runtime,
            &expression_maps,
            48_000,
        );

        assert_eq!(
            preparation
                .supported_target_for_instrument(10)
                .and_then(|target| target.expression_map.as_ref())
                .map(|expression_map| expression_map.id.as_str()),
            Some("staccato-map")
        );
    }

    #[test]
    fn prepare_supported_playback_keeps_default_behavior_for_empty_expression_maps() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        score
            .instruments
            .by_id
            .get_mut(&10)
            .unwrap()
            .playback_route
            .as_mut()
            .unwrap()
            .expression_map_id = Some(String::from("empty-map"));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert_eq!(
            preparation
                .supported_target_for_instrument(10)
                .map(|target| (target.slot_id, target.plugin_id.as_str())),
            Some((100, "plugin-a"))
        );
        assert_eq!(
            preparation
                .resolved_playback_route_by_instrument_id
                .get(&10)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::Ready
        );
    }

    #[test]
    fn prepare_supported_playback_reuses_effective_volume_and_mute_from_score_intent() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        score.instruments.by_id.get_mut(&10).unwrap().volume = 32;
        score.instruments.by_id.get_mut(&10).unwrap().mute = true;

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert_eq!(
            preparation
                .supported_target_for_instrument(10)
                .expect("expected supported target")
                .effective_mix,
            EffectivePlaybackMix {
                volume: 32,
                pan: 64,
                muted: true,
            }
        );
    }

    #[test]
    fn effective_playback_mix_for_instrument_mutes_non_soloed_ready_routes_when_any_supported_solo_is_active(
    ) {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        assign_slot(&mut score, 101, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        score
            .players
            .by_id
            .get_mut(&1)
            .unwrap()
            .instrument_ids
            .push(11);
        score.instruments.by_id.insert(
            11,
            Instrument {
                id: 11,
                definition_id: String::from("strings.viola"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Viola"),
                short_name: String::from("Vla."),
                stave_ids: vec![],
                count: None,
                playback_route: Some(InstrumentPlaybackRoute::direct(101)),
                volume: 90,
                pan: 24,
                mute: false,
                solo: true,
            },
        );

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let resolved = runtime.resolved_playback_route_by_instrument_id(&score);

        assert!(effective_playback_mix_for_instrument(&score, &resolved, 10).muted);
        assert!(!effective_playback_mix_for_instrument(&score, &resolved, 11).muted);
        assert_eq!(
            effective_playback_mix_for_instrument(&score, &resolved, 11).pan,
            24
        );
    }

    #[test]
    fn prepare_supported_playback_preserves_explicit_non_ready_states() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert!(preparation.supported_targets.is_empty());
        assert_eq!(
            preparation
                .resolved_playback_route_by_instrument_id
                .get(&10)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
        assert_eq!(
            preparation.blockers,
            vec![SingleInstrumentTrustGateBlocker::RouteNotReady {
                instrument_id: 10,
                slot_id: 100,
                status: ResolvedPlaybackRouteStatus::NeedsAttention,
            }]
        );
    }

    #[test]
    fn prepare_supported_playback_keeps_a_deterministic_ready_target_when_multiple_routes_are_ready(
    ) {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        score
            .players
            .by_id
            .get_mut(&1)
            .unwrap()
            .instrument_ids
            .push(11);
        score.instruments.by_id.insert(
            11,
            Instrument {
                id: 11,
                definition_id: String::from("strings.viola"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Viola"),
                short_name: String::from("Vla."),
                stave_ids: vec![],
                count: None,
                playback_route: Some(InstrumentPlaybackRoute::direct(101)),
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            },
        );
        assign_slot(&mut score, 100, "plugin-a");
        assign_slot(&mut score, 101, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert_eq!(
            preparation
                .supported_targets
                .first()
                .map(|target| target.instrument_id),
            Some(10)
        );
        assert_eq!(
            preparation
                .supported_targets
                .iter()
                .map(|target| target.instrument_id)
                .collect::<Vec<_>>(),
            vec![10, 11]
        );
        assert_eq!(
            preparation.blockers,
            vec![
                SingleInstrumentTrustGateBlocker::MultipleRoutedInstruments {
                    instrument_ids: vec![10, 11],
                }
            ]
        );
        assert!(!preparation.blocks_supported_transport_start());
    }

    #[test]
    fn prepare_supported_playback_keeps_the_one_ready_target_when_other_routes_need_attention() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        score
            .players
            .by_id
            .get_mut(&1)
            .unwrap()
            .instrument_ids
            .push(11);
        score.instruments.by_id.insert(
            11,
            Instrument {
                id: 11,
                definition_id: String::from("strings.viola"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Viola"),
                short_name: String::from("Vla."),
                stave_ids: vec![],
                count: None,
                playback_route: Some(InstrumentPlaybackRoute::direct(101)),
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            },
        );
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        score.playback.rack.order.push(101);
        score.playback.rack.by_id.insert(
            101,
            PlaybackRackSlot {
                id: 101,
                target: None,
            },
        );

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert_eq!(
            preparation
                .supported_targets
                .first()
                .map(|target| target.instrument_id),
            Some(10)
        );
        assert_eq!(
            preparation
                .resolved_playback_route_by_instrument_id
                .get(&11)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
        assert!(!preparation.blocks_supported_transport_start());
    }

    #[test]
    fn shared_slot_routes_fall_back_to_default_effective_mix_when_capability_is_not_unique() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        score
            .players
            .by_id
            .get_mut(&1)
            .unwrap()
            .instrument_ids
            .push(11);
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        score.instruments.by_id.get_mut(&10).unwrap().volume = 20;
        score.instruments.by_id.get_mut(&10).unwrap().mute = true;
        score.instruments.by_id.insert(
            11,
            Instrument {
                id: 11,
                definition_id: String::from("strings.viola"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Viola"),
                short_name: String::from("Vla."),
                stave_ids: vec![],
                count: None,
                playback_route: Some(InstrumentPlaybackRoute::direct(100)),
                volume: 110,
                pan: 12,
                mute: false,
                solo: true,
            },
        );

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let resolved = runtime.resolved_playback_route_by_instrument_id(&score);

        assert_eq!(
            effective_playback_mix_for_instrument(&score, &resolved, 10),
            EffectivePlaybackMix::default()
        );
        assert_eq!(
            effective_playback_mix_for_instrument(&score, &resolved, 11),
            EffectivePlaybackMix::default()
        );
    }

    #[test]
    fn prepare_supported_playback_keeps_no_route_state_non_blocking() {
        let score = make_score();
        let runtime = PlaybackActivationRuntime::default();

        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert!(preparation.supported_targets.is_empty());
        assert_eq!(
            preparation.blockers,
            vec![SingleInstrumentTrustGateBlocker::NoRoutedInstrument]
        );
        assert!(!preparation.blocks_supported_transport_start());
    }

    #[test]
    fn prepare_supported_playback_keeps_the_ready_target_when_other_instruments_are_unrouted() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        score
            .players
            .by_id
            .get_mut(&1)
            .unwrap()
            .instrument_ids
            .push(11);
        score.instruments.by_id.insert(
            11,
            Instrument {
                id: 11,
                definition_id: String::from("strings.viola"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Viola"),
                short_name: String::from("Vla."),
                stave_ids: vec![],
                count: None,
                playback_route: None,
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            },
        );

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert_eq!(
            preparation
                .supported_targets
                .first()
                .map(|target| target.instrument_id),
            Some(10)
        );
        assert!(preparation.blockers.is_empty());
        assert_eq!(
            preparation
                .resolved_playback_route_by_instrument_id
                .get(&10)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::Ready
        );
        assert_eq!(
            preparation
                .resolved_playback_route_by_instrument_id
                .get(&11)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::Unrouted
        );
    }

    #[test]
    fn prepare_supported_note_events_for_flow_derives_master_track_playing_techniques() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut note_track = Track::new(30);
        note_track
            .create_tone(300, 96, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        note_track
            .create_tone(301, 128, 16, 62, None, 96, Default::default(), None)
            .unwrap();

        let mut master_track = Track::new(31);
        insert_playing_technique(&mut master_track, 200, 96, PlayingTechniqueKind::Pizz);
        insert_playing_technique(&mut master_track, 201, 128, PlayingTechniqueKind::Arco);

        let (score, flow_id) = make_flow_score_with_track_and_master(note_track, master_track);
        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let preparation = prepare_supported_playback(&score, &runtime, 48_000);
        let prepared_events = prepare_supported_note_events_for_flow(&score, flow_id, &preparation);
        let note_events = prepared_events
            .get(&10)
            .expect("expected prepared note events for instrument 10");

        assert_eq!(note_events.len(), 2);
        assert_eq!(
            note_events[0]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Pizz)
        );
        assert_eq!(
            note_events[1]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Arco)
        );
    }

    #[test]
    fn prepared_playing_technique_context_reaches_expression_map_rule_resolution() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut note_track = Track::new(30);
        note_track
            .create_tone(300, 96, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let mut master_track = Track::new(31);
        insert_playing_technique(&mut master_track, 200, 96, PlayingTechniqueKind::Pizz);

        let (mut score, flow_id) = make_flow_score_with_track_and_master(note_track, master_track);
        score
            .instruments
            .by_id
            .get_mut(&10)
            .unwrap()
            .playback_route
            .as_mut()
            .unwrap()
            .expression_map_id = Some(String::from("pizz-map"));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let expression_maps = expression_map_library_with_pizz_map();
        let preparation = prepare_supported_playback_with_expression_maps(
            &score,
            &runtime,
            &expression_maps,
            48_000,
        );
        let prepared_events = prepare_supported_note_events_for_flow(&score, flow_id, &preparation);
        let note_event = &prepared_events
            .get(&10)
            .expect("expected prepared note events for instrument 10")[0];
        let expression_map = preparation
            .supported_target_for_instrument(10)
            .and_then(|target| target.expression_map.as_ref())
            .expect("expected linked expression map");

        let resolved =
            resolve_expression_map_playback(expression_map, &note_event.expression_map_context);

        assert_eq!(
            resolved.source,
            ResolvedExpressionMapPlaybackSource::Rule {
                id: String::from("pizz-rule"),
            }
        );
        assert_eq!(
            resolved.actions,
            vec![ExpressionMapAction::KeyswitchNote {
                id: String::from("pizz-action"),
                midi_note: 24,
                velocity: 96,
            }]
        );
    }

    #[test]
    fn prepared_playing_technique_context_switches_bowed_expression_map_rules_on_arco_tick() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut note_track = Track::new(30);
        note_track
            .create_tone(300, 96, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        note_track
            .create_tone(301, 128, 16, 62, None, 96, Default::default(), None)
            .unwrap();

        let mut master_track = Track::new(31);
        insert_playing_technique(&mut master_track, 200, 96, PlayingTechniqueKind::Pizz);
        insert_playing_technique(&mut master_track, 201, 128, PlayingTechniqueKind::Arco);

        let (mut score, flow_id) = make_flow_score_with_track_and_master(note_track, master_track);
        score
            .instruments
            .by_id
            .get_mut(&10)
            .unwrap()
            .playback_route
            .as_mut()
            .unwrap()
            .expression_map_id = Some(String::from("bowed-map"));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let expression_maps = expression_map_library_with_bowed_map();
        let preparation = prepare_supported_playback_with_expression_maps(
            &score,
            &runtime,
            &expression_maps,
            48_000,
        );
        let prepared_events = prepare_supported_note_events_for_flow(&score, flow_id, &preparation);
        let note_events = prepared_events
            .get(&10)
            .expect("expected prepared note events for instrument 10");
        let expression_map = preparation
            .supported_target_for_instrument(10)
            .and_then(|target| target.expression_map.as_ref())
            .expect("expected linked expression map");

        let resolved_pizz =
            resolve_expression_map_playback(expression_map, &note_events[0].expression_map_context);
        let resolved_arco =
            resolve_expression_map_playback(expression_map, &note_events[1].expression_map_context);

        assert_eq!(
            resolved_pizz.source,
            ResolvedExpressionMapPlaybackSource::Rule {
                id: String::from("pizz-rule"),
            }
        );
        assert_eq!(
            resolved_pizz.actions,
            vec![ExpressionMapAction::KeyswitchNote {
                id: String::from("pizz-action"),
                midi_note: 24,
                velocity: 96,
            }]
        );
        assert_eq!(
            resolved_arco.source,
            ResolvedExpressionMapPlaybackSource::Rule {
                id: String::from("arco-rule"),
            }
        );
        assert_eq!(
            resolved_arco.actions,
            vec![ExpressionMapAction::KeyswitchNote {
                id: String::from("arco-action"),
                midi_note: 25,
                velocity: 96,
            }]
        );
    }
}
