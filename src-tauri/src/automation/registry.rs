use std::collections::HashSet;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::files::load_project_from_file;
use crate::playback::transport::TransportSharedState;
use crate::score::components::articulation::Articulations;
use crate::score::components::duration::NoteDuration;
use crate::score::components::pitch::Accidental;
use crate::score::entries::key_signature::{
    score_entries_key_signature_clear, score_entries_key_signature_create, KeySignatureMode,
};
use crate::score::entries::time_signature::{
    score_entries_time_signature_clear, score_entries_time_signature_create, TimeSignatureDrawType,
};
use crate::score::entries::tone::score_entries_tone_create;
use crate::score::flows::{score_flows_assign_player, score_flows_create, score_flows_rename};
use crate::score::instruments::defs::score_instruments_get_all_definitions;
use crate::score::instruments::score_instruments_create;
use crate::score::meta::score_meta_title_set;
use crate::score::players::{score_players_assign_instrument, score_players_create, PlayerType};
use crate::ui::{ui_flow_id_set, ui_layout_id_set, ui_tab_set, Tab};
use crate::{
    get_general_state_snapshot, reset_app_state, AppContextState, AudioContextState,
    RendererContextState, SamplerContextState,
};

#[derive(Clone, Serialize)]
pub struct ActionArgMetadata {
    name: &'static str,
    required: bool,
    description: &'static str,
}

#[derive(Clone, Serialize)]
pub struct ActionDefinition {
    name: &'static str,
    description: &'static str,
    args: Vec<ActionArgMetadata>,
    examples: Vec<Value>,
    prerequisites: Vec<Value>,
    #[serde(rename = "entityEffects")]
    entity_effects: Vec<Value>,
    returns: Vec<Value>,
}

#[derive(Serialize)]
struct LookupStaveTracks {
    stave_id: u64,
    master_track_id: u64,
    track_ids: Vec<u64>,
}

#[derive(Serialize)]
struct LookupInstrumentFlowTracks {
    flow_id: u64,
    player_id: u64,
    instrument_id: u64,
    definition_id: String,
    long_name: String,
    short_name: String,
    staves: Vec<LookupStaveTracks>,
}

