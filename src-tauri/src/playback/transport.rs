use crate::playback::mixer::MixerHandle;
use crate::playback::sampler::{Cache, CacheArticulation};
use crate::playback::streamer::{self, StreamBuffer, StreamRequest};
use crate::score::components::duration::Tick;
use crate::score::entries::tempo::{EffectiveTempoResolver, DEFAULT_TEMPO_BPM};
use crate::score::Score;
use crate::utils::UniqueId;
use crate::{AppContext, AppContextState, AudioContextState, SamplerContextState};
use rand::RngExt;
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

/// Seconds per tick at a given BPM
fn seconds_per_tick(bpm: f64) -> f64 {
    60.0 / (bpm * TICKS_PER_QUARTER as f64)
}

fn join_transport_thread(handle: Option<std::thread::JoinHandle<()>>) -> Result<(), String> {
    if let Some(handle) = handle {
        handle
            .join()
            .map_err(|_| String::from("Transport thread panicked"))?;
    }

    Ok(())
}

fn active_flow_id(app_ctx: &AppContext) -> Option<UniqueId> {
    app_ctx
        .ui
        .flow_id
        .or_else(|| app_ctx.score.flows.order.first().copied())
}

fn flow_tempo_resolver(score: &Score, flow_id: UniqueId) -> EffectiveTempoResolver {
    let flow = score
        .flows
        .by_id
        .get(&flow_id)
        .unwrap_or_else(|| panic!("Missing flow {} while resolving transport tempo", flow_id));
    let master_track = score.tracks.by_id.get(&flow.master_id).unwrap_or_else(|| {
        panic!(
            "Missing flow master track {} while resolving transport tempo",
            flow.master_id
        )
    });

    master_track.effective_tempo_resolver()
}

fn resolve_score_tempo_bpm(score: &Score, flow_id: Option<UniqueId>, tick: Tick) -> f64 {
    match flow_id {
        Some(flow_id) => flow_tempo_resolver(score, flow_id).resolve(tick).bpm_f64(),
        None => f64::from(DEFAULT_TEMPO_BPM),
    }
}

fn resolve_app_tempo_bpm(app_ctx: &AppContext, tick: Tick) -> f64 {
    resolve_score_tempo_bpm(&app_ctx.score, active_flow_id(app_ctx), tick)
}

fn clamp_tick_to_score_flow(score: &Score, flow_id: Option<UniqueId>, tick: Tick) -> Tick {
    let Some(flow_id) = flow_id else {
        return tick;
    };

    let Some(flow) = score.flows.by_id.get(&flow_id) else {
        return tick;
    };

    if flow.length == 0 {
        0
    } else {
        tick.min(flow.length.saturating_sub(1))
    }
}

fn publish_resolved_tempo(
    shared: &TransportShared,
    tempo_resolver: &EffectiveTempoResolver,
    tick: Tick,
) -> f64 {
    shared.current_tick.store(tick, Ordering::Relaxed);
    let bpm = tempo_resolver.resolve(tick).bpm_f64();
    shared.set_tempo(bpm);
    bpm
}

fn publish_app_tempo(shared: &TransportShared, app_ctx: &AppContext, tick: Tick) -> f64 {
    shared.current_tick.store(tick, Ordering::Relaxed);
    let bpm = resolve_app_tempo_bpm(app_ctx, tick);
    shared.set_tempo(bpm);
    bpm
}

fn accumulate_tick_deadline(
    next_tick_deadline: std::time::Duration,
    bpm: f64,
) -> std::time::Duration {
    next_tick_deadline + std::time::Duration::from_secs_f64(seconds_per_tick(bpm))
}

fn playback_can_start(current_tick: Tick, flow_length: Tick) -> bool {
    current_tick < flow_length
}

