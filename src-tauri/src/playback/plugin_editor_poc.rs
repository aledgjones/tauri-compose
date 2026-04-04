#![allow(non_snake_case)]

use crate::playback::engine::{set_slot_editor_render_active, LiveRenderEvent};
use crate::playback::plugin_host_compatibility::bounded_editor_host_unsupported_reason;
use crate::playback::plugin_inventory::{
    blocked_editor_host_validation, PluginCapabilityEvidence, PluginCapabilityEvidenceStatus,
    PluginEditorHostValidation, PluginEditorHostValidationPhase, PluginEditorHostValidationStatus,
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
use serde::Serialize;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{
    AppHandle, Manager, State, TitleBarStyle, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};

const WINDOW_LABEL: &str = "plugin-host-poc";
const HOST_PAGE_PATH: &str = "plugin-host-poc.html";
const HOST_HEADER_HEIGHT: f64 = 0.0;
const PLUGIN_PATH_ENV: &str = "SOLOC_PLUGIN_EDITOR_POC_VST3_PATH";

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
    Closing,
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
    InitialAttach,
    WindowFocusChanged,
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
        && matches!(
            state.phase,
            PluginHostLifecyclePhase::Attached | PluginHostLifecyclePhase::Closing
        )
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

fn mark_plugin_target_detached(target: &PluginEditorTarget) {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        if lifecycle.target.as_ref() == Some(target) {
            lifecycle.phase = PluginHostLifecyclePhase::Detached;
            if let Some(slot_id) = target.slot_id {
                set_slot_editor_render_active(slot_id, false);
            }
        }
    }
}