pub fn action_definitions() -> Vec<ActionDefinition> {
    vec![
        action("reset", "Reset the current score back to a blank default state.", vec![]),
        action(
            "file.loadFromPath",
            "Load a project from a specific file path.",
            vec![arg("filePath", "Absolute path to a .soloc project file.")],
        ),
        action(
            "score.meta.setTitle",
            "Set the score title.",
            vec![arg("title", "Score title.")],
        ),
        action("score.flow.create", "Create a new flow and return its numeric id.", vec![]),
        action(
            "score.flow.rename",
            "Rename an existing flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("title", "Flow title.")],
        ),
        action(
            "score.flow.assignPlayer",
            "Assign a player to a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("player_id", "Numeric player id.")],
        ),
        action(
            "score.player.create",
            "Create a new player. PlayerType values: 0=Solo, 1=Section.",
            vec![arg("player_type", "Numeric PlayerType enum value. Use 0=Solo or 1=Section.")],
        ),
        action(
            "score.player.assignInstrument",
            "Assign an existing instrument to an existing player.",
            vec![arg("player_id", "Numeric player id."), arg("instrument_id", "Numeric instrument id.")],
        ),
        action(
            "score.instrument.create",
            "Create an instrument from a definition id such as keyboard.piano or strings.viola.",
            vec![arg("definition_id", "Stable instrument definition id.")],
        ),
        action(
            "score.instrument.getDefinitions",
            "List available instrument definition ids.",
            vec![],
        ),
        action(
            "score.keySignature.clearAtTick",
            "Clear a key signature entry at a tick in a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("tick", "Tick position.")],
        ),
        action(
            "score.keySignature.create",
            "Create a key signature entry.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("tick", "Tick position."),
                arg("mode", "Numeric KeySignatureMode enum value. Use 0=Major or 1=Minor."),
                arg("offset", "Key signature accidental offset."),
                arg("hidden", "Whether the key signature is hidden."),
            ],
        ),
        action(
            "score.timeSignature.clearAtTick",
            "Clear a time signature entry at a tick in a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("tick", "Tick position.")],
        ),
        action(
            "score.timeSignature.create",
            "Create a time signature entry.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("tick", "Tick position."),
                arg("beats", "Number of beats."),
                arg("beat_type", "Numeric NoteDuration enum value for the beat unit. Common values are 1=Whole, 2=Half, 4=Quarter, 8=Eighth, 16=Sixteenth, 32=ThirtySecond, 64=SixtyFourth, 128=HundredTwentyEighth."),
                arg("draw_type", "Numeric TimeSignatureDrawType enum value. Use 1=Regular, 2=CommonTime, 3=SplitCommonTime, 4=Open. Do not use 0; legacy hidden time signatures were migrated to hidden=true with draw_type=1."),
                arg("hidden", "Whether the entry is hidden."),
            ],
        ),
        action(
            "score.tone.create",
            "Create a tone entry on a track.",
            vec![
                arg("track_id", "Numeric track id."),
                arg("tick", "Tick position."),
                arg("duration", "Tone duration in ticks."),
                arg("pitch_int", "Chromatic pitch integer."),
                arg("accidental", "Accidental enum value or null. Use -2=DoubleFlat, -1=Flat, 0=Natural, 1=Sharp, 2=DoubleSharp."),
                arg("velocity", "Velocity from 0-127."),
                arg("articulations", "Array of Articulation enum values. Use 0=Staccato, 1=Tenuto, 2=Staccatissimo, 3=StaccatoTenuto, 4=Accent, 5=Marcato, 6=Stressed, 7=Unstressed."),
            ],
        ),
        action(
            "score.lookup.getInstrumentTracks",
            "Resolve the flow-local stave master ids and user track ids for an instrument in a specific flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("instrument_id", "Numeric instrument id.")],
        ),
        action(
            "score.lookup.getScoreSlice",
            "Inspect a runtime score slice by optional flow, player, and instrument filters.",
            vec![
                optional_arg("flow_id", "Optional numeric flow id filter."),
                optional_arg("player_id", "Optional numeric player id filter."),
                optional_arg("instrument_id", "Optional numeric instrument id filter."),
            ],
        ),
        action(
            "ui.setTab",
            "Switch the main window tab. Tab values: 0=Setup, 1=Write, 2=Engrave, 3=Sequence, 4=Print.",
            vec![arg("tab", "Numeric Tab enum value. Use 0=Setup, 1=Write, 2=Engrave, 3=Sequence, 4=Print.")],
        ),
        action(
            "ui.setActiveFlowId",
            "Set the active flow in the main window.",
            vec![arg("flow_id", "Numeric flow id.")],
        ),
        action(
            "ui.setActiveLayoutId",
            "Set the active layout in the main window.",
            vec![arg("layout_id", "Numeric layout id.")],
        ),
    ]
}

