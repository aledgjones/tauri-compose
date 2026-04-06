use crate::playback::engine::{queue_audition_note, set_slot_mix};
use crate::playback::expression_maps::{ExpressionMapAction, ExpressionMapLibraryState};
use crate::playback::preparation::{
    prepare_supported_playback_with_expression_maps, PlaybackPreparation,
};
use crate::playback::runtime_activation::PlaybackActivationRuntime;
use crate::score::playback::{
    PlaybackRouteAddress, PlaybackRouteTarget, ResolvedPlaybackRouteStatus,
};
use crate::score::Score;
use crate::utils::UniqueId;
use crate::{AppContextState, AudioContextState};
use tauri::{AppHandle, State};

pub const SUPPORTED_AUDITION_DURATION_MS: u32 = 500;
pub const SUPPORTED_AUDITION_VELOCITY: u8 = 96;
const SUPPORTED_AUDITION_RESET_DURATION_MS: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditionArticulation {
    Legato,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditionNote {
    pub pitch: u8,
    pub duration_ms: u32,
    pub velocity: u8,
    pub articulation: AuditionArticulation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedAudition {
    pub preparation: PlaybackPreparation,
    pub slot_id: UniqueId,
    pub reset_notes: Vec<AuditionNote>,
    pub note: AuditionNote,
}

#[tauri::command(rename_all = "snake_case")]
pub fn playback_audition_tone(
    _app: AppHandle,
    app_state: State<'_, AppContextState>,
    audio_state: State<'_, AudioContextState>,
    expression_map_library_state: State<'_, crate::ManagedExpressionMapLibraryContextState>,
    instrument_id: UniqueId,
    pitch_int: u8,
) -> Result<(), String> {
    let (prepared, sample_rate) = {
        let sample_rate = audio_state.lock().unwrap().get_sample_rate();
        let expression_map_library = expression_map_library_state.lock().unwrap().snapshot();

        let app_ctx = app_state.lock().unwrap();
        let prepared = prepare_supported_audition_with_expression_maps(
            &app_ctx.score,
            &app_ctx.playback_activation_runtime,
            &expression_map_library,
            sample_rate,
            instrument_id,
            pitch_int,
        )?;

        (prepared, sample_rate)
    };

    for reset_note in &prepared.reset_notes {
        queue_audition_note(
            prepared.slot_id,
            prepared.preparation.sample_rate.max(sample_rate),
            *reset_note,
        );
    }

    queue_audition_note(
        prepared.slot_id,
        prepared.preparation.sample_rate.max(sample_rate),
        prepared.note,
    );
    if let Some(target) = prepared
        .preparation
        .supported_target_for_instrument(instrument_id)
    {
        set_slot_mix(target.slot_id, target.effective_mix);
    }
    Ok(())
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn prepare_supported_audition(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    sample_rate: u32,
    instrument_id: UniqueId,
    pitch: u8,
) -> Result<PreparedAudition, String> {
    prepare_supported_audition_with_expression_maps(
        score,
        playback_runtime,
        &ExpressionMapLibraryState::default(),
        sample_rate,
        instrument_id,
        pitch,
    )
}

pub fn prepare_supported_audition_with_expression_maps(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    expression_map_library: &ExpressionMapLibraryState,
    sample_rate: u32,
    instrument_id: UniqueId,
    pitch: u8,
) -> Result<PreparedAudition, String> {
    let preparation = prepare_supported_playback_with_expression_maps(
        score,
        playback_runtime,
        expression_map_library,
        sample_rate,
    );

    let Some(resolved_route) = preparation
        .resolved_playback_route_by_instrument_id
        .get(&instrument_id)
    else {
        return Err(format!(
            "No resolved playback route is available for instrument {}.",
            instrument_id
        ));
    };

    if resolved_route.status != ResolvedPlaybackRouteStatus::Ready {
        return Err(format!(
            "Instrument {} audition route is {:?}.",
            instrument_id, resolved_route.status
        ));
    }

    let instrument = score
        .instruments
        .by_id
        .get(&instrument_id)
        .ok_or_else(|| format!("Instrument {} does not exist.", instrument_id))?;
    let route = instrument
        .playback_route
        .as_ref()
        .ok_or_else(|| format!("Instrument {} has no playback route.", instrument_id))?;

    if route.target != PlaybackRouteTarget::plugin()
        || route.address != PlaybackRouteAddress::root()
    {
        return Err(format!(
            "Instrument {} audition route shape is unsupported (target={:?}, address={:?}).",
            instrument_id, route.target, route.address
        ));
    }

    if playback_runtime
        .activated_slot_target(route.slot_id)
        .is_none()
    {
        return Err(format!(
            "Instrument {} audition slot {} is not activated.",
            instrument_id, route.slot_id
        ));
    }

    Ok(PreparedAudition {
        reset_notes: preparation
            .supported_target_for_instrument(instrument_id)
            .and_then(|target| target.expression_map.as_ref())
            .map(expression_map_default_reset_notes)
            .unwrap_or_default(),
        preparation,
        slot_id: route.slot_id,
        note: AuditionNote {
            pitch,
            duration_ms: SUPPORTED_AUDITION_DURATION_MS,
            velocity: SUPPORTED_AUDITION_VELOCITY,
            articulation: AuditionArticulation::Legato,
        },
    })
}

fn expression_map_default_reset_notes(
    expression_map: &crate::playback::expression_maps::ExpressionMap,
) -> Vec<AuditionNote> {
    expression_map
        .default_actions
        .iter()
        .filter_map(|action| match action {
            ExpressionMapAction::KeyswitchNote {
                midi_note,
                velocity,
                ..
            } => Some(AuditionNote {
                pitch: *midi_note,
                duration_ms: SUPPORTED_AUDITION_RESET_DURATION_MS,
                velocity: *velocity,
                articulation: AuditionArticulation::Legato,
            }),
            ExpressionMapAction::ControlChange { .. }
            | ExpressionMapAction::NoOp { .. }
            | ExpressionMapAction::NoteLengthTransform { .. } => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        prepare_supported_audition, prepare_supported_audition_with_expression_maps,
        AuditionArticulation, AuditionNote, SUPPORTED_AUDITION_DURATION_MS,
        SUPPORTED_AUDITION_RESET_DURATION_MS, SUPPORTED_AUDITION_VELOCITY,
    };
    use crate::playback::expression_maps::{
        ExpressionMap, ExpressionMapAction, ExpressionMapLibraryState,
    };
    use crate::playback::plugin_inventory::{
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
    };
    use crate::playback::runtime_activation::PlaybackActivationRuntime;
    use crate::score::instruments::{defs::InstrumentType, Instrument, Instruments};
    use crate::score::playback::{InstrumentPlaybackRoute, PlaybackRackSlot, PlaybackSlotTarget};
    use crate::score::players::{Player, PlayerType, Players};
    use crate::score::Score;
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

    fn assign_slot(score: &mut Score, slot_id: u64, plugin_id: &str) {
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

    fn expression_map_library_with_default_keyswitch() -> ExpressionMapLibraryState {
        ExpressionMapLibraryState {
            order: vec![String::from("default-map")],
            by_id: FxHashMap::from_iter([(
                String::from("default-map"),
                ExpressionMap {
                    id: String::from("default-map"),
                    name: String::from("Default Map"),
                    creator: String::new(),
                    version: 1,
                    default_actions: vec![ExpressionMapAction::KeyswitchNote {
                        id: String::from("default-keyswitch"),
                        midi_note: 24,
                        velocity: 96,
                    }],
                    rules: vec![],
                },
            )]),
        }
    }

    #[test]
    fn prepare_supported_audition_reuses_supported_preparation_and_contract() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let prepared = prepare_supported_audition(&score, &runtime, 96_000, 10, 64)
            .expect("expected a supported audition");

        assert_eq!(prepared.preparation.sample_rate, 96_000);
        assert_eq!(
            prepared
                .preparation
                .supported_target_for_instrument(10)
                .unwrap()
                .instrument_id,
            10
        );
        assert_eq!(
            prepared.note,
            AuditionNote {
                pitch: 64,
                duration_ms: SUPPORTED_AUDITION_DURATION_MS,
                velocity: SUPPORTED_AUDITION_VELOCITY,
                articulation: AuditionArticulation::Legato,
            }
        );
        assert!(prepared.reset_notes.is_empty());
        assert_eq!(prepared.slot_id, 100);
    }

    #[test]
    fn prepare_supported_audition_projects_default_keyswitch_reset_notes() {
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
            .expression_map_id = Some(String::from("default-map"));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let prepared = prepare_supported_audition_with_expression_maps(
            &score,
            &runtime,
            &expression_map_library_with_default_keyswitch(),
            48_000,
            10,
            64,
        )
        .expect("expected linked-map audition preparation");

        assert_eq!(
            prepared.reset_notes,
            vec![AuditionNote {
                pitch: 24,
                duration_ms: SUPPORTED_AUDITION_RESET_DURATION_MS,
                velocity: 96,
                articulation: AuditionArticulation::Legato,
            }]
        );
    }

    #[test]
    fn prepare_supported_audition_preserves_explicit_non_ready_route_states() {
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::default();
        let error = prepare_supported_audition(&score, &runtime, 48_000, 10, 60)
            .expect_err("expected a non-ready route error");

        assert!(error.contains("NeedsAttention"));
        assert!(error.contains("10"));
    }

    #[test]
    fn prepare_supported_audition_allows_requested_ready_instrument_with_multiple_routes() {
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
        let prepared = prepare_supported_audition(&score, &runtime, 48_000, 10, 60)
            .expect("expected ready instrument audition to bypass the transport trust gate");

        assert_eq!(prepared.slot_id, 100);
        assert_eq!(prepared.note.pitch, 60);
    }

    #[test]
    fn prepare_supported_audition_refuses_to_sound_a_different_instrument_than_requested() {
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "/Library/Audio/Plug-Ins/VST3/Plugin A.vst3",
        )]);
        let mut score = make_score();
        assign_slot(&mut score, 100, "plugin-a");
        score.instruments.by_id.get_mut(&10).unwrap().playback_route =
            Some(InstrumentPlaybackRoute::direct(100));

        let runtime = PlaybackActivationRuntime::from_score_and_inventory(&score, &inventory);
        let error = prepare_supported_audition(&score, &runtime, 48_000, 99, 60)
            .expect_err("expected instrument targeting to stay explicit");

        assert!(error.contains("99"));
    }
}
