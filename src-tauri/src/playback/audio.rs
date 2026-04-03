use crate::playback::engine;
use crate::playback::mixer::{self, MixerHandle};
use crate::AudioContextState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

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

impl AudioContext {
    pub fn new() -> AudioContextState {
        let host = cpal::default_host();

        let devices = Self::enumerate_output_devices(&host);

        let active_device = AudioContext::default_device();

        // Create the persistent mixer + stream
        let (mixer_handle, stream) =
            Self::build_stream(&active_device.device, active_device.sample_rate);

        Mutex::new(AudioContext {
            host,
            devices,
            active_device,
            mixer_handle,
            _stream: stream,
        })
    }
    fn enumerate_output_devices(host: &Host) -> Vec<DeviceInfo> {
        host.output_devices()
            .expect("Failed to get output devices")
            .filter_map(|device| {
                if let Ok(id) = device.id() {
                    let default_config = device
                        .default_output_config()
                        .expect("Failed to get default output config");

                    let mut sample_rates = FxHashSet::default();
                    if let Ok(configs) = device.supported_output_configs() {
                        for config in configs {
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
            .collect::<Vec<_>>()
    }

    /// Build (or rebuild) the persistent output stream with a mixer.
    fn build_stream(device: &Device, sample_rate: u32) -> (MixerHandle, Stream) {
        let (handle, reader) = mixer::create_mixer();

        // Use stereo output — most hardware is stereo
        let channels: u16 = 2;

        let stream_config = cpal::StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let ch = channels as usize;
        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    data.fill(0.0);
                    engine::process_audio_block(data, ch, sample_rate);
                    reader.mix_into_buffer(data, ch);
                    mixer::MixerReader::clamp_buffer(data);
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .expect("Failed to build persistent output stream");

        stream
            .play()
            .expect("Failed to start persistent output stream");

        (handle, stream)
    }

    fn default_device() -> ActiveDevice {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("Failed to get default output device");

        let id = device
            .id()
            .expect("Failed to get default output device ID")
            .to_string();

        let sample_rate = device
            .default_output_config()
            .expect("Failed to get default output config")
            .sample_rate();

        ActiveDevice {
            device,
            id,
            sample_rate,
        }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.active_device.sample_rate
    }

    pub fn get_mixer_handle(&self) -> &MixerHandle {
        &self.mixer_handle
    }

    pub fn refresh_devices(&mut self) -> bool {
        let old_device_id = self.active_device.id.clone();

        self.devices = Self::enumerate_output_devices(&self.host);

        // Check if the current device is still available
        self.devices.iter().any(|d| d.id == old_device_id)
    }

    pub fn emit(&self, app: &tauri::AppHandle) {
        let audio_state = AudioState {
            devices: self.devices.clone(),
            active_device_id: self.active_device.id.clone(),
            current_sample_rate: self.active_device.sample_rate,
        };

        if let Err(e) = app.emit("soloc://state/audio", &audio_state) {
            eprintln!("Failed to emit audio state: {:?}", e);
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn audio_device_set(
    app: AppHandle,
    state: State<'_, AudioContextState>,
    device_id: String,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();

    let device_option = ctx
        .host
        .output_devices()
        .map_err(|e| e.to_string())?
        .find(|d| {
            if let Ok(id) = d.id() {
                id.to_string() == device_id
            } else {
                false
            }
        });

    if let Some(device) = device_option {
        let id = device.id().map_err(|e| e.to_string())?.to_string();
        let sample_rate = device
            .default_output_config()
            .map_err(|e| e.to_string())?
            .sample_rate();

        // Rebuild the stream on the new device
        let (mixer_handle, stream) = AudioContext::build_stream(&device, sample_rate);

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
        Err("Device not found".to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn audio_device_sample_rate_set(
    app: AppHandle,
    state: State<'_, AudioContextState>,
    sample_rate: u32,
) {
    let mut ctx = state.lock().unwrap();

    ctx.active_device.sample_rate = sample_rate;

    // Rebuild the stream at the new sample rate
    let (mixer_handle, stream) = AudioContext::build_stream(&ctx.active_device.device, sample_rate);
    ctx.mixer_handle = mixer_handle;
    ctx._stream = stream;

    ctx.emit(&app);
}

#[tauri::command(rename_all = "snake_case")]
pub fn audio_devices_refresh(
    app: AppHandle,
    state: State<'_, AudioContextState>,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();

    let device_still_available = ctx.refresh_devices();

    // If the current device is no longer available, switch to default
    if !device_still_available {
        let default_device = AudioContext::default_device();
        let (mixer_handle, stream) =
            AudioContext::build_stream(&default_device.device, default_device.sample_rate);

        ctx.active_device = default_device;
        ctx.mixer_handle = mixer_handle;
        ctx._stream = stream;
    }

    ctx.emit(&app);
    Ok(())
}
