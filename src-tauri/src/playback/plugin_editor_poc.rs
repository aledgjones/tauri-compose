#![allow(non_snake_case)]

use crate::playback::plugin_host_compatibility::bounded_editor_host_unsupported_reason;
use crate::playback::vst3::{
    default_vst3_search_roots, discover_first_valid_vst3_in_roots, validate_vst3_bundle,
    MACOS_VST3_SEARCH_ROOTS,
};
use crate::ManagedPluginInventoryContextState;
use serde::Serialize;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PluginHostLifecyclePhase {
    Idle,
    Reserved,
    Attached,
    Closing,
}

#[derive(Debug)]
struct PluginHostLifecycleState {
    target: Option<PathBuf>,
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
        && state.phase != PluginHostLifecyclePhase::Reserved
    {
        eprintln!(
            "plugin_editor_poc: clearing stale plugin host lifecycle reservation without a live host window"
        );
        reset_plugin_host_lifecycle(state);
    }
}

fn reserve_plugin_target(
    target: &Path,
    host_window_exists: bool,
) -> Result<PluginTargetReservation, String> {
    let mut lifecycle = active_plugin_host_lifecycle()
        .lock()
        .map_err(|_| String::from("Failed to read plugin editor POC target state"))?;

    clear_stale_plugin_host_lifecycle(&mut lifecycle, host_window_exists);

    if let Some(active_target_path) = lifecycle.target.as_deref() {
        if active_target_path != target {
            return Err(String::from(
                "The plugin editor POC window is already open for a different target. Close it before switching plugin paths.",
            ));
        }

        return Ok(PluginTargetReservation::AlreadyReserved(lifecycle.phase));
    }

    lifecycle.target = Some(target.to_path_buf());
    lifecycle.phase = PluginHostLifecyclePhase::Reserved;
    Ok(PluginTargetReservation::Acquired)
}

fn active_plugin_target_path() -> Option<PathBuf> {
    active_plugin_host_lifecycle()
        .lock()
        .ok()
        .and_then(|state| state.target.clone())
}

fn mark_plugin_target_attached(target: &Path) {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        if lifecycle.target.as_deref() == Some(target) {
            lifecycle.phase = PluginHostLifecyclePhase::Attached;
        }
    }
}

fn begin_plugin_target_close(target: Option<&Path>) {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        let should_close = match target {
            Some(target) => lifecycle.target.as_deref() == Some(target),
            None => lifecycle.target.is_some(),
        };

        if should_close {
            lifecycle.phase = PluginHostLifecyclePhase::Closing;
        }
    }
}

fn release_reserved_plugin_target(target: &Path) {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
        if lifecycle.target.as_deref() == Some(target) {
            reset_plugin_host_lifecycle(&mut lifecycle);
        }
    }
}

