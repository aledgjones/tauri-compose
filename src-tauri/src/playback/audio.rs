use crate::playback::engine;
use crate::playback::mixer::{self, MixerHandle};
use crate::playback::plugin_host::prepare_slot_runtime_sessions;
use crate::playback::transport::{
    stop_transport_playback, sync_transport_to_app_state, TransportContext, TransportSharedState,
};
use crate::AudioContextState;
use crate::{AppContext, AppContextState};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    Device, ErrorKind, FromSample, Host, SampleFormat, SizedSample, Stream, SupportedStreamConfig,
};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub sample_rates: Vec<u32>,
    pub default_sample_rate: u32,
}

#[derive(Clone)]
pub struct ActiveDevice {
    pub device: Device,
    pub id: String,
    pub sample_rate: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioState {
    pub devices: Vec<DeviceInfo>,
    pub active_device_id: String,
    pub current_sample_rate: u32,
    pub stream_error_count: u64,
    pub xrun_count: u64,
    pub last_stream_error_kind: u32,
    pub render_error_count: u64,
    pub last_render_error_slot_id: Option<crate::utils::UniqueId>,
    pub last_render_error: Option<String>,
    pub dropped_engine_command_count: u64,
    pub master_overload_count: u64,
    pub master_peak: f32,
    pub current_master_peak: f32,
}

pub struct AudioContext {
    host: Host,
    devices: Vec<DeviceInfo>,
    active_device: ActiveDevice,
    mixer_handle: MixerHandle,
    // Keep the stream alive — dropping it stops audio
    _stream: Stream,
}

const SUPPORTED_SAMPLE_RATES: [u32; 4] = [44100, 48000, 88200, 96000];
const MAX_OUTPUT_CALLBACK_SAMPLES: usize = 16_384 * 8;
static AUDIO_STREAM_ERROR_COUNT: AtomicU64 = AtomicU64::new(0);
static AUDIO_XRUN_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_AUDIO_STREAM_ERROR_KIND: AtomicU32 = AtomicU32::new(0);
static AUDIO_RECOVERY_REQUEST: AtomicU32 = AtomicU32::new(0);

fn audio_error_kind_code(kind: ErrorKind) -> u32 {
    match kind {
        ErrorKind::DeviceBusy => 1,
        ErrorKind::DeviceChanged => 2,
        ErrorKind::DeviceNotAvailable => 3,
        ErrorKind::HostUnavailable => 4,
        ErrorKind::InvalidInput => 5,
        ErrorKind::PermissionDenied => 6,
        ErrorKind::RealtimeDenied => 7,
        ErrorKind::ResourceExhausted => 8,
        ErrorKind::StreamInvalidated => 9,
        ErrorKind::UnsupportedConfig => 10,
        ErrorKind::UnsupportedOperation => 11,
        ErrorKind::Xrun => 12,
        ErrorKind::BackendError => 13,
        ErrorKind::Other => 14,
        _ => 15,
    }
}

impl AudioContext {
    pub fn new(app: &AppHandle) -> Result<AudioContextState, String> {
        let host = cpal::default_host();

        let devices = Self::enumerate_output_devices(&host)?;

        let active_device = AudioContext::default_device()?;

        let (mixer_handle, stream) =
            Self::build_stream(app, &active_device.device, active_device.sample_rate)?;
        stream
            .play()
            .map_err(|error| format!("Failed to start the default audio stream: {error}"))?;

        Ok(Mutex::new(AudioContext {
            host,
            devices,
            active_device,
            mixer_handle,
            _stream: stream,
        }))
    }
    fn enumerate_output_devices(host: &Host) -> Result<Vec<DeviceInfo>, String> {
        let devices = host
            .output_devices()
            .map_err(|error| format!("Failed to enumerate output devices: {error}"))?
            .filter_map(|device| {
                if let Ok(id) = device.id() {
                    let default_config = device.default_output_config().ok()?;

                    let mut sample_rates = FxHashSet::default();
                    if let Ok(configs) = device.supported_output_configs() {
                        for config in configs {
                            if !matches!(
                                config.sample_format(),
                                SampleFormat::F32 | SampleFormat::I16 | SampleFormat::U16
                            ) || !(1..=8).contains(&config.channels())
                            {
                                continue;
                            }
                            let min_rate = config.min_sample_rate();
                            let max_rate = config.max_sample_rate();

                            for &rate in &SUPPORTED_SAMPLE_RATES {
                                if rate >= min_rate && rate <= max_rate {
                                    sample_rates.insert(rate);
                                }
                            }
                        }
                    }

                    let sample_rates = {
                        let mut rates: Vec<u32> = sample_rates.iter().cloned().collect();
                        rates.sort_unstable();
                        rates
                    };

                    if let Ok(info) = device.description() {
                        return Some(DeviceInfo {
                            id: id.to_string(),
                            name: info.name().to_string(),
                            sample_rates,
                            default_sample_rate: default_config.sample_rate(),
                        });
                    }
                };

                None
            })
            .collect::<Vec<_>>();
        Ok(devices)
    }

