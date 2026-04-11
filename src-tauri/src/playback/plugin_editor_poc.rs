#![allow(non_snake_case)]

use crate::playback::engine::{set_slot_editor_render_active, LiveRenderEvent};
use crate::playback::plugin_host_compatibility::bounded_editor_host_unsupported_reason;
use crate::playback::plugin_inventory::{
    blocked_editor_host_validation, PluginCapabilityEvidence, PluginCapabilityEvidenceStatus,
    PluginEditorHostLoadInitProbe, PluginEditorHostLoadInitProbeStatus, PluginEditorHostValidation,
    PluginEditorHostValidationPhase, PluginEditorHostValidationPhaseResult,
    PluginEditorHostValidationPhaseResultStatus, PluginEditorHostValidationStatus,
};
use crate::playback::runtime_activation::ActivatedSlotTarget;
use crate::playback::vst3::{
    default_vst3_search_roots, discover_first_valid_vst3_in_roots, validate_vst3_bundle,
    MACOS_VST3_SEARCH_ROOTS,
};
use crate::score::playback::{PlaybackSlotPluginState, PlaybackSlotTarget};
use crate::score::Score;
use crate::utils::UniqueId;
use crate::{AppContextState, ManagedPluginInventoryContextState};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};

const WINDOW_LABEL: &str = "plugin-host-poc";
const PLUGIN_PATH_ENV: &str = "SOLOC_PLUGIN_EDITOR_POC_VST3_PATH";
const MAINTAINER_LOAD_INIT_PROBE_LOG_FILE: &str = "plugin-editor-poc-load-init-probe.log";

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorPocProbeProfile {
    #[default]
    Default,
    ControllerInitHostOnlyLifecycleAligned,
}

fn current_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn normalize_probe_log_message(message: &str) -> String {
    message.replace("\r\n", "\\n").replace('\n', "\\n")
}

fn probe_profile_code(profile: PluginEditorPocProbeProfile) -> &'static str {
    match profile {
        PluginEditorPocProbeProfile::Default => "default",
        PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned => {
            "controller_init_host_only_lifecycle_aligned"
        }
    }
}

fn parse_probe_profile_code(value: &str) -> Option<PluginEditorPocProbeProfile> {
    match value {
        "default" => Some(PluginEditorPocProbeProfile::Default),
        "controller_init_host_only_lifecycle_aligned" => {
            Some(PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned)
        }
        _ => None,
    }
}

fn probe_profile_defers_component_handler(profile: PluginEditorPocProbeProfile) -> bool {
    matches!(
        profile,
        PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned
    )
}

fn probe_profile_delays_component_runtime_setup(profile: PluginEditorPocProbeProfile) -> bool {
    matches!(
        profile,
        PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned
    )
}

thread_local! {
    static ACTIVE_MAINTAINER_PROBE_CALLBACK_LOG: RefCell<Option<MaintainerProbeLog>> = const { RefCell::new(None) };
    static ACTIVE_MAINTAINER_PROBE_CALLBACK_PHASE: RefCell<Option<PluginEditorHostValidationPhase>> = const { RefCell::new(None) };
}

struct ActiveMaintainerProbeCallbackScope;

impl ActiveMaintainerProbeCallbackScope {
    fn install(probe_log: MaintainerProbeLog) -> Self {
        ACTIVE_MAINTAINER_PROBE_CALLBACK_LOG.with(|active_log| {
            *active_log.borrow_mut() = Some(probe_log);
        });
        ACTIVE_MAINTAINER_PROBE_CALLBACK_PHASE.with(|active_phase| {
            *active_phase.borrow_mut() = None;
        });

        Self
    }
}

impl Drop for ActiveMaintainerProbeCallbackScope {
    fn drop(&mut self) {
        ACTIVE_MAINTAINER_PROBE_CALLBACK_PHASE.with(|active_phase| {
            *active_phase.borrow_mut() = None;
        });
        ACTIVE_MAINTAINER_PROBE_CALLBACK_LOG.with(|active_log| {
            *active_log.borrow_mut() = None;
        });
    }
}

fn set_active_probe_callback_phase(phase: Option<PluginEditorHostValidationPhase>) {
    ACTIVE_MAINTAINER_PROBE_CALLBACK_PHASE.with(|active_phase| {
        *active_phase.borrow_mut() = phase;
    });
}

fn log_active_probe_host_callback(message: &str) {
    let probe_log =
        ACTIVE_MAINTAINER_PROBE_CALLBACK_LOG.with(|active_log| active_log.borrow().clone());
    let Some(probe_log) = probe_log else {
        return;
    };

    let phase_code = ACTIVE_MAINTAINER_PROBE_CALLBACK_PHASE.with(|active_phase| {
        active_phase
            .borrow()
            .map(editor_host_phase_code)
            .unwrap_or("general")
    });

    probe_log.append_note(&format!("Host callback during {}: {}", phase_code, message));
}

fn host_callback_iid_name(iid: &vst3::Steinberg::TUID) -> Option<&'static str> {
    if *iid == vst3::Steinberg::Vst::IAttributeList_iid {
        Some("IAttributeList")
    } else if *iid == vst3::Steinberg::Vst::IAutomationState_iid {
        Some("IAutomationState")
    } else if *iid == vst3::Steinberg::Vst::IComponent_iid {
        Some("IComponent")
    } else if *iid == vst3::Steinberg::Vst::IConnectionPoint_iid {
        Some("IConnectionPoint")
    } else if *iid == vst3::Steinberg::Vst::IComponentHandler_iid {
        Some("IComponentHandler")
    } else if *iid == vst3::Steinberg::Vst::IComponentHandler2_iid {
        Some("IComponentHandler2")
    } else if *iid == vst3::Steinberg::Vst::IComponentHandler3_iid {
        Some("IComponentHandler3")
    } else if *iid == vst3::Steinberg::Vst::IComponentHandlerBusActivation_iid {
        Some("IComponentHandlerBusActivation")
    } else if *iid == vst3::Steinberg::Vst::IComponentHandlerSystemTime_iid {
        Some("IComponentHandlerSystemTime")
    } else if *iid == vst3::Steinberg::Vst::IContextMenu_iid {
        Some("IContextMenu")
    } else if *iid == vst3::Steinberg::Vst::IContextMenuTarget_iid {
        Some("IContextMenuTarget")
    } else if *iid == vst3::Steinberg::Vst::IDataExchangeHandler_iid {
        Some("IDataExchangeHandler")
    } else if *iid == vst3::Steinberg::Vst::IEditController_iid {
        Some("IEditController")
    } else if *iid == vst3::Steinberg::Vst::IEditController2_iid {
        Some("IEditController2")
    } else if *iid == vst3::Steinberg::Vst::IEditControllerHostEditing_iid {
        Some("IEditControllerHostEditing")
    } else if *iid == vst3::Steinberg::Vst::IEventList_iid {
        Some("IEventList")
    } else if *iid == vst3::Steinberg::FUnknown_iid {
        Some("FUnknown")
    } else if *iid == vst3::Steinberg::Vst::IHostApplication_iid {
        Some("IHostApplication")
    } else if *iid == vst3::Steinberg::Vst::IMessage_iid {
        Some("IMessage")
    } else if *iid == vst3::Steinberg::Vst::IPlugInterfaceSupport_iid {
        Some("IPlugInterfaceSupport")
    } else if *iid == vst3::Steinberg::IPluginBase_iid {
        Some("IPluginBase")
    } else if *iid == vst3::Steinberg::Vst::IProgramListData_iid {
        Some("IProgramListData")
    } else if *iid == vst3::Steinberg::Vst::IProgress_iid {
        Some("IProgress")
    } else if *iid == vst3::Steinberg::Linux::IRunLoop_iid {
        Some("IRunLoop")
    } else if *iid == vst3::Steinberg::Vst::IUnitData_iid {
        Some("IUnitData")
    } else if *iid == vst3::Steinberg::Vst::IUnitHandler_iid {
        Some("IUnitHandler")
    } else if *iid == vst3::Steinberg::Vst::IUnitHandler2_iid {
        Some("IUnitHandler2")
    } else if *iid == vst3::Steinberg::Vst::IUnitInfo_iid {
        Some("IUnitInfo")
    } else if *iid == vst3::Steinberg::Vst::IVst3ToAAXWrapper_iid {
        Some("IVst3ToAAXWrapper")
    } else if *iid == vst3::Steinberg::Vst::IVst3ToAUWrapper_iid {
        Some("IVst3ToAUWrapper")
    } else if *iid == vst3::Steinberg::Vst::IVst3ToVst2Wrapper_iid {
        Some("IVst3ToVst2Wrapper")
    } else if *iid == vst3::Steinberg::Vst::IVst3WrapperMPESupport_iid {
        Some("IVst3WrapperMPESupport")
    } else {
        None
    }
}

fn host_callback_iid_hex(iid: &vst3::Steinberg::TUID) -> String {
    iid.chunks_exact(4)
        .map(|chunk| {
            chunk
                .iter()
                .map(|byte| format!("{:02X}", *byte as u8))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn host_callback_iid_label(iid: &vst3::Steinberg::TUID) -> String {
    host_callback_iid_name(iid)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("unknown_interface({})", host_callback_iid_hex(iid)))
}

fn host_callback_iid_name_from_ptr(iid: *const vst3::Steinberg::TUID) -> String {
    if iid.is_null() {
        String::from("null")
    } else {
        unsafe { host_callback_iid_label(&*iid) }
    }
}

fn is_wrapper_probe_interface(iid: &vst3::Steinberg::TUID) -> bool {
    *iid == vst3::Steinberg::Vst::IVst3ToAAXWrapper_iid
        || *iid == vst3::Steinberg::Vst::IVst3ToAUWrapper_iid
        || *iid == vst3::Steinberg::Vst::IVst3ToVst2Wrapper_iid
}

fn emit_probe_console_event(
    status: &str,
    phase: Option<PluginEditorHostValidationPhase>,
    message: &str,
) {
    let mut stderr = std::io::stderr();
    let line = format!(
        "[MAINTAINER_PROBE][{}][{}][{}] {}\n",
        current_timestamp_ms(),
        phase.map(editor_host_phase_code).unwrap_or("general"),
        status,
        normalize_probe_log_message(message),
    );

    let _ = stderr.write_all(line.as_bytes());
    let _ = stderr.flush();
}

fn parse_probe_log_field<'a>(line: &'a str, field_name: &str) -> Option<&'a str> {
    let field_start = line.find(field_name)? + field_name.len();
    let remainder = &line[field_start..];
    let field_end = remainder
        .find(char::is_whitespace)
        .unwrap_or(remainder.len());
    Some(&remainder[..field_end])
}

fn derive_probe_log_profile(contents: Option<&str>) -> Option<PluginEditorPocProbeProfile> {
    let value = contents?
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("probe_profile="))?;

    parse_probe_profile_code(value)
}

fn derive_probe_log_crash_hint(contents: Option<&str>) -> Option<PluginEditorPocProbeLogCrashHint> {
    let last_line = contents?.lines().map(str::trim).rev().find(|line| {
        if line.is_empty() {
            return false;
        }

        matches!(
            parse_probe_log_field(line, "status="),
            Some("starting" | "completed" | "failed" | "skipped")
        )
    })?;

    let phase = parse_probe_log_field(last_line, "phase=")?;
    let status = parse_probe_log_field(last_line, "status=")?;

    if status != "starting" {
        return None;
    }

    Some(PluginEditorPocProbeLogCrashHint {
        phase: phase.to_string(),
        last_line: last_line.to_string(),
        message: format!(
            "Likely hard crash during {phase}. The last recorded probe event started but never completed or failed."
        ),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SlotRuntimeRenderOutcome {
    Rendered,
    SilenceOnly,
    MissingSession,
    Busy,
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    Unsupported,
    Error(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PluginHostLifecyclePhase {
    Idle,
    Reserved,
    Attached,
    Detached,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SlotRuntimePluginStateCapture {
    MissingSession,
    NoTransferableState,
    Captured(PlaybackSlotPluginState),
}

fn apply_slot_runtime_plugin_state_capture(
    target: &mut PlaybackSlotTarget,
    capture: SlotRuntimePluginStateCapture,
) {
    match capture {
        SlotRuntimePluginStateCapture::MissingSession => {}
        SlotRuntimePluginStateCapture::NoTransferableState => target.plugin_state = None,
        SlotRuntimePluginStateCapture::Captured(plugin_state) => {
            target.plugin_state = Some(plugin_state)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PluginEditorTarget {
    plugin_path: PathBuf,
    slot_id: Option<UniqueId>,
    plugin_state: Option<PlaybackSlotPluginState>,
}

impl PluginEditorTarget {
    fn inventory(plugin_path: PathBuf) -> Self {
        Self {
            plugin_path,
            slot_id: None,
            plugin_state: None,
        }
    }

    fn slot_with_state(
        plugin_path: PathBuf,
        slot_id: UniqueId,
        plugin_state: Option<PlaybackSlotPluginState>,
    ) -> Self {
        Self {
            plugin_path,
            slot_id: Some(slot_id),
            plugin_state,
        }
    }

    fn path(&self) -> &Path {
        &self.plugin_path
    }
}

impl fmt::Display for PluginEditorTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.slot_id {
            Some(slot_id) => write!(f, "slot={} path={}", slot_id, self.plugin_path.display()),
            None => write!(f, "path={}", self.plugin_path.display()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MaintainerProbeLog {
    path: PathBuf,
}

impl MaintainerProbeLog {
    fn resolve(app: &AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .map(|dir| {
                dir.join("diagnostics")
                    .join(MAINTAINER_LOAD_INIT_PROBE_LOG_FILE)
            })
            .unwrap_or_else(|_| {
                std::env::temp_dir()
                    .join(format!("compose-{}", MAINTAINER_LOAD_INIT_PROBE_LOG_FILE))
            });

        Self { path }
    }

    #[cfg(test)]
    fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    fn reset_for_probe(
        &self,
        target_description: &str,
        blocked_reason: Option<&str>,
        profile: PluginEditorPocProbeProfile,
    ) {
        emit_probe_console_event(
            "reset",
            None,
            &format!(
                "Preparing maintainer probe log at {} for target={} profile={} blocked_reason={}",
                self.path.display(),
                target_description,
                probe_profile_code(profile),
                blocked_reason.unwrap_or("none"),
            ),
        );

        if let Err(error) = self.try_reset_for_probe(target_description, blocked_reason, profile) {
            eprintln!("Failed to reset maintainer probe log: {error}");
        }
    }

    fn append_note(&self, message: &str) {
        emit_probe_console_event("info", None, message);

        if let Err(error) = self.try_append_event("info", None, message) {
            eprintln!("Failed to append maintainer probe note: {error}");
        }
    }

    fn append_phase_event(
        &self,
        phase: PluginEditorHostValidationPhase,
        status: &str,
        message: &str,
    ) {
        emit_probe_console_event(status, Some(phase), message);

        if let Err(error) = self.try_append_event(status, Some(phase), message) {
            eprintln!("Failed to append maintainer probe phase event: {error}");
        }
    }

    fn read_contents(&self) -> Result<Option<String>, String> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!(
                "Failed to read maintainer probe log '{}': {error}",
                self.path.display()
            )),
        }
    }

    fn snapshot(&self) -> Result<PluginEditorPocProbeLog, String> {
        let contents = self.read_contents()?;

        Ok(PluginEditorPocProbeLog {
            path: self.path.display().to_string(),
            profile: derive_probe_log_profile(contents.as_deref()),
            crash_hint: derive_probe_log_crash_hint(contents.as_deref()),
            contents,
        })
    }

    fn try_reset_for_probe(
        &self,
        target_description: &str,
        blocked_reason: Option<&str>,
        profile: PluginEditorPocProbeProfile,
    ) -> Result<(), String> {
        let mut file = self.open_file(true)?;

        writeln!(file, "=== maintainer load/init probe ===").map_err(|error| error.to_string())?;
        writeln!(file, "timestamp_ms={}", current_timestamp_ms())
            .map_err(|error| error.to_string())?;
        writeln!(file, "probe_profile={}", probe_profile_code(profile))
            .map_err(|error| error.to_string())?;
        writeln!(
            file,
            "target={}",
            normalize_probe_log_message(target_description)
        )
        .map_err(|error| error.to_string())?;

        if let Some(reason) = blocked_reason {
            writeln!(
                file,
                "ordinary_blocked_reason={}",
                normalize_probe_log_message(reason)
            )
            .map_err(|error| error.to_string())?;
        }

        writeln!(file).map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        Ok(())
    }

    fn try_append_event(
        &self,
        status: &str,
        phase: Option<PluginEditorHostValidationPhase>,
        message: &str,
    ) -> Result<(), String> {
        let mut file = self.open_file(false)?;
        writeln!(
            file,
            "[{}] phase={} status={} message={}",
            current_timestamp_ms(),
            phase.map(editor_host_phase_code).unwrap_or("general"),
            status,
            normalize_probe_log_message(message),
        )
        .map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        Ok(())
    }

    fn open_file(&self, truncate: bool) -> Result<std::fs::File, String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create maintainer probe log directory '{}': {error}",
                    parent.display()
                )
            })?;
        }

        let mut options = OpenOptions::new();
        options.create(true).write(true);
        if truncate {
            options.truncate(true);
        } else {
            options.append(true);
        }

        options.open(&self.path).map_err(|error| {
            format!(
                "Failed to open maintainer probe log '{}': {error}",
                self.path.display()
            )
        })
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct PluginEditorPocProbeLogCrashHint {
    pub phase: String,
    pub last_line: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct PluginEditorPocProbeLog {
    pub path: String,
    pub profile: Option<PluginEditorPocProbeProfile>,
    pub crash_hint: Option<PluginEditorPocProbeLogCrashHint>,
    pub contents: Option<String>,
}

#[derive(Debug)]
struct PluginHostLifecycleState {
    target: Option<PluginEditorTarget>,
    phase: PluginHostLifecyclePhase,
}

impl Default for PluginHostLifecycleState {
    fn default() -> Self {
        Self {
            target: None,
            phase: PluginHostLifecyclePhase::Idle,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum PluginTargetReservation {
    Acquired,
    AlreadyReserved(PluginHostLifecyclePhase),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PluginHostSyncReason {
    WindowResized,
}

fn active_plugin_host_lifecycle() -> &'static Mutex<PluginHostLifecycleState> {
    static ACTIVE_PLUGIN_HOST_LIFECYCLE: OnceLock<Mutex<PluginHostLifecycleState>> =
        OnceLock::new();
    ACTIVE_PLUGIN_HOST_LIFECYCLE.get_or_init(|| Mutex::new(PluginHostLifecycleState::default()))
}

fn reset_plugin_host_lifecycle(state: &mut PluginHostLifecycleState) {
    state.target = None;
    state.phase = PluginHostLifecyclePhase::Idle;
}

fn clear_stale_plugin_host_lifecycle(
    state: &mut PluginHostLifecycleState,
    host_window_exists: bool,
) {
    if state.target.is_some()
        && !host_window_exists
        && matches!(state.phase, PluginHostLifecyclePhase::Attached)
    {
        reset_plugin_host_lifecycle(state);
    }
}

fn detached_target_to_replace(
    requested_target: &PluginEditorTarget,
    existing_window_present: bool,
    state: &PluginHostLifecycleState,
) -> Option<PluginEditorTarget> {
    if existing_window_present || state.phase != PluginHostLifecyclePhase::Detached {
        return None;
    }

    let active_target = state.target.as_ref()?;
    if active_target == requested_target {
        return None;
    }

    Some(active_target.clone())
}

fn active_detached_target_to_replace(
    requested_target: &PluginEditorTarget,
    existing_window_present: bool,
) -> Result<Option<PluginEditorTarget>, String> {
    let lifecycle = active_plugin_host_lifecycle()
        .lock()
        .map_err(|_| String::from("Failed to read plugin editor POC target state"))?;

    Ok(detached_target_to_replace(
        requested_target,
        existing_window_present,
        &lifecycle,
    ))
}

fn reserve_plugin_target(
    target: &PluginEditorTarget,
    host_window_exists: bool,
) -> Result<PluginTargetReservation, String> {
    let mut lifecycle = active_plugin_host_lifecycle()
        .lock()
        .map_err(|_| String::from("Failed to read plugin editor POC target state"))?;

    clear_stale_plugin_host_lifecycle(&mut lifecycle, host_window_exists);

    if let Some(active_target) = lifecycle.target.as_ref() {
        if active_target != target {
            return Err(String::from(
                "The plugin editor POC window is already open for a different target. Close it before switching slot or plugin targets.",
            ));
        }

        return Ok(PluginTargetReservation::AlreadyReserved(lifecycle.phase));
    }

    lifecycle.target = Some(target.clone());
    lifecycle.phase = PluginHostLifecyclePhase::Reserved;
    Ok(PluginTargetReservation::Acquired)
}

fn active_plugin_target_identity() -> Option<PluginEditorTarget> {
    active_plugin_host_lifecycle()
        .lock()
        .ok()
        .and_then(|state| state.target.clone())
}

fn slot_runtime_session_in_flight(slot_id: UniqueId, desired_target: &ActivatedSlotTarget) -> bool {
    active_plugin_host_lifecycle()
        .lock()
        .ok()
        .and_then(|state| {
            state.target.as_ref().map(|target| {
                target.slot_id == Some(slot_id)
                    && target.path() == desired_target.bundle_path.as_path()
                    && target.plugin_state.as_ref() == desired_target.plugin_state.as_ref()
                    && state.phase != PluginHostLifecyclePhase::Idle
            })
        })
        .unwrap_or(false)
}

fn mark_plugin_target_attached(target: &PluginEditorTarget) {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        if lifecycle.target.as_ref() == Some(target) {
            lifecycle.phase = PluginHostLifecyclePhase::Attached;
            if let Some(slot_id) = target.slot_id {
                set_slot_editor_render_active(slot_id, true);
            }
        }
    }
}

fn release_reserved_plugin_target(target: &PluginEditorTarget) {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        if lifecycle.target.as_ref() == Some(target) {
            if let Some(slot_id) = target.slot_id {
                set_slot_editor_render_active(slot_id, false);
            }
            reset_plugin_host_lifecycle(&mut lifecycle);
        }
    }
}

fn clear_reserved_plugin_target() {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        if let Some(slot_id) = lifecycle.target.as_ref().and_then(|target| target.slot_id) {
            set_slot_editor_render_active(slot_id, false);
        }
        reset_plugin_host_lifecycle(&mut lifecycle);
    }
}

#[cfg(target_os = "macos")]
fn active_plugin_host_session() -> &'static Mutex<Option<PluginHostSession>> {
    static ACTIVE_PLUGIN_HOST_SESSION: OnceLock<Mutex<Option<PluginHostSession>>> = OnceLock::new();
    ACTIVE_PLUGIN_HOST_SESSION.get_or_init(|| Mutex::new(None))
}

#[derive(Serialize)]
pub struct PluginEditorPocOpenResult {
    pub window_label: String,
    pub plugin_path: String,
    pub native_host_attached: bool,
}

fn editor_host_phase_code(phase: PluginEditorHostValidationPhase) -> &'static str {
    match phase {
        PluginEditorHostValidationPhase::PolicyGate => "policy_gate",
        PluginEditorHostValidationPhase::ModuleLoad => "module_load",
        PluginEditorHostValidationPhase::FactoryCreate => "factory_create",
        PluginEditorHostValidationPhase::HostContext => "host_context",
        PluginEditorHostValidationPhase::ComponentCreate => "component_create",
        PluginEditorHostValidationPhase::ComponentInitialize => "component_initialize",
        PluginEditorHostValidationPhase::ComponentRuntimeSetup => "component_runtime_setup",
        PluginEditorHostValidationPhase::ControllerCreate => "controller_create",
        PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit => {
            "controller_set_component_handler_pre_init"
        }
        PluginEditorHostValidationPhase::ControllerInitialize => "controller_initialize",
        PluginEditorHostValidationPhase::ComponentStateSync => "component_state_sync",
        PluginEditorHostValidationPhase::ControllerSetComponentHandler => {
            "controller_set_component_handler"
        }
        PluginEditorHostValidationPhase::ConnectionPointHandshake => "connection_point_handshake",
        PluginEditorHostValidationPhase::ViewCreate => "view_create",
        PluginEditorHostValidationPhase::ViewSetFrame => "view_set_frame",
        PluginEditorHostValidationPhase::ViewAttach => "view_attach",
        PluginEditorHostValidationPhase::Attached => "attached",
    }
}

fn editor_host_validation_status_code(status: PluginEditorHostValidationStatus) -> &'static str {
    match status {
        PluginEditorHostValidationStatus::Blocked => "blocked",
        PluginEditorHostValidationStatus::Failed => "failed",
        PluginEditorHostValidationStatus::Attached => "attached",
    }
}

fn load_init_probe_status_code(status: PluginEditorHostLoadInitProbeStatus) -> &'static str {
    match status {
        PluginEditorHostLoadInitProbeStatus::BlockedByHostContract => "blocked_by_host_contract",
        PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure => "blocked_by_plugin_failure",
        PluginEditorHostLoadInitProbeStatus::LoadInitSafe => "load_init_safe",
    }
}

fn load_init_probe_phase_order() -> &'static [PluginEditorHostValidationPhase] {
    &[
        PluginEditorHostValidationPhase::ModuleLoad,
        PluginEditorHostValidationPhase::FactoryCreate,
        PluginEditorHostValidationPhase::HostContext,
        PluginEditorHostValidationPhase::ComponentCreate,
        PluginEditorHostValidationPhase::ComponentInitialize,
        PluginEditorHostValidationPhase::ComponentStateSync,
        PluginEditorHostValidationPhase::ComponentRuntimeSetup,
        PluginEditorHostValidationPhase::ControllerCreate,
        PluginEditorHostValidationPhase::ControllerInitialize,
        PluginEditorHostValidationPhase::ConnectionPointHandshake,
        PluginEditorHostValidationPhase::ViewCreate,
        PluginEditorHostValidationPhase::ViewAttach,
        PluginEditorHostValidationPhase::Attached,
    ]
}

fn build_load_init_probe_summary(
    profile: PluginEditorPocProbeProfile,
    status: PluginEditorHostLoadInitProbeStatus,
    first_failing_phase: Option<PluginEditorHostValidationPhase>,
    first_failing_message: Option<&str>,
) -> String {
    match status {
        PluginEditorHostLoadInitProbeStatus::BlockedByHostContract => format!(
            "Maintainer load/init probe ({}) first failed at {} and appears host-contract-related: {}",
            probe_profile_code(profile),
            first_failing_phase
                .map(editor_host_phase_code)
                .unwrap_or("unknown_phase"),
            first_failing_message
                .unwrap_or("No maintainer-facing error message was recorded."),
        ),
        PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure => format!(
            "Maintainer load/init probe ({}) first failed at {} and appears plugin-side: {}",
            probe_profile_code(profile),
            first_failing_phase
                .map(editor_host_phase_code)
                .unwrap_or("unknown_phase"),
            first_failing_message
                .unwrap_or("No maintainer-facing error message was recorded."),
        ),
        PluginEditorHostLoadInitProbeStatus::LoadInitSafe => format!(
            "Maintainer load/init probe ({}) reached a bounded non-crashing attach. Ordinary product support remains unchanged until later stories prove more.",
            probe_profile_code(profile),
        ),
    }
}

#[derive(Clone, Debug, Default)]
struct PluginHostBuildTelemetry {
    enabled: bool,
    probe_profile: PluginEditorPocProbeProfile,
    probe_log: Option<MaintainerProbeLog>,
    last_successful_phase: Option<PluginEditorHostValidationPhase>,
    first_failing_phase: Option<PluginEditorHostValidationPhase>,
    first_failing_message: Option<String>,
    failure_status: Option<PluginEditorHostLoadInitProbeStatus>,
    phase_results: Vec<PluginEditorHostValidationPhaseResult>,
}

impl PluginHostBuildTelemetry {
    fn disabled() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn enabled() -> Self {
        Self {
            enabled: true,
            probe_profile: PluginEditorPocProbeProfile::Default,
            ..Self::default()
        }
    }

    fn enabled_with_probe_log(
        probe_profile: PluginEditorPocProbeProfile,
        probe_log: MaintainerProbeLog,
    ) -> Self {
        Self {
            enabled: true,
            probe_profile,
            probe_log: Some(probe_log),
            ..Self::default()
        }
    }

    fn log_phase_event(&self, phase: PluginEditorHostValidationPhase, status: &str, message: &str) {
        if !self.enabled {
            return;
        }

        if let Some(probe_log) = &self.probe_log {
            probe_log.append_phase_event(phase, status, message);
        }
    }

    fn log_note(&self, message: &str) {
        if !self.enabled {
            return;
        }

        if let Some(probe_log) = &self.probe_log {
            probe_log.append_note(message);
        }
    }

    fn record_starting(&self, phase: PluginEditorHostValidationPhase, message: impl Into<String>) {
        if !self.enabled {
            return;
        }

        let message = message.into();
        set_active_probe_callback_phase(Some(phase));
        self.log_phase_event(phase, "starting", &message);
    }

    fn record_completed(
        &mut self,
        phase: PluginEditorHostValidationPhase,
        message: impl Into<String>,
    ) {
        if !self.enabled {
            return;
        }

        let message = message.into();
        self.last_successful_phase = Some(phase);
        self.log_phase_event(phase, "completed", &message);
        set_active_probe_callback_phase(None);
        self.phase_results
            .push(PluginEditorHostValidationPhaseResult {
                phase,
                status: PluginEditorHostValidationPhaseResultStatus::Completed,
                message,
            });
    }

    fn record_skipped(
        &mut self,
        phase: PluginEditorHostValidationPhase,
        message: impl Into<String>,
    ) {
        if !self.enabled {
            return;
        }

        let message = message.into();
        self.log_phase_event(phase, "skipped", &message);
        set_active_probe_callback_phase(None);
        self.phase_results
            .push(PluginEditorHostValidationPhaseResult {
                phase,
                status: PluginEditorHostValidationPhaseResultStatus::Skipped,
                message,
            });
    }

    fn record_failed(
        &mut self,
        phase: PluginEditorHostValidationPhase,
        probe_status: PluginEditorHostLoadInitProbeStatus,
        message: impl Into<String>,
    ) {
        if !self.enabled {
            return;
        }

        let message = message.into();
        if self.first_failing_phase.is_none() {
            self.first_failing_phase = Some(phase);
            self.first_failing_message = Some(message.clone());
            self.failure_status = Some(probe_status);
        }

        self.log_phase_event(phase, "failed", &message);
        set_active_probe_callback_phase(None);

        self.phase_results
            .push(PluginEditorHostValidationPhaseResult {
                phase,
                status: PluginEditorHostValidationPhaseResultStatus::Failed,
                message,
            });
    }

    fn finish(mut self) -> Option<PluginEditorHostLoadInitProbe> {
        if !self.enabled {
            return None;
        }

        if let Some(first_failing_phase) = self.first_failing_phase {
            for phase in load_init_probe_phase_order() {
                if self
                    .phase_results
                    .iter()
                    .any(|result| result.phase == *phase)
                {
                    continue;
                }

                self.phase_results
                    .push(PluginEditorHostValidationPhaseResult {
                        phase: *phase,
                        status: PluginEditorHostValidationPhaseResultStatus::Skipped,
                        message: format!(
                        "Skipped because the probe stopped after the first failing phase at {}.",
                        editor_host_phase_code(first_failing_phase)
                    ),
                    });
            }
        }

        let status = self
            .failure_status
            .unwrap_or(PluginEditorHostLoadInitProbeStatus::LoadInitSafe);
        let summary = build_load_init_probe_summary(
            self.probe_profile,
            status,
            self.first_failing_phase,
            self.first_failing_message.as_deref(),
        );

        self.log_note(&format!(
            "Finished maintainer load/init probe with profile={} status={} summary={}",
            probe_profile_code(self.probe_profile),
            load_init_probe_status_code(status),
            summary,
        ));

        let probe = PluginEditorHostLoadInitProbe {
            status,
            last_successful_phase: self.last_successful_phase,
            first_failing_phase: self.first_failing_phase,
            first_failing_message: self.first_failing_message.clone(),
            summary,
            phase_results: self.phase_results,
        };

        Some(probe)
    }
}

#[derive(Clone, Debug)]
struct PluginEditorHostAttemptError {
    phase: PluginEditorHostValidationPhase,
    message: String,
    probe_status: PluginEditorHostLoadInitProbeStatus,
}

impl PluginEditorHostAttemptError {
    fn new(
        phase: PluginEditorHostValidationPhase,
        probe_status: PluginEditorHostLoadInitProbeStatus,
        message: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            message: message.into(),
            probe_status,
        }
    }

    fn host_contract_failure(
        phase: PluginEditorHostValidationPhase,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            phase,
            PluginEditorHostLoadInitProbeStatus::BlockedByHostContract,
            message,
        )
    }

    fn plugin_failure(phase: PluginEditorHostValidationPhase, message: impl Into<String>) -> Self {
        Self::new(
            phase,
            PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure,
            message,
        )
    }
}

