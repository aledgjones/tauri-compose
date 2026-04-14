use super::{
    active_plugin_host_session, attached_editor_host_validation, failed_editor_host_validation,
    PluginEditorHostAttemptError, PluginEditorHostValidationPhase, PluginHostAttachOutcome,
    SlotRuntimePluginStateCapture,
};
use crate::playback::engine::{LiveRenderEvent, LiveRenderEventKind};
use crate::playback::plugin_inventory::vst3_bundle_binary_path;
use crate::playback::runtime_activation::ActivatedSlotTarget;
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
    NSAutoresizingMaskOptions, NSBackingStoreType, NSTextField, NSView, NSWindow, NSWindowDelegate,
    NSWindowStyleMask,
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
        std::ptr::copy_nonoverlapping(bytes.as_ptr().add(*cursor), buffer.cast::<u8>(), to_copy);
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
            || self.render_scratch.output_bus_channels_data.len() != self.render_output_buses.len()
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
                super::log_active_host_phase_callback("IComponentHandler2::startGroupEdit invoked");
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
        let registry = INTERFACE_POINTER_LABELS.get_or_init(|| StdMutex::new(FxHashMap::default()));
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
    const HEADER: Self::Header = <PluginComponentHandlerInterfaceTuple as MakeHeader<C, W>>::HEADER;
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
            let new_frame: NSRect = msg_send![&*ns_window, frameRectForContentRect: new_content];
            let _: () = msg_send![&*ns_window, setFrame: new_frame, display: true, animate: false];

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

fn component_class_match_score(candidate: &ComponentClassCandidate, requested_name: &str) -> usize {
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
        if subcategory_key.contains(&requested_key) || requested_key.contains(&subcategory_key) {
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

fn audio_module_class_candidates(factory: &ComPtr<IPluginFactory>) -> Vec<ComponentClassCandidate> {
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

fn decode_component_state_bytes(plugin_state: &PlaybackSlotPluginState) -> Result<Vec<u8>, String> {
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
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
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
            .map_err(|_| String::from("Failed to access slot runtime session for target switch"))?
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
    let sessions = slot_runtime_sessions()
        .lock()
        .map_err(|_| String::from("Failed to access slot runtime sessions for state capture."))?;
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
    let received = receiver
        .recv()
        .map_err(|error| format!("Failed to receive slot runtime state capture result: {error}"));
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
                        .map_err(|_| String::from("Failed to read slot runtime session registry"))?
                        .remove(&slot_id);
                    let mut session = match session {
                        Some(session) => session,
                        None => build_plugin_host_session(&target, target.plugin_state.as_ref())
                            .map_err(|error| error.message.clone())?,
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
                                String::from("Failed to read active inventory plugin host session")
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
                            String::from("Failed to restore active inventory plugin host session")
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