    fn supported_output_config(
        device: &Device,
        sample_rate: u32,
    ) -> Result<SupportedStreamConfig, String> {
        let default_channels = device
            .default_output_config()
            .map_err(|error| format!("Failed to read the default output configuration: {error}"))?
            .channels();
        let mut candidates = device
            .supported_output_configs()
            .map_err(|error| format!("Failed to read supported output configurations: {error}"))?
            .filter(|config| {
                matches!(
                    config.sample_format(),
                    SampleFormat::F32 | SampleFormat::I16 | SampleFormat::U16
                ) && (1..=8).contains(&config.channels())
                    && config.min_sample_rate() <= sample_rate
                    && sample_rate <= config.max_sample_rate()
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|config| {
            (
                config.channels() != 2,
                config.channels() != default_channels,
                config.sample_format() != SampleFormat::F32,
                config.channels(),
            )
        });
        candidates
            .into_iter()
            .next()
            .map(|config| config.with_sample_rate(sample_rate))
            .ok_or_else(|| {
                format!("The selected device has no f32 output configuration at {sample_rate} Hz.")
            })
    }

    /// Build a paused output stream. The caller starts it after dependent processors are ready.
    fn build_stream(
        _app: &AppHandle,
        device: &Device,
        sample_rate: u32,
    ) -> Result<(MixerHandle, Stream), String> {
        let (handle, reader) = mixer::create_mixer();
        let supported_config = Self::supported_output_config(device, sample_rate)?;
        let sample_format = supported_config.sample_format();
        let stream_config = supported_config.config();
        let channels = stream_config.channels as usize;
        let stream = match sample_format {
            SampleFormat::F32 => Self::build_typed_stream::<f32>(
                device,
                stream_config,
                sample_rate,
                channels,
                reader,
            ),
            SampleFormat::I16 => Self::build_typed_stream::<i16>(
                device,
                stream_config,
                sample_rate,
                channels,
                reader,
            ),
            SampleFormat::U16 => Self::build_typed_stream::<u16>(
                device,
                stream_config,
                sample_rate,
                channels,
                reader,
            ),
            other => Err(format!("Unsupported output sample format: {other}")),
        }?;

        Ok((handle, stream))
    }

    fn build_typed_stream<T>(
        device: &Device,
        stream_config: cpal::StreamConfig,
        sample_rate: u32,
        channels: usize,
        mut reader: mixer::MixerReader,
    ) -> Result<Stream, String>
    where
        T: SizedSample + FromSample<f32>,
    {
        let mut render_scratch = vec![0.0_f32; MAX_OUTPUT_CALLBACK_SAMPLES];
        device
            .build_output_stream(
                stream_config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    if data.len() > render_scratch.len() {
                        data.fill(T::EQUILIBRIUM);
                        return;
                    }
                    let output = &mut render_scratch[..data.len()];
                    output.fill(0.0);
                    engine::process_audio_block(output, channels, sample_rate);
                    reader.mix_into_buffer(output, channels);
                    mixer::MixerReader::limit_buffer(output);
                    for (destination, source) in data.iter_mut().zip(output.iter()) {
                        *destination = T::from_sample(*source);
                    }
                },
                move |error| {
                    AUDIO_STREAM_ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
                    AUDIO_XRUN_COUNT.fetch_add(
                        u64::from(error.kind() == ErrorKind::Xrun),
                        Ordering::Relaxed,
                    );
                    LAST_AUDIO_STREAM_ERROR_KIND
                        .store(audio_error_kind_code(error.kind()), Ordering::Relaxed);
                    match error.kind() {
                        ErrorKind::DeviceNotAvailable | ErrorKind::HostUnavailable => {
                            AUDIO_RECOVERY_REQUEST.store(2, Ordering::Release);
                        }
                        ErrorKind::StreamInvalidated => {
                            AUDIO_RECOVERY_REQUEST.store(1, Ordering::Release);
                        }
                        _ => {}
                    }
                },
                None,
            )
            .map_err(|error| format!("Failed to build the output stream: {error}"))
    }