/// Map the score Articulation enum to the sampler articulation ID string.
fn find_articulation_id(
    articulations: &rustc_hash::FxHashMap<String, CacheArticulation>,
) -> Option<String> {
    // TODO: convert this mapping into a dynamic mapping in the manifest,
    // e.g. via a "default_articulation" field or similar — this would allow
    // non-standard articulations and more flexible mappings in the future.
    if articulations.contains_key("sustain") {
        return Some("sustain".to_string());
    }

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
            tempo_bits: AtomicU64::new(f64::from(DEFAULT_TEMPO_BPM).to_bits()),
            mix_params: RwLock::new(FxHashMap::default()),
        });

        let ctx = Mutex::new(TransportContext {
            shared: shared.clone(),
            thread_handle: None,
        });

        (ctx, shared)
    }
}

pub(crate) fn stop_transport_playback(
    state: &Mutex<TransportContext>,
) -> Result<TransportSharedState, String> {
    let (shared, handle) = {
        let mut ctx = state.lock().unwrap();

        ctx.shared.stop_signal.store(true, Ordering::Release);
        ctx.shared.playing.store(false, Ordering::Release);

        (ctx.shared.clone(), ctx.thread_handle.take())
    };

    join_transport_thread(handle)?;
    shared.stop_signal.store(false, Ordering::Release);

    Ok(shared)
}

pub(crate) fn sync_transport_to_app_state(
    shared: &TransportSharedState,
    app_ctx: &AppContext,
    tick: Tick,
) -> f64 {
    let tick = clamp_tick_to_score_flow(&app_ctx.score, active_flow_id(app_ctx), tick);
    publish_app_tempo(shared, app_ctx, tick)
}

/// Render a single note by spawning a disk-streaming thread and submitting
/// the voice to the mixer. The streamer decodes from disk, applies pitch
/// shifting, envelope, and volume/pan, then pushes audio into a shared
/// buffer that the mixer drains in the audio callback.
#[allow(clippy::too_many_arguments)]
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
) -> Vec<(String, u8, u8, u64, UniqueId)> {
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
    tempo_resolver: EffectiveTempoResolver,
}

