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

impl PlaybackActivationRuntime {
    pub fn from_score_and_inventory(score: &Score, inventory: &PluginInventoryContext) -> Self {
        let activated_slot_target_by_slot_id = score
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
            .collect::<FxHashMap<_, _>>();

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

    fn supports_single_target_activation(route: &InstrumentPlaybackRoute) -> bool {
        route.target == PlaybackRouteTarget::plugin()
            && route.address == PlaybackRouteAddress::root()
    }
}

#[cfg(test)]
mod tests {
    use super::PlaybackActivationRuntime;
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
}
