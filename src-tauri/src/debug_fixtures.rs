use crate::playback::sampler::sync_sampler_with_score;
use crate::score::components::articulation::{Articulation, Articulations};
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::defs::get_instrument_definition;
use crate::score::instruments::Instrument;
use crate::score::players::Player;
use crate::score::players::PlayerType;
use crate::score::tracks::Track;
use crate::ui::{Tab, Ui};
use crate::{AppContextState, AudioContextState, RendererContextState, SamplerContextState, Score};
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScoreFixture {
    pub title: String,
    pub active_tab: Option<Tab>,
    pub instruments: Vec<DebugScoreFixtureInstrument>,
    pub flows: Vec<DebugScoreFixtureFlow>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScoreFixtureInstrument {
    pub id: String,
    pub definition_id: String,
    pub player_type: Option<PlayerType>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScoreFixtureFlow {
    pub title: String,
    pub flow_length: Ticks,
    pub time_signatures: Vec<DebugScoreFixtureTimeSignature>,
    pub parts: Vec<DebugScoreFixturePart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScoreFixturePart {
    pub instrument_id: String,
    pub stave_index: Option<usize>,
    pub tones: Vec<DebugScoreFixtureTone>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScoreFixtureTimeSignature {
    pub tick: Tick,
    pub beats: u8,
    pub beat_type: NoteDuration,
    pub draw_type: Option<TimeSignatureDrawType>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugScoreFixtureTone {
    pub tick: Tick,
    pub duration: Ticks,
    pub pitch_int: u8,
    pub velocity: Option<u8>,
    pub accidental: Option<crate::score::components::pitch::Accidental>,
    pub articulations: Option<Vec<Articulation>>,
}

fn to_articulations(articulations: &Option<Vec<Articulation>>) -> Articulations {
    articulations
        .as_ref()
        .map(|values| Articulations(values.iter().copied().collect::<FxHashSet<_>>()))
        .unwrap_or_default()
}

struct FixtureInstrumentBinding {
    player_id: u64,
    stave_ids: Vec<u64>,
}

#[tauri::command(rename_all = "snake_case")]
pub fn debug_load_score_fixture(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    fixture: DebugScoreFixture,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui = Ui::new();
    state.score = Score::new();
    state.next_id = 1;
    state.current_file_path = None;
    renderer.clear();

    state.score.meta.title = fixture.title.clone();
    let mut instruments_by_fixture_id: FxHashMap<String, FixtureInstrumentBinding> =
        FxHashMap::default();

    for fixture_instrument in fixture.instruments {
        let player_id = state.generate_id();
        state.score.players.order.push(player_id);
        state.score.players.by_id.insert(
            player_id,
            Player::new(
                player_id,
                fixture_instrument.player_type.unwrap_or(PlayerType::Solo),
            ),
        );
        state.assign_player_to_layout(0, player_id);

        let instrument_id = state.generate_id();
        let def =
            get_instrument_definition(&fixture_instrument.definition_id).ok_or_else(|| {
                format!(
                    "Unknown instrument definition: {}",
                    fixture_instrument.definition_id
                )
            })?;
        let stave_ids = def
            .staves
            .iter()
            .map(|_| state.generate_id())
            .collect::<Vec<_>>();

        state.score.instruments.by_id.insert(
            instrument_id,
            Instrument {
                id: instrument_id,
                definition_id: fixture_instrument.definition_id.clone(),
                instrument_type: def.instrument_type,
                long_name: String::from(def.long_name),
                short_name: String::from(def.short_name),
                stave_ids: stave_ids.clone(),
                count: None,
                assigned_plugin_id: None,
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            },
        );

        state
            .score
            .players
            .by_id
            .get_mut(&player_id)
            .ok_or_else(|| "Fixture player was not created".to_string())?
            .instrument_ids
            .push(instrument_id);

        instruments_by_fixture_id.insert(
            fixture_instrument.id,
            FixtureInstrumentBinding {
                player_id,
                stave_ids,
            },
        );
    }

    let mut first_flow_id = None;

    for fixture_flow in fixture.flows {
        let flow_id = state.generate_id();
        let master_id = state.generate_id();
        let mut master = Track::new(master_id);
        let mut assigned_player_ids = FxHashSet::default();

        let mut time_signatures = fixture_flow.time_signatures;
        if !time_signatures.iter().any(|signature| signature.tick == 0) {
            time_signatures.insert(
                0,
                DebugScoreFixtureTimeSignature {
                    tick: 0,
                    beats: 4,
                    beat_type: NoteDuration::Quarter,
                    draw_type: Some(TimeSignatureDrawType::Regular),
                },
            );
        }

        time_signatures.sort_by_key(|signature| signature.tick);

        for signature in time_signatures {
            let time_signature_id = state.generate_id();
            master.insert_entry(Entry::TimeSignature(TimeSignature::new(
                time_signature_id,
                signature.tick,
                signature.beats,
                signature.beat_type,
                signature
                    .draw_type
                    .unwrap_or(TimeSignatureDrawType::Regular),
                None,
            )));
        }

        let mut flow = Flow::new(flow_id, master_id);
        flow.title = fixture_flow.title;
        flow.length = fixture_flow.flow_length;

        state.score.tracks.by_id.insert(master_id, master);
        state.score.flows.order.push(flow_id);
        state.score.flows.by_id.insert(flow_id, flow);
        state.assign_flow_to_layout(0, flow_id);

        for part in fixture_flow.parts {
            let binding = instruments_by_fixture_id
                .get(&part.instrument_id)
                .ok_or_else(|| format!("Unknown fixture instrument: {}", part.instrument_id))?;

            if assigned_player_ids.insert(binding.player_id) {
                state.assign_player_to_flow(flow_id, binding.player_id);
            }

            let stave_id = binding
                .stave_ids
                .get(part.stave_index.unwrap_or(0))
                .copied()
                .ok_or_else(|| {
                    format!(
                        "Instrument '{}' does not have stave index {}",
                        part.instrument_id,
                        part.stave_index.unwrap_or(0)
                    )
                })?;
            let track_id = state
                .score
                .flows
                .by_id
                .get(&flow_id)
                .and_then(|flow| flow.staves.by_id.get(&stave_id))
                .and_then(|stave| stave.track_ids.first().copied())
                .ok_or_else(|| {
                    format!(
                        "Fixture track could not be resolved for instrument '{}'",
                        part.instrument_id
                    )
                })?;

            for tone in part.tones {
                let tone_id = state.generate_id();
                let track = state
                    .score
                    .tracks
                    .by_id
                    .get_mut(&track_id)
                    .ok_or_else(|| "Fixture track is missing".to_string())?;

                track.create_tone(
                    tone_id,
                    tone.tick,
                    tone.duration,
                    tone.pitch_int,
                    tone.accidental,
                    tone.velocity.unwrap_or(100),
                    to_articulations(&tone.articulations),
                );
            }
        }

        if first_flow_id.is_none() {
            first_flow_id = Some(flow_id);
        }
    }

    state.score.update_counts();

    state.ui.layout_id = Some(0);
    state.ui.flow_id = first_flow_id;
    state.ui.set_tab(fixture.active_tab.unwrap_or(Tab::Write));

    let score = state.score.clone();
    state.emit(&app, &mut renderer, true);

    drop(state);
    drop(renderer);

    sync_sampler_with_score(
        &app,
        &score,
        sampler_state.inner(),
        audio_state.inner(),
        "after loading debug fixture",
    );

    Ok(())
}
