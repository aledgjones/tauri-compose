use crate::playback::runtime_activation::{
    PlaybackActivationRuntime, SingleInstrumentTrustGateBlocker, SingleInstrumentTrustGateStatus,
};
use crate::score::playback::{
    PlaybackRouteAddress, PlaybackRouteTarget, PlaybackSlotPluginState, ResolvedPlaybackRoute,
};
use crate::score::Score;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedPlaybackTarget {
    pub instrument_id: UniqueId,
    pub slot_id: UniqueId,
    pub plugin_id: String,
    pub bundle_path: PathBuf,
    pub plugin_state: Option<PlaybackSlotPluginState>,
    pub target: PlaybackRouteTarget,
    pub address: PlaybackRouteAddress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackPreparation {
    pub sample_rate: u32,
    pub resolved_playback_route_by_instrument_id: FxHashMap<UniqueId, ResolvedPlaybackRoute>,
    pub supported_target: Option<PreparedPlaybackTarget>,
    pub blockers: Vec<SingleInstrumentTrustGateBlocker>,
}

impl PlaybackPreparation {
    pub fn blocks_supported_transport_start(&self) -> bool {
        self.supported_target.is_none()
            && self.blockers.iter().any(|blocker| {
                !matches!(
                    blocker,
                    SingleInstrumentTrustGateBlocker::NoRoutedInstrument
                )
            })
    }
}

pub fn prepare_supported_playback(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    sample_rate: u32,
) -> PlaybackPreparation {
    let resolved_playback_route_by_instrument_id =
        playback_runtime.resolved_playback_route_by_instrument_id(score);
    let trust_gate = playback_runtime.single_instrument_trust_gate(score);

    let supported_target = if trust_gate.status == SingleInstrumentTrustGateStatus::TransportReady {
        trust_gate
            .instrument_id
            .zip(trust_gate.route.as_ref())
            .and_then(|(instrument_id, activated_route)| {
                let route = score
                    .instruments
                    .by_id
                    .get(&instrument_id)?
                    .playback_route
                    .as_ref()?;
                let slot_target =
                    playback_runtime.activated_slot_target(activated_route.slot_id)?;

                Some(PreparedPlaybackTarget {
                    instrument_id,
                    slot_id: activated_route.slot_id,
                    plugin_id: activated_route.plugin_id.clone(),
                    bundle_path: activated_route.bundle_path.clone(),
                    plugin_state: slot_target.plugin_state.clone(),
                    target: route.target.clone(),
                    address: route.address.clone(),
                })
            })
    } else {
        None
    };

    PlaybackPreparation {
        sample_rate,
        resolved_playback_route_by_instrument_id,
        supported_target,
        blockers: trust_gate.blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::{prepare_supported_playback, PreparedPlaybackTarget};
    use crate::playback::plugin_inventory::{
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
    };
    use crate::playback::runtime_activation::{
        PlaybackActivationRuntime, SingleInstrumentTrustGateBlocker,
    };
    use crate::score::instruments::{defs::InstrumentType, Instrument, Instruments};
    use crate::score::playback::{
        InstrumentPlaybackRoute, PlaybackRackSlot, PlaybackSlotPluginState, PlaybackSlotTarget,
        ResolvedPlaybackRouteStatus,
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
        assert_eq!(
            preparation.supported_target,
            Some(PreparedPlaybackTarget {
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
            })
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

        assert_eq!(preparation.supported_target, None);
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
    fn prepare_supported_playback_blocks_multiple_routed_instruments() {
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

        assert_eq!(preparation.supported_target, None);
        assert_eq!(
            preparation.blockers,
            vec![
                SingleInstrumentTrustGateBlocker::MultipleRoutedInstruments {
                    instrument_ids: vec![10, 11],
                }
            ]
        );
        assert!(preparation.blocks_supported_transport_start());
    }

    #[test]
    fn prepare_supported_playback_keeps_no_route_state_non_blocking() {
        let score = make_score();
        let runtime = PlaybackActivationRuntime::default();

        let preparation = prepare_supported_playback(&score, &runtime, 48_000);

        assert_eq!(preparation.supported_target, None);
        assert_eq!(
            preparation.blockers,
            vec![SingleInstrumentTrustGateBlocker::NoRoutedInstrument]
        );
        assert!(!preparation.blocks_supported_transport_start());
    }
}
