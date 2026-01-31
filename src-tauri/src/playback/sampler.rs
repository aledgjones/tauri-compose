use crate::SamplerContextState;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Read;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use zip;

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginInfo {
    name: String,
    path: String,
    format: String,
}

#[derive(Deserialize)]
struct MetaJson {
    meta: MetaData,
}

#[derive(Deserialize)]
struct MetaData {
    name: String,
    format: String,
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
                        if let Ok(plugin_info) = Self::read_plugin_meta(&entry.path()) {
                            plugins.push(plugin_info);
                        }
                    }
                }
            }
        }
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        plugins
    }

    fn read_plugin_meta(
        scvi_path: &std::path::Path,
    ) -> Result<PluginInfo, Box<dyn std::error::Error>> {
        // Open the ZIP archive
        let file = std::fs::File::open(scvi_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Find and extract meta.json
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name() == "meta.json" {
                // Extract the content
                let mut content = String::new();
                file.read_to_string(&mut content)?;

                match serde_json::from_str::<MetaJson>(&content) {
                    Ok(meta_json) => {
                        return Ok(PluginInfo {
                            name: meta_json.meta.name,
                            path: scvi_path.to_string_lossy().to_string(),
                            format: meta_json.meta.format,
                        });
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to parse meta.json in {}: {}",
                            scvi_path.display(),
                            e
                        );
                        return Err(e.into());
                    }
                }
            }
        }

        eprintln!("meta.json not found in archive: {}", scvi_path.display());
        Err("meta.json not found in archive".into())
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