fn record_probe_failure(
    telemetry: &mut PluginHostBuildTelemetry,
    error: PluginEditorHostAttemptError,
) -> PluginEditorHostAttemptError {
    telemetry.record_failed(error.phase, error.probe_status, error.message.clone());
    error
}

#[derive(Clone, Debug)]
struct PluginHostAttachOutcome {
    attached: bool,
    /// True when the window was shown natively on the main thread during
    /// the same pass as the attach.  When set, the caller must skip the
    /// async Tauri `window.show()` / `set_focus()` dispatch to avoid a
    /// second show that can deadlock the macOS event loop.
    native_shown: bool,
    validation: PluginEditorHostValidation,
}

fn failed_editor_host_validation(
    phase: PluginEditorHostValidationPhase,
    message: impl Into<String>,
) -> PluginEditorHostValidation {
    PluginEditorHostValidation {
        status: PluginEditorHostValidationStatus::Failed,
        phase,
        message: message.into(),
        capability_evidence: None,
        maintainer_load_init_probe: None,
    }
}

fn attached_editor_host_validation() -> PluginEditorHostValidation {
    PluginEditorHostValidation {
        status: PluginEditorHostValidationStatus::Attached,
        phase: PluginEditorHostValidationPhase::Attached,
        message: String::from(
            "The bounded native editor host attached successfully. Current readiness evidence is limited to editor-host usability.",
        ),
        capability_evidence: Some(PluginCapabilityEvidence {
            status: PluginCapabilityEvidenceStatus::NotProven,
            message: String::from(
                "The current bounded seam does not yet inspect HALion Sonic for multi-part or channel capability facts.",
            ),
        }),
        maintainer_load_init_probe: None,
    }
}

fn with_maintainer_load_init_probe(
    mut validation: PluginEditorHostValidation,
    probe: PluginEditorHostLoadInitProbe,
) -> PluginEditorHostValidation {
    validation.maintainer_load_init_probe = Some(probe);
    validation
}

fn blocked_editor_host_validation_message_with_probe(
    blocked_reason: &str,
    probe: &PluginEditorHostLoadInitProbe,
) -> String {
    format!(
        "{blocked_reason} Maintainer load/init probe summary: {}",
        probe.summary
    )
}

fn validation_from_load_init_probe(
    blocked_reason: Option<String>,
    probe: PluginEditorHostLoadInitProbe,
) -> PluginEditorHostValidation {
    match blocked_reason {
        Some(reason) => with_maintainer_load_init_probe(
            blocked_editor_host_validation(blocked_editor_host_validation_message_with_probe(
                &reason, &probe,
            )),
            probe,
        ),
        None => match probe.status {
            PluginEditorHostLoadInitProbeStatus::LoadInitSafe => {
                with_maintainer_load_init_probe(attached_editor_host_validation(), probe)
            }
            PluginEditorHostLoadInitProbeStatus::BlockedByHostContract
            | PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure => {
                with_maintainer_load_init_probe(
                    failed_editor_host_validation(
                        probe
                            .first_failing_phase
                            .unwrap_or(PluginEditorHostValidationPhase::ModuleLoad),
                        probe
                            .first_failing_message
                            .clone()
                            .unwrap_or_else(|| probe.summary.clone()),
                    ),
                    probe,
                )
            }
        },
    }
}

fn emit_editor_host_validation(
    app: &AppHandle,
    plugin_inventory_state: &ManagedPluginInventoryContextState,
    plugin_path: &Path,
    validation: PluginEditorHostValidation,
) {
    let Ok(mut inventory) = plugin_inventory_state.lock() else {
        return;
    };

    match inventory.update_editor_host_validation_for_bundle_path(plugin_path, validation) {
        Ok(true) => inventory.emit(app),
        Ok(false) => {}
        Err(_error) => {}
    }
}

fn queue_editor_host_validation_emit(
    app: AppHandle,
    plugin_path: PathBuf,
    validation: PluginEditorHostValidation,
) {
    tauri::async_runtime::spawn(async move {
        let plugin_inventory_state = app.state::<ManagedPluginInventoryContextState>();
        emit_editor_host_validation(
            &app,
            plugin_inventory_state.inner(),
            &plugin_path,
            validation,
        );
    });
}

#[cfg(target_os = "macos")]
fn live_plugin_host_attached() -> bool {
    macos_host::live_plugin_host_attached()
}

#[cfg(not(target_os = "macos"))]
fn live_plugin_host_attached() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn release_detached_plugin_host_for_target_switch(
    app: &AppHandle,
    requested_target: &PluginEditorTarget,
    existing_window_present: bool,
) -> Result<(), String> {
    let Some(detached_target) =
        active_detached_target_to_replace(requested_target, existing_window_present)?
    else {
        return Ok(());
    };

    if detached_target.slot_id.is_some() {
        release_reserved_plugin_target(&detached_target);
        return Ok(());
    }

    macos_host::shutdown_detached_plugin_host_session(app, &detached_target)
}

#[cfg(not(target_os = "macos"))]
fn release_detached_plugin_host_for_target_switch(
    _app: &AppHandle,
    _requested_target: &PluginEditorTarget,
    _existing_window_present: bool,
) -> Result<(), String> {
    Ok(())
}

fn should_continue_reopening_reserved_target(
    phase: PluginHostLifecyclePhase,
    existing_window_present: bool,
) -> bool {
    !existing_window_present && phase == PluginHostLifecyclePhase::Detached
}

fn preserved_inventory_target_to_replace(
    requested_target: &PluginEditorTarget,
    preserved_target: Option<&PluginEditorTarget>,
) -> Option<PluginEditorTarget> {
    let preserved_target = preserved_target?;
    if preserved_target == requested_target {
        return None;
    }

    if preserved_target.slot_id.is_some() {
        return None;
    }

    Some(preserved_target.clone())
}

#[cfg(target_os = "macos")]
fn release_preserved_inventory_session_for_target_switch(
    app: &AppHandle,
    requested_target: &PluginEditorTarget,
) -> Result<(), String> {
    let preserved_target = active_plugin_host_session()
        .lock()
        .map_err(|_| String::from("Failed to inspect preserved inventory plugin host session"))?
        .as_ref()
        .and_then(|session| {
            preserved_inventory_target_to_replace(requested_target, Some(&session.target))
        });

    let Some(preserved_target) = preserved_target else {
        return Ok(());
    };

    macos_host::shutdown_detached_plugin_host_session(app, &preserved_target)
}

#[cfg(not(target_os = "macos"))]
fn release_preserved_inventory_session_for_target_switch(
    _app: &AppHandle,
    _requested_target: &PluginEditorTarget,
) -> Result<(), String> {
    Ok(())
}

fn resolve_requested_plugin_target(
    slot_id: Option<UniqueId>,
    plugin_id: Option<&str>,
    explicit_plugin_path: Option<&str>,
    app_state: &AppContextState,
    plugin_inventory_state: &ManagedPluginInventoryContextState,
) -> Result<Option<PluginEditorTarget>, String> {
    if let Some(slot_id) = slot_id {
        let requested_plugin_id = plugin_id.map(str::trim).filter(|value| !value.is_empty());
        let slot_plugin_id = {
            let app_state = app_state.lock().map_err(|_| {
                String::from("Failed to read app state while resolving slot target")
            })?;
            let slot = app_state
                .score
                .playback
                .rack
                .by_id
                .get(&slot_id)
                .ok_or_else(|| format!("Playback slot {} no longer exists.", slot_id))?;
            slot.target
                .as_ref()
                .map(|target| target.plugin_id.clone())
                .ok_or_else(|| format!("Playback slot {} is not assigned to a plugin.", slot_id))?
        };

        if let Some(requested_plugin_id) = requested_plugin_id {
            if requested_plugin_id != slot_plugin_id {
                return Err(format!(
                    "Playback slot {} is assigned to '{}', not '{}'.",
                    slot_id, slot_plugin_id, requested_plugin_id
                ));
            }
        }

        let plugin_inventory = plugin_inventory_state
            .lock()
            .map_err(|_| String::from("Failed to read plugin inventory state"))?;
        let plugin_path = plugin_inventory.resolve_available_bundle_path(&slot_plugin_id)?;
        validate_vst3_bundle(&plugin_path).map_err(|error| error.message)?;
        let plugin_state = {
            let app_state = app_state.lock().map_err(|_| {
                String::from("Failed to read app state while resolving slot plugin state")
            })?;
            app_state
                .score
                .playback
                .rack
                .by_id
                .get(&slot_id)
                .and_then(|slot| slot.target.as_ref())
                .and_then(|target| target.plugin_state.clone())
        };
        return Ok(Some(PluginEditorTarget::slot_with_state(
            plugin_path,
            slot_id,
            plugin_state,
        )));
    }

    if let Some(plugin_id) = plugin_id.map(str::trim).filter(|value| !value.is_empty()) {
        let plugin_inventory = plugin_inventory_state
            .lock()
            .map_err(|_| String::from("Failed to read plugin inventory state"))?;
        let plugin_path = plugin_inventory.resolve_available_bundle_path(plugin_id)?;
        validate_vst3_bundle(&plugin_path).map_err(|error| error.message)?;
        return Ok(Some(PluginEditorTarget::inventory(plugin_path)));
    }

    if let Some(path) = requested_plugin_target_candidate(explicit_plugin_path) {
        validate_vst3_bundle(&path).map_err(|error| error.message)?;
        return Ok(Some(PluginEditorTarget::inventory(path)));
    }

    Ok(None)
}

fn resolve_plugin_target_in_roots(
    explicit_plugin_path: Option<&str>,
    default_roots: &[PathBuf],
    inventory_target: Option<PathBuf>,
) -> Result<PathBuf, String> {
    if let Some(path) = inventory_target {
        return Ok(path);
    }

    if let Some(path) = requested_plugin_target_candidate(explicit_plugin_path) {
        validate_vst3_bundle(&path).map_err(|error| error.message)?;
        return Ok(path);
    }

    if let Some(path) = discover_first_valid_vst3_in_roots(default_roots) {
        validate_vst3_bundle(&path).map_err(|error| error.message)?;
        return Ok(path);
    }

    Err(format!(
        "No VST3 bundle path was provided. Pass plugin_path, set {}, or install a .vst3 bundle under {}.",
        PLUGIN_PATH_ENV,
        MACOS_VST3_SEARCH_ROOTS.join(" or ")
    ))
}

fn resolve_plugin_target(
    slot_id: Option<UniqueId>,
    plugin_id: Option<&str>,
    explicit_plugin_path: Option<&str>,
    app_state: &AppContextState,
    plugin_inventory_state: &ManagedPluginInventoryContextState,
) -> Result<PluginEditorTarget, String> {
    if let Some(target) = resolve_requested_plugin_target(
        slot_id,
        plugin_id,
        explicit_plugin_path,
        app_state,
        plugin_inventory_state,
    )? {
        return Ok(target);
    }

    let plugin_path = resolve_plugin_target_in_roots(None, &default_vst3_search_roots(), None)?;
    Ok(PluginEditorTarget::inventory(plugin_path))
}

fn requested_plugin_target_candidate(explicit_plugin_path: Option<&str>) -> Option<PathBuf> {
    explicit_plugin_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var(PLUGIN_PATH_ENV)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
}

fn ensure_editor_host_target_supported(plugin_path: &Path) -> Result<(), String> {
    match bounded_editor_host_unsupported_reason(plugin_path) {
        Some(reason) => Err(reason),
        None => Ok(()),
    }
}

/// Global native host window, managed outside of Tauri's webview layer.
/// Plugin editor hosting requires a plain NSWindow with no WKWebView so that
/// the plugin's NSView is the sole child of contentView and event dispatch is
/// not intercepted by a webview layer.
///
/// Stores a raw pointer because NSWindow is MainThreadOnly and cannot be put
/// into Send/Sync containers directly. All dereferences must happen on the
/// main thread.
#[cfg(target_os = "macos")]
struct NativeWindowPtr(*mut objc2_app_kit::NSWindow);
// SAFETY: The raw pointer is only dereferenced on the main thread.
#[cfg(target_os = "macos")]
unsafe impl Send for NativeWindowPtr {}
// SAFETY: Access is guarded by the Mutex.
#[cfg(target_os = "macos")]
unsafe impl Sync for NativeWindowPtr {}

#[cfg(target_os = "macos")]
fn native_host_window() -> &'static Mutex<Option<NativeWindowPtr>> {
    static NATIVE_HOST_WINDOW: OnceLock<Mutex<Option<NativeWindowPtr>>> = OnceLock::new();
    NATIVE_HOST_WINDOW.get_or_init(|| Mutex::new(None))
}

#[cfg(target_os = "macos")]
fn native_host_window_exists() -> bool {
    native_host_window()
        .lock()
        .ok()
        .is_some_and(|guard| guard.is_some())
}

#[cfg(not(target_os = "macos"))]
fn native_host_window_exists() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn focus_native_host_window() -> Result<(), String> {
    let guard = native_host_window()
        .lock()
        .map_err(|_| String::from("Failed to lock native host window"))?;
    if let Some(ref wrapper) = *guard {
        unsafe {
            let ns_window: &objc2_app_kit::NSWindow = &*wrapper.0;
            let _: () = objc2::msg_send![ns_window, makeKeyAndOrderFront: std::ptr::null::<std::ffi::c_void>()];
        }
        Ok(())
    } else {
        Err(String::from("No native host window to focus"))
    }
}

