use crate::instructions::FlowRenderRecalculationRequest;
use crate::migrations::{MigrationKind, MIGRATIONS};
use crate::playback::plugin_editor_poc::sync_score_slot_targets_with_runtime_state;
use crate::playback::transport::{
    stop_transport_playback, sync_transport_to_app_state, TransportContext,
};
use crate::score::Score;
use crate::soloc::*;
use crate::ui::sequencer::Tool;
use crate::ui::Ui;
use crate::utils::UniqueId;
use crate::{AppContextState, AudioContextState, RendererContextState};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Db;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::menu::{MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Emitter, Manager, State};

const MAX_RECENT_FILES: usize = 10;
const CURRENT_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_TEMP_FILE_ATTEMPTS: usize = 16;

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    ui: Ui,
    score: Score,
    next_id: UniqueId,
}

#[derive(Clone, Copy)]
struct ChunkEntry {
    type_code: u32,
    flags: u16,
    offset: u64,
    length: u64,
}

#[derive(Debug)]
struct ParsedSolocContainer<'a> {
    app_version: String,
    project_chunk_flags: u16,
    project_payload: &'a [u8],
}

fn get_recent_db(app: &AppHandle) -> Result<Db, String> {
    let db_path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("recent_files");
    sled::open(&db_path).map_err(|e| e.to_string())
}

fn get_project_app_version(data: &Value) -> Option<&str> {
    data.get("score")
        .and_then(|s| s.get("meta"))
        .and_then(|m| m.get("app_version"))
        .and_then(|v| v.as_str())
}

fn set_project_app_version(data: &mut Value, app_version: &str) {
    if let Some(score) = data.get_mut("score") {
        if let Some(meta) = score.get_mut("meta") {
            meta["app_version"] = Value::String(app_version.to_string());
        }
    }
}

fn migrate_project_data(mut data: Value, file_version: &str) -> Result<Value, String> {
    let file_version =
        Version::parse(file_version).map_err(|e| format!("Failed to parse version: {}", e))?;

    for migration in MIGRATIONS.iter() {
        match migration.kind() {
            MigrationKind::Shipped(target_version) => {
                let patch_version = Version::parse(target_version)
                    .map_err(|e| format!("Failed to parse migration version: {}", e))?;

                if file_version < patch_version {
                    data = (migration.migrate())(data)?;
                }
            }
            MigrationKind::PendingNext => {
                data = (migration.migrate())(data)?;
            }
        }
    }

    set_project_app_version(&mut data, CURRENT_APP_VERSION);

    Ok(data)
}

fn validate_project_version_match(
    header_app_version: &str,
    project_data: &Value,
) -> Result<(), String> {
    let project_app_version = get_project_app_version(project_data)
        .ok_or("Invalid .soloc version mismatch between header and project metadata".to_string())?;

    if project_app_version != header_app_version {
        return Err(
            "Invalid .soloc version mismatch between header and project metadata".to_string(),
        );
    }

    Ok(())
}

fn build_project_file(state: &crate::AppContext) -> ProjectFile {
    let mut score = state.score.clone();
    score.meta.app_version = CURRENT_APP_VERSION.to_string();

    ProjectFile {
        ui: state.ui.clone(),
        score,
        next_id: state.next_id,
    }
}

fn parse_chunk_entry(file_data: &[u8], offset: usize) -> ChunkEntry {
    let type_code = u32::from_le_bytes(file_data[offset..offset + 4].try_into().unwrap());
    let flags = u16::from_le_bytes(file_data[offset + 4..offset + 6].try_into().unwrap());
    let chunk_offset = u64::from_le_bytes(file_data[offset + 6..offset + 14].try_into().unwrap());
    let length = u64::from_le_bytes(file_data[offset + 14..offset + 22].try_into().unwrap());

    ChunkEntry {
        type_code,
        flags,
        offset: chunk_offset,
        length,
    }
}

fn validate_chunk_bounds(
    entry: ChunkEntry,
    file_len: u64,
    payload_start: u64,
) -> Result<(), String> {
    if entry.flags & SOLOC_RESERVED_CHUNK_FLAGS_MASK != 0 {
        return Err(format!("Unsupported .soloc chunk flags: {}", entry.flags));
    }

    if entry.offset < payload_start {
        return Err("Chunk points outside file bounds".to_string());
    }

    let chunk_end = entry
        .offset
        .checked_add(entry.length)
        .ok_or("Chunk bounds overflow".to_string())?;

    if chunk_end > file_len {
        return Err("Chunk points outside file bounds".to_string());
    }

    Ok(())
}

fn parse_soloc_container(file_data: &[u8]) -> Result<ParsedSolocContainer<'_>, String> {
    let minimum_header_len = SOLOC_MAGIC.len() + 1 + 1 + 2;
    if file_data.len() < minimum_header_len {
        return Err("File too small to be a valid .soloc file".to_string());
    }

    if &file_data[..SOLOC_MAGIC.len()] != SOLOC_MAGIC {
        return Err("Invalid .soloc magic".to_string());
    }

    let container_version = file_data[SOLOC_MAGIC.len()];
    if container_version != SOLOC_CONTAINER_VERSION {
        return Err(format!(
            "Unsupported .soloc container version: {}",
            container_version
        ));
    }

    let app_version_len = file_data[SOLOC_MAGIC.len() + 1] as usize;
    let app_version_start = SOLOC_MAGIC.len() + 2;
    let chunk_count_offset = app_version_start
        .checked_add(app_version_len)
        .ok_or("Truncated .soloc header".to_string())?;
    let header_end = chunk_count_offset
        .checked_add(2)
        .ok_or("Truncated .soloc header".to_string())?;

    if file_data.len() < header_end {
        return Err("Truncated .soloc header".to_string());
    }

    let app_version = std::str::from_utf8(&file_data[app_version_start..chunk_count_offset])
        .map_err(|e| format!("Invalid .soloc app version encoding: {}", e))?
        .to_string();
    Version::parse(&app_version).map_err(|e| format!("Invalid .soloc app version: {}", e))?;
    let chunk_count = u16::from_le_bytes(
        file_data[chunk_count_offset..header_end]
            .try_into()
            .unwrap(),
    ) as usize;
    let directory_len = chunk_count
        .checked_mul(SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE)
        .ok_or("Truncated .soloc chunk directory".to_string())?;
    let directory_end = header_end
        .checked_add(directory_len)
        .ok_or("Truncated .soloc chunk directory".to_string())?;

    if file_data.len() < directory_end {
        return Err("Truncated .soloc chunk directory".to_string());
    }

    let file_len = u64::try_from(file_data.len())
        .map_err(|_| "File too large to be a valid .soloc file".to_string())?;
    let payload_start =
        u64::try_from(directory_end).map_err(|_| "Chunk points outside file bounds".to_string())?;
    let mut project_chunk = None;

    for entry_offset in (header_end..directory_end).step_by(SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE) {
        let entry = parse_chunk_entry(file_data, entry_offset);
        validate_chunk_bounds(entry, file_len, payload_start)?;

        if entry.type_code == SOLOC_PROJECT_JSON_CHUNK_TYPE {
            if project_chunk.is_some() {
                return Err("Duplicate PROJECT_JSON chunk".to_string());
            }

            project_chunk = Some(entry);
        }
    }

    let project_chunk = project_chunk.ok_or("Missing required PROJECT_JSON chunk".to_string())?;
    let project_chunk_end = project_chunk
        .offset
        .checked_add(project_chunk.length)
        .ok_or("Chunk bounds overflow".to_string())?;
    let project_chunk_start = usize::try_from(project_chunk.offset)
        .map_err(|_| "Chunk points outside file bounds".to_string())?;
    let project_chunk_end = usize::try_from(project_chunk_end)
        .map_err(|_| "Chunk points outside file bounds".to_string())?;

    Ok(ParsedSolocContainer {
        app_version,
        project_chunk_flags: project_chunk.flags,
        project_payload: &file_data[project_chunk_start..project_chunk_end],
    })
}

fn build_temp_file_path(destination: &Path, attempt: usize) -> PathBuf {
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project.soloc");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let temp_file_name = format!(
        ".{}.tmp-{}-{}-{}",
        file_name,
        process::id(),
        timestamp,
        attempt
    );

    destination
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(temp_file_name)
}

