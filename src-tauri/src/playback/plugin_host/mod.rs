#![allow(non_snake_case)]

use crate::playback::engine::{set_slot_editor_render_active, LiveRenderEvent};
use crate::playback::plugin_inventory::bounded_editor_host_unsupported_reason;
use crate::playback::plugin_inventory::{
    blocked_editor_host_validation, PluginCapabilityEvidence, PluginCapabilityEvidenceStatus,
    PluginEditorHostLoadInitProbeStatus, PluginEditorHostValidation,
    PluginEditorHostValidationPhase, PluginEditorHostValidationStatus,
};
use crate::playback::plugin_inventory::{
    default_vst3_search_roots, discover_first_valid_vst3_in_roots, validate_vst3_bundle,
    MACOS_VST3_SEARCH_ROOTS,
};
use crate::playback::runtime_activation::ActivatedSlotTarget;
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
mod macos_host;

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
    Err(String::from("plugin_host_open is macOS-only"))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_host_open(
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
        "[DIAG] plugin_host_open: existing_window={}",
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
        eprintln!("[DIAG] plugin_host_open: window already shown natively, skipping async show");
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
    use crate::playback::plugin_inventory::bounded_editor_host_unsupported_reason;
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
