#![allow(non_snake_case)]

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

#[derive(Debug)]
enum PluginTargetReservation {
    Acquired,
    AlreadyReserved,
}

fn active_plugin_target() -> &'static Mutex<Option<PathBuf>> {
    static ACTIVE_PLUGIN_TARGET: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    ACTIVE_PLUGIN_TARGET.get_or_init(|| Mutex::new(None))
}

fn reserve_plugin_target(target: &Path) -> Result<PluginTargetReservation, String> {
    let mut active_target = active_plugin_target()
        .lock()
        .map_err(|_| String::from("Failed to read plugin editor POC target state"))?;

    if let Some(active_target_path) = active_target.as_deref() {
        if active_target_path != target {
            return Err(String::from(
                "The plugin editor POC window is already open for a different target. Close it before switching plugin paths.",
            ));
        }

        return Ok(PluginTargetReservation::AlreadyReserved);
    }

    *active_target = Some(target.to_path_buf());
    Ok(PluginTargetReservation::Acquired)
}

fn release_reserved_plugin_target(target: &Path) {
    if let Ok(mut active_target) = active_plugin_target().lock() {
        if active_target.as_deref() == Some(target) {
            *active_target = None;
        }
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
            sync_active_plugin_host(window, None);
        }
        WindowEvent::Focused(focused) => {
            eprintln!("plugin_editor_poc: host window focus changed: {}", focused);
            #[cfg(target_os = "macos")]
            sync_active_plugin_host(window, Some(*focused));
        }
        WindowEvent::CloseRequested { .. } => {
            eprintln!("plugin_editor_poc: host window close requested");
            #[cfg(target_os = "macos")]
            cleanup_active_plugin_host_session(window);
        }
        WindowEvent::Destroyed => {
            if let Ok(mut active_target) = active_plugin_target().lock() {
                *active_target = None;
            }
            #[cfg(target_os = "macos")]
            cleanup_orphaned_plugin_host_session();
            eprintln!("plugin_editor_poc: host window destroyed");
        }
        _ => {}
    }
}

