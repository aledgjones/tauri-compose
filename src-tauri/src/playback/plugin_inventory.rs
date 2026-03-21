use crate::playback::vst3::{
    default_vst3_search_roots, list_vst3_candidates_in_root, validate_vst3_bundle,
    vst3_bundle_binary_path, Vst3BundleError, Vst3BundleErrorKind,
};
use serde::{Deserialize, Serialize};
use sled::Db;
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
    IoError,
    StorageError,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginInventoryFailure {
    pub kind: PluginInventoryFailureKind,
    pub message: String,
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
    storage_error: Option<PluginInventoryFailure>,
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

        Mutex::new(Self::from_db_path(db_path.as_deref(), startup_errors))
    }

    fn from_db_path(db_path: Option<&Path>, startup_errors: Vec<PluginInventoryFailure>) -> Self {
        let mut state = PluginInventoryState::default();
        let mut storage_error = startup_errors.first().cloned();

        let db = match db_path {
            Some(path) => match sled::open(path) {
                Ok(db) => {
                    state = match load_persisted_state(&db) {
                        Ok(Some(persisted_state)) => persisted_state.into(),
                        Ok(None) => PluginInventoryState::default(),
                        Err(error) => {
                            let mut errored_state = PluginInventoryState::default();
                            errored_state.errors.push(storage_failure(format!(
                                "Failed to load persisted plugin inventory: {error}"
                            )));
                            errored_state
                        }
                    };
                    Some(db)
                }
                Err(error) => {
                    let failure = storage_failure(format!(
                        "Failed to open plugin inventory database '{}': {error}",
                        path.display()
                    ));
                    storage_error = Some(failure.clone());
                    state.errors.push(failure);
                    None
                }
            },
            None => None,
        };

        state.errors.extend(startup_errors);

        Self {
            state,
            db,
            storage_error,
        }
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

        if let Some(storage_error) = &self.storage_error {
            self.state.errors.push(storage_error.clone());
        }

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
    let mut inventory = state.lock().unwrap();
    inventory.discover()?;
    inventory.emit(&app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        discover_inventory_in_roots, PluginInventoryAvailability, PluginInventoryContext,
        PluginInventoryFailureKind,
    };
    use std::path::PathBuf;

    fn create_valid_bundle(root: &PathBuf, name: &str) -> PathBuf {
        let bundle = root.join(format!("{name}.vst3"));
        let macos_dir = bundle.join("Contents").join("MacOS");
        std::fs::create_dir_all(&macos_dir).unwrap();
        std::fs::write(macos_dir.join(name), b"binary").unwrap();
        bundle
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
}
