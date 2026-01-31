use crate::SamplerContextState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginInfo {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SamplerState {
    pub plugins: Vec<PluginInfo>,
}

pub struct SamplerContext {
    plugins: Vec<PluginInfo>,
}

impl SamplerContext {
    pub fn new() -> SamplerContextState {
        let plugins = Self::scan_plugins();
        Mutex::new(SamplerContext { plugins })
    }

    fn scan_plugins() -> Vec<PluginInfo> {
        let path = "/Library/Audio/Plug-Ins/SCVI";
        let mut plugins = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "scvi" {
                        if let Some(name) = entry.path().file_stem() {
                            plugins.push(PluginInfo {
                                name: name.to_string_lossy().to_string(),
                                path: entry.path().to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
        }
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        plugins
    }

    pub fn emit(&self, app: &AppHandle) {
        let state = SamplerState {
            plugins: self.plugins.clone(),
        };
        if let Err(e) = app.emit("soloc://state/sampler", &state) {
            eprintln!("Failed to emit sampler plugins: {:?}", e);
        }
    }
}
