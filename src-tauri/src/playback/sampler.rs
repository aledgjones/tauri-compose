use crate::playback::scvi::ScviContainer;
use crate::AudioContextState;
use crate::SamplerContextState;
use audioadapter_buffers::direct::InterleavedSlice;
use dirs;
use rand::Rng;
use rubato::{Fft, FixedSync, Indexing, Resampler};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::sync::Mutex;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Deserialize, Clone)]
pub struct ArticulationMeta {
    id: String,
    name: String,
    range: [u8; 2],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginMeta {
    id: String,
    name: String,
    path: String,
    author: String,
    version: String,
    cover_image: Option<String>,
    articulations: Vec<ArticulationMeta>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SamplerState {
    pub plugins: Vec<PluginMeta>,
    pub loaded_plugin_ids: Vec<String>,
}

#[derive(Clone)]
pub struct CacheSample {
    pub data: Vec<f32>,   // Audio samples as f32 (-1.0 to 1.0)
    pub sample_rate: u32, // Original sample rate
    pub channels: u8,     // 1 = mono, 2 = stereo
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
                range: art.range,
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
        target_sample_rate: u32,
    ) -> Result<Cache, Box<dyn std::error::Error>> {
        let plugin_info = self
            .plugins
            .iter()
            .find(|p| p.id == plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?
            .clone();

        let path = Path::new(&plugin_info.path);
        let container = ScviContainer::open(path)?;

        let mut articulations = FxHashMap::default();

        for art in &container.manifest.articulations {
            let mut zones = Vec::new();

            for zone in &art.zones {
                let mut velocity_layers = Vec::new();

                for vl in &zone.velocity_layers {
                    let mut samples = Vec::new();

                    for sample_ref in &vl.samples {
                        // Read the raw FLAC data from the container
                        let flac_data = container.read_sample_data(sample_ref.sample_id)?;

                        // Decode FLAC to f32 samples
                        let mut cursor = Cursor::new(flac_data);
                        let cached_sample =
                            self.decode_with_symphonia(&mut cursor, target_sample_rate)?;

                        samples.push(CacheSampleInfo {
                            path: format!("sample_{}", sample_ref.sample_id),
                            sample: cached_sample,
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

    fn decode_with_symphonia<R: Read>(
        &self,
        reader: &mut R,
        target_sample_rate: u32,
    ) -> Result<CacheSample, Box<dyn std::error::Error>> {
        // Read all data into a vector
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        // Create a media source stream
        let cursor = Cursor::new(data);
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

        // Create a hint to help the format registry
        let mut hint = Hint::new();
        hint.with_extension("flac");

        // Probe the media source
        let probed = get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        // Get the format reader
        let mut format = probed.format;

        // Find the first audio track
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or("No audio track found")?;

        // Create a decoder
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())?;

        let track_id = track.id;

        let mut sample_buf = None;
        let mut samples = Vec::new();
        let mut sample_rate = 44100;
        let mut channels = 1;

        // Decode all packets
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    // The track list has been changed, seek back to the beginning
                    continue;
                }
                Err(symphonia::core::errors::Error::IoError(_)) => {
                    // End of stream
                    break;
                }
                Err(err) => return Err(err.into()),
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Get the sample buffer specification
                    if sample_buf.is_none() {
                        sample_rate = decoded.spec().rate;
                        channels = decoded.spec().channels.count();
                        sample_buf = Some(SampleBuffer::<f32>::new(
                            decoded.capacity() as u64,
                            *decoded.spec(),
                        ));
                    }

                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(decoded);
                        samples.extend_from_slice(buf.samples());
                    }
                }
                Err(symphonia::core::errors::Error::IoError(_)) => {
                    // End of stream
                    break;
                }
                Err(err) => return Err(err.into()),
            }
        }

        // Resample to target rate if necessary
        let samples = if sample_rate != target_sample_rate {
            Self::resample_sinc(&samples, sample_rate, target_sample_rate, channels)
        } else {
            samples
        };

        Ok(CacheSample {
            data: samples,
            sample_rate: target_sample_rate,
            channels: channels as u8,
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

    fn resample_sinc(samples: &[f32], from_rate: u32, to_rate: u32, channels: usize) -> Vec<f32> {
        if from_rate == to_rate {
            return samples.to_vec();
        }

        let chunk_size = 1024;

        let mut resampler = Fft::new(
            from_rate as usize,
            to_rate as usize,
            chunk_size,
            channels,
            channels,
            FixedSync::Both,
        )
        .expect("Failed to create resampler");

        // Pad the input to a full chunk boundary so the resampler processes all real audio
        let nbr_input_frames = samples.len() / channels;
        let input_frames_next = resampler.input_frames_next();
        let padded_frames =
            ((nbr_input_frames + input_frames_next - 1) / input_frames_next) * input_frames_next;

        let padded_samples = if padded_frames > nbr_input_frames {
            let mut buf = samples.to_vec();
            buf.resize(padded_frames * channels, 0.0);
            buf
        } else {
            samples.to_vec()
        };

        let input_adapter = InterleavedSlice::new(&padded_samples, channels, padded_frames)
            .expect("Failed to create input adapter");

        // Calculate expected output frames from the original (unpadded) input
        let expected_output_frames =
            (nbr_input_frames as f64 * to_rate as f64 / from_rate as f64).ceil() as usize;

        // Create output buffer with headroom for the padded input
        let max_output_frames =
            (padded_frames as f64 * to_rate as f64 / from_rate as f64).ceil() as usize + chunk_size;
        let mut outdata = vec![0.0f32; channels * max_output_frames];
        let outdata_capacity = outdata.len() / channels;
        let mut output_adapter =
            InterleavedSlice::new_mut(&mut outdata, channels, outdata_capacity)
                .expect("Failed to create output adapter");

        // Process in chunks
        let mut indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: None,
        };

        let mut input_frames_left = padded_frames;
        let mut frames_next = resampler.input_frames_next();

        while input_frames_left >= frames_next {
            let (frames_read, frames_written) = resampler
                .process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))
                .expect("Resampling failed");

            indexing.input_offset += frames_read;
            indexing.output_offset += frames_written;
            input_frames_left -= frames_read;
            frames_next = resampler.input_frames_next();
        }

        // Trim to the expected output length (drop any silence from padding)
        let output_frames = indexing.output_offset.min(expected_output_frames);
        outdata.truncate(channels * output_frames);
        outdata
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
fn find_nearest_zone(
    articulation: &CacheArticulation,
    target_pitch: u8,
) -> Option<(&CacheZone, i32)> {
    if articulation.zones.is_empty() {
        return None;
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

    let (zone, pitch_offset) = find_nearest_zone(articulation, note)
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
    let source = &sample.data;

    // Cap audition at the given duration, plus a 150ms release tail
    let duration_frames = (target_sample_rate as f64 * duration_ms as f64 / 1000.0).ceil() as usize;
    let release_tail = (target_sample_rate as f64 * 0.15).ceil() as usize;
    let max_frames = duration_frames + release_tail;

    let mut audio_data = if pitch_offset != 0 {
        let pitch_ratio = 2.0_f32.powf(pitch_offset as f32 / 12.0);
        let input_frames = source.len() / channels;
        let output_frames = (input_frames as f64 / pitch_ratio as f64).floor() as usize;
        let capped_frames = output_frames.min(max_frames);

        let mut shifted = Vec::with_capacity(capped_frames * channels);
        for frame in 0..capped_frames {
            let src_pos = frame as f64 * pitch_ratio as f64;
            let idx = src_pos.floor() as usize;
            let frac = src_pos - idx as f64;

            for ch in 0..channels {
                let i0 = idx * channels + ch;
                let s0 = source[i0];
                let s1 = if idx + 1 < input_frames {
                    source[(idx + 1) * channels + ch]
                } else {
                    s0
                };
                shifted.push(s0 + (s1 - s0) * frac as f32);
            }
        }
        shifted
    } else {
        let natural_frames = source.len() / channels.max(1);
        let capped_samples = natural_frames.min(max_frames) * channels;
        source[..capped_samples].to_vec()
    };

    let total_frames = audio_data.len() / channels.max(1);
    let natural_frames = source.len() / channels.max(1);
    let truncated = natural_frames > total_frames;

    if truncated {
        // Fade out starting at the 1-second sustain boundary
        let fade_start = duration_frames.min(total_frames);
        let actual_fade_len = total_frames.saturating_sub(fade_start);
        if actual_fade_len > 0 {
            for frame in fade_start..total_frames {
                let gain = 1.0 - (frame - fade_start) as f32 / actual_fade_len as f32;
                for ch in 0..channels {
                    audio_data[frame * channels + ch] *= gain;
                }
            }
        }
    } else {
        // Sample ended naturally — tiny 5ms anti-click fade
        let anti_click = ((target_sample_rate as f32 * 0.005) as usize).min(total_frames);
        let fade_start = total_frames - anti_click;
        for frame in fade_start..total_frames {
            let gain = 1.0 - (frame - fade_start) as f32 / anti_click as f32;
            for ch in 0..channels {
                audio_data[frame * channels + ch] *= gain;
            }
        }
    }

    mixer_handle.play(audio_data, channels);

    Ok(())
}
