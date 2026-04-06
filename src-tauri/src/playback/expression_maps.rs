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
use uuid::Uuid;

const EXPRESSION_MAP_LIBRARY_STATE_KEY: &[u8] = b"expression_map_library_state";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionMapEqualityComparator {
    #[default]
    Equals,
    NotEquals,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionMapNoteLengthComparator {
    LessThan,
    LessThanOrEqual,
    #[default]
    Exactly,
    NotEqual,
    GreaterThanOrEqual,
    GreaterThan,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpressionMapCondition {
    Articulation {
        id: String,
        #[serde(default)]
        comparator: ExpressionMapEqualityComparator,
        articulation: Option<Articulation>,
    },
    WrittenNoteLength {
        id: String,
        #[serde(default)]
        comparator: ExpressionMapNoteLengthComparator,
        note_duration: NoteDuration,
    },
}

impl ExpressionMapCondition {
    fn id(&self) -> &str {
        match self {
            Self::Articulation { id, .. } | Self::WrittenNoteLength { id, .. } => id.as_str(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpressionMapAction {
    KeyswitchNote {
        id: String,
        midi_note: u8,
        velocity: u8,
    },
    ControlChange {
        id: String,
        controller: u8,
        value: u8,
    },
    NoOp {
        id: String,
    },
    NoteLengthTransform {
        id: String,
        ratio_percent: u16,
    },
}

impl ExpressionMapAction {
    fn id(&self) -> &str {
        match self {
            Self::KeyswitchNote { id, .. }
            | Self::ControlChange { id, .. }
            | Self::NoOp { id }
            | Self::NoteLengthTransform { id, .. } => id.as_str(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExpressionMapRule {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub conditions: Vec<ExpressionMapCondition>,
    pub actions: Vec<ExpressionMapAction>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExpressionMap {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub version: u32,
    pub rules: Vec<ExpressionMapRule>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExpressionMapExportPayload {
    pub version: u8,
    pub expression_maps: Vec<ExpressionMap>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ExpressionMapImportPayload {
    Envelope(ExpressionMapExportPayload),
    List(Vec<ExpressionMap>),
    Single(ExpressionMap),
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

    pub fn add_default_rule(
        &mut self,
        expression_map_id: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            expression_map
                .rules
                .push(create_default_expression_map_rule(next_default_rule_name(
                    &expression_map.rules,
                )));
            Ok(())
        })
    }

    pub fn set_rule_name(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        name: String,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            find_rule_mut(expression_map, rule_id)?.name = name;
            Ok(())
        })
    }

    pub fn move_rule(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        to_index: usize,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            if expression_map.rules.is_empty() {
                return Err(String::from("Cannot move a rule in an empty expression map."));
            }

            if to_index >= expression_map.rules.len() {
                return Err(format!(
                    "Rule move target index {to_index} is out of bounds for {} rules.",
                    expression_map.rules.len()
                ));
            }

            let from_index = expression_map
                .rules
                .iter()
                .position(|rule| rule.id == rule_id)
                .ok_or_else(|| format!("Unknown expression map rule: {rule_id}"))?;

            if from_index == to_index {
                return Ok(());
            }

            let rule = expression_map.rules.remove(from_index);
            expression_map.rules.insert(to_index, rule);
            Ok(())
        })
    }

    pub fn remove_rule(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            let rule_count_before = expression_map.rules.len();
            expression_map.rules.retain(|rule| rule.id != rule_id);

            if expression_map.rules.len() == rule_count_before {
                return Err(format!("Unknown expression map rule: {rule_id}"));
            }

            Ok(())
        })
    }

    pub fn add_condition(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        condition_type: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            find_rule_mut(expression_map, rule_id)?
                .conditions
                .push(create_expression_map_condition(condition_type)?);
            Ok(())
        })
    }

    pub fn set_condition_type(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        condition_id: &str,
        condition_type: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            let rule = find_rule_mut(expression_map, rule_id)?;
            let condition = find_condition_mut(rule, condition_id)?;
            let id = condition.id().to_string();
            *condition = create_expression_map_condition_with_id(condition_type, id)?;
            Ok(())
        })
    }

    pub fn update_condition(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        condition: ExpressionMapCondition,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            let rule = find_rule_mut(expression_map, rule_id)?;
            let existing_condition = find_condition_mut(rule, condition.id())?;
            *existing_condition = condition;
            Ok(())
        })
    }

    pub fn remove_condition(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        condition_id: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            let rule = find_rule_mut(expression_map, rule_id)?;
            let condition_count_before = rule.conditions.len();
            rule.conditions
                .retain(|condition| condition.id() != condition_id);

            if rule.conditions.len() == condition_count_before {
                return Err(format!("Unknown expression map condition: {condition_id}"));
            }

            Ok(())
        })
    }

    pub fn add_action(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        action_type: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            find_rule_mut(expression_map, rule_id)?
                .actions
                .push(create_expression_map_action(action_type)?);
            Ok(())
        })
    }

    pub fn set_action_type(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        action_id: &str,
        action_type: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            let rule = find_rule_mut(expression_map, rule_id)?;
            let action = find_action_mut(rule, action_id)?;
            let id = action.id().to_string();
            *action = create_expression_map_action_with_id(action_type, id)?;
            Ok(())
        })
    }

    pub fn update_action(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        action: ExpressionMapAction,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            let rule = find_rule_mut(expression_map, rule_id)?;
            let existing_action = find_action_mut(rule, action.id())?;
            *existing_action = action;
            Ok(())
        })
    }

    pub fn remove_action(
        &mut self,
        expression_map_id: &str,
        rule_id: &str,
        action_id: &str,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            let rule = find_rule_mut(expression_map, rule_id)?;
            let action_count_before = rule.actions.len();
            rule.actions.retain(|action| action.id() != action_id);

            if rule.actions.len() == action_count_before {
                return Err(format!("Unknown expression map action: {action_id}"));
            }

            Ok(())
        })
    }

    pub fn create_expression_map(&mut self, name: Option<String>) -> Result<ExpressionMap, String> {
        let map_name =
            normalize_name(name.as_deref()).unwrap_or_else(|| String::from("New Expression Map"));
        let expression_map = ExpressionMap {
            id: generate_expression_map_id(&self.state),
            name: map_name,
            creator: String::new(),
            version: 1,
            rules: Vec::new(),
        };

        self.insert_expression_map(expression_map)
    }

    pub fn set_expression_map_name(
        &mut self,
        expression_map_id: &str,
        name: String,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            expression_map.name = name;
            Ok(())
        })
    }

    pub fn set_expression_map_creator(
        &mut self,
        expression_map_id: &str,
        creator: String,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            expression_map.creator = creator;
            Ok(())
        })
    }

    pub fn set_expression_map_version(
        &mut self,
        expression_map_id: &str,
        version: u32,
    ) -> Result<ExpressionMap, String> {
        self.update_expression_map(expression_map_id, |expression_map| {
            expression_map.version = version;
            Ok(())
        })
    }

    pub fn duplicate_expression_map(
        &mut self,
        expression_map_id: &str,
    ) -> Result<ExpressionMap, String> {
        let source = self
            .state
            .by_id
            .get(expression_map_id)
            .cloned()
            .ok_or_else(|| format!("Unknown expression map: {expression_map_id}"))?;
        let duplicated_name = format!("{} Copy", source.name);
        let duplicate = ExpressionMap {
            id: generate_expression_map_id(&self.state),
            name: duplicated_name,
            creator: source.creator,
            version: source.version,
            rules: source.rules,
        };

        self.insert_expression_map(duplicate)
    }

    pub fn delete_expression_map(&mut self, expression_map_id: &str) -> Result<(), String> {
        self.state.by_id.remove(expression_map_id);
        self.state.order.retain(|id| id != expression_map_id);
        self.persist_state()
    }

    pub fn import_expression_maps(&mut self, file_path: &Path) -> Result<Vec<String>, String> {
        let contents = fs::read_to_string(file_path).map_err(|error| error.to_string())?;
        self.import_expression_maps_from_json(&contents)
    }

    fn import_expression_maps_from_json(&mut self, contents: &str) -> Result<Vec<String>, String> {
        let payload = serde_json::from_str::<ExpressionMapImportPayload>(contents)
            .map_err(|error| error.to_string())?;
        let maps = match payload {
            ExpressionMapImportPayload::Envelope(envelope) => envelope.expression_maps,
            ExpressionMapImportPayload::List(maps) => maps,
            ExpressionMapImportPayload::Single(map) => vec![map],
        };

        import_expression_maps_into_state(self, maps)
    }

    pub fn export_expression_maps(
        &self,
        file_path: &Path,
        expression_map_ids: &[String],
    ) -> Result<ExpressionMapExportPayload, String> {
        let expression_maps = expression_map_ids
            .iter()
            .map(|expression_map_id| {
                self.state
                    .by_id
                    .get(expression_map_id)
                    .cloned()
                    .ok_or_else(|| format!("Unknown expression map: {expression_map_id}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let payload = ExpressionMapExportPayload {
            version: 1,
            expression_maps,
        };
        let serialized =
            serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?;
        fs::write(file_path, serialized).map_err(|error| error.to_string())?;

        Ok(payload)
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

    fn update_expression_map<F>(
        &mut self,
        expression_map_id: &str,
        updater: F,
    ) -> Result<ExpressionMap, String>
    where
        F: FnOnce(&mut ExpressionMap) -> Result<(), String>,
    {
        let mut expression_map = self
            .state
            .by_id
            .get(expression_map_id)
            .cloned()
            .ok_or_else(|| format!("Unknown expression map: {expression_map_id}"))?;

        updater(&mut expression_map)?;
        validate_expression_map_rule_ids(&expression_map)?;
        self.state
            .by_id
            .insert(expression_map.id.clone(), expression_map.clone());
        self.persist_state()?;
        Ok(expression_map)
    }

    fn insert_expression_map(
        &mut self,
        expression_map: ExpressionMap,
    ) -> Result<ExpressionMap, String> {
        validate_expression_map_rule_ids(&expression_map)?;
        self.state.order.push(expression_map.id.clone());
        self.state
            .by_id
            .insert(expression_map.id.clone(), expression_map.clone());
        self.persist_state()?;
        Ok(expression_map)
    }
}

fn normalize_name(name: Option<&str>) -> Option<String> {
    name.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn generate_expression_map_id(state: &ExpressionMapLibraryState) -> String {
    loop {
        let candidate = Uuid::new_v4().to_string();
        if !state.by_id.contains_key(&candidate) {
            return candidate;
        }
    }
}

fn generate_expression_map_rule_id() -> String {
    Uuid::new_v4().to_string()
}

fn generate_expression_map_condition_id() -> String {
    Uuid::new_v4().to_string()
}

fn generate_expression_map_action_id() -> String {
    Uuid::new_v4().to_string()
}

fn create_default_expression_map_condition() -> ExpressionMapCondition {
    ExpressionMapCondition::Articulation {
        id: generate_expression_map_condition_id(),
        comparator: ExpressionMapEqualityComparator::Equals,
        articulation: None,
    }
}

fn create_expression_map_condition(condition_type: &str) -> Result<ExpressionMapCondition, String> {
    create_expression_map_condition_with_id(
        condition_type,
        generate_expression_map_condition_id(),
    )
}

fn create_expression_map_condition_with_id(
    condition_type: &str,
    id: String,
) -> Result<ExpressionMapCondition, String> {
    match condition_type {
        "articulation" => Ok(ExpressionMapCondition::Articulation {
            id,
            comparator: ExpressionMapEqualityComparator::Equals,
            articulation: None,
        }),
        "written_note_length" => Ok(ExpressionMapCondition::WrittenNoteLength {
            id,
            comparator: ExpressionMapNoteLengthComparator::Exactly,
            note_duration: NoteDuration::Quarter,
        }),
        _ => Err(format!("Unsupported expression map condition type: {condition_type}")),
    }
}

fn create_default_expression_map_action() -> ExpressionMapAction {
    ExpressionMapAction::NoOp {
        id: generate_expression_map_action_id(),
    }
}

fn create_expression_map_action(action_type: &str) -> Result<ExpressionMapAction, String> {
    create_expression_map_action_with_id(action_type, generate_expression_map_action_id())
}

fn create_expression_map_action_with_id(
    action_type: &str,
    id: String,
) -> Result<ExpressionMapAction, String> {
    match action_type {
        "keyswitch_note" => Ok(ExpressionMapAction::KeyswitchNote {
            id,
            midi_note: 24,
            velocity: 127,
        }),
        "control_change" => Ok(ExpressionMapAction::ControlChange {
            id,
            controller: 1,
            value: 64,
        }),
        "note_length_transform" => Ok(ExpressionMapAction::NoteLengthTransform {
            id,
            ratio_percent: 100,
        }),
        "no_op" => Ok(ExpressionMapAction::NoOp { id }),
        _ => Err(format!("Unsupported expression map action type: {action_type}")),
    }
}

fn create_default_expression_map_rule(name: String) -> ExpressionMapRule {
    ExpressionMapRule {
        id: generate_expression_map_rule_id(),
        name,
        conditions: vec![create_default_expression_map_condition()],
        actions: vec![create_default_expression_map_action()],
    }
}

fn parse_default_rule_number(name: &str) -> Option<u32> {
    let trimmed = name.trim();
    let suffix = trimmed.strip_prefix("Rule")?;
    let suffix = suffix.strip_prefix(' ').unwrap_or(suffix);

    if suffix.is_empty()
        || suffix.starts_with('0')
        || !suffix.chars().all(|character| character.is_ascii_digit())
    {
        return None;
    }

    suffix.parse::<u32>().ok()
}

fn next_default_rule_name(rules: &[ExpressionMapRule]) -> String {
    let mut used_rule_numbers = std::collections::HashSet::new();

    for rule in rules {
        if let Some(rule_number) = parse_default_rule_number(&rule.name) {
            used_rule_numbers.insert(rule_number);
        }
    }

    let mut candidate_rule_number = 1;
    while used_rule_numbers.contains(&candidate_rule_number) {
        candidate_rule_number += 1;
    }

    format!("Rule {candidate_rule_number}")
}

fn validate_expression_map_rule_ids(expression_map: &ExpressionMap) -> Result<(), String> {
    let mut used_rule_ids = std::collections::HashSet::new();

    for rule in &expression_map.rules {
        if rule.id.trim().is_empty() {
            return Err(String::from("Expression map rules must have ids."));
        }

        if !used_rule_ids.insert(rule.id.clone()) {
            return Err(format!("Duplicate expression map rule id: {}", rule.id));
        }

        let mut used_condition_ids = std::collections::HashSet::new();
        for condition in &rule.conditions {
            let condition_id = condition.id().trim();

            if condition_id.is_empty() {
                return Err(format!(
                    "Expression map rule {} conditions must have ids.",
                    rule.id
                ));
            }

            if !used_condition_ids.insert(condition_id.to_string()) {
                return Err(format!(
                    "Duplicate expression map condition id in rule {}: {}",
                    rule.id, condition_id
                ));
            }
        }

        let mut used_action_ids = std::collections::HashSet::new();
        for action in &rule.actions {
            let action_id = action.id().trim();

            if action_id.is_empty() {
                return Err(format!(
                    "Expression map rule {} actions must have ids.",
                    rule.id
                ));
            }

            if !used_action_ids.insert(action_id.to_string()) {
                return Err(format!(
                    "Duplicate expression map action id in rule {}: {}",
                    rule.id, action_id
                ));
            }
        }
    }

    Ok(())
}

fn find_rule_mut<'a>(
    expression_map: &'a mut ExpressionMap,
    rule_id: &str,
) -> Result<&'a mut ExpressionMapRule, String> {
    expression_map
        .rules
        .iter_mut()
        .find(|rule| rule.id == rule_id)
        .ok_or_else(|| format!("Unknown expression map rule: {rule_id}"))
}

fn find_condition_mut<'a>(
    rule: &'a mut ExpressionMapRule,
    condition_id: &str,
) -> Result<&'a mut ExpressionMapCondition, String> {
    rule.conditions
        .iter_mut()
        .find(|condition| condition.id() == condition_id)
        .ok_or_else(|| format!("Unknown expression map condition: {condition_id}"))
}