    fn default_device() -> Result<ActiveDevice, String> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| String::from("No default output device is available."))?;

        let id = device
            .id()
            .map_err(|error| format!("Failed to read the default output device ID: {error}"))?
            .to_string();

        let sample_rate = device
            .default_output_config()
            .map_err(|error| format!("Failed to read the default output configuration: {error}"))?
            .sample_rate();

        Ok(ActiveDevice {
            device,
            id,
            sample_rate,
        })
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.active_device.sample_rate
    }

    pub fn get_mixer_handle(&self) -> &MixerHandle {
        &self.mixer_handle
    }

    pub fn refresh_devices(&mut self) -> bool {
        let old_device_id = self.active_device.id.clone();

        let Ok(devices) = Self::enumerate_output_devices(&self.host) else {
            return false;
        };
        self.devices = devices;

        // Check if the current device is still available
        self.devices.iter().any(|d| d.id == old_device_id)
    }

    pub(crate) fn snapshot(&self) -> AudioState {
        let last_render_error = engine::last_render_error();
        AudioState {
            devices: self.devices.clone(),
            active_device_id: self.active_device.id.clone(),
            current_sample_rate: self.active_device.sample_rate,
            stream_error_count: AUDIO_STREAM_ERROR_COUNT.load(Ordering::Relaxed),
            xrun_count: AUDIO_XRUN_COUNT.load(Ordering::Relaxed),
            last_stream_error_kind: LAST_AUDIO_STREAM_ERROR_KIND.load(Ordering::Relaxed),
            render_error_count: engine::render_error_count(),
            last_render_error_slot_id: last_render_error.as_ref().map(|(slot_id, _)| *slot_id),
            last_render_error: last_render_error.map(|(_, error)| error),
            dropped_engine_command_count: engine::dropped_engine_command_count(),
            master_overload_count: mixer::master_overload_count(),
            master_peak: mixer::master_peak(),
            current_master_peak: mixer::current_master_peak(),
        }
    }

    pub fn emit(&self, app: &tauri::AppHandle) {
        let audio_state = self.snapshot();

        if let Err(e) = app.emit("soloc://state/audio", &audio_state) {
            eprintln!("Failed to emit audio state: {:?}", e);
        }
    }
}

pub(crate) fn start_audio_recovery_monitor(app: AppHandle) {
    static STARTED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if STARTED.set(()).is_err() {
        return;
    }

    let _ = std::thread::Builder::new()
        .name(String::from("audio-recovery"))
        .spawn(move || loop {
            let request = AUDIO_RECOVERY_REQUEST.swap(0, Ordering::AcqRel);
            if request == 0 {
                std::thread::park_timeout(std::time::Duration::from_millis(100));
                continue;
            }

            let audio_state = app.state::<AudioContextState>();
            let transport_state = app.state::<Mutex<TransportContext>>();
            let app_state = app.state::<AppContextState>();
            let recovery_result = if request == 2 {
                audio_devices_refresh(app.clone(), audio_state, transport_state, app_state)
            } else {
                let sample_rate = audio_state.lock().unwrap().get_sample_rate();
                audio_device_sample_rate_set(
                    app.clone(),
                    audio_state,
                    transport_state,
                    app_state,
                    sample_rate,
                )
            };

            if recovery_result.is_err() {
                AUDIO_STREAM_ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
                app.state::<AudioContextState>().lock().unwrap().emit(&app);
            }
        });
}

