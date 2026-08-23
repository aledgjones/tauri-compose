use crate::{AppContextState, RendererContextState};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};
use vst3::Steinberg::TUID;

const INVENTORY_STATE_KEY: &[u8] = b"plugin_inventory_state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginInventoryAvailability {
    Available,
    Unavailable,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginInventoryFailureKind {
    InvalidBundle,
    MissingBundleBinary,
    UnsupportedEditorHost,
    IoError,
    StorageError,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginInventoryFailure {
    pub kind: PluginInventoryFailureKind,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorHostValidationStatus {
    Blocked,
    Failed,
    Attached,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorHostValidationPhase {
    PolicyGate,
    ModuleLoad,
    FactoryCreate,
    HostContext,
    ComponentCreate,
    ComponentInitialize,
    ComponentRuntimeSetup,
    ControllerCreate,
    ControllerSetComponentHandlerPreInit,
    ControllerInitialize,
    ComponentStateSync,
    ControllerSetComponentHandler,
    ConnectionPointHandshake,
    ViewCreate,
    ViewSetFrame,
    ViewAttach,
    Attached,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorHostValidationPhaseResultStatus {
    Completed,
    Failed,
    Skipped,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginEditorHostValidationPhaseResult {
    pub phase: PluginEditorHostValidationPhase,
    pub status: PluginEditorHostValidationPhaseResultStatus,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorHostLoadInitProbeStatus {
    BlockedByHostContract,
    BlockedByPluginFailure,
    LoadInitSafe,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginEditorHostLoadInitProbe {
    pub status: PluginEditorHostLoadInitProbeStatus,
    pub last_successful_phase: Option<PluginEditorHostValidationPhase>,
    pub first_failing_phase: Option<PluginEditorHostValidationPhase>,
    pub first_failing_message: Option<String>,
    pub summary: String,
    #[serde(default)]
    pub phase_results: Vec<PluginEditorHostValidationPhaseResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapabilityEvidenceStatus {
    NotProven,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginCapabilityEvidence {
    pub status: PluginCapabilityEvidenceStatus,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginEditorHostValidation {
    pub status: PluginEditorHostValidationStatus,
    pub phase: PluginEditorHostValidationPhase,
    pub message: String,
    #[serde(default)]
    pub capability_evidence: Option<PluginCapabilityEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintainer_load_init_probe: Option<PluginEditorHostLoadInitProbe>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginInventoryEntry {
    pub id: String,
    pub name: String,
    pub bundle_path: String,
    pub binary_path: Option<String>,
    pub source_root: String,
    pub availability: PluginInventoryAvailability,
    pub validation_failure: Option<PluginInventoryFailure>,
    #[serde(default)]
    pub editor_host_validation: Option<PluginEditorHostValidation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginInventoryState {
    pub entries: Vec<PluginInventoryEntry>,
    #[serde(default)]
    pub errors: Vec<PluginInventoryFailure>,
    pub last_discovered_at_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
struct PersistedPluginInventoryState {
    entries: Vec<PluginInventoryEntry>,
    last_discovered_at_ms: Option<u64>,
}

#[derive(Default)]
struct PluginInventoryDiscoveryResult {
    entries: Vec<PluginInventoryEntry>,
    errors: Vec<PluginInventoryFailure>,
}

#[derive(Clone, Debug)]
pub struct PluginInventoryContext {
    state: PluginInventoryState,
    db: Option<Db>,
    sticky_errors: Vec<PluginInventoryFailure>,
}

pub type PluginInventoryContextState = Mutex<PluginInventoryContext>;

impl PluginInventoryContext {
    pub fn new(app: &AppHandle) -> PluginInventoryContextState {
        let mut startup_errors = Vec::new();
        let db_path = match app.path().app_config_dir() {
            Ok(path) => Some(path.join("plugin_inventory")),
            Err(error) => {
                startup_errors.push(storage_failure(format!(
                    "Failed to resolve plugin inventory config directory: {error}"
                )));
                None
            }
        };
        let roots = default_vst3_search_roots();

        Mutex::new(Self::from_db_path_and_roots(
            db_path.as_deref(),
            &roots,
            startup_errors,
        ))
    }

    #[cfg(test)]
    fn from_db_path(db_path: Option<&Path>, startup_errors: Vec<PluginInventoryFailure>) -> Self {
        Self::from_db_path_and_roots(db_path, &[], startup_errors)
    }

    #[cfg(test)]
    pub fn with_entries_for_tests(entries: Vec<PluginInventoryEntry>) -> Self {
        Self {
            state: PluginInventoryState {
                entries,
                errors: vec![],
                last_discovered_at_ms: None,
            },
            db: None,
            sticky_errors: vec![],
        }
    }

    fn from_db_path_and_roots(
        db_path: Option<&Path>,
        roots: &[PathBuf],
        startup_errors: Vec<PluginInventoryFailure>,
    ) -> Self {
        let state = PluginInventoryState::default();
        let mut sticky_errors = startup_errors;
        let mut persisted_state = None;

        let db = match db_path {
            Some(path) => match sled::open(path) {
                Ok(db) => {
                    match load_persisted_state(&db) {
                        Ok(Some(loaded_state)) => {
                            persisted_state = Some(loaded_state);
                        }
                        Ok(None) => {}
                        Err(error) => {
                            sticky_errors.push(storage_failure(format!(
                                "Failed to load persisted plugin inventory: {error}"
                            )));
                        }
                    }
                    Some(db)
                }
                Err(error) => {
                    let failure = storage_failure(format!(
                        "Failed to open plugin inventory database '{}': {error}",
                        path.display()
                    ));
                    sticky_errors.push(failure);
                    None
                }
            },
            None => None,
        };

        let mut context = Self {
            state,
            db,
            sticky_errors,
        };

        if let Some(persisted_state) = persisted_state {
            context.state = validate_persisted_state(persisted_state, roots);
            if let Err(error) = context.persist_state() {
                context.state.errors.push(storage_failure(error));
            }
            context.append_sticky_errors();
        } else if roots.is_empty() {
            context.append_sticky_errors();
        } else if let Err(error) = context.discover_in_roots(roots) {
            context.state.errors.push(storage_failure(error));
            context.append_sticky_errors();
        }

        context
    }

    fn append_sticky_errors(&mut self) {
        self.state.errors.extend(self.sticky_errors.iter().cloned());
    }

    pub fn emit(&self, app: &AppHandle) {
        if let Err(error) = app.emit("soloc://state/plugin_inventory", &self.state) {
            eprintln!("Failed to emit plugin inventory state: {error:?}");
        }
    }

    #[cfg(test)]
    pub fn snapshot(&self) -> PluginInventoryState {
        self.state.clone()
    }

    pub fn resolve_available_bundle_path(&self, plugin_id: &str) -> Result<PathBuf, String> {
        resolve_available_bundle_path_in_entries(plugin_id, &self.state.entries)
    }

    pub fn update_editor_host_validation_for_bundle_path(
        &mut self,
        bundle_path: &Path,
        validation: PluginEditorHostValidation,
    ) -> Result<bool, String> {
        let Some(entry) = self
            .state
            .entries
            .iter_mut()
            .find(|entry| Path::new(&entry.bundle_path) == bundle_path)
        else {
            return Ok(false);
        };

        entry.editor_host_validation = Some(validation);
        self.persist_state()?;
        Ok(true)
    }

    pub fn discover(&mut self) -> Result<(), String> {
        self.discover_in_roots(&default_vst3_search_roots())
    }

    fn discover_in_roots(&mut self, roots: &[PathBuf]) -> Result<(), String> {
        let discovery = discover_inventory_in_roots(roots);
        self.state = PluginInventoryState {
            entries: discovery.entries,
            errors: discovery.errors,
            last_discovered_at_ms: Some(current_unix_time_ms()?),
        };

        self.append_sticky_errors();

        if let Err(error) = self.persist_state() {
            self.state.errors.push(storage_failure(error));
        }

        Ok(())
    }

    fn persist_state(&self) -> Result<(), String> {
        if let Some(ref db) = self.db {
            let payload = serde_json::to_vec(&PersistedPluginInventoryState::from(&self.state))
                .map_err(|error| error.to_string())?;
            db.insert(INVENTORY_STATE_KEY, payload)
                .map_err(|error| error.to_string())?;
            db.flush().map_err(|error| error.to_string())?;
        }

        Ok(())
    }
}

fn current_unix_time_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .map_err(|error| error.to_string())
}

pub fn blocked_editor_host_validation(message: String) -> PluginEditorHostValidation {
    PluginEditorHostValidation {
        status: PluginEditorHostValidationStatus::Blocked,
        phase: PluginEditorHostValidationPhase::PolicyGate,
        message,
        capability_evidence: None,
        maintainer_load_init_probe: None,
    }
}

fn inventory_editor_host_validation(plugin_path: &Path) -> Option<PluginEditorHostValidation> {
    bounded_editor_host_unsupported_reason(plugin_path).map(blocked_editor_host_validation)
}

fn resolve_available_bundle_path_in_entries(
    plugin_id: &str,
    entries: &[PluginInventoryEntry],
) -> Result<PathBuf, String> {
    let requested_plugin_id = plugin_id.trim();
    if requested_plugin_id.is_empty() {
        return Err(String::from("Plugin inventory target id cannot be empty."));
    }

    let legacy_bundle_path = requested_plugin_id
        .strip_prefix("vst3:")
        .filter(|value| !value.contains('#'));
    let entry = entries
        .iter()
        .find(|entry| entry.id == requested_plugin_id)
        .or_else(|| {
            legacy_bundle_path.and_then(|bundle_path| {
                entries
                    .iter()
                    .find(|entry| entry.bundle_path == bundle_path)
            })
        })
        .ok_or_else(|| {
            format!(
                "Plugin inventory does not contain the requested target: '{}'.",
                requested_plugin_id
            )
        })?;

    if entry.availability != PluginInventoryAvailability::Available {
        return Err(format!(
            "Plugin inventory target '{}' is unavailable.",
            entry.name
        ));
    }

    Ok(PathBuf::from(&entry.bundle_path))
}

fn load_persisted_state(db: &Db) -> Result<Option<PersistedPluginInventoryState>, String> {
    match db.get(INVENTORY_STATE_KEY) {
        Ok(Some(value)) => serde_json::from_slice(&value)
            .map(Some)
            .map_err(|error| error.to_string()),
        Ok(None) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn discover_inventory_in_roots(roots: &[PathBuf]) -> PluginInventoryDiscoveryResult {
    let mut result = PluginInventoryDiscoveryResult::default();

    for root in roots {
        let source_root = root.display().to_string();
        match list_vst3_candidates_in_root(root) {
            Ok(candidates) => {
                result.entries.extend(
                    candidates.into_iter().flat_map(|candidate| {
                        build_inventory_entries(candidate, source_root.clone())
                    }),
                );
            }
            Err(error) => result.errors.push(PluginInventoryFailure {
                kind: PluginInventoryFailureKind::IoError,
                message: error,
            }),
        }
    }

    result
        .entries
        .sort_by(|left, right| left.bundle_path.cmp(&right.bundle_path));
    result
        .errors
        .sort_by(|left, right| left.message.cmp(&right.message));

    result
}

fn validate_persisted_state(
    persisted_state: PersistedPluginInventoryState,
    roots: &[PathBuf],
) -> PluginInventoryState {
    let mut entries = persisted_state
        .entries
        .into_iter()
        .flat_map(validate_cached_entries)
        .collect::<Vec<_>>();
    let cached_bundle_paths = entries
        .iter()
        .map(|entry| entry.bundle_path.clone())
        .collect::<BTreeSet<_>>();
    let uncached_discovery = discover_uncached_inventory_in_roots(roots, &cached_bundle_paths);
    let last_discovered_at_ms = if uncached_discovery.entries.is_empty() {
        persisted_state.last_discovered_at_ms
    } else {
        current_unix_time_ms()
            .ok()
            .or(persisted_state.last_discovered_at_ms)
    };

    entries.extend(uncached_discovery.entries);
    entries.sort_by(|left, right| left.bundle_path.cmp(&right.bundle_path));

    PluginInventoryState {
        entries,
        errors: uncached_discovery.errors,
        last_discovered_at_ms,
    }
}

fn discover_uncached_inventory_in_roots(
    roots: &[PathBuf],
    cached_bundle_paths: &BTreeSet<String>,
) -> PluginInventoryDiscoveryResult {
    let mut result = PluginInventoryDiscoveryResult::default();

    for root in roots {
        let source_root = root.display().to_string();
        match list_vst3_candidates_in_root(root) {
            Ok(candidates) => {
                result.entries.extend(
                    candidates
                        .into_iter()
                        .filter_map(|candidate| {
                            let bundle_path = candidate.display().to_string();

                            if cached_bundle_paths.contains(&bundle_path) {
                                None
                            } else {
                                Some(build_inventory_entries(candidate, source_root.clone()))
                            }
                        })
                        .flatten(),
                );
            }
            Err(error) => result.errors.push(PluginInventoryFailure {
                kind: PluginInventoryFailureKind::IoError,
                message: error,
            }),
        }
    }

    result
        .entries
        .sort_by(|left, right| left.bundle_path.cmp(&right.bundle_path));
    result
        .errors
        .sort_by(|left, right| left.message.cmp(&right.message));

    result
}

fn validate_cached_entries(entry: PluginInventoryEntry) -> Vec<PluginInventoryEntry> {
    let bundle_path = PathBuf::from(&entry.bundle_path);

    match validate_vst3_bundle(&bundle_path) {
        Ok(()) => {
            if !moduleinfo_audio_classes(&bundle_path).is_empty() {
                return build_inventory_entries(bundle_path, entry.source_root);
            }
            match vst3_bundle_binary_path(&bundle_path) {
                Ok(binary_path) => vec![PluginInventoryEntry {
                    availability: PluginInventoryAvailability::Available,
                    binary_path: Some(binary_path.display().to_string()),
                    validation_failure: None,
                    editor_host_validation: inventory_editor_host_validation(&bundle_path),
                    ..entry
                }],
                Err(error) => vec![unavailable_cached_entry(entry, error)],
            }
        }
        Err(error) => vec![unavailable_cached_entry(entry, error)],
    }
}

fn unavailable_cached_entry(
    entry: PluginInventoryEntry,
    error: Vst3BundleError,
) -> PluginInventoryEntry {
    PluginInventoryEntry {
        availability: PluginInventoryAvailability::Unavailable,
        binary_path: None,
        validation_failure: Some(PluginInventoryFailure::from(error)),
        editor_host_validation: None,
        ..entry
    }
}

fn moduleinfo_audio_classes(candidate: &Path) -> Vec<(String, String)> {
    let metadata_path = candidate
        .join("Contents")
        .join("Resources")
        .join("moduleinfo.json");
    let Ok(metadata) = std::fs::read_to_string(metadata_path) else {
        return Vec::new();
    };
    let Ok(metadata) = json5::from_str::<serde_json::Value>(&metadata) else {
        return Vec::new();
    };

    metadata
        .get("Classes")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|class| {
            class.get("Category").and_then(serde_json::Value::as_str) == Some("Audio Module Class")
        })
        .filter_map(|class| {
            let cid = class.get("CID")?.as_str()?;
            let name = class.get("Name")?.as_str()?;
            (cid.len() == 32 && cid.bytes().all(|byte| byte.is_ascii_hexdigit()))
                .then(|| (cid.to_ascii_uppercase(), name.to_string()))
        })
        .collect()
}

fn build_inventory_entries(candidate: PathBuf, source_root: String) -> Vec<PluginInventoryEntry> {
    let bundle_path = candidate.display().to_string();
    let fallback_name = candidate
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Unknown Plugin")
        .to_string();

    let validation_failure = match validate_vst3_bundle(&candidate) {
        Ok(()) => match vst3_bundle_binary_path(&candidate) {
            Ok(binary_path) => {
                let classes = moduleinfo_audio_classes(&candidate);
                let classes = if classes.is_empty() {
                    vec![(String::new(), fallback_name)]
                } else {
                    classes
                };
                return classes
                    .into_iter()
                    .map(|(cid, name)| PluginInventoryEntry {
                        id: if cid.is_empty() {
                            format!("vst3:{bundle_path}")
                        } else {
                            format!("vst3:{bundle_path}#{cid}")
                        },
                        name,
                        bundle_path: bundle_path.clone(),
                        binary_path: Some(binary_path.display().to_string()),
                        source_root: source_root.clone(),
                        availability: PluginInventoryAvailability::Available,
                        validation_failure: None,
                        editor_host_validation: inventory_editor_host_validation(&candidate),
                    })
                    .collect();
            }
            Err(error) => PluginInventoryFailure::from(error),
        },
        Err(error) => PluginInventoryFailure::from(error),
    };

    vec![PluginInventoryEntry {
        id: format!("vst3:{}", bundle_path),
        name: fallback_name,
        bundle_path,
        binary_path: None,
        source_root,
        availability: PluginInventoryAvailability::Unavailable,
        validation_failure: Some(validation_failure),
        editor_host_validation: None,
    }]
}

pub(crate) fn component_cid_from_plugin_id(plugin_id: &str) -> Option<TUID> {
    let cid = plugin_id.rsplit_once('#')?.1;
    if cid.len() != 32 {
        return None;
    }
    let mut bytes = [0_i8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&cid[offset..offset + 2], 16).ok()? as i8;
    }
    Some(bytes)
}

impl From<Vst3BundleError> for PluginInventoryFailure {
    fn from(error: Vst3BundleError) -> Self {
        let kind = match error.kind {
            Vst3BundleErrorKind::InvalidBundle => PluginInventoryFailureKind::InvalidBundle,
            Vst3BundleErrorKind::MissingBundleBinary => {
                PluginInventoryFailureKind::MissingBundleBinary
            }
            Vst3BundleErrorKind::IoError => PluginInventoryFailureKind::IoError,
        };

        Self {
            kind,
            message: error.message,
        }
    }
}

impl From<PersistedPluginInventoryState> for PluginInventoryState {
    fn from(value: PersistedPluginInventoryState) -> Self {
        Self {
            entries: value.entries,
            errors: Vec::new(),
            last_discovered_at_ms: value.last_discovered_at_ms,
        }
    }
}

impl From<&PluginInventoryState> for PersistedPluginInventoryState {
    fn from(value: &PluginInventoryState) -> Self {
        Self {
            entries: value.entries.clone(),
            last_discovered_at_ms: value.last_discovered_at_ms,
        }
    }
}

fn storage_failure(message: String) -> PluginInventoryFailure {
    PluginInventoryFailure {
        kind: PluginInventoryFailureKind::StorageError,
        message,
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn plugin_inventory_discover(
    app: AppHandle,
    state: State<'_, PluginInventoryContextState>,
) -> Result<(), String> {
    {
        let mut inventory = state.lock().unwrap();
        inventory.discover()?;
        inventory.emit(&app);
    }

    let app_state_handle = app.state::<AppContextState>();
    let renderer_state_handle = app.state::<RendererContextState>();
    let mut app_state = app_state_handle.lock().unwrap();
    app_state.refresh_playback_activation_runtime_from_app(&app);

    let mut renderer = renderer_state_handle.lock().unwrap();
    app_state.emit_general_state(&app, &mut renderer, None);

    Ok(())
}

// ── VST3 bundle discovery ────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Vst3BundleErrorKind {
    InvalidBundle,
    MissingBundleBinary,
    IoError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3BundleError {
    pub kind: Vst3BundleErrorKind,
    pub message: String,
}

impl Vst3BundleError {
    fn invalid_bundle(message: String) -> Self {
        Self {
            kind: Vst3BundleErrorKind::InvalidBundle,
            message,
        }
    }

    fn missing_bundle_binary(message: String) -> Self {
        Self {
            kind: Vst3BundleErrorKind::MissingBundleBinary,
            message,
        }
    }

    fn io_error(message: String) -> Self {
        Self {
            kind: Vst3BundleErrorKind::IoError,
            message,
        }
    }
}

pub const MACOS_VST3_SEARCH_ROOTS: [&str; 2] = [
    "~/Library/Audio/Plug-Ins/VST3",
    "/Library/Audio/Plug-Ins/VST3",
];

pub fn default_vst3_search_roots() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        MACOS_VST3_SEARCH_ROOTS
            .iter()
            .filter_map(|root| expand_home_dir(root))
            .filter(|root| root.is_dir())
            .collect()
    }

    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

pub fn expand_home_dir(path: &str) -> Option<PathBuf> {
    if path == "~" {
        return std::env::var_os("HOME").map(PathBuf::from);
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        return std::env::var_os("HOME").map(|home| PathBuf::from(home).join(stripped));
    }

    Some(PathBuf::from(path))
}

pub fn validate_vst3_bundle(path: &Path) -> Result<(), Vst3BundleError> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("vst3") {
        return Err(Vst3BundleError::invalid_bundle(format!(
            "Expected a .vst3 bundle path, got '{}'.",
            path.display()
        )));
    }

    if !path.exists() {
        return Err(Vst3BundleError::invalid_bundle(format!(
            "VST3 target does not exist: '{}'.",
            path.display()
        )));
    }

    if !path.is_dir() {
        return Err(Vst3BundleError::invalid_bundle(format!(
            "Expected a VST3 bundle directory, got '{}'.",
            path.display()
        )));
    }

    Ok(())
}

pub fn vst3_bundle_binary_path(path: &Path) -> Result<PathBuf, Vst3BundleError> {
    let macos_dir = path.join("Contents").join("MacOS");
    if !macos_dir.is_dir() {
        return Err(Vst3BundleError::missing_bundle_binary(format!(
            "Expected a Contents/MacOS directory inside '{}'.",
            path.display()
        )));
    }

    if let Some(bundle_name) = path.file_stem().and_then(|value| value.to_str()) {
        let preferred = macos_dir.join(bundle_name);
        if preferred.is_file() {
            return Ok(preferred);
        }
    }

    let mut binaries = Vec::new();
    let read_dir = std::fs::read_dir(&macos_dir).map_err(|error| {
        Vst3BundleError::io_error(format!(
            "Failed to read VST3 bundle binary directory '{}': {error}",
            macos_dir.display()
        ))
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|error| {
            Vst3BundleError::io_error(format!(
                "Failed to inspect VST3 bundle binary entry in '{}': {error}",
                macos_dir.display()
            ))
        })?;
        let path = entry.path();

        if path.is_file() {
            binaries.push(path);
        }
    }

    binaries.sort();

    match binaries.len() {
        0 => Err(Vst3BundleError::missing_bundle_binary(format!(
            "Did not find a bundle executable inside '{}'.",
            macos_dir.display()
        ))),
        _ => Ok(binaries.remove(0)),
    }
}

pub fn list_vst3_candidates_in_root(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut entries = Vec::new();
    let mut roots_to_visit = vec![root.to_path_buf()];
    let mut visited_roots = BTreeSet::new();

    while let Some(current_root) = roots_to_visit.pop() {
        let visited_key = std::fs::canonicalize(&current_root).unwrap_or(current_root.clone());
        if !visited_roots.insert(visited_key) {
            continue;
        }

        let read_dir = std::fs::read_dir(&current_root).map_err(|error| {
            format!(
                "Failed to read VST3 discovery root '{}': {error}",
                current_root.display()
            )
        })?;

        for entry in read_dir {
            let entry = entry.map_err(|error| {
                format!(
                    "Failed to inspect VST3 discovery root entry in '{}': {error}",
                    current_root.display()
                )
            })?;
            let path = entry.path();

            if path.extension().and_then(|value| value.to_str()) == Some("vst3") {
                entries.push(path);
                continue;
            }

            if path.is_dir() {
                roots_to_visit.push(path);
            }
        }
    }

    entries.sort();
    Ok(entries)
}

pub fn discover_first_valid_vst3_in_roots(roots: &[PathBuf]) -> Option<PathBuf> {
    for root in roots {
        if let Ok(candidates) = list_vst3_candidates_in_root(root) {
            for path in candidates {
                if validate_vst3_bundle(&path).is_ok() {
                    return Some(path);
                }
            }
        }
    }

    None
}

// ── Plugin host compatibility ────────────────────────────────────────

struct UnsupportedEditorHostBundle {
    name: &'static str,
    reason: &'static str,
}

const UNSUPPORTED_EDITOR_HOST_BUNDLES: &[UnsupportedEditorHostBundle] = &[
    UnsupportedEditorHostBundle {
        name: "HALion Sonic SE",
        reason: "it returns a null plugin factory during factory creation on the bounded seam.",
    },
    UnsupportedEditorHostBundle {
        name: "Groove Agent SE",
        reason: "it crashes during controller initialization.",
    },
    UnsupportedEditorHostBundle {
        name: "Kontakt 7",
        reason: "it crashes during component createInstance.",
    },
    UnsupportedEditorHostBundle {
        name: "MT-PowerDrumKit",
        reason: "it currently requires an MVP-host teardown exception we do not want to carry in the bounded seam.",
    },
];

fn bundle_stem(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|value| value.to_str())
}

fn unsupported_editor_host_bundle(path: &Path) -> Option<&'static UnsupportedEditorHostBundle> {
    let bundle_stem = bundle_stem(path)?;

    UNSUPPORTED_EDITOR_HOST_BUNDLES
        .iter()
        .find(|bundle| bundle_stem.eq_ignore_ascii_case(bundle.name))
}

pub fn bounded_editor_host_unsupported_reason(plugin_path: &Path) -> Option<String> {
    unsupported_editor_host_bundle(plugin_path).map(|bundle| {
        format!(
            "Unsupported plugin target for the current macOS native editor host POC: {} remains on the current unsupported list for the bounded seam because {} Other discovered bundles remain eligible unless they demonstrate host-seam issues and are added to this unsupported list.",
            bundle.name, bundle.reason
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        blocked_editor_host_validation, bounded_editor_host_unsupported_reason,
        build_inventory_entries, component_cid_from_plugin_id, discover_first_valid_vst3_in_roots,
        discover_inventory_in_roots, expand_home_dir, list_vst3_candidates_in_root,
        load_persisted_state, resolve_available_bundle_path_in_entries, validate_vst3_bundle,
        vst3_bundle_binary_path, PluginEditorHostLoadInitProbe,
        PluginEditorHostLoadInitProbeStatus, PluginEditorHostValidation,
        PluginEditorHostValidationPhase, PluginEditorHostValidationPhaseResult,
        PluginEditorHostValidationPhaseResultStatus, PluginEditorHostValidationStatus,
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
        PluginInventoryFailure, PluginInventoryFailureKind, Vst3BundleErrorKind,
    };
    use sled::Db;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    fn available_entry(id: &str, name: &str, bundle_path: &str) -> PluginInventoryEntry {
        PluginInventoryEntry {
            id: id.to_string(),
            name: name.to_string(),
            bundle_path: bundle_path.to_string(),
            binary_path: Some(format!("{bundle_path}/Contents/MacOS/{name}")),
            source_root: String::from("/Library/Audio/Plug-Ins/VST3"),
            availability: PluginInventoryAvailability::Available,
            validation_failure: None,
            editor_host_validation: None,
        }
    }

    fn create_valid_bundle(root: &Path, name: &str) -> PathBuf {
        let bundle = root.join(format!("{name}.vst3"));
        let macos_dir = bundle.join("Contents").join("MacOS");
        std::fs::create_dir_all(&macos_dir).unwrap();
        std::fs::write(macos_dir.join(name), b"binary").unwrap();
        bundle
    }

    fn open_db(path: &std::path::Path) -> Db {
        sled::open(path).unwrap()
    }

    #[test]
    fn component_cid_parses_canonical_inventory_suffix() {
        assert_eq!(
            component_cid_from_plugin_id("vst3:/tmp/Synth.vst3#00112233445566778899AABBCCDDEEFF"),
            Some([
                0x00,
                0x11,
                0x22,
                0x33,
                0x44,
                0x55,
                0x66,
                0x77,
                0x88_u8 as i8,
                0x99_u8 as i8,
                0xAA_u8 as i8,
                0xBB_u8 as i8,
                0xCC_u8 as i8,
                0xDD_u8 as i8,
                0xEE_u8 as i8,
                0xFF_u8 as i8,
            ])
        );
        assert_eq!(
            component_cid_from_plugin_id("vst3:/tmp/Synth.vst3#xyz"),
            None
        );
    }

    #[test]
    fn inventory_expands_moduleinfo_audio_classes_with_stable_cids() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-moduleinfo-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle = create_valid_bundle(&root, "MultiSynth");
        let resources = bundle.join("Contents").join("Resources");
        std::fs::create_dir_all(&resources).unwrap();
        std::fs::write(
            resources.join("moduleinfo.json"),
            r#"{
                "Classes": [
                    { "CID": "00112233445566778899AABBCCDDEEFF", "Category": "Audio Module Class", "Name": "Synth A", },
                    { "CID": "FFEEDDCCBBAA99887766554433221100", "Category": "Audio Module Class", "Name": "Synth B", },
                    { "CID": "11111111111111111111111111111111", "Category": "Component Controller Class", "Name": "Controller", },
                ],
            }"#,
        )
        .unwrap();

        let entries = build_inventory_entries(bundle.clone(), root.display().to_string());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "Synth A");
        assert_eq!(
            entries[0].id,
            format!("vst3:{}#00112233445566778899AABBCCDDEEFF", bundle.display())
        );
        assert_eq!(entries[1].name, "Synth B");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_available_bundle_path_returns_matching_available_entry() {
        let bundle_path = "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3";
        let resolved = resolve_available_bundle_path_in_entries(
            "vst3:/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
            &[available_entry(
                "vst3:/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
                "Grand Piano",
                bundle_path,
            )],
        )
        .unwrap();

        assert_eq!(resolved, PathBuf::from(bundle_path));
    }

    #[test]
    fn resolve_available_bundle_path_rejects_missing_inventory_ids() {
        let error = resolve_available_bundle_path_in_entries(
            "vst3:/Library/Audio/Plug-Ins/VST3/Missing.vst3",
            &[available_entry(
                "vst3:/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
                "Grand Piano",
                "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
            )],
        )
        .unwrap_err();

        assert!(error.contains("does not contain the requested target"));
    }

    #[test]
    fn resolve_available_bundle_path_rejects_unavailable_entries() {
        let error = resolve_available_bundle_path_in_entries(
            "vst3:/Library/Audio/Plug-Ins/VST3/Broken.vst3",
            &[PluginInventoryEntry {
                availability: PluginInventoryAvailability::Unavailable,
                validation_failure: Some(PluginInventoryFailure {
                    kind: PluginInventoryFailureKind::MissingBundleBinary,
                    message: String::from("Missing Contents/MacOS binary"),
                }),
                editor_host_validation: None,
                ..available_entry(
                    "vst3:/Library/Audio/Plug-Ins/VST3/Broken.vst3",
                    "Broken Plugin",
                    "/Library/Audio/Plug-Ins/VST3/Broken.vst3",
                )
            }],
        )
        .unwrap_err();

        assert!(error.contains("is unavailable"));
    }

    #[test]
    fn discover_inventory_in_roots_keeps_deterministic_order_and_records_failures() {
        let root =
            std::env::temp_dir().join(format!("plugin-inventory-discover-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::create_dir_all(root.join("Zoo.vst3")).unwrap();
        create_valid_bundle(&root, "Alpha");
        std::fs::write(root.join("Broken.vst3"), b"not-a-bundle").unwrap();

        let discovery = discover_inventory_in_roots(std::slice::from_ref(&root));
        let entries = discovery.entries;

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Alpha", "Broken", "Zoo"]
        );
        assert_eq!(
            entries[0].availability,
            PluginInventoryAvailability::Available
        );
        assert_eq!(
            entries[1].availability,
            PluginInventoryAvailability::Unavailable
        );
        assert_eq!(
            entries[1]
                .validation_failure
                .as_ref()
                .map(|failure| &failure.kind),
            Some(&PluginInventoryFailureKind::InvalidBundle)
        );
        assert_eq!(
            entries[2].availability,
            PluginInventoryAvailability::Unavailable
        );
        assert_eq!(
            entries[2]
                .validation_failure
                .as_ref()
                .map(|failure| &failure.kind),
            Some(&PluginInventoryFailureKind::MissingBundleBinary)
        );
        assert!(discovery.errors.is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn discover_inventory_in_roots_records_root_read_errors() {
        let root =
            std::env::temp_dir().join(format!("plugin-inventory-root-file-{}", std::process::id()));
        let _ = std::fs::remove_file(&root);
        std::fs::write(&root, b"not-a-directory").unwrap();

        let discovery = discover_inventory_in_roots(std::slice::from_ref(&root));
        assert!(discovery.entries.is_empty());
        assert_eq!(discovery.errors.len(), 1);
        assert_eq!(
            discovery.errors[0].kind,
            PluginInventoryFailureKind::IoError
        );
        assert!(discovery.errors[0]
            .message
            .contains("Failed to read VST3 discovery root"));

        let _ = std::fs::remove_file(&root);
    }

    #[test]
    fn plugin_inventory_context_persists_and_reloads_state() {
        let root =
            std::env::temp_dir().join(format!("plugin-inventory-persist-{}", std::process::id()));
        let db_path = root.join("db");
        let discovery_root = root.join("plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&discovery_root).unwrap();
        create_valid_bundle(&discovery_root, "Alpha");

        let mut context = PluginInventoryContext::from_db_path(Some(&db_path), Vec::new());
        context
            .discover_in_roots(std::slice::from_ref(&discovery_root))
            .unwrap();
        let snapshot = context.snapshot();
        drop(context);

        let reloaded = PluginInventoryContext::from_db_path(Some(&db_path), Vec::new());
        assert_eq!(reloaded.snapshot(), snapshot);
        assert_eq!(reloaded.snapshot().entries.len(), 1);
        assert!(reloaded.snapshot().errors.is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plugin_inventory_startup_reuses_cached_entries_when_no_new_bundles_exist() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-startup-cache-reuse-{}",
            std::process::id()
        ));
        let db_path = root.join("db");
        let cached_root = root.join("cached-plugins");
        let empty_root = root.join("other-plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&cached_root).unwrap();
        std::fs::create_dir_all(&empty_root).unwrap();
        create_valid_bundle(&cached_root, "Alpha");

        let mut context = PluginInventoryContext::from_db_path(Some(&db_path), Vec::new());
        context
            .discover_in_roots(std::slice::from_ref(&cached_root))
            .unwrap();
        drop(context);

        let reloaded = PluginInventoryContext::from_db_path_and_roots(
            Some(&db_path),
            std::slice::from_ref(&empty_root),
            Vec::new(),
        );
        let snapshot = reloaded.snapshot();

        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].name, "Alpha");
        assert_eq!(
            snapshot.entries[0].availability,
            PluginInventoryAvailability::Available
        );
        assert!(snapshot.errors.is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plugin_inventory_startup_treats_empty_persisted_state_as_cache_present() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-startup-empty-cache-present-{}",
            std::process::id()
        ));
        let db_path = root.join("db");
        let discovery_root = root.join("plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&discovery_root).unwrap();

        let db = open_db(&db_path);
        let payload = serde_json::to_vec(&super::PersistedPluginInventoryState {
            entries: vec![],
            last_discovered_at_ms: Some(123),
        })
        .unwrap();
        db.insert(super::INVENTORY_STATE_KEY, payload).unwrap();
        db.flush().unwrap();
        drop(db);

        let reloaded = PluginInventoryContext::from_db_path_and_roots(
            Some(&db_path),
            std::slice::from_ref(&discovery_root),
            Vec::new(),
        );
        let snapshot = reloaded.snapshot();

        assert!(snapshot.entries.is_empty());
        assert!(snapshot.errors.is_empty());
        assert_eq!(snapshot.last_discovered_at_ms, Some(123));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn manual_full_rescan_replaces_bounded_startup_snapshot_and_persists_new_truth() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-manual-full-rescan-{}",
            std::process::id()
        ));
        let db_path = root.join("db");
        let cached_root = root.join("cached-plugins");
        let startup_root = root.join("startup-plugins");
        let manual_root = root.join("manual-plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&cached_root).unwrap();
        std::fs::create_dir_all(&startup_root).unwrap();
        std::fs::create_dir_all(&manual_root).unwrap();
        create_valid_bundle(&cached_root, "Alpha");
        create_valid_bundle(&manual_root, "Beta");

        let mut seeded = PluginInventoryContext::from_db_path(Some(&db_path), Vec::new());
        seeded
            .discover_in_roots(std::slice::from_ref(&cached_root))
            .unwrap();
        drop(seeded);

        let mut reloaded = PluginInventoryContext::from_db_path_and_roots(
            Some(&db_path),
            std::slice::from_ref(&startup_root),
            Vec::new(),
        );
        let startup_snapshot = reloaded.snapshot();

        assert_eq!(startup_snapshot.entries.len(), 1);
        assert_eq!(startup_snapshot.entries[0].name, "Alpha");

        reloaded
            .discover_in_roots(std::slice::from_ref(&manual_root))
            .unwrap();
        let rescan_snapshot = reloaded.snapshot();
        drop(reloaded);

        assert_eq!(rescan_snapshot.entries.len(), 1);
        assert_eq!(rescan_snapshot.entries[0].name, "Beta");
        assert_eq!(
            rescan_snapshot.entries[0].availability,
            PluginInventoryAvailability::Available
        );
        assert!(rescan_snapshot.last_discovered_at_ms.is_some());

        let persisted = load_persisted_state(&open_db(&db_path)).unwrap().unwrap();
        assert_eq!(persisted.entries.len(), 1);
        assert_eq!(persisted.entries[0].name, "Beta");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plugin_inventory_startup_discovers_new_uncached_bundles_during_bounded_sweep() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-startup-new-bundles-{}",
            std::process::id()
        ));
        let db_path = root.join("db");
        let cached_root = root.join("cached-plugins");
        let new_root = root.join("new-plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&cached_root).unwrap();
        std::fs::create_dir_all(&new_root).unwrap();
        create_valid_bundle(&cached_root, "Alpha");
        create_valid_bundle(&new_root, "Beta");

        let mut context = PluginInventoryContext::from_db_path(Some(&db_path), Vec::new());
        context
            .discover_in_roots(std::slice::from_ref(&cached_root))
            .unwrap();
        let alpha_id = context.snapshot().entries[0].id.clone();
        drop(context);

        let reloaded = PluginInventoryContext::from_db_path_and_roots(
            Some(&db_path),
            std::slice::from_ref(&new_root),
            Vec::new(),
        );
        let snapshot = reloaded.snapshot();

        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(snapshot.entries[0].id, alpha_id);
        assert_eq!(snapshot.entries[0].name, "Alpha");
        assert_eq!(snapshot.entries[1].name, "Beta");
        assert_eq!(
            snapshot.entries[1].availability,
            PluginInventoryAvailability::Available
        );
        assert!(snapshot.errors.is_empty());

        drop(reloaded);
        let persisted = load_persisted_state(&open_db(&db_path)).unwrap().unwrap();
        assert_eq!(persisted.entries.len(), 2);
        assert_eq!(persisted.entries[1].name, "Beta");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plugin_inventory_startup_discovers_when_no_cache_exists() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-startup-discover-{}",
            std::process::id()
        ));
        let db_path = root.join("db");
        let discovery_root = root.join("plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&discovery_root).unwrap();
        create_valid_bundle(&discovery_root, "Alpha");

        let context = PluginInventoryContext::from_db_path_and_roots(
            Some(&db_path),
            std::slice::from_ref(&discovery_root),
            Vec::new(),
        );
        let snapshot = context.snapshot();
        drop(context);

        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].name, "Alpha");
        assert_eq!(
            snapshot.entries[0].availability,
            PluginInventoryAvailability::Available
        );
        assert!(snapshot.last_discovered_at_ms.is_some());

        let persisted = load_persisted_state(&open_db(&db_path)).unwrap().unwrap();
        assert_eq!(persisted.entries.len(), 1);
        assert_eq!(persisted.entries[0].name, "Alpha");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plugin_inventory_startup_marks_missing_cached_entries_unavailable() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-startup-stale-entry-{}",
            std::process::id()
        ));
        let db_path = root.join("db");
        let discovery_root = root.join("plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&discovery_root).unwrap();
        let bundle = create_valid_bundle(&discovery_root, "Alpha");

        let mut context = PluginInventoryContext::from_db_path(Some(&db_path), Vec::new());
        context
            .discover_in_roots(std::slice::from_ref(&discovery_root))
            .unwrap();
        let persisted_id = context.snapshot().entries[0].id.clone();
        drop(context);

        std::fs::remove_dir_all(&bundle).unwrap();

        let reloaded = PluginInventoryContext::from_db_path_and_roots(
            Some(&db_path),
            std::slice::from_ref(&discovery_root),
            Vec::new(),
        );
        let snapshot = reloaded.snapshot();

        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].id, persisted_id);
        assert_eq!(
            snapshot.entries[0].availability,
            PluginInventoryAvailability::Unavailable
        );
        assert_eq!(
            snapshot.entries[0]
                .validation_failure
                .as_ref()
                .map(|failure| &failure.kind),
            Some(&PluginInventoryFailureKind::InvalidBundle)
        );
        assert!(snapshot.entries[0]
            .validation_failure
            .as_ref()
            .unwrap()
            .message
            .contains("does not exist"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plugin_inventory_startup_recovers_from_load_failures_via_discovery() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-startup-load-failure-{}",
            std::process::id()
        ));
        let db_path = root.join("db");
        let discovery_root = root.join("plugins");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&discovery_root).unwrap();
        create_valid_bundle(&discovery_root, "Alpha");

        let db = open_db(&db_path);
        db.insert(super::INVENTORY_STATE_KEY, b"not-json".as_slice())
            .unwrap();
        db.flush().unwrap();
        drop(db);

        let context = PluginInventoryContext::from_db_path_and_roots(
            Some(&db_path),
            std::slice::from_ref(&discovery_root),
            Vec::new(),
        );
        let snapshot = context.snapshot();
        drop(context);

        assert_eq!(snapshot.entries.len(), 1);
        assert_eq!(snapshot.entries[0].name, "Alpha");
        assert_eq!(
            snapshot.entries[0].availability,
            PluginInventoryAvailability::Available
        );
        assert_eq!(snapshot.errors.len(), 1);
        assert_eq!(
            snapshot.errors[0].kind,
            PluginInventoryFailureKind::StorageError
        );
        assert!(snapshot.errors[0]
            .message
            .contains("Failed to load persisted plugin inventory"));

        let persisted = load_persisted_state(&open_db(&db_path)).unwrap().unwrap();
        assert_eq!(persisted.entries.len(), 1);
        assert_eq!(persisted.entries[0].name, "Alpha");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plugin_inventory_context_reports_storage_open_failures_explicitly() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-open-failure-{}",
            std::process::id()
        ));
        let db_path = root.join("db-file");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&db_path, b"not-a-directory").unwrap();

        let context = PluginInventoryContext::from_db_path(Some(&db_path), Vec::new());
        let snapshot = context.snapshot();

        assert!(snapshot.entries.is_empty());
        assert_eq!(snapshot.errors.len(), 1);
        assert_eq!(
            snapshot.errors[0].kind,
            PluginInventoryFailureKind::StorageError
        );
        assert!(snapshot.errors[0]
            .message
            .contains("Failed to open plugin inventory database"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn discover_inventory_marks_known_unsupported_editor_hosts_without_hiding_bundle_availability()
    {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-unsupported-editor-host-{}",
            std::process::id()
        ));
        let steinberg_root = root.join("Steinberg");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&steinberg_root).unwrap();
        create_valid_bundle(&steinberg_root, "HALion Sonic SE");

        let discovery = discover_inventory_in_roots(std::slice::from_ref(&root));

        assert_eq!(discovery.entries.len(), 1);
        assert_eq!(discovery.entries[0].name, "HALion Sonic SE");
        assert_eq!(
            discovery.entries[0].availability,
            PluginInventoryAvailability::Available
        );
        assert_eq!(
            discovery.entries[0]
                .editor_host_validation
                .as_ref()
                .map(|validation| &validation.status),
            Some(&PluginEditorHostValidationStatus::Blocked)
        );
        assert_eq!(
            discovery.entries[0]
                .editor_host_validation
                .as_ref()
                .map(|validation| &validation.phase),
            Some(&PluginEditorHostValidationPhase::PolicyGate)
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn discover_inventory_leaves_halion_sonic_available_for_real_path_investigation() {
        let root = std::env::temp_dir().join(format!(
            "plugin-inventory-halion-sonic-{}",
            std::process::id()
        ));
        let steinberg_root = root.join("Steinberg");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&steinberg_root).unwrap();
        create_valid_bundle(&steinberg_root, "HALion Sonic");

        let discovery = discover_inventory_in_roots(std::slice::from_ref(&root));

        assert_eq!(discovery.entries.len(), 1);
        assert_eq!(discovery.entries[0].name, "HALion Sonic");
        assert_eq!(
            discovery.entries[0].availability,
            PluginInventoryAvailability::Available
        );
        assert_eq!(discovery.entries[0].validation_failure, None);
        assert_eq!(discovery.entries[0].editor_host_validation, None);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn update_editor_host_validation_updates_matching_entry() {
        let mut context = PluginInventoryContext::with_entries_for_tests(vec![available_entry(
            "plugin-a",
            "Plugin A",
            "/tmp/plugin-a.vst3",
        )]);

        let updated = context
            .update_editor_host_validation_for_bundle_path(
                Path::new("/tmp/plugin-a.vst3"),
                blocked_editor_host_validation(String::from("blocked")),
            )
            .unwrap();

        assert!(updated);
        assert_eq!(
            context.snapshot().entries[0].editor_host_validation,
            Some(blocked_editor_host_validation(String::from("blocked")))
        );
    }

    #[test]
    fn maintainer_load_init_probe_round_trips_through_serialization() {
        let validation = PluginEditorHostValidation {
            status: PluginEditorHostValidationStatus::Blocked,
            phase: PluginEditorHostValidationPhase::PolicyGate,
            message: String::from("blocked"),
            capability_evidence: None,
            maintainer_load_init_probe: Some(PluginEditorHostLoadInitProbe {
                status: PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure,
                last_successful_phase: Some(PluginEditorHostValidationPhase::ControllerCreate),
                first_failing_phase: Some(PluginEditorHostValidationPhase::ControllerInitialize),
                first_failing_message: Some(String::from("controller init failed")),
                summary: String::from(
                    "Maintainer load/init probe first failed at controller_initialize and appears plugin-side.",
                ),
                phase_results: vec![
                    PluginEditorHostValidationPhaseResult {
                        phase: PluginEditorHostValidationPhase::ModuleLoad,
                        status: PluginEditorHostValidationPhaseResultStatus::Completed,
                        message: String::from("Loaded module."),
                    },
                    PluginEditorHostValidationPhaseResult {
                        phase: PluginEditorHostValidationPhase::ControllerInitialize,
                        status: PluginEditorHostValidationPhaseResultStatus::Failed,
                        message: String::from("controller init failed"),
                    },
                ],
            }),
        };

        let serialized = serde_json::to_string(&validation).unwrap();
        let deserialized: PluginEditorHostValidation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, validation);
    }

    // ── VST3 bundle discovery tests ──────────────────────────────────

    #[test]
    fn validate_vst3_bundle_rejects_non_vst3_paths() {
        let error = validate_vst3_bundle(Path::new("/tmp/test-plugin.component")).unwrap_err();
        assert_eq!(error.kind, Vst3BundleErrorKind::InvalidBundle);
        assert!(error.message.contains(".vst3 bundle path"));
    }

    #[test]
    fn validate_vst3_bundle_rejects_missing_paths() {
        let error = validate_vst3_bundle(Path::new("/tmp/missing-test-plugin.vst3")).unwrap_err();
        assert_eq!(error.kind, Vst3BundleErrorKind::InvalidBundle);
        assert!(error.message.contains("does not exist"));
    }

    #[test]
    fn validate_vst3_bundle_rejects_non_directory_vst3_paths() {
        let path = std::env::temp_dir().join("vst3-helper-file.vst3");
        std::fs::write(&path, b"not-a-bundle").unwrap();

        let error = validate_vst3_bundle(&path).unwrap_err();
        assert_eq!(error.kind, Vst3BundleErrorKind::InvalidBundle);
        assert!(error.message.contains("bundle directory"));

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn discover_first_valid_vst3_in_roots_prefers_sorted_valid_bundle() {
        let root = std::env::temp_dir().join(format!("vst3-helper-search-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::write(root.join("Ignore Me.vst3"), b"not-a-bundle").unwrap();

        let later_bundle = root.join("Zoo Instrument.vst3");
        let earlier_bundle = root.join("Alpha Instrument.vst3");
        std::fs::create_dir_all(&later_bundle).unwrap();
        std::fs::create_dir_all(&earlier_bundle).unwrap();

        let discovered = discover_first_valid_vst3_in_roots(std::slice::from_ref(&root)).unwrap();
        assert_eq!(discovered, earlier_bundle);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_ignores_non_vst3_entries() {
        let root =
            std::env::temp_dir().join(format!("vst3-helper-candidates-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        std::fs::create_dir_all(root.join("Alpha.vst3")).unwrap();
        std::fs::create_dir_all(root.join("Beta.component")).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();
        assert_eq!(candidates, vec![root.join("Alpha.vst3")]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_recurses_into_nested_folders() {
        let root = std::env::temp_dir().join(format!("vst3-helper-nested-{}", std::process::id()));
        let nested = root.join("Steinberg").join("VST3");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&nested).unwrap();

        let nested_bundle = nested.join("HALion Sonic.vst3");
        let top_level_bundle = root.join("Alpha.vst3");
        std::fs::create_dir_all(&nested_bundle).unwrap();
        std::fs::create_dir_all(&top_level_bundle).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();
        assert_eq!(candidates, vec![top_level_bundle, nested_bundle]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_does_not_descend_into_bundle_contents() {
        let root = std::env::temp_dir().join(format!(
            "vst3-helper-bundle-recursion-{}",
            std::process::id()
        ));
        let bundle = root.join("NestedHost.vst3");
        let fake_nested_bundle = bundle.join("Contents").join("MacOS").join("Ghost.vst3");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&fake_nested_bundle).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();
        assert_eq!(candidates, vec![bundle]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn list_vst3_candidates_in_root_skips_symlink_cycles() {
        let root =
            std::env::temp_dir().join(format!("vst3-helper-symlink-cycle-{}", std::process::id()));
        let loop_dir = root.join("loop");
        let bundle = root.join("Alpha.vst3");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&bundle).unwrap();
        symlink(&root, &loop_dir).unwrap();

        let candidates = list_vst3_candidates_in_root(&root).unwrap();

        assert_eq!(candidates, vec![bundle]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn expand_home_dir_replaces_tilde_prefix() {
        let previous = std::env::var_os("HOME");
        std::env::set_var("HOME", "/tmp/vst3-helper-home");

        let expanded = expand_home_dir("~/Library/Audio/Plug-Ins/VST3").unwrap();
        assert_eq!(
            expanded,
            Path::new("/tmp/vst3-helper-home/Library/Audio/Plug-Ins/VST3")
        );

        match previous {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn vst3_bundle_binary_path_prefers_named_executable() {
        let root = std::env::temp_dir().join(format!("vst3-helper-bundle-{}", std::process::id()));
        let bundle = root.join("My Instrument.vst3");
        let macos = bundle.join("Contents").join("MacOS");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&macos).unwrap();
        std::fs::write(macos.join("OtherBinary"), b"x").unwrap();
        let expected = macos.join("My Instrument");
        std::fs::write(&expected, b"x").unwrap();

        let resolved = vst3_bundle_binary_path(&bundle).unwrap();
        assert_eq!(resolved, expected);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_vst3_candidates_in_root_reports_read_errors() {
        let root =
            std::env::temp_dir().join(format!("vst3-helper-file-root-{}", std::process::id()));
        let _ = std::fs::remove_file(&root);
        std::fs::write(&root, b"not-a-directory").unwrap();

        let error = list_vst3_candidates_in_root(&root).unwrap_err();
        assert!(error.contains("Failed to read VST3 discovery root"));

        let _ = std::fs::remove_file(&root);
    }

    // ── Plugin host compatibility tests ──────────────────────────────

    #[test]
    fn rejects_known_unsupported_vendor_example() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/Groove Agent SE.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("current unsupported list"));
        assert!(reason.contains("Groove Agent SE"));
        assert!(reason.contains("controller initialization"));
    }

    #[test]
    fn allows_unknown_bundle_even_without_a_vendor_folder() {
        let path = Path::new("/tmp/Some Future Plugin.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }

    #[test]
    fn rejects_kontakt_7_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Kontakt 7.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("Kontakt 7"));
        assert!(reason.contains("component createInstance"));
    }

    #[test]
    fn rejects_mt_power_drumkit_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/MT-PowerDrumKit.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("MT-PowerDrumKit"));
        assert!(reason.contains("MVP-host teardown exception"));
    }

    #[test]
    fn allows_halion_sonic_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }

    #[test]
    fn rejects_halion_sonic_se_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic SE.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("HALion Sonic SE"));
        assert!(reason.contains("null plugin factory"));
    }

    #[test]
    fn allows_non_unsupported_control_case() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/BBC Symphony Orchestra.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }
}
