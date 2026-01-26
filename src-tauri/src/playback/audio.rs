use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, DeviceId, Host,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::AudioContextState;

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceInfoSampleRate {
    pub sample_rate: u32,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    // pub sample_rates: Vec<DeviceInfoSampleRate>,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioState {
    pub devices: Vec<DeviceInfo>,
    pub device: Option<DeviceInfo>,
}

pub struct AudioContext {
    host: Host,
    device: Device,
    devices: Vec<DeviceInfo>,
}

const SUPPORTED_SAMPLE_RATES: [u32; 4] = [44100, 48000, 88200, 96000];

impl AudioContext {
    pub fn new() -> AudioContextState {
        let host = cpal::default_host();
        let default_device = host
            .default_output_device()
            .expect("No default output device available");

        // Get all available devices
        let devices = host
            .output_devices()
            .expect("Failed to get output devices")
            .filter_map(|device| {
                if let Ok(id) = device.id() {
                    if let Ok(info) = device.description() {
                        return Some(DeviceInfo {
                            id: id.to_string(),
                            name: info.name().to_string(),
                            is_default: default_device.id() == device.id(),
                        });
                    }
                };

                None
            })
            .collect::<Vec<_>>();

        Mutex::new(AudioContext {
            host,
            device: default_device,
            devices,
        })
    }

    pub fn emit(&self, app: &tauri::AppHandle) {
        let audio_state = AudioState {
            devices: self.devices.clone(),
            device: self
                .devices
                .iter()
                .find(|d| {
                    if let Ok(info) = self.device.description() {
                        info.name() == d.name
                    } else {
                        false
                    }
                })
                .cloned(),
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
    let mut state = state.lock().unwrap();

    let device_option = state
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
        state.device = device;
        state.emit(&app);
        Ok(())
    } else {
        Err("Device not found".to_string())
    }
}