#[cfg(target_os = "macos")]
fn close_native_host_window() {
    // Clear the delegate before ordering out to prevent callbacks during
    // programmatic close.
    macos_host::clear_window_delegate();

    let wrapper = native_host_window()
        .lock()
        .ok()
        .and_then(|mut guard| guard.take());
    if let Some(wrapper) = wrapper {
        unsafe {
            // Reclaim the Retained to release the window's retain count when
            // this scope ends. Then order the window out of the screen.
            let retained: Option<objc2::rc::Retained<objc2_app_kit::NSWindow>> =
                objc2::rc::Retained::from_raw(wrapper.0);
            if let Some(ref window) = retained {
                let ns_ref: &objc2_app_kit::NSWindow = window;
                let _: () =
                    objc2::msg_send![ns_ref, orderOut: std::ptr::null::<std::ffi::c_void>()];
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos_host {
    use super::{
        active_plugin_host_session, attached_editor_host_validation, failed_editor_host_validation,
        PluginEditorHostAttemptError, PluginEditorHostValidationPhase, PluginHostAttachOutcome,
        SlotRuntimePluginStateCapture,
    };
    use crate::playback::engine::{LiveRenderEvent, LiveRenderEventKind};
    use crate::playback::runtime_activation::ActivatedSlotTarget;
    use crate::playback::vst3::vst3_bundle_binary_path;
    use crate::score::playback::PlaybackSlotPluginState;
    use crate::utils::UniqueId;
    use base64::prelude::{Engine as _, BASE64_STANDARD};
    use core_foundation::base::{CFRelease, CFRetain, CFTypeRef, TCFType};
    use core_foundation::bundle::CFBundle;
    use core_foundation::string::CFString;
    use core_foundation::url::CFURL;
    use core_foundation_sys::bundle::{CFBundleLoadExecutable, CFBundleUnloadExecutable};

    use objc2::rc::Retained;
    use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
    use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSAutoresizingMaskOptions, NSBackingStoreType, NSTextField, NSView, NSWindow,
        NSWindowDelegate, NSWindowStyleMask,
    };
    use objc2_foundation::{NSNotification, NSPoint, NSRect, NSSize, NSString};
    use rustc_hash::FxHashMap;
    use std::ffi::{c_void, CStr, CString};
    use std::mem::zeroed;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::{Arc, Mutex as StdMutex};
    use std::time::Instant;
    use tauri::AppHandle;
    use vst3::Steinberg::IBStream_::IStreamSeekMode_::{kIBSeekCur, kIBSeekEnd, kIBSeekSet};
    use vst3::Steinberg::Linux::{
        FileDescriptor, IEventHandler, IRunLoop, IRunLoopTrait, IRunLoop_iid, ITimerHandler,
        TimerInterval,
    };
    use vst3::Steinberg::Vst::{
        self, AudioBusBuffers, AudioBusBuffers__type0, BusInfo, IAttributeList,
        IAttributeListTrait, IAudioProcessorTrait, IAutomationState, IAutomationStateTrait,
        IComponent, IComponentHandler, IComponentHandler2, IComponentHandler2Trait,
        IComponentHandler3, IComponentHandler3Trait, IComponentHandlerBusActivation,
        IComponentHandlerBusActivationTrait, IComponentHandlerSystemTime,
        IComponentHandlerSystemTimeTrait, IComponentHandlerTrait, IComponentTrait, IComponent_iid,
        IConnectionPoint, IConnectionPointTrait, IContextMenu, IContextMenuTarget,
        IContextMenuTargetTrait, IDataExchangeHandler, IDataExchangeHandlerTrait, IEditController,
        IEditControllerHostEditing, IEditControllerHostEditingTrait, IEditControllerTrait,
        IEditController_iid, IEventList, IEventListTrait, IHostApplication, IHostApplicationTrait,
        IMessage, IMessageTrait, IPlugInterfaceSupport, IPlugInterfaceSupportTrait, IProgress,
        IProgressTrait, IUnitHandler, IUnitHandler2, IUnitHandler2Trait, IUnitHandlerTrait,
        String128, TChar, ViewType,
    };
    use vst3::Steinberg::Vst::{
        Event,
        Event_::EventFlags_ as VstEventFlags,
        Event_::EventTypes_ as VstEventTypes,
        IAttributeList_::AttrID,
        IAttributeList_iid, IAutomationState_iid, IComponentHandler2_iid, IComponentHandler3_iid,
        IComponentHandlerBusActivation_iid, IComponentHandlerSystemTime_iid, IComponentHandler_iid,
        IContextMenuTarget_iid, IDataExchangeHandler_iid, IEditControllerHostEditing_iid,
        IHostApplicationVtbl, IHostApplication_iid, IMessage_iid, IPlugInterfaceSupport_iid,
        IProgress_::{ProgressType, ID as ProgressID},
        IProgress_iid, IUnitHandler2_iid, IUnitHandler_iid, NoteOffEvent, NoteOnEvent, ProcessData,
        ProcessSetup,
    };
    use vst3::Steinberg::{
        self, int64, kInvalidArgument, kNoInterface, kNotImplemented, kPlatformTypeNSView,
        kResultFalse, kResultOk, kResultTrue, tresult, uint32, FIDString, FUnknown, FUnknownVtbl,
        FUnknown_iid, IBStream, IBStreamTrait, IPlugFrame, IPlugFrameTrait, IPlugView,
        IPlugViewTrait, IPluginBaseTrait, IPluginFactory, IPluginFactory2, IPluginFactory2Trait,
        IPluginFactory3, IPluginFactory3Trait, IPluginFactoryTrait, TBool, ViewRect, TUID,
    };
    use vst3::{Class, ComPtr, ComWrapper};

    type GetPluginFactoryFn = unsafe extern "system" fn() -> *mut IPluginFactory;
    type BundleEntryFn = unsafe extern "system" fn(*mut c_void) -> bool;
    type BundleExitFn = unsafe extern "system" fn() -> bool;

    // ── NSWindowDelegate for the native host window ──────────────────────
    //
    // Handles close-button cleanup and resize sync for the plugin editor
    // window. The delegate is stored in a global to keep it alive (NSWindow
    // holds its delegate as a weak reference).

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "PluginHostWindowDelegate"]
        struct PluginHostWindowDelegate;

        unsafe impl NSObjectProtocol for PluginHostWindowDelegate {}

        unsafe impl NSWindowDelegate for PluginHostWindowDelegate {
            #[unsafe(method(windowWillClose:))]
            fn window_will_close(&self, _notification: &NSNotification) {
                // Clean up the plugin host session (detach view, etc).
                cleanup_active_plugin_host_session();
                // Release the native window pointer from the global.
                // Do NOT call orderOut — AppKit is already closing the window.
                let wrapper = super::native_host_window()
                    .lock()
                    .ok()
                    .and_then(|mut guard| guard.take());
                if let Some(wrapper) = wrapper {
                    unsafe {
                        // Reclaim Retained to balance the retain count from
                        // into_raw. The ObjC runtime keeps the window alive
                        // for the duration of the close sequence.
                        let _retained: Option<Retained<NSWindow>> = Retained::from_raw(wrapper.0);
                    }
                }
                // Clear the delegate global.
                clear_window_delegate();
            }

            #[unsafe(method(windowDidResize:))]
            fn window_did_resize(&self, _notification: &NSNotification) {
                sync_active_plugin_host(None, super::PluginHostSyncReason::WindowResized);
            }
        }
    );

    impl PluginHostWindowDelegate {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = mtm.alloc().set_ivars(());
            unsafe { msg_send![super(this), init] }
        }
    }

    struct NativeDelegatePtr(*mut PluginHostWindowDelegate);
    // SAFETY: The raw pointer is only dereferenced on the main thread.
    unsafe impl Send for NativeDelegatePtr {}
    // SAFETY: Access is guarded by the Mutex.
    unsafe impl Sync for NativeDelegatePtr {}

    fn window_delegate_global() -> &'static StdMutex<Option<NativeDelegatePtr>> {
        static DELEGATE: std::sync::OnceLock<StdMutex<Option<NativeDelegatePtr>>> =
            std::sync::OnceLock::new();
        DELEGATE.get_or_init(|| StdMutex::new(None))
    }

    pub fn clear_window_delegate() {
        let wrapper = window_delegate_global()
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());
        if let Some(wrapper) = wrapper {
            unsafe {
                // Reclaim Retained to release the delegate.
                let _retained: Option<Retained<PluginHostWindowDelegate>> =
                    Retained::from_raw(wrapper.0);
            }
        }
    }

    /// Recursion guard for IPlugFrame::resizeView, matching the Steinberg
    /// editorhost `resizeViewRecursionGard` pattern.
    pub(crate) fn resizing_from_plugin() -> &'static AtomicBool {
        static RESIZING_FROM_PLUGIN: AtomicBool = AtomicBool::new(false);
        &RESIZING_FROM_PLUGIN
    }

    pub(crate) fn clear_pending_plugin_resize_request() {
        resizing_from_plugin().store(false, Ordering::Release);
    }

    fn slot_runtime_sessions() -> &'static StdMutex<FxHashMap<UniqueId, PluginHostSession>> {
        static SLOT_RUNTIME_SESSIONS: std::sync::OnceLock<
            StdMutex<FxHashMap<UniqueId, PluginHostSession>>,
        > = std::sync::OnceLock::new();
        SLOT_RUNTIME_SESSIONS.get_or_init(|| StdMutex::new(FxHashMap::default()))
    }

    fn active_slot_id() -> Option<UniqueId> {
        super::active_plugin_target_identity().and_then(|target| target.slot_id)
    }

    fn active_session_view_state() -> Option<(ComPtr<IPlugView>, usize, bool)> {
        match super::active_plugin_target_identity() {
            Some(target) => match target.slot_id {
                Some(slot_id) => slot_runtime_sessions().lock().ok().and_then(|sessions| {
                    sessions
                        .get(&slot_id)
                        .and_then(PluginHostSession::view_state)
                }),
                None => active_plugin_host_session()
                    .lock()
                    .ok()
                    .and_then(|active_session| {
                        active_session
                            .as_ref()
                            .and_then(PluginHostSession::view_state)
                    }),
            },
            None => None,
        }
    }

    fn log_main_thread_wait_start(_context: &str) -> Instant {
        Instant::now()
    }

    fn log_main_thread_wait_finish(_context: &str, _started_at: Instant) {}

    pub(crate) fn live_plugin_host_attached() -> bool {
        active_session_view_state().is_some()
    }

    fn finish_plugin_host_session_shutdown(session: PluginHostSession) {
        let target = session.target.clone();
        clear_pending_plugin_resize_request();
        session.shutdown();
        super::release_reserved_plugin_target(&target);
    }

    struct CreatedController {
        controller: ComPtr<IEditController>,
        shared_with_component: bool,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) struct ComponentClassCandidate {
        pub(crate) cid: TUID,
        pub(crate) name: String,
        pub(crate) category: String,
        pub(crate) subcategories: String,
    }

    #[derive(Default)]
    struct HostMemoryStream {
        bytes: Arc<StdMutex<Vec<u8>>>,
        cursor: Arc<StdMutex<usize>>,
    }

    impl Clone for HostMemoryStream {
        fn clone(&self) -> Self {
            Self {
                bytes: Arc::clone(&self.bytes),
                cursor: Arc::clone(&self.cursor),
            }
        }
    }

    impl HostMemoryStream {
        fn snapshot_bytes(&self) -> Result<Vec<u8>, String> {
            self.bytes
                .lock()
                .map(|bytes| bytes.clone())
                .map_err(|_| String::from("Failed to read the VST3 memory stream shim."))
        }

        fn replace_bytes(&self, bytes: Vec<u8>) -> Result<(), String> {
            self.bytes
                .lock()
                .map_err(|_| String::from("Failed to write the VST3 memory stream shim."))
                .map(|mut current| *current = bytes)?;
            self.cursor
                .lock()
                .map_err(|_| String::from("Failed to reset the VST3 memory stream shim."))
                .map(|mut cursor| *cursor = 0)
        }
    }

    impl Class for HostMemoryStream {
        type Interfaces = (IBStream,);
    }

    impl IBStreamTrait for HostMemoryStream {
        unsafe fn read(
            &self,
            buffer: *mut c_void,
            num_bytes: i32,
            num_bytes_read: *mut i32,
        ) -> tresult {
            if !num_bytes_read.is_null() {
                *num_bytes_read = 0;
            }
            if num_bytes < 0 {
                return kInvalidArgument;
            }
            if num_bytes == 0 {
                return kResultOk;
            }
            if buffer.is_null() {
                return kInvalidArgument;
            }

            let Ok(bytes) = self.bytes.lock() else {
                return kInvalidArgument;
            };
            let Ok(mut cursor) = self.cursor.lock() else {
                return kInvalidArgument;
            };

            let available = bytes.len().saturating_sub(*cursor);
            let to_copy = available.min(num_bytes as usize);
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr().add(*cursor),
                buffer.cast::<u8>(),
                to_copy,
            );
            *cursor += to_copy;
            if !num_bytes_read.is_null() {
                *num_bytes_read = to_copy as i32;
            }

            kResultOk
        }

        unsafe fn write(
            &self,
            buffer: *mut c_void,
            num_bytes: i32,
            num_bytes_written: *mut i32,
        ) -> tresult {
            if !num_bytes_written.is_null() {
                *num_bytes_written = 0;
            }
            if num_bytes < 0 {
                return kInvalidArgument;
            }
            if num_bytes == 0 {
                return kResultOk;
            }
            if buffer.is_null() {
                return kInvalidArgument;
            }

            let Ok(mut bytes) = self.bytes.lock() else {
                return kInvalidArgument;
            };
            let Ok(mut cursor) = self.cursor.lock() else {
                return kInvalidArgument;
            };

            let end = (*cursor).saturating_add(num_bytes as usize);
            if end > bytes.len() {
                bytes.resize(end, 0);
            }

            std::ptr::copy_nonoverlapping(
                buffer.cast::<u8>(),
                bytes.as_mut_ptr().add(*cursor),
                num_bytes as usize,
            );
            *cursor = end;
            if !num_bytes_written.is_null() {
                *num_bytes_written = num_bytes;
            }

            kResultOk
        }

        unsafe fn seek(&self, pos: i64, mode: i32, result: *mut i64) -> tresult {
            let Ok(bytes) = self.bytes.lock() else {
                return kInvalidArgument;
            };
            let Ok(mut cursor) = self.cursor.lock() else {
                return kInvalidArgument;
            };

            let base = match mode {
                value if value == kIBSeekSet as i32 => 0_i64,
                value if value == kIBSeekCur as i32 => *cursor as i64,
                value if value == kIBSeekEnd as i32 => bytes.len() as i64,
                _ => return kInvalidArgument,
            };

            let Some(next) = base.checked_add(pos) else {
                return kInvalidArgument;
            };
            if next < 0 {
                return kInvalidArgument;
            }

            *cursor = next as usize;
            if !result.is_null() {
                *result = next;
            }

            kResultOk
        }

        unsafe fn tell(&self, pos: *mut i64) -> tresult {
            if pos.is_null() {
                return kInvalidArgument;
            }

            let Ok(cursor) = self.cursor.lock() else {
                return kInvalidArgument;
            };
            *pos = *cursor as i64;
            kResultOk
        }
    }

    enum HostAttributeValue {
        Int(int64),
        Float(f64),
        String(Vec<TChar>),
        Binary(Box<[u8]>),
    }

    struct HostAttributeEntry {
        key: Vec<u8>,
        value: HostAttributeValue,
    }

    #[derive(Default)]
    struct HostAttributeList {
        entries: StdMutex<Vec<HostAttributeEntry>>,
    }

    impl Class for HostAttributeList {
        type Interfaces = (IAttributeList,);
    }

    impl HostAttributeList {
        fn set_entry(&self, key: Vec<u8>, value: HostAttributeValue) -> tresult {
            let Ok(mut entries) = self.entries.lock() else {
                return kInvalidArgument;
            };

            if let Some(entry) = entries.iter_mut().find(|entry| entry.key == key) {
                entry.value = value;
            } else {
                entries.push(HostAttributeEntry { key, value });
            }

            kResultOk
        }
    }

    impl IAttributeListTrait for HostAttributeList {
        unsafe fn setInt(&self, id: AttrID, value: int64) -> tresult {
            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            self.set_entry(key, HostAttributeValue::Int(value))
        }

        unsafe fn getInt(&self, id: AttrID, value: *mut int64) -> tresult {
            if value.is_null() {
                return kInvalidArgument;
            }

            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            let Ok(entries) = self.entries.lock() else {
                return kInvalidArgument;
            };
            let Some(entry) = entries.iter().find(|entry| entry.key == key) else {
                return kResultFalse;
            };

            match &entry.value {
                HostAttributeValue::Int(stored) => {
                    *value = *stored;
                    kResultOk
                }
                _ => kResultFalse,
            }
        }

        unsafe fn setFloat(&self, id: AttrID, value: f64) -> tresult {
            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            self.set_entry(key, HostAttributeValue::Float(value))
        }

        unsafe fn getFloat(&self, id: AttrID, value: *mut f64) -> tresult {
            if value.is_null() {
                return kInvalidArgument;
            }

            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            let Ok(entries) = self.entries.lock() else {
                return kInvalidArgument;
            };
            let Some(entry) = entries.iter().find(|entry| entry.key == key) else {
                return kResultFalse;
            };

            match &entry.value {
                HostAttributeValue::Float(stored) => {
                    *value = *stored;
                    kResultOk
                }
                _ => kResultFalse,
            }
        }

        unsafe fn setString(&self, id: AttrID, string: *const TChar) -> tresult {
            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            let Some(value) = copy_tchar_string(string) else {
                return kInvalidArgument;
            };

            self.set_entry(key, HostAttributeValue::String(value))
        }

        unsafe fn getString(
            &self,
            id: AttrID,
            string: *mut TChar,
            size_in_bytes: uint32,
        ) -> tresult {
            if string.is_null() || size_in_bytes == 0 {
                return kInvalidArgument;
            }

            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            let Ok(entries) = self.entries.lock() else {
                return kInvalidArgument;
            };
            let Some(entry) = entries.iter().find(|entry| entry.key == key) else {
                return kResultFalse;
            };

            match &entry.value {
                HostAttributeValue::String(stored) => {
                    copy_tchar_string_to_buffer(stored, string, size_in_bytes)
                }
                _ => kResultFalse,
            }
        }

        unsafe fn setBinary(
            &self,
            id: AttrID,
            data: *const ::std::ffi::c_void,
            size_in_bytes: uint32,
        ) -> tresult {
            if data.is_null() && size_in_bytes > 0 {
                return kInvalidArgument;
            }

            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            let value = if size_in_bytes == 0 {
                Box::<[u8]>::default()
            } else {
                std::slice::from_raw_parts(data.cast::<u8>(), size_in_bytes as usize)
                    .to_vec()
                    .into_boxed_slice()
            };

            self.set_entry(key, HostAttributeValue::Binary(value))
        }

        unsafe fn getBinary(
            &self,
            id: AttrID,
            data: *mut *const ::std::ffi::c_void,
            size_in_bytes: *mut uint32,
        ) -> tresult {
            if data.is_null() || size_in_bytes.is_null() {
                return kInvalidArgument;
            }

            let Some(key) = attribute_key(id) else {
                return kInvalidArgument;
            };
            let Ok(entries) = self.entries.lock() else {
                return kInvalidArgument;
            };
            let Some(entry) = entries.iter().find(|entry| entry.key == key) else {
                return kResultFalse;
            };

            match &entry.value {
                HostAttributeValue::Binary(stored) => {
                    *data = stored.as_ptr().cast::<c_void>();
                    *size_in_bytes = stored.len() as uint32;
                    kResultOk
                }
                _ => kResultFalse,
            }
        }
    }

    struct HostMessage {
        message_id: StdMutex<Option<CString>>,
        attributes: ComPtr<IAttributeList>,
    }

    #[derive(Default)]
    struct HostEventList {
        events: Arc<StdMutex<Vec<Event>>>,
    }

    impl Class for HostEventList {
        type Interfaces = (IEventList,);
    }

    impl IEventListTrait for HostEventList {
        unsafe fn getEventCount(&self) -> Steinberg::int32 {
            self.events
                .lock()
                .map(|events| events.len() as Steinberg::int32)
                .unwrap_or(0)
        }

        unsafe fn getEvent(&self, index: Steinberg::int32, event: *mut Event) -> tresult {
            let Ok(events) = self.events.lock() else {
                return kInvalidArgument;
            };
            let Some(stored) = events.get(index as usize) else {
                return kResultFalse;
            };
            if event.is_null() {
                return kInvalidArgument;
            }
            *event = *stored;
            kResultOk
        }

        unsafe fn addEvent(&self, event: *mut Event) -> tresult {
            if event.is_null() {
                return kInvalidArgument;
            }
            let Ok(mut events) = self.events.lock() else {
                return kInvalidArgument;
            };
            events.push(*event);
            kResultOk
        }
    }

    impl HostMessage {
        fn new() -> Result<Self, String> {
            let attributes = ComWrapper::new(HostAttributeList::default())
                .to_com_ptr::<IAttributeList>()
                .ok_or_else(|| String::from("Failed to allocate the VST3 attribute-list shim."))?;

            Ok(Self {
                message_id: StdMutex::new(None),
                attributes,
            })
        }
    }

    impl Class for HostMessage {
        type Interfaces = (IMessage,);
    }

    impl IMessageTrait for HostMessage {
        unsafe fn getMessageID(&self) -> FIDString {
            let Ok(message_id) = self.message_id.lock() else {
                return std::ptr::null();
            };

            message_id
                .as_ref()
                .map(|message_id| message_id.as_ptr())
                .unwrap_or(std::ptr::null())
        }

        unsafe fn setMessageID(&self, id: FIDString) {
            let Ok(mut message_id) = self.message_id.lock() else {
                return;
            };

            if id.is_null() {
                *message_id = None;
                return;
            }

            *message_id = Some(CStr::from_ptr(id).to_owned());
        }

        unsafe fn getAttributes(&self) -> *mut IAttributeList {
            self.attributes.clone().into_raw()
        }
    }

    fn make_live_render_event(event: LiveRenderEvent) -> Event {
        Event {
            busIndex: 0,
            sampleOffset: event.sample_offset as i32,
            ppqPosition: 0.0,
            flags: VstEventFlags::kIsLive as u16,
            r#type: match event.kind {
                LiveRenderEventKind::NoteOn => VstEventTypes::kNoteOnEvent as u16,
                LiveRenderEventKind::NoteOff => VstEventTypes::kNoteOffEvent as u16,
            },
            __field0: match event.kind {
                LiveRenderEventKind::NoteOn => vst3::Steinberg::Vst::Event__type0 {
                    noteOn: NoteOnEvent {
                        channel: 0,
                        pitch: i16::from(event.pitch),
                        tuning: 0.0,
                        velocity: f32::from(event.velocity) / 127.0,
                        length: 0,
                        noteId: event.note_id,
                    },
                },
                LiveRenderEventKind::NoteOff => vst3::Steinberg::Vst::Event__type0 {
                    noteOff: NoteOffEvent {
                        channel: 0,
                        pitch: i16::from(event.pitch),
                        velocity: f32::from(event.velocity) / 127.0,
                        noteId: event.note_id,
                        tuning: 0.0,
                    },
                },
            },
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct OutputBusLayout {
        bus_index: i32,
        channel_count: usize,
    }

    #[derive(Default)]
    struct RenderScratch {
        events: Arc<StdMutex<Vec<Event>>>,
        event_list: Option<ComPtr<IEventList>>,
        output_bus_channels_data: Vec<Vec<Vec<f32>>>,
    }

    fn output_bus_layouts(component: &ComPtr<IComponent>) -> Vec<OutputBusLayout> {
        let bus_count = unsafe {
            component.getBusCount(
                Vst::MediaTypes_::kAudio as i32,
                Vst::BusDirections_::kOutput as i32,
            )
        };

        if bus_count <= 0 {
            return vec![OutputBusLayout {
                bus_index: 0,
                channel_count: 2,
            }];
        }

        let mut layouts = Vec::new();
        for bus_index in 0..bus_count {
            let mut bus_info: BusInfo = unsafe { zeroed() };
            let result = unsafe {
                component.getBusInfo(
                    Vst::MediaTypes_::kAudio as i32,
                    Vst::BusDirections_::kOutput as i32,
                    bus_index,
                    &mut bus_info,
                )
            };

            if result == kResultOk && bus_info.channelCount > 0 {
                layouts.push(OutputBusLayout {
                    bus_index,
                    channel_count: bus_info.channelCount as usize,
                });
            }
        }

        if layouts.is_empty() {
            vec![OutputBusLayout {
                bus_index: 0,
                channel_count: 2,
            }]
        } else {
            layouts
        }
    }

    fn activate_component_buses(component: &ComPtr<IComponent>) -> Result<(), String> {
        for media_type in [
            Vst::MediaTypes_::kAudio as i32,
            Vst::MediaTypes_::kEvent as i32,
        ] {
            for direction in [
                Vst::BusDirections_::kInput as i32,
                Vst::BusDirections_::kOutput as i32,
            ] {
                let bus_count = unsafe { component.getBusCount(media_type, direction) };
                if bus_count < 0 {
                    return Err(format!(
                        "Failed to query VST bus count (media_type={}, direction={}, count={}).",
                        media_type, direction, bus_count
                    ));
                }

                for bus_index in 0..bus_count {
                    let activate_result =
                        unsafe { component.activateBus(media_type, direction, bus_index, 1) };
                    if activate_result != kResultOk && activate_result != kResultTrue {
                        return Err(format!(
                            "Failed to activate VST bus (media_type={}, direction={}, bus_index={}, tresult={}).",
                            media_type, direction, bus_index, activate_result
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    impl PluginHostSession {
        fn ensure_render_scratch(&mut self, frames_per_block: usize) -> Result<(), String> {
            if self.render_scratch.event_list.is_none() {
                self.render_scratch.event_list = ComWrapper::new(HostEventList {
                    events: self.render_scratch.events.clone(),
                })
                .to_com_ptr::<IEventList>();
            }

            if self.render_scratch.event_list.is_none() {
                return Err(String::from("Failed to allocate the VST event-list shim."));
            }

            if self.render_scratch.output_bus_channels_data.is_empty()
                || self.render_scratch.output_bus_channels_data.len()
                    != self.render_output_buses.len()
                || self
                    .render_scratch
                    .output_bus_channels_data
                    .iter()
                    .zip(&self.render_output_buses)
                    .any(|(channels, layout)| {
                        channels.len() != layout.channel_count
                            || channels
                                .iter()
                                .any(|channel| channel.len() < frames_per_block)
                    })
            {
                self.render_scratch.output_bus_channels_data = self
                    .render_output_buses
                    .iter()
                    .map(|layout| {
                        (0..layout.channel_count)
                            .map(|_| vec![0.0_f32; frames_per_block])
                            .collect::<Vec<_>>()
                    })
                    .collect();
            }

            Ok(())
        }

        fn ensure_render_processing(
            &mut self,
            sample_rate: u32,
            max_samples_per_block: i32,
        ) -> Result<ComPtr<Vst::IAudioProcessor>, String> {
            let processor = self
                .component
                .cast::<Vst::IAudioProcessor>()
                .ok_or_else(|| {
                    format!(
                        "Playback slot {:?} does not expose an audio processor for realtime rendering.",
                        self.target.slot_id
                    )
                })?;

            if !self.render_buses_activated {
                activate_component_buses(&self.component)?;
                self.render_output_buses = output_bus_layouts(&self.component);
                self.render_buses_activated = true;
            }

            let sample_rate_changed = self.render_sample_rate != Some(sample_rate);
            let block_size_changed = max_samples_per_block > self.render_max_block_size;
            if self.render_processing_active && (sample_rate_changed || block_size_changed) {
                let _ = unsafe { processor.setProcessing(0) };
                self.render_processing_active = false;
            }

            if !self.render_processing_active || sample_rate_changed || block_size_changed {
                let mut setup = ProcessSetup {
                    processMode: 0,
                    symbolicSampleSize: 0,
                    maxSamplesPerBlock: max_samples_per_block,
                    sampleRate: f64::from(sample_rate),
                };

                let setup_result = unsafe { processor.setupProcessing(&mut setup) };
                if setup_result != kResultOk {
                    return Err(format!(
                        "Failed to configure slot realtime processing (tresult = {}).",
                        setup_result
                    ));
                }

                let processing_result = unsafe { processor.setProcessing(1) };
                if processing_result != kResultOk {
                    return Err(format!(
                        "Failed to enable slot realtime processing (tresult = {}).",
                        processing_result
                    ));
                }

                self.render_processing_active = true;
                self.render_sample_rate = Some(sample_rate);
                self.render_max_block_size = self.render_max_block_size.max(max_samples_per_block);
            }

            Ok(processor)
        }

        fn stop_render_processing(&mut self) {
            if !self.render_processing_active {
                return;
            }

            if let Some(processor) = self.component.cast::<Vst::IAudioProcessor>() {
                let _ = unsafe { processor.setProcessing(0) };
            }

            self.render_processing_active = false;
            self.render_sample_rate = None;
            self.render_max_block_size = 0;
        }
    }

    fn render_plugin_session_block(
        session: &mut PluginHostSession,
        sample_rate: u32,
        frames_per_block: usize,
        events: &[LiveRenderEvent],
        output: &mut [f32],
        output_channels: usize,
    ) -> Result<bool, String> {
        let processor = session.ensure_render_processing(sample_rate, frames_per_block as i32)?;
        if session.render_output_buses.is_empty() {
            session.render_output_buses = vec![OutputBusLayout {
                bus_index: 0,
                channel_count: 2,
            }];
        }
        session.ensure_render_scratch(frames_per_block)?;

        {
            let mut shared_event_buffer = session
                .render_scratch
                .events
                .lock()
                .map_err(|_| String::from("Failed to populate slot realtime MIDI events."))?;
            shared_event_buffer.clear();
            shared_event_buffer.extend(events.iter().copied().map(make_live_render_event));
        }

        let mut output_bus_ptrs =
            Vec::with_capacity(session.render_scratch.output_bus_channels_data.len());
        let mut output_buses =
            Vec::with_capacity(session.render_scratch.output_bus_channels_data.len());
        for bus_index in 0..session.render_scratch.output_bus_channels_data.len() {
            let channels = &mut session.render_scratch.output_bus_channels_data[bus_index];
            let mut channel_ptrs = Vec::with_capacity(channels.len());

            for channel in channels.iter_mut() {
                channel[..frames_per_block].fill(0.0);
                channel_ptrs.push(channel.as_mut_ptr());
            }

            output_buses.push(AudioBusBuffers {
                numChannels: channels.len() as i32,
                silenceFlags: 0,
                __field0: AudioBusBuffers__type0 {
                    channelBuffers32: channel_ptrs.as_mut_ptr(),
                },
            });
            output_bus_ptrs.push(channel_ptrs);
        }

        for (bus, channel_ptrs) in output_buses.iter_mut().zip(output_bus_ptrs.iter_mut()) {
            bus.__field0 = AudioBusBuffers__type0 {
                channelBuffers32: channel_ptrs.as_mut_ptr(),
            };
        }

        let event_list = session
            .render_scratch
            .event_list
            .as_ref()
            .ok_or_else(|| String::from("Failed to allocate the VST event-list shim."))?;

        let mut process_data = ProcessData {
            processMode: 0,
            symbolicSampleSize: 0,
            numSamples: frames_per_block as i32,
            numInputs: 0,
            numOutputs: output_buses.len() as i32,
            inputs: std::ptr::null_mut(),
            outputs: output_buses.as_mut_ptr(),
            inputParameterChanges: std::ptr::null_mut(),
            outputParameterChanges: std::ptr::null_mut(),
            inputEvents: event_list.as_ptr(),
            outputEvents: std::ptr::null_mut(),
            processContext: std::ptr::null_mut(),
        };

        let process_result = unsafe { processor.process(&mut process_data) };
        if process_result != kResultOk {
            return Err(format!(
                "Slot realtime processing failed while rendering audio (tresult = {}).",
                process_result
            ));
        }

        let bus_peaks = session
            .render_scratch
            .output_bus_channels_data
            .iter()
            .map(|bus_channels| {
                let mut bus_peak = 0.0_f32;
                for channel in bus_channels {
                    for sample in channel {
                        bus_peak = bus_peak.max(sample.abs());
                    }
                }
                bus_peak
            })
            .collect::<Vec<_>>();

        let selected_bus_index = bus_peaks
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| {
                left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index);

        let mut produced_audio = false;
        if let Some(selected_bus_index) = selected_bus_index {
            let bus_channels = &session.render_scratch.output_bus_channels_data[selected_bus_index];
            if !bus_channels.is_empty() {
                for frame_index in 0..frames_per_block {
                    for out_channel in 0..output_channels {
                        let session_channel = if bus_channels.len() == 1 {
                            0
                        } else {
                            out_channel.min(bus_channels.len() - 1)
                        };
                        let sample = bus_channels[session_channel][frame_index];
                        output[frame_index * output_channels + out_channel] += sample;
                        produced_audio |= sample.abs() > f32::EPSILON;
                    }
                }
            }
        }

        Ok(produced_audio)
    }

    pub(crate) fn render_slot_runtime_block(
        slot_id: UniqueId,
        sample_rate: u32,
        frames_per_block: usize,
        events: &[LiveRenderEvent],
        output: &mut [f32],
        output_channels: usize,
    ) -> super::SlotRuntimeRenderOutcome {
        let Ok(mut sessions) = slot_runtime_sessions().try_lock() else {
            return super::SlotRuntimeRenderOutcome::Busy;
        };
        let Some(session) = sessions.get_mut(&slot_id) else {
            return super::SlotRuntimeRenderOutcome::MissingSession;
        };

        match render_plugin_session_block(
            session,
            sample_rate,
            frames_per_block,
            events,
            output,
            output_channels,
        ) {
            Ok(true) => super::SlotRuntimeRenderOutcome::Rendered,
            Ok(false) => super::SlotRuntimeRenderOutcome::SilenceOnly,
            Err(error) => super::SlotRuntimeRenderOutcome::Error(error),
        }
    }

    pub(crate) fn stop_slot_runtime_render(slot_id: UniqueId) {
        let Ok(mut sessions) = slot_runtime_sessions().try_lock() else {
            return;
        };

        if let Some(session) = sessions.get_mut(&slot_id) {
            session.stop_render_processing();
        }
    }

    unsafe fn attribute_key(id: AttrID) -> Option<Vec<u8>> {
        if id.is_null() {
            return None;
        }

        Some(CStr::from_ptr(id).to_bytes().to_vec())
    }

    unsafe fn copy_tchar_string(string: *const TChar) -> Option<Vec<TChar>> {
        if string.is_null() {
            return None;
        }

        let mut length = 0_usize;
        while *string.add(length) != 0 {
            length += 1;
        }

        Some(std::slice::from_raw_parts(string, length).to_vec())
    }

    unsafe fn copy_tchar_string_to_buffer(
        value: &[TChar],
        buffer: *mut TChar,
        size_in_bytes: uint32,
    ) -> tresult {
        if buffer.is_null() || size_in_bytes < std::mem::size_of::<TChar>() as uint32 {
            return kInvalidArgument;
        }

        let max_units = size_in_bytes as usize / std::mem::size_of::<TChar>();
        if max_units == 0 {
            return kInvalidArgument;
        }

        let units_to_copy = value.len().min(max_units.saturating_sub(1));
        std::ptr::copy_nonoverlapping(value.as_ptr(), buffer, units_to_copy);
        *buffer.add(units_to_copy) = 0;
        kResultOk
    }

    unsafe fn tuid_matches(candidate: *mut TUID, expected: &TUID) -> bool {
        !candidate.is_null() && *candidate == *expected
    }

    unsafe fn query_host_support_interface(
        instance: ComPtr<FUnknown>,
        iid: *mut TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        let ptr = instance.as_ptr();
        ((*(*ptr).vtbl).queryInterface)(ptr, iid, obj)
    }

    fn full_host_supports_interface(iid: &TUID) -> bool {
        *iid == IAutomationState_iid
            || *iid == IComponentHandler_iid
            || *iid == IHostApplication_iid
            || *iid == IPlugInterfaceSupport_iid
            || *iid == IRunLoop_iid
            || *iid == FUnknown_iid
            || *iid == IMessage_iid
            || *iid == IAttributeList_iid
            || *iid == IComponentHandler2_iid
            || *iid == IComponentHandler3_iid
            || *iid == IComponentHandlerBusActivation_iid
            || *iid == IComponentHandlerSystemTime_iid
            || *iid == IContextMenuTarget_iid
            || *iid == IDataExchangeHandler_iid
            || *iid == IEditControllerHostEditing_iid
            || *iid == IProgress_iid
            || *iid == IUnitHandler_iid
            || *iid == IUnitHandler2_iid
    }

    unsafe fn host_application_create_instance(
        cid: *mut TUID,
        iid: *mut TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        super::log_active_probe_host_callback(&format!(
            "IHostApplication::createInstance requested cid={} iid={}",
            super::host_callback_iid_name_from_ptr(cid.cast_const()),
            super::host_callback_iid_name_from_ptr(iid.cast_const())
        ));

        if !obj.is_null() {
            *obj = std::ptr::null_mut();
        }

        if tuid_matches(cid, &IMessage_iid) || tuid_matches(iid, &IMessage_iid) {
            let Ok(message) = HostMessage::new() else {
                return kNotImplemented;
            };
            let Some(instance) = ComWrapper::new(message).to_com_ptr::<FUnknown>() else {
                return kNotImplemented;
            };
            return query_host_support_interface(instance, iid, obj);
        }

        if tuid_matches(cid, &IAttributeList_iid) || tuid_matches(iid, &IAttributeList_iid) {
            let Some(instance) =
                ComWrapper::new(HostAttributeList::default()).to_com_ptr::<FUnknown>()
            else {
                return kNotImplemented;
            };
            return query_host_support_interface(instance, iid, obj);
        }

        kNotImplemented
    }

    unsafe fn host_application_support_result(
        iid: *const TUID,
        supports: fn(&TUID) -> bool,
    ) -> tresult {
        if iid.is_null() {
            return kInvalidArgument;
        }

        let interface_name = super::host_callback_iid_label(&*iid);
        let is_supported = supports(&*iid);
        super::log_active_probe_host_callback(&format!(
            "IPlugInterfaceSupport::isPlugInterfaceSupported queried {} => {}",
            interface_name,
            if is_supported {
                "supported"
            } else {
                "unsupported"
            }
        ));

        if is_supported {
            kResultTrue
        } else {
            kResultFalse
        }
    }

    #[repr(C)]
    struct WrapperProbeFallbackUnknown {
        unknown: FUnknown,
        parent: *mut LoggingHostOnlyInitHostApplication,
    }

    impl WrapperProbeFallbackUnknown {
        fn new() -> Self {
            Self {
                unknown: FUnknown {
                    vtbl: &WRAPPER_PROBE_FALLBACK_UNKNOWN_VTBL,
                },
                parent: std::ptr::null_mut(),
            }
        }
    }

    #[repr(C)]
    struct LoggingHostOnlyInitHostApplication {
        host_application: IHostApplication,
        ref_count: AtomicU32,
        wrapper_probe_fallback_unknown: WrapperProbeFallbackUnknown,
    }

    impl LoggingHostOnlyInitHostApplication {
        fn new() -> Self {
            Self {
                host_application: IHostApplication {
                    vtbl: &LOGGING_HOST_ONLY_INIT_HOST_APPLICATION_VTBL,
                },
                ref_count: AtomicU32::new(1),
                wrapper_probe_fallback_unknown: WrapperProbeFallbackUnknown::new(),
            }
        }
    }

    static WRAPPER_PROBE_FALLBACK_UNKNOWN_VTBL: FUnknownVtbl = FUnknownVtbl {
        queryInterface: wrapper_probe_fallback_unknown_query_interface,
        addRef: wrapper_probe_fallback_unknown_add_ref,
        release: wrapper_probe_fallback_unknown_release,
    };

    static LOGGING_HOST_ONLY_INIT_HOST_APPLICATION_VTBL: IHostApplicationVtbl =
        IHostApplicationVtbl {
            base: FUnknownVtbl {
                queryInterface: logging_host_only_init_query_interface,
                addRef: logging_host_only_init_add_ref,
                release: logging_host_only_init_release,
            },
            getName: logging_host_only_init_get_name,
            createInstance: logging_host_only_init_create_instance,
        };

    unsafe fn logging_host_only_init_instance_from_unknown(
        this: *mut FUnknown,
    ) -> *mut LoggingHostOnlyInitHostApplication {
        this.cast::<LoggingHostOnlyInitHostApplication>()
    }

    unsafe fn wrapper_probe_fallback_unknown_from_unknown(
        this: *mut FUnknown,
    ) -> *mut WrapperProbeFallbackUnknown {
        this.cast::<WrapperProbeFallbackUnknown>()
    }

    unsafe fn logging_host_only_init_add_ref_count(
        instance: *mut LoggingHostOnlyInitHostApplication,
    ) -> uint32 {
        (*instance).ref_count.fetch_add(1, Ordering::AcqRel) + 1
    }

    unsafe fn logging_host_only_init_release_ref_count(
        instance: *mut LoggingHostOnlyInitHostApplication,
    ) -> uint32 {
        let remaining = (*instance).ref_count.fetch_sub(1, Ordering::AcqRel) - 1;
        if remaining == 0 {
            drop(Box::from_raw(instance));
        }
        remaining
    }

    unsafe fn wrapper_probe_fallback_parent(
        this: *mut FUnknown,
    ) -> *mut LoggingHostOnlyInitHostApplication {
        (*wrapper_probe_fallback_unknown_from_unknown(this)).parent
    }

    unsafe extern "system" fn wrapper_probe_fallback_unknown_query_interface(
        this: *mut FUnknown,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        if !obj.is_null() {
            *obj = std::ptr::null_mut();
        }

        if iid.is_null() {
            return kInvalidArgument;
        }

        if *iid != FUnknown_iid {
            return kNoInterface;
        }

        let instance = wrapper_probe_fallback_unknown_from_unknown(this);
        let parent = wrapper_probe_fallback_parent(this);
        let _ = logging_host_only_init_add_ref_count(parent);
        if !obj.is_null() {
            *obj = std::ptr::addr_of_mut!((*instance).unknown).cast::<c_void>();
        }

        kResultOk
    }

    unsafe extern "system" fn wrapper_probe_fallback_unknown_add_ref(
        this: *mut FUnknown,
    ) -> uint32 {
        logging_host_only_init_add_ref_count(wrapper_probe_fallback_parent(this))
    }

    unsafe extern "system" fn wrapper_probe_fallback_unknown_release(
        this: *mut FUnknown,
    ) -> uint32 {
        logging_host_only_init_release_ref_count(wrapper_probe_fallback_parent(this))
    }

    unsafe extern "system" fn logging_host_only_init_query_interface(
        this: *mut FUnknown,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        if !obj.is_null() {
            *obj = std::ptr::null_mut();
        }

        if iid.is_null() {
            return kInvalidArgument;
        }

        let requested_interface = super::host_callback_iid_label(&*iid);
        let is_supported = *iid == IHostApplication_iid || *iid == FUnknown_iid;
        let uses_wrapper_fallback = super::is_wrapper_probe_interface(&*iid);
        super::log_active_probe_host_callback(&format!(
            "Init host context queryInterface requested {} => {}",
            requested_interface,
            if is_supported {
                "supported"
            } else if uses_wrapper_fallback {
                "unsupported_with_fallback"
            } else {
                "unsupported"
            }
        ));

        let instance = logging_host_only_init_instance_from_unknown(this);

        if !is_supported && !uses_wrapper_fallback {
            return kNoInterface;
        }

        let _ = logging_host_only_init_add_ref_count(instance);
        if !obj.is_null() {
            *obj = if uses_wrapper_fallback {
                std::ptr::addr_of_mut!((*instance).wrapper_probe_fallback_unknown.unknown)
                    .cast::<c_void>()
            } else {
                std::ptr::addr_of_mut!((*instance).host_application).cast::<c_void>()
            };
        }

        if uses_wrapper_fallback {
            return kNoInterface;
        }

        kResultOk
    }

    unsafe extern "system" fn logging_host_only_init_add_ref(this: *mut FUnknown) -> uint32 {
        let instance = logging_host_only_init_instance_from_unknown(this);
        logging_host_only_init_add_ref_count(instance)
    }

    unsafe extern "system" fn logging_host_only_init_release(this: *mut FUnknown) -> uint32 {
        let instance = logging_host_only_init_instance_from_unknown(this);
        logging_host_only_init_release_ref_count(instance)
    }

    unsafe extern "system" fn logging_host_only_init_get_name(
        _this: *mut IHostApplication,
        name: *mut String128,
    ) -> tresult {
        if name.is_null() {
            return kInvalidArgument;
        }

        super::log_active_probe_host_callback("IHostApplication::getName invoked");
        write_string128(&mut *name, "Solo Compose Plugin Host POC");
        kResultOk
    }

    unsafe extern "system" fn logging_host_only_init_create_instance(
        _this: *mut IHostApplication,
        cid: *mut TUID,
        iid: *mut TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        host_application_create_instance(cid, iid, obj)
    }

    fn create_logging_host_only_init_host_application() -> ComPtr<IHostApplication> {
        let instance = Box::new(LoggingHostOnlyInitHostApplication::new());
        let raw_instance = Box::into_raw(instance);
        unsafe {
            (*raw_instance).wrapper_probe_fallback_unknown.parent = raw_instance;
        }
        let host_application_ptr =
            unsafe { std::ptr::addr_of_mut!((*raw_instance).host_application) };

        unsafe { ComPtr::from_raw_unchecked(host_application_ptr) }
    }

    pub struct PluginHostSession {
        pub target: super::PluginEditorTarget,
        pub plugin_path: PathBuf,
        restorable_plugin_state: Option<PlaybackSlotPluginState>,
        _module: LoadedVst3Module,
        component: ComPtr<IComponent>,
        controller: ComPtr<IEditController>,
        view: Option<ComPtr<IPlugView>>,
        _host_application: ComPtr<IHostApplication>,
        _controller_initialize_host_application: ComPtr<IHostApplication>,
        _component_handler_host_application: ComPtr<IHostApplication>,
        _plug_frame: Option<ComPtr<IPlugFrame>>,
        host_view_ptr: usize,
        plugin_can_resize: bool,
        view_attached: bool,
        connection_points_connected: bool,
        component_initialized: bool,
        controller_initialized: bool,
        controller_shared_with_component: bool,
        close_prepared: bool,
        render_processing_active: bool,
        render_sample_rate: Option<u32>,
        render_max_block_size: i32,
        render_buses_activated: bool,
        render_output_buses: Vec<OutputBusLayout>,
        render_scratch: RenderScratch,
    }

    impl PluginHostSession {
        fn attach_view(
            &mut self,
            ns_window: *mut NSWindow,
            parent_view: &NSView,
            telemetry: &mut super::PluginHostBuildTelemetry,
        ) -> Result<(), PluginEditorHostAttemptError> {
            eprintln!(
                "[DIAG] attach_view: start (close_prepared={}, view_attached={}, view_exists={})",
                self.close_prepared,
                self.view_attached,
                self.view.is_some()
            );

            if self.component_initialized && self.close_prepared {
                let _ = unsafe { self.component.setActive(1) };
            }

            telemetry.record_starting(
                PluginEditorHostValidationPhase::ViewCreate,
                "About to create and prepare the VST3 editor view.",
            );
            let plug_frame = ComWrapper::new(PluginHostFrame {
                ns_window_ptr: ns_window,
            })
            .to_com_ptr::<IPlugFrame>()
            .ok_or_else(|| {
                super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::host_contract_failure(
                        PluginEditorHostValidationPhase::ViewCreate,
                        "Failed to allocate the VST3 plug-frame shim.",
                    ),
                )
            })?;

            let view = unsafe { ComPtr::from_raw(self.controller.createView(ViewType::kEditor)) }
                .ok_or_else(|| {
                    super::record_probe_failure(
                        telemetry,
                        PluginEditorHostAttemptError::plugin_failure(
                            PluginEditorHostValidationPhase::ViewCreate,
                            "Plugin editor POC did not receive a plugin editor view. The selected VST3 may not expose an editor.",
                        ),
                    )
                })?;

            let platform_supported = unsafe { view.isPlatformTypeSupported(kPlatformTypeNSView) };
            if platform_supported != kResultOk {
                return Err(super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ViewCreate,
                        "Plugin editor POC editor view does not report NSView support.",
                    ),
                ));
            }

            let mut plug_view_size = ViewRect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if unsafe { view.getSize(&mut plug_view_size) } == kResultOk {
                let width = (plug_view_size.right - plug_view_size.left).max(1) as f64;
                let height = (plug_view_size.bottom - plug_view_size.top).max(1) as f64;
                eprintln!(
                    "[DIAG] attach_view: getSize returned {width}x{height}, calling setContentSize"
                );
                if let Some(ns_win) = unsafe { Retained::retain(ns_window) } {
                    ns_win.setContentSize(NSSize::new(width, height));
                }
            }

            let plugin_can_resize = unsafe { view.canResize() } == kResultOk;
            if let Some(ns_win) = unsafe { Retained::retain(ns_window) } {
                let mut style = ns_win.styleMask();
                if plugin_can_resize {
                    style |= NSWindowStyleMask::Resizable;
                } else {
                    style -= NSWindowStyleMask::Resizable;
                }
                ns_win.setStyleMask(style);
            }

            telemetry.record_completed(
                PluginEditorHostValidationPhase::ViewCreate,
                "Created the VST3 editor view and prepared the host window.",
            );

            let _ = unsafe { view.setFrame(plug_frame.as_ptr()) };

            telemetry.record_starting(
                PluginEditorHostValidationPhase::ViewAttach,
                "About to attach the VST3 editor view to the bounded host surface.",
            );
            let parent_ptr = (parent_view as *const NSView).cast_mut().cast::<c_void>();
            let attached = unsafe { view.attached(parent_ptr, kPlatformTypeNSView) };
            if attached != kResultOk {
                return Err(super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ViewAttach,
                        format!(
                            "Plugin editor POC failed to attach the VST3 editor view to NSView (tresult = {}).",
                            attached
                        ),
                    ),
                ));
            }

            telemetry.record_completed(
                PluginEditorHostValidationPhase::ViewAttach,
                "Attached the plugin editor view to the bounded host surface.",
            );

            self.view = Some(view);
            self._plug_frame = Some(plug_frame);
            self.host_view_ptr = parent_ptr as usize;
            self.plugin_can_resize = plugin_can_resize;
            self.view_attached = true;
            self.close_prepared = false;

            Ok(())
        }

        fn prepare_for_window_close(&mut self) {
            unsafe {
                if self.close_prepared {
                    return;
                }

                if self.view_attached {
                    let Some(view) = self.view.as_ref() else {
                        self.view_attached = false;
                        self.close_prepared = true;
                        return;
                    };
                    let _ = view.setFrame(std::ptr::null_mut());
                    let _ = view.removed();
                    self.view_attached = false;
                    self.view = None;
                    self._plug_frame = None;
                }
                self.close_prepared = true;
            }
        }

        fn shutdown(mut self) {
            self.prepare_for_window_close();

            unsafe {
                if self.controller_initialized && !self.controller_shared_with_component {
                    let _ = self.controller.setComponentHandler(std::ptr::null_mut());
                }
                if self.connection_points_connected {
                    disconnect_component_and_controller(&self.component, &self.controller);
                    self.connection_points_connected = false;
                }
                if self.controller_initialized && !self.controller_shared_with_component {
                    let _ = self.controller.terminate();
                    self.controller_initialized = false;
                }
                if self.component_initialized {
                    let _ = self.component.setActive(0);
                    let _ = self.component.terminate();
                    self.component_initialized = false;
                }
            }
        }

        fn view_state(&self) -> Option<(ComPtr<IPlugView>, usize, bool)> {
            self.view
                .as_ref()
                .map(|view| (view.clone(), self.host_view_ptr, self.plugin_can_resize))
        }
    }

    // These helper functions are no longer needed — the synchronous
    // Steinberg-aligned resize model handles layout directly in resizeView
    // and sync_active_plugin_host.

    struct LoadedVst3Module {
        binary_path: PathBuf,
        bundle_ref: Option<usize>,
        get_plugin_factory: GetPluginFactoryFn,
        bundle_exit: Option<BundleExitFn>,
    }

    impl Drop for LoadedVst3Module {
        fn drop(&mut self) {
            if let Some(bundle_exit) = self.bundle_exit {
                unsafe {
                    let _ = bundle_exit();
                }
            }
            if let Some(bundle_ref) = self.bundle_ref.take() {
                unsafe {
                    CFBundleUnloadExecutable(bundle_ref as _);
                    CFRelease(bundle_ref as CFTypeRef);
                }
            }
        }
    }

    macro_rules! impl_common_host_application_traits {
        ($ty:ty, $supports:path) => {
            impl IHostApplicationTrait for $ty {
                unsafe fn getName(&self, name: *mut String128) -> tresult {
                    if name.is_null() {
                        return kInvalidArgument;
                    }

                    super::log_active_probe_host_callback("IHostApplication::getName invoked");
                    write_string128(&mut *name, "Solo Compose Plugin Host POC");
                    kResultOk
                }

                unsafe fn createInstance(
                    &self,
                    cid: *mut TUID,
                    iid: *mut TUID,
                    obj: *mut *mut c_void,
                ) -> tresult {
                    host_application_create_instance(cid, iid, obj)
                }
            }

            impl IPlugInterfaceSupportTrait for $ty {
                unsafe fn isPlugInterfaceSupported(&self, iid: *const TUID) -> tresult {
                    host_application_support_result(iid, $supports)
                }
            }

            impl IAutomationStateTrait for $ty {
                unsafe fn setAutomationState(&self, _state: i32) -> tresult {
                    kResultOk
                }
            }

            impl IRunLoopTrait for $ty {
                unsafe fn registerEventHandler(
                    &self,
                    handler: *mut IEventHandler,
                    _fd: FileDescriptor,
                ) -> tresult {
                    if handler.is_null() {
                        return kInvalidArgument;
                    }
                    kResultOk
                }

                unsafe fn unregisterEventHandler(&self, handler: *mut IEventHandler) -> tresult {
                    if handler.is_null() {
                        return kInvalidArgument;
                    }
                    kResultOk
                }

                unsafe fn registerTimer(
                    &self,
                    handler: *mut ITimerHandler,
                    _milliseconds: TimerInterval,
                ) -> tresult {
                    if handler.is_null() {
                        return kInvalidArgument;
                    }
                    kResultOk
                }

                unsafe fn unregisterTimer(&self, handler: *mut ITimerHandler) -> tresult {
                    if handler.is_null() {
                        return kInvalidArgument;
                    }
                    kResultOk
                }
            }

            impl IComponentHandlerTrait for $ty {
                unsafe fn beginEdit(&self, _id: Vst::ParamID) -> tresult {
                    kResultOk
                }

                unsafe fn performEdit(
                    &self,
                    _id: Vst::ParamID,
                    _value_normalized: Vst::ParamValue,
                ) -> tresult {
                    kResultOk
                }

                unsafe fn endEdit(&self, _id: Vst::ParamID) -> tresult {
                    kResultOk
                }

                unsafe fn restartComponent(&self, _flags: i32) -> tresult {
                    kResultOk
                }
            }
        };
    }

    macro_rules! impl_extended_host_application_traits {
        ($ty:ty) => {
            impl IComponentHandler2Trait for $ty {
                unsafe fn setDirty(&self, _state: TBool) -> tresult {
                    kResultOk
                }

                unsafe fn requestOpenEditor(&self, _name: FIDString) -> tresult {
                    kResultOk
                }

                unsafe fn startGroupEdit(&self) -> tresult {
                    kResultOk
                }

                unsafe fn finishGroupEdit(&self) -> tresult {
                    kResultOk
                }
            }

            impl IComponentHandler3Trait for $ty {
                unsafe fn createContextMenu(
                    &self,
                    _plugView: *mut IPlugView,
                    _paramID: *const Vst::ParamID,
                ) -> *mut IContextMenu {
                    std::ptr::null_mut()
                }
            }

            impl IComponentHandlerBusActivationTrait for $ty {
                unsafe fn requestBusActivation(
                    &self,
                    _type: Vst::MediaType,
                    _dir: Vst::BusDirection,
                    _index: i32,
                    _state: TBool,
                ) -> tresult {
                    kResultOk
                }
            }

            impl IComponentHandlerSystemTimeTrait for $ty {
                unsafe fn getSystemTime(&self, systemTime: *mut int64) -> tresult {
                    if systemTime.is_null() {
                        return kInvalidArgument;
                    }

                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|duration| duration.as_millis() as int64)
                        .unwrap_or_default();
                    *systemTime = now;
                    kResultOk
                }
            }

            impl IContextMenuTargetTrait for $ty {
                unsafe fn executeMenuItem(&self, _tag: i32) -> tresult {
                    kResultOk
                }
            }

            impl IDataExchangeHandlerTrait for $ty {
                unsafe fn openQueue(
                    &self,
                    _processor: *mut Vst::IAudioProcessor,
                    _blockSize: uint32,
                    _numBlocks: uint32,
                    _alignment: uint32,
                    _userContextID: Vst::DataExchangeUserContextID,
                    outID: *mut Vst::DataExchangeQueueID,
                ) -> tresult {
                    if !outID.is_null() {
                        *outID = 0;
                    }
                    kResultOk
                }

                unsafe fn closeQueue(&self, _queueID: Vst::DataExchangeQueueID) -> tresult {
                    kResultOk
                }

                unsafe fn lockBlock(
                    &self,
                    _queueId: Vst::DataExchangeQueueID,
                    _block: *mut Vst::DataExchangeBlock,
                ) -> tresult {
                    kResultOk
                }

                unsafe fn freeBlock(
                    &self,
                    _queueId: Vst::DataExchangeQueueID,
                    _blockID: Vst::DataExchangeBlockID,
                    _sendToController: TBool,
                ) -> tresult {
                    kResultOk
                }
            }

            impl IEditControllerHostEditingTrait for $ty {
                unsafe fn beginEditFromHost(&self, _paramID: Vst::ParamID) -> tresult {
                    kResultOk
                }

                unsafe fn endEditFromHost(&self, _paramID: Vst::ParamID) -> tresult {
                    kResultOk
                }
            }

            impl IUnitHandlerTrait for $ty {
                unsafe fn notifyUnitSelection(&self, _unitId: Vst::UnitID) -> tresult {
                    kResultOk
                }

                unsafe fn notifyProgramListChange(
                    &self,
                    _listId: Vst::ProgramListID,
                    _programIndex: i32,
                ) -> tresult {
                    kResultOk
                }
            }

            impl IUnitHandler2Trait for $ty {
                unsafe fn notifyUnitByBusChange(&self) -> tresult {
                    kResultOk
                }
            }

            impl IProgressTrait for $ty {
                unsafe fn start(
                    &self,
                    _type: ProgressType,
                    _optionalDescription: *const vst3::Steinberg::tchar,
                    outID: *mut ProgressID,
                ) -> tresult {
                    if !outID.is_null() {
                        *outID = 0;
                    }
                    kResultOk
                }

                unsafe fn update(&self, _id: ProgressID, _normValue: Vst::ParamValue) -> tresult {
                    kResultOk
                }

                unsafe fn finish(&self, _id: ProgressID) -> tresult {
                    kResultOk
                }
            }
        };
    }

    pub(crate) struct PluginHostApplication;

    impl Class for PluginHostApplication {
        type Interfaces = (
            IHostApplication,
            IPlugInterfaceSupport,
            IRunLoop,
            IAutomationState,
            IComponentHandler,
            IComponentHandler2,
            IComponentHandler3,
            IComponentHandlerBusActivation,
            IComponentHandlerSystemTime,
            IContextMenuTarget,
            IDataExchangeHandler,
            IEditControllerHostEditing,
            IUnitHandler,
            IUnitHandler2,
            IProgress,
        );
    }

    impl_common_host_application_traits!(PluginHostApplication, full_host_supports_interface);
    impl_extended_host_application_traits!(PluginHostApplication);

    pub(crate) fn create_controller_init_host_application(
        profile: super::PluginEditorPocProbeProfile,
    ) -> Result<ComPtr<IHostApplication>, String> {
        match profile {
            super::PluginEditorPocProbeProfile::Default => ComWrapper::new(PluginHostApplication)
                .to_com_ptr::<IHostApplication>()
                .ok_or_else(|| String::from("Failed to allocate the VST3 host-application shim.")),
            super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned => {
                Ok(create_logging_host_only_init_host_application())
            }
        }
    }

    fn create_component_handler_host_application(
        _profile: super::PluginEditorPocProbeProfile,
    ) -> Result<ComPtr<IHostApplication>, String> {
        ComWrapper::new(PluginHostApplication)
            .to_com_ptr::<IHostApplication>()
            .ok_or_else(|| String::from("Failed to allocate the VST3 component-handler host shim."))
    }

    struct PluginHostFrame {
        /// Raw pointer to the native NSWindow.  The PluginHostFrame does not own
        /// the window lifetime — the global `native_host_window()` holds the
        /// Retained<NSWindow>.  This pointer is valid for as long as the window
        /// exists.
        ns_window_ptr: *mut NSWindow,
    }

    // SAFETY: The ns_window_ptr is only dereferenced on the main thread (inside
    // resizeView which is called synchronously by the plugin on the main thread).
    unsafe impl Send for PluginHostFrame {}
    unsafe impl Sync for PluginHostFrame {}

    impl Class for PluginHostFrame {
        type Interfaces = (IPlugFrame,);
    }

    impl IPlugFrameTrait for PluginHostFrame {
        unsafe fn resizeView(&self, view: *mut IPlugView, new_size: *mut ViewRect) -> tresult {
            if new_size.is_null() || view.is_null() {
                return kInvalidArgument;
            }

            let new_size = *new_size;
            eprintln!(
                "[DIAG] resizeView: called with {}x{}",
                new_size.right - new_size.left,
                new_size.bottom - new_size.top
            );

            // Recursion guard matching the Steinberg editorhost pattern.
            // If this resizeView is already in-flight (e.g. the plugin called
            // resizeView from inside onSize), return immediately.
            if resizing_from_plugin().swap(true, Ordering::AcqRel) {
                return kResultFalse;
            }

            // Steinberg editorhost pattern: check getSize first and return
            // immediately if the view already reports the requested size.
            let view_ptr = ComPtr::from_raw(view);
            let result = (|| -> tresult {
                let Some(ref view_com) = view_ptr else {
                    return kResultFalse;
                };

                let mut current_rect = ViewRect {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                if view_com.getSize(&mut current_rect) == kResultOk
                    && current_rect.left == new_size.left
                    && current_rect.top == new_size.top
                    && current_rect.right == new_size.right
                    && current_rect.bottom == new_size.bottom
                {
                    return kResultTrue;
                }

                let width = (new_size.right - new_size.left).max(1) as f64;
                let height = (new_size.bottom - new_size.top).max(1) as f64;

                let Some(ns_window) = Retained::retain(self.ns_window_ptr) else {
                    return kResultFalse;
                };

                // Steinberg editorhost Window::resize pattern:
                // compute current content rect, build new frame rect from
                // desired content size, apply with setFrame:display:animate:.
                let current_frame = ns_window.frame();
                let current_content: NSRect =
                    msg_send![&*ns_window, contentRectForFrameRect: current_frame];
                let mut new_content = current_content;
                // Anchor top-left: adjust origin.y for height change.
                new_content.origin.y += current_content.size.height - height;
                new_content.size.width = width;
                new_content.size.height = height;
                let new_frame: NSRect =
                    msg_send![&*ns_window, frameRectForContentRect: new_content];
                let _: () =
                    msg_send![&*ns_window, setFrame: new_frame, display: true, animate: false];

                // After resize, check if view still disagrees and forward
                // onSize if needed (Steinberg editorhost pattern).
                let mut post_rect = ViewRect {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                if view_com.getSize(&mut post_rect) == kResultOk
                    && (post_rect.left != new_size.left
                        || post_rect.top != new_size.top
                        || post_rect.right != new_size.right
                        || post_rect.bottom != new_size.bottom)
                {
                    let mut target_rect = new_size;
                    view_com.onSize(&mut target_rect);
                }

                kResultTrue
            })();

            // Prevent ComPtr from releasing the view — we don't own it.
            std::mem::forget(view_ptr);

            resizing_from_plugin().store(false, Ordering::Release);
            result
        }
    }

    fn write_string128(buffer: &mut String128, value: &str) {
        buffer.fill(0);
        for (index, code_unit) in value
            .encode_utf16()
            .take(buffer.len().saturating_sub(1))
            .enumerate()
        {
            buffer[index] = code_unit;
        }
    }

    fn load_vst3_module(plugin_path: &Path) -> Result<LoadedVst3Module, String> {
        let binary_path = vst3_bundle_binary_path(plugin_path).map_err(|error| error.message)?;
        let bundle = CFURL::from_path(plugin_path, true)
            .and_then(CFBundle::new)
            .ok_or_else(|| {
                format!(
                    "Failed to create a CFBundle for VST3 bundle '{}'.",
                    plugin_path.display()
                )
            })?;
        let bundle_ref = unsafe { CFRetain(bundle.as_concrete_TypeRef() as CFTypeRef) as usize };
        let executable_loaded = unsafe { CFBundleLoadExecutable(bundle_ref as _) != 0 };
        if !executable_loaded {
            unsafe {
                CFRelease(bundle_ref as CFTypeRef);
            }
            return Err(format!(
                "Failed to load the VST3 bundle executable for '{}'.",
                plugin_path.display()
            ));
        }

        let get_plugin_factory_ptr =
            bundle.function_pointer_for_name(CFString::from_static_string("GetPluginFactory"));
        if get_plugin_factory_ptr.is_null() {
            unsafe {
                CFBundleUnloadExecutable(bundle_ref as _);
                CFRelease(bundle_ref as CFTypeRef);
            }
            return Err(format!(
                "Failed to resolve GetPluginFactory in '{}'.",
                binary_path.display()
            ));
        }

        let get_plugin_factory: GetPluginFactoryFn =
            unsafe { std::mem::transmute(get_plugin_factory_ptr) };

        let bundle_entry_ptr =
            bundle.function_pointer_for_name(CFString::from_static_string("BundleEntry"));
        if !bundle_entry_ptr.is_null() {
            let bundle_entry: BundleEntryFn = unsafe { std::mem::transmute(bundle_entry_ptr) };
            let entered = unsafe { bundle_entry(bundle_ref as *mut c_void) };
            if !entered {
                unsafe {
                    CFBundleUnloadExecutable(bundle_ref as _);
                    CFRelease(bundle_ref as CFTypeRef);
                }
                return Err(format!(
                    "VST3 bundle entry point returned false for '{}'.",
                    binary_path.display()
                ));
            }
        }

        let bundle_exit = {
            let bundle_exit_ptr =
                bundle.function_pointer_for_name(CFString::from_static_string("BundleExit"));
            if bundle_exit_ptr.is_null() {
                None
            } else {
                Some(unsafe { std::mem::transmute::<*const c_void, BundleExitFn>(bundle_exit_ptr) })
            }
        };

        Ok(LoadedVst3Module {
            binary_path,
            bundle_ref: Some(bundle_ref),
            get_plugin_factory,
            bundle_exit,
        })
    }

    fn create_plugin_factory(module: &LoadedVst3Module) -> Result<ComPtr<IPluginFactory>, String> {
        let raw_factory = unsafe { (module.get_plugin_factory)() };
        unsafe { ComPtr::from_raw(raw_factory) }.ok_or_else(|| {
            format!(
                "GetPluginFactory returned a null factory pointer for '{}'.",
                module.binary_path.display()
            )
        })
    }

    fn configure_factory_host_context(
        factory: &ComPtr<IPluginFactory>,
        host_context: &ComPtr<FUnknown>,
    ) -> Option<tresult> {
        let factory3 = factory.cast::<IPluginFactory3>()?;

        let host_context_result = unsafe { factory3.setHostContext(host_context.as_ptr()) };
        Some(host_context_result)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn should_pass_host_context_to_controller_initialize(
        _factory_host_context_result: Option<tresult>,
    ) -> bool {
        true
    }

    fn class_info_string(field: &[i8]) -> String {
        let bytes = field
            .iter()
            .copied()
            .take_while(|byte| *byte != 0)
            .map(|byte| byte as u8)
            .collect::<Vec<_>>();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub(crate) fn class_info_is_audio_module(info: &Steinberg::PClassInfo) -> bool {
        class_info_string(&info.category) == "Audio Module Class"
    }

    fn normalized_plugin_match_key(value: &str) -> String {
        value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .map(|character| character.to_ascii_lowercase())
            .collect()
    }

    fn requested_component_name(plugin_path: &Path) -> Option<String> {
        plugin_path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn component_class_match_score(
        candidate: &ComponentClassCandidate,
        requested_name: &str,
    ) -> usize {
        let requested_key = normalized_plugin_match_key(requested_name);
        if requested_key.is_empty() {
            return 0;
        }

        let name_key = normalized_plugin_match_key(&candidate.name);
        if !name_key.is_empty() {
            if name_key == requested_key {
                return 400;
            }
            if name_key.starts_with(&requested_key) || requested_key.starts_with(&name_key) {
                return 300;
            }
            if name_key.contains(&requested_key) || requested_key.contains(&name_key) {
                return 200;
            }
        }

        let subcategory_key = normalized_plugin_match_key(&candidate.subcategories);
        if !subcategory_key.is_empty() {
            if subcategory_key == requested_key {
                return 150;
            }
            if subcategory_key.contains(&requested_key) || requested_key.contains(&subcategory_key)
            {
                return 100;
            }
        }

        0
    }

    pub(crate) fn select_component_class_candidate<'a>(
        candidates: &'a [ComponentClassCandidate],
        requested_name: Option<&str>,
    ) -> Option<&'a ComponentClassCandidate> {
        let Some(requested_name) = requested_name else {
            return candidates.first();
        };

        let best_match = candidates
            .iter()
            .max_by_key(|candidate| component_class_match_score(candidate, requested_name));

        match best_match {
            Some(candidate) if component_class_match_score(candidate, requested_name) > 0 => {
                Some(candidate)
            }
            _ => candidates.first(),
        }
    }

    fn audio_module_class_candidates(
        factory: &ComPtr<IPluginFactory>,
    ) -> Vec<ComponentClassCandidate> {
        if let Some(factory2) = factory.cast::<IPluginFactory2>() {
            let mut candidates = Vec::new();

            for index in 0..unsafe { factory2.countClasses() } {
                let mut info = unsafe { zeroed::<Steinberg::PClassInfo2>() };
                if unsafe { factory2.getClassInfo2(index, &mut info) } != kResultOk {
                    continue;
                }

                let category = class_info_string(&info.category);
                let name = class_info_string(&info.name);
                if category != "Audio Module Class" {
                    continue;
                }

                candidates.push(ComponentClassCandidate {
                    cid: info.cid,
                    name,
                    category,
                    subcategories: class_info_string(&info.subCategories),
                });
            }

            return candidates;
        }

        let mut candidates = Vec::new();
        for index in 0..unsafe { factory.countClasses() } {
            let mut info = unsafe { zeroed::<Steinberg::PClassInfo>() };
            if unsafe { factory.getClassInfo(index, &mut info) } != kResultOk {
                continue;
            }

            if !class_info_is_audio_module(&info) {
                continue;
            }

            candidates.push(ComponentClassCandidate {
                cid: info.cid,
                name: class_info_string(&info.name),
                category: class_info_string(&info.category),
                subcategories: String::new(),
            });
        }

        candidates
    }

    fn create_component(
        factory: &ComPtr<IPluginFactory>,
        plugin_path: &Path,
    ) -> Result<ComPtr<IComponent>, String> {
        let candidates = audio_module_class_candidates(factory);
        let requested_name = requested_component_name(plugin_path);

        let candidate = select_component_class_candidate(&candidates, requested_name.as_deref())
            .ok_or_else(|| {
                String::from(
                    "Plugin editor POC could not find an audio-module component class in the VST3 factory.",
                )
            })?;
        let mut raw_object: *mut c_void = std::ptr::null_mut();
        let result = unsafe {
            factory.createInstance(
                candidate.cid.as_ptr(),
                IComponent_iid.as_ptr(),
                &mut raw_object,
            )
        };
        if result != kResultOk {
            return Err(format!(
                "Plugin editor POC failed to create the selected VST3 component class '{}' (tresult = {}).",
                candidate.name,
                result
            ));
        }

        unsafe { ComPtr::from_raw(raw_object.cast::<IComponent>()) }.ok_or_else(|| {
            format!(
                "Plugin editor POC received a null VST3 component pointer for '{}'.",
                candidate.name
            )
        })
    }

    fn create_controller(
        factory: &ComPtr<IPluginFactory>,
        component: &ComPtr<IComponent>,
    ) -> Result<CreatedController, String> {
        if let Some(controller) = component.cast::<IEditController>() {
            return Ok(CreatedController {
                controller,
                shared_with_component: true,
            });
        }

        let mut controller_cid: TUID = [0; 16];
        let result = unsafe { component.getControllerClassId(&mut controller_cid) };
        if result != kResultOk || controller_cid.iter().all(|byte| *byte == 0) {
            return Err(String::from(
                "Plugin editor POC could not resolve a controller class ID for the VST3 component.",
            ));
        }
        let mut raw_object: *mut c_void = std::ptr::null_mut();
        let create_result = unsafe {
            factory.createInstance(
                controller_cid.as_ptr(),
                IEditController_iid.as_ptr(),
                &mut raw_object,
            )
        };
        if create_result != kResultOk {
            return Err(String::from(
                "Plugin editor POC failed to create the VST3 edit controller instance.",
            ));
        }

        unsafe { ComPtr::from_raw(raw_object.cast::<IEditController>()) }
            .map(|controller| CreatedController {
                controller,
                shared_with_component: false,
            })
            .ok_or_else(|| {
                String::from("Plugin editor POC received a null VST3 edit controller pointer.")
            })
    }

    fn create_memory_stream() -> Result<(HostMemoryStream, ComPtr<IBStream>), String> {
        let stream = HostMemoryStream::default();
        let stream_ptr = ComWrapper::new(stream.clone())
            .to_com_ptr::<IBStream>()
            .ok_or_else(|| String::from("Failed to allocate the VST3 memory stream shim."))?;

        Ok((stream, stream_ptr))
    }

    fn sync_component_state_to_controller(
        component: &ComPtr<IComponent>,
        controller: &ComPtr<IEditController>,
    ) {
        let Ok((_, stream)) = create_memory_stream() else {
            return;
        };

        let get_state_result = unsafe { component.getState(stream.as_ptr()) };
        if get_state_result != kResultOk {
            return;
        }

        let mut rewind_position = 0_i64;
        let rewind_result = unsafe { stream.seek(0, kIBSeekSet as i32, &mut rewind_position) };
        if rewind_result != kResultOk {
            return;
        }

        let _set_state_result = unsafe { controller.setComponentState(stream.as_ptr()) };
    }

    fn export_component_state(
        component: &ComPtr<IComponent>,
    ) -> Result<SlotRuntimePluginStateCapture, String> {
        let (stream_state, stream) = create_memory_stream()?;
        let get_state_result = unsafe { component.getState(stream.as_ptr()) };
        if get_state_result != kResultOk {
            return Ok(SlotRuntimePluginStateCapture::NoTransferableState);
        }

        let bytes = stream_state.snapshot_bytes()?;
        Ok(SlotRuntimePluginStateCapture::Captured(
            PlaybackSlotPluginState {
                vst3_component_state_base64: BASE64_STANDARD.encode(bytes),
            },
        ))
    }

    fn restore_component_state(
        component: &ComPtr<IComponent>,
        plugin_state: &PlaybackSlotPluginState,
    ) -> Result<(), String> {
        let bytes = BASE64_STANDARD
            .decode(plugin_state.vst3_component_state_base64.as_bytes())
            .map_err(|error| format!("Failed to decode persisted VST3 component state: {error}"))?;
        let (stream_state, stream) = create_memory_stream()?;
        stream_state.replace_bytes(bytes)?;

        let set_state_result = unsafe { component.setState(stream.as_ptr()) };
        if set_state_result != kResultOk {
            return Err(format!(
                "Plugin editor POC failed to restore the persisted VST3 component state (tresult = {}).",
                set_state_result
            ));
        }

        Ok(())
    }

    pub(crate) fn slot_runtime_session_matches_target_path_and_state(
        session_plugin_path: &Path,
        session_plugin_state: Option<&PlaybackSlotPluginState>,
        desired_target: &ActivatedSlotTarget,
    ) -> bool {
        session_plugin_path == desired_target.bundle_path.as_path()
            && session_plugin_state == desired_target.plugin_state.as_ref()
    }

    fn slot_runtime_session_matches_target(
        session: &PluginHostSession,
        desired_target: &ActivatedSlotTarget,
    ) -> bool {
        slot_runtime_session_matches_target_path_and_state(
            &session.plugin_path,
            session.restorable_plugin_state.as_ref(),
            desired_target,
        )
    }

    unsafe fn disconnect_component_and_controller(
        component: &ComPtr<IComponent>,
        controller: &ComPtr<IEditController>,
    ) {
        let Some(component_connection) = component.cast::<IConnectionPoint>() else {
            return;
        };
        let Some(controller_connection) = controller.cast::<IConnectionPoint>() else {
            return;
        };

        let _ = controller_connection.disconnect(component_connection.as_ptr());
        let _ = component_connection.disconnect(controller_connection.as_ptr());
    }

    enum ConnectionPointHandshakeOutcome {
        Connected(String),
        Skipped(String),
    }

    fn connect_component_and_controller(
        component: &ComPtr<IComponent>,
        controller: &ComPtr<IEditController>,
    ) -> ConnectionPointHandshakeOutcome {
        let Some(component_connection) = component.cast::<IConnectionPoint>() else {
            return ConnectionPointHandshakeOutcome::Skipped(String::from(
                "Plugin component does not expose IConnectionPoint; handshake skipped.",
            ));
        };
        let Some(controller_connection) = controller.cast::<IConnectionPoint>() else {
            return ConnectionPointHandshakeOutcome::Skipped(String::from(
                "Plugin controller does not expose IConnectionPoint; handshake skipped.",
            ));
        };

        let component_connect_result =
            unsafe { component_connection.connect(controller_connection.as_ptr()) };
        let controller_connect_result =
            unsafe { controller_connection.connect(component_connection.as_ptr()) };
        if component_connect_result == kResultOk && controller_connect_result == kResultOk {
            return ConnectionPointHandshakeOutcome::Connected(String::from(
                "Connected component and controller connection points.",
            ));
        }
        unsafe {
            let _ = controller_connection.disconnect(component_connection.as_ptr());
            let _ = component_connection.disconnect(controller_connection.as_ptr());
        }
        ConnectionPointHandshakeOutcome::Skipped(format!(
            "Component/controller connection-point handshake returned component={} and controller={}; continuing without a connected pair.",
            component_connect_result, controller_connect_result
        ))
    }

    fn apply_component_runtime_setup(
        component: &ComPtr<IComponent>,
        telemetry: &mut super::PluginHostBuildTelemetry,
    ) -> Result<(), PluginEditorHostAttemptError> {
        if let Some(processor) = component.cast::<Vst::IAudioProcessor>() {
            activate_component_buses(component).map_err(|error| {
                super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ComponentRuntimeSetup,
                        error,
                    ),
                )
            })?;

            let mut setup = Vst::ProcessSetup {
                processMode: 0,
                symbolicSampleSize: 0,
                maxSamplesPerBlock: 1024,
                sampleRate: 44100.0,
            };
            let _ = unsafe { processor.setupProcessing(&mut setup) };
            let _ = unsafe { component.setActive(1) };
        }

        Ok(())
    }

    fn attach_error_message(
        mtm: MainThreadMarker,
        host_view: &NSView,
        host_frame: NSRect,
        plugin_path: &Path,
        error: &str,
    ) {
        let message = format!(
            "Live VST3 attach failed for:\n{}\n\n{}",
            plugin_path.display(),
            error
        );
        let label_text = NSString::from_str(&message);
        let label = NSTextField::wrappingLabelWithString(&label_text, mtm);
        label.setFrame(NSRect::new(
            NSPoint::new(24.0, 24.0),
            NSSize::new(
                (host_frame.size.width - 48.0).max(220.0),
                (host_frame.size.height - 48.0).max(120.0),
            ),
        ));
        label.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        host_view.addSubview(&label);
    }

    fn build_plugin_host_session(
        target: &super::PluginEditorTarget,
        restorable_plugin_state: Option<&PlaybackSlotPluginState>,
    ) -> Result<PluginHostSession, PluginEditorHostAttemptError> {
        let mut telemetry = super::PluginHostBuildTelemetry::disabled();
        build_plugin_host_session_with_telemetry(
            target,
            restorable_plugin_state,
            super::PluginEditorPocProbeProfile::Default,
            &mut telemetry,
        )
    }

    fn build_plugin_host_session_with_telemetry(
        target: &super::PluginEditorTarget,
        restorable_plugin_state: Option<&PlaybackSlotPluginState>,
        probe_profile: super::PluginEditorPocProbeProfile,
        telemetry: &mut super::PluginHostBuildTelemetry,
    ) -> Result<PluginHostSession, PluginEditorHostAttemptError> {
        let plugin_path = target.path();
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ModuleLoad,
            "About to load the VST3 module binary.",
        );
        let module = load_vst3_module(plugin_path).map_err(|error| {
            super::record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::host_contract_failure(
                    PluginEditorHostValidationPhase::ModuleLoad,
                    error,
                ),
            )
        })?;
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ModuleLoad,
            "Loaded the VST3 module binary.",
        );
        telemetry.record_starting(
            PluginEditorHostValidationPhase::FactoryCreate,
            "About to resolve the VST3 plugin factory.",
        );
        let factory = create_plugin_factory(&module).map_err(|error| {
            super::record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::FactoryCreate,
                    error,
                ),
            )
        })?;
        telemetry.record_completed(
            PluginEditorHostValidationPhase::FactoryCreate,
            "Resolved the VST3 plugin factory.",
        );

        telemetry.record_starting(
            PluginEditorHostValidationPhase::HostContext,
            "About to allocate the VST3 host application and configure the factory host context.",
        );
        let host_application =
            create_component_handler_host_application(probe_profile).map_err(|error| {
                super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::host_contract_failure(
                        PluginEditorHostValidationPhase::HostContext,
                        error,
                    ),
                )
            })?;
        let controller_initialize_host_application = match probe_profile {
            super::PluginEditorPocProbeProfile::Default => host_application.clone(),
            super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned => {
                create_controller_init_host_application(probe_profile).map_err(|error| {
                    super::record_probe_failure(
                        telemetry,
                        PluginEditorHostAttemptError::host_contract_failure(
                            PluginEditorHostValidationPhase::HostContext,
                            error,
                        ),
                    )
                })?
            }
        };
        let component_handler_host_application =
            create_component_handler_host_application(probe_profile).map_err(|error| {
                super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::host_contract_failure(
                        PluginEditorHostValidationPhase::HostContext,
                        error,
                    ),
                )
            })?;
        let host_context = host_application.clone().upcast::<FUnknown>();
        configure_factory_host_context(&factory, &host_context);
        telemetry.record_completed(
            PluginEditorHostValidationPhase::HostContext,
            "Created the host context and configured the VST3 factory.",
        );

        telemetry.record_starting(
            PluginEditorHostValidationPhase::ComponentCreate,
            "About to create the VST3 component instance.",
        );
        let component = create_component(&factory, plugin_path).map_err(|error| {
            super::record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::ComponentCreate,
                    error,
                ),
            )
        })?;
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ComponentCreate,
            "Created the VST3 component instance.",
        );
        let delay_component_runtime_setup =
            super::probe_profile_delays_component_runtime_setup(probe_profile);
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ComponentInitialize,
            if delay_component_runtime_setup {
                "About to initialize the VST3 component. Runtime bus activation and processing setup will be deferred until after controller initialization."
            } else {
                "About to initialize the VST3 component before controller initialization."
            },
        );
        let component_init = unsafe { component.initialize(host_context.as_ptr()) };
        if component_init != kResultOk {
            return Err(super::record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::ComponentInitialize,
                    format!(
                        "Plugin editor POC failed to initialize the VST3 component (tresult = {}).",
                        component_init
                    ),
                ),
            ));
        }

        if let Some(plugin_state) = restorable_plugin_state {
            telemetry.record_starting(
                PluginEditorHostValidationPhase::ComponentStateSync,
                "About to restore the portable component state into the VST3 component.",
            );
            restore_component_state(&component, plugin_state).map_err(|error| {
                super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ComponentStateSync,
                        error,
                    ),
                )
            })?;
            telemetry.record_completed(
                PluginEditorHostValidationPhase::ComponentStateSync,
                "Restored the portable component state into the VST3 component.",
            );
        } else {
            telemetry.record_skipped(
                PluginEditorHostValidationPhase::ComponentStateSync,
                "Skipped because no portable component state was provided.",
            );
        }

        telemetry.record_completed(
            PluginEditorHostValidationPhase::ComponentInitialize,
            if delay_component_runtime_setup {
                "Initialized the VST3 component and deferred runtime bus activation, processing setup, and component activation until after controller initialization."
            } else {
                "Initialized the VST3 component."
            },
        );

        if !delay_component_runtime_setup {
            telemetry.record_starting(
                PluginEditorHostValidationPhase::ComponentRuntimeSetup,
                "About to activate buses, configure processing, and activate the VST3 component before controller initialization.",
            );
            apply_component_runtime_setup(&component, telemetry)?;
            telemetry.record_completed(
                PluginEditorHostValidationPhase::ComponentRuntimeSetup,
                "Activated buses, configured processing, and activated the VST3 component before controller initialization.",
            );
        }

        telemetry.record_starting(
            PluginEditorHostValidationPhase::ControllerCreate,
            "About to create the VST3 edit controller.",
        );
        let CreatedController {
            controller,
            shared_with_component: controller_shared_with_component,
        } = match create_controller(&factory, &component) {
            Ok(controller) => controller,
            Err(error) => {
                unsafe {
                    let _ = component.terminate();
                }
                return Err(super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ControllerCreate,
                        error,
                    ),
                ));
            }
        };
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ControllerCreate,
            "Created the VST3 edit controller.",
        );

        // IComponentHandler is now on PluginHostApplication, so we use
        // the host_application ComPtr cast for setComponentHandler.

        // Set the component handler BEFORE controller.initialize().
        // HALion Sonic's controller may rely on having a valid handler
        // during initialization.
        let component_handler_ptr = component_handler_host_application
            .clone()
            .upcast::<FUnknown>()
            .cast::<IComponentHandler>()
            .expect("PluginHostApplication must expose IComponentHandler");
        let set_component_handler_pre_init =
            !super::probe_profile_defers_component_handler(probe_profile);

        if !controller_shared_with_component {
            if set_component_handler_pre_init {
                telemetry.record_starting(
                    PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
                    "About to set the VST3 component handler before controller initialization.",
                );
                let _ = unsafe { controller.setComponentHandler(component_handler_ptr.as_ptr()) };
                telemetry.log_phase_event(
                    PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
                    "completed",
                    "Set the VST3 component handler before controller initialization.",
                );
            } else {
                telemetry.record_skipped(
                    PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
                    format!(
                        "Skipped because the {} probe profile defers component-handler wiring until after controller initialization.",
                        super::probe_profile_code(probe_profile)
                    ),
                );
            }
        } else {
            telemetry.record_skipped(
                PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
                "Skipped because the edit controller is shared with the component instance.",
            );
        }

        // Initialize the edit controller on the main thread.
        // The committed working version (ffaad55a) uses main-thread init
        // with pre-init setComponentHandler, 15 interfaces, and factory
        // host context.
        let controller_initialized = if controller_shared_with_component {
            telemetry.record_skipped(
                PluginEditorHostValidationPhase::ControllerInitialize,
                "Skipped because the edit controller is shared with the component instance.",
            );
            telemetry.record_skipped(
                PluginEditorHostValidationPhase::ControllerSetComponentHandler,
                "Skipped because the edit controller is shared with the component instance.",
            );
            false
        } else {
            let controller_initialize_host_context = controller_initialize_host_application
                .clone()
                .upcast::<FUnknown>();
            telemetry.record_starting(
                PluginEditorHostValidationPhase::ControllerInitialize,
                "About to initialize the VST3 edit controller.",
            );
            let controller_init =
                unsafe { controller.initialize(controller_initialize_host_context.as_ptr()) };
            if controller_init != kResultOk {
                unsafe {
                    let _ = component.terminate();
                }
                return Err(super::record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ControllerInitialize,
                        format!(
                            "Plugin editor POC failed to initialize the VST3 edit controller (tresult = {}).",
                            controller_init
                        ),
                    ),
                ));
            }
            telemetry.record_completed(
                PluginEditorHostValidationPhase::ControllerInitialize,
                "Initialized the VST3 edit controller.",
            );

            if set_component_handler_pre_init {
                telemetry.record_skipped(
                    PluginEditorHostValidationPhase::ControllerSetComponentHandler,
                    "Skipped because the default probe profile already wired the component handler before controller initialization.",
                );
            } else {
                telemetry.record_starting(
                    PluginEditorHostValidationPhase::ControllerSetComponentHandler,
                    "About to set the VST3 component handler after controller initialization.",
                );
                let _ = unsafe { controller.setComponentHandler(component_handler_ptr.as_ptr()) };
                telemetry.record_completed(
                    PluginEditorHostValidationPhase::ControllerSetComponentHandler,
                    "Set the VST3 component handler after controller initialization.",
                );
            }
            true
        };

        // Match original POC order exactly:
        // setComponentHandler → init → state sync → connect points

        // State sync before connect points
        if !controller_shared_with_component {
            telemetry.log_phase_event(
                PluginEditorHostValidationPhase::ComponentStateSync,
                "starting",
                "About to synchronize the live component state into the controller before connection-point setup.",
            );
            sync_component_state_to_controller(&component, &controller);
            telemetry.log_phase_event(
                PluginEditorHostValidationPhase::ComponentStateSync,
                "completed",
                "Synchronized the live component state into the controller before connection-point setup.",
            );
        }

        let connection_points_connected = if controller_shared_with_component {
            telemetry.record_skipped(
                PluginEditorHostValidationPhase::ConnectionPointHandshake,
                "Skipped because the component and controller share one instance.",
            );
            false
        } else {
            telemetry.record_starting(
                PluginEditorHostValidationPhase::ConnectionPointHandshake,
                "About to connect the VST3 component and controller connection points.",
            );
            match connect_component_and_controller(&component, &controller) {
                ConnectionPointHandshakeOutcome::Connected(message) => {
                    telemetry.record_completed(
                        PluginEditorHostValidationPhase::ConnectionPointHandshake,
                        message,
                    );
                    true
                }
                ConnectionPointHandshakeOutcome::Skipped(message) => {
                    telemetry.record_skipped(
                        PluginEditorHostValidationPhase::ConnectionPointHandshake,
                        message,
                    );
                    false
                }
            }
        };

        if delay_component_runtime_setup {
            telemetry.record_starting(
                PluginEditorHostValidationPhase::ComponentRuntimeSetup,
                "About to activate buses, configure processing, and activate the VST3 component after controller initialization.",
            );
            apply_component_runtime_setup(&component, telemetry)?;
            telemetry.record_completed(
                PluginEditorHostValidationPhase::ComponentRuntimeSetup,
                "Activated buses, configured processing, and activated the VST3 component after controller initialization.",
            );
        }

        Ok(PluginHostSession {
            target: target.clone(),
            plugin_path: plugin_path.to_path_buf(),
            restorable_plugin_state: restorable_plugin_state.cloned(),
            _module: module,
            component,
            controller,
            view: None,
            _host_application: host_application,
            _controller_initialize_host_application: controller_initialize_host_application,
            _component_handler_host_application: component_handler_host_application,
            _plug_frame: None,
            host_view_ptr: 0,
            plugin_can_resize: false,
            view_attached: false,
            connection_points_connected,
            component_initialized: true,
            controller_initialized,
            controller_shared_with_component,
            close_prepared: false,
            render_processing_active: false,
            render_sample_rate: None,
            render_max_block_size: 0,
            render_buses_activated: false,
            render_output_buses: Vec::new(),
            render_scratch: RenderScratch::default(),
        })
    }

    fn prepare_plugin_host_session_for_close_now() {
        // Take the session OUT of its registry before calling prepare_for_window_close.
        // prepare_for_window_close calls plugin methods (setFrame, removed) which can
        // trigger synchronous window events. sync_active_plugin_host → active_session_view_state
        // locks these same registries, so the lock must not be held during plugin calls.

        if let Some(slot_id) = active_slot_id() {
            let session = slot_runtime_sessions()
                .lock()
                .ok()
                .and_then(|mut sessions| sessions.remove(&slot_id));
            if let Some(mut session) = session {
                session.prepare_for_window_close();
                if let Ok(mut sessions) = slot_runtime_sessions().lock() {
                    sessions.insert(slot_id, session);
                }
                return;
            }
        }

        let session = active_plugin_host_session()
            .lock()
            .ok()
            .and_then(|mut active_session| active_session.take());
        if let Some(mut session) = session {
            session.prepare_for_window_close();
            if let Ok(mut active_session) = active_plugin_host_session().lock() {
                *active_session = Some(session);
            }
        }
    }

    pub fn cleanup_active_plugin_host_session() {
        // Standard VST3 host pattern: preserve the session (component/controller)
        // across editor open/close cycles.  Only the view is destroyed (already
        // handled by CloseRequested → prepare_for_window_close).  The session is
        // reused on reopen — attach_view creates a fresh view from the existing
        // controller.
        //
        // Destroying and reloading the entire module on every close/reopen cycle
        // corrupts plugin state on macOS because CFBundle may cache the bundle
        // reference and the plugin's global initialisation is not re-entrant.

        // Safety: ensure prepare_for_window_close was called even if
        // CloseRequested didn't fire (e.g., programmatic window destruction).
        prepare_plugin_host_session_for_close_now();

        clear_pending_plugin_resize_request();
        super::clear_reserved_plugin_target();
    }

    fn restore_taken_plugin_host_session(session_state: &Arc<StdMutex<Option<PluginHostSession>>>) {
        let Some(session) = session_state
            .lock()
            .ok()
            .and_then(|mut session_state| session_state.take())
        else {
            return;
        };

        if let Ok(mut active_session) = active_plugin_host_session().lock() {
            *active_session = Some(session);
        }
    }

    fn restore_taken_slot_runtime_session(
        slot_id: UniqueId,
        session_state: &Arc<StdMutex<Option<PluginHostSession>>>,
    ) {
        let Some(session) = session_state
            .lock()
            .ok()
            .and_then(|mut session_state| session_state.take())
        else {
            return;
        };

        if let Ok(mut sessions) = slot_runtime_sessions().lock() {
            sessions.insert(slot_id, session);
        }
    }

    pub fn shutdown_detached_plugin_host_session(
        app: &AppHandle,
        detached_target: &super::PluginEditorTarget,
    ) -> Result<(), String> {
        let session = if let Some(slot_id) = detached_target.slot_id {
            slot_runtime_sessions()
                .lock()
                .map_err(|_| {
                    String::from("Failed to access slot runtime session for target switch")
                })?
                .remove(&slot_id)
        } else {
            active_plugin_host_session()
                .lock()
                .map_err(|_| {
                    String::from("Failed to access detached plugin host session for target switch")
                })?
                .take()
        };

        let Some(session) = session else {
            super::release_reserved_plugin_target(detached_target);
            return Ok(());
        };

        if session.target != *detached_target {
            if let Some(slot_id) = detached_target.slot_id {
                slot_runtime_sessions()
                    .lock()
                    .map_err(|_| {
                        String::from(
                            "Failed to restore slot runtime session after detached target switch mismatch",
                        )
                    })?
                    .insert(slot_id, session);
            } else {
                active_plugin_host_session()
                    .lock()
                    .map_err(|_| {
                        String::from(
                            "Failed to restore plugin host session after detached target switch mismatch",
                        )
                    })?
                    .replace(session);
            }
            return Err(format!(
                "Failed to switch plugin editor targets because the detached session changed from {}.",
                detached_target
            ));
        }

        if MainThreadMarker::new().is_some() {
            finish_plugin_host_session_shutdown(session);
            return Ok(());
        }

        let session_state = Arc::new(StdMutex::new(Some(session)));
        let session_state_for_main = Arc::clone(&session_state);
        let (sender, receiver) = std::sync::mpsc::channel();
        app.run_on_main_thread(move || {
            let result = if let Ok(mut session_state) = session_state_for_main.lock() {
                if let Some(session) = session_state.take() {
                    finish_plugin_host_session_shutdown(session);
                }
                Ok(())
            } else {
                Err(String::from(
                    "Failed to access detached plugin host session while switching targets.",
                ))
            };
            let _ = sender.send(result);
        })
        .map_err(|error| {
            if let Some(slot_id) = detached_target.slot_id {
                restore_taken_slot_runtime_session(slot_id, &session_state);
            } else {
                restore_taken_plugin_host_session(&session_state);
            }
            format!("Failed to schedule detached plugin host teardown: {error}")
        })?;

        let wait_started_at = log_main_thread_wait_start("shutdown_detached_plugin_host_session");
        let teardown_result = receiver.recv().map_err(|error| {
            if let Some(slot_id) = detached_target.slot_id {
                restore_taken_slot_runtime_session(slot_id, &session_state);
            } else {
                restore_taken_plugin_host_session(&session_state);
            }
            format!("Failed to confirm detached plugin host teardown: {error}")
        });
        log_main_thread_wait_finish("shutdown_detached_plugin_host_session", wait_started_at);
        let teardown_result = teardown_result?;

        if let Err(error) = teardown_result {
            if let Some(slot_id) = detached_target.slot_id {
                restore_taken_slot_runtime_session(slot_id, &session_state);
            } else {
                restore_taken_plugin_host_session(&session_state);
            }
            return Err(error);
        }

        Ok(())
    }

    fn build_slot_runtime_session_on_main_thread(
        app: &AppHandle,
        slot_id: UniqueId,
        target: &ActivatedSlotTarget,
    ) -> Result<PluginHostSession, String> {
        let target_identity = super::PluginEditorTarget::slot_with_state(
            target.bundle_path.clone(),
            slot_id,
            target.plugin_state.clone(),
        );
        let restorable_plugin_state = target.plugin_state.clone();

        if MainThreadMarker::new().is_some() {
            return build_plugin_host_session(&target_identity, restorable_plugin_state.as_ref())
                .map_err(|error| error.message);
        }

        let (sender, receiver) = std::sync::mpsc::channel();
        let build_target = target_identity.clone();
        app.run_on_main_thread(move || {
            let result = build_plugin_host_session(&build_target, restorable_plugin_state.as_ref())
                .map_err(|error| error.message);
            let _ = sender.send(result);
        })
        .map_err(|error| format!("Failed to schedule slot runtime creation: {error}"))?;

        let wait_started_at = log_main_thread_wait_start("build_slot_runtime_session");
        let received = receiver
            .recv()
            .map_err(|error| format!("Failed to receive slot runtime creation result: {error}"));
        log_main_thread_wait_finish("build_slot_runtime_session", wait_started_at);
        received?
    }

    fn capture_slot_runtime_plugin_state_now(
        slot_id: UniqueId,
    ) -> Result<SlotRuntimePluginStateCapture, String> {
        let sessions = slot_runtime_sessions().lock().map_err(|_| {
            String::from("Failed to access slot runtime sessions for state capture.")
        })?;
        let Some(session) = sessions.get(&slot_id) else {
            return Ok(SlotRuntimePluginStateCapture::MissingSession);
        };

        export_component_state(&session.component)
    }

    pub(crate) fn capture_slot_runtime_plugin_state(
        app: &AppHandle,
        slot_id: UniqueId,
    ) -> Result<SlotRuntimePluginStateCapture, String> {
        if MainThreadMarker::new().is_some() {
            return capture_slot_runtime_plugin_state_now(slot_id);
        }

        let (sender, receiver) = std::sync::mpsc::channel();
        app.run_on_main_thread(move || {
            let result = capture_slot_runtime_plugin_state_now(slot_id);
            let _ = sender.send(result);
        })
        .map_err(|error| format!("Failed to schedule slot runtime state capture: {error}"))?;

        let wait_started_at = log_main_thread_wait_start("capture_slot_runtime_plugin_state");
        let received = receiver.recv().map_err(|error| {
            format!("Failed to receive slot runtime state capture result: {error}")
        });
        log_main_thread_wait_finish("capture_slot_runtime_plugin_state", wait_started_at);
        received?
    }

    fn shutdown_plugin_host_session_on_main_thread(
        app: &AppHandle,
        session: PluginHostSession,
    ) -> Result<(), String> {
        if MainThreadMarker::new().is_some() {
            finish_plugin_host_session_shutdown(session);
            return Ok(());
        }

        let session_state = Arc::new(StdMutex::new(Some(session)));
        let session_state_for_main = Arc::clone(&session_state);
        let (sender, receiver) = std::sync::mpsc::channel();
        app.run_on_main_thread(move || {
            let result = if let Ok(mut session_state) = session_state_for_main.lock() {
                if let Some(session) = session_state.take() {
                    finish_plugin_host_session_shutdown(session);
                }
                Ok(())
            } else {
                Err(String::from(
                    "Failed to access plugin host session during ownership teardown.",
                ))
            };
            let _ = sender.send(result);
        })
        .map_err(|error| format!("Failed to schedule plugin host teardown: {error}"))?;

        let wait_started_at = log_main_thread_wait_start("shutdown_plugin_host_session");
        let received = receiver
            .recv()
            .map_err(|error| format!("Failed to confirm plugin host teardown: {error}"));
        log_main_thread_wait_finish("shutdown_plugin_host_session", wait_started_at);
        received?
    }

    pub(crate) fn sync_slot_runtime_sessions(
        app: &AppHandle,
        desired_targets: &FxHashMap<UniqueId, ActivatedSlotTarget>,
    ) -> FxHashMap<UniqueId, ActivatedSlotTarget> {
        let mut sessions_to_shutdown = Vec::new();

        if let Ok(mut sessions) = slot_runtime_sessions().lock() {
            let slot_ids_to_remove = sessions
                .iter()
                .filter_map(|(slot_id, session)| match desired_targets.get(slot_id) {
                    Some(desired_target)
                        if slot_runtime_session_matches_target(session, desired_target) =>
                    {
                        None
                    }
                    _ => Some(*slot_id),
                })
                .collect::<Vec<_>>();

            for slot_id in slot_ids_to_remove {
                if let Some(session) = sessions.remove(&slot_id) {
                    sessions_to_shutdown.push(session);
                }
            }
        }

        for session in sessions_to_shutdown {
            if let Err(_error) = shutdown_plugin_host_session_on_main_thread(app, session) {}
        }

        for (slot_id, target) in desired_targets {
            let already_present = slot_runtime_sessions()
                .lock()
                .ok()
                .map(|sessions| sessions.contains_key(slot_id))
                .unwrap_or(false)
                || super::slot_runtime_session_in_flight(*slot_id, target);
            if already_present {
                continue;
            }

            match build_slot_runtime_session_on_main_thread(app, *slot_id, target) {
                Ok(session) => {
                    if let Ok(mut sessions) = slot_runtime_sessions().lock() {
                        sessions.insert(*slot_id, session);
                    }
                }
                Err(_error) => {}
            }
        }

        desired_targets
            .iter()
            .filter_map(|(slot_id, target)| {
                let session_present = slot_runtime_sessions()
                    .lock()
                    .ok()
                    .map(|sessions| sessions.contains_key(slot_id))
                    .unwrap_or(false)
                    || super::slot_runtime_session_in_flight(*slot_id, target);

                session_present.then_some((*slot_id, target.clone()))
            })
            .collect()
    }

    pub fn sync_active_plugin_host(_focused: Option<bool>, reason: super::PluginHostSyncReason) {
        // Steinberg editorhost pattern: windowDidResize → onResize → onSize
        // runs synchronously on the main thread. No async dispatch.
        // This is critical: dispatching via run_on_main_thread would cause
        // the resizing_from_plugin guard to be cleared by the time our
        // handler runs, leading to redundant onSize calls and resize ping-pong.

        // Skip if currently inside a plugin-requested resizeView call
        // to avoid recursion (Steinberg reentrancy guard pattern).
        let guard_active = resizing_from_plugin().load(Ordering::Acquire);
        if guard_active {
            return;
        }

        let Some((view, _host_view_ptr, plugin_can_resize)) = active_session_view_state() else {
            return;
        };

        match reason {
            super::PluginHostSyncReason::WindowResized => {
                if plugin_can_resize {
                    // Steinberg editorhost onResize pattern: get content rect
                    // size, create ViewRect, check getSize, if different call
                    // onSize.
                    let ns_window_ptr = super::native_host_window()
                        .lock()
                        .ok()
                        .and_then(|guard| guard.as_ref().map(|w| w.0));
                    let Some(ns_window_ptr) = ns_window_ptr else {
                        return;
                    };
                    // SAFETY: ptr is valid and we are on the main thread
                    // (sync_active_plugin_host runs synchronously from window events).
                    let ns_window: &NSWindow = unsafe { &*ns_window_ptr };
                    let Some(content_view) = ns_window.contentView() else {
                        return;
                    };
                    let content_frame = content_view.frame();
                    let mut new_rect = ViewRect {
                        left: 0,
                        top: 0,
                        right: content_frame.size.width as i32,
                        bottom: content_frame.size.height as i32,
                    };
                    let mut current_rect = ViewRect {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                    };
                    unsafe {
                        if view.getSize(&mut current_rect) == kResultOk
                            && (current_rect.left != new_rect.left
                                || current_rect.top != new_rect.top
                                || current_rect.right != new_rect.right
                                || current_rect.bottom != new_rect.bottom)
                        {
                            let _ = view.onSize(&mut new_rect);
                        }
                    }
                }
            }
        }
    }

    pub fn attach_native_host_surface(
        app: &AppHandle,
        target: &super::PluginEditorTarget,
    ) -> Result<PluginHostAttachOutcome, String> {
        clear_pending_plugin_resize_request();
        let (sender, receiver) = std::sync::mpsc::channel();
        let app_clone = app.clone();
        let target = target.clone();
        let plugin_path = target.path().to_path_buf();

        app_clone
            .run_on_main_thread(move || {
                let result = (|| -> Result<PluginHostAttachOutcome, String> {
                    let mtm = MainThreadMarker::new().ok_or_else(|| {
                        String::from("Plugin host attachment must run on the main thread")
                    })?;

                    // Create a plain native NSWindow (no WKWebView layer).
                    // This matches the Steinberg editorhost and MuseScore
                    // patterns. JUCE plugins own their own event dispatch
                    // through MessageManager — a WKWebView contentView
                    // intercepts mouse events and causes deadlocks.
                    let ns_window_ptr: *mut NSWindow = {
                        let mut guard = super::native_host_window()
                            .lock()
                            .map_err(|_| String::from("Failed to lock native host window"))?;
                        match *guard {
                            Some(ref existing) => existing.0,
                            None => {
                                let initial_frame =
                                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(640.0, 480.0));
                                let style = NSWindowStyleMask::Titled | NSWindowStyleMask::Closable;
                                let window = unsafe {
                                    NSWindow::initWithContentRect_styleMask_backing_defer(
                                        mtm.alloc(),
                                        initial_frame,
                                        style,
                                        NSBackingStoreType::Buffered,
                                        false,
                                    )
                                };
                                window.center();
                                let title = NSString::from_str("Plugin Editor");
                                window.setTitle(&title);
                                window.setAcceptsMouseMovedEvents(true);
                                unsafe { window.setReleasedWhenClosed(false) };
                                // Transfer ownership to the global via into_raw
                                // (bumps retain count). close_native_host_window
                                // reclaims via from_raw.
                                let ptr = Retained::into_raw(window);
                                guard.replace(super::NativeWindowPtr(ptr));

                                // Create and set the window delegate for close
                                // and resize handling.
                                let delegate = PluginHostWindowDelegate::new(mtm);
                                let ns_win: &NSWindow = unsafe { &*ptr };
                                ns_win.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
                                // Store the delegate in a global to keep it alive
                                // (NSWindow holds its delegate as a weak reference).
                                let delegate_ptr = Retained::into_raw(delegate);
                                if let Ok(mut dg) = window_delegate_global().lock() {
                                    dg.replace(NativeDelegatePtr(delegate_ptr));
                                }

                                ptr
                            }
                        }
                    };
                    // SAFETY: ptr is valid and we are on the main thread.
                    let ns_window: &NSWindow = unsafe { &*ns_window_ptr };
                    let content_view = ns_window
                        .contentView()
                        .ok_or_else(|| String::from("Failed to resolve host contentView"))?;

                    // Attach directly to contentView, matching the Steinberg
                    // editorhost sample. No intermediate container view layer.

                    let attach_result = if let Some(slot_id) = target.slot_id {
                        let session = slot_runtime_sessions()
                            .lock()
                            .map_err(|_| {
                                String::from("Failed to read slot runtime session registry")
                            })?
                            .remove(&slot_id);
                        let mut session = match session {
                            Some(session) => session,
                            None => {
                                build_plugin_host_session(&target, target.plugin_state.as_ref())
                                    .map_err(|error| error.message.clone())?
                            }
                        };
                        let mut telemetry = super::PluginHostBuildTelemetry::disabled();
                        let result =
                            session.attach_view(ns_window_ptr, &content_view, &mut telemetry);
                        slot_runtime_sessions()
                            .lock()
                            .map_err(|_| String::from("Failed to restore slot runtime session"))?
                            .insert(slot_id, session);
                        result
                    } else {
                        // attach_view can trigger synchronous window events (e.g.
                        // windowDidResize) that call active_session_view_state(),
                        // which also locks this mutex. Holding through attach would
                        // deadlock.
                        let mut session = {
                            let mut active_session =
                                active_plugin_host_session().lock().map_err(|_| {
                                    String::from(
                                        "Failed to read active inventory plugin host session",
                                    )
                                })?;
                            match active_session.take() {
                                Some(session) => session,
                                None => build_plugin_host_session(&target, None)
                                    .map_err(|error| error.message.clone())?,
                            }
                        };
                        let mut telemetry = super::PluginHostBuildTelemetry::disabled();
                        let result =
                            session.attach_view(ns_window_ptr, &content_view, &mut telemetry);
                        active_plugin_host_session()
                            .lock()
                            .map_err(|_| {
                                String::from(
                                    "Failed to restore active inventory plugin host session",
                                )
                            })?
                            .replace(session);
                        result
                    };

                    if let Err(error) = attach_result {
                        attach_error_message(
                            mtm,
                            &content_view,
                            content_view.frame(),
                            &plugin_path,
                            &error.message,
                        );
                        return Ok(PluginHostAttachOutcome {
                            attached: false,
                            native_shown: false,
                            validation: failed_editor_host_validation(error.phase, error.message),
                        });
                    }

                    super::mark_plugin_target_attached(&target);

                    // Steinberg editorhost pattern: show the window on the same
                    // main-thread pass as the attach so the plugin view becomes
                    // visible in one atomic event-loop cycle.
                    unsafe {
                        let _: () =
                            msg_send![ns_window, makeKeyAndOrderFront: std::ptr::null::<c_void>()];
                    }

                    Ok(PluginHostAttachOutcome {
                        attached: true,
                        native_shown: true,
                        validation: attached_editor_host_validation(),
                    })
                })();

                let _ = sender.send(result);
            })
            .map_err(|error| format!("Failed to schedule native host attachment: {error}"))?;

        let wait_started_at = log_main_thread_wait_start("attach_native_host_surface");
        let received = receiver
            .recv()
            .map_err(|error| format!("Failed to receive native host attachment result: {error}"));
        log_main_thread_wait_finish("attach_native_host_surface", wait_started_at);
        received?
    }

    pub fn probe_plugin_host_load_init(
        app: &AppHandle,
        target: &super::PluginEditorTarget,
        profile: super::PluginEditorPocProbeProfile,
    ) -> Result<crate::playback::plugin_inventory::PluginEditorHostValidation, String> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let app_clone = app.clone();
        let target = target.clone();
        let blocked_reason = super::bounded_editor_host_unsupported_reason(target.path());
        let probe_log = super::MaintainerProbeLog::resolve(app);
        probe_log.reset_for_probe(&target.to_string(), blocked_reason.as_deref(), profile);
        probe_log.append_note(&format!(
            "Using maintainer load/init probe profile={}",
            super::probe_profile_code(profile)
        ));
        probe_log.append_note("Scheduling the maintainer load/init probe on the main thread.");
        let probe_log_for_main = probe_log.clone();

        app_clone
            .run_on_main_thread(move || {
                probe_log_for_main.append_note(
                    "Maintainer load/init probe entered the main thread.",
                );
                let result = (|| -> Result<crate::playback::plugin_inventory::PluginEditorHostValidation, String> {
                    let _callback_scope = super::ActiveMaintainerProbeCallbackScope::install(
                        probe_log_for_main.clone(),
                    );
                    let mtm = MainThreadMarker::new().ok_or_else(|| {
                        String::from("Maintainer load/init probe must run on the main thread")
                    })?;
                    let mut telemetry = super::PluginHostBuildTelemetry::enabled_with_probe_log(
                        profile,
                        probe_log_for_main.clone(),
                    );

                    let mut session = match build_plugin_host_session_with_telemetry(
                        &target,
                        target.plugin_state.as_ref(),
                        profile,
                        &mut telemetry,
                    ) {
                        Ok(session) => session,
                        Err(_error) => {
                            let probe = telemetry.finish().ok_or_else(|| {
                                String::from(
                                    "Failed to finalize maintainer load/init probe after host build failure.",
                                )
                            })?;
                            return Ok(super::validation_from_load_init_probe(
                                blocked_reason.clone(),
                                probe,
                            ));
                        }
                    };

                    telemetry.log_note(
                        "Creating a temporary AppKit window for the maintainer probe attach pass.",
                    );

                    let probe_window = unsafe {
                        NSWindow::initWithContentRect_styleMask_backing_defer(
                            mtm.alloc(),
                            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(640.0, 480.0)),
                            NSWindowStyleMask::Titled,
                            NSBackingStoreType::Buffered,
                            false,
                        )
                    };
                    let title = NSString::from_str("Plugin Load Probe");
                    probe_window.setTitle(&title);

                    let Some(content_view) = probe_window.contentView() else {
                        telemetry.record_failed(
                            PluginEditorHostValidationPhase::ViewCreate,
                            crate::playback::plugin_inventory::PluginEditorHostLoadInitProbeStatus::BlockedByHostContract,
                            "Failed to resolve the temporary maintainer probe content view.",
                        );
                        session.shutdown();
                        let probe = telemetry.finish().ok_or_else(|| {
                            String::from(
                                "Failed to finalize maintainer load/init probe after temporary window setup failed.",
                            )
                        })?;
                        return Ok(super::validation_from_load_init_probe(
                            blocked_reason.clone(),
                            probe,
                        ));
                    };

                    let ns_window_ptr = (&*probe_window as *const NSWindow).cast_mut();
                    match session.attach_view(ns_window_ptr, &content_view, &mut telemetry) {
                        Ok(()) => telemetry.record_completed(
                            PluginEditorHostValidationPhase::Attached,
                            "Reached a bounded editor-view attach on the temporary maintainer probe surface.",
                        ),
                        Err(_error) => {}
                    }

                    session.shutdown();

                    let probe = telemetry.finish().ok_or_else(|| {
                        String::from(
                            "Failed to finalize maintainer load/init probe after the attach pass.",
                        )
                    })?;

                    let validation = super::validation_from_load_init_probe(
                        blocked_reason.clone(),
                        probe,
                    );
                    probe_log_for_main.append_note(&format!(
                        "Maintainer load/init probe completed with profile={} validation_status={} validation_phase={}",
                        super::probe_profile_code(profile),
                        super::editor_host_validation_status_code(validation.status),
                        super::editor_host_phase_code(validation.phase),
                    ));

                    Ok(validation)
                })();

                if let Err(error) = &result {
                    probe_log_for_main.append_note(&format!(
                        "Maintainer load/init probe returned an outer error: {}",
                        error
                    ));
                }

                let _ = sender.send(result);
            })
            .map_err(|error| {
                probe_log.append_note(&format!(
                    "Failed to schedule maintainer load/init probe: {}",
                    error
                ));
                format!("Failed to schedule maintainer load/init probe: {error}")
            })?;

        let wait_started_at = log_main_thread_wait_start("probe_plugin_host_load_init");
        let received = receiver.recv().map_err(|error| {
            probe_log.append_note(&format!(
                "Failed to receive maintainer load/init probe result: {}",
                error
            ));
            format!("Failed to receive maintainer load/init probe result: {error}")
        });
        log_main_thread_wait_finish("probe_plugin_host_load_init", wait_started_at);
        received?
    }
}

