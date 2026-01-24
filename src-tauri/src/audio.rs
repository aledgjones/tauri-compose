use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::AudioContextState;

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
}

pub struct AudioContext {
    host: Host,
    device: Device,
}

impl AudioContext {
    pub fn new() -> AudioContextState {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device available");

        Mutex::new(AudioContext { host, device })
    }

    pub fn get_available_devices(&self) -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
        let default_device_name = self.device.name().unwrap_or_default();
        let devices = self
            .host
            .output_devices()?
            .map(|device| {
                let name = device
                    .name()
                    .unwrap_or_else(|_| "Unknown Device".to_string());
                let is_default = name == default_device_name;
                DeviceInfo { name, is_default }
            })
            .collect();

        Ok(devices)
    }

    pub fn select_device(&mut self, device_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let device = self
            .host
            .output_devices()?
            .find(|device| device.name().unwrap_or_default() == device_name)
            .ok_or(format!("Device '{}' not found", device_name))?;

        self.device = device;
        Ok(())
    }
}

#[tauri::command]
pub fn audio_get_available_devices(
    state: tauri::State<'_, AudioContextState>,
) -> Result<Vec<DeviceInfo>, String> {
    let context = state.lock().map_err(|e| e.to_string())?;
    context.get_available_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn audio_select_device(
    state: tauri::State<'_, AudioContextState>,
    device_name: String,
) -> Result<(), String> {
    let mut context = state.lock().map_err(|e| e.to_string())?;
    context
        .select_device(&device_name)
        .map_err(|e| e.to_string())
}
