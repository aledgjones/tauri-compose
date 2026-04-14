#![allow(non_snake_case)]

use crate::playback::engine::{set_slot_editor_render_active, LiveRenderEvent};
use crate::playback::plugin_host_compatibility::bounded_editor_host_unsupported_reason;
use crate::playback::plugin_inventory::{
    blocked_editor_host_validation, PluginCapabilityEvidence, PluginCapabilityEvidenceStatus,
    PluginEditorHostLoadInitProbeStatus, PluginEditorHostValidation,
    PluginEditorHostValidationPhase, PluginEditorHostValidationStatus,
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
use std::ffi::CStr;
use std::ffi::{c_char, c_void};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager, State};

const WINDOW_LABEL: &str = "plugin-host-poc";
const PLUGIN_PATH_ENV: &str = "SOLOC_PLUGIN_EDITOR_POC_VST3_PATH";

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginEditorPocProbeProfile {
    #[default]
    Default,
    ControllerInitHostOnlyLifecycleAligned,
}

fn normalize_terminal_log_message(message: &str) -> String {
    message.replace("\r\n", "\\n").replace('\n', "\\n")
}

#[derive(Clone, Debug)]
struct ActivePluginHostPhaseContext {
    origin: &'static str,
    target: String,
    phase: PluginEditorHostValidationPhase,
}

thread_local! {
    static ACTIVE_PLUGIN_HOST_PHASE_CONTEXT: RefCell<Option<ActivePluginHostPhaseContext>> = const { RefCell::new(None) };
}

fn set_active_plugin_host_phase_context(
    origin: &'static str,
    target: &str,
    phase: Option<PluginEditorHostValidationPhase>,
) {
    ACTIVE_PLUGIN_HOST_PHASE_CONTEXT.with(|active_phase| {
        *active_phase.borrow_mut() = phase.map(|phase| ActivePluginHostPhaseContext {
            origin,
            target: target.to_string(),
            phase,
        });
    });
}

fn emit_plugin_host_terminal_event(
    origin: &str,
    target: &str,
    phase: Option<PluginEditorHostValidationPhase>,
    status: &str,
    message: &str,
) {
    eprintln!(
        "[PLUGIN_HOST][origin={}][target={}][phase={}][status={}] {}",
        origin,
        normalize_terminal_log_message(target),
        phase.map(editor_host_phase_code).unwrap_or("general"),
        status,
        normalize_terminal_log_message(message),
    );
}

fn emit_plugin_host_terminal_target_event(
    origin: &str,
    target: &PluginEditorTarget,
    phase: Option<PluginEditorHostValidationPhase>,
    status: &str,
    message: &str,
) {
    emit_plugin_host_terminal_event(origin, &target.to_string(), phase, status, message);
}

fn log_active_host_phase_callback(message: &str) {
    let active_phase =
        ACTIVE_PLUGIN_HOST_PHASE_CONTEXT.with(|active_phase| active_phase.borrow().clone());

    if let Some(active_phase) = active_phase {
        emit_plugin_host_terminal_event(
            active_phase.origin,
            &active_phase.target,
            Some(active_phase.phase),
            "callback",
            message,
        );
    }
}