fn begin_plugin_target_close(target: Option<&PluginEditorTarget>) {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        let should_close = match target {
            Some(target) => lifecycle.target.as_ref() == Some(target),
            None => lifecycle.target.is_some(),
        };

        if should_close {
            lifecycle.phase = PluginHostLifecyclePhase::Closing;
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

#[derive(Clone, Debug)]
struct PluginEditorHostAttemptError {
    phase: PluginEditorHostValidationPhase,
    message: String,
}

impl PluginEditorHostAttemptError {
    fn new(phase: PluginEditorHostValidationPhase, message: impl Into<String>) -> Self {
        Self {
            phase,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
struct PluginHostAttachOutcome {
    attached: bool,
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

fn build_host_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    let window =
        WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App(HOST_PAGE_PATH.into()))
            .title("Plugin Host POC")
            .title_bar_style(TitleBarStyle::Transparent)
            .hidden_title(false)
            .visible(false)
            .inner_size(960.0, 720.0)
            .resizable(false)
            .center()
            .accept_first_mouse(true)
            .build()
            .map_err(|error| format!("Failed to create plugin host POC window: {error}"))?;

    #[cfg(target_os = "macos")]
    window
        .set_title_bar_style(TitleBarStyle::Transparent)
        .map_err(|error| format!("Failed to apply plugin host title bar style: {error}"))?;

    let event_window = window.clone();
    window.on_window_event(move |event| log_window_event(&event_window, event));

    Ok(window)
}

fn log_window_event(window: &WebviewWindow, event: &WindowEvent) {
    match event {
        WindowEvent::Resized(_size) => {
            #[cfg(target_os = "macos")]
            sync_active_plugin_host(window, None, PluginHostSyncReason::WindowResized);
        }
        WindowEvent::Focused(focused) => {
            #[cfg(target_os = "macos")]
            sync_active_plugin_host(
                window,
                Some(*focused),
                PluginHostSyncReason::WindowFocusChanged,
            );
        }
        WindowEvent::CloseRequested { .. } => {
            let active_target = active_plugin_target_identity();
            begin_plugin_target_close(active_target.as_ref());
            #[cfg(target_os = "macos")]
            prepare_active_plugin_host_session_for_close(window);
        }
        WindowEvent::Destroyed => {
            #[cfg(target_os = "macos")]
            cleanup_active_plugin_host_session(window);
        }
        _ => {}
    }
}

#[cfg(target_os = "macos")]
mod macos_host {
    use super::{
        active_plugin_host_session, attached_editor_host_validation, failed_editor_host_validation,
        PluginEditorHostAttemptError, PluginEditorHostValidationPhase, PluginHostAttachOutcome,
        SlotRuntimePluginStateCapture, HOST_HEADER_HEIGHT,
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
    use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSAutoresizingMaskOptions, NSTextField, NSView, NSWindow};
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
    use rustc_hash::FxHashMap;
    use std::ffi::{c_void, CStr, CString};
    use std::mem::zeroed;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex as StdMutex};
    use std::time::{Duration, Instant};
    use tauri::{AppHandle, WebviewWindow};
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
        IHostApplication_iid, IMessage_iid, IPlugInterfaceSupport_iid,
        IProgress_::{ProgressType, ID as ProgressID},
        IProgress_iid, IUnitHandler2_iid, IUnitHandler_iid, NoteOffEvent, NoteOnEvent, ProcessData,
        ProcessSetup,
    };
    use vst3::Steinberg::{
        self, int64, kInvalidArgument, kNotImplemented, kPlatformTypeNSView, kResultFalse,
        kResultOk, kResultTrue, tresult, uint32, FIDString, FUnknown, FUnknown_iid, IBStream,
        IBStreamTrait, IPlugFrame, IPlugFrameTrait, IPlugView, IPlugViewTrait, IPluginBaseTrait,
        IPluginFactory, IPluginFactory2, IPluginFactory2Trait, IPluginFactory3,
        IPluginFactory3Trait, IPluginFactoryTrait, TBool, ViewRect, TUID,
    };
    use vst3::{Class, ComPtr, ComWrapper};

    const BUFFERED_REATTACH_RESIZE_DELAY_MS: u64 = 50;

    type GetPluginFactoryFn = unsafe extern "system" fn() -> *mut IPluginFactory;
    type BundleEntryFn = unsafe extern "system" fn(*mut c_void) -> bool;
    type BundleExitFn = unsafe extern "system" fn() -> bool;

    define_class!(
        #[unsafe(super(NSView))]
        #[name = "SoloComposePluginHostView"]
        struct PluginHostContainerView;

        impl PluginHostContainerView {
            #[unsafe(method(isFlipped))]
            fn is_flipped(&self) -> bool {
                true
            }
        }
    );

    fn sync_host_container_subviews_to_bounds(host_view: &NSView) {
        let host_bounds = host_view.bounds();
        let subviews = host_view.subviews();

        for subview in subviews {
            subview.setFrame(host_bounds);
            subview.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewHeightSizable,
            );
        }
    }

    pub(crate) fn pending_plugin_requested_resize() -> &'static AtomicBool {
        static PENDING_PLUGIN_REQUESTED_RESIZE: AtomicBool = AtomicBool::new(false);
        &PENDING_PLUGIN_REQUESTED_RESIZE
    }

    fn queued_plugin_requested_resize() -> &'static StdMutex<Option<(f64, f64)>> {
        static QUEUED_PLUGIN_REQUESTED_RESIZE: std::sync::OnceLock<StdMutex<Option<(f64, f64)>>> =
            std::sync::OnceLock::new();
        QUEUED_PLUGIN_REQUESTED_RESIZE.get_or_init(|| StdMutex::new(None))
    }

    fn plugin_requested_resize_dispatch_in_flight() -> &'static AtomicBool {
        static PLUGIN_REQUESTED_RESIZE_DISPATCH_IN_FLIGHT: AtomicBool = AtomicBool::new(false);
        &PLUGIN_REQUESTED_RESIZE_DISPATCH_IN_FLIGHT
    }

    pub(crate) fn pending_non_resizable_attach_layout_sync() -> &'static AtomicBool {
        static PENDING_NON_RESIZABLE_ATTACH_LAYOUT_SYNC: AtomicBool = AtomicBool::new(false);
        &PENDING_NON_RESIZABLE_ATTACH_LAYOUT_SYNC
    }

    pub(crate) fn clear_pending_plugin_resize_request() {
        pending_plugin_requested_resize().store(false, Ordering::Release);
        plugin_requested_resize_dispatch_in_flight().store(false, Ordering::Release);
        pending_non_resizable_attach_layout_sync().store(false, Ordering::Release);
        if let Ok(mut queued_resize) = queued_plugin_requested_resize().lock() {
            *queued_resize = None;
        }
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

    fn active_host_view_ptr() -> Option<usize> {
        active_session_view_state().map(|(_, host_view_ptr, _)| host_view_ptr)
    }

    fn active_session_plugin_can_resize() -> Option<bool> {
        match super::active_plugin_target_identity() {
            Some(target) => match target.slot_id {
                Some(slot_id) => slot_runtime_sessions().lock().ok().and_then(|sessions| {
                    sessions
                        .get(&slot_id)
                        .map(|session| session.plugin_can_resize)
                }),
                None => active_plugin_host_session()
                    .lock()
                    .ok()
                    .and_then(|active_session| {
                        active_session
                            .as_ref()
                            .map(|session| session.plugin_can_resize)
                    }),
            },
            None => None,
        }
    }

    fn schedule_non_resizable_attach_layout_sync(window: &WebviewWindow) {
        let scheduled_window = window.clone();
        tauri::async_runtime::spawn(async move {
            let callback_window = scheduled_window.clone();
            let _ = scheduled_window.run_on_main_thread(move || {
                let Some((view, host_view_ptr, plugin_can_resize)) = active_session_view_state()
                else {
                    return;
                };

                if plugin_can_resize {
                    return;
                }

                let Ok(ns_window) = callback_window.ns_window() else {
                    return;
                };
                let Some(ns_window) = (unsafe { Retained::retain(ns_window as *mut NSWindow) }) else {
                    return;
                };
                let Some(content_view) = ns_window.contentView() else {
                    return;
                };

                let host_view = host_view_ptr as *mut NSView;
                if host_view.is_null() {
                    return;
                }

                let host_frame = host_frame_for_content_frame(content_view.frame());
                unsafe {
                    (*host_view).setFrame(host_frame);
                    sync_host_container_subviews_to_bounds(&*host_view);
                }
                let mut rect = view_rect_for_host_frame(host_frame);
                let _ = unsafe { view.onSize(&mut rect) };
            });
        });
    }

    fn apply_deferred_host_resize(
        window: &WebviewWindow,
        width: f64,
        height: f64,
    ) -> Result<(), String> {

        let resize_window = window.clone();
        let resize_window_for_callback = resize_window.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        let schedule_result = resize_window.run_on_main_thread(move || {
            let result = (|| -> Result<(), String> {
                let ns_window = resize_window_for_callback
                    .ns_window()
                    .map_err(|error| format!("Failed to get host NSWindow: {error}"))?
                    as *mut NSWindow;
                let Some(ns_window) = (unsafe { Retained::retain(ns_window) }) else {
                    return Err(String::from("Failed to retain host NSWindow"));
                };

                let target_content_size = NSSize::new(width, height + HOST_HEADER_HEIGHT);
                let _: () = unsafe { msg_send![&*ns_window, setContentSize: target_content_size] };
                if let Some(content_view) = ns_window.contentView() {
                    sync_host_container_frame_to_content_view(&content_view);
                }
                Ok(())
            })();

            let _ = sender.send(result);
        });

        let resize_result = match schedule_result {
            Ok(()) => {
                let wait_started_at = log_main_thread_wait_start("deferred_host_resize");
                let received = receiver
                    .recv()
                    .map_err(|error| format!("Failed to receive host resize result: {error}"))
                    .and_then(|result| result);
                log_main_thread_wait_finish("deferred_host_resize", wait_started_at);
                received
            }
            Err(error) => Err(format!(
                "Failed to schedule host resize on the main thread: {error}"
            )),
        };

        match resize_result {
            Ok(()) => {
                pending_plugin_requested_resize().store(true, Ordering::Release);
                let followup_window = window.clone();
                if let Err(_error) = followup_window.run_on_main_thread(move || {
                }) {
                }
                let delayed_window = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(50));
                    let delayed_callback_window = delayed_window.clone();
                    if let Err(_error) = delayed_window.run_on_main_thread(move || {
                        let content_size = delayed_callback_window
                            .ns_window()
                            .ok()
                            .and_then(|ns_window| {
                                unsafe { Retained::retain(ns_window as *mut NSWindow) }
                            })
                            .and_then(|ns_window| {
                                ns_window.contentView().map(|content_view| {
                                    let frame = content_view.frame();
                                    (frame.size.width, frame.size.height)
                                })
                            });

                        match content_size {
                            Some((_current_width, _current_height)) => {}
                            None => {}
                        }
                    }) {
                    }
                });
                Ok(())
            }
            Err(error) => {
                Err(error)
            }
        }
    }

    fn flush_queued_plugin_resize(window: &WebviewWindow) -> Result<(), String> {
        loop {
            let next_resize = queued_plugin_requested_resize()
                .lock()
                .ok()
                .and_then(|mut queued_resize| queued_resize.take());

            let Some((width, height)) = next_resize else {
                plugin_requested_resize_dispatch_in_flight().store(false, Ordering::Release);

                let should_restart = queued_plugin_requested_resize()
                    .lock()
                    .ok()
                    .map(|queued_resize| queued_resize.is_some())
                    .unwrap_or(false);
                if should_restart
                    && plugin_requested_resize_dispatch_in_flight()
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                {
                    continue;
                }
                return Ok(());
            };

            apply_deferred_host_resize(window, width, height)?;
        }
    }

    fn dispatch_queued_plugin_resize(window: WebviewWindow) {
        tauri::async_runtime::spawn(async move {
            let _ = flush_queued_plugin_resize(&window);
        });
    }

    fn queue_plugin_requested_resize(
        window: &WebviewWindow,
        width: f64,
        height: f64,
        dispatch_immediately: bool,
    ) {
        if let Ok(mut queued_resize) = queued_plugin_requested_resize().lock() {
            *queued_resize = Some((width, height));
        } else {
            return;
        }

        if !dispatch_immediately {
            return;
        }

        let already_dispatching = plugin_requested_resize_dispatch_in_flight()
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err();

        if already_dispatching {
            return;
        }

        dispatch_queued_plugin_resize(window.clone());
    }

    fn dispatch_buffered_plugin_resize_if_needed(window: &WebviewWindow) -> Result<(), String> {
        let has_queued_resize = queued_plugin_requested_resize()
            .lock()
            .ok()
            .map(|queued_resize| queued_resize.is_some())
            .unwrap_or(false);
        if !has_queued_resize {
            return Ok(());
        }

        let already_dispatching = plugin_requested_resize_dispatch_in_flight()
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err();
        if already_dispatching {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(BUFFERED_REATTACH_RESIZE_DELAY_MS));
        flush_queued_plugin_resize(window)
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

    fn host_supports_interface(iid: &TUID) -> bool {
        *iid == IMessage_iid
            || *iid == IAttributeList_iid
            || *iid == IAutomationState_iid
            || *iid == IComponentHandler_iid
            || *iid == IComponentHandler2_iid
            || *iid == IComponentHandler3_iid
            || *iid == IComponentHandlerBusActivation_iid
            || *iid == IComponentHandlerSystemTime_iid
            || *iid == IContextMenuTarget_iid
            || *iid == IDataExchangeHandler_iid
            || *iid == IEditControllerHostEditing_iid
            || *iid == IHostApplication_iid
            || *iid == IPlugInterfaceSupport_iid
            || *iid == IProgress_iid
            || *iid == IRunLoop_iid
            || *iid == IUnitHandler_iid
            || *iid == IUnitHandler2_iid
            || *iid == FUnknown_iid
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
            window: &WebviewWindow,
            parent_view: &NSView,
            host_frame: NSRect,
        ) -> Result<(), PluginEditorHostAttemptError> {
            let plug_frame = ComWrapper::new(PluginHostFrame {
                window: window.clone(),
            })
            .to_com_ptr::<IPlugFrame>()
            .ok_or_else(|| {
                PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::ViewCreate,
                    "Failed to allocate the VST3 plug-frame shim.",
                )
            })?;
            let view = unsafe { ComPtr::from_raw(self.controller.createView(ViewType::kEditor)) }
                .ok_or_else(|| {
                    PluginEditorHostAttemptError::new(
                        PluginEditorHostValidationPhase::ViewCreate,
                        "Plugin editor POC did not receive a plugin editor view. The selected VST3 may not expose an editor.",
                    )
                })?;

            let platform_supported = unsafe { view.isPlatformTypeSupported(kPlatformTypeNSView) };
            if platform_supported != kResultOk {
                return Err(PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::ViewCreate,
                    "Plugin editor POC editor view does not report NSView support.",
                ));
            }

            let _ = unsafe { view.setFrame(plug_frame.as_ptr()) };

            let parent_ptr = (parent_view as *const NSView).cast_mut().cast::<c_void>();
            let attached = unsafe { view.attached(parent_ptr, kPlatformTypeNSView) };
            if attached != kResultOk {
                return Err(PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::ViewAttach,
                    format!(
                        "Plugin editor POC failed to attach the VST3 editor view to NSView (tresult = {}).",
                        attached
                    ),
                ));
            }

            let mut view_rect = view_rect_for_host_frame(host_frame);
            let get_size_result = unsafe { view.getSize(&mut view_rect) };
            if get_size_result == kResultOk {
            }

            let plugin_can_resize = unsafe { view.canResize() } == kResultOk;
            if let Err(_error) = window.set_resizable(plugin_can_resize) {}

            if plugin_can_resize {
                let _ = unsafe { view.checkSizeConstraint(&mut view_rect) };
                let _ = unsafe { view.onSize(&mut view_rect) };
            }

            sync_host_container_subviews_to_bounds(parent_view);

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
                    let _ = view.removed();
                    let _ = view.setFrame(std::ptr::null_mut());
                    self.view_attached = false;
                    self.view = None;
                    self._plug_frame = None;
                }
                self.close_prepared = true;
            }
        }

        fn finish_window_detach(&mut self) {
            self.host_view_ptr = 0;
            self.view_attached = false;
            self.close_prepared = true;
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

    pub(crate) fn should_notify_plugin_view_layout(
        reason: super::PluginHostSyncReason,
        plugin_can_resize: bool,
        plugin_requested_resize: bool,
    ) -> bool {
        match reason {
            super::PluginHostSyncReason::InitialAttach => true,
            super::PluginHostSyncReason::WindowFocusChanged => false,
            super::PluginHostSyncReason::WindowResized => {
                plugin_can_resize && !plugin_requested_resize
            }
        }
    }

    pub(crate) fn should_skip_initial_attach_layout_for_pending_resize(
        plugin_can_resize: bool,
    ) -> bool {
        !plugin_can_resize
            && (pending_plugin_requested_resize().load(Ordering::Acquire)
                || plugin_requested_resize_dispatch_in_flight().load(Ordering::Acquire)
                || queued_plugin_requested_resize()
                    .lock()
                    .ok()
                    .map(|queued_resize| queued_resize.is_some())
                    .unwrap_or(false))
    }

    pub(crate) fn should_accept_plugin_requested_resize(
        lifecycle_phase: super::PluginHostLifecyclePhase,
        plugin_can_resize: Option<bool>,
    ) -> bool {
        match lifecycle_phase {
            super::PluginHostLifecyclePhase::Reserved => true,
            super::PluginHostLifecyclePhase::Attached => {
                let _ = plugin_can_resize;
                true
            }
            super::PluginHostLifecyclePhase::Detached => true,
            super::PluginHostLifecyclePhase::Closing | super::PluginHostLifecyclePhase::Idle => {
                false
            }
        }
    }

    pub(crate) fn should_schedule_non_resizable_attach_layout_followup(
        lifecycle_phase: super::PluginHostLifecyclePhase,
        plugin_can_resize: Option<bool>,
    ) -> bool {
        plugin_can_resize != Some(true)
            && matches!(
                lifecycle_phase,
                super::PluginHostLifecyclePhase::Reserved
                    | super::PluginHostLifecyclePhase::Detached
            )
    }

    pub(crate) fn should_forward_plugin_focus(plugin_can_resize: bool) -> bool {
        plugin_can_resize
    }

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

    impl IHostApplicationTrait for PluginHostApplication {
        unsafe fn getName(&self, name: *mut String128) -> tresult {
            if name.is_null() {
                return kInvalidArgument;
            }

            write_string128(&mut *name, "Solo Compose Plugin Host POC");
            kResultOk
        }

        unsafe fn createInstance(
            &self,
            cid: *mut TUID,
            iid: *mut TUID,
            obj: *mut *mut c_void,
        ) -> tresult {
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
    }

    impl IPlugInterfaceSupportTrait for PluginHostApplication {
        unsafe fn isPlugInterfaceSupported(&self, iid: *const TUID) -> tresult {
            if iid.is_null() {
                return kInvalidArgument;
            }

            if host_supports_interface(&*iid) {
                kResultTrue
            } else {
                kResultFalse
            }
        }
    }

    impl IAutomationStateTrait for PluginHostApplication {
        unsafe fn setAutomationState(&self, _state: i32) -> tresult {
            kResultOk
        }
    }

    impl IRunLoopTrait for PluginHostApplication {
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

    impl IComponentHandlerTrait for PluginHostApplication {
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

    impl IComponentHandler2Trait for PluginHostApplication {
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

    impl IComponentHandler3Trait for PluginHostApplication {
        unsafe fn createContextMenu(
            &self,
            _plugView: *mut IPlugView,
            _paramID: *const Vst::ParamID,
        ) -> *mut IContextMenu {
            std::ptr::null_mut()
        }
    }

    impl IComponentHandlerBusActivationTrait for PluginHostApplication {
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

    impl IComponentHandlerSystemTimeTrait for PluginHostApplication {
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

    impl IContextMenuTargetTrait for PluginHostApplication {
        unsafe fn executeMenuItem(&self, _tag: i32) -> tresult {
            kResultOk
        }
    }

    impl IDataExchangeHandlerTrait for PluginHostApplication {
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

    impl IEditControllerHostEditingTrait for PluginHostApplication {
        unsafe fn beginEditFromHost(&self, _paramID: Vst::ParamID) -> tresult {
            kResultOk
        }

        unsafe fn endEditFromHost(&self, _paramID: Vst::ParamID) -> tresult {
            kResultOk
        }
    }

    impl IUnitHandlerTrait for PluginHostApplication {
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

    impl IUnitHandler2Trait for PluginHostApplication {
        unsafe fn notifyUnitByBusChange(&self) -> tresult {
            kResultOk
        }
    }

    impl IProgressTrait for PluginHostApplication {
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

    struct PluginHostFrame {
        window: WebviewWindow,
    }

    impl Class for PluginHostFrame {
        type Interfaces = (IPlugFrame,);
    }

    impl IPlugFrameTrait for PluginHostFrame {
        unsafe fn resizeView(&self, _view: *mut IPlugView, new_size: *mut ViewRect) -> tresult {
            if new_size.is_null() {
                return kInvalidArgument;
            }

            let new_size = *new_size;
            let width = (new_size.right - new_size.left).max(1) as f64;
            let height = (new_size.bottom - new_size.top).max(1) as f64;

            let lifecycle_phase = super::active_plugin_host_lifecycle()
                .lock()
                .map(|state| state.phase)
                .unwrap_or(super::PluginHostLifecyclePhase::Idle);
            let plugin_can_resize = active_session_plugin_can_resize();

            if !should_accept_plugin_requested_resize(lifecycle_phase, plugin_can_resize) {
                return kResultOk;
            }

            if should_schedule_non_resizable_attach_layout_followup(
                lifecycle_phase,
                plugin_can_resize,
            ) {
                pending_non_resizable_attach_layout_sync().store(true, Ordering::Release);
            }

            queue_plugin_requested_resize(
                &self.window,
                width,
                height,
                lifecycle_phase != super::PluginHostLifecyclePhase::Detached,
            );
            kResultOk
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
        let Some(factory3) = factory.cast::<IPluginFactory3>() else {
            return None;
        };

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

    fn connect_component_and_controller(
        component: &ComPtr<IComponent>,
        controller: &ComPtr<IEditController>,
    ) -> bool {
        let Some(component_connection) = component.cast::<IConnectionPoint>() else {
            return false;
        };
        let Some(controller_connection) = controller.cast::<IConnectionPoint>() else {
            return false;
        };

        let component_connect_result =
            unsafe { component_connection.connect(controller_connection.as_ptr()) };
        let controller_connect_result =
            unsafe { controller_connection.connect(component_connection.as_ptr()) };
        if component_connect_result == kResultOk && controller_connect_result == kResultOk {
            return true;
        }
        unsafe {
            let _ = controller_connection.disconnect(component_connection.as_ptr());
            let _ = component_connection.disconnect(controller_connection.as_ptr());
        }
        false
    }

    fn view_rect_for_host_frame(frame: NSRect) -> ViewRect {
        ViewRect {
            left: 0,
            top: 0,
            right: frame.size.width.round().max(0.0) as i32,
            bottom: frame.size.height.round().max(0.0) as i32,
        }
    }

    fn host_frame_for_content_frame(content_frame: NSRect) -> NSRect {
        NSRect::new(
            NSPoint::new(0.0, HOST_HEADER_HEIGHT),
            NSSize::new(
                content_frame.size.width,
                (content_frame.size.height - HOST_HEADER_HEIGHT).max(0.0),
            ),
        )
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

    fn sync_host_container_frame_to_content_view(content_view: &NSView) {
        let Some(host_view_ptr) = active_host_view_ptr() else {
            return;
        };

        let host_view = host_view_ptr as *mut NSView;
        if host_view.is_null() {
            return;
        }

        let host_frame = host_frame_for_content_frame(content_view.frame());
        unsafe {
            (*host_view).setFrame(host_frame);
            sync_host_container_subviews_to_bounds(&*host_view);
        }
    }

    fn build_plugin_host_session(
        target: &super::PluginEditorTarget,
        restorable_plugin_state: Option<&PlaybackSlotPluginState>,
    ) -> Result<PluginHostSession, PluginEditorHostAttemptError> {
        let plugin_path = target.path();
        let module = load_vst3_module(plugin_path).map_err(|error| {
            PluginEditorHostAttemptError::new(PluginEditorHostValidationPhase::ModuleLoad, error)
        })?;
        let factory = create_plugin_factory(&module).map_err(|error| {
            PluginEditorHostAttemptError::new(PluginEditorHostValidationPhase::FactoryCreate, error)
        })?;

        let host_application = ComWrapper::new(PluginHostApplication)
            .to_com_ptr::<IHostApplication>()
            .ok_or_else(|| {
                PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::HostContext,
                    "Failed to allocate the VST3 host-application shim.",
                )
            })?;
        let host_context = host_application.clone().upcast::<FUnknown>();
        configure_factory_host_context(&factory, &host_context);

        let component = create_component(&factory, plugin_path).map_err(|error| {
            PluginEditorHostAttemptError::new(
                PluginEditorHostValidationPhase::ComponentCreate,
                error,
            )
        })?;
        let component_init = unsafe { component.initialize(host_context.as_ptr()) };
        if component_init != kResultOk {
            return Err(PluginEditorHostAttemptError::new(
                PluginEditorHostValidationPhase::ComponentInitialize,
                format!(
                    "Plugin editor POC failed to initialize the VST3 component (tresult = {}).",
                    component_init
                ),
            ));
        }

        if let Some(plugin_state) = restorable_plugin_state {
            restore_component_state(&component, plugin_state).map_err(|error| {
                PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::ComponentStateSync,
                    error,
                )
            })?;
        }

        if let Some(processor) = component.cast::<Vst::IAudioProcessor>() {
            activate_component_buses(&component).map_err(|error| {
                PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::ComponentInitialize,
                    error,
                )
            })?;
            let mut setup = Vst::ProcessSetup {
                processMode: 0,        // kRealtime
                symbolicSampleSize: 0, // kSample32
                maxSamplesPerBlock: 1024,
                sampleRate: 44100.0,
            };
            let _ = unsafe { processor.setupProcessing(&mut setup) };
            let _ = unsafe { component.setActive(1) };
        } else {
        }

        let CreatedController {
            controller,
            shared_with_component: controller_shared_with_component,
        } = match create_controller(&factory, &component) {
            Ok(controller) => controller,
            Err(error) => {
                unsafe {
                    let _ = component.terminate();
                }
                return Err(PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::ControllerCreate,
                    error,
                ));
            }
        };

        // IComponentHandler is now on PluginHostApplication, so we use
        // the host_application ComPtr cast for setComponentHandler.

        // Set the component handler BEFORE controller.initialize().
        // HALion Sonic's controller may rely on having a valid handler
        // during initialization.
        let component_handler_ptr = host_application
            .clone()
            .upcast::<FUnknown>()
            .cast::<IComponentHandler>()
            .expect("PluginHostApplication must expose IComponentHandler");

        if !controller_shared_with_component {
            let _ = unsafe { controller.setComponentHandler(component_handler_ptr.as_ptr()) };
        }

        // Initialize the edit controller on the main thread.
        // The committed working version (ffaad55a) uses main-thread init
        // with pre-init setComponentHandler, 15 interfaces, and factory
        // host context.
        let controller_initialized = if controller_shared_with_component {
            false
        } else {
            let controller_init = unsafe { controller.initialize(host_context.as_ptr()) };
            if controller_init != kResultOk {
                unsafe {
                    let _ = component.terminate();
                }
                return Err(PluginEditorHostAttemptError::new(
                    PluginEditorHostValidationPhase::ControllerInitialize,
                    format!(
                        "Plugin editor POC failed to initialize the VST3 edit controller (tresult = {}).",
                        controller_init
                    ),
                ));
            }
            true
        };

        // Match original POC order exactly:
        // setComponentHandler → init → state sync → connect points

        // State sync before connect points
        if !controller_shared_with_component {
            sync_component_state_to_controller(&component, &controller);
        }

        let connection_points_connected = if controller_shared_with_component {
            false
        } else {
            connect_component_and_controller(&component, &controller)
        };

        Ok(PluginHostSession {
            target: target.clone(),
            plugin_path: plugin_path.to_path_buf(),
            restorable_plugin_state: restorable_plugin_state.cloned(),
            _module: module,
            component,
            controller,
            view: None,
            _host_application: host_application,
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
        if let Some(slot_id) = active_slot_id() {
            if let Ok(mut sessions) = slot_runtime_sessions().lock() {
                if let Some(session) = sessions.get_mut(&slot_id) {
                    session.prepare_for_window_close();
                    return;
                }
            }
        }

        if let Ok(mut active_session) = active_plugin_host_session().lock() {
            if let Some(session) = active_session.as_mut() {
                session.prepare_for_window_close();
            } else {
            }
        }
    }

    pub fn prepare_active_plugin_host_session_for_close(window: &WebviewWindow) {

        if MainThreadMarker::new().is_some() {
            prepare_plugin_host_session_for_close_now();
            return;
        }

        let cleanup_window = window.clone();
        if cleanup_window
            .run_on_main_thread(move || {
                prepare_plugin_host_session_for_close_now();
            })
            .is_err()
        {}
    }

    pub fn cleanup_active_plugin_host_session(window: &WebviewWindow) {
        if let Some(slot_id) = active_slot_id() {
            let close_prepared = slot_runtime_sessions()
                .lock()
                .ok()
                .and_then(|sessions| sessions.get(&slot_id).map(|session| session.close_prepared));

            match close_prepared {
                Some(true) => {
                    if let Ok(mut sessions) = slot_runtime_sessions().lock() {
                        if let Some(session) = sessions.get_mut(&slot_id) {
                            let target = session.target.clone();
                            session.finish_window_detach();
                            clear_pending_plugin_resize_request();
                            super::mark_plugin_target_detached(&target);
                            return;
                        }
                    }
                }
                Some(false) => {
                    let session = slot_runtime_sessions()
                        .lock()
                        .ok()
                        .and_then(|mut sessions| sessions.remove(&slot_id));
                    if let Some(session) = session {
                        let session_state = Arc::new(StdMutex::new(Some(session)));
                        let session_state_for_main = Arc::clone(&session_state);
                        let cleanup_window = window.clone();
                        if cleanup_window
                            .run_on_main_thread(move || {
                                if let Ok(mut session_state) = session_state_for_main.lock() {
                                    if let Some(session) = session_state.take() {
                                        finish_plugin_host_session_shutdown(session);
                                    }
                                }
                            })
                            .is_err()
                        {
                            clear_pending_plugin_resize_request();
                            super::clear_reserved_plugin_target();
                            if let Ok(mut session_state) = session_state.lock() {
                                if let Some(session) = session_state.take() {
                                    std::mem::forget(session);
                                }
                            }
                        }
                        return;
                    }
                }
                None => {}
            }
        }

        let session = active_plugin_host_session()
            .lock()
            .ok()
            .and_then(|mut active_session| active_session.take());

        let Some(session) = session else {
            clear_pending_plugin_resize_request();
            super::clear_reserved_plugin_target();
            return;
        };

        if session.close_prepared {
            let target = session.target.clone();
            if let Ok(mut active_session) = active_plugin_host_session().lock() {
                let mut detached_session = session;
                detached_session.finish_window_detach();
                *active_session = Some(detached_session);
            }
            clear_pending_plugin_resize_request();
            super::mark_plugin_target_detached(&target);
            return;
        }

        if MainThreadMarker::new().is_some() {
            finish_plugin_host_session_shutdown(session);
            return;
        }

        let cleanup_window = window.clone();
        let session_state = Arc::new(StdMutex::new(Some(session)));
        let session_state_for_main = Arc::clone(&session_state);
        if cleanup_window
            .run_on_main_thread(move || {
                if let Ok(mut session_state) = session_state_for_main.lock() {
                    if let Some(session) = session_state.take() {
                        finish_plugin_host_session_shutdown(session);
                    }
                }
            })
            .is_err()
        {
            clear_pending_plugin_resize_request();
            super::clear_reserved_plugin_target();
            if let Ok(mut session_state) = session_state.lock() {
                if let Some(session) = session_state.take() {
                    std::mem::forget(session);
                }
            }
        }
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

    pub fn sync_active_plugin_host(
        window: &WebviewWindow,
        focused: Option<bool>,
        reason: super::PluginHostSyncReason,
    ) {
        let Some((view, host_view_ptr, plugin_can_resize)) = active_session_view_state() else {
            return;
        };

        let scheduler_window = window.clone();
        let sync_window = window.clone();
        let _ = scheduler_window.run_on_main_thread(move || {
            let Ok(ns_window) = sync_window.ns_window() else {
                return;
            };
            let Some(ns_window) = (unsafe { Retained::retain(ns_window as *mut NSWindow) }) else {
                return;
            };
            let Some(content_view) = ns_window.contentView() else {
                return;
            };

            let host_view = host_view_ptr as *mut NSView;
            if host_view.is_null() {
                return;
            }

            let host_frame = host_frame_for_content_frame(content_view.frame());
            unsafe {
                (*host_view).setFrame(host_frame);
                sync_host_container_subviews_to_bounds(&*host_view);
            }

            let plugin_requested_resize =
                matches!(reason, super::PluginHostSyncReason::WindowResized)
                    && pending_plugin_requested_resize().swap(false, Ordering::AcqRel);
            let skip_initial_attach_layout =
                matches!(reason, super::PluginHostSyncReason::InitialAttach)
                    && should_skip_initial_attach_layout_for_pending_resize(plugin_can_resize);
            let schedule_non_resizable_attach_layout =
                matches!(reason, super::PluginHostSyncReason::WindowResized)
                    && !plugin_can_resize
                    && pending_non_resizable_attach_layout_sync().swap(false, Ordering::AcqRel);
            let forward_focus = focused.is_some() && should_forward_plugin_focus(plugin_can_resize);

            if schedule_non_resizable_attach_layout {
                schedule_non_resizable_attach_layout_sync(&sync_window);
            }

            if !skip_initial_attach_layout
                && should_notify_plugin_view_layout(reason, plugin_can_resize, plugin_requested_resize)
            {
                let mut rect = view_rect_for_host_frame(host_frame);
                let _ = unsafe { view.onSize(&mut rect) };
            }
            if forward_focus {
                let focused = focused.unwrap_or(false);
                let _ = unsafe { view.onFocus(if focused { 1 } else { 0 }) };
            }
        });
    }

    pub fn attach_native_host_surface(
        window: &WebviewWindow,
        target: &super::PluginEditorTarget,
    ) -> Result<PluginHostAttachOutcome, String> {
        clear_pending_plugin_resize_request();
        let (sender, receiver) = std::sync::mpsc::channel();
        let scheduler_window = window.clone();
        let attachment_window = window.clone();
        let target = target.clone();
        let plugin_path = target.path().to_path_buf();

        scheduler_window
            .run_on_main_thread(move || {
                let result = (|| -> Result<PluginHostAttachOutcome, String> {
                    let mtm = MainThreadMarker::new().ok_or_else(|| {
                        String::from("Plugin host attachment must run on the main thread")
                    })?;

                    let ns_window = attachment_window
                        .ns_window()
                        .map_err(|error| format!("Failed to get host NSWindow: {error}"))?
                        as *mut NSWindow;
                    let ns_window = unsafe { Retained::retain(ns_window) }
                        .ok_or_else(|| String::from("Failed to retain host NSWindow"))?;
                    let content_view = ns_window
                        .contentView()
                        .ok_or_else(|| String::from("Failed to resolve host contentView"))?;
                    let host_frame = host_frame_for_content_frame(content_view.frame());

                    let native_host_view: Retained<PluginHostContainerView> = unsafe {
                        msg_send![PluginHostContainerView::alloc(mtm), initWithFrame: host_frame]
                    };
                    native_host_view.setAutoresizingMask(
                        NSAutoresizingMaskOptions::ViewWidthSizable
                            | NSAutoresizingMaskOptions::ViewHeightSizable,
                    );
                    native_host_view.setAutoresizesSubviews(true);
                    content_view.addSubview(&native_host_view);

                    let attach_result = if let Some(slot_id) = target.slot_id {
                        let mut session = slot_runtime_sessions()
                            .lock()
                            .map_err(|_| String::from("Failed to read slot runtime session registry"))?
                            .remove(&slot_id)
                            .ok_or_else(|| {
                                format!(
                                    "Playback slot {} does not have a prepared live runtime session.",
                                    slot_id
                                )
                            })?;
                        let result = session.attach_view(
                            &attachment_window,
                            &native_host_view,
                            host_frame,
                        );
                        slot_runtime_sessions()
                            .lock()
                            .map_err(|_| String::from("Failed to restore slot runtime session"))?
                            .insert(slot_id, session);
                        result
                    } else {
                        let mut active_session = active_plugin_host_session().lock().map_err(|_| {
                            String::from("Failed to read active inventory plugin host session")
                        })?;
                        let mut session = match active_session.take() {
                            Some(session) => session,
                            None => build_plugin_host_session(&target, None)
                                .map_err(|error| error.message.clone())?,
                        };
                        let result = session.attach_view(
                            &attachment_window,
                            &native_host_view,
                            host_frame,
                        );
                        *active_session = Some(session);
                        result
                    };

                    if let Err(error) = attach_result {
                        attach_error_message(
                            mtm,
                            &native_host_view,
                            host_frame,
                            &plugin_path,
                            &error.message,
                        );
                        return Ok(PluginHostAttachOutcome {
                            attached: false,
                            validation: failed_editor_host_validation(error.phase, error.message),
                        });
                    }

                    super::mark_plugin_target_attached(&target);
                    sync_active_plugin_host(
                        &attachment_window,
                        Some(true),
                        super::PluginHostSyncReason::InitialAttach,
                    );

                    Ok(PluginHostAttachOutcome {
                        attached: true,
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
        let outcome = received?;
        let outcome = outcome?;
        if outcome.attached {
            dispatch_buffered_plugin_resize_if_needed(window)?;
        }
        Ok(outcome)
    }
}

#[cfg(target_os = "macos")]
use macos_host::{
    attach_native_host_surface as attach_native_host_surface_impl,
    capture_slot_runtime_plugin_state as capture_slot_runtime_plugin_state_impl,
    cleanup_active_plugin_host_session, prepare_active_plugin_host_session_for_close,
    render_slot_runtime_block as render_slot_runtime_block_impl,
    stop_slot_runtime_render as stop_slot_runtime_render_impl, sync_active_plugin_host,
    sync_slot_runtime_sessions as sync_slot_runtime_sessions_impl, PluginHostSession,
};

#[cfg(target_os = "macos")]
fn attach_native_host_surface(
    window: &WebviewWindow,
    target: &PluginEditorTarget,
) -> Result<PluginHostAttachOutcome, String> {
    attach_native_host_surface_impl(window, target)
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
    _window: &WebviewWindow,
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
    if let Err(reason) = ensure_editor_host_target_supported(target.path()) {
        queue_editor_host_validation_emit(
            app.clone(),
            target.path().to_path_buf(),
            blocked_editor_host_validation(reason.clone()),
        );
        return Err(reason);
    }

    let mut existing_window = app.get_webview_window(WINDOW_LABEL);
    release_detached_plugin_host_for_target_switch(&app, &target, existing_window.is_some())?;
    existing_window = app.get_webview_window(WINDOW_LABEL);

    match reserve_plugin_target(&target, existing_window.is_some())? {
        PluginTargetReservation::Acquired => {}
        PluginTargetReservation::AlreadyReserved(phase) => {
            if let Some(window) = existing_window {
                window
                    .set_focus()
                    .map_err(|error| format!("Failed to focus plugin host POC window: {error}"))?;

                return Ok(PluginEditorPocOpenResult {
                    window_label: WINDOW_LABEL.to_string(),
                    plugin_path: target.path().display().to_string(),
                    native_host_attached: live_plugin_host_attached(),
                });
            }

            if should_continue_reopening_reserved_target(phase, existing_window.is_some()) {
            } else {
                return Err(String::from(
                    "The plugin editor POC window is already opening for this target.",
                ));
            }
        }
    }

    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window
            .set_focus()
            .map_err(|error| format!("Failed to focus plugin host POC window: {error}"))?;

        return Ok(PluginEditorPocOpenResult {
            window_label: WINDOW_LABEL.to_string(),
            plugin_path: target.path().display().to_string(),
            native_host_attached: live_plugin_host_attached(),
        });
    }

    let window = match build_host_window(&app) {
        Ok(window) => window,
        Err(error) => {
            release_reserved_plugin_target(&target);
            return Err(error);
        }
    };

    let attach_outcome = match attach_native_host_surface(&window, &target) {
        Ok(outcome) => outcome,
        Err(error) => {
            release_reserved_plugin_target(&target);
            let _ = window.close();
            return Err(error);
        }
    };
    queue_editor_host_validation_emit(
        app.clone(),
        target.path().to_path_buf(),
        attach_outcome.validation,
    );

    window
        .show()
        .map_err(|error| format!("Failed to show plugin host POC window: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("Failed to focus plugin host POC window: {error}"))?;

    Ok(PluginEditorPocOpenResult {
        window_label: WINDOW_LABEL.to_string(),
        plugin_path: target.path().display().to_string(),
        native_host_attached: attach_outcome.attached,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        active_plugin_host_lifecycle, begin_plugin_target_close, clear_reserved_plugin_target,
        ensure_editor_host_target_supported, mark_plugin_target_attached,
        release_reserved_plugin_target, reserve_plugin_target, resolve_plugin_target_in_roots,
        PluginHostLifecyclePhase, PluginHostSyncReason, PluginTargetReservation,
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
    fn reserve_plugin_target_clears_stale_reservation_when_window_is_gone() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, true).unwrap();
        begin_plugin_target_close(Some(&target));

        let reservation = reserve_plugin_target(&target, false).unwrap();

        assert!(matches!(reservation, PluginTargetReservation::Acquired));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Reserved);

        reset_active_plugin_target();
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
    fn begin_plugin_target_close_marks_matching_target_only() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, false).unwrap();

        begin_plugin_target_close(Some(&slot_target("/tmp/other-plugin.vst3", 12)));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Reserved);

        begin_plugin_target_close(Some(&target));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Closing);

        reset_active_plugin_target();
    }

    #[test]
    fn plugin_host_layout_notifications_remain_enabled_for_attached_views() {
        assert!(super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::InitialAttach,
            false,
            false,
        ));
        assert!(!super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::WindowFocusChanged,
            true,
            false,
        ));
        assert!(!super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::WindowResized,
            false,
            false,
        ));
        assert!(super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::WindowResized,
            true,
            false,
        ));
        assert!(!super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::WindowResized,
            true,
            true,
        ));
    }

    #[test]
    fn plugin_requested_resize_is_accepted_for_attached_live_sessions() {
        assert!(super::macos_host::should_accept_plugin_requested_resize(
            PluginHostLifecyclePhase::Reserved,
            None,
        ));
        assert!(super::macos_host::should_accept_plugin_requested_resize(
            PluginHostLifecyclePhase::Attached,
            Some(false),
        ));
        assert!(super::macos_host::should_accept_plugin_requested_resize(
            PluginHostLifecyclePhase::Attached,
            Some(true),
        ));
        assert!(super::macos_host::should_accept_plugin_requested_resize(
            PluginHostLifecyclePhase::Detached,
            Some(false),
        ));
        assert!(!super::macos_host::should_accept_plugin_requested_resize(
            PluginHostLifecyclePhase::Closing,
            Some(true),
        ));
    }

    #[test]
    fn reserve_plugin_target_preserves_detached_target_without_window() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, true).unwrap();
        mark_plugin_target_attached(&target);
        super::mark_plugin_target_detached(&target);

        let reservation = reserve_plugin_target(&target, false).unwrap();

        assert!(matches!(
            reservation,
            PluginTargetReservation::AlreadyReserved(PluginHostLifecyclePhase::Detached)
        ));

        reset_active_plugin_target();
    }

    #[test]
    fn detached_reservation_without_window_allows_reopen_to_continue() {
        assert!(super::should_continue_reopening_reserved_target(
            PluginHostLifecyclePhase::Detached,
            false,
        ));
        assert!(!super::should_continue_reopening_reserved_target(
            PluginHostLifecyclePhase::Detached,
            true,
        ));
        assert!(!super::should_continue_reopening_reserved_target(
            PluginHostLifecyclePhase::Reserved,
            false,
        ));
    }

    #[test]
    fn release_reserved_plugin_target_clears_detached_slot_target_for_switch() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = slot_target("/tmp/test-plugin.vst3", 12);
        reserve_plugin_target(&target, true).unwrap();
        mark_plugin_target_attached(&target);
        super::mark_plugin_target_detached(&target);

        super::release_reserved_plugin_target(&target);

        assert_eq!(active_plugin_target(), None);
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Idle);
    }

    #[test]
    fn detached_slot_target_can_switch_to_same_bundle_different_slot_after_release() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let slot_1 = slot_target("/tmp/test-plugin.vst3", 12);
        let slot_2 = slot_target("/tmp/test-plugin.vst3", 13);

        reserve_plugin_target(&slot_1, true).unwrap();
        mark_plugin_target_attached(&slot_1);
        super::mark_plugin_target_detached(&slot_1);
        super::release_reserved_plugin_target(&slot_1);

        let reservation = reserve_plugin_target(&slot_2, false).unwrap();

        assert!(matches!(reservation, PluginTargetReservation::Acquired));
        assert_eq!(active_plugin_target(), Some(slot_2));

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

    #[test]
    fn initial_attach_layout_is_skipped_when_non_resizable_resize_is_pending() {
        super::macos_host::clear_pending_plugin_resize_request();
        assert!(!super::macos_host::should_skip_initial_attach_layout_for_pending_resize(false,));

        super::macos_host::pending_plugin_requested_resize()
            .store(true, std::sync::atomic::Ordering::Release);
        assert!(super::macos_host::should_skip_initial_attach_layout_for_pending_resize(false,));
        assert!(!super::macos_host::should_skip_initial_attach_layout_for_pending_resize(true,));
        super::macos_host::clear_pending_plugin_resize_request();
    }

    #[test]
    fn pending_non_resizable_attach_layout_flag_resets_with_resize_cleanup() {
        super::macos_host::clear_pending_plugin_resize_request();
        assert!(
            !super::macos_host::pending_non_resizable_attach_layout_sync()
                .load(std::sync::atomic::Ordering::Acquire)
        );

        super::macos_host::pending_non_resizable_attach_layout_sync()
            .store(true, std::sync::atomic::Ordering::Release);
        assert!(
            super::macos_host::pending_non_resizable_attach_layout_sync()
                .load(std::sync::atomic::Ordering::Acquire)
        );

        super::macos_host::clear_pending_plugin_resize_request();
        assert!(
            !super::macos_host::pending_non_resizable_attach_layout_sync()
                .load(std::sync::atomic::Ordering::Acquire)
        );
    }

    #[test]
    fn non_resizable_attach_layout_followup_is_only_scheduled_before_attached() {
        assert!(
            super::macos_host::should_schedule_non_resizable_attach_layout_followup(
                PluginHostLifecyclePhase::Reserved,
                Some(false),
            )
        );
        assert!(
            super::macos_host::should_schedule_non_resizable_attach_layout_followup(
                PluginHostLifecyclePhase::Detached,
                Some(false),
            )
        );
        assert!(
            !super::macos_host::should_schedule_non_resizable_attach_layout_followup(
                PluginHostLifecyclePhase::Attached,
                Some(false),
            )
        );
        assert!(
            !super::macos_host::should_schedule_non_resizable_attach_layout_followup(
                PluginHostLifecyclePhase::Attached,
                Some(true),
            )
        );
    }

    #[test]
    fn plugin_focus_is_only_forwarded_for_resizable_editors() {
        assert!(!super::macos_host::should_forward_plugin_focus(false));
        assert!(super::macos_host::should_forward_plugin_focus(true));
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
}
