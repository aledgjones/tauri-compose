use crate::AudioContextState;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};
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
}

const SUPPORTED_SAMPLE_RATES: [u32; 4] = [44100, 48000, 88200, 96000];

impl AudioContext {
    pub fn new() -> AudioContextState {
        let host = cpal::default_host();

        let devices = host
            .output_devices()
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
            .collect::<Vec<_>>();

        let active_device = AudioContext::default_device();

        Mutex::new(AudioContext {
            host,
            devices,
            active_device,
        })
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

        ctx.active_device = ActiveDevice {
            device,
            id,
            sample_rate,
        };

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

    ctx.emit(&app);
}