fn transport_thread(
    shared: Arc<TransportShared>,
    snapshot: PlaybackSnapshot,
    mixer_handle: crate::playback::mixer::MixerHandle,
) {
    let start = std::time::Instant::now();
    let mut next_tick_offset = std::time::Duration::ZERO;

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

        let bpm = publish_resolved_tempo(&shared, &snapshot.tempo_resolver, current_tick);
        let spt = seconds_per_tick(bpm);

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
        for (plugin_id, pitch, velocity, duration_ticks, instrument_id) in raw_notes {
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

            let art_id = match find_articulation_id(&cache.articulations) {
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

        next_tick_offset = accumulate_tick_deadline(next_tick_offset, bpm);
        let next_tick_deadline = start + next_tick_offset;

        // Sleep until the next tick deadline — accounts for time spent rendering
        let now = std::time::Instant::now();
        if next_tick_deadline > now {
            std::thread::sleep(next_tick_deadline - now);
        }
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
    let (shared, stale_handle) = {
        let mut ctx = state.lock().unwrap();

        if ctx.shared.playing.load(Ordering::Acquire) {
            return Ok(());
        }

        (ctx.shared.clone(), ctx.thread_handle.take())
    };

    join_transport_thread(stale_handle)?;

    // Build a snapshot of everything the thread needs — snapshot locks are brief and one-time
    let (snapshot, mixer_handle, initial_tempo) = {
        let app_ctx = app_state.lock().unwrap();
        let flow_id = active_flow_id(&app_ctx);

        let flow_id = match flow_id {
            Some(id) => id,
            None => {
                shared.set_tempo(f64::from(DEFAULT_TEMPO_BPM));
                return Ok(());
            }
        };

        let flow = app_ctx.score.flows.by_id.get(&flow_id).unwrap_or_else(|| {
            panic!("Missing flow {} while starting transport playback", flow_id)
        });
        let tempo_resolver = flow_tempo_resolver(&app_ctx.score, flow_id);
        let current_tick = shared.current_tick.load(Ordering::Relaxed);
        let initial_tempo = tempo_resolver.resolve(current_tick).bpm_f64();

        if !playback_can_start(current_tick, flow.length) {
            shared.set_tempo(initial_tempo);
            return Ok(());
        }

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
            flow_length: flow.length,
            sample_rate,
            tempo_resolver,
        };

        (snapshot, mixer_handle, initial_tempo)
    };

    let mut ctx = state.lock().unwrap();

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
    // Reset stop signal
    ctx.shared.stop_signal.store(false, Ordering::Release);
    ctx.shared.playing.store(true, Ordering::Release);
    ctx.shared.set_tempo(initial_tempo);

    let shared = ctx.shared.clone();

    ctx.thread_handle = Some(std::thread::spawn(move || {
        transport_thread(shared, snapshot, mixer_handle);
    }));

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_pause(
    state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
) -> Result<(), String> {
    let (shared, handle, was_playing) = {
        let mut ctx = state.lock().unwrap();
        let was_playing = ctx.shared.playing.load(Ordering::Acquire);

        if was_playing {
            ctx.shared.stop_signal.store(true, Ordering::Release);
            ctx.shared.playing.store(false, Ordering::Release);
        }

        (ctx.shared.clone(), ctx.thread_handle.take(), was_playing)
    };

    if was_playing {
        join_transport_thread(handle)?;
    }

    let current_tick = shared.current_tick.load(Ordering::Relaxed);
    let app_ctx = app_state.lock().unwrap();
    publish_app_tempo(&shared, &app_ctx, current_tick);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_stop(
    state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
) -> Result<(), String> {
    let (shared, handle) = {
        let mut ctx = state.lock().unwrap();

        ctx.shared.stop_signal.store(true, Ordering::Release);
        ctx.shared.playing.store(false, Ordering::Release);

        (ctx.shared.clone(), ctx.thread_handle.take())
    };

    join_transport_thread(handle)?;

    let app_ctx = app_state.lock().unwrap();
    publish_app_tempo(&shared, &app_ctx, 0);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_seek(
    state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
    tick: u64,
) -> Result<(), String> {
    let (shared, playing) = {
        let ctx = state.lock().unwrap();
        (
            ctx.shared.clone(),
            ctx.shared.playing.load(Ordering::Acquire),
        )
    };

    shared.current_tick.store(tick, Ordering::Relaxed);

    if !playing {
        let app_ctx = app_state.lock().unwrap();
        publish_app_tempo(&shared, &app_ctx, tick);
    }

    Ok(())
}

/// Lightweight poll command — reads atomics directly, no mutex lock needed.
/// Frontend calls this via requestAnimationFrame during playback.
#[tauri::command(rename_all = "snake_case")]
pub fn transport_get_state(
    state: State<'_, TransportSharedState>,
    app_state: State<'_, AppContextState>,
) -> TransportState {
    let playing = state.playing.load(Ordering::Relaxed);
    let current_tick = state.current_tick.load(Ordering::Relaxed);
    let tempo = if playing {
        state.get_tempo()
    } else {
        let app_ctx = app_state.lock().unwrap();
        let tempo = resolve_app_tempo_bpm(&app_ctx, current_tick);
        state.set_tempo(tempo);
        tempo
    };

    TransportState {
        playing,
        current_tick,
        tempo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::duration::NoteDuration;
    use crate::score::entries::{tempo::Tempo, Entry};
    use crate::score::flows::Flow;
    use crate::score::tracks::Track;

    fn make_transport_shared() -> TransportShared {
        TransportShared {
            playing: AtomicBool::new(false),
            current_tick: AtomicU64::new(0),
            stop_signal: AtomicBool::new(false),
            tempo_bits: AtomicU64::new(f64::from(DEFAULT_TEMPO_BPM).to_bits()),
            mix_params: RwLock::new(FxHashMap::default()),
        }
    }

    fn make_tempo(id: UniqueId, tick: Tick, bpm: u16) -> Tempo {
        Tempo::new(
            id,
            tick,
            String::new(),
            NoteDuration::Quarter,
            0,
            bpm,
            false,
            false,
            true,
            false,
        )
    }

    fn make_preferences() -> crate::preferences::Preferences {
        serde_json::from_str::<crate::preferences::Preferences>(r#"{"audition":true}"#).unwrap()
    }

    fn make_app_context(score: Score, flow_id: Option<UniqueId>) -> AppContext {
        let mut ui = crate::ui::Ui::new();
        ui.flow_id = flow_id;

        AppContext {
            ui,
            score,
            preferences: make_preferences(),
            next_id: 1,
            current_file_path: None,
        }
    }

    fn add_flow_with_tempos(
        score: &mut Score,
        flow_id: UniqueId,
        master_id: UniqueId,
        length: Tick,
        tempos: &[(Tick, u16)],
    ) {
        let mut flow = Flow::new(flow_id, master_id);
        let mut master_track = Track::new(master_id);
        flow.length = length;

        for (index, (tick, bpm)) in tempos.iter().enumerate() {
            master_track.insert_entry(Entry::Tempo(make_tempo(
                100 + flow_id * 10 + index as UniqueId,
                *tick,
                *bpm,
            )));
            flow.length = flow.length.max(*tick + 1);
        }

        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, flow);
        score.tracks.by_id.insert(master_id, master_track);
    }

    fn make_score_with_tempos(tempos: &[(Tick, u16)]) -> (Score, UniqueId) {
        let flow_id = 1;
        let master_id = 2;
        let mut score = Score::new();
        let mut flow = Flow::new(flow_id, master_id);
        let mut master_track = Track::new(master_id);
        flow.length = 0;

        for (index, (tick, bpm)) in tempos.iter().enumerate() {
            master_track.insert_entry(Entry::Tempo(make_tempo(
                100 + index as UniqueId,
                *tick,
                *bpm,
            )));
            flow.length = flow.length.max(*tick + 1);
        }

        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, flow);
        score.tracks.by_id.insert(master_id, master_track);

        (score, flow_id)
    }

    #[test]
    fn resolve_score_tempo_bpm_uses_fallback_when_flow_has_no_tempos() {
        let (score, flow_id) = make_score_with_tempos(&[]);

        assert_eq!(resolve_score_tempo_bpm(&score, Some(flow_id), 0), 120.0);
        assert_eq!(resolve_score_tempo_bpm(&score, Some(flow_id), 64), 120.0);
    }

    #[test]
    fn resolve_score_tempo_bpm_uses_tick_zero_tempo_before_fallback() {
        let (score, flow_id) = make_score_with_tempos(&[(0, 96)]);

        assert_eq!(resolve_score_tempo_bpm(&score, Some(flow_id), 0), 96.0);
    }

    #[test]
    fn resolve_score_tempo_bpm_inherits_latest_tempo_for_mid_flow_start() {
        let (score, flow_id) = make_score_with_tempos(&[(0, 96), (64, 144)]);

        assert_eq!(resolve_score_tempo_bpm(&score, Some(flow_id), 63), 96.0);
        assert_eq!(resolve_score_tempo_bpm(&score, Some(flow_id), 64), 144.0);
        assert_eq!(resolve_score_tempo_bpm(&score, Some(flow_id), 80), 144.0);
    }

    #[test]
    fn publish_resolved_tempo_updates_shared_state_for_stopped_seek() {
        let shared = make_transport_shared();
        let (score, flow_id) = make_score_with_tempos(&[(0, 90), (96, 132)]);
        let resolver = flow_tempo_resolver(&score, flow_id);

        let bpm = publish_resolved_tempo(&shared, &resolver, 96);

        assert_eq!(bpm, 132.0);
        assert_eq!(shared.current_tick.load(Ordering::Relaxed), 96);
        assert_eq!(shared.get_tempo(), 132.0);
    }

    #[test]
    fn publish_resolved_tempo_resets_to_tick_zero_tempo() {
        let shared = make_transport_shared();
        let (score, flow_id) = make_score_with_tempos(&[(0, 72), (96, 132)]);
        let resolver = flow_tempo_resolver(&score, flow_id);

        publish_resolved_tempo(&shared, &resolver, 128);
        let bpm = publish_resolved_tempo(&shared, &resolver, 0);

        assert_eq!(bpm, 72.0);
        assert_eq!(shared.current_tick.load(Ordering::Relaxed), 0);
        assert_eq!(shared.get_tempo(), 72.0);
    }

    #[test]
    fn publish_resolved_tempo_re_resolves_when_resume_uses_fresh_snapshot() {
        let shared = make_transport_shared();
        let (old_score, old_flow_id) = make_score_with_tempos(&[(0, 96)]);
        let (new_score, new_flow_id) = make_score_with_tempos(&[(0, 96), (64, 144)]);
        let old_resolver = flow_tempo_resolver(&old_score, old_flow_id);
        let new_resolver = flow_tempo_resolver(&new_score, new_flow_id);

        publish_resolved_tempo(&shared, &old_resolver, 80);
        let resumed_bpm = publish_resolved_tempo(&shared, &new_resolver, 80);

        assert_eq!(resumed_bpm, 144.0);
        assert_eq!(shared.get_tempo(), 144.0);
    }

    #[test]
    fn accumulate_tick_deadline_adds_variable_bpm_durations_without_rebasing() {
        let mut deadline = std::time::Duration::ZERO;

        deadline = accumulate_tick_deadline(deadline, 120.0);
        deadline = accumulate_tick_deadline(deadline, 60.0);
        deadline = accumulate_tick_deadline(deadline, 120.0);

        let expected = std::time::Duration::from_secs_f64(
            seconds_per_tick(120.0) + seconds_per_tick(60.0) + seconds_per_tick(120.0),
        );

        assert_eq!(deadline, expected);
    }

    #[test]
    #[should_panic(expected = "Tempo BPM 12 out of range at tick 64")]
    fn resolve_score_tempo_bpm_fails_hard_for_malformed_tempo_data() {
        let (score, flow_id) = make_score_with_tempos(&[(64, 12)]);

        let _ = resolve_score_tempo_bpm(&score, Some(flow_id), 64);
    }

    #[test]
    fn playback_can_start_rejects_positions_at_or_past_flow_end() {
        assert!(playback_can_start(31, 32));
        assert!(!playback_can_start(32, 32));
        assert!(!playback_can_start(64, 32));
    }

    #[test]
    fn clamp_tick_to_score_flow_clamps_tick_to_last_playable_position() {
        let (score, flow_id) = make_score_with_tempos(&[(0, 90), (96, 132)]);
        assert_eq!(clamp_tick_to_score_flow(&score, Some(flow_id), 9_999), 96);
    }

    #[test]
    fn sync_transport_to_app_state_uses_the_current_active_flow_when_multiple_flows_exist() {
        let shared = Arc::new(make_transport_shared());
        let mut score = Score::new();

        add_flow_with_tempos(&mut score, 1, 11, 32, &[(0, 90)]);
        add_flow_with_tempos(&mut score, 2, 22, 128, &[(0, 144)]);

        let app_ctx = make_app_context(score, Some(2));
        let bpm = sync_transport_to_app_state(&shared, &app_ctx, 96);

        assert_eq!(bpm, 144.0);
        assert_eq!(shared.current_tick.load(Ordering::Relaxed), 96);
        assert_eq!(shared.get_tempo(), 144.0);
    }

    #[test]
    fn sync_transport_to_app_state_falls_back_to_the_first_flow_when_none_is_active() {
        let shared = Arc::new(make_transport_shared());
        let mut score = Score::new();

        add_flow_with_tempos(&mut score, 1, 11, 32, &[(0, 90)]);
        add_flow_with_tempos(&mut score, 2, 22, 128, &[(0, 144)]);

        let app_ctx = make_app_context(score, None);
        let bpm = sync_transport_to_app_state(&shared, &app_ctx, 96);

        assert_eq!(bpm, 90.0);
        assert_eq!(shared.current_tick.load(Ordering::Relaxed), 31);
        assert_eq!(shared.get_tempo(), 90.0);
    }
}
