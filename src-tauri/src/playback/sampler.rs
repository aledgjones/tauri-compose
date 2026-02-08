use crate::playback::scvi::ScviContainer;
use crate::playback::streamer::{self, StreamBuffer, StreamRequest};
use crate::AudioContextState;
use crate::SamplerContextState;
use dirs;
use rand::Rng;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Deserialize, Clone)]
pub struct ArticulationMeta {
    id: String,
    name: String,
    range: Vec<[u8; 2]>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InstrumentType {
    Pitched,
    Percussion,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub path: String,
    pub author: String,
    pub version: String,
    pub cover_image: Option<String>,
    pub articulations: Vec<ArticulationMeta>,
    pub r#type: InstrumentType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SamplerState {
    pub plugins: Vec<PluginMeta>,
    pub loaded_plugin_ids: Vec<String>,
}

#[derive(Clone)]
pub struct CacheSample {
    /// 1 = mono, 2 = stereo
    pub channels: u8,
    /// Path to the .scvi container (for disk streaming)
    pub container_path: PathBuf,
    /// Source sample rate of the FLAC data
    pub source_sample_rate: u32,
    /// Byte offset of the FLAC data within the container
    pub flac_offset: u64,
    /// Byte length of the FLAC data within the container
    pub flac_length: u64,
}

#[derive(Clone)]
pub struct CacheSampleInfo {
    pub path: String,
    pub sample: CacheSample,
}

#[derive(Clone)]
pub struct CacheVelocityLayer {
    pub range: [u8; 2],
    pub samples: Vec<CacheSampleInfo>,
}

#[derive(Clone)]
pub struct CacheZone {
    pub pitch: u8,
    pub velocity_layers: Vec<CacheVelocityLayer>,
}

#[derive(Clone)]
pub struct CacheArticulation {
    pub id: String,
    pub name: String,
    pub zones: Vec<CacheZone>,
}

#[derive(Clone)]
pub struct Cache {
    pub meta: PluginMeta,
    pub articulations: FxHashMap<String, CacheArticulation>,
}

pub struct SamplerContext {
    plugins: Vec<PluginMeta>,
    cache: FxHashMap<String, Cache>,
}

impl SamplerContext {
    pub fn new() -> SamplerContextState {
        let plugins = Self::scan_plugins();
        Mutex::new(SamplerContext {
            plugins,
            cache: FxHashMap::default(),
        })
    }

    /// Clone the entire sampler cache (used by transport to take a playback snapshot)
    pub fn clone_cache(&self) -> FxHashMap<String, Cache> {
        self.cache.clone()
    }

    fn scan_plugins() -> Vec<PluginMeta> {
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
    ) -> Result<PluginMeta, Box<dyn std::error::Error>> {
        let container = ScviContainer::open(scvi_path)?;

        // Cache cover image
        let cover_image_path = if let Ok(Some(image_data)) = container.read_cover() {
            Some(Self::cache_cover_image(
                &container.manifest.meta.id,
                &image_data,
            )?)
        } else {
            None
        };

        // Build articulations metadata
        let articulations: Vec<ArticulationMeta> = container
            .manifest
            .articulations
            .iter()
            .map(|art| ArticulationMeta {
                id: art.id.clone(),
                name: art.name.clone(),
                range: art.range.clone(),
            })
            .collect();

        Ok(PluginMeta {
            id: container.manifest.meta.id,
            name: container.manifest.meta.name,
            path: scvi_path.to_string_lossy().to_string(),
            author: container.manifest.meta.author,
            version: container.manifest.meta.version,
            cover_image: cover_image_path,
            articulations,
            r#type: match container.manifest.meta.r#type {
                crate::playback::scvi::InstrumentType::Pitched => InstrumentType::Pitched,
                crate::playback::scvi::InstrumentType::Percussion => InstrumentType::Percussion,
            },
        })
    }

    fn cache_cover_image(
        meta_id: &str,
        image_data: &[u8],
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create cache directory
        let cache_dir = dirs::data_dir()
            .ok_or("Could not find data directory")?
            .join("Solo Compose")
            .join("covers");

        fs::create_dir_all(&cache_dir)?;

        // Generate cache filename based on meta.id
        let cache_filename = format!("{}.webp", meta_id);
        let cache_path = cache_dir.join(&cache_filename);

        // Always cache the image to ensure it's up to date
        let mut cache_file = File::create(&cache_path)?;
        cache_file.write_all(image_data)?;

        Ok(cache_path.to_string_lossy().to_string())
    }

