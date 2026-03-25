use crate::playback::plugin_inventory::PluginInventoryContext;
use crate::score::playback::{
    InstrumentPlaybackRoute, PlaybackRouteAddress, PlaybackRouteTarget, ResolvedPlaybackRoute,
    ResolvedPlaybackRouteStatus,
};
use crate::score::Score;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivatedSlotTarget {
    pub plugin_id: String,
    pub bundle_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivatedInstrumentRoute {
    pub slot_id: UniqueId,
    pub plugin_id: String,
    pub bundle_path: PathBuf,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaybackActivationRuntime {
    activated_slot_target_by_slot_id: FxHashMap<UniqueId, ActivatedSlotTarget>,
    activated_instrument_route_by_instrument_id: FxHashMap<UniqueId, ActivatedInstrumentRoute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SingleInstrumentTrustGateStatus {
    TransportReady,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SingleInstrumentTrustGateBlocker {
    NoRoutedInstrument,
    MultipleRoutedInstruments {
        instrument_ids: Vec<UniqueId>,
    },
    UnsupportedRouteShape {
        instrument_id: UniqueId,
        slot_id: UniqueId,
    },
    RouteNotReady {
        instrument_id: UniqueId,
        slot_id: UniqueId,
        status: ResolvedPlaybackRouteStatus,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SingleInstrumentTrustGate {
    pub status: SingleInstrumentTrustGateStatus,
    pub instrument_id: Option<UniqueId>,
    pub route: Option<ActivatedInstrumentRoute>,
    pub blockers: Vec<SingleInstrumentTrustGateBlocker>,
}

impl SingleInstrumentTrustGate {
    pub fn is_transport_ready(&self) -> bool {
        self.status == SingleInstrumentTrustGateStatus::TransportReady
    }
}

impl PlaybackActivationRuntime {
    pub fn expected_slot_targets_from_score_and_inventory(
        score: &Score,
        inventory: &PluginInventoryContext,
    ) -> FxHashMap<UniqueId, ActivatedSlotTarget> {
        score
            .playback
            .rack
            .by_id
            .iter()
            .filter_map(|(slot_id, slot)| {
                let target = slot.target.as_ref()?;
                let bundle_path = inventory
                    .resolve_available_bundle_path(&target.plugin_id)
                    .ok()?;

                Some((
                    *slot_id,
                    ActivatedSlotTarget {
                        plugin_id: target.plugin_id.clone(),
                        bundle_path,
                    },
                ))
            })
            .collect()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_score_and_inventory(score: &Score, inventory: &PluginInventoryContext) -> Self {
        let activated_slot_target_by_slot_id =
            Self::expected_slot_targets_from_score_and_inventory(score, inventory);

        Self::from_live_slot_targets(score, activated_slot_target_by_slot_id)
    }

    pub fn from_live_slot_targets(
        score: &Score,
        activated_slot_target_by_slot_id: FxHashMap<UniqueId, ActivatedSlotTarget>,
    ) -> Self {
        let activated_instrument_route_by_instrument_id = score
            .instruments
            .by_id
            .iter()
            .filter_map(|(instrument_id, instrument)| {
                let route = instrument.playback_route.as_ref()?;
                if !Self::supports_single_target_activation(route) {
                    return None;
                }

                let activated_slot_target = activated_slot_target_by_slot_id.get(&route.slot_id)?;

                Some((
                    *instrument_id,
                    ActivatedInstrumentRoute {
                        slot_id: route.slot_id,
                        plugin_id: activated_slot_target.plugin_id.clone(),
                        bundle_path: activated_slot_target.bundle_path.clone(),
                    },
                ))
            })
            .collect::<FxHashMap<_, _>>();

        Self {
            activated_slot_target_by_slot_id,
            activated_instrument_route_by_instrument_id,
        }
    }

    pub fn effective_plugin_id_for_instrument(&self, instrument_id: UniqueId) -> Option<&str> {
        self.activated_instrument_route_by_instrument_id
            .get(&instrument_id)
            .map(|route| route.plugin_id.as_str())
    }

    pub fn resolved_playback_route_by_instrument_id(
        &self,
        score: &Score,
    ) -> FxHashMap<UniqueId, ResolvedPlaybackRoute> {
        score
            .instruments
            .by_id
            .iter()
            .map(|(instrument_id, instrument)| {
                let resolved = match instrument.playback_route.as_ref() {
                    None => ResolvedPlaybackRoute {
                        status: ResolvedPlaybackRouteStatus::Unrouted,
                        slot_id: None,
                        slot_target: None,
                        effective_plugin_id: None,
                        target: None,
                        address: None,
                    },
                    Some(route) => match score.playback.rack.by_id.get(&route.slot_id) {
                        None => ResolvedPlaybackRoute {
                            status: ResolvedPlaybackRouteStatus::NeedsAttention,
                            slot_id: Some(route.slot_id),
                            slot_target: None,
                            effective_plugin_id: None,
                            target: Some(route.target.clone()),
                            address: Some(route.address.clone()),
                        },
                        Some(slot) => {
                            let activated_route = self
                                .activated_instrument_route_by_instrument_id
                                .get(instrument_id);
                            let status = if slot.target.is_none() {
                                ResolvedPlaybackRouteStatus::Unrouted
                            } else if activated_route.is_some() {
                                ResolvedPlaybackRouteStatus::Ready
                            } else {
                                ResolvedPlaybackRouteStatus::NeedsAttention
                            };

                            ResolvedPlaybackRoute {
                                status,
                                slot_id: Some(route.slot_id),
                                slot_target: slot.target.clone(),
                                effective_plugin_id: activated_route
                                    .map(|activated| activated.plugin_id.clone()),
                                target: Some(route.target.clone()),
                                address: Some(route.address.clone()),
                            }
                        }
                    },
                };

                (*instrument_id, resolved)
            })
            .collect()
    }

    pub fn single_instrument_trust_gate(&self, score: &Score) -> SingleInstrumentTrustGate {
        let routed_instrument_ids = score
            .instruments
            .by_id
            .iter()
            .filter_map(|(instrument_id, instrument)| {
                instrument.playback_route.as_ref().map(|_| *instrument_id)
            })
            .collect::<Vec<_>>();

        if routed_instrument_ids.is_empty() {
            return SingleInstrumentTrustGate {
                status: SingleInstrumentTrustGateStatus::Blocked,
                instrument_id: None,
                route: None,
                blockers: vec![SingleInstrumentTrustGateBlocker::NoRoutedInstrument],
            };
        }

        if routed_instrument_ids.len() > 1 {
            return SingleInstrumentTrustGate {
                status: SingleInstrumentTrustGateStatus::Blocked,
                instrument_id: None,
                route: None,
                blockers: vec![
                    SingleInstrumentTrustGateBlocker::MultipleRoutedInstruments {
                        instrument_ids: routed_instrument_ids,
                    },
                ],
            };
        }

        let instrument_id = routed_instrument_ids[0];
        let route = score.instruments.by_id[&instrument_id]
            .playback_route
            .as_ref()
            .expect("single instrument trust gate requires an existing route");

        if !Self::supports_single_target_activation(route) {
            return SingleInstrumentTrustGate {
                status: SingleInstrumentTrustGateStatus::Blocked,
                instrument_id: Some(instrument_id),
                route: None,
                blockers: vec![SingleInstrumentTrustGateBlocker::UnsupportedRouteShape {
                    instrument_id,
                    slot_id: route.slot_id,
                }],
            };
        }

        let resolved = self.resolved_playback_route_by_instrument_id(score);
        let resolved_route = resolved
            .get(&instrument_id)
            .expect("resolved playback route should exist for routed instrument");

        if resolved_route.status != ResolvedPlaybackRouteStatus::Ready {
            return SingleInstrumentTrustGate {
                status: SingleInstrumentTrustGateStatus::Blocked,
                instrument_id: Some(instrument_id),
                route: None,
                blockers: vec![SingleInstrumentTrustGateBlocker::RouteNotReady {
                    instrument_id,
                    slot_id: route.slot_id,
                    status: resolved_route.status,
                }],
            };
        }

        SingleInstrumentTrustGate {
            status: SingleInstrumentTrustGateStatus::TransportReady,
            instrument_id: Some(instrument_id),
            route: self
                .activated_instrument_route_by_instrument_id
                .get(&instrument_id)
                .cloned(),
            blockers: vec![],
        }
    }

    fn supports_single_target_activation(route: &InstrumentPlaybackRoute) -> bool {
        route.target == PlaybackRouteTarget::plugin()
            && route.address == PlaybackRouteAddress::root()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActivatedSlotTarget, PlaybackActivationRuntime, SingleInstrumentTrustGateBlocker,
        SingleInstrumentTrustGateStatus,
    };
    use crate::playback::plugin_inventory::{
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
    };
    use crate::score::instruments::{defs::InstrumentType, Instrument, Instruments};
    use crate::score::playback::{
        InstrumentPlaybackRoute, PlaybackRackSlot, PlaybackRouteAddress, PlaybackRouteAddressMode,
        PlaybackRouteTarget, PlaybackRouteTargetMode, PlaybackSlotTarget,
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
                }),
            },
        );
    }

    #[test]
    fn activates_single_target_routes_against_available_inventory_entries() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let resolved = runtime.resolved_playback_route_by_instrument_id(&score);

        assert_eq!(
            runtime.effective_plugin_id_for_instrument(10),
            Some("plugin-a")
        );
        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::Ready
        );
        assert_eq!(
            resolved.get(&10).unwrap().effective_plugin_id.as_deref(),
            Some("plugin-a")
        );
    }

    #[test]
    fn slot_assignment_and_unassignment_refresh_runtime_activation() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        score.playback.rack.order.push(100);
        score.playback.rack.by_id.insert(
            100,
            PlaybackRackSlot {
                id: 100,
                target: None,
            },
        );
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let before_assignment =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let before_assignment_status = before_assignment
            .resolved_playback_route_by_instrument_id(&score)
            .get(&10)
            .unwrap()
            .status;
        score.playback.rack.by_id.get_mut(&100).unwrap().target = Some(PlaybackSlotTarget {
            plugin_id: String::from("plugin-a"),
        });
        let after_assignment =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let after_assignment_status = after_assignment
            .resolved_playback_route_by_instrument_id(&score)
            .get(&10)
            .unwrap()
            .status;
        score.playback.rack.by_id.get_mut(&100).unwrap().target = None;
        let after_unassignment =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let after_unassignment_status = after_unassignment
            .resolved_playback_route_by_instrument_id(&score)
            .get(&10)
            .unwrap()
            .status;

        assert_eq!(
            before_assignment_status,
            ResolvedPlaybackRouteStatus::Unrouted
        );
        assert_eq!(after_assignment_status, ResolvedPlaybackRouteStatus::Ready);
        assert_eq!(
            after_unassignment_status,
            ResolvedPlaybackRouteStatus::Unrouted
        );
    }

    #[test]
    fn route_assignment_and_clearing_refresh_runtime_activation() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");

        let before_assignment =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let before_assignment_status = before_assignment
            .resolved_playback_route_by_instrument_id(&score)
            .get(&10)
            .unwrap()
            .status;
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));
        let after_assignment =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let after_assignment_status = after_assignment
            .resolved_playback_route_by_instrument_id(&score)
            .get(&10)
            .unwrap()
            .status;
        score.instruments.by_id.get_mut(&10).unwrap().playback_route = None;
        let after_clearing =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let after_clearing_status = after_clearing
            .resolved_playback_route_by_instrument_id(&score)
            .get(&10)
            .unwrap()
            .status;

        assert_eq!(
            before_assignment_status,
            ResolvedPlaybackRouteStatus::Unrouted
        );
        assert_eq!(after_assignment_status, ResolvedPlaybackRouteStatus::Ready);
        assert_eq!(after_clearing_status, ResolvedPlaybackRouteStatus::Unrouted);
    }

    #[test]
    fn unsupported_multi_target_routes_stay_attention_needed() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute {
                slot_id: 100,
                target: PlaybackRouteTarget {
                    mode: PlaybackRouteTargetMode::PluginPart,
                    index: Some(0),
                },
                address: PlaybackRouteAddress {
                    mode: PlaybackRouteAddressMode::Part,
                    index: Some(0),
                },
            });

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let resolved = runtime.resolved_playback_route_by_instrument_id(&score);

        assert_eq!(runtime.effective_plugin_id_for_instrument(10), None);
        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
    }

    #[test]
    fn degraded_inventory_refresh_clears_previous_runtime_activation() {
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let available_inventory =
            PluginInventoryContext::with_entries_for_tests(vec![available_entry(
                "plugin-a",
                "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
            )]);
        let unavailable_inventory = PluginInventoryContext::with_entries_for_tests(vec![]);

        let active_runtime =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &available_inventory);
        let degraded_runtime =
            PlaybackActivationRuntime::from_score_and_inventory(&score, &unavailable_inventory);
        let degraded = degraded_runtime.resolved_playback_route_by_instrument_id(&score);

        assert_eq!(
            active_runtime.effective_plugin_id_for_instrument(10),
            Some("plugin-a")
        );
        assert_eq!(
            degraded_runtime.effective_plugin_id_for_instrument(10),
            None
        );
        assert_eq!(
            degraded.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
    }

    #[test]
    fn live_slot_targets_drive_readiness_instead_of_slot_intent_alone() {
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let no_live_runtime =
            PlaybackActivationRuntime::from_live_slot_targets(&score, FxHashMap::default());
        let with_live_runtime = PlaybackActivationRuntime::from_live_slot_targets(
            &score,
            FxHashMap::from_iter([(
                100,
                ActivatedSlotTarget {
                    plugin_id: String::from("plugin-a"),
                    bundle_path: PathBuf::from("/Library/Audio/Plug-Ins/VST3/Plugin A.vst3"),
                },
            )]),
        );

        assert_eq!(
            no_live_runtime
                .resolved_playback_route_by_instrument_id(&score)
                .get(&10)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
        assert_eq!(
            with_live_runtime
                .resolved_playback_route_by_instrument_id(&score)
                .get(&10)
                .unwrap()
                .status,
            ResolvedPlaybackRouteStatus::Ready
        );
    }

    #[test]
    fn same_bundle_slots_keep_distinct_live_runtime_identity() {
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

        let runtime = PlaybackActivationRuntime::from_live_slot_targets(
            &score,
            FxHashMap::from_iter([
                (
                    100,
                    ActivatedSlotTarget {
                        plugin_id: String::from("plugin-a"),
                        bundle_path: PathBuf::from("/Library/Audio/Plug-Ins/VST3/Plugin A.vst3"),
                    },
                ),
                (
                    101,
                    ActivatedSlotTarget {
                        plugin_id: String::from("plugin-a"),
                        bundle_path: PathBuf::from("/Library/Audio/Plug-Ins/VST3/Plugin A.vst3"),
                    },
                ),
            ]),
        );

        let resolved = runtime.resolved_playback_route_by_instrument_id(&score);

        assert_eq!(resolved.get(&10).unwrap().slot_id, Some(100));
        assert_eq!(resolved.get(&11).unwrap().slot_id, Some(101));
        assert_eq!(
            resolved.get(&10).unwrap().effective_plugin_id.as_deref(),
            Some("plugin-a")
        );
        assert_eq!(
            resolved.get(&11).unwrap().effective_plugin_id.as_deref(),
            Some("plugin-a")
        );
    }

    #[test]
    fn single_instrument_trust_gate_reports_transport_ready_for_one_ready_direct_route() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let trust_gate = runtime.single_instrument_trust_gate(&score);

        assert_eq!(
            trust_gate.status,
            SingleInstrumentTrustGateStatus::TransportReady
        );
        assert!(trust_gate.is_transport_ready());
        assert_eq!(trust_gate.instrument_id, Some(10));
        assert_eq!(
            trust_gate.route.as_ref().map(|route| route.slot_id),
            Some(100)
        );
        assert_eq!(
            trust_gate
                .route
                .as_ref()
                .map(|route| route.plugin_id.as_str()),
            Some("plugin-a")
        );
        assert!(trust_gate.blockers.is_empty());
    }

    #[test]
    fn single_instrument_trust_gate_blocks_missing_inventory_resolution() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let trust_gate = runtime.single_instrument_trust_gate(&score);

        assert_eq!(trust_gate.status, SingleInstrumentTrustGateStatus::Blocked);
        assert_eq!(
            trust_gate.blockers,
            vec![SingleInstrumentTrustGateBlocker::RouteNotReady {
                instrument_id: 10,
                slot_id: 100,
                status: ResolvedPlaybackRouteStatus::NeedsAttention,
            }]
        );
    }

    #[test]
    fn single_instrument_trust_gate_blocks_unsupported_route_shapes() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute {
                slot_id: 100,
                target: PlaybackRouteTarget {
                    mode: PlaybackRouteTargetMode::PluginPart,
                    index: Some(0),
                },
                address: PlaybackRouteAddress {
                    mode: PlaybackRouteAddressMode::Part,
                    index: Some(0),
                },
            });

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let trust_gate = runtime.single_instrument_trust_gate(&score);

        assert_eq!(trust_gate.status, SingleInstrumentTrustGateStatus::Blocked);
        assert_eq!(
            trust_gate.blockers,
            vec![SingleInstrumentTrustGateBlocker::UnsupportedRouteShape {
                instrument_id: 10,
                slot_id: 100,
            }]
        );
    }

    #[test]
    fn single_instrument_trust_gate_blocks_multiple_routed_instruments() {
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
                playback_route: None,
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
        score.instruments.by_id.get_mut(&11).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(101));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let trust_gate = runtime.single_instrument_trust_gate(&score);

        assert_eq!(trust_gate.status, SingleInstrumentTrustGateStatus::Blocked);
        assert_eq!(
            trust_gate.blockers,
            vec![
                SingleInstrumentTrustGateBlocker::MultipleRoutedInstruments {
                    instrument_ids: vec![10, 11],
                }
            ]
        );
    }
}
