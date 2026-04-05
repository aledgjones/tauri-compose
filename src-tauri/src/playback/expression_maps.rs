use crate::score::components::articulation::Articulation;
use crate::score::components::duration::NoteDuration;
use crate::score::playback::ResolvedExpressionMapLink;
use crate::score::Score;
use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

const EXPRESSION_MAP_LIBRARY_STATE_KEY: &[u8] = b"expression_map_library_state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpressionMapCondition {
    Articulation { articulation: Articulation },
    WrittenNoteLength { note_duration: NoteDuration },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpressionMapAction {
    KeyswitchNote { midi_note: u8, velocity: u8 },
    ControlChange { controller: u8, value: u8 },
    NoOp,
    NoteLengthTransform { ratio_percent: u16 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExpressionMapRule {
    pub conditions: Vec<ExpressionMapCondition>,
    pub actions: Vec<ExpressionMapAction>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExpressionMap {
    pub id: String,
    pub name: String,
    pub rules: Vec<ExpressionMapRule>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExpressionMapLibraryState {
    pub order: Vec<String>,
    pub by_id: FxHashMap<String, ExpressionMap>,
}

impl ExpressionMapLibraryState {
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            by_id: FxHashMap::default(),
        }
    }
}

impl Default for ExpressionMapLibraryState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionMapLibraryContext {
    state: ExpressionMapLibraryState,
    db: Option<Db>,
}

pub type ExpressionMapLibraryContextState = Mutex<ExpressionMapLibraryContext>;

impl ExpressionMapLibraryContext {
    pub fn new(app: &AppHandle) -> ExpressionMapLibraryContextState {
        let db_path = match app.path().app_config_dir() {
            Ok(path) => Some(path.join("expression_maps")),
            Err(error) => {
                eprintln!("Failed to resolve expression map library config path: {error:?}");
                None
            }
        };

        Mutex::new(Self::from_db_path(db_path.as_deref()))
    }

    fn from_db_path(db_path: Option<&Path>) -> Self {
        let db = match db_path {
            Some(path) => {
                if let Some(parent) = path.parent() {
                    if let Err(error) = fs::create_dir_all(parent) {
                        eprintln!("Failed to create expression map library directory: {error}");
                        return Self {
                            state: ExpressionMapLibraryState::default(),
                            db: None,
                        };
                    }
                }

                match sled::open(path) {
                    Ok(db) => Some(db),
                    Err(error) => {
                        eprintln!("Failed to open expression map library storage: {error}");
                        None
                    }
                }
            }
            None => None,
        };

        let state = match db.as_ref() {
            Some(db) => match load_persisted_state(db) {
                Ok(Some(state)) => state,
                Ok(None) => ExpressionMapLibraryState::default(),
                Err(error) => {
                    eprintln!(
                        "Failed to decode expression map library storage; starting empty: {error}"
                    );
                    ExpressionMapLibraryState::default()
                }
            },
            None => ExpressionMapLibraryState::default(),
        };

        Self { state, db }
    }

    #[cfg(test)]
    pub fn with_maps_for_tests(maps: Vec<ExpressionMap>) -> Self {
        let mut state = ExpressionMapLibraryState::new();

        for map in maps {
            state.order.push(map.id.clone());
            state.by_id.insert(map.id.clone(), map);
        }

        Self { state, db: None }
    }

    pub fn snapshot(&self) -> ExpressionMapLibraryState {
        self.state.clone()
    }

    pub fn emit(&self, app: &AppHandle) {
        if let Err(error) = app.emit("soloc://state/expression_maps", &self.state) {
            eprintln!("Failed to emit expression map library state: {error:?}");
        }
    }

    pub fn expression_map(&self, expression_map_id: &str) -> Option<&ExpressionMap> {
        self.state.by_id.get(expression_map_id)
    }

    pub fn upsert_expression_map(&mut self, expression_map: ExpressionMap) -> Result<(), String> {
        if !self.state.by_id.contains_key(&expression_map.id) {
            self.state.order.push(expression_map.id.clone());
        }

        self.state
            .by_id
            .insert(expression_map.id.clone(), expression_map);

        self.persist_state()
    }

    pub fn resolved_expression_map_links_by_instrument_id(
        &self,
        score: &Score,
    ) -> FxHashMap<UniqueId, ResolvedExpressionMapLink> {
        score
            .instruments
            .by_id
            .iter()
            .map(|(instrument_id, instrument)| {
                let resolved = instrument
                    .playback_route
                    .as_ref()
                    .and_then(|route| route.expression_map_id.as_deref())
                    .and_then(|expression_map_id| self.expression_map(expression_map_id))
                    .map(|expression_map| {
                        ResolvedExpressionMapLink::assigned(
                            expression_map.id.clone(),
                            expression_map.name.clone(),
                        )
                    })
                    .unwrap_or_default();

                (*instrument_id, resolved)
            })
            .collect()
    }

    fn persist_state(&self) -> Result<(), String> {
        let Some(db) = &self.db else {
            return Ok(());
        };

        let bytes = serde_json::to_vec(&self.state).map_err(|error| error.to_string())?;
        db.insert(EXPRESSION_MAP_LIBRARY_STATE_KEY, bytes)
            .map_err(|error| error.to_string())?;
        db.flush().map_err(|error| error.to_string())?;

        Ok(())
    }
}

fn load_persisted_state(db: &Db) -> Result<Option<ExpressionMapLibraryState>, String> {
    let Some(bytes) = db
        .get(EXPRESSION_MAP_LIBRARY_STATE_KEY)
        .map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };

    let state = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    Ok(Some(state))
}

#[cfg(test)]
mod tests {
    use super::{
        ExpressionMap, ExpressionMapAction, ExpressionMapCondition, ExpressionMapLibraryContext,
        ExpressionMapRule,
    };
    use crate::score::instruments::{defs::InstrumentType, Instrument, Instruments};
    use crate::score::playback::{
        InstrumentPlaybackRoute, PlaybackRouteAddress, PlaybackRouteTarget,
        ResolvedExpressionMapLinkStatus,
    };
    use crate::score::players::{Player, PlayerType, Players};
    use crate::score::Score;
    use rustc_hash::FxHashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn expression_map_with_name(id: &str, name: &str) -> ExpressionMap {
        ExpressionMap {
            id: id.to_string(),
            name: name.to_string(),
            rules: vec![ExpressionMapRule {
                conditions: vec![ExpressionMapCondition::Articulation {
                    articulation: crate::score::components::articulation::Articulation::Accent,
                }],
                actions: vec![ExpressionMapAction::KeyswitchNote {
                    midi_note: 24,
                    velocity: 96,
                }],
            }],
        }
    }

    fn make_score_with_expression_map_reference(expression_map_id: Option<&str>) -> Score {
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
                    playback_route: Some(InstrumentPlaybackRoute {
                        slot_id: 100,
                        target: PlaybackRouteTarget::plugin(),
                        address: PlaybackRouteAddress::root(),
                        expression_map_id: expression_map_id.map(String::from),
                    }),
                    volume: 100,
                    pan: 64,
                    mute: false,
                    solo: false,
                },
            )]),
        };

        score
    }

    fn unique_temp_db_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("compose-expression-map-tests-{nanos}"))
    }

    #[test]
    fn expression_map_library_persists_and_reloads_state() {
        let db_path = unique_temp_db_path();
        let mut library = ExpressionMapLibraryContext::from_db_path(Some(&db_path));

        library
            .upsert_expression_map(expression_map_with_name("legato", "Legato"))
            .unwrap();
        drop(library);

        let reloaded = ExpressionMapLibraryContext::from_db_path(Some(&db_path));
        let snapshot = reloaded.snapshot();

        assert_eq!(snapshot.order, vec![String::from("legato")]);
        assert_eq!(snapshot.by_id["legato"].name, "Legato");

        let _ = fs::remove_dir_all(&db_path);
    }

    #[test]
    fn expression_map_library_recovers_from_decode_failures_with_an_empty_state() {
        let db_path = unique_temp_db_path();
        let db = sled::open(&db_path).unwrap();
        db.insert(
            super::EXPRESSION_MAP_LIBRARY_STATE_KEY,
            b"not-json".as_slice(),
        )
        .unwrap();
        db.flush().unwrap();
        drop(db);

        let mut library = ExpressionMapLibraryContext::from_db_path(Some(&db_path));

        assert!(library.snapshot().order.is_empty());
        library
            .upsert_expression_map(expression_map_with_name("legato", "Legato"))
            .unwrap();
        drop(library);

        let reloaded = ExpressionMapLibraryContext::from_db_path(Some(&db_path));

        assert_eq!(reloaded.snapshot().order, vec![String::from("legato")]);

        let _ = fs::remove_dir_all(&db_path);
    }

    #[test]
    fn resolved_expression_map_links_clear_missing_references_to_unassigned() {
        let library = ExpressionMapLibraryContext::with_maps_for_tests(vec![]);
        let score = make_score_with_expression_map_reference(Some("missing-map"));

        let resolved = library.resolved_expression_map_links_by_instrument_id(&score);

        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedExpressionMapLinkStatus::Unassigned
        );
        assert_eq!(resolved.get(&10).unwrap().expression_map_id, None);
        assert_eq!(resolved.get(&10).unwrap().expression_map_name, None);
    }

    #[test]
    fn resolved_expression_map_links_assign_existing_map_names() {
        let library =
            ExpressionMapLibraryContext::with_maps_for_tests(vec![expression_map_with_name(
                "staccato", "Staccato",
            )]);
        let score = make_score_with_expression_map_reference(Some("staccato"));

        let resolved = library.resolved_expression_map_links_by_instrument_id(&score);

        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedExpressionMapLinkStatus::Assigned
        );
        assert_eq!(
            resolved.get(&10).unwrap().expression_map_id.as_deref(),
            Some("staccato")
        );
        assert_eq!(
            resolved.get(&10).unwrap().expression_map_name.as_deref(),
            Some("Staccato")
        );
    }
}