#[cfg(target_os = "macos")]
use macos_host::{
    attach_native_host_surface as attach_native_host_surface_impl,
    capture_slot_runtime_plugin_state as capture_slot_runtime_plugin_state_impl,
    probe_plugin_host_load_init as probe_plugin_host_load_init_impl,
    render_slot_runtime_block as render_slot_runtime_block_impl,
    stop_slot_runtime_render as stop_slot_runtime_render_impl,
    sync_slot_runtime_sessions as sync_slot_runtime_sessions_impl, PluginHostSession,
};

#[cfg(target_os = "macos")]
fn attach_native_host_surface(
    app: &AppHandle,
    target: &PluginEditorTarget,
) -> Result<PluginHostAttachOutcome, String> {
    attach_native_host_surface_impl(app, target)
}

#[cfg(target_os = "macos")]
fn probe_plugin_host_load_init(
    app: &AppHandle,
    target: &PluginEditorTarget,
    profile: PluginEditorPocProbeProfile,
) -> Result<PluginEditorHostValidation, String> {
    probe_plugin_host_load_init_impl(app, target, profile)
}

#[cfg(target_os = "macos")]
pub(crate) fn sync_slot_runtime_sessions(
    app: &AppHandle,
    desired_targets: &FxHashMap<UniqueId, ActivatedSlotTarget>,
) -> FxHashMap<UniqueId, ActivatedSlotTarget> {
    sync_slot_runtime_sessions_impl(app, desired_targets)
}

