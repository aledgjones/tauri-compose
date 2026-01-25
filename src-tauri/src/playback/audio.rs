use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::AudioContextState;

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AudioState {
    pub devices: Vec<DeviceInfo>,
    pub selected_device: Option<String>,
}

pub struct AudioContext {
    host: Host,
    device: Device,
    devices: Vec<DeviceInfo>,
    selected_device: Option<String>,
}

impl AudioContext {
    pub fn new() -> AudioContextState {
        let host = cpal::default_host();
        let default_device = host
            .default_output_device()
            .expect("No default output device available");

        let default_device_name = if let Ok(description) = default_device.description() {
            Some(description.name().to_string())
        } else {
            None
        };

        // Get all available devices
        let devices = host
            .output_devices()
            .expect("Failed to get output devices")
            .filter_map(|d| {
                if let Ok(info) = d.description() {
                    return Some(DeviceInfo {
                        name: info.name().to_string(),
                        is_default: default_device_name == Some(info.name().to_string()),
                    });
                }

                None
            })
            .collect::<Vec<_>>();

        Mutex::new(AudioContext {
            host,
            device: default_device,
            devices,
            selected_device: default_device_name,
        })
    }

    pub fn emit(&self, app: &tauri::AppHandle) {
        let audio_state = AudioState {
            devices: self.devices.clone(),
            selected_device: self.selected_device.clone(),
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
    device_name: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    let device_option = state
        .host
        .output_devices()
        .map_err(|e| e.to_string())?
        .find(|d| {
            if let Ok(info) = d.description() {
                info.name() == device_name
            } else {
                false
            }
        });

    if let Some(device) = device_option {
        state.device = device;
        state.selected_device = Some(device_name);
        state.emit(&app);
        Ok(())
    } else {
        Err(format!("Device '{}' not found", device_name))
    }
}