fn sync_parent_directory(parent: &Path) -> Result<(), String> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|e| format!("Failed to sync destination directory: {}", e))
}

fn write_file_atomically(destination: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));

    for attempt in 0..MAX_TEMP_FILE_ATTEMPTS {
        let temp_path = build_temp_file_path(destination, attempt);
        let open_result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path);

        let mut temp_file = match open_result {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Failed to create temp file: {}", error)),
        };

        if let Err(error) = temp_file.write_all(contents) {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("Failed to write file: {}", error));
        }

        if let Err(error) = temp_file.sync_all() {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("Failed to sync temp file: {}", error));
        }

        drop(temp_file);

        if let Err(error) = fs::rename(&temp_path, destination) {
            let _ = fs::remove_file(&temp_path);
            return Err(format!("Failed to rename temp file into place: {}", error));
        }

        sync_parent_directory(parent)?;
        return Ok(());
    }

    Err("Failed to create a unique temp file for atomic save".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{prepare_flow_render_plan, FlowRenderRecalculationRequest};
    use crate::migrations::{MigrationKind, MIGRATIONS};
    use crate::playback::plugin_inventory::{
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
    };
    use crate::playback::runtime_activation::PlaybackActivationRuntime;
    use crate::playback::transport::{sync_transport_to_app_state, TransportContext};
    use crate::preferences::Preferences;
    use crate::score::components::{
        articulation::Articulations,
        duration::NoteDuration,
        pitch::{Accidental, Pitch},
        velocity::Velocity,
    };
    use crate::score::entries::{
        tone::Tone,
        tuplet::{Tuplet, TupletEntries, TupletEntry},
        Entry,
    };
    use crate::score::flows::Flow;
    use crate::score::instruments::{defs::InstrumentType, Instrument, Instruments};
    use crate::score::layouts::{Layout, LayoutType};
    use crate::score::playback::{
        InstrumentPlaybackRoute, PlaybackSlotPluginState, ResolvedPlaybackRouteStatus,
    };
    use crate::score::players::{Player, PlayerType, Players};
    use crate::score::tracks::Track;
    use crate::{score::Score, ui::Ui, AppContext};
    use rustc_hash::FxHashMap;
    use serde_json::json;

    fn test_app_version() -> &'static str {
        CURRENT_APP_VERSION
    }

    fn build_test_json(app_version: &str) -> Vec<u8> {
        format!(
            "{{\"score\":{{\"meta\":{{\"app_version\":\"{}\"}}}},\"ui\":{{}},\"next_id\":1}}",
            app_version
        )
        .into_bytes()
    }

    fn build_test_container() -> Vec<u8> {
        let app_version = test_app_version();
        let json_payload = build_test_json(app_version);
        let compressed_payload = compress_project_payload(&json_payload).unwrap();
        build_soloc_container(app_version, &compressed_payload).unwrap()
    }

    fn header_end(app_version: &str) -> usize {
        SOLOC_MAGIC.len() + 1 + 1 + app_version.len() + 2
    }

    fn flags_offset(app_version: &str) -> usize {
        header_end(app_version) + 4
    }

    fn type_offset(app_version: &str) -> usize {
        header_end(app_version)
    }

    fn payload_offset_offset(app_version: &str) -> usize {
        type_offset(app_version) + 4 + 2
    }

    fn pending_migration_registered() -> bool {
        MIGRATIONS
            .iter()
            .any(|migration| matches!(migration.kind(), MigrationKind::PendingNext))
    }

    fn make_preferences() -> Preferences {
        serde_json::from_str::<Preferences>(r#"{"audition":true}"#).unwrap()
    }

    fn make_tuplet_entry(id: UniqueId, tick: u64) -> Entry {
        let mut entries = TupletEntries::new();
        entries.insert_entry(TupletEntry::Tone(Tone::new(
            id + 1,
            16,
            8,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(90),
            Articulations::new(),
        )));

        Entry::Tuplet(Box::new(Tuplet {
            id,
            tick,
            note_count: 3,
            in_space_of: 2,
            base_note_value: NoteDuration::Eighth,
            groupings: vec![3],
            entries,
        }))
    }

    fn build_score_with_visible_flows(
        flow_order: &[UniqueId],
        visible_flow_ids: &[UniqueId],
    ) -> Score {
        let mut score = Score::new();

        score.flows.order = flow_order.to_vec();
        score.flows.by_id.clear();
        score.tracks.by_id.clear();

        for flow_id in flow_order {
            let track_id = flow_id + 100;
            score
                .flows
                .by_id
                .insert(*flow_id, Flow::new(*flow_id, track_id));
            score.tracks.by_id.insert(track_id, Track::new(track_id));
        }

        let mut layout = Layout::new(0, LayoutType::FullScore);
        for flow_id in visible_flow_ids {
            layout.flow_ids.insert(*flow_id);
        }

        score.layouts.order = vec![0];
        score.layouts.by_id.clear();
        score.layouts.by_id.insert(0, layout);

        score
    }

    fn round_trip_project_file(project_file: &ProjectFile) -> ProjectFile {
        let json_data = serde_json::to_vec(project_file).unwrap();
        let compressed_data = compress_project_payload(&json_data).unwrap();
        let container = build_soloc_container(test_app_version(), &compressed_data).unwrap();
        project_file_from_container_bytes(&container).unwrap()
    }

    fn build_soloc_payload_for_project(project_file: &ProjectFile) -> Vec<u8> {
        let json_data = serde_json::to_vec(project_file).unwrap();
        let compressed_data = compress_project_payload(&json_data).unwrap();

        build_soloc_container(test_app_version(), &compressed_data).unwrap()
    }

    fn project_json_from_container(container: &[u8]) -> (String, Vec<u8>, Value) {
        let parsed = parse_soloc_container(container).unwrap();
        let decompressed =
            decompress_project_payload(parsed.project_payload, parsed.project_chunk_flags).unwrap();
        let project_data: Value = serde_json::from_slice(&decompressed).unwrap();

        (parsed.app_version, decompressed, project_data)
    }

    fn project_file_from_container_bytes(file_data: &[u8]) -> Result<ProjectFile, String> {
        let parsed_container = parse_soloc_container(file_data)?;
        let json_data = decompress_project_payload(
            parsed_container.project_payload,
            parsed_container.project_chunk_flags,
        )?;
        let project_data: Value = serde_json::from_slice(&json_data)
            .map_err(|e| format!("Failed to parse project JSON: {}", e))?;
        validate_project_version_match(&parsed_container.app_version, &project_data)?;

        let project_data = migrate_project_data(project_data, &parsed_container.app_version)
            .map_err(|e| format!("Failed to migrate project data: {}", e))?;
        let mut project_file: ProjectFile = serde_json::from_value(project_data)
            .map_err(|e| format!("Failed to deserialize project file: {}", e))?;
        project_file.score.meta.app_version = CURRENT_APP_VERSION.to_string();

        Ok(project_file)
    }

    fn restore_loaded_project_for_test(
        state: &mut AppContext,
        transport_context: &std::sync::Arc<crate::playback::transport::TransportShared>,
        inventory: &PluginInventoryContext,
        project_file: ProjectFile,
        file_path: String,
    ) {
        state.ui = project_file.ui;
        state.ui.sequencer.tool = Tool::Select;
        state.ui.screens.setup.selection = None;
        state.ui.sequencer.selection = None;

        state.score = project_file.score;
        state.next_id = project_file.next_id;
        state.current_file_path = Some(file_path);
        sync_transport_to_app_state(transport_context, state, 0);
        let live_slot_targets =
            PlaybackActivationRuntime::expected_slot_targets_from_score_and_inventory(
                &state.score,
                inventory,
            );
        state.playback_activation_runtime =
            PlaybackActivationRuntime::from_live_slot_targets(&state.score, live_slot_targets);
    }

    fn available_inventory_entry(plugin_id: &str) -> PluginInventoryEntry {
        PluginInventoryEntry {
            id: plugin_id.to_string(),
            name: plugin_id.to_string(),
            bundle_path: format!("/Library/Audio/Plug-Ins/VST3/{plugin_id}.vst3"),
            binary_path: Some(format!(
                "/Library/Audio/Plug-Ins/VST3/{plugin_id}.vst3/Contents/MacOS/{plugin_id}"
            )),
            source_root: String::from("/Library/Audio/Plug-Ins/VST3"),
            availability: PluginInventoryAvailability::Available,
            validation_failure: None,
            editor_host_validation: None,
        }
    }

    fn legacy_non_tuplet_fixture_payload() -> Value {
        let mut fixture = serde_json::from_str::<Value>(
            r#"{
    "next_id": 300,
    "score": {
        "config": {
            "auto_count_styles": {
                "section": 1,
                "solo": 0
            }
        },
        "engraves": {
            "by_layout_type": {
                "0": {
                    "base_note_space": 4,
                    "bracket_single_staves": false,
                    "bracket_style": 1,
                    "bracketing_approach": 1,
                    "frame_padding": {
                        "bottom": 20,
                        "left": 0,
                        "right": 0,
                        "top": 20
                    },
                    "instrument_name_font": {
                        "align": 1,
                        "family": 1,
                        "justify": 2,
                        "size": 1.75
                    },
                    "instrument_name_spacing": 2,
                    "instrument_spacing": 8,
                    "maximum_beam_slant": 0.5,
                    "minimum_note_space": 0.8999999761581421,
                    "minimum_tie_space": 1.600000023841858,
                    "note_space_ratio": 1.409999966621399,
                    "page_padding": {
                        "bottom": 30,
                        "left": 20,
                        "right": 20,
                        "top": 30
                    },
                    "space": 1.5,
                    "stave_spacing": 6,
                    "system_start_padding": 0.75,
                    "systemic_barline_single_instrument_system": true
                },
                "1": {
                    "base_note_space": 4,
                    "bracket_single_staves": false,
                    "bracket_style": 1,
                    "bracketing_approach": 1,
                    "frame_padding": {
                        "bottom": 20,
                        "left": 0,
                        "right": 0,
                        "top": 20
                    },
                    "instrument_name_font": {
                        "align": 1,
                        "family": 1,
                        "justify": 2,
                        "size": 1.75
                    },
                    "instrument_name_spacing": 2,
                    "instrument_spacing": 8,
                    "maximum_beam_slant": 0.5,
                    "minimum_note_space": 0.8999999761581421,
                    "minimum_tie_space": 1.600000023841858,
                    "note_space_ratio": 1.409999966621399,
                    "page_padding": {
                        "bottom": 30,
                        "left": 20,
                        "right": 20,
                        "top": 30
                    },
                    "space": 2,
                    "stave_spacing": 6,
                    "system_start_padding": 0.75,
                    "systemic_barline_single_instrument_system": false
                }
            }
        },
        "flows": {
            "order": [5, 14, 23, 32],
            "by_id": {
                "5": {
                    "id": 5,
                    "title": "Tick 0 Allegro",
                    "player_ids": [1],
                    "length": 256,
                    "master_id": 6,
                    "staves": {
                        "by_id": {
                            "3": {
                                "id": 3,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 8,
                                "track_ids": [10]
                            },
                            "4": {
                                "id": 4,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 11,
                                "track_ids": [13]
                            }
                        }
                    }
                },
                "14": {
                    "id": 14,
                    "title": "Tick 0 Allegro q=120",
                    "player_ids": [1],
                    "length": 256,
                    "master_id": 15,
                    "staves": {
                        "by_id": {
                            "3": {
                                "id": 3,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 17,
                                "track_ids": [19]
                            },
                            "4": {
                                "id": 4,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 20,
                                "track_ids": [22]
                            }
                        }
                    }
                },
                "23": {
                    "id": 23,
                    "title": "Tick 0 Allegro (q=120)",
                    "player_ids": [1],
                    "length": 256,
                    "master_id": 24,
                    "staves": {
                        "by_id": {
                            "3": {
                                "id": 3,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 26,
                                "track_ids": [28]
                            },
                            "4": {
                                "id": 4,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 29,
                                "track_ids": [31]
                            }
                        }
                    }
                },
                "32": {
                    "id": 32,
                    "title": "Metronome + Mid-flow Changes",
                    "player_ids": [1],
                    "length": 256,
                    "master_id": 33,
                    "staves": {
                        "by_id": {
                            "3": {
                                "id": 3,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 35,
                                "track_ids": [37]
                            },
                            "4": {
                                "id": 4,
                                "lines": [1, 1, 1, 1, 1],
                                "master_id": 38,
                                "track_ids": [40]
                            }
                        }
                    }
                }
            }
        },
        "instruments": {
            "by_id": {
                "2": {
                    "id": 2,
                    "definition_id": "keyboard.piano",
                    "instrument_type": 0,
                    "long_name": "Piano",
                    "short_name": "Pno.",
                    "stave_ids": [3, 4],
                    "count": null,
                    "assigned_plugin_id": null,
                    "volume": 100,
                    "pan": 64,
                    "mute": false,
                    "solo": false
                }
            }
        },
        "layouts": {
            "by_id": {
                "0": {
                    "id": 0,
                    "layout_type": 0,
                    "title": "",
                    "player_ids": [1],
                    "flow_ids": [5, 14, 23, 32]
                }
            },
            "order": [0]
        },
        "meta": {
            "app_version": "1.23.3",
            "arranger": "",
            "artist": "",
            "composer": "",
            "composer_dates": "",
            "composition_year": "",
            "copyist": "",
            "copyright": "",
            "created": 1772928000000,
            "dedication": "",
            "editor": "",
            "lyricist": "",
            "other_information": "",
            "publisher": "",
            "subtitle": "Visual Regression Fixture",
            "title": "Tempo Markings",
            "work_number": ""
        },
        "players": {
            "order": [1],
            "by_id": {
                "1": {
                    "id": 1,
                    "player_type": 0,
                    "instrument_ids": [2]
                }
            }
        },
        "tracks": {
            "by_id": {
                "6": {
                    "id": 6,
                    "entries": {
                        "by_tick": {
                            "0": [7, 70]
                        },
                        "by_id": {
                            "7": {
                                "type": "TimeSignature",
                                "id": 7,
                                "tick": 0,
                                "beats": 4,
                                "beat_type": 4,
                                "draw_type": 1,
                                "groupings": [2, 2],
                                "hidden": false
                            },
                            "70": {
                                "type": "Tempo",
                                "id": 70,
                                "tick": 0,
                                "text": "Allegro",
                                "beat_type": 4,
                                "dots": 0,
                                "bpm": 130,
                                "hidden": false,
                                "show_text": true,
                                "show_metronome": false,
                                "parenthesized": false
                            }
                        }
                    }
                },
                "8": {
                    "id": 8,
                    "entries": {
                        "by_tick": {
                            "0": [9]
                        },
                        "by_id": {
                            "9": {
                                "type": "Clef",
                                "id": 9,
                                "tick": 0,
                                "draw_as": 1,
                                "pitch": {
                                    "int": 67,
                                    "accidental": 0
                                },
                                "offset": 2,
                                "hidden": false
                            }
                        }
                    }
                },
                "10": {
                    "id": 10,
                    "entries": {
                        "by_tick": {
                            "0": [100],
                            "32": [101],
                            "64": [102],
                            "96": [103]
                        },
                        "by_id": {
                            "100": {
                                "type": "Tone",
                                "id": 100,
                                "tick": 0,
                                "duration": 32,
                                "pitch": {
                                    "int": 60,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "101": {
                                "type": "Tone",
                                "id": 101,
                                "tick": 32,
                                "duration": 32,
                                "pitch": {
                                    "int": 62,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "102": {
                                "type": "Tone",
                                "id": 102,
                                "tick": 64,
                                "duration": 32,
                                "pitch": {
                                    "int": 64,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "103": {
                                "type": "Tone",
                                "id": 103,
                                "tick": 96,
                                "duration": 32,
                                "pitch": {
                                    "int": 65,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            }
                        }
                    }
                },
                "11": {
                    "id": 11,
                    "entries": {
                        "by_tick": {
                            "0": [12]
                        },
                        "by_id": {
                            "12": {
                                "type": "Clef",
                                "id": 12,
                                "tick": 0,
                                "draw_as": 2,
                                "pitch": {
                                    "int": 53,
                                    "accidental": 0
                                },
                                "offset": -2,
                                "hidden": false
                            }
                        }
                    }
                },
                "13": {
                    "id": 13,
                    "entries": {
                        "by_tick": {},
                        "by_id": {}
                    }
                },
                "15": {
                    "id": 15,
                    "entries": {
                        "by_tick": {
                            "0": [16, 71]
                        },
                        "by_id": {
                            "16": {
                                "type": "TimeSignature",
                                "id": 16,
                                "tick": 0,
                                "beats": 4,
                                "beat_type": 4,
                                "draw_type": 1,
                                "groupings": [2, 2],
                                "hidden": false
                            },
                            "71": {
                                "type": "Tempo",
                                "id": 71,
                                "tick": 0,
                                "text": "Allegro",
                                "beat_type": 4,
                                "dots": 0,
                                "bpm": 120,
                                "hidden": false,
                                "show_text": true,
                                "show_metronome": true,
                                "parenthesized": false
                            }
                        }
                    }
                },
                "17": {
                    "id": 17,
                    "entries": {
                        "by_tick": {
                            "0": [18]
                        },
                        "by_id": {
                            "18": {
                                "type": "Clef",
                                "id": 18,
                                "tick": 0,
                                "draw_as": 1,
                                "pitch": {
                                    "int": 67,
                                    "accidental": 0
                                },
                                "offset": 2,
                                "hidden": false
                            }
                        }
                    }
                },
                "19": {
                    "id": 19,
                    "entries": {
                        "by_tick": {
                            "0": [104],
                            "32": [105],
                            "64": [106],
                            "96": [107]
                        },
                        "by_id": {
                            "104": {
                                "type": "Tone",
                                "id": 104,
                                "tick": 0,
                                "duration": 32,
                                "pitch": {
                                    "int": 67,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "105": {
                                "type": "Tone",
                                "id": 105,
                                "tick": 32,
                                "duration": 32,
                                "pitch": {
                                    "int": 69,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "106": {
                                "type": "Tone",
                                "id": 106,
                                "tick": 64,
                                "duration": 32,
                                "pitch": {
                                    "int": 71,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "107": {
                                "type": "Tone",
                                "id": 107,
                                "tick": 96,
                                "duration": 32,
                                "pitch": {
                                    "int": 72,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            }
                        }
                    }
                },
                "20": {
                    "id": 20,
                    "entries": {
                        "by_tick": {
                            "0": [21]
                        },
                        "by_id": {
                            "21": {
                                "type": "Clef",
                                "id": 21,
                                "tick": 0,
                                "draw_as": 2,
                                "pitch": {
                                    "int": 53,
                                    "accidental": 0
                                },
                                "offset": -2,
                                "hidden": false
                            }
                        }
                    }
                },
                "22": {
                    "id": 22,
                    "entries": {
                        "by_tick": {},
                        "by_id": {}
                    }
                },
                "24": {
                    "id": 24,
                    "entries": {
                        "by_tick": {
                            "0": [25, 72]
                        },
                        "by_id": {
                            "25": {
                                "type": "TimeSignature",
                                "id": 25,
                                "tick": 0,
                                "beats": 4,
                                "beat_type": 4,
                                "draw_type": 1,
                                "groupings": [2, 2],
                                "hidden": false
                            },
                            "72": {
                                "type": "Tempo",
                                "id": 72,
                                "tick": 0,
                                "text": "Allegro",
                                "beat_type": 4,
                                "dots": 0,
                                "bpm": 120,
                                "hidden": false,
                                "show_text": true,
                                "show_metronome": true,
                                "parenthesized": true
                            }
                        }
                    }
                },
                "26": {
                    "id": 26,
                    "entries": {
                        "by_tick": {
                            "0": [27]
                        },
                        "by_id": {
                            "27": {
                                "type": "Clef",
                                "id": 27,
                                "tick": 0,
                                "draw_as": 1,
                                "pitch": {
                                    "int": 67,
                                    "accidental": 0
                                },
                                "offset": 2,
                                "hidden": false
                            }
                        }
                    }
                },
                "28": {
                    "id": 28,
                    "entries": {
                        "by_tick": {
                            "0": [108],
                            "32": [109],
                            "64": [110],
                            "96": [111]
                        },
                        "by_id": {
                            "108": {
                                "type": "Tone",
                                "id": 108,
                                "tick": 0,
                                "duration": 32,
                                "pitch": {
                                    "int": 72,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "109": {
                                "type": "Tone",
                                "id": 109,
                                "tick": 32,
                                "duration": 32,
                                "pitch": {
                                    "int": 74,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "110": {
                                "type": "Tone",
                                "id": 110,
                                "tick": 64,
                                "duration": 32,
                                "pitch": {
                                    "int": 76,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "111": {
                                "type": "Tone",
                                "id": 111,
                                "tick": 96,
                                "duration": 32,
                                "pitch": {
                                    "int": 77,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            }
                        }
                    }
                },
                "29": {
                    "id": 29,
                    "entries": {
                        "by_tick": {
                            "0": [30]
                        },
                        "by_id": {
                            "30": {
                                "type": "Clef",
                                "id": 30,
                                "tick": 0,
                                "draw_as": 2,
                                "pitch": {
                                    "int": 53,
                                    "accidental": 0
                                },
                                "offset": -2,
                                "hidden": false
                            }
                        }
                    }
                },
                "31": {
                    "id": 31,
                    "entries": {
                        "by_tick": {},
                        "by_id": {}
                    }
                },
                "33": {
                    "id": 33,
                    "entries": {
                        "by_tick": {
                            "0": [34, 73],
                            "64": [76],
                            "128": [74],
                            "192": [75]
                        },
                        "by_id": {
                            "34": {
                                "type": "TimeSignature",
                                "id": 34,
                                "tick": 0,
                                "beats": 4,
                                "beat_type": 4,
                                "draw_type": 1,
                                "groupings": [2, 2],
                                "hidden": false
                            },
                            "73": {
                                "type": "Tempo",
                                "id": 73,
                                "tick": 0,
                                "text": null,
                                "beat_type": 4,
                                "dots": 0,
                                "bpm": 120,
                                "hidden": false,
                                "show_text": false,
                                "show_metronome": true,
                                "parenthesized": false
                            },
                            "74": {
                                "type": "Tempo",
                                "id": 74,
                                "tick": 128,
                                "text": "Moderato",
                                "beat_type": 4,
                                "dots": 0,
                                "bpm": 110,
                                "hidden": false,
                                "show_text": true,
                                "show_metronome": false,
                                "parenthesized": false
                            },
                            "75": {
                                "type": "Tempo",
                                "id": 75,
                                "tick": 192,
                                "text": "Presto",
                                "beat_type": 4,
                                "dots": 0,
                                "bpm": 180,
                                "hidden": false,
                                "show_text": true,
                                "show_metronome": false,
                                "parenthesized": false
                            },
                            "76": {
                                "type": "Tempo",
                                "id": 76,
                                "tick": 64,
                                "text": "Largo",
                                "beat_type": 4,
                                "dots": 0,
                                "bpm": 50,
                                "hidden": false,
                                "show_text": true,
                                "show_metronome": false,
                                "parenthesized": false
                            }
                        }
                    }
                },
                "35": {
                    "id": 35,
                    "entries": {
                        "by_tick": {
                            "0": [36]
                        },
                        "by_id": {
                            "36": {
                                "type": "Clef",
                                "id": 36,
                                "tick": 0,
                                "draw_as": 1,
                                "pitch": {
                                    "int": 67,
                                    "accidental": 0
                                },
                                "offset": 2,
                                "hidden": false
                            }
                        }
                    }
                },
                "37": {
                    "id": 37,
                    "entries": {
                        "by_tick": {
                            "0": [112],
                            "64": [113],
                            "128": [114],
                            "192": [115]
                        },
                        "by_id": {
                            "112": {
                                "type": "Tone",
                                "id": 112,
                                "tick": 0,
                                "duration": 32,
                                "pitch": {
                                    "int": 60,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "113": {
                                "type": "Tone",
                                "id": 113,
                                "tick": 64,
                                "duration": 32,
                                "pitch": {
                                    "int": 64,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "114": {
                                "type": "Tone",
                                "id": 114,
                                "tick": 128,
                                "duration": 32,
                                "pitch": {
                                    "int": 67,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            },
                            "115": {
                                "type": "Tone",
                                "id": 115,
                                "tick": 192,
                                "duration": 32,
                                "pitch": {
                                    "int": 72,
                                    "accidental": 0
                                },
                                "velocity": {
                                    "int": 90
                                },
                                "articulations": []
                            }
                        }
                    }
                },
                "38": {
                    "id": 38,
                    "entries": {
                        "by_tick": {
                            "0": [39]
                        },
                        "by_id": {
                            "39": {
                                "type": "Clef",
                                "id": 39,
                                "tick": 0,
                                "draw_as": 2,
                                "pitch": {
                                    "int": 53,
                                    "accidental": 0
                                },
                                "offset": -2,
                                "hidden": false
                            }
                        }
                    }
                },
                "40": {
                    "id": 40,
                    "entries": {
                        "by_tick": {},
                        "by_id": {}
                    }
                }
            }
        }
    },
    "ui": {
        "flow_id": 5,
        "layout_id": 0,
        "renderer": {
            "debug": false,
            "pan_x": 100,
            "pan_y": 100,
            "zoom": 100
        },
        "screens": {
            "sequence": {
                "bottom_panel_minimized": false,
                "bottom_panel_tab": 1,
                "show_active_only": true
            },
            "setup": {
                "expanded": [],
                "selection": null
            },
            "write": {
                "bottom_panel_minimized": false,
                "bottom_panel_tab": 1
            }
        },
        "sequencer": {
            "flow": null,
            "instrument": null,
            "selection": null,
            "snap": 8,
            "tool": 0,
            "track": null,
            "x": 0,
            "y": 1008
        },
        "tab": 1
    }
}"#,
        )
        .unwrap();

        fixture["score"]["tracks"]["by_id"]["6"]["entries"]["by_id"]["70"]["text"] = Value::Null;

        fixture
    }

    fn make_round_trip_tuplet_project() -> ProjectFile {
        let mut ui = Ui::new();
        ui.layout_id = Some(0);
        ui.flow_id = Some(10);

        let mut project_file = ProjectFile {
            ui,
            score: build_score_with_visible_flows(&[10], &[10]),
            next_id: 250,
        };

        let track = project_file.score.tracks.by_id.get_mut(&110).unwrap();
        track.insert_entry(Entry::Tone(Tone::new(
            41,
            48,
            48,
            Pitch::new(58, Accidental::Natural),
            Velocity::new(88),
            Articulations::new(),
        )));

        let mut child_entries = TupletEntries::new();
        child_entries.insert_entry(TupletEntry::Tone(Tone::new(
            42,
            16,
            32,
            Pitch::new(62, Accidental::Sharp),
            Velocity::new(91),
            Articulations::new(),
        )));

        track.insert_entry(Entry::Tuplet(Box::new(Tuplet {
            id: 99,
            tick: 64,
            note_count: 3,
            in_space_of: 2,
            base_note_value: NoteDuration::Eighth,
            groupings: vec![1, 2],
            entries: child_entries,
        })));

        project_file
    }

    #[test]
    fn parse_soloc_container_round_trips_project_payload() {
        let container = build_test_container();
        let app_version = test_app_version();

        let parsed = parse_soloc_container(&container).unwrap();
        let decompressed =
            decompress_project_payload(parsed.project_payload, parsed.project_chunk_flags).unwrap();

        assert_eq!(parsed.app_version, app_version);
        assert_eq!(
            std::str::from_utf8(&decompressed).unwrap(),
            String::from_utf8(build_test_json(app_version)).unwrap()
        );
    }

    #[test]
    fn parse_soloc_container_rejects_invalid_magic() {
        let mut container = build_test_container();
        container[0] = b'X';

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Invalid .soloc magic");
    }

    #[test]
    fn parse_soloc_container_rejects_missing_project_chunk() {
        let mut container = build_test_container();
        let type_offset = type_offset(test_app_version());
        container[type_offset..type_offset + 4]
            .copy_from_slice(&u32::from_le_bytes(*b"META").to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Missing required PROJECT_JSON chunk");
    }

    #[test]
    fn parse_soloc_container_rejects_unsupported_flags() {
        let mut container = build_test_container();
        let flags_offset = flags_offset(test_app_version());
        container[flags_offset..flags_offset + 2].copy_from_slice(&2_u16.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Unsupported .soloc chunk flags: 2");
    }

    #[test]
    fn parse_soloc_container_rejects_out_of_bounds_chunk() {
        let mut container = build_test_container();
        let payload_offset_offset = payload_offset_offset(test_app_version());
        container[payload_offset_offset..payload_offset_offset + 8]
            .copy_from_slice(&u64::MAX.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Chunk bounds overflow");
    }

    #[test]
    fn parse_soloc_container_rejects_duplicate_project_chunk() {
        let app_version = test_app_version();
        let mut container = build_test_container();
        let header_end = header_end(app_version);
        let original_entry =
            container[header_end..header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE].to_vec();
        let chunk_count_offset = header_end - 2;
        container[chunk_count_offset..header_end].copy_from_slice(&2_u16.to_le_bytes());
        container.splice(
            header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE
                ..header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE,
            original_entry,
        );

        let new_payload_offset =
            u64::try_from(header_end + (SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE * 2)).unwrap();
        let first_offset = header_end + 6;
        let second_offset = header_end + SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE + 6;
        container[first_offset..first_offset + 8]
            .copy_from_slice(&new_payload_offset.to_le_bytes());
        container[second_offset..second_offset + 8]
            .copy_from_slice(&new_payload_offset.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Duplicate PROJECT_JSON chunk");
    }

    #[test]
    fn parse_soloc_container_rejects_chunk_pointing_into_header() {
        let mut container = build_test_container();
        let payload_offset_offset = payload_offset_offset(test_app_version());
        container[payload_offset_offset..payload_offset_offset + 8]
            .copy_from_slice(&0_u64.to_le_bytes());

        let error = parse_soloc_container(&container).unwrap_err();

        assert_eq!(error, "Chunk points outside file bounds");
    }

    #[test]
    fn validate_project_version_match_rejects_mismatch() {
        let project_data = json!({
            "score": {
                "meta": {
                    "app_version": "0.0.1"
                }
            }
        });

        let error = validate_project_version_match(test_app_version(), &project_data).unwrap_err();

        assert_eq!(
            error,
            "Invalid .soloc version mismatch between header and project metadata"
        );
    }

    #[test]
    fn migrate_project_data_applies_shipped_migrations_for_older_files() {
        let migrated = migrate_project_data(
            json!({
                "score": {
                    "meta": {
                        "app_version": "1.24.0"
                    },
                    "tracks": {
                        "by_id": {
                            "1": {
                                "entries": {
                                    "by_id": {
                                        "10": {
                                            "type": "Tempo",
                                            "text": null
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "ui": {},
                "next_id": 1
            }),
            "1.24.0",
        )
        .unwrap();

        assert_eq!(
            migrated["score"]["tracks"]["by_id"]["1"]["entries"]["by_id"]["10"]["text"],
            ""
        );
        assert_eq!(
            migrated["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
    }

    #[test]
    fn migrate_project_data_applies_pending_migration_in_development_state() {
        if !pending_migration_registered() {
            return;
        }

        let migrated = migrate_project_data(
            json!({
                "score": {
                    "meta": {
                        "app_version": test_app_version()
                    }
                },
                "ui": {},
                "next_id": 1
            }),
            test_app_version(),
        )
        .unwrap();

        assert_eq!(
            migrated["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
    }

    #[test]
    fn migrate_project_data_handles_mixed_shipped_and_pending_migrations() {
        if !pending_migration_registered() {
            return;
        }

        let migrated = migrate_project_data(
            json!({
                "score": {
                    "meta": {
                        "app_version": "1.24.0"
                    },
                    "tracks": {
                        "by_id": {
                            "1": {
                                "entries": {
                                    "by_id": {
                                        "10": {
                                            "type": "Tempo",
                                            "text": null
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "ui": {},
                "next_id": 1
            }),
            "1.24.0",
        )
        .unwrap();

        assert_eq!(
            migrated["score"]["tracks"]["by_id"]["1"]["entries"]["by_id"]["10"]["text"],
            ""
        );
    }

    #[test]
    fn build_project_file_keeps_the_authoritative_project_dto_shape() {
        let mut ui = Ui::new();
        ui.layout_id = Some(0);
        ui.flow_id = Some(20);

        let mut score = build_score_with_visible_flows(&[10, 20], &[10, 20]);
        score.meta.app_version = "0.0.1".to_string();

        let state = AppContext {
            ui,
            score,
            playback_activation_runtime: PlaybackActivationRuntime::default(),
            preferences: make_preferences(),
            next_id: 42,
            current_file_path: Some("/tmp/example.soloc".to_string()),
        };

        let value = serde_json::to_value(build_project_file(&state)).unwrap();
        let object = value.as_object().unwrap();

        assert_eq!(object.len(), 3);
        assert!(object.contains_key("ui"));
        assert!(object.contains_key("score"));
        assert!(object.contains_key("next_id"));
        assert!(!object.contains_key("preferences"));
        assert!(!object.contains_key("current_file_path"));
        assert_eq!(
            value["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
    }

    #[test]
    fn soloc_round_trip_preserves_multi_flow_projects_and_rebuilds_visible_flow_rendering() {
        let mut ui = Ui::new();
        ui.layout_id = Some(0);
        ui.flow_id = Some(20);

        let project_file = ProjectFile {
            ui,
            score: build_score_with_visible_flows(&[10, 20, 30], &[20, 10]),
            next_id: 77,
        };

        let restored = round_trip_project_file(&project_file);

        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&project_file).unwrap()
        );

        let render_plan = prepare_flow_render_plan(
            &restored.score,
            restored.ui.layout_id.unwrap(),
            restored.ui.flow_id.unwrap(),
            FlowRenderRecalculationRequest::full_visible_flows(),
        )
        .unwrap();

        assert_eq!(render_plan.prepared_flow_ids(), vec![10, 20]);
        assert_eq!(render_plan.recalculated_flow_ids(), vec![10, 20]);
        assert!(render_plan.active_flow_visible_in_layout);
    }

    #[test]
    fn soloc_round_trip_preserves_single_flow_projects_without_multi_flow_regressions() {
        let mut ui = Ui::new();
        ui.layout_id = Some(0);
        ui.flow_id = Some(10);

        let project_file = ProjectFile {
            ui,
            score: build_score_with_visible_flows(&[10], &[10]),
            next_id: 12,
        };

        let restored = round_trip_project_file(&project_file);
        let render_plan = prepare_flow_render_plan(
            &restored.score,
            restored.ui.layout_id.unwrap(),
            restored.ui.flow_id.unwrap(),
            FlowRenderRecalculationRequest::full_visible_flows(),
        )
        .unwrap();

        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&project_file).unwrap()
        );
        assert_eq!(render_plan.prepared_flow_ids(), vec![10]);
        assert_eq!(render_plan.recalculated_flow_ids(), vec![10]);
        assert_eq!(render_plan.prepared_flow_count(), 1);
    }

    #[test]
    fn current_non_tuplet_round_trip_does_not_invent_tuplet_entries() {
        let mut ui = Ui::new();
        ui.layout_id = Some(0);
        ui.flow_id = Some(10);

        let project_file = ProjectFile {
            ui,
            score: build_score_with_visible_flows(&[10], &[10]),
            next_id: 12,
        };

        let restored = round_trip_project_file(&project_file);

        assert!(restored
            .score
            .tracks
            .by_id
            .values()
            .flat_map(|track| track.entries.by_id.values())
            .all(|entry| !matches!(entry, Entry::Tuplet(_))));
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&project_file).unwrap()
        );
    }

    #[test]
    fn soloc_round_trip_preserves_tuplet_entries() {
        let mut ui = Ui::new();
        ui.layout_id = Some(0);
        ui.flow_id = Some(10);

        let mut project_file = ProjectFile {
            ui,
            score: build_score_with_visible_flows(&[10], &[10]),
            next_id: 150,
        };

        project_file
            .score
            .tracks
            .by_id
            .get_mut(&110)
            .unwrap()
            .insert_entry(make_tuplet_entry(99, 64));

        let restored = round_trip_project_file(&project_file);

        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&project_file).unwrap()
        );
    }

    #[test]
    fn soloc_round_trip_preserves_tuplet_ownership_and_groupings() {
        let project_file = make_round_trip_tuplet_project();

        let restored = round_trip_project_file(&project_file);
        let restored_track = restored.score.tracks.by_id.get(&110).unwrap();

        let Some(Entry::Tone(host_tone)) = restored_track.entries.by_id.get(&41) else {
            panic!("expected crossing host-owned tone to remain on the track");
        };
        let Some(Entry::Tuplet(tuplet)) = restored_track.entries.by_id.get(&99) else {
            panic!("expected persisted tuplet container");
        };
        let Some(TupletEntry::Tone(child_tone)) = tuplet.entries.by_id.get(&42) else {
            panic!("expected child-owned tone to remain inside the tuplet");
        };

        assert_eq!(host_tone.tick, 48);
        assert_eq!(host_tone.duration, 48);
        assert_eq!(restored_track.entries.by_tick.get(&48), Some(&vec![41]));
        assert_eq!(tuplet.tick, 64);
        assert_eq!(tuplet.groupings, vec![1, 2]);
        assert_eq!(tuplet.entries.by_tick.get(&16), Some(&vec![42]));
        assert_eq!(child_tone.tick, 16);
        assert_eq!(child_tone.duration, 32);
        assert!(!restored_track.entries.by_id.contains_key(&42));
    }

    #[test]
    fn repeated_tuplet_round_trips_remain_structurally_deterministic() {
        let project_file = make_round_trip_tuplet_project();

        let first_container = build_soloc_payload_for_project(&project_file);
        let (first_header_version, first_project_json_bytes, first_project_json) =
            project_json_from_container(&first_container);
        let first_round_trip = round_trip_project_file(&project_file);

        let second_container = build_soloc_payload_for_project(&first_round_trip);
        let (second_header_version, second_project_json_bytes, second_project_json) =
            project_json_from_container(&second_container);
        let second_round_trip = round_trip_project_file(&first_round_trip);

        assert_eq!(first_header_version, test_app_version());
        assert_eq!(second_header_version, test_app_version());
        assert_eq!(
            first_project_json["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
        assert_eq!(
            second_project_json["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
        assert_eq!(first_project_json_bytes, second_project_json_bytes);
        assert_eq!(first_project_json, second_project_json);
        assert_eq!(
            serde_json::to_value(&first_round_trip).unwrap(),
            serde_json::to_value(&second_round_trip).unwrap()
        );
    }

    #[test]
    fn migrate_project_data_keeps_legacy_non_tuplet_entries_loadable() {
        let migrated = migrate_project_data(
            json!({
                "score": {
                    "meta": {
                        "app_version": test_app_version()
                    },
                    "tracks": {
                        "by_id": {
                            "1": {
                                "id": 1,
                                "entries": {
                                    "by_tick": {
                                        "0": [10]
                                    },
                                    "by_id": {
                                        "10": {
                                            "type": "Tone",
                                            "id": 10,
                                            "tick": 0,
                                            "duration": 8,
                                            "pitch": {
                                                "int": 60,
                                                "accidental": 0
                                            },
                                            "velocity": {
                                                "int": 90
                                            },
                                            "articulations": []
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "ui": {},
                "next_id": 11
            }),
            test_app_version(),
        )
        .unwrap();

        assert_eq!(
            migrated["score"]["tracks"]["by_id"]["1"]["entries"]["by_id"]["10"]["type"],
            Value::String(String::from("Tone"))
        );
        assert_eq!(
            migrated["score"]["meta"]["app_version"],
            Value::String(test_app_version().to_string())
        );
    }

    #[test]
    fn legacy_non_tuplet_soloc_payloads_still_follow_the_existing_load_path() {
        let legacy_version = legacy_non_tuplet_fixture_payload()["score"]["meta"]["app_version"]
            .as_str()
            .unwrap()
            .to_string();
        let legacy_project = legacy_non_tuplet_fixture_payload();

        let compressed_payload =
            compress_project_payload(&serde_json::to_vec(&legacy_project).unwrap()).unwrap();
        let container =
            build_soloc_container(legacy_version.as_str(), &compressed_payload).unwrap();
        let restored = project_file_from_container_bytes(&container).unwrap();

        assert_eq!(restored.score.meta.app_version, test_app_version());
        assert_eq!(restored.score.flows.order, vec![5, 14, 23, 32]);
        assert!(!restored
            .score
            .tracks
            .by_id
            .values()
            .flat_map(|track| track.entries.by_id.values())
            .any(|entry| matches!(entry, Entry::Tuplet(_))));
        assert!(matches!(
            restored
                .score
                .tracks
                .by_id
                .get(&6)
                .and_then(|track| track.entries.by_id.get(&70)),
            Some(Entry::Tempo(tempo)) if tempo.text.is_empty()
        ));
    }

    #[test]
    fn applying_loaded_tuplet_project_resets_ephemeral_state_and_updates_transport() {
        let (_transport_state, transport_shared) = TransportContext::new();
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![]);
        let mut state = AppContext {
            ui: Ui::new(),
            score: build_score_with_visible_flows(&[30], &[30]),
            playback_activation_runtime: PlaybackActivationRuntime::default(),
            preferences: make_preferences(),
            next_id: 5,
            current_file_path: None,
        };
        state.ui.sequencer.tool = Tool::Write;
        state.ui.sequencer.selection = Some(crate::ui::sequencer::Selection::Rest {
            flow_id: 30,
            instrument_id: 130,
            stave_id: 131,
            track_id: 130,
            tick: 32,
            tuplet_id: None,
        });
        state.ui.screens.setup.selection = Some(
            serde_json::from_value(json!({
                "id": 1,
                "selection_type": 0
            }))
            .unwrap(),
        );

        restore_loaded_project_for_test(
            &mut state,
            &transport_shared,
            &inventory,
            make_round_trip_tuplet_project(),
            "/tmp/tuplets.soloc".to_string(),
        );

        assert!(matches!(state.ui.sequencer.tool, Tool::Select));
        assert_eq!(state.ui.sequencer.selection, None);
        assert!(state.ui.screens.setup.selection.is_none());
        assert_eq!(
            state.current_file_path.as_deref(),
            Some("/tmp/tuplets.soloc")
        );
        assert!(matches!(
            state
                .score
                .tracks
                .by_id
                .get(&110)
                .and_then(|track| track.entries.by_id.get(&99)),
            Some(Entry::Tuplet(_))
        ));
        assert_eq!(
            transport_shared
                .current_tick
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        assert!(transport_shared.get_tempo() > 0.0);
    }

    #[test]
    fn applying_loaded_project_can_refresh_single_target_runtime_activation() {
        let (_transport_state, transport_shared) = TransportContext::new();
        let inventory =
            PluginInventoryContext::with_entries_for_tests(vec![available_inventory_entry(
                "plugin-a",
            )]);
        let mut score = build_score_with_visible_flows(&[30], &[30]);
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
        score.create_playback_slot(300).unwrap();
        score
            .assign_plugin_to_slot(300, String::from("plugin-a"))
            .unwrap();
        score
            .playback
            .rack
            .by_id
            .get_mut(&300)
            .unwrap()
            .target
            .as_mut()
            .unwrap()
            .plugin_state = Some(PlaybackSlotPluginState {
            vst3_component_state_base64: String::from("AQID"),
        });
        score.assign_instrument_route_to_slot(10, 300).unwrap();
        let project_file = ProjectFile {
            ui: Ui::new(),
            score,
            next_id: 500,
        };
        let mut state = AppContext {
            ui: Ui::new(),
            score: Score::new(),
            playback_activation_runtime: PlaybackActivationRuntime::default(),
            preferences: make_preferences(),
            next_id: 1,
            current_file_path: None,
        };

        restore_loaded_project_for_test(
            &mut state,
            &transport_shared,
            &inventory,
            project_file,
            "/tmp/runtime-refresh.soloc".to_string(),
        );

        let resolved = state
            .playback_activation_runtime
            .resolved_playback_route_by_instrument_id(&state.score);
        let trust_gate = state
            .playback_activation_runtime
            .single_instrument_trust_gate(&state.score);

        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::Ready
        );
        assert_eq!(
            resolved.get(&10).unwrap().effective_plugin_id.as_deref(),
            Some("plugin-a")
        );
        assert_eq!(
            state.current_file_path.as_deref(),
            Some("/tmp/runtime-refresh.soloc")
        );
        assert!(trust_gate.is_transport_ready());
    }

    #[test]
    fn persisted_playback_intent_round_trips_while_runtime_degrades_against_unavailable_inventory()
    {
        let (_transport_state, transport_shared) = TransportContext::new();
        let inventory = PluginInventoryContext::with_entries_for_tests(vec![]);
        let mut score = build_score_with_visible_flows(&[30], &[30]);
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
        score.create_playback_slot(300).unwrap();
        score
            .assign_plugin_to_slot(300, String::from("plugin-a"))
            .unwrap();
        score
            .playback
            .rack
            .by_id
            .get_mut(&300)
            .unwrap()
            .target
            .as_mut()
            .unwrap()
            .plugin_state = Some(PlaybackSlotPluginState {
            vst3_component_state_base64: String::from("AQID"),
        });
        score.assign_instrument_route_to_slot(10, 300).unwrap();

        let project_file = round_trip_project_file(&ProjectFile {
            ui: Ui::new(),
            score,
            next_id: 500,
        });
        let mut state = AppContext {
            ui: Ui::new(),
            score: Score::new(),
            playback_activation_runtime: PlaybackActivationRuntime::default(),
            preferences: make_preferences(),
            next_id: 1,
            current_file_path: None,
        };

        restore_loaded_project_for_test(
            &mut state,
            &transport_shared,
            &inventory,
            project_file,
            "/tmp/runtime-degraded.soloc".to_string(),
        );

        let resolved = state
            .playback_activation_runtime
            .resolved_playback_route_by_instrument_id(&state.score);
        let trust_gate = state
            .playback_activation_runtime
            .single_instrument_trust_gate(&state.score);

        assert_eq!(
            state.score.instruments.by_id[&10].playback_route,
            Some(InstrumentPlaybackRoute::direct(300))
        );
        assert_eq!(
            state.score.playback.rack.by_id[&300]
                .target
                .as_ref()
                .map(|target| target.plugin_id.as_str()),
            Some("plugin-a")
        );
        assert_eq!(
            state.score.playback.rack.by_id[&300]
                .target
                .as_ref()
                .and_then(|target| target.plugin_state.as_ref()),
            Some(&PlaybackSlotPluginState {
                vst3_component_state_base64: String::from("AQID"),
            })
        );
        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedPlaybackRouteStatus::NeedsAttention
        );
        assert_eq!(resolved.get(&10).unwrap().effective_plugin_id, None);
        assert!(!trust_gate.is_transport_ready());
    }

    #[test]
    fn persisted_nested_tuplet_child_entries_are_rejected() {
        let result = serde_json::from_value::<ProjectFile>(json!({
            "score": {
                "meta": {
                    "app_version": test_app_version()
                },
                "tracks": {
                    "by_id": {
                        "1": {
                            "id": 1,
                            "entries": {
                                "by_tick": {
                                    "64": [10]
                                },
                                "by_id": {
                                    "10": {
                                        "type": "Tuplet",
                                        "id": 10,
                                        "tick": 64,
                                        "note_count": 3,
                                        "in_space_of": 2,
                                        "base_note_value": 8,
                                        "groupings": [3],
                                        "entries": {
                                            "by_tick": {
                                                "16": [12]
                                            },
                                            "by_id": {
                                                "12": {
                                                    "type": "Tuplet",
                                                    "id": 12,
                                                    "tick": 16,
                                                    "note_count": 3,
                                                    "in_space_of": 2,
                                                    "base_note_value": 8,
                                                    "groupings": [3],
                                                    "entries": {
                                                        "by_tick": {},
                                                        "by_id": {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "ui": {},
            "next_id": 13
        }));

        let error = match result {
            Ok(_) => panic!("expected nested tuplet child entries to be rejected"),
            Err(error) => error,
        };

        assert!(!error.to_string().is_empty());
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_project_to_file(
    app: AppHandle,
    state: State<'_, AppContextState>,
    file_path: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    state.score.meta.app_version = CURRENT_APP_VERSION.to_string();
    sync_score_slot_targets_with_runtime_state(&app, &mut state.score)?;

    let project_file = build_project_file(&state);
    let json_data = serde_json::to_vec(&project_file)
        .map_err(|e| format!("Failed to serialize project data: {}", e))?;
    let compressed_data = compress_project_payload(&json_data)?;
    let container = build_soloc_container(CURRENT_APP_VERSION, &compressed_data)?;
    write_file_atomically(Path::new(&file_path), &container)?;

    // Update the current file path
    state.current_file_path = Some(file_path);

    // Emit event to trigger menu refresh
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)] // Tauri command wiring requires separate managed state parameters here.
pub fn load_project_from_file(
    app: AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    audio_state: State<'_, AudioContextState>,
    transport_state: State<'_, Mutex<TransportContext>>,
    file_path: String,
) -> Result<(), String> {
    let transport_shared = stop_transport_playback(transport_state.inner(), audio_state.inner())?;

    let file_data = fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let parsed_container = parse_soloc_container(&file_data)?;
    let json_data = decompress_project_payload(
        parsed_container.project_payload,
        parsed_container.project_chunk_flags,
    )?;
    let project_data: Value = serde_json::from_slice(&json_data)
        .map_err(|e| format!("Failed to parse project JSON: {}", e))?;
    validate_project_version_match(&parsed_container.app_version, &project_data)?;

    let project_data = migrate_project_data(project_data, &parsed_container.app_version)
        .map_err(|e| format!("Failed to migrate project data: {}", e))?;
    let mut project_file: ProjectFile = serde_json::from_value(project_data)
        .map_err(|e| format!("Failed to deserialize project file: {}", e))?;
    project_file.score.meta.app_version = CURRENT_APP_VERSION.to_string();

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    state.ui = project_file.ui;

    // Reset ephemral state
    state.ui.sequencer.tool = Tool::Select;
    state.ui.screens.setup.selection = None;
    state.ui.sequencer.selection = None;

    state.score = project_file.score;
    state.next_id = project_file.next_id;

    // Update the current file path
    state.current_file_path = Some(file_path);
    sync_transport_to_app_state(&transport_shared, &state, 0);
    state.refresh_playback_activation_runtime_from_app(&app);

    // Emit the updated state
    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );

    // Drop state locks before emitting side-channel menu events
    drop(state);
    drop(renderer);

    // Emit event to trigger menu refresh
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Get the current file path (if any)
#[tauri::command(rename_all = "snake_case")]
pub fn get_current_file_path(state: State<'_, AppContextState>) -> Option<String> {
    let state = state.lock().unwrap();
    state.current_file_path.clone()
}

/// Set the current file path and update menu state
#[tauri::command(rename_all = "snake_case")]
pub fn set_current_file_path(
    app: AppHandle,
    state: State<'_, AppContextState>,
    file_path: Option<String>,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let has_file = file_path.is_some();
    state.current_file_path = file_path.clone();

    // Enable/disable the Save menu item based on whether we have a file path
    if let Some(window) = app.get_webview_window("main") {
        if let Some(menu) = window.menu() {
            if let Some(save_item) = menu.get("save") {
                if let Some(item) = save_item.as_menuitem() {
                    item.set_enabled(has_file).ok();
                }
            }
        }
    }

    // Emit event to update UI
    app.emit("soloc://file/path-changed", file_path).ok();

    Ok(())
}

/// Add a file to recent files list
#[tauri::command(rename_all = "snake_case")]
pub fn add_recent_file(app: AppHandle, file_path: String) -> Result<(), String> {
    let db = get_recent_db(&app)?;

    // Get existing recent files
    let mut recent_files: Vec<String> = db
        .get(b"recent_files")
        .map_err(|e| e.to_string())?
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();

    // Remove the file if it already exists (we'll add it to the front)
    recent_files.retain(|f| f != &file_path);

    // Add to front
    recent_files.insert(0, file_path);

    // Keep only the last MAX_RECENT_FILES
    recent_files.truncate(MAX_RECENT_FILES);

    // Save to DB
    let data = serde_json::to_vec(&recent_files).map_err(|e| e.to_string())?;
    db.insert(b"recent_files", data)
        .map_err(|e| e.to_string())?;
    db.flush().map_err(|e| e.to_string())?;

    // Emit event so frontend knows to update menu
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Refresh the menu UI (called by frontend after window is ready)
#[tauri::command(rename_all = "snake_case")]
pub fn refresh_menus(app: AppHandle, state: State<'_, AppContextState>) -> Result<(), String> {
    // Try to update recent files menu (don't fail if menu isn't ready)
    let _ = update_recent_files_menu(&app);

    // Update save menu item based on current file path
    let state = state.lock().unwrap();
    let has_file = state.current_file_path.is_some();

    // Access menu from app handle (not window, since it's application-level on macOS)
    if let Some(menu) = app.menu() {
        if let Some(save_item) = menu.get("save") {
            if let Some(item) = save_item.as_menuitem() {
                item.set_enabled(has_file).ok();
            }
        } else {
            // Save item is in the File submenu, need to find it there
            if let Ok(items) = menu.items() {
                for item in items {
                    if let Some(submenu) = item.as_submenu() {
                        if submenu.text().map(|t| t == "File").unwrap_or(false) {
                            if let Some(save_item) = submenu.get("save") {
                                if let Some(item) = save_item.as_menuitem() {
                                    item.set_enabled(has_file).ok();
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get the list of recent files
#[tauri::command(rename_all = "snake_case")]
pub fn get_recent_files(app: AppHandle) -> Result<Vec<String>, String> {
    let db = get_recent_db(&app)?;

    let recent_files: Vec<String> = db
        .get(b"recent_files")
        .map_err(|e| e.to_string())?
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();

    Ok(recent_files)
}

/// Clear all recent files
#[tauri::command(rename_all = "snake_case")]
pub fn clear_recent_files(app: AppHandle) -> Result<(), String> {
    let db = get_recent_db(&app)?;

    let empty: Vec<String> = vec![];
    let data = serde_json::to_vec(&empty).map_err(|e| e.to_string())?;
    db.insert(b"recent_files", data)
        .map_err(|e| e.to_string())?;
    db.flush().map_err(|e| e.to_string())?;

    // Emit event so frontend knows to update menu
    app.emit("soloc://recents-changed", ()).ok();

    Ok(())
}

/// Update the Recent Files menu
pub fn update_recent_files_menu(app: &AppHandle) -> Result<(), String> {
    let menu = app.menu().ok_or("Menu not found")?;
    let items = menu.items().map_err(|e| e.to_string())?;

    // Find the File submenu
    let file_menu = items
        .iter()
        .find_map(|item| {
            item.as_submenu().filter(|submenu| {
                submenu.id().as_ref() == "file_menu"
                    || submenu.text().map(|t| t == "File").unwrap_or(false)
            })
        })
        .ok_or("File menu not found")?;

    // Get the Open Recents submenu
    let recent_items = file_menu
        .get("open_recent")
        .ok_or("Open Recents menu not found")?;

    let recents = recent_items.as_submenu_unchecked();

    // Remove all existing items
    let items = recents.items().map_err(|e| e.to_string())?;
    for item in items {
        recents.remove(&item).ok();
    }

    // Get recent files from DB
    let db = get_recent_db(app)?;
    let recent_files: Vec<String> = db
        .get(b"recent_files")
        .map_err(|e| e.to_string())?
        .and_then(|v| serde_json::from_slice(&v).ok())
        .unwrap_or_default();

    // Add new items
    if recent_files.is_empty() {
        // Add a disabled "No Recent Files" item
        let no_items = MenuItem::with_id(
            app,
            "no_recent_files",
            "No Recent Files",
            false,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        recents.append(&no_items).map_err(|e| e.to_string())?;
    } else {
        for (i, file_path) in recent_files.iter().enumerate() {
            // Extract just the filename for display
            let file_name = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);

            // Create a unique ID using the index to avoid issues with special characters in paths
            let menu_id = format!("recent_{}", i);

            let item = MenuItem::with_id(app, &menu_id, file_name, true, None::<&str>)
                .map_err(|e| e.to_string())?;

            recents.append(&item).map_err(|e| e.to_string())?;
        }

        // Add separator and "Clear Recent Files" option
        let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
        recents.append(&separator).map_err(|e| e.to_string())?;

        let clear_item = MenuItem::with_id(
            app,
            "clear_recent_files",
            "Clear Recent Files",
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;

        recents.append(&clear_item).map_err(|e| e.to_string())?;
    }

    Ok(())
}