fn resync_transport_after_audio_rebuild(
    transport_shared: &TransportSharedState,
    app_state: &AppContextState,
) -> Result<(), String> {
    let current_tick = transport_shared.current_tick.load(Ordering::Relaxed);
    let app_ctx = app_state.lock().unwrap();
    sync_transport_to_app_state(transport_shared, &app_ctx, current_tick);
    Ok(())
}

fn reconcile_pending_playback_refresh<F>(
    app_state: &AppContextState,
    refresh_topology_runtime: F,
) -> Result<(), String>
where
    F: FnOnce(&mut AppContext) -> Result<(), String>,
{
    let mut app_ctx = app_state.lock().unwrap();
    if app_ctx.pending_playback_refresh_kind()
        == Some(crate::playback::transport::DeferredPlaybackRefreshKind::PlaybackTopology)
    {
        refresh_topology_runtime(&mut app_ctx)?;
    }
    app_ctx.clear_pending_playback_refresh();

    Ok(())
}

pub(crate) fn reset_transport_for_audio_rebuild_with_stop<F, R>(
    app_state: &AppContextState,
    stop_transport: F,
    refresh_pending_playback_runtime: R,
) -> Result<(), String>
where
    F: FnOnce() -> Result<TransportSharedState, String>,
    R: FnOnce(&AppContextState) -> Result<(), String>,
{
    let transport_shared = stop_transport()?;
    refresh_pending_playback_runtime(app_state)?;
    resync_transport_after_audio_rebuild(&transport_shared, app_state).map(|_| {
        transport_shared.last_queued_tick.store(
            transport_shared.current_tick.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
    })
}

fn reset_transport_for_audio_rebuild(
    app: &AppHandle,
    transport_state: &Mutex<TransportContext>,
    audio_state: &AudioContextState,
    app_state: &AppContextState,
) -> Result<(), String> {
    reset_transport_for_audio_rebuild_with_stop(
        app_state,
        || stop_transport_playback(transport_state, audio_state),
        |app_state| {
            reconcile_pending_playback_refresh(app_state, |app_ctx| {
                app_ctx.refresh_playback_activation_runtime_from_app(app);
                Ok(())
            })
        },
    )
}

fn start_replacement_stream(
    app: &AppHandle,
    audio_state: &AudioContextState,
    stream: &Stream,
    previous_sample_rate: u32,
) -> Result<(), String> {
    {
        let context = audio_state.lock().unwrap();
        if let Err(error) = context._stream.pause() {
            let _ = prepare_slot_runtime_sessions(app, previous_sample_rate);
            return Err(format!("Failed to pause the current audio stream: {error}"));
        }
    }
    if let Err(error) = stream.play() {
        let _ = audio_state.lock().unwrap()._stream.play();
        let _ = prepare_slot_runtime_sessions(app, previous_sample_rate);
        return Err(format!(
            "Failed to start the replacement audio stream: {error}"
        ));
    }
    Ok(())
}

fn prepare_replacement_sessions(
    app: &AppHandle,
    sample_rate: u32,
    previous_sample_rate: u32,
) -> Result<(), String> {
    if let Err(error) = prepare_slot_runtime_sessions(app, sample_rate) {
        let _ = prepare_slot_runtime_sessions(app, previous_sample_rate);
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::runtime_activation::PlaybackActivationRuntime;
    use crate::score::components::duration::NoteDuration;
    use crate::score::entries::{tempo::Tempo, Entry};
    use crate::score::flows::Flow;
    use crate::score::tracks::Track;
    use crate::ui::Ui;
    use crate::AppContext;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    fn make_app_context_with_tempo(flow_id: u64, tick: u64, bpm: u16) -> AppContextState {
        let mut score = crate::score::Score::new();
        let master_id = 2;
        let mut flow = Flow::new(flow_id, master_id);
        flow.length = tick + 32;

        let mut master_track = Track::new(master_id);
        master_track.insert_entry(Entry::Tempo(Tempo::new(
            10,
            tick,
            String::new(),
            NoteDuration::Quarter,
            0,
            bpm,
            false,
            false,
            true,
            false,
        )));

        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, flow);
        score.tracks.by_id.insert(master_id, master_track);

        let mut ui = Ui::new();
        ui.flow_id = Some(flow_id);

        Mutex::new(AppContext {
            ui,
            score,
            playback_activation_runtime: PlaybackActivationRuntime::default(),
            pending_playback_refresh: None,
            preferences: serde_json::from_str(r#"{"audition":true}"#).unwrap(),
            next_id: 1,
            current_file_path: None,
        })
    }

    #[test]
    fn reset_transport_for_audio_rebuild_republishes_backend_tempo_after_stop() {
        let (_transport_state, transport_shared) = TransportContext::new();
        transport_shared.current_tick.store(96, Ordering::Relaxed);
        transport_shared
            .last_queued_tick
            .store(144, Ordering::Relaxed);
        transport_shared.playing.store(false, Ordering::Release);
        transport_shared.set_tempo(72.0);

        let app_state = make_app_context_with_tempo(1, 64, 144);
        let stop_called = Arc::new(AtomicBool::new(false));
        let stop_called_flag = stop_called.clone();
        let shared_for_stop = transport_shared.clone();

        reset_transport_for_audio_rebuild_with_stop(
            &app_state,
            move || {
                stop_called_flag.store(true, Ordering::Release);
                Ok(shared_for_stop.clone())
            },
            |_| Ok(()),
        )
        .unwrap();

        assert!(stop_called.load(Ordering::Acquire));
        assert_eq!(transport_shared.current_tick.load(Ordering::Relaxed), 95);
        assert_eq!(
            transport_shared.last_queued_tick.load(Ordering::Relaxed),
            95
        );
        assert_eq!(transport_shared.get_tempo(), 144.0);
    }

    #[test]
    fn reset_transport_for_audio_rebuild_keeps_the_rewound_stop_boundary_tick_over_stale_last_queued_tick(
    ) {
        let (_transport_state, transport_shared) = TransportContext::new();
        transport_shared.current_tick.store(63, Ordering::Relaxed);
        transport_shared
            .last_queued_tick
            .store(80, Ordering::Relaxed);
        transport_shared.playing.store(false, Ordering::Release);
        transport_shared.set_tempo(72.0);

        let app_state = make_app_context_with_tempo(1, 64, 144);
        let shared_for_stop = transport_shared.clone();

        reset_transport_for_audio_rebuild_with_stop(
            &app_state,
            move || Ok(shared_for_stop.clone()),
            |_| Ok(()),
        )
        .unwrap();

        assert_eq!(transport_shared.current_tick.load(Ordering::Relaxed), 63);
        assert_eq!(
            transport_shared.last_queued_tick.load(Ordering::Relaxed),
            63
        );
        assert_eq!(transport_shared.get_tempo(), 120.0);
    }

    #[test]
    fn reset_transport_for_audio_rebuild_reconciles_pending_topology_refresh_before_clearing() {
        let (_transport_state, transport_shared) = TransportContext::new();
        let app_state = make_app_context_with_tempo(1, 64, 144);
        {
            let mut app_ctx = app_state.lock().unwrap();
            app_ctx.defer_playback_refresh(
                crate::playback::transport::DeferredPlaybackRefreshKind::PlaybackTopology,
            );
        }

        let refresh_called = Arc::new(AtomicBool::new(false));
        let refresh_called_flag = refresh_called.clone();
        let shared_for_stop = transport_shared.clone();

        reset_transport_for_audio_rebuild_with_stop(
            &app_state,
            move || Ok(shared_for_stop.clone()),
            |app_state| {
                reconcile_pending_playback_refresh(app_state, |_app_ctx| {
                    refresh_called_flag.store(true, Ordering::Release);
                    Ok(())
                })
            },
        )
        .unwrap();

        assert!(refresh_called.load(Ordering::Acquire));
        assert_eq!(
            app_state.lock().unwrap().pending_playback_refresh_kind(),
            None
        );
    }

    #[test]
    fn reset_transport_for_audio_rebuild_clears_score_structure_refresh_without_topology_runtime_work(
    ) {
        let (_transport_state, transport_shared) = TransportContext::new();
        let app_state = make_app_context_with_tempo(1, 64, 144);
        {
            let mut app_ctx = app_state.lock().unwrap();
            app_ctx.defer_playback_refresh(
                crate::playback::transport::DeferredPlaybackRefreshKind::ScoreStructure,
            );
        }

        let refresh_called = Arc::new(AtomicBool::new(false));
        let refresh_called_flag = refresh_called.clone();
        let shared_for_stop = transport_shared.clone();

        reset_transport_for_audio_rebuild_with_stop(
            &app_state,
            move || Ok(shared_for_stop.clone()),
            |app_state| {
                reconcile_pending_playback_refresh(app_state, |_app_ctx| {
                    refresh_called_flag.store(true, Ordering::Release);
                    Ok(())
                })
            },
        )
        .unwrap();

        assert!(!refresh_called.load(Ordering::Acquire));
        assert_eq!(
            app_state.lock().unwrap().pending_playback_refresh_kind(),
            None
        );
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn audio_device_set(
    app: AppHandle,
    state: State<'_, AudioContextState>,
    transport_state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
    device_id: String,
) -> Result<(), String> {
    let previous_sample_rate = state.lock().unwrap().active_device.sample_rate;
    let device_option = {
        let ctx = state.lock().unwrap();
        ctx.host
            .output_devices()
            .map_err(|e| e.to_string())?
            .find(|d| {
                if let Ok(id) = d.id() {
                    id.to_string() == device_id
                } else {
                    false
                }
            })
    };

    if let Some(device) = device_option {
        let id = device.id().map_err(|e| e.to_string())?.to_string();
        let sample_rate = device
            .default_output_config()
            .map_err(|e| e.to_string())?
            .sample_rate();
        let (mixer_handle, stream) = AudioContext::build_stream(&app, &device, sample_rate)?;

        reset_transport_for_audio_rebuild(
            &app,
            transport_state.inner(),
            state.inner(),
            app_state.inner(),
        )?;
        prepare_replacement_sessions(&app, sample_rate, previous_sample_rate)?;
        start_replacement_stream(&app, state.inner(), &stream, previous_sample_rate)?;

        let mut ctx = state.lock().unwrap();
        ctx.active_device = ActiveDevice {
            device,
            id,
            sample_rate,
        };
        ctx.mixer_handle = mixer_handle;
        ctx._stream = stream;

        ctx.emit(&app);

        Ok(())
    } else {
        Err(String::from("Device not found"))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn audio_device_sample_rate_set(
    app: AppHandle,
    state: State<'_, AudioContextState>,
    transport_state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
    sample_rate: u32,
) -> Result<(), String> {
    let (device, previous_sample_rate) = {
        let ctx = state.lock().unwrap();
        (
            ctx.active_device.device.clone(),
            ctx.active_device.sample_rate,
        )
    };
    let (mixer_handle, stream) = AudioContext::build_stream(&app, &device, sample_rate)?;

    reset_transport_for_audio_rebuild(
        &app,
        transport_state.inner(),
        state.inner(),
        app_state.inner(),
    )?;
    prepare_replacement_sessions(&app, sample_rate, previous_sample_rate)?;
    start_replacement_stream(&app, state.inner(), &stream, previous_sample_rate)?;

    let mut ctx = state.lock().unwrap();

    ctx.active_device.sample_rate = sample_rate;
    ctx.mixer_handle = mixer_handle;
    ctx._stream = stream;

    ctx.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn audio_devices_refresh(
    app: AppHandle,
    state: State<'_, AudioContextState>,
    transport_state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
) -> Result<(), String> {
    let device_still_available = {
        let mut ctx = state.lock().unwrap();
        ctx.refresh_devices()
    };

    // If the current device is no longer available, switch to default
    if !device_still_available {
        let previous_sample_rate = state.lock().unwrap().active_device.sample_rate;
        let default_device = AudioContext::default_device()?;
        let (mixer_handle, stream) =
            AudioContext::build_stream(&app, &default_device.device, default_device.sample_rate)?;
        reset_transport_for_audio_rebuild(
            &app,
            transport_state.inner(),
            state.inner(),
            app_state.inner(),
        )?;

        prepare_replacement_sessions(&app, default_device.sample_rate, previous_sample_rate)?;
        start_replacement_stream(&app, state.inner(), &stream, previous_sample_rate)?;

        let mut ctx = state.lock().unwrap();

        ctx.active_device = default_device;
        ctx.mixer_handle = mixer_handle;
        ctx._stream = stream;

        ctx.emit(&app);
        return Ok(());
    }

    let ctx = state.lock().unwrap();
    ctx.emit(&app);
    Ok(())
}