    fn load_instrument(
        &self,
        plugin_id: &str,
        _target_sample_rate: u32,
    ) -> Result<Cache, Box<dyn std::error::Error>> {
        let plugin_info = self
            .plugins
            .iter()
            .find(|p| p.id == plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?
            .clone();

        let path = Path::new(&plugin_info.path);
        let container = ScviContainer::open(path)?;
        let container_path = path.to_path_buf();

        let mut articulations = FxHashMap::default();

        for art in &container.manifest.articulations {
            let mut zones = Vec::new();

            for zone in &art.zones {
                let mut velocity_layers = Vec::new();

                for vl in &zone.velocity_layers {
                    let mut samples = Vec::new();

                    for sample_ref in &vl.samples {
                        let index = container
                            .get_sample_index(sample_ref.sample_id)
                            .ok_or_else(|| {
                                format!("Sample {} not in index", sample_ref.sample_id)
                            })?;

                        samples.push(CacheSampleInfo {
                            path: format!("sample_{}", sample_ref.sample_id),
                            sample: CacheSample {
                                channels: index.channels,
                                container_path: container_path.clone(),
                                source_sample_rate: index.sample_rate,
                                flac_offset: index.file_offset,
                                flac_length: index.length,
                            },
                        });
                    }

                    velocity_layers.push(CacheVelocityLayer {
                        range: vl.range,
                        samples,
                    });
                }

                zones.push(CacheZone {
                    pitch: zone.pitch,
                    velocity_layers,
                });
            }

            articulations.insert(
                art.id.clone(),
                CacheArticulation {
                    id: art.id.clone(),
                    name: art.name.clone(),
                    zones,
                },
            );
        }

        Ok(Cache {
            meta: plugin_info,
            articulations,
        })
    }

    pub fn emit(&self, app: &AppHandle) {
        let state = SamplerState {
            plugins: self.plugins.clone(),
            loaded_plugin_ids: self.cache.keys().cloned().collect(),
        };

        if let Err(e) = app.emit("soloc://state/sampler", &state) {
            eprintln!("Failed to emit sampler state: {:?}", e);
        }
    }

    /// Sync sampler with score instruments: load assigned plugins and unload unused ones
    pub fn sync_with_instruments(
        &mut self,
        assigned_plugin_ids: &[String],
        target_sample_rate: u32,
    ) -> Result<(), String> {
        // Collect unique assigned plugin IDs
        let required_plugins: std::collections::HashSet<&String> =
            assigned_plugin_ids.iter().collect();

        // Load plugins that are assigned but not yet loaded
        for plugin_id in &required_plugins {
            if !self.cache.contains_key(*plugin_id) {
                match self.load_instrument(plugin_id, target_sample_rate) {
                    Ok(cache) => {
                        self.cache.insert((*plugin_id).clone(), cache);
                    }
                    Err(e) => {
                        eprintln!("Failed to load plugin {}: {}", plugin_id, e);
                    }
                }
            }
        }

        // Unload plugins that are no longer assigned to any instrument
        self.cache
            .retain(|plugin_id, _| required_plugins.contains(plugin_id));

        Ok(())
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn sampler_load_plugin(
    app: AppHandle,
    state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    plugin_id: String,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();
    let audio_ctx = audio_state.lock().unwrap();

    let result = match ctx.load_instrument(&plugin_id, audio_ctx.get_sample_rate()) {
        Ok(cache) => {
            ctx.cache.insert(plugin_id.clone(), cache);

            Ok(())
        }
        Err(e) => Err(format!("Failed to load instrument: {}", e)),
    };

    ctx.emit(&app);

    result
}

#[tauri::command(rename_all = "snake_case")]
pub fn sampler_rescan_plugins(
    app: AppHandle,
    state: State<'_, SamplerContextState>,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();

    // Rescan plugins from filesystem
    ctx.plugins = SamplerContext::scan_plugins();

    // Emit updated state to frontend
    ctx.emit(&app);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn sampler_sync_with_instruments(
    app: AppHandle,
    state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    assigned_plugin_ids: Vec<String>,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();
    let audio_ctx = audio_state.lock().unwrap();

    ctx.sync_with_instruments(&assigned_plugin_ids, audio_ctx.get_sample_rate())?;
    ctx.emit(&app);

    Ok(())
}

/// Find the nearest available zone for a given pitch
/// Returns the zone and the pitch offset in semitones (positive = shift up, negative = shift down)
/// For percussion instruments, only exact pitch matches are allowed (no pitch shifting)
fn find_nearest_zone(
    articulation: &CacheArticulation,
    target_pitch: u8,
    is_percussion: bool,
) -> Option<(&CacheZone, i32)> {
    if articulation.zones.is_empty() {
        return None;
    }

    // For percussion, only exact matches, no shifting
    if is_percussion {
        return articulation
            .zones
            .iter()
            .find(|z| z.pitch == target_pitch)
            .map(|z| (z, 0));
    }

    // Try exact match first
    if let Some(zone) = articulation.zones.iter().find(|z| z.pitch == target_pitch) {
        return Some((zone, 0));
    }

    // Find nearest zone
    let mut nearest_zone = &articulation.zones[0];
    let mut min_distance = (articulation.zones[0].pitch as i32 - target_pitch as i32).abs();

    for zone in &articulation.zones[1..] {
        let distance = (zone.pitch as i32 - target_pitch as i32).abs();
        if distance < min_distance {
            min_distance = distance;
            nearest_zone = zone;
        }
    }

    // Calculate pitch offset: how many semitones we need to shift
    // Positive means shift up, negative means shift down
    let pitch_offset = target_pitch as i32 - nearest_zone.pitch as i32;

    Some((nearest_zone, pitch_offset))
}

/// Play a note preview using the instrument's sampler cache.
/// If `articulation_id` is provided, uses that articulation; otherwise picks the first available.
/// Streams the sample from disk — same path as transport playback.
#[tauri::command(rename_all = "snake_case")]
pub fn sampler_audition(
    state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
    plugin_id: String,
    note: u8,
    velocity: u8,
    articulation_id: Option<String>,
    duration_ms: u32,
) -> Result<(), String> {
    let ctx = state.lock().unwrap();

    let cache = ctx
        .cache
        .get(&plugin_id)
        .ok_or_else(|| format!("Plugin not loaded: {}", plugin_id))?;

    // Use the specified articulation, or fall back to the first available
    let (articulation_id, articulation) = if let Some(ref id) = articulation_id {
        let art = cache
            .articulations
            .get(id)
            .ok_or_else(|| format!("Articulation not found: {}", id))?;
        (id.as_str(), art)
    } else {
        let (id, art) = cache
            .articulations
            .iter()
            .next()
            .ok_or_else(|| "No articulations available".to_string())?;
        (id.as_str(), art)
    };

    let velocity = velocity.clamp(1, 127);

    let is_percussion = matches!(cache.meta.r#type, InstrumentType::Percussion);

    let (zone, pitch_offset) = find_nearest_zone(articulation, note, is_percussion)
        .ok_or_else(|| format!("No zones available in articulation: {}", articulation_id))?;

    let velocity_layer = zone
        .velocity_layers
        .iter()
        .find(|vl| velocity >= vl.range[0] && velocity <= vl.range[1])
        .ok_or_else(|| format!("No velocity layer for velocity: {}", velocity))?;

    if velocity_layer.samples.is_empty() {
        return Err("No samples in velocity layer".to_string());
    }

    let sample_index = if velocity_layer.samples.len() > 1 {
        rand::rng().random_range(0..velocity_layer.samples.len())
    } else {
        0
    };
    let sample = &velocity_layer.samples[sample_index].sample;

    let audio_ctx = audio_state.lock().unwrap();
    let target_sample_rate = audio_ctx.get_sample_rate();
    let mixer_handle = audio_ctx.get_mixer_handle();
    drop(audio_ctx);

    let channels = sample.channels as usize;

    // Duration with 150ms release tail
    let duration_frames = (target_sample_rate as f64 * duration_ms as f64 / 1000.0).ceil() as usize;
    let release_tail = (target_sample_rate as f64 * 0.15).ceil() as usize;
    let max_frames = duration_frames + release_tail;

    let stream_buffer = Arc::new(StreamBuffer::new());

    let request = StreamRequest {
        container_path: sample.container_path.clone(),
        flac_offset: sample.flac_offset,
        flac_length: sample.flac_length,
        source_sample_rate: sample.source_sample_rate,
        target_sample_rate,
        channels,
        pitch_offset,
        max_output_frames: max_frames,
        fade_start_frame: duration_frames,
        volume_gain: 1.0,
        pan_left: 1.0,
        pan_right: 1.0,
        stream_buffer: stream_buffer.clone(),
    };

    streamer::start_stream(request);
    mixer_handle.play(stream_buffer, channels);

    Ok(())
}