#[cfg(target_os = "macos")]
pub(crate) fn capture_slot_runtime_plugin_state(
    app: &AppHandle,
    slot_id: UniqueId,
) -> Result<SlotRuntimePluginStateCapture, String> {
    capture_slot_runtime_plugin_state_impl(app, slot_id)
}

#[cfg(target_os = "macos")]
pub(crate) fn render_slot_runtime_block(
    slot_id: UniqueId,
    sample_rate: u32,
    frames_per_block: usize,
    events: &[LiveRenderEvent],
    output: &mut [f32],
    output_channels: usize,
) -> SlotRuntimeRenderOutcome {
    render_slot_runtime_block_impl(
        slot_id,
        sample_rate,
        frames_per_block,
        events,
        output,
        output_channels,
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn stop_slot_runtime_render(slot_id: UniqueId) {
    stop_slot_runtime_render_impl(slot_id)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn sync_slot_runtime_sessions(
    _app: &AppHandle,
    desired_targets: &FxHashMap<UniqueId, ActivatedSlotTarget>,
) -> FxHashMap<UniqueId, ActivatedSlotTarget> {
    desired_targets.clone()
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn capture_slot_runtime_plugin_state(
    _app: &AppHandle,
    _slot_id: UniqueId,
) -> Result<SlotRuntimePluginStateCapture, String> {
    Ok(SlotRuntimePluginStateCapture::MissingSession)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn render_slot_runtime_block(
    _slot_id: UniqueId,
    _sample_rate: u32,
    _frames_per_block: usize,
    _events: &[LiveRenderEvent],
    _output: &mut [f32],
    _output_channels: usize,
) -> SlotRuntimeRenderOutcome {
    SlotRuntimeRenderOutcome::Unsupported
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn stop_slot_runtime_render(_slot_id: UniqueId) {}

fn sync_score_slot_targets_with_capture<F>(score: &mut Score, mut capture: F) -> Result<(), String>
where
    F: FnMut(UniqueId) -> Result<SlotRuntimePluginStateCapture, String>,
{
    for (slot_id, slot) in &mut score.playback.rack.by_id {
        let Some(target) = slot.target.as_mut() else {
            continue;
        };

        let capture_result = capture(*slot_id).map_err(|error| {
            format!(
                "Failed to capture slot runtime state for slot {}: {}",
                slot_id, error
            )
        })?;
        apply_slot_runtime_plugin_state_capture(target, capture_result);
    }

    Ok(())
}

pub(crate) fn sync_score_slot_targets_with_runtime_state(
    app: &AppHandle,
    score: &mut Score,
) -> Result<(), String> {
    sync_score_slot_targets_with_capture(score, |slot_id| {
        capture_slot_runtime_plugin_state(app, slot_id)
    })
}

#[cfg(not(target_os = "macos"))]
fn attach_native_host_surface(
    _app: &AppHandle,
    _target: &PluginEditorTarget,
) -> Result<PluginHostAttachOutcome, String> {
    Err(String::from("plugin_editor_poc_open is macOS-only"))
}

#[cfg(not(target_os = "macos"))]
fn probe_plugin_host_load_init(
    _app: &AppHandle,
    _target: &PluginEditorTarget,
    _profile: PluginEditorPocProbeProfile,
) -> Result<PluginEditorHostValidation, String> {
    Err(String::from(
        "plugin_editor_poc_probe_load_init is macOS-only",
    ))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_editor_poc_read_probe_log(
    app: AppHandle,
) -> Result<PluginEditorPocProbeLog, String> {
    MaintainerProbeLog::resolve(&app).snapshot()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_editor_poc_prepare_probe_log(
    app: AppHandle,
    app_state: State<'_, AppContextState>,
    plugin_inventory_state: State<'_, ManagedPluginInventoryContextState>,
    plugin_id: Option<String>,
    plugin_path: Option<String>,
    profile: Option<PluginEditorPocProbeProfile>,
) -> Result<PluginEditorPocProbeLog, String> {
    let target = resolve_plugin_target(
        None,
        plugin_id.as_deref(),
        plugin_path.as_deref(),
        app_state.inner(),
        plugin_inventory_state.inner(),
    )?;
    let blocked_reason = bounded_editor_host_unsupported_reason(target.path());
    let probe_log = MaintainerProbeLog::resolve(&app);
    let profile = profile.unwrap_or_default();

    probe_log.reset_for_probe(&target.to_string(), blocked_reason.as_deref(), profile);
    probe_log.append_note(
        &format!(
            "Prepared the maintainer load/init probe log before invoking the risky host probe with profile={}",
            probe_profile_code(profile)
        ),
    );

    probe_log.snapshot()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_editor_poc_open(
    app: AppHandle,
    app_state: State<'_, AppContextState>,
    plugin_inventory_state: State<'_, ManagedPluginInventoryContextState>,
    slot_id: Option<UniqueId>,
    plugin_id: Option<String>,
    plugin_path: Option<String>,
) -> Result<PluginEditorPocOpenResult, String> {
    let target = resolve_plugin_target(
        slot_id,
        plugin_id.as_deref(),
        plugin_path.as_deref(),
        app_state.inner(),
        plugin_inventory_state.inner(),
    )?;
    if let Err(reason) = ensure_editor_host_target_supported(target.path()) {
        queue_editor_host_validation_emit(
            app.clone(),
            target.path().to_path_buf(),
            blocked_editor_host_validation(reason.clone()),
        );
        return Err(reason);
    }

    let existing_window = native_host_window_exists();
    eprintln!(
        "[DIAG] plugin_editor_poc_open: existing_window={}",
        existing_window
    );
    release_detached_plugin_host_for_target_switch(&app, &target, existing_window)?;
    release_preserved_inventory_session_for_target_switch(&app, &target)?;
    let existing_window = native_host_window_exists();

    match reserve_plugin_target(&target, existing_window)? {
        PluginTargetReservation::Acquired => {}
        PluginTargetReservation::AlreadyReserved(phase) => {
            if existing_window {
                focus_native_host_window()?;

                return Ok(PluginEditorPocOpenResult {
                    window_label: WINDOW_LABEL.to_string(),
                    plugin_path: target.path().display().to_string(),
                    native_host_attached: live_plugin_host_attached(),
                });
            }

            if should_continue_reopening_reserved_target(phase, existing_window) {
            } else {
                return Err(String::from(
                    "The plugin editor POC window is already opening for this target.",
                ));
            }
        }
    }

    if native_host_window_exists() {
        focus_native_host_window()?;

        return Ok(PluginEditorPocOpenResult {
            window_label: WINDOW_LABEL.to_string(),
            plugin_path: target.path().display().to_string(),
            native_host_attached: live_plugin_host_attached(),
        });
    }

    let attach_outcome = match attach_native_host_surface(&app, &target) {
        Ok(outcome) => outcome,
        Err(error) => {
            release_reserved_plugin_target(&target);
            close_native_host_window();
            return Err(error);
        }
    };
    queue_editor_host_validation_emit(
        app.clone(),
        target.path().to_path_buf(),
        attach_outcome.validation,
    );

    if !attach_outcome.native_shown {
        // attach_native_host_surface shows via makeKeyAndOrderFront in the
        // normal success path. This branch covers error-overlay cases.
        let _ = focus_native_host_window();
    } else {
        eprintln!(
            "[DIAG] plugin_editor_poc_open: window already shown natively, skipping async show"
        );
    }

    // Diagnostic: verify main thread is responsive after open.
    let diag_app = app.clone();
    let _ = diag_app.run_on_main_thread(move || {});

    Ok(PluginEditorPocOpenResult {
        window_label: WINDOW_LABEL.to_string(),
        plugin_path: target.path().display().to_string(),
        native_host_attached: attach_outcome.attached,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_editor_poc_probe_load_init(
    app: AppHandle,
    app_state: State<'_, AppContextState>,
    plugin_inventory_state: State<'_, ManagedPluginInventoryContextState>,
    plugin_id: Option<String>,
    plugin_path: Option<String>,
    profile: Option<PluginEditorPocProbeProfile>,
) -> Result<PluginEditorHostValidation, String> {
    let target = resolve_plugin_target(
        None,
        plugin_id.as_deref(),
        plugin_path.as_deref(),
        app_state.inner(),
        plugin_inventory_state.inner(),
    )?;

    let validation = probe_plugin_host_load_init(&app, &target, profile.unwrap_or_default())?;
    emit_editor_host_validation(
        &app,
        plugin_inventory_state.inner(),
        target.path(),
        validation.clone(),
    );

    Ok(validation)
}

#[cfg(test)]
mod tests {
    use super::{
        active_plugin_host_lifecycle, clear_reserved_plugin_target,
        ensure_editor_host_target_supported, mark_plugin_target_attached,
        release_reserved_plugin_target, reserve_plugin_target, resolve_plugin_target_in_roots,
        PluginHostLifecyclePhase, PluginTargetReservation,
    };
    use crate::playback::plugin_host_compatibility::bounded_editor_host_unsupported_reason;
    use crate::playback::plugin_inventory::{
        PluginEditorHostLoadInitProbe, PluginEditorHostLoadInitProbeStatus,
        PluginEditorHostValidationPhase, PluginEditorHostValidationPhaseResultStatus,
        PluginEditorHostValidationStatus,
    };
    use crate::playback::runtime_activation::ActivatedSlotTarget;
    use crate::score::playback::{PlaybackSlotPluginState, PlaybackSlotTarget};
    use crate::utils::UniqueId;
    #[cfg(target_os = "macos")]
    use std::ffi::c_void;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    #[cfg(target_os = "macos")]
    fn write_class_info_field<const N: usize>(buffer: &mut [i8; N], value: &str) {
        buffer.fill(0);
        for (index, byte) in value
            .as_bytes()
            .iter()
            .copied()
            .take(N.saturating_sub(1))
            .enumerate()
        {
            buffer[index] = byte as i8;
        }
    }

    #[cfg(target_os = "macos")]
    fn component_class_candidate(
        name: &str,
        subcategories: &str,
    ) -> super::macos_host::ComponentClassCandidate {
        super::macos_host::ComponentClassCandidate {
            cid: [0; 16],
            name: name.to_string(),
            category: String::from("Audio Module Class"),
            subcategories: subcategories.to_string(),
        }
    }

    fn plugin_target_test_mutex() -> &'static Mutex<()> {
        static PLUGIN_TARGET_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        PLUGIN_TARGET_TEST_MUTEX.get_or_init(|| Mutex::new(()))
    }

    fn lock_plugin_target_test_mutex() -> std::sync::MutexGuard<'static, ()> {
        plugin_target_test_mutex()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn reset_active_plugin_target() {
        clear_reserved_plugin_target();
    }

    fn slot_target(path: &str, slot_id: UniqueId) -> super::PluginEditorTarget {
        super::PluginEditorTarget::slot_with_state(PathBuf::from(path), slot_id, None)
    }

    fn slot_target_with_state(
        path: &str,
        slot_id: UniqueId,
        plugin_state: Option<PlaybackSlotPluginState>,
    ) -> super::PluginEditorTarget {
        super::PluginEditorTarget::slot_with_state(PathBuf::from(path), slot_id, plugin_state)
    }

    fn inventory_target(path: &str) -> super::PluginEditorTarget {
        super::PluginEditorTarget::inventory(PathBuf::from(path))
    }

    fn active_plugin_target() -> Option<super::PluginEditorTarget> {
        active_plugin_host_lifecycle()
            .lock()
            .ok()
            .and_then(|state| state.target.clone())
    }

    fn active_plugin_phase() -> PluginHostLifecyclePhase {
        active_plugin_host_lifecycle()
            .lock()
            .map(|state| state.phase)
            .unwrap_or(PluginHostLifecyclePhase::Idle)
    }

    #[test]
    fn resolve_plugin_target_requires_explicit_env_or_discovered_bundle() {
        let previous = std::env::var(super::PLUGIN_PATH_ENV).ok();
        std::env::remove_var(super::PLUGIN_PATH_ENV);

        let error = resolve_plugin_target_in_roots(None, &[], None).unwrap_err();
        assert!(error.contains(super::PLUGIN_PATH_ENV));

        match previous {
            Some(value) => std::env::set_var(super::PLUGIN_PATH_ENV, value),
            None => std::env::remove_var(super::PLUGIN_PATH_ENV),
        }
    }

    #[test]
    fn resolve_plugin_target_prefers_inventory_target_before_discovery() {
        let inventory_target = PathBuf::from("/tmp/test-plugin.vst3");

        let resolved =
            resolve_plugin_target_in_roots(None, &[], Some(inventory_target.clone())).unwrap();

        assert_eq!(resolved, inventory_target);
    }

    #[test]
    fn reserve_plugin_target_marks_first_request_as_acquired() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = inventory_target("/tmp/test-plugin.vst3");
        let reservation = reserve_plugin_target(&target, false).unwrap();

        assert!(matches!(reservation, PluginTargetReservation::Acquired));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Reserved);

        reset_active_plugin_target();
    }

    #[test]
    fn reserve_plugin_target_reuses_matching_target() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, true).unwrap();

        let reservation = reserve_plugin_target(&target, true).unwrap();

        assert!(matches!(
            reservation,
            PluginTargetReservation::AlreadyReserved(PluginHostLifecyclePhase::Reserved)
        ));

        reset_active_plugin_target();
    }

    #[test]
    fn reserve_plugin_target_rejects_different_target_while_reserved() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        reserve_plugin_target(&slot_target("/tmp/test-plugin-a.vst3", 12), true).unwrap();

        let error =
            reserve_plugin_target(&slot_target("/tmp/test-plugin-b.vst3", 12), true).unwrap_err();

        assert!(error.contains("already open for a different target"));

        reset_active_plugin_target();
    }

    #[test]
    fn reserve_plugin_target_rejects_same_bundle_for_a_different_slot() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        reserve_plugin_target(&slot_target("/tmp/test-plugin.vst3", 12), true).unwrap();

        let error =
            reserve_plugin_target(&slot_target("/tmp/test-plugin.vst3", 13), true).unwrap_err();

        assert!(error.contains("already open for a different target"));

        reset_active_plugin_target();
    }

    #[test]
    fn detached_target_to_replace_requires_a_different_target_without_a_window() {
        let active_target = slot_target("/tmp/test-plugin.vst3", 12);
        let requested_target = slot_target("/tmp/test-plugin.vst3", 13);
        let detached_state = super::PluginHostLifecycleState {
            target: Some(active_target.clone()),
            phase: PluginHostLifecyclePhase::Detached,
        };

        assert_eq!(
            super::detached_target_to_replace(&requested_target, false, &detached_state),
            Some(active_target.clone())
        );
        assert_eq!(
            super::detached_target_to_replace(&active_target, false, &detached_state),
            None
        );
        assert_eq!(
            super::detached_target_to_replace(&requested_target, true, &detached_state),
            None
        );
        assert_eq!(
            super::detached_target_to_replace(
                &requested_target,
                false,
                &super::PluginHostLifecycleState {
                    target: Some(active_target),
                    phase: PluginHostLifecyclePhase::Attached,
                },
            ),
            None
        );
    }

    #[test]
    fn release_reserved_plugin_target_clears_matching_target_only() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let active_target = slot_target("/tmp/test-plugin-a.vst3", 12);
        reserve_plugin_target(&active_target, true).unwrap();
        release_reserved_plugin_target(&slot_target("/tmp/test-plugin-a.vst3", 13));

        assert_eq!(active_plugin_target(), Some(active_target.clone()));

        release_reserved_plugin_target(&active_target);

        assert_eq!(active_plugin_target(), None);
        reset_active_plugin_target();
    }

    #[test]
    fn unsupported_editor_host_target_reason_rejects_steinberg_vendor_paths() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/Groove Agent SE.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("current unsupported list"));
        assert!(reason.contains("Groove Agent SE"));
    }

    #[test]
    fn unsupported_editor_host_target_reason_rejects_known_steinberg_bundle_names() {
        let path = Path::new("/tmp/Groove Agent SE.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("Groove Agent SE"));
    }

    #[test]
    fn unsupported_editor_host_target_reason_rejects_kontakt_7_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Kontakt 7.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("Kontakt 7"));
    }

    #[test]
    fn unsupported_editor_host_target_reason_rejects_mt_power_drumkit_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/MT-PowerDrumKit.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("MT-PowerDrumKit"));
    }

    #[test]
    fn unsupported_editor_host_target_reason_rejects_halion_sonic_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("HALion Sonic"));
    }

    #[test]
    fn unsupported_editor_host_target_reason_rejects_halion_sonic_se_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic SE.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("HALion Sonic SE"));
    }

    #[test]
    fn unsupported_editor_host_target_reason_allows_non_unsupported_control_case() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/BBC Symphony Orchestra.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }

    #[test]
    fn ensure_editor_host_target_supported_rejects_without_touching_lifecycle_state() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let error = ensure_editor_host_target_supported(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Steinberg/Groove Agent SE.vst3",
        ))
        .unwrap_err();

        assert!(error.contains("current unsupported list"));
        assert!(error.contains("Groove Agent SE"));
        assert_eq!(active_plugin_target(), None);
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Idle);
    }

    #[test]
    fn ensure_editor_host_target_supported_rejects_halion_sonic_without_touching_lifecycle_state() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let error = ensure_editor_host_target_supported(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3",
        ))
        .unwrap_err();

        assert!(error.contains("HALion Sonic"));
        assert_eq!(active_plugin_target(), None);
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Idle);
    }

    #[test]
    fn ensure_editor_host_target_supported_rejects_halion_sonic_se_without_touching_lifecycle_state(
    ) {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let error = ensure_editor_host_target_supported(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic SE.vst3",
        ))
        .unwrap_err();

        assert!(error.contains("HALion Sonic SE"));
        assert_eq!(active_plugin_target(), None);
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Idle);
    }

    #[test]
    fn ensure_editor_host_target_supported_rejects_kontakt_7_without_touching_lifecycle_state() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let error = ensure_editor_host_target_supported(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Kontakt 7.vst3",
        ))
        .unwrap_err();

        assert!(error.contains("Kontakt 7"));
        assert_eq!(active_plugin_target(), None);
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Idle);
    }

    #[test]
    fn reserve_plugin_target_keeps_in_flight_reservation_without_window() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, false).unwrap();

        let reservation = reserve_plugin_target(&target, false).unwrap();

        assert!(matches!(
            reservation,
            PluginTargetReservation::AlreadyReserved(PluginHostLifecyclePhase::Reserved)
        ));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Reserved);

        reset_active_plugin_target();
    }

    #[test]
    fn preserved_inventory_target_only_reuses_matching_target() {
        let matching_target = inventory_target("/tmp/test-plugin.vst3");
        let different_inventory_target = inventory_target("/tmp/other-plugin.vst3");
        let slot_target = slot_target("/tmp/test-plugin.vst3", 12);

        assert_eq!(
            super::preserved_inventory_target_to_replace(&matching_target, Some(&matching_target),),
            None
        );
        assert_eq!(
            super::preserved_inventory_target_to_replace(
                &matching_target,
                Some(&different_inventory_target),
            ),
            Some(different_inventory_target)
        );
        assert_eq!(
            super::preserved_inventory_target_to_replace(&matching_target, Some(&slot_target)),
            None
        );
    }

    #[test]
    fn mark_plugin_target_attached_promotes_reserved_target() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, false).unwrap();

        mark_plugin_target_attached(&target);

        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Attached);

        reset_active_plugin_target();
    }

    #[test]
    fn slot_runtime_session_in_flight_matches_reserved_slot_target() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, false).unwrap();

        assert!(super::slot_runtime_session_in_flight(
            12,
            &ActivatedSlotTarget {
                plugin_id: String::from("plugin-a"),
                bundle_path: PathBuf::from("/tmp/test-plugin.vst3"),
                plugin_state: None,
            },
        ));
        assert!(!super::slot_runtime_session_in_flight(
            12,
            &ActivatedSlotTarget {
                plugin_id: String::from("plugin-a"),
                bundle_path: PathBuf::from("/tmp/other-plugin.vst3"),
                plugin_state: None,
            },
        ));

        reset_active_plugin_target();
    }

    #[test]
    fn slot_runtime_session_in_flight_rejects_restorable_state_mismatch() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target_with_state(
            "/tmp/test-plugin.vst3",
            12,
            Some(PlaybackSlotPluginState {
                vst3_component_state_base64: String::from("AQID"),
            }),
        );
        reserve_plugin_target(&target, false).unwrap();

        assert!(!super::slot_runtime_session_in_flight(
            12,
            &ActivatedSlotTarget {
                plugin_id: String::from("plugin-a"),
                bundle_path: PathBuf::from("/tmp/test-plugin.vst3"),
                plugin_state: Some(PlaybackSlotPluginState {
                    vst3_component_state_base64: String::from("BAUG"),
                }),
            },
        ));

        reset_active_plugin_target();
    }

    #[test]
    fn slot_runtime_session_target_match_rejects_restorable_state_mismatch() {
        let desired_target = ActivatedSlotTarget {
            plugin_id: String::from("plugin-a"),
            bundle_path: PathBuf::from("/tmp/test-plugin.vst3"),
            plugin_state: Some(PlaybackSlotPluginState {
                vst3_component_state_base64: String::from("AQID"),
            }),
        };

        let matches_same_state =
            super::macos_host::slot_runtime_session_matches_target_path_and_state(
                Path::new("/tmp/test-plugin.vst3"),
                desired_target.plugin_state.as_ref(),
                &desired_target,
            );
        let mismatches_saved_state =
            super::macos_host::slot_runtime_session_matches_target_path_and_state(
                Path::new("/tmp/test-plugin.vst3"),
                Some(&PlaybackSlotPluginState {
                    vst3_component_state_base64: String::from("BAUG"),
                }),
                &desired_target,
            );

        assert!(matches_same_state);
        assert!(!mismatches_saved_state);
    }

    #[test]
    fn sync_score_slot_targets_preserves_missing_session_state_and_clears_non_transferable_state() {
        let mut score = crate::score::Score::new();
        score.playback.rack.order.push(100);
        score.playback.rack.by_id.insert(
            100,
            crate::score::playback::PlaybackRackSlot {
                id: 100,
                target: Some(PlaybackSlotTarget {
                    plugin_id: String::from("plugin-a"),
                    plugin_state: Some(PlaybackSlotPluginState {
                        vst3_component_state_base64: String::from("AQID"),
                    }),
                }),
            },
        );
        score.playback.rack.order.push(200);
        score.playback.rack.by_id.insert(
            200,
            crate::score::playback::PlaybackRackSlot {
                id: 200,
                target: Some(PlaybackSlotTarget {
                    plugin_id: String::from("plugin-b"),
                    plugin_state: Some(PlaybackSlotPluginState {
                        vst3_component_state_base64: String::from("BAUG"),
                    }),
                }),
            },
        );

        super::sync_score_slot_targets_with_capture(&mut score, |slot_id| match slot_id {
            100 => Ok(super::SlotRuntimePluginStateCapture::MissingSession),
            200 => Ok(super::SlotRuntimePluginStateCapture::NoTransferableState),
            _ => unreachable!(),
        })
        .unwrap();

        assert_eq!(
            score.playback.rack.by_id[&100]
                .target
                .as_ref()
                .and_then(|target| target.plugin_state.as_ref()),
            Some(&PlaybackSlotPluginState {
                vst3_component_state_base64: String::from("AQID"),
            })
        );
        assert_eq!(
            score.playback.rack.by_id[&200]
                .target
                .as_ref()
                .and_then(|target| target.plugin_state.as_ref()),
            None
        );
    }

    #[test]
    fn sync_score_slot_targets_returns_capture_errors() {
        let mut score = crate::score::Score::new();
        score.playback.rack.order.push(100);
        score.playback.rack.by_id.insert(
            100,
            crate::score::playback::PlaybackRackSlot {
                id: 100,
                target: Some(PlaybackSlotTarget {
                    plugin_id: String::from("plugin-a"),
                    plugin_state: None,
                }),
            },
        );

        let error = super::sync_score_slot_targets_with_capture(&mut score, |_| {
            Err(String::from("capture failed"))
        })
        .unwrap_err();

        assert!(error.contains("slot 100"));
        assert!(error.contains("capture failed"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn class_info_audio_module_filter_only_accepts_component_category() {
        let mut component_info = unsafe { std::mem::zeroed::<vst3::Steinberg::PClassInfo>() };
        write_class_info_field(&mut component_info.category, "Audio Module Class");

        let mut controller_info = unsafe { std::mem::zeroed::<vst3::Steinberg::PClassInfo>() };
        write_class_info_field(&mut controller_info.category, "Component Controller Class");

        assert!(super::macos_host::class_info_is_audio_module(
            &component_info
        ));
        assert!(!super::macos_host::class_info_is_audio_module(
            &controller_info
        ));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn component_class_selector_prefers_exact_bundle_name_match() {
        let candidates = vec![
            component_class_candidate("HALion Sonic SE", "Instrument"),
            component_class_candidate("HALion Sonic", "Instrument"),
        ];

        let selected =
            super::macos_host::select_component_class_candidate(&candidates, Some("HALion Sonic"))
                .unwrap();

        assert_eq!(selected.name, "HALion Sonic");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn component_class_selector_falls_back_to_first_audio_module_without_name_match() {
        let candidates = vec![
            component_class_candidate("Kontakt 7", "Instrument"),
            component_class_candidate("Kontakt 7 FX", "Fx"),
        ];

        let selected = super::macos_host::select_component_class_candidate(
            &candidates,
            Some("Some Other Plugin"),
        )
        .unwrap();

        assert_eq!(selected.name, "Kontakt 7");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_create_instance_returns_message_for_message_iid() {
        use vst3::Steinberg::kResultOk;
        use vst3::Steinberg::Vst::{IHostApplication, IHostApplicationTrait, IMessage_iid};
        use vst3::{ComPtr, ComWrapper};

        let host_application = ComWrapper::new(super::macos_host::PluginHostApplication)
            .to_com_ptr::<IHostApplication>()
            .unwrap();

        let mut raw_message: *mut c_void = std::ptr::null_mut();
        let mut message_cid = IMessage_iid;
        let mut message_iid = IMessage_iid;
        let message_result = unsafe {
            host_application.createInstance(&mut message_cid, &mut message_iid, &mut raw_message)
        };
        assert_eq!(message_result, kResultOk);
        assert!(!raw_message.is_null());

        let _keep_alive: ComPtr<IHostApplication> = host_application;
    }

    #[test]
    fn host_callback_iid_label_names_additional_standard_interfaces() {
        use vst3::Steinberg::Vst::{IConnectionPoint_iid, IUnitInfo_iid};

        assert_eq!(
            super::host_callback_iid_label(&IConnectionPoint_iid),
            "IConnectionPoint"
        );
        assert_eq!(super::host_callback_iid_label(&IUnitInfo_iid), "IUnitInfo");
    }

    #[test]
    fn host_callback_iid_label_names_wrapper_probe_interfaces() {
        use vst3::Steinberg::Vst::{
            IVst3ToAAXWrapper_iid, IVst3ToAUWrapper_iid, IVst3ToVst2Wrapper_iid,
        };

        assert_eq!(
            super::host_callback_iid_label(&IVst3ToVst2Wrapper_iid),
            "IVst3ToVst2Wrapper"
        );
        assert_eq!(
            super::host_callback_iid_label(&IVst3ToAAXWrapper_iid),
            "IVst3ToAAXWrapper"
        );
        assert_eq!(
            super::host_callback_iid_label(&IVst3ToAUWrapper_iid),
            "IVst3ToAUWrapper"
        );
    }

    #[test]
    fn host_callback_iid_label_falls_back_to_raw_hex_for_unknown_interfaces() {
        use vst3::Steinberg::TUID;

        let unknown_iid: TUID = [
            0x10_i8, 0x20_i8, 0x30_i8, 0x40_i8, 0x50_i8, 0x60_i8, 0x70_i8, 0x11_i8, 0x22_i8,
            0x33_i8, 0x44_i8, 0x55_i8, 0x66_i8, 0x77_i8, 0x7F_i8, 0x01_i8,
        ];

        assert_eq!(
            super::host_callback_iid_label(&unknown_iid),
            "unknown_interface(10203040-50607011-22334455-66777F01)"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_exposes_full_interface_set() {
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Linux::IRunLoop;
        use vst3::Steinberg::Vst::{IComponentHandler, IHostApplication, IPlugInterfaceSupport};
        use vst3::{ComPtr, ComWrapper};

        let host_application = ComWrapper::new(super::macos_host::PluginHostApplication)
            .to_com_ptr::<IHostApplication>()
            .unwrap();
        let unknown = host_application.clone().upcast::<FUnknown>();

        assert!(unknown.cast::<IHostApplication>().is_some());
        assert!(unknown.cast::<IRunLoop>().is_some());
        assert!(unknown.cast::<IComponentHandler>().is_some());
        assert!(unknown.cast::<IPlugInterfaceSupport>().is_some());

        let _keep_alive: ComPtr<IHostApplication> = host_application;
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_exposes_plug_interface_support() {
        use vst3::ComWrapper;
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Vst::{IHostApplication, IPlugInterfaceSupport};

        let host_application = ComWrapper::new(super::macos_host::PluginHostApplication)
            .to_com_ptr::<IHostApplication>()
            .unwrap();
        let unknown = host_application.upcast::<FUnknown>();

        assert!(unknown.cast::<IPlugInterfaceSupport>().is_some());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_only_init_host_application_exposes_only_host_application() {
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Linux::IRunLoop;
        use vst3::Steinberg::Vst::{
            IAutomationState, IComponentHandler, IHostApplication, IPlugInterfaceSupport,
        };

        let host_application = super::macos_host::create_controller_init_host_application(
            super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned,
        )
        .unwrap();
        let unknown = host_application.clone().upcast::<FUnknown>();

        assert!(unknown.cast::<IHostApplication>().is_some());
        assert!(unknown.cast::<IPlugInterfaceSupport>().is_none());
        assert!(unknown.cast::<IComponentHandler>().is_none());
        assert!(unknown.cast::<IRunLoop>().is_none());
        assert!(unknown.cast::<IAutomationState>().is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_only_init_host_application_returns_non_null_fallback_for_wrapper_probes() {
        use std::ffi::c_void;
        use vst3::Steinberg::Vst::{IHostApplication, IVst3ToAUWrapper_iid};
        use vst3::Steinberg::{kNoInterface, FUnknown};

        let host_application = super::macos_host::create_controller_init_host_application(
            super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned,
        )
        .unwrap();
        let unknown = host_application.clone().upcast::<FUnknown>();
        let mut wrapper_iid = IVst3ToAUWrapper_iid;
        let mut raw_interface: *mut c_void = std::ptr::null_mut();

        let result = unsafe {
            ((*(*unknown.as_ptr()).vtbl).queryInterface)(
                unknown.as_ptr(),
                &mut wrapper_iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kNoInterface);
        assert!(!raw_interface.is_null());
        assert_ne!(raw_interface, host_application.as_ptr().cast::<c_void>());

        let fallback_unknown = raw_interface.cast::<FUnknown>();
        let remaining = unsafe { ((*(*fallback_unknown).vtbl).release)(fallback_unknown) };

        assert_eq!(remaining, 2);

        let _keep_alive: vst3::ComPtr<IHostApplication> = host_application;
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn controller_initialize_always_passes_host_context() {
        use vst3::Steinberg::{kNotImplemented, kResultOk};

        assert!(super::macos_host::should_pass_host_context_to_controller_initialize(None));
        assert!(
            super::macos_host::should_pass_host_context_to_controller_initialize(Some(kResultOk,))
        );
        assert!(
            super::macos_host::should_pass_host_context_to_controller_initialize(Some(
                kNotImplemented,
            ))
        );
        assert!(super::macos_host::should_pass_host_context_to_controller_initialize(Some(3)));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn resizing_from_plugin_guard_prevents_sync_during_resize() {
        use std::sync::atomic::Ordering;

        // Ensure the guard starts clear.
        super::macos_host::clear_pending_plugin_resize_request();
        assert!(
            !super::macos_host::resizing_from_plugin().load(Ordering::Acquire),
            "Guard should start clear"
        );

        // Simulate a plugin resize in flight.
        super::macos_host::resizing_from_plugin().store(true, Ordering::Release);
        assert!(
            super::macos_host::resizing_from_plugin().load(Ordering::Acquire),
            "Guard should be set during plugin resize"
        );

        // clear_pending_plugin_resize_request resets the guard.
        super::macos_host::clear_pending_plugin_resize_request();
        assert!(
            !super::macos_host::resizing_from_plugin().load(Ordering::Acquire),
            "Guard should be clear after reset"
        );
    }

    #[test]
    fn load_init_probe_marks_remaining_phases_skipped_after_first_failure() {
        let mut telemetry = super::PluginHostBuildTelemetry::enabled();

        telemetry.record_completed(
            PluginEditorHostValidationPhase::ModuleLoad,
            "Loaded the VST3 module binary.",
        );
        telemetry.record_completed(
            PluginEditorHostValidationPhase::FactoryCreate,
            "Resolved the VST3 plugin factory.",
        );
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ControllerCreate,
            "Created the VST3 edit controller.",
        );
        telemetry.record_failed(
            PluginEditorHostValidationPhase::ControllerInitialize,
            PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure,
            "controller init failed",
        );

        let probe = telemetry.finish().unwrap();

        assert_eq!(
            probe.status,
            PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure
        );
        assert_eq!(
            probe.last_successful_phase,
            Some(PluginEditorHostValidationPhase::ControllerCreate)
        );
        assert_eq!(
            probe.first_failing_phase,
            Some(PluginEditorHostValidationPhase::ControllerInitialize)
        );
        assert_eq!(
            probe.first_failing_message.as_deref(),
            Some("controller init failed")
        );
        assert_eq!(
            probe
                .phase_results
                .iter()
                .find(|result| result.phase == PluginEditorHostValidationPhase::ViewAttach)
                .map(|result| result.status),
            Some(PluginEditorHostValidationPhaseResultStatus::Skipped)
        );
    }

    #[test]
    fn blocked_validation_keeps_policy_gate_status_while_preserving_probe_payload() {
        let probe = PluginEditorHostLoadInitProbe {
            status: PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure,
            last_successful_phase: Some(PluginEditorHostValidationPhase::ControllerCreate),
            first_failing_phase: Some(PluginEditorHostValidationPhase::ControllerInitialize),
            first_failing_message: Some(String::from("controller init failed")),
            summary: String::from(
                "Maintainer load/init probe first failed at controller_initialize and appears plugin-side: controller init failed",
            ),
            phase_results: vec![],
        };

        let validation = super::validation_from_load_init_probe(
            Some(String::from("Blocked for ordinary user-facing flows.")),
            probe.clone(),
        );

        assert_eq!(validation.status, PluginEditorHostValidationStatus::Blocked);
        assert_eq!(
            validation.phase,
            PluginEditorHostValidationPhase::PolicyGate
        );
        assert!(validation.message.contains("controller_initialize"));
        assert_eq!(validation.maintainer_load_init_probe, Some(probe));
    }

    #[test]
    fn maintainer_probe_log_persists_phase_events() {
        let root = std::env::temp_dir().join(format!(
            "plugin-editor-poc-log-test-{}-{}",
            std::process::id(),
            super::current_timestamp_ms()
        ));
        let log = super::MaintainerProbeLog::from_path(root.join("probe.log"));

        log.try_reset_for_probe(
            "path=/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            Some("Blocked for ordinary user-facing flows."),
            super::PluginEditorPocProbeProfile::Default,
        )
        .unwrap();
        log.try_append_event(
            "starting",
            Some(PluginEditorHostValidationPhase::ModuleLoad),
            "About to load the VST3 module binary.",
        )
        .unwrap();
        log.try_append_event(
            "failed",
            Some(PluginEditorHostValidationPhase::ControllerInitialize),
            "controller init failed",
        )
        .unwrap();

        let contents = log.read_contents().unwrap().unwrap();

        assert!(contents.contains("=== maintainer load/init probe ==="));
        assert!(contents.contains("target=path=/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3"));
        assert!(
            contents.contains("ordinary_blocked_reason=Blocked for ordinary user-facing flows.")
        );
        assert!(contents.contains("probe_profile=default"));
        assert!(contents.contains("phase=module_load status=starting"));
        assert!(contents
            .contains("phase=controller_initialize status=failed message=controller init failed"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn maintainer_probe_log_snapshot_derives_likely_crash_hint_from_unfinished_phase() {
        let root = std::env::temp_dir().join(format!(
            "plugin-editor-poc-log-snapshot-test-{}-{}",
            std::process::id(),
            super::current_timestamp_ms()
        ));
        let log = super::MaintainerProbeLog::from_path(root.join("probe.log"));

        log.try_reset_for_probe(
            "path=/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            Some("Blocked for ordinary user-facing flows."),
            super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned,
        )
        .unwrap();
        log.try_append_event(
            "starting",
            Some(PluginEditorHostValidationPhase::ControllerInitialize),
            "About to initialize the VST3 edit controller.",
        )
        .unwrap();

        let snapshot = log.snapshot().unwrap();
        assert_eq!(
            snapshot.profile,
            Some(super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned)
        );
        let crash_hint = snapshot.crash_hint.unwrap();

        assert_eq!(crash_hint.phase, "controller_initialize");
        assert!(crash_hint
            .last_line
            .contains("phase=controller_initialize status=starting"));
        assert_eq!(
            crash_hint.message,
            "Likely hard crash during controller_initialize. The last recorded probe event started but never completed or failed."
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn maintainer_probe_log_snapshot_ignores_info_lines_after_started_phase() {
        let root = std::env::temp_dir().join(format!(
            "plugin-editor-poc-log-info-after-start-test-{}-{}",
            std::process::id(),
            super::current_timestamp_ms()
        ));
        let log = super::MaintainerProbeLog::from_path(root.join("probe.log"));

        log.try_reset_for_probe(
            "path=/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            Some("Blocked for ordinary user-facing flows."),
            super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned,
        )
        .unwrap();
        log.try_append_event(
            "starting",
            Some(PluginEditorHostValidationPhase::ControllerInitialize),
            "About to initialize the VST3 edit controller.",
        )
        .unwrap();
        log.try_append_event(
            "info",
            None,
            "Host callback during controller_initialize: Init host context queryInterface requested IVst3ToAUWrapper => unsupported_with_fallback",
        )
        .unwrap();

        let snapshot = log.snapshot().unwrap();
        let crash_hint = snapshot.crash_hint.unwrap();

        assert_eq!(crash_hint.phase, "controller_initialize");
        assert!(crash_hint
            .last_line
            .contains("phase=controller_initialize status=starting"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn maintainer_probe_log_snapshot_parses_host_only_lifecycle_aligned_profile() {
        let root = std::env::temp_dir().join(format!(
            "plugin-editor-poc-log-profile-test-{}-{}",
            std::process::id(),
            super::current_timestamp_ms()
        ));
        let log = super::MaintainerProbeLog::from_path(root.join("probe.log"));

        log.try_reset_for_probe(
            "path=/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            Some("Blocked for ordinary user-facing flows."),
            super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned,
        )
        .unwrap();

        let snapshot = log.snapshot().unwrap();

        assert_eq!(
            snapshot.profile,
            Some(super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned)
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn maintainer_probe_log_snapshot_omits_crash_hint_after_terminal_phase_event() {
        let root = std::env::temp_dir().join(format!(
            "plugin-editor-poc-log-terminal-test-{}-{}",
            std::process::id(),
            super::current_timestamp_ms()
        ));
        let log = super::MaintainerProbeLog::from_path(root.join("probe.log"));

        log.try_reset_for_probe(
            "path=/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            Some("Blocked for ordinary user-facing flows."),
            super::PluginEditorPocProbeProfile::Default,
        )
        .unwrap();
        log.try_append_event(
            "failed",
            Some(PluginEditorHostValidationPhase::ControllerInitialize),
            "controller init failed",
        )
        .unwrap();

        let snapshot = log.snapshot().unwrap();

        assert_eq!(
            snapshot.profile,
            Some(super::PluginEditorPocProbeProfile::Default)
        );
        assert_eq!(snapshot.crash_hint, None);

        fs::remove_dir_all(&root).unwrap();
    }
}