pub async fn execute_action(app: &AppHandle, name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "reset" => {
            reset_app_state(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<SamplerContextState>(),
                app.state::<AudioContextState>(),
                app.state(),
            )?;
            Ok(Value::Null)
        }
        "file.loadFromPath" => {
            load_project_from_file(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<SamplerContextState>(),
                app.state::<AudioContextState>(),
                app.state(),
                arg_string(args, 0, "filePath")?,
            )?;
            Ok(Value::Null)
        }
        "score.meta.setTitle" => {
            score_meta_title_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "title")?,
            );
            Ok(Value::Null)
        }
        "score.flow.create" => Ok(json!(score_flows_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
        ))),
        "score.flow.rename" => {
            score_flows_rename(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_string(args, 1, "title")?,
            );
            Ok(Value::Null)
        }
        "score.flow.assignPlayer" => Ok(json!(score_flows_assign_player(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "flow_id")?,
            arg_id(args, 1, "player_id")?,
        ))),
        "score.player.create" => Ok(json!(score_players_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_enum::<PlayerType>(args, 0, "player_type")?,
        ))),
        "score.player.assignInstrument" => {
            score_players_assign_instrument(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "player_id")?,
                arg_id(args, 1, "instrument_id")?,
            );
            Ok(Value::Null)
        }
        "score.instrument.create" => Ok(json!(score_instruments_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_string(args, 0, "definition_id")?,
        ))),
        "score.instrument.getDefinitions" => Ok(json!(score_instruments_get_all_definitions())),
        "score.keySignature.clearAtTick" => {
            score_entries_key_signature_clear(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_u64(args, 1, "tick")?,
            );
            Ok(Value::Null)
        }
        "score.keySignature.create" => Ok(json!(score_entries_key_signature_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "flow_id")?,
            arg_u64(args, 1, "tick")?,
            arg_enum::<KeySignatureMode>(args, 2, "mode")?,
            arg_i8(args, 3, "offset")?,
            arg_bool(args, 4, "hidden")?,
        ))),
        "score.timeSignature.clearAtTick" => {
            score_entries_time_signature_clear(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_u64(args, 1, "tick")?,
            );
            Ok(Value::Null)
        }
        "score.timeSignature.create" => Ok(json!(score_entries_time_signature_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "flow_id")?,
            arg_u64(args, 1, "tick")?,
            arg_u8(args, 2, "beats")?,
            arg_enum::<NoteDuration>(args, 3, "beat_type")?,
            arg_enum::<TimeSignatureDrawType>(args, 4, "draw_type")?,
            arg_bool(args, 5, "hidden")?,
        ))),
        "score.tone.create" => Ok(json!(score_entries_tone_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "track_id")?,
            arg_u64(args, 1, "tick")?,
            arg_u64(args, 2, "duration")?,
            arg_u8(args, 3, "pitch_int")?,
            arg_optional_enum::<Accidental>(args, 4)?,
            arg_u8(args, 5, "velocity")?,
            arg_articulations(args, 6)?,
        )?)),
        "score.lookup.getInstrumentTracks" => Ok(json!(derive_instrument_tracks(
            app,
            arg_id(args, 0, "flow_id")?,
            arg_id(args, 1, "instrument_id")?,
        )?)),
        "score.lookup.getScoreSlice" => Ok(derive_score_slice(
            app,
            optional_id(args, 0)?,
            optional_id(args, 1)?,
            optional_id(args, 2)?,
        )?),
        "ui.setTab" => {
            ui_tab_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<Tab>(args, 0, "tab")?,
            );
            Ok(Value::Null)
        }
        "ui.setActiveFlowId" => {
            ui_flow_id_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<TransportSharedState>(),
                arg_id(args, 0, "flow_id")?,
            );
            Ok(Value::Null)
        }
        "ui.setActiveLayoutId" => {
            ui_layout_id_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "layout_id")?,
            );
            Ok(Value::Null)
        }
        _ => Err(format!("Unknown automation action: {name}")),
    }
}

fn action(
    name: &'static str,
    description: &'static str,
    args: Vec<ActionArgMetadata>,
) -> ActionDefinition {
    ActionDefinition {
        name,
        description,
        args,
        examples: vec![],
        prerequisites: vec![],
        entity_effects: vec![],
        returns: vec![],
    }
}

fn arg(name: &'static str, description: &'static str) -> ActionArgMetadata {
    ActionArgMetadata {
        name,
        required: true,
        description,
    }
}

fn optional_arg(name: &'static str, description: &'static str) -> ActionArgMetadata {
    ActionArgMetadata {
        name,
        required: false,
        description,
    }
}

