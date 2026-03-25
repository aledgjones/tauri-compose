use crate::playback::plugin_host_compatibility::bounded_editor_host_unsupported_reason;
use crate::playback::vst3::{
    default_vst3_search_roots, list_vst3_candidates_in_root, validate_vst3_bundle,
    vst3_bundle_binary_path, Vst3BundleError, Vst3BundleErrorKind,
};
use crate::{AppContextState, RendererContextState};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorHostValidationStatus {
    Blocked,
    Failed,
    Attached,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorHostValidationPhase {
    PolicyGate,
    ModuleLoad,
    FactoryCreate,
    HostContext,
    ComponentCreate,
    ComponentInitialize,
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

    let entry = entries
        .iter()
        .find(|entry| entry.id == requested_plugin_id)
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
                    candidates
                        .into_iter()
                        .map(|candidate| build_inventory_entry(candidate, source_root.clone())),
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
        .map(validate_cached_entry)
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
                result
                    .entries
                    .extend(candidates.into_iter().filter_map(|candidate| {
                        let bundle_path = candidate.display().to_string();

                        if cached_bundle_paths.contains(&bundle_path) {
                            None
                        } else {
                            Some(build_inventory_entry(candidate, source_root.clone()))
                        }
                    }));
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

fn validate_cached_entry(entry: PluginInventoryEntry) -> PluginInventoryEntry {
    let bundle_path = PathBuf::from(&entry.bundle_path);

    match validate_vst3_bundle(&bundle_path) {
        Ok(()) => match vst3_bundle_binary_path(&bundle_path) {
            Ok(binary_path) => PluginInventoryEntry {
                availability: PluginInventoryAvailability::Available,
                binary_path: Some(binary_path.display().to_string()),
                validation_failure: None,
                editor_host_validation: inventory_editor_host_validation(&bundle_path),
                ..entry
            },
            Err(error) => unavailable_cached_entry(entry, error),
        },
        Err(error) => unavailable_cached_entry(entry, error),
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

fn build_inventory_entry(candidate: PathBuf, source_root: String) -> PluginInventoryEntry {
    let bundle_path = candidate.display().to_string();
    let name = candidate
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Unknown Plugin")
        .to_string();

    let validation_failure = match validate_vst3_bundle(&candidate) {
        Ok(()) => match vst3_bundle_binary_path(&candidate) {
            Ok(binary_path) => {
                return PluginInventoryEntry {
                    id: format!("vst3:{}", bundle_path),
                    name,
                    bundle_path,
                    binary_path: Some(binary_path.display().to_string()),
                    source_root,
                    availability: PluginInventoryAvailability::Available,
                    validation_failure: None,
                    editor_host_validation: inventory_editor_host_validation(&candidate),
                };
            }
            Err(error) => PluginInventoryFailure::from(error),
        },
        Err(error) => PluginInventoryFailure::from(error),
    };

    PluginInventoryEntry {
        id: format!("vst3:{}", bundle_path),
        name,
        bundle_path,
        binary_path: None,
        source_root,
        availability: PluginInventoryAvailability::Unavailable,
        validation_failure: Some(validation_failure),
        editor_host_validation: None,
    }
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

#[cfg(test)]
mod tests {
    use super::{
        blocked_editor_host_validation, discover_inventory_in_roots, load_persisted_state,
        resolve_available_bundle_path_in_entries, PluginEditorHostValidationPhase,
        PluginEditorHostValidationStatus, PluginInventoryAvailability, PluginInventoryContext,
        PluginInventoryEntry, PluginInventoryFailure, PluginInventoryFailureKind,
    };
    use sled::Db;
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

    fn create_valid_bundle(root: &PathBuf, name: &str) -> PathBuf {
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
    fn discover_inventory_marks_halion_sonic_as_blocked_without_hiding_bundle_availability() {
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
}
