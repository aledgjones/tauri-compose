use crate::{AudioContextState, VstContextState};
use rack::prelude::*;
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone, Debug)]
pub struct VstPluginInfoMeta {
    pub id: String,
    pub name: String,
}

pub struct VstPluginInfo {
    pub meta: VstPluginInfoMeta,
    pub info: PluginInfo,
}

#[derive(Serialize, Clone, Debug)]
pub struct VstPluginMeta {
    pub id: u32,
    pub name: String,
    pub plugin_id: String,
}

pub struct VstPlugin {
    pub meta: VstPluginMeta,
    pub instance: Plugin,
}

pub struct VstContext {
    scanner: Scanner,
    plugins: FxHashMap<String, VstPluginInfo>,
    instances: FxHashMap<u32, VstPlugin>,
    next_plugin_id: u32,
}

#[derive(Serialize, Clone)]
pub struct VstState {
    plugins: FxHashMap<String, VstPluginInfoMeta>,
    instances: FxHashMap<u32, VstPluginMeta>,
}

impl VstContext {
    pub fn new() -> VstContextState {
        let scanner = Scanner::new().unwrap();
        let plugin_infos = scanner.scan().unwrap();
        let registry = VstContext::create_plugin_list(&plugin_infos);

        Mutex::new(VstContext {
            scanner,
            plugins: registry,
            instances: FxHashMap::default(),
            next_plugin_id: 0,
        })
    }

    pub fn emit(&self, app: &tauri::AppHandle) {
        let plugins = self
            .plugins
            .iter()
            .map(|(id, info)| (id.clone(), info.meta.clone()))
            .collect();
        let instances = self
            .instances
            .iter()
            .map(|(id, plugin)| (*id, plugin.meta.clone()))
            .collect();

        let vst_state = VstState { plugins, instances };

        if let Err(e) = app.emit("soloc://state/vst", &vst_state) {
            eprintln!("Failed to emit VST state: {:?}", e);
        }
    }

    fn create_plugin_list(plugin_infos: &[PluginInfo]) -> FxHashMap<String, VstPluginInfo> {
        let mut registry = FxHashMap::default();

        for plugin_info in plugin_infos {
            if plugin_info.plugin_type == PluginType::Instrument {
                registry.insert(
                    plugin_info.unique_id.clone(),
                    VstPluginInfo {
                        meta: VstPluginInfoMeta {
                            id: plugin_info.unique_id.clone(),
                            name: plugin_info.name.clone(),
                        },
                        info: plugin_info.clone(),
                    },
                );
            };
        }

        registry
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn vst_plugin_load(
    app: AppHandle,
    vst_state: State<'_, VstContextState>,
    audio_state: State<'_, AudioContextState>,
    plugin_id: String,
) -> std::result::Result<(), String> {
    let mut vst = vst_state.lock().unwrap();
    let audio = audio_state.lock().unwrap();

    let sample_rate = audio.get_current_sample_rate();

    let plugin_entry = vst
        .plugins
        .get(&plugin_id)
        .ok_or_else(|| format!("Plugin {} not found in registry", plugin_id))?;

    let mut instance: Plugin = vst
        .scanner
        .load(&plugin_entry.info)
        .map_err(|e| format!("Failed to load plugin {}: {}", plugin_entry.meta.name, e))?;

    instance.initialize(sample_rate as f64, 512).map_err(|e| {
        format!(
            "Failed to initialize plugin {}: {}",
            plugin_entry.meta.name, e
        )
    })?;

    let instance_id = vst.next_plugin_id;

    let vst_plugin = VstPlugin {
        meta: VstPluginMeta {
            id: instance_id,
            name: plugin_entry.meta.name.clone(),
            plugin_id: plugin_id.clone(),
        },
        instance,
    };

    vst.instances.insert(instance_id, vst_plugin);
    vst.next_plugin_id += 1;
    vst.emit(&app);
    Ok(())
}