fn clear_reserved_plugin_target() {
    if let Ok(mut lifecycle) = active_plugin_host_lifecycle().lock() {
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

#[cfg(target_os = "macos")]
fn live_plugin_host_attached() -> bool {
    active_plugin_host_session()
        .lock()
        .ok()
        .and_then(|session| session.as_ref().map(|_| true))
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
fn live_plugin_host_attached() -> bool {
    false
}

fn resolve_requested_plugin_target(
    plugin_id: Option<&str>,
    explicit_plugin_path: Option<&str>,
    plugin_inventory_state: &ManagedPluginInventoryContextState,
) -> Result<Option<PathBuf>, String> {
    if let Some(plugin_id) = plugin_id.map(str::trim).filter(|value| !value.is_empty()) {
        let plugin_inventory = plugin_inventory_state
            .lock()
            .map_err(|_| String::from("Failed to read plugin inventory state"))?;
        let plugin_path = plugin_inventory.resolve_available_bundle_path(plugin_id)?;
        validate_vst3_bundle(&plugin_path).map_err(|error| error.message)?;
        return Ok(Some(plugin_path));
    }

    if let Some(path) = requested_plugin_target_candidate(explicit_plugin_path) {
        validate_vst3_bundle(&path).map_err(|error| error.message)?;
        return Ok(Some(path));
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
    plugin_id: Option<&str>,
    explicit_plugin_path: Option<&str>,
    plugin_inventory_state: &ManagedPluginInventoryContextState,
) -> Result<PathBuf, String> {
    let inventory_target =
        resolve_requested_plugin_target(plugin_id, None, plugin_inventory_state)?;
    resolve_plugin_target_in_roots(
        explicit_plugin_path,
        &default_vst3_search_roots(),
        inventory_target,
    )
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
        WindowEvent::Resized(size) => {
            eprintln!(
                "plugin_editor_poc: host window resized to {}x{}",
                size.width, size.height
            );
            #[cfg(target_os = "macos")]
            sync_active_plugin_host(window, None, PluginHostSyncReason::WindowResized);
        }
        WindowEvent::Focused(focused) => {
            eprintln!("plugin_editor_poc: host window focus changed: {}", focused);
            #[cfg(target_os = "macos")]
            sync_active_plugin_host(
                window,
                Some(*focused),
                PluginHostSyncReason::WindowFocusChanged,
            );
        }
        WindowEvent::CloseRequested { .. } => {
            eprintln!("plugin_editor_poc: host window close requested");
            begin_plugin_target_close(active_plugin_target_path().as_deref());
            #[cfg(target_os = "macos")]
            prepare_active_plugin_host_session_for_close(window);
        }
        WindowEvent::Destroyed => {
            clear_reserved_plugin_target();
            #[cfg(target_os = "macos")]
            cleanup_active_plugin_host_session(window);
            eprintln!("plugin_editor_poc: host window destroyed");
        }
        _ => {}
    }
}

#[cfg(target_os = "macos")]
mod macos_host {
    use super::{active_plugin_host_session, HOST_HEADER_HEIGHT};
    use crate::playback::vst3::vst3_bundle_binary_path;
    use core_foundation::base::{CFRelease, CFRetain, CFTypeRef, TCFType};
    use core_foundation::bundle::CFBundle;
    use core_foundation::string::CFString;
    use core_foundation::url::CFURL;
    use core_foundation_sys::bundle::{CFBundleLoadExecutable, CFBundleUnloadExecutable};
    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSAutoresizingMaskOptions, NSTextField, NSView, NSWindow};
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
    use std::ffi::{c_void, CStr, CString};
    use std::mem::zeroed;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex as StdMutex};
    use tauri::{LogicalSize, WebviewWindow};
    use vst3::Steinberg::IBStream_::IStreamSeekMode_::{kIBSeekCur, kIBSeekEnd, kIBSeekSet};
    use vst3::Steinberg::Linux::{
        FileDescriptor, IEventHandler, IRunLoop, IRunLoopTrait, IRunLoop_iid, ITimerHandler,
        TimerInterval,
    };
    use vst3::Steinberg::Vst::{
        self, IAttributeList, IAttributeListTrait, IAutomationState, IAutomationStateTrait,
        IComponent, IComponentHandler, IComponentHandler2, IComponentHandler2Trait,
        IComponentHandler3, IComponentHandler3Trait, IComponentHandlerBusActivation,
        IComponentHandlerBusActivationTrait, IComponentHandlerSystemTime,
        IComponentHandlerSystemTimeTrait, IComponentHandlerTrait, IComponentTrait, IComponent_iid,
        IConnectionPoint, IConnectionPointTrait, IContextMenu, IContextMenuTarget,
        IContextMenuTargetTrait, IDataExchangeHandler, IDataExchangeHandlerTrait, IEditController,
        IEditControllerHostEditing, IEditControllerHostEditingTrait, IEditControllerTrait,
        IEditController_iid, IHostApplication, IHostApplicationTrait, IMessage, IMessageTrait,
        IPlugInterfaceSupport, IPlugInterfaceSupportTrait, IProgress, IProgressTrait, IUnitHandler,
        IUnitHandler2, IUnitHandler2Trait, IUnitHandlerTrait, String128, TChar, ViewType,
    };
    use vst3::Steinberg::Vst::{
        IAttributeList_::AttrID,
        IAttributeList_iid, IAutomationState_iid, IComponentHandler2_iid, IComponentHandler3_iid,
        IComponentHandlerBusActivation_iid, IComponentHandlerSystemTime_iid, IComponentHandler_iid,
        IContextMenuTarget_iid, IDataExchangeHandler_iid, IEditControllerHostEditing_iid,
        IHostApplication_iid, IMessage_iid, IMidiLearn_iid, INoteExpressionController_iid,
        INoteExpressionPhysicalUIMapping_iid, IParameterFunctionName_iid,
        IPlugInterfaceSupport_iid,
        IProgress_::{ProgressType, ID as ProgressID},
        IProgress_iid, IUnitHandler2_iid, IUnitHandler_iid,
    };
    use vst3::Steinberg::{
        self, int64, kInvalidArgument, kNotImplemented, kPlatformTypeNSView, kResultFalse,
        kResultOk, kResultTrue, tresult, uint32, FIDString, FUnknown, FUnknown_iid, IBStream,
        IBStreamTrait, IPlugFrame, IPlugFrameTrait, IPlugView, IPlugViewTrait, IPluginBaseTrait,
        IPluginFactory, IPluginFactory2, IPluginFactory2Trait, IPluginFactory3,
        IPluginFactory3Trait, IPluginFactoryTrait, IUpdateHandlerTrait, IUpdateHandler_iid, TBool,
        ViewRect, TUID,
    };
    use vst3::{Class, ComPtr, ComWrapper};

    type GetPluginFactoryFn = unsafe extern "system" fn() -> *mut IPluginFactory;
    type BundleEntryFn = unsafe extern "system" fn(*mut c_void) -> bool;
    type BundleExitFn = unsafe extern "system" fn() -> bool;

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
        bytes: StdMutex<Vec<u8>>,
        cursor: StdMutex<usize>,
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

    fn format_tuid(iid: &TUID) -> String {
        iid.iter()
            .map(|byte| format!("{:02X}", *byte as u8))
            .collect::<Vec<_>>()
            .join("")
    }

    fn describe_host_interface(iid: &TUID) -> &'static str {
        if *iid == IMessage_iid {
            "IMessage"
        } else if *iid == IAttributeList_iid {
            "IAttributeList"
        } else if *iid == IComponentHandler_iid {
            "IComponentHandler"
        } else if *iid == IComponentHandler2_iid {
            "IComponentHandler2"
        } else if *iid == IComponentHandler3_iid {
            "IComponentHandler3"
        } else if *iid == IComponentHandlerBusActivation_iid {
            "IComponentHandlerBusActivation"
        } else if *iid == IComponentHandlerSystemTime_iid {
            "IComponentHandlerSystemTime"
        } else if *iid == IContextMenuTarget_iid {
            "IContextMenuTarget"
        } else if *iid == IDataExchangeHandler_iid {
            "IDataExchangeHandler"
        } else if *iid == IAutomationState_iid {
            "IAutomationState"
        } else if *iid == IEditControllerHostEditing_iid {
            "IEditControllerHostEditing"
        } else if *iid == IHostApplication_iid {
            "IHostApplication"
        } else if *iid == IMidiLearn_iid {
            "IMidiLearn"
        } else if *iid == INoteExpressionController_iid {
            "INoteExpressionController"
        } else if *iid == INoteExpressionPhysicalUIMapping_iid {
            "INoteExpressionPhysicalUIMapping"
        } else if *iid == IParameterFunctionName_iid {
            "IParameterFunctionName"
        } else if *iid == vst3::Steinberg::Vst::ChannelContext::IInfoListener_iid {
            "ChannelContext::IInfoListener"
        } else if *iid == IPlugInterfaceSupport_iid {
            "IPlugInterfaceSupport"
        } else if *iid == IProgress_iid {
            "IProgress"
        } else if *iid == IRunLoop_iid {
            "IRunLoop"
        } else if *iid == IUpdateHandler_iid {
            "IUpdateHandler"
        } else if *iid == IUnitHandler_iid {
            "IUnitHandler"
        } else if *iid == IUnitHandler2_iid {
            "IUnitHandler2"
        } else if *iid == FUnknown_iid {
            "FUnknown"
        } else {
            "unknown"
        }
    }

    fn describe_requested_interface(iid: *const TUID) -> String {
        if iid.is_null() {
            return String::from("null");
        }

        let iid = unsafe { &*iid };
        format!("{} ({})", describe_host_interface(iid), format_tuid(iid))
    }

    unsafe fn query_host_support_interface(
        instance: ComPtr<FUnknown>,
        iid: *mut TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        eprintln!(
            "plugin_editor_poc: host queryInterface requested {}",
            describe_requested_interface(iid.cast_const())
        );
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
        pub plugin_path: PathBuf,
        _module: LoadedVst3Module,
        component: ComPtr<IComponent>,
        controller: ComPtr<IEditController>,
        view: ComPtr<IPlugView>,
        _host_application: ComPtr<IHostApplication>,
        _component_handler: ComPtr<IComponentHandler>,
        _plug_frame: ComPtr<IPlugFrame>,
        host_view_ptr: usize,
        plugin_can_resize: bool,
        view_attached: bool,
        connection_points_connected: bool,
        component_initialized: bool,
        controller_initialized: bool,
        controller_shared_with_component: bool,
        close_prepared: bool,
    }

    impl PluginHostSession {
        fn prepare_for_window_close(&mut self) {
            unsafe {
                if self.close_prepared {
                    return;
                }

                if self.view_attached {
                    eprintln!(
                        "plugin_editor_poc: removing editor view for '{}'",
                        self.plugin_path.display()
                    );
                    let _ = self.view.removed();
                    let clear_frame_result = self.view.setFrame(std::ptr::null_mut());
                    eprintln!(
                        "plugin_editor_poc: clear editor frame result {} for '{}'",
                        clear_frame_result,
                        self.plugin_path.display()
                    );
                    self.view_attached = false;
                }
                self.close_prepared = true;
            }
        }

        fn shutdown(mut self) {
            self.prepare_for_window_close();

            unsafe {
                if self.controller_initialized && !self.controller_shared_with_component {
                    let clear_handler_result =
                        self.controller.setComponentHandler(std::ptr::null_mut());
                    eprintln!(
                        "plugin_editor_poc: clear component handler result {} for '{}'",
                        clear_handler_result,
                        self.plugin_path.display()
                    );
                }
                if self.connection_points_connected {
                    eprintln!(
                        "plugin_editor_poc: disconnecting component/controller connection points for '{}'",
                        self.plugin_path.display()
                    );
                    disconnect_component_and_controller(&self.component, &self.controller);
                    self.connection_points_connected = false;
                }
                if self.controller_initialized && !self.controller_shared_with_component {
                    eprintln!(
                        "plugin_editor_poc: terminating edit controller for '{}'",
                        self.plugin_path.display()
                    );
                    let _ = self.controller.terminate();
                    self.controller_initialized = false;
                }
                if self.component_initialized {
                    eprintln!(
                        "plugin_editor_poc: terminating component for '{}'",
                        self.plugin_path.display()
                    );
                    let _ = self.component.terminate();
                    self.component_initialized = false;
                }
            }
        }

        fn view_state(&self) -> (ComPtr<IPlugView>, usize, bool) {
            (
                self.view.clone(),
                self.host_view_ptr,
                self.plugin_can_resize,
            )
        }
    }

    pub(crate) fn should_notify_plugin_view_layout(
        _reason: super::PluginHostSyncReason,
        _plugin_can_resize: bool,
    ) -> bool {
        true
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

    impl IComponentHandlerTrait for PluginHostApplication {
        unsafe fn beginEdit(&self, id: Vst::ParamID) -> tresult {
            eprintln!("plugin_editor_poc: host beginEdit {}", id);
            kResultOk
        }

        unsafe fn performEdit(
            &self,
            id: Vst::ParamID,
            value_normalized: Vst::ParamValue,
        ) -> tresult {
            eprintln!(
                "plugin_editor_poc: host performEdit {} => {}",
                id, value_normalized
            );
            kResultOk
        }

        unsafe fn endEdit(&self, id: Vst::ParamID) -> tresult {
            eprintln!("plugin_editor_poc: host endEdit {}", id);
            kResultOk
        }

        unsafe fn restartComponent(&self, flags: i32) -> tresult {
            eprintln!("plugin_editor_poc: host restartComponent flags={}", flags);
            kResultOk
        }
    }

    impl IComponentHandler2Trait for PluginHostApplication {
        unsafe fn setDirty(&self, state: TBool) -> tresult {
            eprintln!("plugin_editor_poc: host setDirty state={}", state);
            kResultOk
        }

        unsafe fn requestOpenEditor(&self, _name: FIDString) -> tresult {
            eprintln!("plugin_editor_poc: host requestOpenEditor");
            kResultOk
        }

        unsafe fn startGroupEdit(&self) -> tresult {
            eprintln!("plugin_editor_poc: host startGroupEdit");
            kResultOk
        }

        unsafe fn finishGroupEdit(&self) -> tresult {
            eprintln!("plugin_editor_poc: host finishGroupEdit");
            kResultOk
        }
    }

    impl IComponentHandler3Trait for PluginHostApplication {
        unsafe fn createContextMenu(
            &self,
            _plugView: *mut IPlugView,
            _paramID: *const Vst::ParamID,
        ) -> *mut IContextMenu {
            eprintln!("plugin_editor_poc: host createContextMenu");
            std::ptr::null_mut()
        }
    }

    impl IComponentHandlerBusActivationTrait for PluginHostApplication {
        unsafe fn requestBusActivation(
            &self,
            r#type: Vst::MediaType,
            dir: Vst::BusDirection,
            index: i32,
            state: TBool,
        ) -> tresult {
            eprintln!(
                "plugin_editor_poc: host requestBusActivation type={} dir={} index={} state={}",
                r#type, dir, index, state
            );
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
            eprintln!("plugin_editor_poc: host getSystemTime => {}", now);
            kResultOk
        }
    }

    impl IAutomationStateTrait for PluginHostApplication {
        unsafe fn setAutomationState(&self, state: i32) -> tresult {
            eprintln!("plugin_editor_poc: host setAutomationState state={}", state);
            kResultOk
        }
    }

    impl IRunLoopTrait for PluginHostApplication {
        unsafe fn registerEventHandler(
            &self,
            handler: *mut IEventHandler,
            fd: FileDescriptor,
        ) -> tresult {
            if handler.is_null() {
                return kInvalidArgument;
            }

            eprintln!(
                "plugin_editor_poc: run-loop registerEventHandler fd={} accepted as a no-op shim",
                fd
            );
            kResultOk
        }

        unsafe fn unregisterEventHandler(&self, handler: *mut IEventHandler) -> tresult {
            if handler.is_null() {
                return kInvalidArgument;
            }

            eprintln!(
                "plugin_editor_poc: run-loop unregisterEventHandler accepted as a no-op shim"
            );
            kResultOk
        }

        unsafe fn registerTimer(
            &self,
            handler: *mut ITimerHandler,
            milliseconds: TimerInterval,
        ) -> tresult {
            if handler.is_null() {
                return kInvalidArgument;
            }

            eprintln!(
                "plugin_editor_poc: run-loop registerTimer interval={} accepted as a no-op shim",
                milliseconds
            );
            kResultOk
        }

        unsafe fn unregisterTimer(&self, handler: *mut ITimerHandler) -> tresult {
            if handler.is_null() {
                return kInvalidArgument;
            }

            eprintln!("plugin_editor_poc: run-loop unregisterTimer accepted as a no-op shim");
            kResultOk
        }
    }

    impl IContextMenuTargetTrait for PluginHostApplication {
        unsafe fn executeMenuItem(&self, tag: i32) -> tresult {
            eprintln!("plugin_editor_poc: host executeMenuItem tag={}", tag);
            kResultOk
        }
    }

    impl IDataExchangeHandlerTrait for PluginHostApplication {
        unsafe fn openQueue(
            &self,
            _processor: *mut Vst::IAudioProcessor,
            blockSize: uint32,
            numBlocks: uint32,
            alignment: uint32,
            userContextID: Vst::DataExchangeUserContextID,
            outID: *mut Vst::DataExchangeQueueID,
        ) -> tresult {
            eprintln!(
                "plugin_editor_poc: host openQueue blockSize={} numBlocks={} alignment={} userContextID={}",
                blockSize, numBlocks, alignment, userContextID
            );
            if !outID.is_null() {
                *outID = 0;
            }
            kResultOk
        }

        unsafe fn closeQueue(&self, queueID: Vst::DataExchangeQueueID) -> tresult {
            eprintln!("plugin_editor_poc: host closeQueue queueID={}", queueID);
            kResultOk
        }

        unsafe fn lockBlock(
            &self,
            queueId: Vst::DataExchangeQueueID,
            _block: *mut Vst::DataExchangeBlock,
        ) -> tresult {
            eprintln!("plugin_editor_poc: host lockBlock queueId={}", queueId);
            kResultOk
        }

        unsafe fn freeBlock(
            &self,
            queueId: Vst::DataExchangeQueueID,
            blockID: Vst::DataExchangeBlockID,
            sendToController: TBool,
        ) -> tresult {
            eprintln!(
                "plugin_editor_poc: host freeBlock queueId={} blockID={} sendToController={}",
                queueId, blockID, sendToController
            );
            kResultOk
        }
    }

    impl IEditControllerHostEditingTrait for PluginHostApplication {
        unsafe fn beginEditFromHost(&self, paramID: Vst::ParamID) -> tresult {
            eprintln!(
                "plugin_editor_poc: host beginEditFromHost paramID={}",
                paramID
            );
            kResultOk
        }

        unsafe fn endEditFromHost(&self, paramID: Vst::ParamID) -> tresult {
            eprintln!(
                "plugin_editor_poc: host endEditFromHost paramID={}",
                paramID
            );
            kResultOk
        }
    }

    impl IUnitHandlerTrait for PluginHostApplication {
        unsafe fn notifyUnitSelection(&self, unitId: Vst::UnitID) -> tresult {
            eprintln!(
                "plugin_editor_poc: host notifyUnitSelection unitId={}",
                unitId
            );
            kResultOk
        }

        unsafe fn notifyProgramListChange(
            &self,
            listId: Vst::ProgramListID,
            programIndex: i32,
        ) -> tresult {
            eprintln!(
                "plugin_editor_poc: host notifyProgramListChange listId={} programIndex={}",
                listId, programIndex
            );
            kResultOk
        }
    }

    impl IUnitHandler2Trait for PluginHostApplication {
        unsafe fn notifyUnitByBusChange(&self) -> tresult {
            eprintln!("plugin_editor_poc: host notifyUnitByBusChange");
            kResultOk
        }
    }

    impl IProgressTrait for PluginHostApplication {
        unsafe fn start(
            &self,
            r#type: ProgressType,
            _optionalDescription: *const vst3::Steinberg::tchar,
            outID: *mut ProgressID,
        ) -> tresult {
            eprintln!("plugin_editor_poc: host progress start type={}", r#type);
            if !outID.is_null() {
                *outID = 0;
            }
            kResultOk
        }

        unsafe fn update(&self, id: ProgressID, normValue: Vst::ParamValue) -> tresult {
            eprintln!(
                "plugin_editor_poc: host progress update id={} value={}",
                id, normValue
            );
            kResultOk
        }

        unsafe fn finish(&self, id: ProgressID) -> tresult {
            eprintln!("plugin_editor_poc: host progress finish id={}", id);
            kResultOk
        }
    }

    impl IUpdateHandlerTrait for PluginHostApplication {
        unsafe fn addDependent(
            &self,
            _object: *mut FUnknown,
            _dependent: *mut vst3::Steinberg::IDependent,
        ) -> tresult {
            eprintln!("plugin_editor_poc: host addDependent");
            kResultOk
        }

        unsafe fn removeDependent(
            &self,
            _object: *mut FUnknown,
            _dependent: *mut vst3::Steinberg::IDependent,
        ) -> tresult {
            eprintln!("plugin_editor_poc: host removeDependent");
            kResultOk
        }

        unsafe fn triggerUpdates(&self, _object: *mut FUnknown, message: i32) -> tresult {
            eprintln!("plugin_editor_poc: host triggerUpdates message={}", message);
            kResultOk
        }

        unsafe fn deferUpdates(&self, _object: *mut FUnknown, message: i32) -> tresult {
            eprintln!("plugin_editor_poc: host deferUpdates message={}", message);
            kResultOk
        }
    }

    impl IPlugInterfaceSupportTrait for PluginHostApplication {
        unsafe fn isPlugInterfaceSupported(&self, iid: *const TUID) -> tresult {
            if iid.is_null() {
                return kInvalidArgument;
            }

            eprintln!(
                "plugin_editor_poc: isPlugInterfaceSupported asked for {}",
                describe_requested_interface(iid)
            );

            if host_supports_interface(&*iid) {
                kResultTrue
            } else {
                kResultFalse
            }
        }
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

            eprintln!(
                "plugin_editor_poc: host createInstance cid={} iid={}",
                describe_requested_interface(cid.cast_const()),
                describe_requested_interface(iid.cast_const())
            );

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
            eprintln!(
                "plugin_editor_poc: plugin requested host-driven resize to {}x{}",
                width, height
            );

            match self.window.set_size(LogicalSize::new(width, height)) {
                Ok(()) => kResultOk,
                Err(error) => {
                    eprintln!("plugin_editor_poc: failed to apply host-driven resize: {error}");
                    kNotImplemented
                }
            }
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
    ) {
        let Some(factory3) = factory.cast::<IPluginFactory3>() else {
            eprintln!(
                "plugin_editor_poc: VST3 factory does not expose IPluginFactory3 host-context support"
            );
            return;
        };

        let host_context_result = unsafe { factory3.setHostContext(host_context.as_ptr()) };
        eprintln!(
            "plugin_editor_poc: IPluginFactory3::setHostContext result {}",
            host_context_result
        );
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
                    eprintln!(
                        "plugin_editor_poc: skipping non-component class '{}' in category '{}'",
                        name, category
                    );
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
                eprintln!(
                    "plugin_editor_poc: skipping non-component class '{}' in category '{}'",
                    class_info_string(&info.name),
                    class_info_string(&info.category)
                );
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

        if candidates.len() > 1 {
            eprintln!(
                "plugin_editor_poc: available audio-module classes for '{}' => {}",
                plugin_path.display(),
                candidates
                    .iter()
                    .map(|candidate| candidate.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        let candidate = select_component_class_candidate(&candidates, requested_name.as_deref())
            .ok_or_else(|| {
                String::from(
                    "Plugin editor POC could not find an audio-module component class in the VST3 factory.",
                )
            })?;

        eprintln!(
            "plugin_editor_poc: selected component class '{}' for '{}'",
            candidate.name,
            plugin_path.display()
        );

        eprintln!(
            "plugin_editor_poc: creating component instance '{}' for '{}'",
            candidate.name,
            plugin_path.display()
        );
        let mut raw_object: *mut c_void = std::ptr::null_mut();
        let result = unsafe {
            factory.createInstance(
                candidate.cid.as_ptr(),
                IComponent_iid.as_ptr(),
                &mut raw_object,
            )
        };
        eprintln!(
            "plugin_editor_poc: component createInstance result {} for '{}'",
            result, candidate.name
        );
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
            eprintln!(
                "plugin_editor_poc: component already exposes IEditController; reusing component as controller"
            );
            return Ok(CreatedController {
                controller,
                shared_with_component: true,
            });
        }

        let mut controller_cid: TUID = [0; 16];
        let result = unsafe { component.getControllerClassId(&mut controller_cid) };
        eprintln!(
            "plugin_editor_poc: getControllerClassId result {} with cid {}",
            result,
            format_tuid(&controller_cid)
        );
        if result != kResultOk || controller_cid.iter().all(|byte| *byte == 0) {
            return Err(String::from(
                "Plugin editor POC could not resolve a controller class ID for the VST3 component.",
            ));
        }

        eprintln!(
            "plugin_editor_poc: creating edit controller instance for controller cid {}",
            format_tuid(&controller_cid)
        );
        let mut raw_object: *mut c_void = std::ptr::null_mut();
        let create_result = unsafe {
            factory.createInstance(
                controller_cid.as_ptr(),
                IEditController_iid.as_ptr(),
                &mut raw_object,
            )
        };
        eprintln!(
            "plugin_editor_poc: edit controller createInstance result {}",
            create_result
        );
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

    fn create_memory_stream() -> Result<ComPtr<IBStream>, String> {
        ComWrapper::new(HostMemoryStream::default())
            .to_com_ptr::<IBStream>()
            .ok_or_else(|| String::from("Failed to allocate the VST3 memory stream shim."))
    }

    fn sync_component_state_to_controller(
        component: &ComPtr<IComponent>,
        controller: &ComPtr<IEditController>,
    ) {
        let Ok(stream) = create_memory_stream() else {
            eprintln!(
                "plugin_editor_poc: failed to allocate memory stream for controller state sync"
            );
            return;
        };

        let get_state_result = unsafe { component.getState(stream.as_ptr()) };
        if get_state_result != kResultOk {
            eprintln!(
                "plugin_editor_poc: component did not provide transferable state for the controller (tresult = {})",
                get_state_result
            );
            return;
        }

        let mut rewind_position = 0_i64;
        let rewind_result = unsafe { stream.seek(0, kIBSeekSet as i32, &mut rewind_position) };
        if rewind_result != kResultOk {
            eprintln!(
                "plugin_editor_poc: failed to rewind component state stream before controller sync (tresult = {})",
                rewind_result
            );
            return;
        }

        let set_state_result = unsafe { controller.setComponentState(stream.as_ptr()) };
        eprintln!(
            "plugin_editor_poc: controller state sync result {} after component state export {}",
            set_state_result, get_state_result
        );
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
            eprintln!("plugin_editor_poc: component does not expose IConnectionPoint");
            return false;
        };
        let Some(controller_connection) = controller.cast::<IConnectionPoint>() else {
            eprintln!("plugin_editor_poc: controller does not expose IConnectionPoint");
            return false;
        };

        let component_connect_result =
            unsafe { component_connection.connect(controller_connection.as_ptr()) };
        let controller_connect_result =
            unsafe { controller_connection.connect(component_connection.as_ptr()) };
        if component_connect_result == kResultOk && controller_connect_result == kResultOk {
            eprintln!("plugin_editor_poc: connected component and controller connection points");
            return true;
        }

        eprintln!(
            "plugin_editor_poc: component/controller connection-point handshake failed (component = {}, controller = {})",
            component_connect_result,
            controller_connect_result
        );
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

    fn build_plugin_host_session(
        window: &WebviewWindow,
        plugin_path: &Path,
        parent_view: &NSView,
        host_frame: NSRect,
    ) -> Result<PluginHostSession, String> {
        let module = load_vst3_module(plugin_path)?;
        let factory = create_plugin_factory(&module)?;

        let host_application = ComWrapper::new(PluginHostApplication)
            .to_com_ptr::<IHostApplication>()
            .ok_or_else(|| String::from("Failed to allocate the VST3 host-application shim."))?;
        let host_context = host_application.clone().upcast::<FUnknown>();
        configure_factory_host_context(&factory, &host_context);

        let component = create_component(&factory, plugin_path)?;
        eprintln!(
            "plugin_editor_poc: initializing component for '{}'",
            plugin_path.display()
        );
        let component_init = unsafe { component.initialize(host_context.as_ptr()) };
        eprintln!(
            "plugin_editor_poc: component initialize result {} for '{}'",
            component_init,
            plugin_path.display()
        );
        if component_init != kResultOk {
            return Err(format!(
                "Plugin editor POC failed to initialize the VST3 component (tresult = {}).",
                component_init
            ));
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
                return Err(error);
            }
        };

        let component_handler = host_application
            .clone()
            .cast::<IComponentHandler>()
            .ok_or_else(|| {
                String::from("Failed to expose the VST3 host application as a component handler.")
            })?;

        let controller_initialized = if controller_shared_with_component {
            eprintln!("plugin_editor_poc: component also implements the edit controller interface");
            false
        } else {
            let pre_init_handler_result =
                unsafe { controller.setComponentHandler(component_handler.as_ptr()) };
            eprintln!(
                "plugin_editor_poc: pre-initialize setComponentHandler result {} for '{}'",
                pre_init_handler_result,
                plugin_path.display()
            );
            eprintln!(
                "plugin_editor_poc: initializing edit controller for '{}'",
                plugin_path.display()
            );
            let controller_init = unsafe { controller.initialize(host_context.as_ptr()) };
            eprintln!(
                "plugin_editor_poc: edit controller initialize result {} for '{}'",
                controller_init,
                plugin_path.display()
            );
            if controller_init != kResultOk {
                unsafe {
                    let _ = component.terminate();
                }
                return Err(format!(
                    "Plugin editor POC failed to initialize the VST3 edit controller (tresult = {}).",
                    controller_init
                ));
            }
            true
        };

        let cleanup_initialized_plugin = |connection_points_connected: bool| unsafe {
            if connection_points_connected {
                disconnect_component_and_controller(&component, &controller);
            }
            if controller_initialized && !controller_shared_with_component {
                let _ = controller.terminate();
            }
            let _ = component.terminate();
        };

        if !controller_shared_with_component {
            eprintln!(
                "plugin_editor_poc: syncing component state to controller for '{}'",
                plugin_path.display()
            );
            sync_component_state_to_controller(&component, &controller);
        }
        let set_handler_result =
            unsafe { controller.setComponentHandler(component_handler.as_ptr()) };
        eprintln!(
            "plugin_editor_poc: setComponentHandler result {} for '{}'",
            set_handler_result,
            plugin_path.display()
        );

        let connection_points_connected = if controller_shared_with_component {
            false
        } else {
            eprintln!(
                "plugin_editor_poc: connecting component/controller connection points for '{}'",
                plugin_path.display()
            );
            connect_component_and_controller(&component, &controller)
        };

        let plug_frame = ComWrapper::new(PluginHostFrame {
            window: window.clone(),
        })
        .to_com_ptr::<IPlugFrame>()
        .ok_or_else(|| String::from("Failed to allocate the VST3 plug-frame shim."))?;

        eprintln!(
            "plugin_editor_poc: creating editor view for '{}'",
            plugin_path.display()
        );
        let view = unsafe { ComPtr::from_raw(controller.createView(ViewType::kEditor)) }
            .ok_or_else(|| {
                cleanup_initialized_plugin(connection_points_connected);
                String::from(
                    "Plugin editor POC did not receive a plugin editor view. The selected VST3 may not expose an editor.",
                )
            })?;
        eprintln!(
            "plugin_editor_poc: created editor view for '{}'",
            plugin_path.display()
        );

        let platform_supported = unsafe { view.isPlatformTypeSupported(kPlatformTypeNSView) };
        eprintln!(
            "plugin_editor_poc: isPlatformTypeSupported(NSView) result {} for '{}'",
            platform_supported,
            plugin_path.display()
        );
        if platform_supported != kResultOk {
            cleanup_initialized_plugin(connection_points_connected);
            return Err(String::from(
                "Plugin editor POC editor view does not report NSView support.",
            ));
        }

        let set_frame_result = unsafe { view.setFrame(plug_frame.as_ptr()) };
        eprintln!(
            "plugin_editor_poc: setFrame result {} for '{}'",
            set_frame_result,
            plugin_path.display()
        );

        let parent_ptr = (parent_view as *const NSView).cast_mut().cast::<c_void>();
        eprintln!(
            "plugin_editor_poc: attaching editor view to NSView for '{}'",
            plugin_path.display()
        );
        let attached = unsafe { view.attached(parent_ptr, kPlatformTypeNSView) };
        eprintln!(
            "plugin_editor_poc: view.attached result {} for '{}'",
            attached,
            plugin_path.display()
        );
        if attached != kResultOk {
            cleanup_initialized_plugin(connection_points_connected);
            return Err(format!(
                "Plugin editor POC failed to attach the VST3 editor view to NSView (tresult = {}).",
                attached
            ));
        }

        let mut view_rect = view_rect_for_host_frame(host_frame);
        let get_size_result = unsafe { view.getSize(&mut view_rect) };
        if get_size_result == kResultOk {
            eprintln!(
                "plugin_editor_poc: plugin reported editor size {}x{}",
                view_rect.right - view_rect.left,
                view_rect.bottom - view_rect.top
            );
        }

        let plugin_can_resize = unsafe { view.canResize() } == kResultOk;
        if let Err(error) = window.set_resizable(plugin_can_resize) {
            eprintln!(
                "plugin_editor_poc: failed to apply plugin resizable state {}: {}",
                plugin_can_resize, error
            );
        }
        eprintln!(
            "plugin_editor_poc: plugin reported resizable editor = {}",
            plugin_can_resize
        );

        if plugin_can_resize {
            let _ = unsafe { view.checkSizeConstraint(&mut view_rect) };
            let _ = unsafe { view.onSize(&mut view_rect) };
        }

        Ok(PluginHostSession {
            plugin_path: plugin_path.to_path_buf(),
            _module: module,
            component,
            controller,
            view,
            _host_application: host_application,
            _component_handler: component_handler,
            _plug_frame: plug_frame,
            host_view_ptr: parent_ptr as usize,
            plugin_can_resize,
            view_attached: true,
            connection_points_connected,
            component_initialized: true,
            controller_initialized,
            controller_shared_with_component,
            close_prepared: false,
        })
    }

    pub fn prepare_active_plugin_host_session_for_close(window: &WebviewWindow) {
        let cleanup_window = window.clone();
        if cleanup_window
            .run_on_main_thread(move || {
                if let Ok(mut active_session) = active_plugin_host_session().lock() {
                    if let Some(session) = active_session.as_mut() {
                        eprintln!(
                            "plugin_editor_poc: preparing active plugin host for close {}",
                            session.plugin_path.display()
                        );
                        session.prepare_for_window_close();
                    }
                }
            })
            .is_err()
        {
            eprintln!(
                "plugin_editor_poc: failed to schedule close preparation on the main thread; deferring native close preparation"
            );
        }
    }

    pub fn cleanup_active_plugin_host_session(window: &WebviewWindow) {
        let session = active_plugin_host_session()
            .lock()
            .ok()
            .and_then(|mut active_session| active_session.take());

        let Some(session) = session else {
            return;
        };

        let cleanup_window = window.clone();
        let session_state = Arc::new(StdMutex::new(Some(session)));
        let session_state_for_main = Arc::clone(&session_state);
        if cleanup_window
            .run_on_main_thread(move || {
                if let Ok(mut session_state) = session_state_for_main.lock() {
                    if let Some(session) = session_state.take() {
                        eprintln!(
                            "plugin_editor_poc: tearing down active plugin host for {}",
                            session.plugin_path.display()
                        );
                        session.shutdown();
                    }
                }
            })
            .is_err()
        {
            eprintln!(
                "plugin_editor_poc: failed to schedule native host teardown on the main thread; leaking session handles instead of tearing down off-thread"
            );
            if let Ok(mut session_state) = session_state.lock() {
                if let Some(session) = session_state.take() {
                    std::mem::forget(session);
                }
            }
        }
    }

    pub fn sync_active_plugin_host(
        window: &WebviewWindow,
        focused: Option<bool>,
        reason: super::PluginHostSyncReason,
    ) {
        let Some((view, host_view_ptr, plugin_can_resize)) = active_plugin_host_session()
            .lock()
            .ok()
            .and_then(|active_session| active_session.as_ref().map(PluginHostSession::view_state))
        else {
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
            }

            if should_notify_plugin_view_layout(reason, plugin_can_resize) {
                let mut rect = view_rect_for_host_frame(host_frame);
                let _ = unsafe { view.onSize(&mut rect) };
            }
            if let Some(focused) = focused {
                let _ = unsafe { view.onFocus(if focused { 1 } else { 0 }) };
            }
        });
    }

    pub fn attach_native_host_surface(
        window: &WebviewWindow,
        plugin_path: &Path,
    ) -> Result<bool, String> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let scheduler_window = window.clone();
        let attachment_window = window.clone();
        let plugin_path = plugin_path.to_path_buf();

        scheduler_window
            .run_on_main_thread(move || {
                let result = (|| -> Result<bool, String> {
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

                    let native_host_view = NSView::initWithFrame(NSView::alloc(mtm), host_frame);
                    native_host_view.setAutoresizingMask(
                        NSAutoresizingMaskOptions::ViewWidthSizable
                            | NSAutoresizingMaskOptions::ViewHeightSizable,
                    );
                    content_view.addSubview(&native_host_view);

                    let session = match build_plugin_host_session(
                        &attachment_window,
                        &plugin_path,
                        &native_host_view,
                        host_frame,
                    ) {
                        Ok(session) => session,
                        Err(error) => {
                            eprintln!(
                                "plugin_editor_poc: live VST3 attach failed for {}: {}",
                                plugin_path.display(),
                                error
                            );
                            attach_error_message(
                                mtm,
                                &native_host_view,
                                host_frame,
                                &plugin_path,
                                &error,
                            );
                            return Ok(false);
                        }
                    };
                    let session_target = session.plugin_path.clone();

                    if let Ok(mut active_session) = active_plugin_host_session().lock() {
                        *active_session = Some(session);
                    } else {
                        return Err(String::from("Failed to store plugin host session"));
                    }

                    super::mark_plugin_target_attached(&session_target);
                    sync_active_plugin_host(
                        &attachment_window,
                        Some(true),
                        super::PluginHostSyncReason::InitialAttach,
                    );

                    eprintln!(
                        "plugin_editor_poc: attached live VST3 editor host for {}",
                        plugin_path.display()
                    );

                    Ok(true)
                })();

                let _ = sender.send(result);
            })
            .map_err(|error| format!("Failed to schedule native host attachment: {error}"))?;

        receiver
            .recv()
            .map_err(|error| format!("Failed to receive native host attachment result: {error}"))?
    }
}

#[cfg(target_os = "macos")]
use macos_host::{
    attach_native_host_surface as attach_native_host_surface_impl,
    cleanup_active_plugin_host_session, prepare_active_plugin_host_session_for_close,
    sync_active_plugin_host, PluginHostSession,
};

#[cfg(target_os = "macos")]
fn attach_native_host_surface(window: &WebviewWindow, plugin_path: &Path) -> Result<bool, String> {
    attach_native_host_surface_impl(window, plugin_path)
}

#[cfg(not(target_os = "macos"))]
fn attach_native_host_surface(
    _window: &WebviewWindow,
    _plugin_path: &Path,
) -> Result<bool, String> {
    Err(String::from("plugin_editor_poc_open is macOS-only"))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_editor_poc_open(
    app: AppHandle,
    plugin_inventory_state: State<'_, ManagedPluginInventoryContextState>,
    plugin_id: Option<String>,
    plugin_path: Option<String>,
) -> Result<PluginEditorPocOpenResult, String> {
    let plugin_path = resolve_plugin_target(
        plugin_id.as_deref(),
        plugin_path.as_deref(),
        plugin_inventory_state.inner(),
    )?;
    ensure_editor_host_target_supported(&plugin_path)?;

    let existing_window = app.get_webview_window(WINDOW_LABEL);

    match reserve_plugin_target(&plugin_path, existing_window.is_some())? {
        PluginTargetReservation::Acquired => {}
        PluginTargetReservation::AlreadyReserved(_) => {
            if let Some(window) = existing_window {
                window
                    .set_focus()
                    .map_err(|error| format!("Failed to focus plugin host POC window: {error}"))?;

                return Ok(PluginEditorPocOpenResult {
                    window_label: WINDOW_LABEL.to_string(),
                    plugin_path: plugin_path.display().to_string(),
                    native_host_attached: live_plugin_host_attached(),
                });
            }

            return Err(String::from(
                "The plugin editor POC window is already opening for this target.",
            ));
        }
    }

    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window
            .set_focus()
            .map_err(|error| format!("Failed to focus plugin host POC window: {error}"))?;

        return Ok(PluginEditorPocOpenResult {
            window_label: WINDOW_LABEL.to_string(),
            plugin_path: plugin_path.display().to_string(),
            native_host_attached: live_plugin_host_attached(),
        });
    }

    let window = match build_host_window(&app) {
        Ok(window) => window,
        Err(error) => {
            release_reserved_plugin_target(&plugin_path);
            return Err(error);
        }
    };

    let native_host_attached = match attach_native_host_surface(&window, &plugin_path) {
        Ok(attached) => attached,
        Err(error) => {
            eprintln!(
                "plugin_editor_poc: fatal attach error for {}: {}",
                plugin_path.display(),
                error
            );
            release_reserved_plugin_target(&plugin_path);
            let _ = window.close();
            return Err(error);
        }
    };

    window
        .set_focus()
        .map_err(|error| format!("Failed to focus plugin host POC window: {error}"))?;

    Ok(PluginEditorPocOpenResult {
        window_label: WINDOW_LABEL.to_string(),
        plugin_path: plugin_path.display().to_string(),
        native_host_attached,
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
    #[cfg(target_os = "macos")]
    use std::ffi::{c_void, CStr, CString};
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

    fn active_plugin_target() -> Option<PathBuf> {
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

        let target = Path::new("/tmp/test-plugin.vst3");
        let reservation = reserve_plugin_target(target, false).unwrap();

        assert!(matches!(reservation, PluginTargetReservation::Acquired));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Reserved);

        reset_active_plugin_target();
    }

    #[test]
    fn reserve_plugin_target_reuses_matching_target() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = Path::new("/tmp/test-plugin.vst3");
        reserve_plugin_target(target, true).unwrap();

        let reservation = reserve_plugin_target(target, true).unwrap();

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

        reserve_plugin_target(Path::new("/tmp/test-plugin-a.vst3"), true).unwrap();

        let error = reserve_plugin_target(Path::new("/tmp/test-plugin-b.vst3"), true).unwrap_err();

        assert!(error.contains("already open for a different target"));

        reset_active_plugin_target();
    }

    #[test]
    fn release_reserved_plugin_target_clears_matching_target_only() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let active_target = Path::new("/tmp/test-plugin-a.vst3");
        reserve_plugin_target(active_target, true).unwrap();
        release_reserved_plugin_target(Path::new("/tmp/test-plugin-b.vst3"));

        assert_eq!(active_plugin_target().as_deref(), Some(active_target));

        release_reserved_plugin_target(active_target);

        assert_eq!(active_plugin_target().as_deref(), None);
        reset_active_plugin_target();
    }

    #[test]
    fn unsupported_editor_host_target_reason_rejects_steinberg_vendor_paths() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3");

        let reason = bounded_editor_host_unsupported_reason(path).unwrap();

        assert!(reason.contains("current unsupported list"));
        assert!(reason.contains("HALion Sonic"));
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
    fn unsupported_editor_host_target_reason_allows_non_unsupported_control_case() {
        let path = Path::new("/Library/Audio/Plug-Ins/VST3/BBC Symphony Orchestra.vst3");

        assert_eq!(bounded_editor_host_unsupported_reason(path), None);
    }

    #[test]
    fn ensure_editor_host_target_supported_rejects_without_touching_lifecycle_state() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let error = ensure_editor_host_target_supported(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3",
        ))
        .unwrap_err();

        assert!(error.contains("current unsupported list"));
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

        let target = Path::new("/tmp/test-plugin.vst3");
        reserve_plugin_target(target, true).unwrap();
        begin_plugin_target_close(Some(target));

        let reservation = reserve_plugin_target(target, false).unwrap();

        assert!(matches!(reservation, PluginTargetReservation::Acquired));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Reserved);

        reset_active_plugin_target();
    }

    #[test]
    fn reserve_plugin_target_keeps_in_flight_reservation_without_window() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = Path::new("/tmp/test-plugin.vst3");
        reserve_plugin_target(target, false).unwrap();

        let reservation = reserve_plugin_target(target, false).unwrap();

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

        let target = Path::new("/tmp/test-plugin.vst3");
        reserve_plugin_target(target, false).unwrap();

        mark_plugin_target_attached(target);

        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Attached);

        reset_active_plugin_target();
    }

    #[test]
    fn begin_plugin_target_close_marks_matching_target_only() {
        let _guard = lock_plugin_target_test_mutex();
        reset_active_plugin_target();

        let target = Path::new("/tmp/test-plugin.vst3");
        reserve_plugin_target(target, false).unwrap();

        begin_plugin_target_close(Some(Path::new("/tmp/other-plugin.vst3")));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Reserved);

        begin_plugin_target_close(Some(target));
        assert_eq!(active_plugin_phase(), PluginHostLifecyclePhase::Closing);

        reset_active_plugin_target();
    }

    #[test]
    fn plugin_host_layout_notifications_remain_enabled_for_attached_views() {
        assert!(super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::InitialAttach,
            false,
        ));
        assert!(super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::WindowFocusChanged,
            true,
        ));
        assert!(super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::WindowResized,
            false,
        ));
        assert!(super::macos_host::should_notify_plugin_view_layout(
            PluginHostSyncReason::WindowResized,
            true,
        ));
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
    fn host_application_create_instance_supports_messages_and_attributes() {
        use vst3::Steinberg::Vst::{
            IAttributeList, IAttributeListTrait, IAttributeList_iid, IHostApplication,
            IHostApplicationTrait, IMessage, IMessageTrait, IMessage_iid,
        };
        use vst3::Steinberg::{int64, kResultOk, FUnknown_iid};
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

        let message = unsafe { ComPtr::from_raw(raw_message.cast::<IMessage>()) }.unwrap();
        let message_id = CString::new("host-message").unwrap();
        unsafe {
            message.setMessageID(message_id.as_ptr());
        }

        let returned_id = unsafe { message.getMessageID() };
        assert_eq!(
            unsafe { CStr::from_ptr(returned_id) }.to_str().unwrap(),
            "host-message"
        );

        let attributes = unsafe { ComPtr::from_raw(message.getAttributes()) }.unwrap();
        let key = CString::new("host-attribute").unwrap();
        let mut value: int64 = 0;
        assert_eq!(unsafe { attributes.setInt(key.as_ptr(), 42) }, kResultOk);
        assert_eq!(
            unsafe { attributes.getInt(key.as_ptr(), &mut value) },
            kResultOk
        );
        assert_eq!(value, 42);

        let mut raw_attributes: *mut c_void = std::ptr::null_mut();
        let mut attribute_cid = IAttributeList_iid;
        let mut attribute_iid = IAttributeList_iid;
        let attribute_result = unsafe {
            host_application.createInstance(
                &mut attribute_cid,
                &mut attribute_iid,
                &mut raw_attributes,
            )
        };
        assert_eq!(attribute_result, kResultOk);
        assert!(unsafe { ComPtr::from_raw(raw_attributes.cast::<IAttributeList>()) }.is_some());

        let mut raw_unknown: *mut c_void = std::ptr::null_mut();
        let mut unknown_message_cid = IMessage_iid;
        let mut unknown_iid = FUnknown_iid;
        let unknown_result = unsafe {
            host_application.createInstance(
                &mut unknown_message_cid,
                &mut unknown_iid,
                &mut raw_unknown,
            )
        };
        assert_eq!(unknown_result, kResultOk);
        assert!(!raw_unknown.is_null());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_exposes_every_supported_interface() {
        use vst3::Steinberg::FUnknown;
        use vst3::Steinberg::IUpdateHandler;
        use vst3::Steinberg::Linux::IRunLoop;
        use vst3::Steinberg::Vst::{
            IAutomationState, IComponentHandler, IComponentHandler2, IComponentHandler3,
            IComponentHandlerBusActivation, IComponentHandlerSystemTime, IContextMenuTarget,
            IDataExchangeHandler, IEditControllerHostEditing, IHostApplication,
            IPlugInterfaceSupport, IProgress, IUnitHandler, IUnitHandler2,
        };
        use vst3::{ComPtr, ComWrapper};

        let host_application = ComWrapper::new(super::macos_host::PluginHostApplication)
            .to_com_ptr::<IHostApplication>()
            .unwrap();
        let unknown = host_application.clone().upcast::<FUnknown>();

        assert!(unknown.cast::<IHostApplication>().is_some());
        assert!(unknown.cast::<IPlugInterfaceSupport>().is_some());
        assert!(unknown.cast::<IRunLoop>().is_some());
        assert!(unknown.cast::<IAutomationState>().is_some());
        assert!(unknown.cast::<IComponentHandler>().is_some());
        assert!(unknown.cast::<IComponentHandler2>().is_some());
        assert!(unknown.cast::<IComponentHandler3>().is_some());
        assert!(unknown.cast::<IComponentHandlerBusActivation>().is_some());
        assert!(unknown.cast::<IComponentHandlerSystemTime>().is_some());
        assert!(unknown.cast::<IContextMenuTarget>().is_some());
        assert!(unknown.cast::<IDataExchangeHandler>().is_some());
        assert!(unknown.cast::<IEditControllerHostEditing>().is_some());
        assert!(unknown.cast::<IUnitHandler>().is_some());
        assert!(unknown.cast::<IUnitHandler2>().is_some());
        assert!(unknown.cast::<IProgress>().is_some());
        assert!(unknown.cast::<IUpdateHandler>().is_none());

        let _keep_alive: ComPtr<IHostApplication> = host_application;
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_application_reports_supported_interfaces() {
        use vst3::Steinberg::Linux::IRunLoop_iid;
        use vst3::Steinberg::Vst::{
            IAttributeList_iid, IAutomationState_iid, IComponentHandler2_iid,
            IComponentHandler3_iid, IComponentHandlerBusActivation_iid,
            IComponentHandlerSystemTime_iid, IComponentHandler_iid, IConnectionPoint_iid,
            IContextMenuTarget_iid, IDataExchangeHandler_iid, IEditControllerHostEditing_iid,
            IHostApplication_iid, IMessage_iid, IPlugInterfaceSupport, IPlugInterfaceSupportTrait,
            IPlugInterfaceSupport_iid, IProgress_iid, IUnitHandler2_iid, IUnitHandler_iid,
        };
        use vst3::Steinberg::{kResultFalse, kResultTrue, FUnknown_iid, IUpdateHandler_iid};
        use vst3::{ComPtr, ComWrapper};

        let host_support = ComWrapper::new(super::macos_host::PluginHostApplication)
            .to_com_ptr::<IPlugInterfaceSupport>()
            .unwrap();

        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IMessage_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IAttributeList_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IAutomationState_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IHostApplication_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IComponentHandler_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IComponentHandler2_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IComponentHandler3_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IComponentHandlerBusActivation_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IComponentHandlerSystemTime_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IContextMenuTarget_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IDataExchangeHandler_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IEditControllerHostEditing_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IPlugInterfaceSupport_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IProgress_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IRunLoop_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IUnitHandler_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IUnitHandler2_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IUpdateHandler_iid) },
            kResultFalse
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&FUnknown_iid) },
            kResultTrue
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&IConnectionPoint_iid) },
            kResultFalse
        );
        assert_eq!(
            unsafe { host_support.isPlugInterfaceSupported(&vst3::Steinberg::Vst::IMidiLearn_iid) },
            kResultFalse
        );
        assert_eq!(
            unsafe {
                host_support
                    .isPlugInterfaceSupported(&vst3::Steinberg::Vst::INoteExpressionController_iid)
            },
            kResultFalse
        );
        assert_eq!(
            unsafe {
                host_support.isPlugInterfaceSupported(
                    &vst3::Steinberg::Vst::INoteExpressionPhysicalUIMapping_iid,
                )
            },
            kResultFalse
        );
        assert_eq!(
            unsafe {
                host_support
                    .isPlugInterfaceSupported(&vst3::Steinberg::Vst::IParameterFunctionName_iid)
            },
            kResultFalse
        );
        assert_eq!(
            unsafe {
                host_support.isPlugInterfaceSupported(
                    &vst3::Steinberg::Vst::ChannelContext::IInfoListener_iid,
                )
            },
            kResultFalse
        );

        let raw_support = host_support.clone().into_raw();
        assert!(unsafe { ComPtr::from_raw(raw_support.cast::<IPlugInterfaceSupport>()) }.is_some());
    }
}
