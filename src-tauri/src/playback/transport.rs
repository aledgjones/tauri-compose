use crate::playback::mixer::MixerHandle;
use crate::playback::sampler::{Cache, CacheArticulation};
use crate::playback::streamer::{self, StreamBuffer, StreamRequest};
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::Tick;
use crate::score::Score;
use crate::utils::UniqueId;
use crate::{AppContextState, AudioContextState, SamplerContextState};
use rand::Rng;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use tauri::State;

/// Live instrument mix parameters shared between the UI thread and the transport thread.
/// Updated in real-time by volume/pan/mute/solo commands.
#[derive(Clone)]
pub struct InstrumentMixParams {
    pub volume: u8,
    pub pan: u8,
    pub mute: bool,
    pub solo: bool,
}

/// Ticks per quarter note
const TICKS_PER_QUARTER: u64 = 32;

/// Default tempo in BPM
const DEFAULT_TEMPO: f64 = 120.0;

/// Seconds per tick at a given BPM
fn seconds_per_tick(bpm: f64) -> f64 {
    60.0 / (bpm * TICKS_PER_QUARTER as f64)
}

/// Map the score Articulation enum to the sampler articulation ID string.
fn find_articulation_id(
    articulation: &Articulation,
    articulations: &rustc_hash::FxHashMap<String, CacheArticulation>,
) -> Option<String> {
    let target = match articulation {
        Articulation::None => None,
        Articulation::Staccato => Some("staccato"),
        Articulation::Staccatissimo => Some("staccatissimo"),
        Articulation::Tenuto => Some("tenuto"),
        Articulation::StaccatoTenuto => Some("staccato-tenuto"),
    };

    // Try exact match first
    if let Some(target_id) = target {
        if articulations.contains_key(target_id) {
            return Some(target_id.to_string());
        }
    }

    // Fall back to the first available articulation (the "default" / natural articulation)
    articulations.keys().next().cloned()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransportState {
    pub playing: bool,
    pub current_tick: u64,
    pub tempo: f64,
}

/// Shared state between the transport thread and the main thread.
/// Exposed as a separate managed state so poll commands can read it
/// without locking TransportContext.
pub struct TransportShared {
    pub playing: AtomicBool,
    pub current_tick: AtomicU64,
    pub stop_signal: AtomicBool,
    /// Tempo stored as f64 bits via AtomicU64 for lock-free reads
    pub tempo_bits: AtomicU64,
    /// Live instrument mix parameters — updated by UI commands, read by transport thread.
    /// Uses RwLock so the transport thread (reader) never blocks other readers,
    /// and UI writes are infrequent.
    pub mix_params: RwLock<FxHashMap<UniqueId, InstrumentMixParams>>,
}

impl TransportShared {
    pub fn get_tempo(&self) -> f64 {
        f64::from_bits(self.tempo_bits.load(Ordering::Relaxed))
    }

    pub fn set_tempo(&self, bpm: f64) {
        self.tempo_bits.store(bpm.to_bits(), Ordering::Relaxed);
    }
}

/// Type alias for the separately managed Arc<TransportShared>
pub type TransportSharedState = Arc<TransportShared>;

pub struct TransportContext {
    shared: Arc<TransportShared>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl TransportContext {
    pub fn new() -> (Mutex<Self>, Arc<TransportShared>) {
        let shared = Arc::new(TransportShared {
            playing: AtomicBool::new(false),
            current_tick: AtomicU64::new(0),
            stop_signal: AtomicBool::new(false),
            tempo_bits: AtomicU64::new(DEFAULT_TEMPO.to_bits()),
            mix_params: RwLock::new(FxHashMap::default()),
        });

        let ctx = Mutex::new(TransportContext {
            shared: shared.clone(),
            thread_handle: None,
        });

        (ctx, shared)
    }
}

/// Render a single note by spawning a disk-streaming thread and submitting
/// the voice to the mixer. The streamer decodes from disk, applies pitch
/// shifting, envelope, and volume/pan, then pushes audio into a shared
/// buffer that the mixer drains in the audio callback.
fn render_note(
    cache: &Cache,
    articulation_id: &str,
    pitch: u8,
    velocity: u8,
    duration_seconds: f64,
    sample_rate: u32,
    mixer_handle: &MixerHandle,
    volume: u8,
    pan: u8,
) {
    let articulation = match cache.articulations.get(articulation_id) {
        Some(a) => a,
        None => return,
    };

    let velocity = velocity.clamp(1, 127);

    let is_percussion = matches!(
        cache.meta.r#type,
        crate::playback::sampler::InstrumentType::Percussion
    );

    // Find nearest zone
    let (zone, pitch_offset) = {
        if articulation.zones.is_empty() {
            return;
        }

        if is_percussion {
            // For percussion, only exact matches
            if let Some(zone) = articulation.zones.iter().find(|z| z.pitch == pitch) {
                (zone, 0)
            } else {
                return;
            }
        } else {
            // For pitched instruments, find nearest
            let mut nearest = &articulation.zones[0];
            let mut min_dist = (nearest.pitch as i32 - pitch as i32).abs();

            for zone in &articulation.zones[1..] {
                let dist = (zone.pitch as i32 - pitch as i32).abs();
                if dist < min_dist {
                    min_dist = dist;
                    nearest = zone;
                }
            }

            let offset = pitch as i32 - nearest.pitch as i32;
            (nearest, offset)
        }
    };

    // Find velocity layer
    let velocity_layer = match zone
        .velocity_layers
        .iter()
        .find(|vl| velocity >= vl.range[0] && velocity <= vl.range[1])
    {
        Some(vl) => vl,
        None => return,
    };

    if velocity_layer.samples.is_empty() {
        return;
    }

    // Random round-robin
    let sample_index = if velocity_layer.samples.len() > 1 {
        rand::rng().random_range(0..velocity_layer.samples.len())
    } else {
        0
    };

    let sample = &velocity_layer.samples[sample_index].sample;
    let channels = sample.channels as usize;

    // Duration and envelope
    let duration_frames = (duration_seconds * sample_rate as f64).ceil() as usize;
    let release_frames = (sample_rate as f64 * 0.15).ceil() as usize;
    let max_frames = duration_frames + release_frames;

    let (vol_gain, pan_left, pan_right) = instrument_gain_and_pan(volume, pan);

    let stream_buffer = Arc::new(StreamBuffer::new());

    let request = StreamRequest {
        container_path: sample.container_path.clone(),
        flac_offset: sample.flac_offset,
        flac_length: sample.flac_length,
        source_sample_rate: sample.source_sample_rate,
        target_sample_rate: sample_rate,
        channels,
        pitch_offset,
        max_output_frames: max_frames,
        fade_start_frame: duration_frames,
        volume_gain: vol_gain,
        pan_left,
        pan_right,
        stream_buffer: stream_buffer.clone(),
    };

    streamer::start_stream(request);
    mixer_handle.play(stream_buffer, channels);
}

/// Collect all notes at a given tick from all instrument tracks in the active flow.
/// Returns (plugin_id, articulation, pitch, velocity, duration_ticks, instrument_id) tuples.
fn collect_notes_at_tick(
    score: &Score,
    flow_id: UniqueId,
    tick: Tick,
) -> Vec<(String, Articulation, u8, u8, u64, UniqueId)> {
    let mut notes = Vec::new();

    let flow = match score.flows.by_id.get(&flow_id) {
        Some(f) => f,
        None => return notes,
    };

    // Walk all players assigned to this flow
    for player_id in &flow.player_ids {
        let player = match score.players.by_id.get(player_id) {
            Some(p) => p,
            None => continue,
        };

        for instrument_id in &player.instrument_ids {
            let instrument = match score.instruments.by_id.get(instrument_id) {
                Some(i) => i,
                None => continue,
            };

            let plugin_id = match &instrument.assigned_plugin_id {
                Some(id) => id.clone(),
                None => continue,
            };

            // Find staves for this instrument in this flow
            for stave_id in &instrument.stave_ids {
                let stave = match flow.staves.by_id.get(stave_id) {
                    Some(s) => s,
                    None => continue,
                };

                // Walk the tracks of this stave
                for track_id in &stave.track_ids {
                    let track = match score.tracks.by_id.get(track_id) {
                        Some(t) => t,
                        None => continue,
                    };

                    // Get tones at this tick
                    let tones = track.get_tones_at_tick(&tick);
                    for tone in tones {
                        notes.push((
                            plugin_id.clone(),
                            tone.articulation.clone(),
                            tone.pitch.int,
                            tone.velocity.int(),
                            tone.duration,
                            *instrument_id,
                        ));
                    }
                }
            }
        }
    }

    notes
}

/// Get volume and pan for an instrument. Volume is 0-127 mapped to 0.0-1.0 gain.
/// Pan is 0-127 where 64 is center, mapped to left/right gain multipliers.
fn instrument_gain_and_pan(volume: u8, pan: u8) -> (f32, f32, f32) {
    let gain = volume as f32 / 127.0;
    // Pan: 0=left, 64=center, 127=right
    // Use constant-power panning
    let pan_norm = pan as f32 / 127.0; // 0..1
    let angle = pan_norm * std::f32::consts::FRAC_PI_2; // 0..PI/2
    let left_gain = angle.cos();
    let right_gain = angle.sin();
    (gain, left_gain, right_gain)
}

/// The transport playback thread. Walks through ticks at tempo rate,
/// collects notes, renders them, and submits to the mixer.
/// A snapshot of all data the transport thread needs for playback.
/// Cloned once at play-start so the thread never locks AppContextState or SamplerContextState.
struct PlaybackSnapshot {
    score: Score,
    sampler_cache: FxHashMap<String, Cache>,
    flow_id: UniqueId,
    flow_length: u64,
    sample_rate: u32,
}

fn transport_thread(
    shared: Arc<TransportShared>,
    tempo: f64,
    snapshot: PlaybackSnapshot,
    mixer_handle: crate::playback::mixer::MixerHandle,
) {
    let spt = seconds_per_tick(tempo);
    let tick_duration = std::time::Duration::from_secs_f64(spt);
    let mut next_tick_deadline = std::time::Instant::now() + tick_duration;

    loop {
        // Check stop signal
        if shared.stop_signal.load(Ordering::Acquire) {
            break;
        }

        let current_tick = shared.current_tick.load(Ordering::Relaxed);

        // Check if we've reached the end of the flow
        if current_tick >= snapshot.flow_length {
            break;
        }

        // Snapshot the live mix params into a local copy — hold the lock as briefly as possible
        // so that the heavy render_note work below is entirely lock-free.
        let (local_mix, has_solo) = {
            let mix = shared.mix_params.read().unwrap();
            let has_solo = mix.values().any(|p| p.solo);
            (mix.clone(), has_solo)
        };

        // Collect notes at this tick from the snapshot (no mutex needed)
        let raw_notes = collect_notes_at_tick(&snapshot.score, snapshot.flow_id, current_tick);

        // Render and submit each note
        for (plugin_id, articulation, pitch, velocity, duration_ticks, instrument_id) in raw_notes {
            // Get live mix params for this instrument
            let params = match local_mix.get(&instrument_id) {
                Some(p) => p,
                None => continue,
            };

            // Skip muted instruments
            if params.mute {
                continue;
            }

            // If solo mode is active, only play soloed instruments
            if has_solo && !params.solo {
                continue;
            }

            let duration_seconds = duration_ticks as f64 * spt;

            // Look up the sampler cache from the snapshot
            let cache = match snapshot.sampler_cache.get(&plugin_id) {
                Some(c) => c,
                None => continue,
            };

            let art_id = match find_articulation_id(&articulation, &cache.articulations) {
                Some(id) => id,
                None => continue,
            };

            render_note(
                cache,
                &art_id,
                pitch,
                velocity,
                duration_seconds,
                snapshot.sample_rate,
                &mixer_handle,
                params.volume,
                params.pan,
            );
        }

        // Advance tick
        shared
            .current_tick
            .store(current_tick + 1, Ordering::Relaxed);

        // Sleep until the next tick deadline — accounts for time spent rendering
        let now = std::time::Instant::now();
        if next_tick_deadline > now {
            std::thread::sleep(next_tick_deadline - now);
        }
        next_tick_deadline += tick_duration;
    }

    // Playback ended — mark as not playing via atomic
    // (transport_stop already set playing=false if externally stopped)
    if !shared.stop_signal.load(Ordering::Acquire) {
        shared.playing.store(false, Ordering::Release);
    }
}

// -- Tauri Commands --

#[tauri::command(rename_all = "snake_case")]
pub fn transport_play(
    state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
    sampler_state: State<'_, SamplerContextState>,
    audio_state: State<'_, AudioContextState>,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();

    // Already playing? Ignore.
    if ctx.shared.playing.load(Ordering::Acquire) {
        return Ok(());
    }

    // Build a snapshot of everything the thread needs — snapshot locks are brief and one-time
    let (snapshot, mixer_handle) = {
        let app_ctx = app_state.lock().unwrap();
        let flow_id = app_ctx
            .ui
            .flow_id
            .or_else(|| app_ctx.score.flows.order.first().copied());

        let flow_id = match flow_id {
            Some(id) => id,
            None => return Ok(()), // No flow to play
        };

        let flow_length = app_ctx
            .score
            .flows
            .by_id
            .get(&flow_id)
            .map(|f| f.length)
            .unwrap_or(0);

        let sampler = sampler_state.lock().unwrap();
        let sampler_cache = sampler.clone_cache();
        drop(sampler);

        let audio = audio_state.lock().unwrap();
        let sample_rate = audio.get_sample_rate();
        let mixer_handle = audio.get_mixer_handle().clone();
        drop(audio);

        let snapshot = PlaybackSnapshot {
            score: app_ctx.score.clone(),
            sampler_cache,
            flow_id,
            flow_length,
            sample_rate,
        };

        (snapshot, mixer_handle)
    };

    // Populate live mix params from the score
    {
        let mut mix = ctx.shared.mix_params.write().unwrap();
        mix.clear();
        for (id, inst) in &snapshot.score.instruments.by_id {
            mix.insert(
                *id,
                InstrumentMixParams {
                    volume: inst.volume,
                    pan: inst.pan,
                    mute: inst.mute,
                    solo: inst.solo,
                },
            );
        }
    }

    // Drop any old finished thread handle
    let _ = ctx.thread_handle.take();

    // Reset stop signal
    ctx.shared.stop_signal.store(false, Ordering::Release);
    ctx.shared.playing.store(true, Ordering::Release);

    let shared = ctx.shared.clone();
    let tempo = ctx.shared.get_tempo();

    ctx.thread_handle = Some(std::thread::spawn(move || {
        transport_thread(shared, tempo, snapshot, mixer_handle);
    }));

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_pause(state: State<'_, Mutex<TransportContext>>) -> Result<(), String> {
    let ctx = state.lock().unwrap();

    if ctx.shared.playing.load(Ordering::Acquire) {
        ctx.shared.stop_signal.store(true, Ordering::Release);
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_stop(state: State<'_, Mutex<TransportContext>>) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();

    // Signal the thread to stop
    ctx.shared.stop_signal.store(true, Ordering::Release);
    ctx.shared.playing.store(false, Ordering::Release);

    // Drop the old thread handle (let thread exit on its own — no join to avoid deadlock)
    let _ = ctx.thread_handle.take();

    // Reset to beginning
    ctx.shared.current_tick.store(0, Ordering::Relaxed);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_seek(state: State<'_, Mutex<TransportContext>>, tick: u64) -> Result<(), String> {
    let ctx = state.lock().unwrap();
    ctx.shared.current_tick.store(tick, Ordering::Relaxed);
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_set_tempo(state: State<'_, TransportSharedState>, bpm: f64) -> Result<(), String> {
    state.set_tempo(bpm.clamp(20.0, 400.0));
    Ok(())
}

/// Lightweight poll command — reads atomics directly, no mutex lock needed.
/// Frontend calls this via requestAnimationFrame during playback.
#[tauri::command(rename_all = "snake_case")]
pub fn transport_get_state(state: State<'_, TransportSharedState>) -> TransportState {
    TransportState {
        playing: state.playing.load(Ordering::Relaxed),
        current_tick: state.current_tick.load(Ordering::Relaxed),
        tempo: state.get_tempo(),
    }
}