fn derive_instrument_tracks(
    app: &AppHandle,
    flow_id: u64,
    instrument_id: u64,
) -> Result<LookupInstrumentFlowTracks, String> {
    let state = get_general_state_snapshot(app.state::<AppContextState>());
    let flow = state
        .score
        .flows
        .by_id
        .get(&flow_id)
        .ok_or_else(|| format!("Flow {flow_id} not found"))?;
    let instrument = state
        .score
        .instruments
        .by_id
        .get(&instrument_id)
        .ok_or_else(|| format!("Instrument {instrument_id} not found"))?;

    let player_id = state
        .score
        .players
        .order
        .iter()
        .copied()
        .find(|current_player_id| {
            state
                .score
                .players
                .by_id
                .get(current_player_id)
                .is_some_and(|player| player.instrument_ids.contains(&instrument_id))
        })
        .ok_or_else(|| {
            format!("Instrument {instrument_id} is not currently assigned to any player")
        })?;

    if !flow.player_ids.contains(&player_id) {
        return Err(format!(
            "Instrument {instrument_id} is owned by player {player_id} but that player is not assigned to flow {flow_id}"
        ));
    }

    let staves = instrument
        .stave_ids
        .iter()
        .copied()
        .map(|stave_id| {
            let stave = flow.staves.by_id.get(&stave_id).ok_or_else(|| {
                format!("Flow {flow_id} is missing stave {stave_id} for instrument {instrument_id}")
            })?;

            Ok(LookupStaveTracks {
                stave_id,
                master_track_id: stave.master_id,
                track_ids: stave.track_ids.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(LookupInstrumentFlowTracks {
        flow_id,
        player_id,
        instrument_id,
        definition_id: instrument.definition_id.clone(),
        long_name: instrument.long_name.clone(),
        short_name: instrument.short_name.clone(),
        staves,
    })
}

fn derive_score_slice(
    app: &AppHandle,
    flow_id: Option<u64>,
    player_id: Option<u64>,
    instrument_id: Option<u64>,
) -> Result<Value, String> {
    let state = get_general_state_snapshot(app.state::<AppContextState>());

    if let Some(id) = flow_id {
        if !state.score.flows.by_id.contains_key(&id) {
            return Err(format!("Flow {id} not found"));
        }
    }
    if let Some(id) = player_id {
        if !state.score.players.by_id.contains_key(&id) {
            return Err(format!("Player {id} not found"));
        }
    }
    if let Some(id) = instrument_id {
        if !state.score.instruments.by_id.contains_key(&id) {
            return Err(format!("Instrument {id} not found"));
        }
    }

    let player_ids = player_id
        .map(|id| vec![id])
        .unwrap_or_else(|| state.score.players.order.clone());

    let mut owned_instrument_ids = Vec::new();
    for current_player_id in &state.score.players.order {
        if let Some(player) = state.score.players.by_id.get(current_player_id) {
            for current_instrument_id in &player.instrument_ids {
                if !owned_instrument_ids.contains(current_instrument_id) {
                    owned_instrument_ids.push(*current_instrument_id);
                }
            }
        }
    }

    let orphaned_instrument_ids = state
        .score
        .instruments
        .by_id
        .keys()
        .copied()
        .filter(|id| !owned_instrument_ids.contains(id))
        .collect::<Vec<_>>();

    let instrument_ids = instrument_id.map(|id| vec![id]).unwrap_or_else(|| {
        let mut ids = owned_instrument_ids.clone();
        ids.extend(orphaned_instrument_ids);
        ids
    });
    let flow_ids = flow_id
        .map(|id| vec![id])
        .unwrap_or_else(|| state.score.flows.order.clone());

    let players = player_ids
        .iter()
        .map(|current_player_id| {
            let player = &state.score.players.by_id[current_player_id];
            json!({
                "id": player.id,
                "player_type": player.player_type,
                "instrument_ids": player.instrument_ids,
            })
        })
        .collect::<Vec<_>>();
    let instruments = instrument_ids
        .iter()
        .map(|current_instrument_id| {
            let instrument = &state.score.instruments.by_id[current_instrument_id];
            json!({
                "id": instrument.id,
                "definition_id": instrument.definition_id,
                "long_name": instrument.long_name,
                "short_name": instrument.short_name,
                "stave_ids": instrument.stave_ids,
                "assigned_plugin_id": instrument.assigned_plugin_id,
            })
        })
        .collect::<Vec<_>>();
    let flows = flow_ids
        .iter()
        .map(|current_flow_id| {
            let flow = &state.score.flows.by_id[current_flow_id];
            let ordered_player_ids = state
                .score
                .players
                .order
                .iter()
                .copied()
                .filter(|current_player_id| flow.player_ids.contains(current_player_id))
                .collect::<Vec<_>>();

            json!({
                "id": flow.id,
                "title": flow.title,
                "master_track_id": flow.master_id,
                "player_ids": ordered_player_ids,
            })
        })
        .collect::<Vec<_>>();

    let allowed_player_ids = player_ids.iter().copied().collect::<HashSet<_>>();
    let mut flow_instruments = Vec::new();
    for current_flow_id in &flow_ids {
        for current_instrument_id in &instrument_ids {
            if let Ok(tracks) =
                derive_instrument_tracks(app, *current_flow_id, *current_instrument_id)
            {
                if allowed_player_ids.contains(&tracks.player_id) {
                    flow_instruments.push(json!(tracks));
                }
            }
        }
    }

    Ok(json!({
        "players": players,
        "instruments": instruments,
        "flows": flows,
        "flow_instruments": flow_instruments,
    }))
}

fn arg_value<'a>(args: &'a [Value], index: usize, name: &str) -> Result<&'a Value, String> {
    args.get(index)
        .ok_or_else(|| format!("Missing argument {name} at position {index}"))
}

fn arg_string(args: &[Value], index: usize, name: &str) -> Result<String, String> {
    arg_value(args, index, name)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("Argument {name} must be a string"))
}

fn arg_id(args: &[Value], index: usize, name: &str) -> Result<u64, String> {
    arg_value(args, index, name)?
        .as_u64()
        .ok_or_else(|| format!("Argument {name} must be a positive integer"))
}

fn optional_id(args: &[Value], index: usize) -> Result<Option<u64>, String> {
    match args.get(index) {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            format!("Argument at position {index} must be a positive integer or null")
        }),
    }
}