fn find_action_mut<'a>(
    rule: &'a mut ExpressionMapRule,
    action_id: &str,
) -> Result<&'a mut ExpressionMapAction, String> {
    rule.actions
        .iter_mut()
        .find(|action| action.id() == action_id)
        .ok_or_else(|| format!("Unknown expression map action: {action_id}"))
}

fn import_expression_maps_into_state(
    library: &mut ExpressionMapLibraryContext,
    maps: Vec<ExpressionMap>,
) -> Result<Vec<String>, String> {
    if maps.is_empty() {
        return Err(String::from(
            "No expression maps were found in the import file.",
        ));
    }

    let mut imported_ids = Vec::with_capacity(maps.len());
    for map in maps {
        let map_name = normalize_name(Some(&map.name))
            .unwrap_or_else(|| String::from("Imported Expression Map"));
        let next_id = if library.state.by_id.contains_key(&map.id) {
            generate_expression_map_id(&library.state)
        } else {
            map.id.clone()
        };
        let imported = ExpressionMap {
            id: next_id.clone(),
            name: map_name,
            creator: map.creator,
            version: map.version,
            rules: map.rules,
        };

        library.insert_expression_map(imported)?;
        imported_ids.push(next_id);
    }

    Ok(imported_ids)
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
        ExpressionMap, ExpressionMapAction, ExpressionMapCondition,
        ExpressionMapEqualityComparator, ExpressionMapLibraryContext,
        ExpressionMapNoteLengthComparator, ExpressionMapRule,
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
    use uuid::Uuid;

    fn expression_map_with_name(id: &str, name: &str) -> ExpressionMap {
        ExpressionMap {
            id: id.to_string(),
            name: name.to_string(),
            creator: String::new(),
            version: 1,
            rules: vec![ExpressionMapRule {
                id: String::from("accent-rule"),
                name: String::new(),
                conditions: vec![ExpressionMapCondition::Articulation {
                    id: String::from("accent-condition"),
                    comparator: ExpressionMapEqualityComparator::Equals,
                    articulation: Some(
                        crate::score::components::articulation::Articulation::Accent,
                    ),
                }],
                actions: vec![ExpressionMapAction::KeyswitchNote {
                    id: String::from("accent-action"),
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
            .insert_expression_map(expression_map_with_name("legato", "Legato"))
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
            .insert_expression_map(expression_map_with_name("legato", "Legato"))
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

    #[test]
    fn expression_map_library_deletes_maps_without_touching_score_links() {
        let mut library =
            ExpressionMapLibraryContext::with_maps_for_tests(vec![expression_map_with_name(
                "staccato", "Staccato",
            )]);
        let score = make_score_with_expression_map_reference(Some("staccato"));

        library.delete_expression_map("staccato").unwrap();

        let resolved = library.resolved_expression_map_links_by_instrument_id(&score);
        assert_eq!(
            resolved.get(&10).unwrap().status,
            ResolvedExpressionMapLinkStatus::Unassigned
        );
    }

    #[test]
    fn expression_map_library_imports_and_exports_json_payloads() {
        let db_path = unique_temp_db_path();
        let export_path = db_path.join("expression-maps.json");
        fs::create_dir_all(&db_path).unwrap();

        let mut library = ExpressionMapLibraryContext::from_db_path(Some(&db_path));
        let imported_ids = library
            .import_expression_maps_from_json(
                r#"{"version":1,"expression_maps":[{"id":"legato","name":"Legato","creator":"","version":1,"rules":[]}]}"#,
            )
            .unwrap();

        assert_eq!(imported_ids, vec![String::from("legato")]);

        let payload = library
            .export_expression_maps(&export_path, &[String::from("legato")])
            .unwrap();

        assert_eq!(payload.version, 1);
        assert_eq!(payload.expression_maps.len(), 1);
        assert!(export_path.exists());

        let _ = fs::remove_dir_all(&db_path);
    }

    #[test]
    fn expression_map_library_creates_maps_with_backend_owned_uuid_ids() {
        let mut library = ExpressionMapLibraryContext::with_maps_for_tests(vec![]);

        let expression_map = library.create_expression_map(Some(String::from("Legato"))).unwrap();

        assert_eq!(expression_map.name, "Legato");
        assert!(Uuid::parse_str(&expression_map.id).is_ok());
        assert_eq!(library.snapshot().order, vec![expression_map.id.clone()]);
        assert!(library.snapshot().by_id.contains_key(&expression_map.id));
    }

    #[test]
    fn expression_map_library_duplicates_maps_with_new_backend_owned_uuid_ids() {
        let mut library =
            ExpressionMapLibraryContext::with_maps_for_tests(vec![expression_map_with_name(
                "legato", "Legato",
            )]);

        let expression_map = library
            .duplicate_expression_map("legato")
            .unwrap();

        assert_eq!(expression_map.name, "Legato Copy");
        assert_ne!(expression_map.id, "legato");
        assert!(Uuid::parse_str(&expression_map.id).is_ok());
        assert_eq!(library.snapshot().order.len(), 2);
        assert!(library.snapshot().by_id.contains_key("legato"));
        assert!(library.snapshot().by_id.contains_key(&expression_map.id));
    }

    #[test]
    fn expression_map_library_decodes_current_condition_payloads() {
        let payload = r#"{"order":["legato"],"by_id":{"legato":{"id":"legato","name":"Legato","creator":"","version":1,"rules":[{"id":"accent-rule","conditions":[{"id":"condition-1","type":"articulation","comparator":"equals","articulation":4},{"id":"condition-2","type":"written_note_length","comparator":"exactly","note_duration":4}],"actions":[{"id":"action-1","type":"no_op"}]}]}}}"#;

        let state = serde_json::from_str::<super::ExpressionMapLibraryState>(payload).unwrap();
        let rule = &state.by_id["legato"].rules[0];

        assert_eq!(rule.id, "accent-rule");

        assert_eq!(
            rule.conditions[0],
            ExpressionMapCondition::Articulation {
                id: String::from("condition-1"),
                comparator: ExpressionMapEqualityComparator::Equals,
                articulation: Some(crate::score::components::articulation::Articulation::Accent,),
            }
        );
        assert_eq!(
            rule.conditions[1],
            ExpressionMapCondition::WrittenNoteLength {
                id: String::from("condition-2"),
                comparator: ExpressionMapNoteLengthComparator::Exactly,
                note_duration: crate::score::components::duration::NoteDuration::Quarter,
            }
        );
    }

    #[test]
    fn expression_map_library_rejects_payloads_without_nested_ids() {
        let payload = r#"{"order":["legato"],"by_id":{"legato":{"id":"legato","name":"Legato","creator":"","version":1,"rules":[{"id":"accent-rule","conditions":[{"type":"articulation","comparator":"equals","articulation":4}],"actions":[{"type":"no_op"}]}]}}}"#;

        assert!(serde_json::from_str::<super::ExpressionMapLibraryState>(payload).is_err());
    }

    #[test]
    fn expression_map_library_adds_default_rules_with_backend_owned_ids() {
        let mut library = ExpressionMapLibraryContext::with_maps_for_tests(vec![ExpressionMap {
            id: String::from("legato"),
            name: String::from("Legato"),
            creator: String::new(),
            version: 1,
            rules: vec![ExpressionMapRule {
                id: String::from("rule-2"),
                name: String::from("Rule 2"),
                conditions: vec![ExpressionMapCondition::Articulation {
                    id: String::from("rule-2-condition"),
                    comparator: ExpressionMapEqualityComparator::Equals,
                    articulation: Some(
                        crate::score::components::articulation::Articulation::Accent,
                    ),
                }],
                actions: vec![ExpressionMapAction::NoOp {
                    id: String::from("rule-2-action"),
                }],
            }],
        }]);

        let updated = library.add_default_rule("legato").unwrap();
        let added_rule = updated.rules.last().unwrap();

        assert_eq!(added_rule.name, "Rule 1");
        assert!(!added_rule.id.is_empty());
        assert_ne!(added_rule.id, "rule-2");
        assert_eq!(added_rule.conditions.len(), 1);
        assert_eq!(added_rule.actions.len(), 1);
        match &added_rule.conditions[0] {
            ExpressionMapCondition::Articulation {
                id,
                comparator,
                articulation,
            } => {
                assert!(!id.is_empty());
                assert_eq!(*comparator, ExpressionMapEqualityComparator::Equals);
                assert_eq!(*articulation, None);
            }
            other => panic!("Expected articulation condition, got {other:?}"),
        }
        match &added_rule.actions[0] {
            ExpressionMapAction::NoOp { id } => {
                assert!(!id.is_empty());
            }
            other => panic!("Expected no-op action, got {other:?}"),
        }
    }
}