#[cfg(target_os = "macos")]
mod macos_host {
    use super::{active_plugin_host_session, HOST_HEADER_HEIGHT};
    use crate::playback::vst3::vst3_bundle_binary_path;
    use libloading::Library;
    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSAutoresizingMaskOptions, NSTextField, NSView, NSWindow};
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
    use std::ffi::c_void;
    use std::mem::zeroed;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex as StdMutex};
    use tauri::{LogicalSize, WebviewWindow};
    use vst3::Steinberg::IBStream_::IStreamSeekMode_::{kIBSeekCur, kIBSeekEnd, kIBSeekSet};
    use vst3::Steinberg::Vst::{
        self, IComponent, IComponentHandler, IComponentHandlerTrait, IComponentTrait,
        IComponent_iid, IConnectionPoint, IConnectionPointTrait, IEditController,
        IEditControllerTrait, IEditController_iid, IHostApplication, IHostApplicationTrait,
        String128, ViewType,
    };
    use vst3::Steinberg::{
        self, kInvalidArgument, kNotImplemented, kPlatformTypeNSView, kResultOk, tresult, FUnknown,
        IBStream, IBStreamTrait, IPlugFrame, IPlugFrameTrait, IPlugView, IPlugViewTrait,
        IPluginBaseTrait, IPluginFactory, IPluginFactoryTrait, ViewRect, TUID,
    };
    use vst3::{Class, ComPtr, ComWrapper};

    type GetPluginFactoryFn = unsafe extern "system" fn() -> *mut IPluginFactory;
    type BundleEntryFn = unsafe extern "system" fn(*mut c_void) -> bool;
    type BundleExitFn = unsafe extern "system" fn() -> bool;

    struct CreatedController {
        controller: ComPtr<IEditController>,
        shared_with_component: bool,
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
        view_attached: bool,
        connection_points_connected: bool,
        component_initialized: bool,
        controller_initialized: bool,
        controller_shared_with_component: bool,
    }

    impl PluginHostSession {
        fn shutdown(mut self) {
            unsafe {
                if self.connection_points_connected {
                    disconnect_component_and_controller(&self.component, &self.controller);
                    self.connection_points_connected = false;
                }
                if self.view_attached {
                    let _ = self.view.removed();
                    self.view_attached = false;
                }
                if self.controller_initialized && !self.controller_shared_with_component {
                    let _ = self.controller.terminate();
                    self.controller_initialized = false;
                }
                if self.component_initialized {
                    let _ = self.component.terminate();
                    self.component_initialized = false;
                }
            }
        }

        fn view_state(&self) -> (ComPtr<IPlugView>, usize) {
            (self.view.clone(), self.host_view_ptr)
        }
    }

    struct LoadedVst3Module {
        binary_path: PathBuf,
        library: Library,
        bundle_exit: Option<BundleExitFn>,
    }

    impl Drop for LoadedVst3Module {
        fn drop(&mut self) {
            if let Some(bundle_exit) = self.bundle_exit {
                unsafe {
                    let _ = bundle_exit();
                }
            }
        }
    }

    struct PluginHostApplication;

    impl Class for PluginHostApplication {
        type Interfaces = (IHostApplication,);
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
            _cid: *mut TUID,
            _iid: *mut TUID,
            obj: *mut *mut c_void,
        ) -> tresult {
            if !obj.is_null() {
                *obj = std::ptr::null_mut();
            }
            kNotImplemented
        }
    }

    struct PluginComponentHandler;

    impl Class for PluginComponentHandler {
        type Interfaces = (IComponentHandler,);
    }

    impl IComponentHandlerTrait for PluginComponentHandler {
        unsafe fn beginEdit(&self, id: Vst::ParamID) -> tresult {
            eprintln!("plugin_editor_poc: plugin began editing parameter {}", id);
            kResultOk
        }

        unsafe fn performEdit(
            &self,
            id: Vst::ParamID,
            value_normalized: Vst::ParamValue,
        ) -> tresult {
            eprintln!(
                "plugin_editor_poc: plugin changed parameter {} to normalized value {}",
                id, value_normalized
            );
            kResultOk
        }

        unsafe fn endEdit(&self, id: Vst::ParamID) -> tresult {
            eprintln!("plugin_editor_poc: plugin ended editing parameter {}", id);
            kResultOk
        }

        unsafe fn restartComponent(&self, flags: i32) -> tresult {
            eprintln!(
                "plugin_editor_poc: plugin requested component restart with flags {}",
                flags
            );
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
        let library = unsafe { Library::new(&binary_path) }.map_err(|error| {
            format!(
                "Failed to load VST3 bundle binary '{}': {error}",
                binary_path.display()
            )
        })?;

        if let Ok(bundle_entry) = unsafe { library.get::<BundleEntryFn>(b"BundleEntry") } {
            let entered = unsafe { bundle_entry(std::ptr::null_mut()) };
            if !entered {
                return Err(format!(
                    "VST3 bundle entry point returned false for '{}'.",
                    binary_path.display()
                ));
            }
        }

        let bundle_exit = unsafe { library.get::<BundleExitFn>(b"BundleExit") }
            .ok()
            .map(|symbol| *symbol);

        Ok(LoadedVst3Module {
            binary_path,
            library,
            bundle_exit,
        })
    }

    fn create_plugin_factory(module: &LoadedVst3Module) -> Result<ComPtr<IPluginFactory>, String> {
        let get_plugin_factory = unsafe {
            module
                .library
                .get::<GetPluginFactoryFn>(b"GetPluginFactory")
        }
        .map_err(|error| {
            format!(
                "Failed to resolve GetPluginFactory in '{}': {error}",
                module.binary_path.display()
            )
        })?;
        let raw_factory = unsafe { get_plugin_factory() };
        unsafe { ComPtr::from_raw(raw_factory) }.ok_or_else(|| {
            format!(
                "GetPluginFactory returned a null factory pointer for '{}'.",
                module.binary_path.display()
            )
        })
    }

    fn create_component(factory: &ComPtr<IPluginFactory>) -> Result<ComPtr<IComponent>, String> {
        for index in 0..unsafe { factory.countClasses() } {
            let mut info = unsafe { zeroed::<Steinberg::PClassInfo>() };
            if unsafe { factory.getClassInfo(index, &mut info) } != kResultOk {
                continue;
            }

            let mut raw_object: *mut c_void = std::ptr::null_mut();
            let result = unsafe {
                factory.createInstance(info.cid.as_ptr(), IComponent_iid.as_ptr(), &mut raw_object)
            };
            if result != kResultOk {
                continue;
            }

            if let Some(component) = unsafe { ComPtr::from_raw(raw_object.cast::<IComponent>()) } {
                return Ok(component);
            }
        }

        Err(String::from(
            "Plugin editor POC could not create an IComponent instance from the VST3 factory.",
        ))
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

        let component = create_component(&factory)?;
        let component_init = unsafe { component.initialize(host_context.as_ptr()) };
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

        let controller_initialized = if controller_shared_with_component {
            eprintln!("plugin_editor_poc: component also implements the edit controller interface");
            false
        } else {
            let controller_init = unsafe { controller.initialize(host_context.as_ptr()) };
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
            sync_component_state_to_controller(&component, &controller);
        }

        let component_handler = ComWrapper::new(PluginComponentHandler)
            .to_com_ptr::<IComponentHandler>()
            .ok_or_else(|| String::from("Failed to allocate the VST3 component-handler shim."))?;
        let _ = unsafe { controller.setComponentHandler(component_handler.as_ptr()) };

        let connection_points_connected = if controller_shared_with_component {
            false
        } else {
            connect_component_and_controller(&component, &controller)
        };

        let plug_frame = ComWrapper::new(PluginHostFrame {
            window: window.clone(),
        })
        .to_com_ptr::<IPlugFrame>()
        .ok_or_else(|| String::from("Failed to allocate the VST3 plug-frame shim."))?;

        let view = unsafe { ComPtr::from_raw(controller.createView(ViewType::kEditor)) }
            .ok_or_else(|| {
                cleanup_initialized_plugin(connection_points_connected);
                String::from(
                    "Plugin editor POC did not receive a plugin editor view. The selected VST3 may not expose an editor.",
                )
            })?;

        let platform_supported = unsafe { view.isPlatformTypeSupported(kPlatformTypeNSView) };
        if platform_supported != kResultOk {
            cleanup_initialized_plugin(connection_points_connected);
            return Err(String::from(
                "Plugin editor POC editor view does not report NSView support.",
            ));
        }

        let _ = unsafe { view.setFrame(plug_frame.as_ptr()) };

        let parent_ptr = (parent_view as *const NSView).cast_mut().cast::<c_void>();
        let attached = unsafe { view.attached(parent_ptr, kPlatformTypeNSView) };
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
            view_attached: true,
            connection_points_connected,
            component_initialized: true,
            controller_initialized,
            controller_shared_with_component,
        })
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
            if let Ok(mut session_state) = session_state.lock() {
                if let Some(session) = session_state.take() {
                    session.shutdown();
                }
            }
        }
    }

    pub fn cleanup_orphaned_plugin_host_session() {
        let session = active_plugin_host_session()
            .lock()
            .ok()
            .and_then(|mut active_session| active_session.take());

        if let Some(session) = session {
            session.shutdown();
        }
    }

    pub fn sync_active_plugin_host(window: &WebviewWindow, focused: Option<bool>) {
        let Some((view, host_view_ptr)) = active_plugin_host_session()
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

            let mut rect = view_rect_for_host_frame(host_frame);
            let _ = unsafe { view.onSize(&mut rect) };
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
                    sync_active_plugin_host(&attachment_window, Some(true));

                    let mut active_session = active_plugin_host_session()
                        .lock()
                        .map_err(|_| String::from("Failed to store plugin host session"))?;
                    *active_session = Some(session);

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
    cleanup_active_plugin_host_session, cleanup_orphaned_plugin_host_session,
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

    match reserve_plugin_target(&plugin_path)? {
        PluginTargetReservation::Acquired => {}
        PluginTargetReservation::AlreadyReserved => {
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
        active_plugin_target, release_reserved_plugin_target, reserve_plugin_target,
        resolve_plugin_target_in_roots, PluginTargetReservation,
    };
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    fn plugin_target_test_mutex() -> &'static Mutex<()> {
        static PLUGIN_TARGET_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        PLUGIN_TARGET_TEST_MUTEX.get_or_init(|| Mutex::new(()))
    }

    fn reset_active_plugin_target() {
        if let Ok(mut active_target) = active_plugin_target().lock() {
            *active_target = None;
        }
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
        let _guard = plugin_target_test_mutex().lock().unwrap();
        reset_active_plugin_target();

        let target = Path::new("/tmp/test-plugin.vst3");
        let reservation = reserve_plugin_target(target).unwrap();

        assert!(matches!(reservation, PluginTargetReservation::Acquired));

        reset_active_plugin_target();
    }

    #[test]
    fn reserve_plugin_target_reuses_matching_target() {
        let _guard = plugin_target_test_mutex().lock().unwrap();
        reset_active_plugin_target();

        let target = Path::new("/tmp/test-plugin.vst3");
        reserve_plugin_target(target).unwrap();

        let reservation = reserve_plugin_target(target).unwrap();

        assert!(matches!(
            reservation,
            PluginTargetReservation::AlreadyReserved
        ));

        reset_active_plugin_target();
    }

    #[test]
    fn reserve_plugin_target_rejects_different_target_while_reserved() {
        let _guard = plugin_target_test_mutex().lock().unwrap();
        reset_active_plugin_target();

        reserve_plugin_target(Path::new("/tmp/test-plugin-a.vst3")).unwrap();

        let error = reserve_plugin_target(Path::new("/tmp/test-plugin-b.vst3")).unwrap_err();

        assert!(error.contains("already open for a different target"));

        reset_active_plugin_target();
    }

    #[test]
    fn release_reserved_plugin_target_clears_matching_target_only() {
        let _guard = plugin_target_test_mutex().lock().unwrap();
        reset_active_plugin_target();

        let active_target = Path::new("/tmp/test-plugin-a.vst3");
        reserve_plugin_target(active_target).unwrap();
        release_reserved_plugin_target(Path::new("/tmp/test-plugin-b.vst3"));

        assert_eq!(
            active_plugin_target().lock().unwrap().as_deref(),
            Some(active_target)
        );

        release_reserved_plugin_target(active_target);

        assert_eq!(active_plugin_target().lock().unwrap().as_deref(), None);
        reset_active_plugin_target();
    }
}