fn arg_u64(args: &[Value], index: usize, name: &str) -> Result<u64, String> {
    arg_id(args, index, name)
}

fn arg_u8(args: &[Value], index: usize, name: &str) -> Result<u8, String> {
    let value = arg_id(args, index, name)?;
    u8::try_from(value).map_err(|_| format!("Argument {name} is out of range for u8"))
}

fn arg_i8(args: &[Value], index: usize, name: &str) -> Result<i8, String> {
    let value = arg_value(args, index, name)?
        .as_i64()
        .ok_or_else(|| format!("Argument {name} must be an integer"))?;
    i8::try_from(value).map_err(|_| format!("Argument {name} is out of range for i8"))
}

fn arg_bool(args: &[Value], index: usize, name: &str) -> Result<bool, String> {
    arg_value(args, index, name)?
        .as_bool()
        .ok_or_else(|| format!("Argument {name} must be a boolean"))
}

fn arg_enum<T>(args: &[Value], index: usize, name: &str) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    serde_json::from_value(arg_value(args, index, name)?.clone())
        .map_err(|error| format!("Argument {name} could not be parsed: {error}"))
}

fn arg_optional_enum<T>(args: &[Value], index: usize) -> Result<Option<T>, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    match args.get(index) {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|error| format!("Argument at position {index} could not be parsed: {error}")),
    }
}

fn arg_articulations(args: &[Value], index: usize) -> Result<Articulations, String> {
    serde_json::from_value(arg_value(args, index, "articulations")?.clone())
        .map_err(|error| format!("Argument articulations could not be parsed: {error}"))
}
