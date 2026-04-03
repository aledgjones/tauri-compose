use crate::playback::plugin_editor_poc::render_slot_runtime_audition;
use crate::playback::preparation::{prepare_supported_playback, PlaybackPreparation};
use crate::playback::runtime_activation::PlaybackActivationRuntime;
use crate::score::playback::ResolvedPlaybackRouteStatus;
use crate::score::Score;
use crate::utils::UniqueId;
use crate::{AppContextState, AudioContextState};
use tauri::{AppHandle, State};

pub const SUPPORTED_AUDITION_DURATION_MS: u32 = 1_000;
pub const SUPPORTED_AUDITION_VELOCITY: u8 = 96;

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
    pub note: AuditionNote,
}

#[tauri::command(rename_all = "snake_case")]
pub fn playback_audition_tone(
    app: AppHandle,
    app_state: State<'_, AppContextState>,
    audio_state: State<'_, AudioContextState>,
    instrument_id: UniqueId,
    pitch_int: u8,
) -> Result<(), String> {
    let (prepared, mixer_handle) = {
        let sample_rate;
        let mixer_handle;
        {
            let audio = audio_state.lock().unwrap();
            sample_rate = audio.get_sample_rate();
            mixer_handle = audio.get_mixer_handle().clone();
        }

        let app_ctx = app_state.lock().unwrap();
        let prepared = prepare_supported_audition(
            &app_ctx.score,
            &app_ctx.playback_activation_runtime,
            sample_rate,
            instrument_id,
            pitch_int,
        )?;

        (prepared, mixer_handle)
    };

    let target = prepared
        .preparation
        .supported_target
        .as_ref()
        .ok_or_else(|| String::from("Supported audition target disappeared before playback."))?;

    render_slot_runtime_audition(
        &app,
        target.slot_id,
        prepared.preparation.sample_rate,
        prepared.note,
        mixer_handle,
    )
}

pub fn prepare_supported_audition(
    score: &Score,
    playback_runtime: &PlaybackActivationRuntime,
    sample_rate: u32,
    instrument_id: UniqueId,
    pitch: u8,
) -> Result<PreparedAudition, String> {
    let preparation = prepare_supported_playback(score, playback_runtime, sample_rate);

    if let Some(target) = preparation.supported_target.as_ref() {
        if target.instrument_id == instrument_id {
            return Ok(PreparedAudition {
                preparation,
                note: AuditionNote {
                    pitch,
                    duration_ms: SUPPORTED_AUDITION_DURATION_MS,
                    velocity: SUPPORTED_AUDITION_VELOCITY,
                    articulation: AuditionArticulation::Legato,
                },
            });
        }

        return Err(format!(
            "Audition target instrument {} does not match prepared supported instrument {}.",
            instrument_id, target.instrument_id
        ));
    }

    if let Some(route) = preparation
        .resolved_playback_route_by_instrument_id
        .get(&instrument_id)
    {
        if route.status != ResolvedPlaybackRouteStatus::Ready {
            return Err(format!(
                "Instrument {} audition route is {:?}.",
                instrument_id, route.status
            ));
        }
    }

    if !preparation.blockers.is_empty() {
        return Err(format!("Audition blocked by {:?}.", preparation.blockers));
    }

    Err(format!(
        "No supported audition target is available for instrument {}.",
        instrument_id
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        prepare_supported_audition, AuditionArticulation, AuditionNote,
        SUPPORTED_AUDITION_DURATION_MS, SUPPORTED_AUDITION_VELOCITY,
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
                .supported_target
                .as_ref()
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
    fn prepare_supported_audition_blocks_multiple_routed_instruments_with_transport_gate() {
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
        let error = prepare_supported_audition(&score, &runtime, 48_000, 10, 60)
            .expect_err("expected the single-target trust gate to block audition");

        assert!(error.contains("MultipleRoutedInstruments"));
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
