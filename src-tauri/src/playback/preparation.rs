use crate::playback::runtime_activation::{
    PlaybackActivationRuntime, SingleInstrumentTrustGateBlocker,
};
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
    pub target: PlaybackRouteTarget,
    pub address: PlaybackRouteAddress,
    pub supported_mix_controls: SupportedPlaybackMixControls,
    pub effective_mix: EffectivePlaybackMix,
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

    Some(PreparedPlaybackTarget {
        instrument_id,
        slot_id: route.slot_id,
        plugin_id: slot_target.plugin_id.clone(),
        bundle_path: slot_target.bundle_path.clone(),
        plugin_state: slot_target.plugin_state.clone(),
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
                resolved_playback_route_by_instrument_id,
                instrument_id,
            )
        })
        .collect()
}

pub fn prepare_supported_playback(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    sample_rate: u32,
) -> PlaybackPreparation {
    let resolved_playback_route_by_instrument_id =
        playback_runtime.resolved_playback_route_by_instrument_id(score);
    let trust_gate = playback_runtime.single_instrument_trust_gate(score);
    let supported_targets = supported_transport_targets(
        score,
        playback_runtime,
        &resolved_playback_route_by_instrument_id,
    );

    PlaybackPreparation {
        sample_rate,
        resolved_playback_route_by_instrument_id,
        supported_targets,
        blockers: trust_gate.blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        effective_playback_mix_for_instrument, prepare_supported_playback, EffectivePlaybackMix,
        PreparedPlaybackTarget,
    };
    use crate::playback::plugin_inventory::{
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
    };
    use crate::playback::runtime_activation::{
        PlaybackActivationRuntime, SingleInstrumentTrustGateBlocker,
    };
    use crate::score::instruments::{defs::InstrumentType, Instrument, Instruments};
    use crate::score::playback::{
        InstrumentPlaybackRoute, PlaybackRackSlot, PlaybackSlotPluginState, PlaybackSlotTarget,
        ResolvedPlaybackRouteStatus, SupportedPlaybackMixControls,
    };
    use crate::score::players::{Player, PlayerType, Players};
    use crate::score::Score;
    use crate::utils::UniqueId;
    use rustc_hash::FxHashMap;
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
}