#[cfg(target_os = "macos")]
fn host_callback_object_output_label(pointer: *mut c_void) -> String {
    if pointer.is_null() {
        String::from("null")
    } else {
        format!("0x{:X}", pointer as usize)
    }
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "C" fn solo_compose_plugin_host_log_query_interface(
    iid_label: *const c_char,
    result: i32,
    returned_object: *mut c_void,
) {
    let iid_label = host_callback_c_string_label(iid_label);

    log_active_host_phase_callback(&format!(
        "IHostApplication::queryInterface requested {} => {} tresult={} returned_object={}",
        iid_label,
        if result == vst3::Steinberg::kResultOk || result == vst3::Steinberg::kResultTrue {
            "supported"
        } else {
            "unsupported"
        },
        result,
        host_callback_object_output_label(returned_object),
    ));
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "C" fn solo_compose_plugin_host_log_get_name_invoked() {
    log_active_host_phase_callback("IHostApplication::getName invoked");
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "C" fn solo_compose_plugin_host_log_get_name_result(result: i32) {
    log_active_host_phase_callback(&format!(
        "IHostApplication::getName returned tresult={}",
        result
    ));
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "C" fn solo_compose_plugin_host_log_is_plug_interface_supported(
    iid_label: *const c_char,
    result: i32,
) {
    let iid_label = host_callback_c_string_label(iid_label);

    log_active_host_phase_callback(&format!(
        "IPlugInterfaceSupport::isPlugInterfaceSupported requested {} => {} tresult={}",
        iid_label,
        if result == vst3::Steinberg::kResultOk || result == vst3::Steinberg::kResultTrue {
            "supported"
        } else {
            "unsupported"
        },
        result,
    ));
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "C" fn solo_compose_plugin_host_log_create_instance_requested(
    cid_label: *const c_char,
    iid_label: *const c_char,
) {
    let cid_label = host_callback_c_string_label(cid_label);
    let iid_label = host_callback_c_string_label(iid_label);

    log_active_host_phase_callback(&format!(
        "IHostApplication::createInstance requested cid={} iid={}",
        cid_label, iid_label,
    ));
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "C" fn solo_compose_plugin_host_log_create_instance_result(
    cid_label: *const c_char,
    iid_label: *const c_char,
    result: i32,
    returned_object: *mut c_void,
) {
    let cid_label = host_callback_c_string_label(cid_label);
    let iid_label = host_callback_c_string_label(iid_label);

    log_active_host_phase_callback(&format!(
        "IHostApplication::createInstance completed cid={} iid={} tresult={} returned_object={}",
        cid_label,
        iid_label,
        result,
        host_callback_object_output_label(returned_object),
    ));
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

fn restart_component_flags_label(flags: i32) -> String {
    let known_flags = [
        (
            vst3::Steinberg::Vst::RestartFlags_::kReloadComponent,
            "kReloadComponent",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kIoChanged,
            "kIoChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kParamValuesChanged,
            "kParamValuesChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kLatencyChanged,
            "kLatencyChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kParamTitlesChanged,
            "kParamTitlesChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kMidiCCAssignmentChanged,
            "kMidiCCAssignmentChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kNoteExpressionChanged,
            "kNoteExpressionChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kIoTitlesChanged,
            "kIoTitlesChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kPrefetchableSupportChanged,
            "kPrefetchableSupportChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kRoutingInfoChanged,
            "kRoutingInfoChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kKeyswitchChanged,
            "kKeyswitchChanged",
        ),
        (
            vst3::Steinberg::Vst::RestartFlags_::kParamIDMappingChanged,
            "kParamIDMappingChanged",
        ),
    ];
    let mut active_flags = Vec::new();
    let mut known_mask = 0;

    for (flag, label) in known_flags {
        known_mask |= flag;
        if flags & flag != 0 {
            active_flags.push(label.to_string());
        }
    }

    let unknown_bits = flags & !known_mask;
    if unknown_bits != 0 {
        active_flags.push(format!("unknown_bits=0x{unknown_bits:X}"));
    }

    if active_flags.is_empty() {
        String::from("none")
    } else {
        active_flags.join("|")
    }
}

fn restart_component_result(flags: i32) -> vst3::Steinberg::tresult {
    let _ = flags;
    vst3::Steinberg::kResultOk
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

#[cfg_attr(not(test), allow(dead_code))]
const MIN_PROBABLE_HOST_CALLBACK_POINTER_ADDRESS: usize = 0x1_0000_0000;

#[cfg_attr(not(test), allow(dead_code))]
fn is_probable_host_callback_pointer<T>(pointer: *const T) -> bool {
    let pointer_address = pointer as usize;
    !pointer.is_null()
        && pointer_address >= MIN_PROBABLE_HOST_CALLBACK_POINTER_ADDRESS
        && pointer_address.is_multiple_of(std::mem::align_of::<T>())
}

fn host_callback_pointer_label<T>(pointer: *const T) -> String {
    if pointer.is_null() {
        String::from("null")
    } else {
        format!("invalid_pointer(0x{:X})", pointer as usize)
    }
}

#[cfg(target_os = "macos")]
fn host_callback_c_string_label(label: *const c_char) -> String {
    if label.is_null() {
        String::from("null")
    } else {
        unsafe { CStr::from_ptr(label) }
            .to_string_lossy()
            .into_owned()
    }
}

#[cfg_attr(not(test), allow(dead_code))]
unsafe fn copy_host_callback_tuid(
    iid: *const vst3::Steinberg::TUID,
) -> Option<vst3::Steinberg::TUID> {
    if !is_probable_host_callback_pointer(iid) {
        return None;
    }

    Some(*iid)
}

fn host_callback_iid_label(iid: &vst3::Steinberg::TUID) -> String {
    host_callback_iid_name(iid)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("unknown_interface({})", host_callback_iid_hex(iid)))
}

#[cfg_attr(not(test), allow(dead_code))]
fn host_callback_iid_name_from_ptr(iid: *const vst3::Steinberg::TUID) -> String {
    unsafe {
        copy_host_callback_tuid(iid)
            .map(|value| host_callback_iid_label(&value))
            .unwrap_or_else(|| host_callback_pointer_label(iid))
    }
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

#[derive(Clone, Debug)]
struct PluginHostPhaseLogger {
    origin: &'static str,
    target: String,
}

impl PluginHostPhaseLogger {
    fn terminal(origin: &'static str, target: &PluginEditorTarget) -> Self {
        Self {
            origin,
            target: target.to_string(),
        }
    }

    fn log_phase_event(&self, phase: PluginEditorHostValidationPhase, status: &str, message: &str) {
        if status == "starting" {
            set_active_plugin_host_phase_context(self.origin, &self.target, Some(phase));
        }

        emit_plugin_host_terminal_event(self.origin, &self.target, Some(phase), status, message);

        if matches!(status, "completed" | "failed" | "skipped") {
            set_active_plugin_host_phase_context(self.origin, &self.target, None);
        }
    }

    fn record_starting(&self, phase: PluginEditorHostValidationPhase, message: impl Into<String>) {
        let message = message.into();
        self.log_phase_event(phase, "starting", &message);
    }

    fn record_completed(
        &mut self,
        phase: PluginEditorHostValidationPhase,
        message: impl Into<String>,
    ) {
        let message = message.into();
        self.log_phase_event(phase, "completed", &message);
    }

    fn record_skipped(
        &mut self,
        phase: PluginEditorHostValidationPhase,
        message: impl Into<String>,
    ) {
        let message = message.into();
        self.log_phase_event(phase, "skipped", &message);
    }

    fn record_failed(
        &mut self,
        phase: PluginEditorHostValidationPhase,
        probe_status: PluginEditorHostLoadInitProbeStatus,
        message: impl Into<String>,
    ) {
        let message = format!(
            "failure_kind={} {}",
            match probe_status {
                PluginEditorHostLoadInitProbeStatus::BlockedByHostContract => {
                    "blocked_by_host_contract"
                }
                PluginEditorHostLoadInitProbeStatus::BlockedByPluginFailure => {
                    "blocked_by_plugin_failure"
                }
                PluginEditorHostLoadInitProbeStatus::LoadInitSafe => "load_init_safe",
            },
            message.into()
        );
        self.log_phase_event(phase, "failed", &message);
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

fn record_phase_failure(
    telemetry: &mut PluginHostPhaseLogger,
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
            preserved_inventory_target_to_replace(requested_target, Some(session.target()))
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
fn focus_native_host_window_now() -> Result<(), String> {
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
fn focus_native_host_window(app: &AppHandle) -> Result<(), String> {
    if objc2::MainThreadMarker::new().is_some() {
        return focus_native_host_window_now();
    }

    let (sender, receiver) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = sender.send(focus_native_host_window_now());
    })
    .map_err(|error| format!("Failed to schedule native host window focus: {error}"))?;

    receiver
        .recv()
        .map_err(|error| format!("Failed to confirm native host window focus: {error}"))?
}

#[cfg(target_os = "macos")]
fn close_native_host_window_now() {
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
fn close_native_host_window(app: &AppHandle) -> Result<(), String> {
    if objc2::MainThreadMarker::new().is_some() {
        close_native_host_window_now();
        return Ok(());
    }

    let (sender, receiver) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        close_native_host_window_now();
        let _ = sender.send(());
    })
    .map_err(|error| format!("Failed to schedule native host window close: {error}"))?;

    receiver
        .recv()
        .map_err(|error| format!("Failed to confirm native host window close: {error}"))?;

    Ok(())
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
    use std::ffi::c_void;
    use std::mem::{zeroed, ManuallyDrop};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex as StdMutex, OnceLock};
    use std::time::Instant;
    use tauri::AppHandle;
    use vst3::com_scrape_types::{Guid, InterfaceList, MakeHeader, Wrapper};
    use vst3::Steinberg::IBStream_::IStreamSeekMode_::{kIBSeekCur, kIBSeekEnd, kIBSeekSet};
    use vst3::Steinberg::Vst::{
        self, AudioBusBuffers, AudioBusBuffers__type0, BusInfo, IAudioProcessorTrait, IComponent,
        IComponentHandler, IComponentHandler2, IComponentHandler2Trait, IComponentHandlerTrait,
        IComponentTrait, IComponent_iid, IConnectionPoint, IConnectionPointTrait, IEditController,
        IEditControllerTrait, IEditController_iid, IEventList, IEventListTrait, IHostApplication,
        IMessage, ViewType,
    };
    use vst3::Steinberg::Vst::{
        Event, Event_::EventFlags_ as VstEventFlags, Event_::EventTypes_ as VstEventTypes,
        NoteOffEvent, NoteOnEvent, ProcessData, ProcessSetup,
    };
    use vst3::Steinberg::{
        self, kInvalidArgument, kPlatformTypeNSView, kResultFalse, kResultOk, kResultTrue, tresult,
        FIDString, FUnknown, IBStream, IBStreamTrait, IPlugFrame, IPlugFrameTrait, IPlugView,
        IPlugViewTrait, IPluginBaseTrait, IPluginFactory, IPluginFactory2, IPluginFactory2Trait,
        IPluginFactory3, IPluginFactory3Trait, IPluginFactoryTrait, TBool, ViewRect, TUID,
    };
    use vst3::{Class, ComPtr, ComWrapper};

    type GetPluginFactoryFn = unsafe extern "system" fn() -> *mut IPluginFactory;
    type BundleEntryFn = unsafe extern "system" fn(*mut c_void) -> bool;
    type BundleExitFn = unsafe extern "system" fn() -> bool;

    unsafe extern "C" {
        fn solo_compose_create_steinberg_host_application() -> *mut IHostApplication;
        fn solo_compose_destroy_steinberg_host_application(host_application: *mut IHostApplication);
    }

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
                        .and_then(|session| session.view_state())
                }),
                None => active_plugin_host_session()
                    .lock()
                    .ok()
                    .and_then(|active_session| {
                        active_session
                            .as_ref()
                            .and_then(|session| session.view_state())
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

                // VST3 lifecycle: setActive before setProcessing
                // (matches MuseScore ensureActivity and Steinberg spec order).
                if !self.render_component_active {
                    let _ = unsafe { self.component.setActive(1) };
                    self.render_component_active = true;
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

            // VST3 lifecycle: setActive(0) after setProcessing(0)
            // (matches MuseScore disableActivity order).
            if self.render_component_active {
                let _ = unsafe { self.component.setActive(0) };
                self.render_component_active = false;
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

    macro_rules! impl_component_handler2_traits {
        ($ty:ty) => {
            impl IComponentHandler2Trait for $ty {
                unsafe fn setDirty(&self, state: TBool) -> tresult {
                    super::log_active_host_phase_callback(&format!(
                        "IComponentHandler2::setDirty state={state}",
                    ));
                    kResultOk
                }

                unsafe fn requestOpenEditor(&self, name: FIDString) -> tresult {
                    super::log_active_host_phase_callback(&format!(
                        "IComponentHandler2::requestOpenEditor name_ptr={}",
                        super::host_callback_pointer_label(name.cast::<c_void>())
                    ));
                    kResultOk
                }

                unsafe fn startGroupEdit(&self) -> tresult {
                    super::log_active_host_phase_callback(
                        "IComponentHandler2::startGroupEdit invoked",
                    );
                    kResultOk
                }

                unsafe fn finishGroupEdit(&self) -> tresult {
                    super::log_active_host_phase_callback(
                        "IComponentHandler2::finishGroupEdit invoked",
                    );
                    kResultOk
                }
            }
        };
    }

    macro_rules! impl_component_handler_traits {
        ($ty:ty) => {
            impl IComponentHandlerTrait for $ty {
                unsafe fn beginEdit(&self, id: Vst::ParamID) -> tresult {
                    super::log_active_host_phase_callback(&format!(
                        "IComponentHandler::beginEdit param_id={id}",
                    ));
                    kResultOk
                }

                unsafe fn performEdit(
                    &self,
                    id: Vst::ParamID,
                    value_normalized: Vst::ParamValue,
                ) -> tresult {
                    super::log_active_host_phase_callback(&format!(
                        "IComponentHandler::performEdit param_id={} value_normalized={}",
                        id, value_normalized,
                    ));
                    kResultOk
                }

                unsafe fn endEdit(&self, id: Vst::ParamID) -> tresult {
                    super::log_active_host_phase_callback(&format!(
                        "IComponentHandler::endEdit param_id={id}",
                    ));
                    kResultOk
                }

                unsafe fn restartComponent(&self, flags: i32) -> tresult {
                    let result = super::restart_component_result(flags);
                    super::log_active_host_phase_callback(&format!(
                        "IComponentHandler::restartComponent flags={} ({}) => {}",
                        flags,
                        super::restart_component_flags_label(flags),
                        if result == kResultOk {
                            "ok"
                        } else {
                            "not_implemented"
                        },
                    ));
                    result
                }
            }
        };
    }

    type PluginComponentHandlerInterfaceTuple = (IComponentHandler, IComponentHandler2);

    fn guid_to_tuid(iid: &Guid) -> TUID {
        [
            iid[0] as i8,
            iid[1] as i8,
            iid[2] as i8,
            iid[3] as i8,
            iid[4] as i8,
            iid[5] as i8,
            iid[6] as i8,
            iid[7] as i8,
            iid[8] as i8,
            iid[9] as i8,
            iid[10] as i8,
            iid[11] as i8,
            iid[12] as i8,
            iid[13] as i8,
            iid[14] as i8,
            iid[15] as i8,
        ]
    }

    static INTERFACE_POINTER_LABELS: OnceLock<StdMutex<FxHashMap<usize, usize>>> = OnceLock::new();
    static NEXT_INTERFACE_POINTER_LABEL: AtomicUsize = AtomicUsize::new(1);

    pub(super) fn interface_pointer_label<T>(pointer: *mut T) -> String {
        if pointer.is_null() {
            String::from("null")
        } else {
            let registry =
                INTERFACE_POINTER_LABELS.get_or_init(|| StdMutex::new(FxHashMap::default()));
            let pointer_address = pointer.cast::<c_void>() as usize;
            let mut registry = registry.lock().expect("pointer label registry poisoned");
            let label = registry
                .entry(pointer_address)
                .or_insert_with(|| NEXT_INTERFACE_POINTER_LABEL.fetch_add(1, Ordering::Relaxed));
            format!("instance#{}", label)
        }
    }

    fn log_component_handler_query_interface(iid: &Guid, supported: bool) {
        let iid_tuid = guid_to_tuid(iid);
        super::log_active_host_phase_callback(&format!(
            "PluginComponentHandler::queryInterface requested {} => {}",
            super::host_callback_iid_label(&iid_tuid),
            if supported {
                "supported"
            } else {
                "unsupported"
            }
        ));
    }

    pub(crate) struct PluginComponentHandlerInterfaces;

    unsafe impl InterfaceList for PluginComponentHandlerInterfaces {
        type Header = <PluginComponentHandlerInterfaceTuple as InterfaceList>::Header;

        fn query(iid: &Guid) -> Option<isize> {
            let result = <PluginComponentHandlerInterfaceTuple as InterfaceList>::query(iid);
            log_component_handler_query_interface(iid, result.is_some());
            result
        }
    }

    unsafe impl<C, W> MakeHeader<C, W> for PluginComponentHandlerInterfaces
    where
        C: Class<Interfaces = PluginComponentHandlerInterfaces>,
        W: Wrapper<C>,
        PluginComponentHandlerInterfaceTuple: MakeHeader<C, W>,
    {
        const HEADER: Self::Header =
            <PluginComponentHandlerInterfaceTuple as MakeHeader<C, W>>::HEADER;
    }

    pub(crate) struct PluginComponentHandler;

    impl Class for PluginComponentHandler {
        type Interfaces = PluginComponentHandlerInterfaces;
    }

    impl_component_handler_traits!(PluginComponentHandler);
    impl_component_handler2_traits!(PluginComponentHandler);

    pub(super) struct PluginHostApplicationHandle {
        interface: ManuallyDrop<ComPtr<IHostApplication>>,
    }

    impl PluginHostApplicationHandle {
        pub(super) fn as_ptr(&self) -> *mut IHostApplication {
            self.interface.as_ptr()
        }

        pub(super) fn clone_interface(&self) -> ComPtr<IHostApplication> {
            (&*self.interface).clone()
        }
    }

    impl Drop for PluginHostApplicationHandle {
        fn drop(&mut self) {
            let raw = self.interface.as_ptr();
            unsafe {
                ManuallyDrop::drop(&mut self.interface);
                solo_compose_destroy_steinberg_host_application(raw);
            }
        }
    }

    unsafe impl Send for PluginHostApplicationHandle {}
    unsafe impl Sync for PluginHostApplicationHandle {}

    pub(super) fn create_plugin_host_application() -> Result<PluginHostApplicationHandle, String> {
        let raw = unsafe { solo_compose_create_steinberg_host_application() };
        if raw.is_null() {
            return Err(String::from(
                "Failed to allocate the Steinberg SDK host application bridge.",
            ));
        }
        let host_application = unsafe { ComPtr::from_raw_unchecked(raw) };
        super::log_active_host_phase_callback(&format!(
            "Allocated PluginHostApplication host_application_ptr={}",
            interface_pointer_label(host_application.as_ptr())
        ));
        Ok(PluginHostApplicationHandle {
            interface: ManuallyDrop::new(host_application),
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn create_component_handler() -> Result<ComPtr<IComponentHandler>, String> {
        let component_handler = ComWrapper::new(PluginComponentHandler)
            .to_com_ptr::<IComponentHandler>()
            .ok_or_else(|| String::from("Failed to allocate the VST3 component-handler shim."))?;
        super::log_active_host_phase_callback(&format!(
            "Allocated PluginComponentHandler component_handler_ptr={}",
            interface_pointer_label(component_handler.as_ptr())
        ));
        Ok(component_handler)
    }

    fn connection_point_connect_succeeded(result: tresult) -> bool {
        result == kResultOk || result == kResultTrue
    }

    #[path = "vst3_host_core.rs"]
    pub(super) mod vst3_host_core;

    pub(crate) type PluginHostSession = vst3_host_core::PluginHostCoreSession;

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
            bundle.function_pointer_for_name(CFString::from_static_string("bundleEntry"));
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
                bundle.function_pointer_for_name(CFString::from_static_string("bundleExit"));
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

    fn decode_component_state_bytes(
        plugin_state: &PlaybackSlotPluginState,
    ) -> Result<Vec<u8>, String> {
        BASE64_STANDARD
            .decode(plugin_state.vst3_component_state_base64.as_bytes())
            .map_err(|error| format!("Failed to decode persisted VST3 component state: {error}"))
    }

    pub(super) fn restore_component_state(
        component: &ComPtr<IComponent>,
        plugin_state: &PlaybackSlotPluginState,
    ) -> Result<Vec<u8>, String> {
        let bytes = decode_component_state_bytes(plugin_state)?;
        let (stream_state, stream) = create_memory_stream()?;
        stream_state.replace_bytes(bytes.clone())?;

        let set_state_result = unsafe { component.setState(stream.as_ptr()) };
        if set_state_result != kResultOk {
            return Err(format!(
                "Plugin editor POC failed to restore the persisted VST3 component state (tresult = {}).",
                set_state_result
            ));
        }

        Ok(bytes)
    }

    pub(super) fn sync_component_state_to_controller(
        controller: &ComPtr<IEditController>,
        component_state_bytes: &[u8],
    ) -> Result<(), String> {
        let (stream_state, stream) = create_memory_stream()?;
        stream_state.replace_bytes(component_state_bytes.to_vec())?;

        let set_component_state_result = unsafe { controller.setComponentState(stream.as_ptr()) };
        if set_component_state_result != kResultOk {
            return Err(format!(
                "Plugin editor POC failed to mirror the persisted VST3 component state into the edit controller (tresult = {}).",
                set_component_state_result
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

    // ── ConnectionProxy ─────────────────────────────────────────────────
    //
    // Rust port of the Steinberg SDK ConnectionProxy
    // (public.sdk/source/vst/hosting/connectionproxy.h).
    //
    // The proxy sits between the host and a plugin connection point so that
    // messages flow through a host-owned intermediary, matching the
    // PlugProvider / MuseScore pattern that HALion Sonic requires.

    struct ConnectionProxy {
        dst_connection: StdMutex<Option<ComPtr<IConnectionPoint>>>,
    }

    impl ConnectionProxy {
        fn new(destination: ComPtr<IConnectionPoint>) -> Self {
            Self {
                dst_connection: StdMutex::new(Some(destination)),
            }
        }
    }

    impl Class for ConnectionProxy {
        type Interfaces = (IConnectionPoint,);
    }

    impl IConnectionPointTrait for ConnectionProxy {
        unsafe fn connect(&self, other: *mut IConnectionPoint) -> tresult {
            // The plugin may call connect on the proxy; store the peer.
            if other.is_null() {
                return kInvalidArgument;
            }
            // We already have a destination set at construction; accept.
            kResultOk
        }

        unsafe fn disconnect(&self, _other: *mut IConnectionPoint) -> tresult {
            let Ok(mut dst) = self.dst_connection.lock() else {
                return kResultFalse;
            };
            *dst = None;
            kResultOk
        }

        unsafe fn notify(&self, message: *mut IMessage) -> tresult {
            let Ok(dst) = self.dst_connection.lock() else {
                return kResultFalse;
            };
            if let Some(ref dst) = *dst {
                return dst.notify(message);
            }
            kResultFalse
        }
    }

    // ── Connection point wiring ──────────────────────────────────────────

    unsafe fn disconnect_connection_proxies(
        component: &ComPtr<IComponent>,
        controller: &ComPtr<IEditController>,
        component_proxy: &Option<ComPtr<IConnectionPoint>>,
        controller_proxy: &Option<ComPtr<IConnectionPoint>>,
    ) {
        if let (Some(comp_cp), Some(comp_proxy)) =
            (component.cast::<IConnectionPoint>(), component_proxy)
        {
            let _ = comp_cp.disconnect(comp_proxy.as_ptr());
        }
        if let (Some(ctrl_cp), Some(ctrl_proxy)) =
            (controller.cast::<IConnectionPoint>(), controller_proxy)
        {
            let _ = ctrl_cp.disconnect(ctrl_proxy.as_ptr());
        }
    }

    enum ConnectionPointHandshakeOutcome {
        Connected {
            message: String,
            component_proxy: ComPtr<IConnectionPoint>,
            controller_proxy: ComPtr<IConnectionPoint>,
        },
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

        // Create proxies following Steinberg PlugProvider pattern:
        // component_proxy wraps the controller destination — component sends
        // messages to this proxy which forwards to the controller.
        // controller_proxy wraps the component destination — controller sends
        // messages to this proxy which forwards to the component.
        let Some(component_proxy) =
            ComWrapper::new(ConnectionProxy::new(controller_connection.clone()))
                .to_com_ptr::<IConnectionPoint>()
        else {
            return ConnectionPointHandshakeOutcome::Skipped(String::from(
                "Failed to allocate VST3 component ConnectionProxy.",
            ));
        };
        let Some(controller_proxy) =
            ComWrapper::new(ConnectionProxy::new(component_connection.clone()))
                .to_com_ptr::<IConnectionPoint>()
        else {
            return ConnectionPointHandshakeOutcome::Skipped(String::from(
                "Failed to allocate VST3 controller ConnectionProxy.",
            ));
        };

        // Tell the component to send messages to our proxy.
        let component_connect_result =
            unsafe { component_connection.connect(component_proxy.as_ptr()) };
        if !connection_point_connect_succeeded(component_connect_result) {
            return ConnectionPointHandshakeOutcome::Skipped(format!(
                "Component connection-point connect returned {}; continuing without proxied pair.",
                component_connect_result
            ));
        }

        // Tell the controller to send messages to our proxy.
        let controller_connect_result =
            unsafe { controller_connection.connect(controller_proxy.as_ptr()) };
        if !connection_point_connect_succeeded(controller_connect_result) {
            unsafe {
                let _ = component_connection.disconnect(component_proxy.as_ptr());
            }
            return ConnectionPointHandshakeOutcome::Skipped(format!(
                "Controller connection-point connect returned {}; continuing without proxied pair.",
                controller_connect_result
            ));
        }

        ConnectionPointHandshakeOutcome::Connected {
            message: String::from(
                "Connected component and controller connection points via ConnectionProxy.",
            ),
            component_proxy,
            controller_proxy,
        }
    }

    #[allow(dead_code)] // Retained for reference; init-time runtime setup is now deferred to ensure_render_processing.
    fn apply_component_runtime_setup(
        component: &ComPtr<IComponent>,
        phase_logger: &mut super::PluginHostPhaseLogger,
    ) -> Result<(), PluginEditorHostAttemptError> {
        if let Some(processor) = component.cast::<Vst::IAudioProcessor>() {
            activate_component_buses(component).map_err(|error| {
                super::record_phase_failure(
                    phase_logger,
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
        let mut phase_logger = super::PluginHostPhaseLogger::terminal("session_build", target);
        build_plugin_host_session_with_logger(
            target,
            restorable_plugin_state,
            super::PluginEditorPocProbeProfile::Default,
            &mut phase_logger,
        )
    }

    fn build_plugin_host_session_with_logger(
        target: &super::PluginEditorTarget,
        restorable_plugin_state: Option<&PlaybackSlotPluginState>,
        probe_profile: super::PluginEditorPocProbeProfile,
        phase_logger: &mut super::PluginHostPhaseLogger,
    ) -> Result<PluginHostSession, PluginEditorHostAttemptError> {
        vst3_host_core::build_plugin_host_core_session_with_logger(
            target,
            restorable_plugin_state,
            probe_profile,
            phase_logger,
        )
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

        let session = if let Ok(mut active_session) = active_plugin_host_session().lock() {
            active_session.take()
        } else {
            None
        };
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

        let session_state: Arc<StdMutex<Option<PluginHostSession>>> =
            Arc::new(StdMutex::new(Some(session)));
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
        super::emit_plugin_host_terminal_target_event(
            "view_attach",
            target,
            None,
            "starting",
            "Scheduling native host attachment on the main thread.",
        );
        let (sender, receiver) = std::sync::mpsc::channel();
        let app_clone = app.clone();
        let target = target.clone();
        let schedule_error_target = target.clone();
        let plugin_path = target.path().to_path_buf();

        app_clone
            .run_on_main_thread(move || {
                super::emit_plugin_host_terminal_target_event(
                    "view_attach",
                    &target,
                    None,
                    "info",
                    "Entered the main-thread native host attach flow.",
                );
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
                        let mut phase_logger =
                            super::PluginHostPhaseLogger::terminal("view_attach", &target);
                        let result =
                            session.attach_view(ns_window_ptr, &content_view, &mut phase_logger);
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
                        let mut phase_logger =
                            super::PluginHostPhaseLogger::terminal("view_attach", &target);
                        let result =
                            session.attach_view(ns_window_ptr, &content_view, &mut phase_logger);
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
                        super::emit_plugin_host_terminal_target_event(
                            "view_attach",
                            &target,
                            Some(error.phase),
                            "failed",
                            &error.message,
                        );
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

                    super::emit_plugin_host_terminal_target_event(
                        "view_attach",
                        &target,
                        Some(PluginEditorHostValidationPhase::Attached),
                        "completed",
                        "Completed native host attachment on the bounded editor-open path.",
                    );

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
            .map_err(|error| {
                super::emit_plugin_host_terminal_target_event(
                    "view_attach",
                    &schedule_error_target,
                    None,
                    "failed",
                    &format!("Failed to schedule native host attachment: {error}"),
                );
                format!("Failed to schedule native host attachment: {error}")
            })?;

        let wait_started_at = log_main_thread_wait_start("attach_native_host_surface");
        let received = receiver
            .recv()
            .map_err(|error| format!("Failed to receive native host attachment result: {error}"));
        log_main_thread_wait_finish("attach_native_host_surface", wait_started_at);
        received?
    }
}

#[cfg(target_os = "macos")]
use macos_host::{
    attach_native_host_surface as attach_native_host_surface_impl,
    capture_slot_runtime_plugin_state as capture_slot_runtime_plugin_state_impl,
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
    emit_plugin_host_terminal_target_event(
        "editor_open",
        &target,
        Some(PluginEditorHostValidationPhase::PolicyGate),
        "starting",
        "Received an ordinary bounded editor-open request.",
    );
    if let Err(reason) = ensure_editor_host_target_supported(target.path()) {
        emit_plugin_host_terminal_target_event(
            "editor_open",
            &target,
            Some(PluginEditorHostValidationPhase::PolicyGate),
            "blocked",
            &reason,
        );
        queue_editor_host_validation_emit(
            app.clone(),
            target.path().to_path_buf(),
            blocked_editor_host_validation(reason.clone()),
        );
        return Err(reason);
    }
    emit_plugin_host_terminal_target_event(
        "editor_open",
        &target,
        Some(PluginEditorHostValidationPhase::PolicyGate),
        "completed",
        "Target passed the ordinary bounded policy gate.",
    );

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
                focus_native_host_window(&app)?;

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
        focus_native_host_window(&app)?;

        return Ok(PluginEditorPocOpenResult {
            window_label: WINDOW_LABEL.to_string(),
            plugin_path: target.path().display().to_string(),
            native_host_attached: live_plugin_host_attached(),
        });
    }

    let attach_outcome = match attach_native_host_surface(&app, &target) {
        Ok(outcome) => outcome,
        Err(error) => {
            emit_plugin_host_terminal_target_event(
                "editor_open",
                &target,
                None,
                "failed",
                &format!("attach_native_host_surface returned an outer error: {error}"),
            );
            release_reserved_plugin_target(&target);
            let _ = close_native_host_window(&app);
            return Err(error);
        }
    };
    let attach_validation = attach_outcome.validation.clone();
    queue_editor_host_validation_emit(
        app.clone(),
        target.path().to_path_buf(),
        attach_outcome.validation,
    );

    if !attach_outcome.native_shown {
        // attach_native_host_surface shows via makeKeyAndOrderFront in the
        // normal success path. This branch covers error-overlay cases.
        let _ = focus_native_host_window(&app);
    } else {
        eprintln!(
            "[DIAG] plugin_editor_poc_open: window already shown natively, skipping async show"
        );
    }

    emit_plugin_host_terminal_target_event(
        "editor_open",
        &target,
        Some(attach_validation.phase),
        if attach_outcome.attached {
            "completed"
        } else {
            "failed"
        },
        &attach_validation.message,
    );

    // Diagnostic: verify main thread is responsive after open.
    let diag_app = app.clone();
    let _ = diag_app.run_on_main_thread(move || {});

    Ok(PluginEditorPocOpenResult {
        window_label: WINDOW_LABEL.to_string(),
        plugin_path: target.path().display().to_string(),
        native_host_attached: attach_outcome.attached,
    })
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
    use crate::playback::runtime_activation::ActivatedSlotTarget;
    use crate::score::playback::{PlaybackSlotPluginState, PlaybackSlotTarget};
    use crate::utils::UniqueId;
    #[cfg(target_os = "macos")]
    use std::ffi::c_void;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    #[cfg(target_os = "macos")]
    fn new_host_application() -> super::macos_host::PluginHostApplicationHandle {
        super::macos_host::create_plugin_host_application()
            .expect("expected a plugin host application")
    }

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
    fn unsupported_editor_host_target_reason_allows_halion_sonic_bundle_name() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
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
    fn ensure_editor_host_target_supported_allows_halion_sonic_without_touching_lifecycle_state() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        ensure_editor_host_target_supported(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3",
        ))
        .unwrap();
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
        use vst3::Steinberg::kResultTrue;
        use vst3::Steinberg::Vst::{IHostApplicationTrait, IMessage_iid};

        let host_application = new_host_application();
        let host_application_ptr = host_application.clone_interface();

        let mut raw_message: *mut c_void = std::ptr::null_mut();
        let mut message_cid = IMessage_iid;
        let mut message_iid = IMessage_iid;
        let message_result = unsafe {
            host_application_ptr.createInstance(
                &mut message_cid,
                &mut message_iid,
                &mut raw_message,
            )
        };
        assert_eq!(message_result, kResultTrue);
        assert!(!raw_message.is_null());
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

    #[test]
    fn host_callback_iid_name_from_ptr_marks_invalid_sentinel_pointers() {
        use vst3::Steinberg::TUID;

        assert_eq!(
            super::host_callback_iid_name_from_ptr(0xFFFF_FFFF_usize as *const TUID),
            "invalid_pointer(0xFFFFFFFF)"
        );
    }

    #[test]
    fn restart_component_flags_label_names_note_expression_flag() {
        assert_eq!(
            super::restart_component_flags_label(
                vst3::Steinberg::Vst::RestartFlags_::kNoteExpressionChanged,
            ),
            "kNoteExpressionChanged"
        );
    }

    #[test]
    fn restart_component_flags_label_includes_unknown_bits() {
        assert_eq!(
            super::restart_component_flags_label(4096),
            "unknown_bits=0x1000"
        );
    }

    #[test]
    fn restart_component_result_keeps_note_expression_restart_supported() {
        assert_eq!(
            super::restart_component_result(
                vst3::Steinberg::Vst::RestartFlags_::kNoteExpressionChanged,
            ),
            vst3::Steinberg::kResultOk
        );
    }

    #[test]
    fn restart_component_result_keeps_plain_restarts_supported() {
        assert_eq!(
            super::restart_component_result(vst3::Steinberg::Vst::RestartFlags_::kReloadComponent),
            vst3::Steinberg::kResultOk
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_exposes_steinberg_host_application_surface() {
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Vst::{
            IComponentHandler, IComponentHandler2, IHostApplication, IPlugInterfaceSupport,
            IUnitHandler, IUnitHandler2,
        };

        let host_application = new_host_application();
        let host_application_ptr = host_application.clone_interface();
        let unknown = host_application_ptr.clone().upcast::<FUnknown>();

        assert!(unknown.cast::<IHostApplication>().is_some());
        assert!(unknown.cast::<IPlugInterfaceSupport>().is_some());
        assert!(unknown.cast::<IComponentHandler>().is_none());
        assert!(unknown.cast::<IComponentHandler2>().is_none());
        assert!(unknown.cast::<IUnitHandler>().is_none());
        assert!(unknown.cast::<IUnitHandler2>().is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_support_reports_supported_vst_plugin_interfaces() {
        use vst3::Steinberg::kResultFalse;
        use vst3::Steinberg::kResultTrue;
        use vst3::Steinberg::Vst::IAttributeList_iid;
        use vst3::Steinberg::Vst::{
            IAudioProcessor_iid, IComponentHandler2_iid, IComponentHandler_iid, IComponent_iid,
            IConnectionPoint_iid, IEditController2_iid, IEditController_iid, IHostApplication_iid,
            IMessage_iid, IMidiMapping_iid, IPlugInterfaceSupport, IPlugInterfaceSupportTrait,
            IPlugInterfaceSupport_iid, IProgramListData_iid, IUnitData_iid, IUnitHandler_iid,
            IUnitInfo_iid,
        };

        let host_application = new_host_application();
        let host_application_ptr = host_application.clone_interface();
        let plug_interface_support: vst3::ComPtr<IPlugInterfaceSupport> = host_application_ptr
            .clone()
            .upcast::<vst3::Steinberg::FUnknown>()
            .cast::<IPlugInterfaceSupport>()
            .unwrap();

        let host_application_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IHostApplication_iid) };
        let plug_interface_support_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IPlugInterfaceSupport_iid) };
        let message_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IMessage_iid) };
        let attribute_list_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IAttributeList_iid) };
        let component_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IComponent_iid) };
        let audio_processor_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IAudioProcessor_iid) };
        let edit_controller_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IEditController_iid) };
        let connection_point_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IConnectionPoint_iid) };
        let unit_info_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IUnitInfo_iid) };
        let unit_data_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IUnitData_iid) };
        let program_list_data_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IProgramListData_iid) };
        let midi_mapping_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IMidiMapping_iid) };
        let edit_controller2_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IEditController2_iid) };
        let component_handler_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IComponentHandler_iid) };
        let component_handler2_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IComponentHandler2_iid) };
        let unit_handler_result =
            unsafe { plug_interface_support.isPlugInterfaceSupported(&IUnitHandler_iid) };

        assert_eq!(host_application_result, kResultFalse);
        assert_eq!(plug_interface_support_result, kResultFalse);
        assert_eq!(message_result, kResultFalse);
        assert_eq!(attribute_list_result, kResultFalse);
        assert_eq!(component_result, kResultTrue);
        assert_eq!(audio_processor_result, kResultTrue);
        assert_eq!(edit_controller_result, kResultTrue);
        assert_eq!(connection_point_result, kResultTrue);
        assert_eq!(unit_info_result, kResultTrue);
        assert_eq!(unit_data_result, kResultTrue);
        assert_eq!(program_list_data_result, kResultTrue);
        assert_eq!(midi_mapping_result, kResultTrue);
        assert_eq!(edit_controller2_result, kResultTrue);
        assert_eq!(component_handler_result, kResultFalse);
        assert_eq!(component_handler2_result, kResultFalse);
        assert_eq!(unit_handler_result, kResultFalse);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn component_handler_matches_split_musescore_style_surface() {
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Linux::IRunLoop;
        use vst3::Steinberg::Vst::{
            IComponentHandler, IComponentHandler2, IComponentHandler3, IHostApplication,
            IPlugInterfaceSupport, IProgress, IUnitHandler, IUnitHandler2,
        };

        let component_handler = super::macos_host::create_component_handler().unwrap();
        let unknown = component_handler.clone().upcast::<FUnknown>();

        assert!(unknown.cast::<IComponentHandler>().is_some());
        assert!(unknown.cast::<IComponentHandler2>().is_some());
        assert!(unknown.cast::<IComponentHandler3>().is_none());
        assert!(unknown.cast::<IProgress>().is_none());
        assert!(unknown.cast::<IUnitHandler>().is_none());
        assert!(unknown.cast::<IUnitHandler2>().is_none());
        assert!(unknown.cast::<IHostApplication>().is_none());
        assert!(unknown.cast::<IPlugInterfaceSupport>().is_none());
        assert!(unknown.cast::<IRunLoop>().is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_exposes_plug_interface_support() {
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Vst::IPlugInterfaceSupport;

        let host_application = new_host_application();
        let unknown = host_application.clone_interface().upcast::<FUnknown>();

        assert!(unknown.cast::<IPlugInterfaceSupport>().is_some());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_query_interface_returns_distinct_plug_interface_support_object() {
        use std::ffi::c_void;
        use vst3::Steinberg::kResultOk;
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Vst::IPlugInterfaceSupport_iid;

        let host_application = new_host_application();
        let unknown = host_application.clone_interface().upcast::<FUnknown>();
        let mut raw_interface: *mut c_void = std::ptr::null_mut();
        let mut iid = IPlugInterfaceSupport_iid;

        let result = unsafe {
            ((*(*unknown.as_ptr()).vtbl).queryInterface)(
                unknown.as_ptr(),
                &mut iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kResultOk);
        assert!(!raw_interface.is_null());
        assert_ne!(raw_interface, host_application.as_ptr().cast::<c_void>());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_does_not_expose_component_handler_interface() {
        use std::ffi::c_void;
        use vst3::Steinberg::kResultFalse;
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Vst::IComponentHandler_iid;

        let host_application = new_host_application();
        let unknown = host_application.clone_interface().upcast::<FUnknown>();
        let mut raw_interface: *mut c_void = std::ptr::null_mut();
        let mut iid = IComponentHandler_iid;

        let result = unsafe {
            ((*(*unknown.as_ptr()).vtbl).queryInterface)(
                unknown.as_ptr(),
                &mut iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kResultFalse);
        assert!(raw_interface.is_null());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_refcount_methods_follow_steinberg_hostclass_semantics() {
        use vst3::Steinberg::FUnknown;

        let host_application = new_host_application();
        let unknown = host_application.as_ptr().cast::<FUnknown>();

        let add_ref = unsafe { ((*(*unknown).vtbl).addRef)(unknown) };
        let release = unsafe { ((*(*unknown).vtbl).release)(unknown) };

        assert_eq!(add_ref, 1);
        assert_eq!(release, 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_returns_truthful_unsupported_for_non_public_interfaces() {
        use std::ffi::c_void;
        use vst3::Steinberg::Vst::IVst3ToAUWrapper_iid;
        use vst3::Steinberg::{kResultFalse, FUnknown, TUID};

        let host_application = new_host_application();
        let unknown = host_application.clone_interface().upcast::<FUnknown>();
        let mut raw_interface: *mut c_void = std::ptr::null_mut();
        let mut wrapper_iid = IVst3ToAUWrapper_iid;

        let result = unsafe {
            ((*(*unknown.as_ptr()).vtbl).queryInterface)(
                unknown.as_ptr(),
                &mut wrapper_iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kResultFalse);
        assert!(raw_interface.is_null());

        let mut unknown_iid: TUID = [
            0x10_i8, 0x20_i8, 0x30_i8, 0x40_i8, 0x50_i8, 0x60_i8, 0x70_i8, 0x11_i8, 0x22_i8,
            0x33_i8, 0x44_i8, 0x55_i8, 0x66_i8, 0x77_i8, 0x7F_i8, 0x01_i8,
        ];
        raw_interface = std::ptr::null_mut();

        let result = unsafe {
            ((*(*unknown.as_ptr()).vtbl).queryInterface)(
                unknown.as_ptr(),
                &mut unknown_iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kResultFalse);
        assert!(raw_interface.is_null());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_create_instance_returns_false_for_unknown_class_id() {
        use std::ffi::c_void;
        use vst3::Steinberg::Vst::{IHostApplicationTrait, IMessage_iid};
        use vst3::Steinberg::{kResultFalse, TUID};

        let host_application = new_host_application();
        let host_application_ptr = host_application.clone_interface();
        let mut raw_interface: *mut c_void = std::ptr::null_mut();
        let mut message_iid = IMessage_iid;
        let mut unknown_cid: TUID = [
            0x10_i8, 0x20_i8, 0x30_i8, 0x40_i8, 0x50_i8, 0x60_i8, 0x70_i8, 0x11_i8, 0x22_i8,
            0x33_i8, 0x44_i8, 0x55_i8, 0x66_i8, 0x77_i8, 0x7F_i8, 0x01_i8,
        ];

        let result = unsafe {
            host_application_ptr.createInstance(
                &mut unknown_cid,
                &mut message_iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kResultFalse);
        assert!(raw_interface.is_null());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_create_instance_returns_false_for_mismatched_interface() {
        use std::ffi::c_void;
        use vst3::Steinberg::kResultFalse;
        use vst3::Steinberg::Vst::{IAttributeList_iid, IHostApplicationTrait, IMessage_iid};

        let host_application = new_host_application();
        let host_application_ptr = host_application.clone_interface();
        let mut raw_interface: *mut c_void = std::ptr::null_mut();
        let mut message_cid = IMessage_iid;
        let mut attribute_list_iid = IAttributeList_iid;

        let result = unsafe {
            host_application_ptr.createInstance(
                &mut message_cid,
                &mut attribute_list_iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kResultFalse);
        assert!(raw_interface.is_null());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_create_instance_returns_attribute_list_for_attribute_list_iid() {
        use std::ffi::c_void;
        use vst3::Steinberg::kResultTrue;
        use vst3::Steinberg::Vst::{IAttributeList_iid, IHostApplicationTrait};

        let host_application = new_host_application();
        let host_application_ptr = host_application.clone_interface();
        let mut raw_attribute_list: *mut c_void = std::ptr::null_mut();
        let mut attribute_list_cid = IAttributeList_iid;
        let mut attribute_list_iid = IAttributeList_iid;

        let result = unsafe {
            host_application_ptr.createInstance(
                &mut attribute_list_cid,
                &mut attribute_list_iid,
                &mut raw_attribute_list,
            )
        };

        assert_eq!(result, kResultTrue);
        assert!(!raw_attribute_list.is_null());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_create_instance_returns_false_for_unknown_interface() {
        use std::ffi::c_void;
        use vst3::Steinberg::kResultFalse;
        use vst3::Steinberg::Vst::{IHostApplicationTrait, IMessage_iid};
        use vst3::Steinberg::TUID;

        let host_application = new_host_application();
        let host_application_ptr = host_application.clone_interface();
        let mut raw_interface: *mut c_void = std::ptr::null_mut();
        let mut message_cid = IMessage_iid;
        let mut unknown_iid: TUID = [
            0x10_i8, 0x20_i8, 0x30_i8, 0x40_i8, 0x50_i8, 0x60_i8, 0x70_i8, 0x11_i8, 0x22_i8,
            0x33_i8, 0x44_i8, 0x55_i8, 0x66_i8, 0x77_i8, 0x7F_i8, 0x01_i8,
        ];

        let result = unsafe {
            host_application_ptr.createInstance(
                &mut message_cid,
                &mut unknown_iid,
                &mut raw_interface,
            )
        };

        assert_eq!(result, kResultFalse);
        assert!(raw_interface.is_null());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_reports_sdk_default_name_for_valid_get_name_buffers() {
        use vst3::Steinberg::kResultTrue;
        use vst3::Steinberg::Vst::String128;

        let host_application = new_host_application();
        let mut name: String128 = [0; 128];

        let result = unsafe {
            ((*(*host_application.as_ptr()).vtbl).getName)(host_application.as_ptr(), &mut name)
        };
        let expected: Vec<u16> = "My VST3 HostApplication".encode_utf16().collect();

        assert_eq!(result, kResultTrue);
        assert_eq!(&name[..expected.len()], expected.as_slice());
        assert_eq!(name[expected.len()], 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_writes_name_for_valid_get_name_buffers() {
        use vst3::Steinberg::kResultTrue;
        use vst3::Steinberg::Vst::String128;

        let host_application = new_host_application();
        let mut name: String128 = [0; 128];

        let result = unsafe {
            ((*(*host_application.as_ptr()).vtbl).getName)(host_application.as_ptr(), &mut name)
        };

        let expected: Vec<u16> = "My VST3 HostApplication".encode_utf16().collect();

        assert_eq!(result, kResultTrue);
        assert_eq!(&name[..expected.len()], expected.as_slice());
        assert_eq!(name[expected.len()], 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_core_profiles_share_the_same_plug_provider_defaults() {
        let default_profile =
            super::macos_host::vst3_host_core::Vst3HostCoreLifecycleProfile::from_probe_profile(
                super::PluginEditorPocProbeProfile::Default,
            );
        let comparison_profile =
            super::macos_host::vst3_host_core::Vst3HostCoreLifecycleProfile::from_probe_profile(
                super::PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned,
            );

        assert_eq!(default_profile, comparison_profile);
        assert!(!default_profile.delays_component_runtime_setup());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_component_handler_is_distinct_from_host_application() {
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::Vst::IComponentHandler2;

        let host_application = new_host_application();
        let component_handler = super::macos_host::create_component_handler().unwrap();
        let host_unknown = host_application.clone_interface().upcast::<FUnknown>();
        let component_handler_unknown = component_handler.clone().upcast::<FUnknown>();

        assert_ne!(component_handler_unknown.as_ptr(), host_unknown.as_ptr());
        assert!(component_handler_unknown
            .cast::<IComponentHandler2>()
            .is_some());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_core_teardown_steps_preserve_reverse_teardown_order() {
        use super::macos_host::vst3_host_core::{
            teardown_steps, Vst3HostCoreTeardownState, Vst3HostCoreTeardownStep,
        };

        let steps = teardown_steps(Vst3HostCoreTeardownState {
            component_initialized: true,
            controller_initialized: true,
            controller_shared_with_component: false,
            connection_points_connected: true,
        });

        assert_eq!(
            steps,
            vec![
                Vst3HostCoreTeardownStep::PrepareWindowClose,
                Vst3HostCoreTeardownStep::ClearControllerComponentHandler,
                Vst3HostCoreTeardownStep::DisconnectConnectionPoints,
                Vst3HostCoreTeardownStep::TerminateController,
                Vst3HostCoreTeardownStep::TerminateComponent,
            ]
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_core_teardown_skips_controller_only_steps_for_shared_instances() {
        use super::macos_host::vst3_host_core::{
            teardown_steps, Vst3HostCoreTeardownState, Vst3HostCoreTeardownStep,
        };

        let steps = teardown_steps(Vst3HostCoreTeardownState {
            component_initialized: true,
            controller_initialized: true,
            controller_shared_with_component: true,
            connection_points_connected: false,
        });

        assert_eq!(
            steps,
            vec![
                Vst3HostCoreTeardownStep::PrepareWindowClose,
                Vst3HostCoreTeardownStep::TerminateComponent,
            ]
        );
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
}
