use crate::playback::engine::{
    clear_transport_playback, queue_transport_note, set_slot_mix, set_transport_render_targets,
    transport_playback_has_pending_work,
};
use crate::playback::expression_maps::{
    resolve_expression_map_playback, ExpressionMapAction, ExpressionMapLibraryState,
};
use crate::playback::preparation::{
    effective_playback_mix_for_instrument, prepare_supported_note_events_for_flow,
    prepare_supported_playback_with_expression_maps, PlaybackPreparation,
};
use crate::playback::tuplet_projection::{PlaybackTick, PlaybackToneEvent};
use crate::score::components::duration::Tick;
use crate::score::entries::tempo::{EffectiveTempoResolver, DEFAULT_TEMPO_BPM};
use crate::score::instruments::Instrument;
use crate::score::playback::ResolvedPlaybackRoute;
use crate::score::Score;
use crate::utils::UniqueId;
use crate::{AppContext, AppContextState, AudioContextState};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use tauri::State;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferredPlaybackRefreshKind {
    ScoreStructure,
    PlaybackTopology,
}

/// Live instrument mix parameters shared between the UI thread and the transport thread.
/// Updated in real-time by volume/pan/mute/solo commands.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone)]
pub struct InstrumentMixParams {
    pub volume: u8,
    pub pan: u8,
    pub mute: bool,
    pub solo: bool,
}

/// Ticks per quarter note
const TICKS_PER_QUARTER: u64 = 32;
const TRANSPORT_LOOKAHEAD: std::time::Duration = std::time::Duration::from_millis(5);
const TRANSPORT_WAIT_QUANTUM: std::time::Duration = std::time::Duration::from_millis(1);
const EXPRESSION_MAP_KEYSWITCH_TICKS: Tick = 1;

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

/// Playback is always scoped to exactly one flow: the explicit active flow when present,
/// otherwise the first ordered flow as a narrow fallback. Transport must never merge flows.
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

fn transport_wait_duration_until_queue_window(
    remaining_until_tick_deadline: std::time::Duration,
) -> Option<std::time::Duration> {
    if remaining_until_tick_deadline <= TRANSPORT_LOOKAHEAD {
        None
    } else {
        Some(
            remaining_until_tick_deadline
                .saturating_sub(TRANSPORT_LOOKAHEAD)
                .min(TRANSPORT_WAIT_QUANTUM),
        )
    }
}

fn duration_to_frames(duration: std::time::Duration, sample_rate: u32) -> u64 {
    (duration.as_secs_f64() * sample_rate as f64).round() as u64
}

fn wait_for_transport_work_to_drain(shared: &TransportSharedState) -> bool {
    while transport_playback_has_pending_work() {
        if shared.stop_signal.load(Ordering::Acquire) {
            return false;
        }

        std::thread::sleep(TRANSPORT_WAIT_QUANTUM);
    }

    true
}

fn playback_can_start(current_tick: Tick, flow_length: Tick) -> bool {
    current_tick < flow_length
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn effective_mix_params_for_instrument(
    score: &Score,
    playback_runtime: &crate::playback::runtime_activation::PlaybackActivationRuntime,
    instrument_id: UniqueId,
) -> Option<InstrumentMixParams> {
    let resolved_playback_route_by_instrument_id =
        playback_runtime.resolved_playback_route_by_instrument_id(score);
    let instrument = score.instruments.by_id.get(&instrument_id)?;

    Some(effective_mix_params_from_route(
        score,
        instrument,
        &resolved_playback_route_by_instrument_id,
    ))
}

fn effective_mix_params_from_route(
    score: &Score,
    instrument: &Instrument,
    resolved_playback_route_by_instrument_id: &rustc_hash::FxHashMap<
        UniqueId,
        ResolvedPlaybackRoute,
    >,
) -> InstrumentMixParams {
    let effective_mix = effective_playback_mix_for_instrument(
        score,
        resolved_playback_route_by_instrument_id,
        instrument.id,
    );
    let supported_mix_controls = resolved_playback_route_by_instrument_id
        .get(&instrument.id)
        .map(|route| route.supported_mix_controls)
        .unwrap_or_default();

    InstrumentMixParams {
        volume: effective_mix.volume,
        pan: effective_mix.pan,
        mute: effective_mix.muted,
        solo: supported_mix_controls.solo && instrument.solo,
    }
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
    pub last_queued_tick: AtomicU64,
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
            last_queued_tick: AtomicU64::new(0),
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

fn paused_tick_after_transport_stop(shared: &TransportSharedState) -> Tick {
    let current_tick = shared.current_tick.load(Ordering::Relaxed);
    if transport_playback_has_pending_work() {
        shared
            .last_queued_tick
            .load(Ordering::Relaxed)
            .min(current_tick)
    } else {
        current_tick
    }
}

pub(crate) fn stop_transport_playback(
    state: &Mutex<TransportContext>,
    audio_state: &AudioContextState,
) -> Result<TransportSharedState, String> {
    let (shared, handle) = {
        let mut ctx = state.lock().unwrap();

        ctx.shared.stop_signal.store(true, Ordering::Release);
        ctx.shared.playing.store(false, Ordering::Release);

        (ctx.shared.clone(), ctx.thread_handle.take())
    };

    join_transport_thread(handle)?;
    let paused_tick = paused_tick_after_transport_stop(&shared);
    reset_transport_runtime_state(&shared, paused_tick, || flush_active_audio(audio_state));

    Ok(shared)
}

fn reset_transport_runtime_state<F>(shared: &TransportSharedState, reset_tick: Tick, flush_audio: F)
where
    F: FnOnce(),
{
    shared.current_tick.store(reset_tick, Ordering::Relaxed);
    shared.last_queued_tick.store(reset_tick, Ordering::Relaxed);
    clear_transport_playback();
    flush_audio();
    shared.stop_signal.store(false, Ordering::Release);
}

fn flush_active_audio(audio_state: &AudioContextState) {
    let audio = audio_state.lock().unwrap();
    audio.get_mixer_handle().stop_all();
}

pub(crate) fn sync_transport_to_app_state(
    shared: &TransportSharedState,
    app_ctx: &AppContext,
    tick: Tick,
) -> f64 {
    let tick = clamp_tick_to_score_flow(&app_ctx.score, active_flow_id(app_ctx), tick);
    publish_app_tempo(shared, app_ctx, tick)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScheduledNote {
    start: PlaybackTick,
    duration: PlaybackTick,
    slot_id: UniqueId,
    plugin_id: String,
    pitch: u8,
    velocity: u8,
    instrument_id: UniqueId,
    keyswitches: Vec<ScheduledKeyswitch>,
    duration_ratio_percent: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScheduledKeyswitch {
    pitch: u8,
    velocity: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TransportQueuedNote {
    sample_offset: usize,
    duration_frames: u64,
    pitch: u8,
    velocity: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TransportTickEvents {
    slot_id: UniqueId,
    events: Vec<TransportQueuedNote>,
}

/// Keep transport ordering aligned to projected host musical time. When notes share an exact
/// onset, fall back to stable metadata so simultaneous events stay deterministic without asking
/// transport to reinterpret tuplet semantics locally.
fn compare_scheduled_notes(left: &ScheduledNote, right: &ScheduledNote) -> CmpOrdering {
    left.start
        .cmp(&right.start)
        .then(left.pitch.cmp(&right.pitch))
        .then(left.instrument_id.cmp(&right.instrument_id))
        .then(left.velocity.cmp(&right.velocity))
        .then(left.duration.cmp(&right.duration))
        .then(left.plugin_id.cmp(&right.plugin_id))
}

fn playback_tick_to_f64(tick: &PlaybackTick) -> f64 {
    tick.numerator as f64 / tick.denominator as f64
}

fn playback_tick_fractional_within_tick(tick: &PlaybackTick) -> f64 {
    (tick.numerator % tick.denominator) as f64 / tick.denominator as f64
}

fn playback_tick_duration_frames(tick: &PlaybackTick, sample_rate: u32, bpm: f64) -> u64 {
    ((playback_tick_to_f64(tick) * seconds_per_tick(bpm) * sample_rate as f64).round() as u64)
        .max(1)
}

fn playback_tick_sample_offset_frames(tick: &PlaybackTick, sample_rate: u32, bpm: f64) -> usize {
    (playback_tick_fractional_within_tick(tick) * seconds_per_tick(bpm) * sample_rate as f64)
        .round() as usize
}

fn transport_event_sample_offset_frames(
    tick: &PlaybackTick,
    sample_rate: u32,
    bpm: f64,
    lead_frames: u64,
) -> usize {
    lead_frames
        .saturating_add(playback_tick_sample_offset_frames(tick, sample_rate, bpm) as u64)
        .min(usize::MAX as u64) as usize
}

fn scale_duration_frames(duration_frames: u64, ratio_percent: u16) -> u64 {
    let scaled = ((duration_frames as u128 * ratio_percent as u128) + 50) / 100;
    (scaled as u64).max(1)
}

fn scheduled_note_transport_events(
    note: &ScheduledNote,
    sample_rate: u32,
    bpm: f64,
    lead_frames: u64,
) -> Vec<TransportQueuedNote> {
    let sample_offset =
        transport_event_sample_offset_frames(&note.start, sample_rate, bpm, lead_frames);
    let mut events = Vec::with_capacity(note.keyswitches.len() + 1);

    for keyswitch in &note.keyswitches {
        events.push(TransportQueuedNote {
            sample_offset,
            duration_frames: playback_tick_duration_frames(
                &PlaybackTick::from_tick(EXPRESSION_MAP_KEYSWITCH_TICKS),
                sample_rate,
                bpm,
            ),
            pitch: keyswitch.pitch,
            velocity: keyswitch.velocity,
        });
    }

    let duration_frames = playback_tick_duration_frames(&note.duration, sample_rate, bpm);
    events.push(TransportQueuedNote {
        sample_offset,
        duration_frames: scale_duration_frames(duration_frames, note.duration_ratio_percent),
        pitch: note.pitch,
        velocity: note.velocity,
    });

    events
}

fn transport_events_for_tick(
    snapshot: &PlaybackSnapshot,
    current_tick: Tick,
    sample_rate: u32,
    bpm: f64,
    lead_frames: u64,
) -> Vec<TransportTickEvents> {
    let Some(scheduled_notes) = snapshot.scheduled_notes.get(&current_tick) else {
        return Vec::new();
    };

    let mut events_by_slot_id: FxHashMap<UniqueId, Vec<TransportQueuedNote>> = FxHashMap::default();

    for note in scheduled_notes {
        events_by_slot_id
            .entry(note.slot_id)
            .or_default()
            .extend(scheduled_note_transport_events(
                note,
                sample_rate,
                bpm,
                lead_frames,
            ));
    }

    let mut slot_ids = events_by_slot_id.keys().copied().collect::<Vec<_>>();
    slot_ids.sort_unstable();

    slot_ids
        .into_iter()
        .map(|slot_id| TransportTickEvents {
            slot_id,
            events: events_by_slot_id.remove(&slot_id).unwrap_or_default(),
        })
        .collect()
}

/// Collect all notes in the active flow, projected into exact host musical time and bucketed by
/// the integer host tick that owns each onset.
fn collect_scheduled_notes(
    score: &Score,
    flow_id: UniqueId,
    preparation: &PlaybackPreparation,
) -> FxHashMap<Tick, Vec<ScheduledNote>> {
    let note_events_by_instrument_id =
        prepare_supported_note_events_for_flow(score, flow_id, preparation);

    collect_scheduled_notes_from_prepared_events(preparation, &note_events_by_instrument_id)
}

fn collect_scheduled_notes_from_prepared_events(
    preparation: &PlaybackPreparation,
    note_events_by_instrument_id: &FxHashMap<UniqueId, Vec<PlaybackToneEvent>>,
) -> FxHashMap<Tick, Vec<ScheduledNote>> {
    let mut notes_by_tick: FxHashMap<Tick, Vec<ScheduledNote>> = FxHashMap::default();
    let supported_targets_by_instrument_id = preparation
        .supported_targets
        .iter()
        .map(|target| (target.instrument_id, target))
        .collect::<FxHashMap<_, _>>();
    if supported_targets_by_instrument_id.is_empty() {
        return notes_by_tick;
    }

    for (instrument_id, tones) in note_events_by_instrument_id {
        let Some(prepared_target) = supported_targets_by_instrument_id.get(instrument_id) else {
            continue;
        };

        let plugin_id = prepared_target.plugin_id.clone();

        for tone in tones {
            let mut keyswitches = Vec::new();
            let mut duration_ratio_percent = 100;

            if let Some(expression_map) = prepared_target.expression_map.as_ref() {
                let resolved =
                    resolve_expression_map_playback(expression_map, &tone.expression_map_context);

                for action in resolved.actions {
                    match action {
                        ExpressionMapAction::KeyswitchNote {
                            midi_note,
                            velocity,
                            ..
                        } => {
                            keyswitches.push(ScheduledKeyswitch {
                                pitch: midi_note,
                                velocity,
                            });
                        }
                        ExpressionMapAction::NoteLengthTransform { ratio_percent, .. } => {
                            duration_ratio_percent = ratio_percent;
                        }
                        ExpressionMapAction::ControlChange { .. }
                        | ExpressionMapAction::NoOp { .. } => {}
                    }
                }
            }

            notes_by_tick
                .entry(tone.start.floor())
                .or_default()
                .push(ScheduledNote {
                    start: tone.start.clone(),
                    duration: tone.duration.clone(),
                    slot_id: prepared_target.slot_id,
                    plugin_id: plugin_id.clone(),
                    pitch: tone.pitch,
                    velocity: tone.velocity,
                    instrument_id: *instrument_id,
                    keyswitches,
                    duration_ratio_percent,
                });
        }
    }

    for notes in notes_by_tick.values_mut() {
        notes.sort_by(compare_scheduled_notes);
    }

    notes_by_tick
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlaybackExecutionMode {
    TransportOnly,
}

/// A snapshot of all data the transport thread needs for supported playback.
/// Cloned once at play-start so the thread never locks AppContextState.
struct PlaybackSnapshot {
    score: Score,
    #[cfg_attr(not(test), allow(dead_code))]
    preparation: PlaybackPreparation,
    scheduled_notes: FxHashMap<Tick, Vec<ScheduledNote>>,
    flow_length: u64,
    tempo_resolver: EffectiveTempoResolver,
    execution_mode: PlaybackExecutionMode,
}

fn build_supported_playback_snapshot_with_expression_maps(
    app_ctx: &AppContext,
    current_tick: Tick,
    expression_map_library: &ExpressionMapLibraryState,
    sample_rate: u32,
) -> Option<(PlaybackSnapshot, f64)> {
    let flow_id = active_flow_id(app_ctx)?;
    let flow =
        app_ctx.score.flows.by_id.get(&flow_id).unwrap_or_else(|| {
            panic!("Missing flow {} while starting transport playback", flow_id)
        });
    let tempo_resolver = flow_tempo_resolver(&app_ctx.score, flow_id);
    let initial_tempo = tempo_resolver.resolve(current_tick).bpm_f64();

    if !playback_can_start(current_tick, flow.length) {
        return None;
    }

    let preparation = prepare_supported_playback_with_expression_maps(
        &app_ctx.score,
        &app_ctx.playback_activation_runtime,
        expression_map_library,
        sample_rate,
    );

    Some((
        PlaybackSnapshot {
            score: app_ctx.score.clone(),
            preparation: preparation.clone(),
            scheduled_notes: collect_scheduled_notes(&app_ctx.score, flow_id, &preparation),
            flow_length: flow.length,
            tempo_resolver,
            execution_mode: PlaybackExecutionMode::TransportOnly,
        },
        initial_tempo,
    ))
}

#[cfg_attr(not(test), allow(dead_code))]
fn build_supported_playback_snapshot(
    app_ctx: &AppContext,
    current_tick: Tick,
    sample_rate: u32,
) -> Option<(PlaybackSnapshot, f64)> {
    build_supported_playback_snapshot_with_expression_maps(
        app_ctx,
        current_tick,
        &ExpressionMapLibraryState::default(),
        sample_rate,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TickScheduleState {
    next_tick_offset: std::time::Duration,
    expected_tick: Option<Tick>,
}

impl TickScheduleState {
    fn new() -> Self {
        Self {
            next_tick_offset: std::time::Duration::ZERO,
            expected_tick: None,
        }
    }

    fn align_for_tick(&mut self, current_tick: Tick) -> bool {
        let rebased = self.expected_tick != Some(current_tick);

        if rebased {
            self.next_tick_offset = std::time::Duration::ZERO;
        }

        rebased
    }

    fn clear_expected_tick(&mut self) {
        self.expected_tick = None;
    }

    fn advance(&mut self, current_tick: Tick, bpm: f64) {
        self.expected_tick = Some(current_tick + 1);
        self.next_tick_offset = accumulate_tick_deadline(self.next_tick_offset, bpm);
    }
}

fn transport_thread(shared: Arc<TransportShared>, snapshot: PlaybackSnapshot) {
    let mut tick_origin = std::time::Instant::now();
    let mut schedule_state = TickScheduleState::new();

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

        if schedule_state.align_for_tick(current_tick) {
            tick_origin = std::time::Instant::now();
        }

        let next_tick_deadline = tick_origin + schedule_state.next_tick_offset;
        let lead_duration = loop {
            if shared.stop_signal.load(Ordering::Acquire) {
                break None;
            }

            let now = std::time::Instant::now();
            let remaining_until_tick_deadline = next_tick_deadline.saturating_duration_since(now);

            match transport_wait_duration_until_queue_window(remaining_until_tick_deadline) {
                Some(wait_duration) => std::thread::sleep(wait_duration),
                None => break Some(remaining_until_tick_deadline),
            }
        };

        let Some(lead_duration) = lead_duration else {
            break;
        };

        let bpm = publish_resolved_tempo(&shared, &snapshot.tempo_resolver, current_tick);

        if matches!(
            snapshot.execution_mode,
            PlaybackExecutionMode::TransportOnly
        ) {
            let tick_events = transport_events_for_tick(
                &snapshot,
                current_tick,
                snapshot.preparation.sample_rate,
                bpm,
                duration_to_frames(lead_duration, snapshot.preparation.sample_rate),
            );

            if !tick_events.is_empty() {
                shared
                    .last_queued_tick
                    .store(current_tick, Ordering::Relaxed);
            }

            for tick_events in tick_events {
                for event in tick_events.events {
                    queue_transport_note(
                        tick_events.slot_id,
                        event.sample_offset,
                        event.duration_frames,
                        event.pitch,
                        event.velocity,
                    );
                }
            }
        }

        if shared.stop_signal.load(Ordering::Acquire) {
            break;
        }

        if shared.current_tick.load(Ordering::Relaxed) != current_tick {
            schedule_state.clear_expected_tick();
            continue;
        }

        // Advance tick
        shared
            .current_tick
            .store(current_tick + 1, Ordering::Relaxed);
        schedule_state.advance(current_tick, bpm);
    }

    if !shared.stop_signal.load(Ordering::Acquire) {
        if wait_for_transport_work_to_drain(&shared) {
            clear_transport_playback();
        }

        if !shared.stop_signal.load(Ordering::Acquire) {
            shared.playing.store(false, Ordering::Release);
        }
    }
}

fn apply_transport_snapshot(
    ctx: &mut TransportContext,
    snapshot: PlaybackSnapshot,
    initial_tempo: f64,
) {
    {
        let mut mix = ctx.shared.mix_params.write().unwrap();
        mix.clear();
        for (id, inst) in &snapshot.score.instruments.by_id {
            mix.insert(
                *id,
                effective_mix_params_from_route(
                    &snapshot.score,
                    inst,
                    &snapshot
                        .preparation
                        .resolved_playback_route_by_instrument_id,
                ),
            );
        }
    }

    ctx.shared.stop_signal.store(false, Ordering::Release);
    ctx.shared.playing.store(true, Ordering::Release);
    ctx.shared.set_tempo(initial_tempo);
    ctx.shared.last_queued_tick.store(
        ctx.shared.current_tick.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );

    for target in &snapshot.preparation.supported_targets {
        set_slot_mix(target.slot_id, target.effective_mix);
    }

    set_transport_render_targets(
        snapshot
            .preparation
            .supported_targets
            .iter()
            .map(|target| target.slot_id),
    );

    let shared = ctx.shared.clone();
    ctx.thread_handle = Some(std::thread::spawn(move || {
        transport_thread(shared, snapshot);
    }));
}

fn start_transport_from_current_tick_with_expression_maps(
    ctx: &mut TransportContext,
    app_state: &AppContextState,
    expression_map_library: &ExpressionMapLibraryState,
    sample_rate: u32,
) -> Result<(), String> {
    let current_tick = ctx.shared.current_tick.load(Ordering::Relaxed);

    let (snapshot, initial_tempo) = {
        let app_ctx = app_state.lock().unwrap();

        match build_supported_playback_snapshot_with_expression_maps(
            &app_ctx,
            current_tick,
            expression_map_library,
            sample_rate,
        ) {
            Some((snapshot, initial_tempo)) => (snapshot, initial_tempo),
            None => {
                ctx.shared.playing.store(false, Ordering::Release);
                reset_transport_runtime_state(&ctx.shared, current_tick, || {});
                let flow_id = active_flow_id(&app_ctx);
                ctx.shared.set_tempo(resolve_score_tempo_bpm(
                    &app_ctx.score,
                    flow_id,
                    current_tick,
                ));
                return Ok(());
            }
        }
    };

    apply_transport_snapshot(ctx, snapshot, initial_tempo);

    Ok(())
}

#[allow(dead_code)]
fn start_transport_from_current_tick(
    ctx: &mut TransportContext,
    app_state: &AppContextState,
    sample_rate: u32,
) -> Result<(), String> {
    start_transport_from_current_tick_with_expression_maps(
        ctx,
        app_state,
        &ExpressionMapLibraryState::default(),
        sample_rate,
    )
}

fn transport_play_with_states_and_expression_maps(
    state: &Mutex<TransportContext>,
    app_state: &AppContextState,
    expression_map_library: &ExpressionMapLibraryState,
    sample_rate: u32,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();
    if ctx.shared.playing.load(Ordering::Acquire) {
        return Ok(());
    }

    let stale_handle = ctx.thread_handle.take();
    join_transport_thread(stale_handle)?;
    clear_transport_playback();
    start_transport_from_current_tick_with_expression_maps(
        &mut ctx,
        app_state,
        expression_map_library,
        sample_rate,
    )?;

    Ok(())
}

#[cfg_attr(not(test), allow(dead_code))]
fn transport_play_with_states(
    state: &Mutex<TransportContext>,
    app_state: &AppContextState,
    sample_rate: u32,
) -> Result<(), String> {
    transport_play_with_states_and_expression_maps(
        state,
        app_state,
        &ExpressionMapLibraryState::default(),
        sample_rate,
    )
}

fn transport_seek_with_states_and_expression_maps(
    state: &Mutex<TransportContext>,
    app_state: &AppContextState,
    expression_map_library: &ExpressionMapLibraryState,
    sample_rate: u32,
    tick: Tick,
) -> Result<(), String> {
    let mut ctx = state.lock().unwrap();
    let playing = ctx.shared.playing.load(Ordering::Acquire);
    if playing {
        ctx.shared.stop_signal.store(true, Ordering::Release);
        ctx.shared.playing.store(false, Ordering::Release);
    }

    let stale_handle = ctx.thread_handle.take();
    join_transport_thread(stale_handle)?;

    {
        let app_ctx = app_state.lock().unwrap();
        sync_transport_to_app_state(&ctx.shared, &app_ctx, tick);
    }

    reset_transport_runtime_state(
        &ctx.shared,
        ctx.shared.current_tick.load(Ordering::Relaxed),
        || {},
    );

    if !playing {
        return Ok(());
    }

    start_transport_from_current_tick_with_expression_maps(
        &mut ctx,
        app_state,
        expression_map_library,
        sample_rate,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
fn transport_seek_with_states(
    state: &Mutex<TransportContext>,
    app_state: &AppContextState,
    sample_rate: u32,
    tick: Tick,
) -> Result<(), String> {
    transport_seek_with_states_and_expression_maps(
        state,
        app_state,
        &ExpressionMapLibraryState::default(),
        sample_rate,
        tick,
    )
}

fn transport_pause_with_states<F>(
    state: &Mutex<TransportContext>,
    app_state: &AppContextState,
    flush_audio: F,
) -> Result<(), String>
where
    F: FnOnce(),
{
    let (shared, handle) = {
        let mut ctx = state.lock().unwrap();
        ctx.shared.stop_signal.store(true, Ordering::Release);
        ctx.shared.playing.store(false, Ordering::Release);

        (ctx.shared.clone(), ctx.thread_handle.take())
    };

    join_transport_thread(handle)?;
    let paused_tick = paused_tick_after_transport_stop(&shared);
    reset_transport_runtime_state(&shared, paused_tick, flush_audio);

    let app_ctx = app_state.lock().unwrap();
    publish_app_tempo(&shared, &app_ctx, paused_tick);

    Ok(())
}

fn reconcile_pending_playback_refresh<F>(
    app_state: &AppContextState,
    refresh_topology_runtime: F,
) -> Result<(), String>
where
    F: FnOnce(&mut AppContext) -> Result<(), String>,
{
    let mut app_ctx = app_state.lock().unwrap();
    if app_ctx.pending_playback_refresh_kind()
        == Some(DeferredPlaybackRefreshKind::PlaybackTopology)
    {
        refresh_topology_runtime(&mut app_ctx)?;
    }
    app_ctx.clear_pending_playback_refresh();

    Ok(())
}

// -- Tauri Commands --

#[tauri::command(rename_all = "snake_case")]
pub fn transport_play(
    app: tauri::AppHandle,
    state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
    audio_state: State<'_, AudioContextState>,
    renderer_state: State<'_, crate::RendererContextState>,
    expression_map_library_state: State<'_, crate::ManagedExpressionMapLibraryContextState>,
) -> Result<(), String> {
    let sample_rate = {
        let audio = audio_state.lock().unwrap();
        audio.get_sample_rate()
    };

    reconcile_pending_playback_refresh(app_state.inner(), |app_ctx| {
        app_ctx.refresh_playback_activation_runtime_from_app(&app);
        Ok(())
    })?;
    let expression_map_library = expression_map_library_state.lock().unwrap().snapshot();
    transport_play_with_states_and_expression_maps(
        state.inner(),
        app_state.inner(),
        &expression_map_library,
        sample_rate,
    )?;

    let mut app_ctx = app_state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    app_ctx.emit_general_state(&app, &mut renderer, None);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_pause(
    app: tauri::AppHandle,
    state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
    audio_state: State<'_, AudioContextState>,
    renderer_state: State<'_, crate::RendererContextState>,
) -> Result<(), String> {
    transport_pause_with_states(state.inner(), app_state.inner(), || {
        flush_active_audio(audio_state.inner())
    })?;

    reconcile_pending_playback_refresh(app_state.inner(), |app_ctx| {
        app_ctx.refresh_playback_activation_runtime_from_app(&app);
        Ok(())
    })?;

    let mut app_ctx = app_state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    app_ctx.emit_general_state(&app, &mut renderer, None);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn transport_seek(
    app: tauri::AppHandle,
    state: State<'_, Mutex<TransportContext>>,
    app_state: State<'_, AppContextState>,
    audio_state: State<'_, AudioContextState>,
    renderer_state: State<'_, crate::RendererContextState>,
    expression_map_library_state: State<'_, crate::ManagedExpressionMapLibraryContextState>,
    tick: u64,
) -> Result<(), String> {
    let sample_rate = {
        let audio = audio_state.lock().unwrap();
        audio.get_sample_rate()
    };

    reconcile_pending_playback_refresh(app_state.inner(), |app_ctx| {
        app_ctx.refresh_playback_activation_runtime_from_app(&app);
        Ok(())
    })?;
    let expression_map_library = expression_map_library_state.lock().unwrap().snapshot();
    transport_seek_with_states_and_expression_maps(
        state.inner(),
        app_state.inner(),
        &expression_map_library,
        sample_rate,
        tick,
    )?;

    let mut app_ctx = app_state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();
    app_ctx.emit_general_state(&app, &mut renderer, None);

    Ok(())
}

/// Lightweight poll command — reads atomics directly, no mutex lock needed.
/// Frontend calls this via requestAnimationFrame during playback.
#[tauri::command(rename_all = "snake_case")]
pub fn transport_get_state(state: State<'_, TransportSharedState>) -> TransportState {
    let playing = state.playing.load(Ordering::Relaxed);
    let current_tick = state.current_tick.load(Ordering::Relaxed);
    // Keep the transport poll path lock-free. Shared tempo is synchronized when
    // the score, active flow, or transport position changes, so polling should
    // never need to acquire AppContextState from the main thread.
    let tempo = state.get_tempo();

    TransportState {
        playing,
        current_tick,
        tempo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::audio::reset_transport_for_audio_rebuild_with_stop;
    use crate::playback::engine::{
        reset_continuous_render_engine_for_tests, transport_playback_debug_state,
    };
    use crate::playback::expression_maps::{
        ExpressionMap, ExpressionMapAction, ExpressionMapCondition,
        ExpressionMapEqualityComparator, ExpressionMapLibraryState,
        ExpressionMapNoteLengthComparator, ExpressionMapRule,
    };
    use crate::playback::plugin_inventory::{
        PluginInventoryAvailability, PluginInventoryContext, PluginInventoryEntry,
    };
    use crate::playback::runtime_activation::PlaybackActivationRuntime;
    use crate::playback::runtime_activation::SingleInstrumentTrustGateBlocker;
    use crate::score::components::articulation::{Articulation, Articulations};
    use crate::score::components::duration::NoteDuration;
    use crate::score::entries::{tempo::Tempo, Entry};
    use crate::score::flows::Flow;
    use crate::score::instruments::{defs::InstrumentType, Instrument};
    use crate::score::playback::ResolvedPlaybackRouteStatus;
    use crate::score::players::{Player, PlayerType};
    use crate::score::staves::Stave;
    use crate::score::tracks::Track;
    use rustc_hash::FxHashSet;
    use std::sync::OnceLock;

    fn transport_test_mutex() -> &'static Mutex<()> {
        static TRANSPORT_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        TRANSPORT_TEST_MUTEX.get_or_init(|| Mutex::new(()))
    }
    fn make_transport_shared() -> TransportShared {
        TransportShared {
            playing: AtomicBool::new(false),
            current_tick: AtomicU64::new(0),
            last_queued_tick: AtomicU64::new(0),
            stop_signal: AtomicBool::new(false),
            tempo_bits: AtomicU64::new(f64::from(DEFAULT_TEMPO_BPM).to_bits()),
            mix_params: RwLock::new(FxHashMap::default()),
        }
    }

    fn make_transport_context_state() -> Mutex<TransportContext> {
        TransportContext::new().0
    }

    fn stop_test_transport(state: &Mutex<TransportContext>) {
        let handle = {
            let mut ctx = state.lock().unwrap();
            ctx.shared.stop_signal.store(true, Ordering::Release);
            ctx.shared.playing.store(false, Ordering::Release);
            ctx.thread_handle.take()
        };

        join_transport_thread(handle).unwrap();

        let ctx = state.lock().unwrap();
        ctx.shared.stop_signal.store(false, Ordering::Release);
        drop(ctx);

        reset_continuous_render_engine_for_tests();
    }

    fn wait_for_transport_engine_work(slot_ids: &[UniqueId]) {
        for _ in 0..1_000 {
            if slot_ids.iter().all(|slot_id| {
                transport_playback_debug_state(*slot_id).is_some_and(|state| {
                    state.transport_active
                        || state.queued_transport_event_count > 0
                        || state.active_transport_note_count > 0
                })
            }) {
                return;
            }

            std::thread::yield_now();
        }

        panic!("Timed out waiting for transport work to reach the engine");
    }

    fn transport_engine_work_is_cleared(slot_id: UniqueId) -> bool {
        transport_playback_debug_state(slot_id).is_none_or(|state| {
            !state.transport_active
                && state.queued_transport_event_count == 0
                && state.active_transport_note_count == 0
        })
    }

    fn wait_for_transport_current_tick(shared: &TransportSharedState, expected_tick: Tick) {
        for _ in 0..1_000 {
            if shared.current_tick.load(Ordering::Relaxed) >= expected_tick {
                return;
            }

            std::thread::yield_now();
        }

        panic!("Timed out waiting for transport tick to advance");
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
            playback_activation_runtime: make_playback_runtime_for_score(&score),
            score,
            pending_playback_refresh: None,
            preferences: make_preferences(),
            next_id: 1,
            current_file_path: None,
        }
    }

    fn make_playback_runtime_for_score(score: &Score) -> PlaybackActivationRuntime {
        let inventory = PluginInventoryContext::with_entries_for_tests(
            score
                .playback
                .rack
                .by_id
                .values()
                .filter_map(|slot| slot.target.as_ref())
                .map(|target| PluginInventoryEntry {
                    id: target.plugin_id.clone(),
                    name: target.plugin_id.clone(),
                    bundle_path: format!("/tmp/{}.vst3", target.plugin_id),
                    binary_path: Some(format!(
                        "/tmp/{}.vst3/Contents/MacOS/{}",
                        target.plugin_id, target.plugin_id
                    )),
                    source_root: String::from("/tmp"),
                    availability: PluginInventoryAvailability::Available,
                    validation_failure: None,
                    editor_host_validation: None,
                })
                .collect(),
        );

        PlaybackActivationRuntime::from_score_and_inventory(score, &inventory)
    }

    fn make_playback_preparation_for_score(score: &Score, sample_rate: u32) -> PlaybackPreparation {
        let runtime = make_playback_runtime_for_score(score);

        crate::playback::preparation::prepare_supported_playback(score, &runtime, sample_rate)
    }

    fn staccato_articulations() -> Articulations {
        Articulations::new().toggle(Articulation::Staccato)
    }

    fn proof_expression_map_library_state() -> ExpressionMapLibraryState {
        ExpressionMapLibraryState {
            order: vec![String::from("staccato-proof")],
            by_id: FxHashMap::from_iter([(
                String::from("staccato-proof"),
                ExpressionMap {
                    id: String::from("staccato-proof"),
                    name: String::from("Staccato Proof"),
                    creator: String::new(),
                    version: 1,
                    default_actions: vec![ExpressionMapAction::NoOp {
                        id: String::from("default-proof-action"),
                    }],
                    rules: vec![
                        ExpressionMapRule {
                            id: String::from("short-staccato"),
                            name: String::from("Short staccato"),
                            conditions: vec![
                                ExpressionMapCondition::Articulation {
                                    id: String::from("short-staccato-articulation"),
                                    comparator: ExpressionMapEqualityComparator::Equals,
                                    articulation: Some(Articulation::Staccato),
                                },
                                ExpressionMapCondition::WrittenNoteLength {
                                    id: String::from("short-staccato-length"),
                                    comparator: ExpressionMapNoteLengthComparator::LessThanOrEqual,
                                    note_duration: NoteDuration::Eighth,
                                },
                            ],
                            actions: vec![
                                ExpressionMapAction::KeyswitchNote {
                                    id: String::from("short-staccato-keyswitch"),
                                    midi_note: 24,
                                    velocity: 96,
                                },
                                ExpressionMapAction::NoteLengthTransform {
                                    id: String::from("short-staccato-length-transform"),
                                    ratio_percent: 50,
                                },
                            ],
                        },
                        ExpressionMapRule {
                            id: String::from("long-staccato"),
                            name: String::from("Long staccato"),
                            conditions: vec![ExpressionMapCondition::Articulation {
                                id: String::from("long-staccato-articulation"),
                                comparator: ExpressionMapEqualityComparator::Equals,
                                articulation: Some(Articulation::Staccato),
                            }],
                            actions: vec![ExpressionMapAction::NoteLengthTransform {
                                id: String::from("long-staccato-length-transform"),
                                ratio_percent: 55,
                            }],
                        },
                    ],
                },
            )]),
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

    fn make_instrument(id: UniqueId, stave_id: UniqueId, _plugin_id: &str) -> Instrument {
        Instrument {
            id,
            definition_id: String::from("test.instrument"),
            instrument_type: InstrumentType::Melodic,
            long_name: String::from("Test"),
            short_name: String::from("T."),
            stave_ids: vec![stave_id],
            count: None,
            playback_route: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    fn make_instrument_with_mix(
        id: UniqueId,
        stave_id: UniqueId,
        plugin_id: &str,
        mute: bool,
        solo: bool,
    ) -> Instrument {
        let mut instrument = make_instrument(id, stave_id, plugin_id);
        instrument.mute = mute;
        instrument.solo = solo;
        instrument
    }

    fn make_flow_score_with_track(track: Track) -> (Score, UniqueId) {
        let flow_id = 1;
        let master_id = 2;
        let player_id = 3;
        let instrument_id = 4;
        let stave_id = 5;
        let track_id = track.id;
        let mut score = Score::new();
        let mut flow = Flow::new(flow_id, master_id);
        flow.length = 160;
        flow.player_ids = FxHashSet::from_iter([player_id]);
        flow.staves.by_id.insert(
            stave_id,
            Stave {
                id: stave_id,
                lines: vec![1, 1, 1, 1, 1],
                master_id,
                track_ids: vec![track_id],
            },
        );

        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, flow);
        score.tracks.by_id.insert(master_id, Track::new(master_id));
        score.tracks.by_id.insert(track_id, track);
        score.players.order.push(player_id);
        score
            .players
            .by_id
            .insert(player_id, Player::new(player_id, PlayerType::Solo));
        score
            .players
            .by_id
            .get_mut(&player_id)
            .unwrap()
            .instrument_ids = vec![instrument_id];
        score.instruments.by_id.insert(
            instrument_id,
            make_instrument(instrument_id, stave_id, "plugin-a"),
        );
        score
            .create_playback_slot(6)
            .expect("slot creation should succeed");
        score
            .assign_plugin_to_slot(6, String::from("plugin-a"))
            .expect("slot assignment should succeed");
        score
            .assign_instrument_route_to_slot(instrument_id, 6)
            .expect("route assignment should succeed");

        (score, flow_id)
    }

    fn make_flow_score_with_track_and_tempos(
        track: Track,
        tempos: &[(Tick, u16)],
    ) -> (Score, UniqueId) {
        let (mut score, flow_id) = make_flow_score_with_track(track);
        let master_id = score.flows.by_id.get(&flow_id).unwrap().master_id;
        let flow = score.flows.by_id.get_mut(&flow_id).unwrap();
        let master_track = score.tracks.by_id.get_mut(&master_id).unwrap();

        for (index, (tick, bpm)) in tempos.iter().enumerate() {
            master_track.insert_entry(Entry::Tempo(make_tempo(
                200 + index as UniqueId,
                *tick,
                *bpm,
            )));
            flow.length = flow.length.max(*tick + 1);
        }

        (score, flow_id)
    }

    fn make_flow_score_with_tracks(
        tracks: Vec<(Track, &'static str, bool, bool)>,
    ) -> (Score, UniqueId, Vec<UniqueId>) {
        let flow_id = 1;
        let master_id = 2;
        let player_id = 3;
        let mut score = Score::new();
        let mut flow = Flow::new(flow_id, master_id);
        flow.length = 160;
        flow.player_ids = FxHashSet::from_iter([player_id]);

        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, flow);
        score.tracks.by_id.insert(master_id, Track::new(master_id));
        score.players.order.push(player_id);
        score
            .players
            .by_id
            .insert(player_id, Player::new(player_id, PlayerType::Solo));

        let mut instrument_ids = Vec::with_capacity(tracks.len());

        for (index, (track, plugin_id, mute, solo)) in tracks.into_iter().enumerate() {
            let track_id = track.id;
            let stave_id = 5 + index as UniqueId;
            let instrument_id = 20 + index as UniqueId;

            score
                .flows
                .by_id
                .get_mut(&flow_id)
                .unwrap()
                .staves
                .by_id
                .insert(
                    stave_id,
                    Stave {
                        id: stave_id,
                        lines: vec![1, 1, 1, 1, 1],
                        master_id,
                        track_ids: vec![track_id],
                    },
                );

            score.tracks.by_id.insert(track_id, track);
            score.instruments.by_id.insert(
                instrument_id,
                make_instrument_with_mix(instrument_id, stave_id, plugin_id, mute, solo),
            );
            let slot_id = 200 + index as UniqueId;
            score
                .create_playback_slot(slot_id)
                .expect("slot creation should succeed");
            score
                .assign_plugin_to_slot(slot_id, String::from(plugin_id))
                .expect("slot assignment should succeed");
            score
                .assign_instrument_route_to_slot(instrument_id, slot_id)
                .expect("route assignment should succeed");
            score
                .players
                .by_id
                .get_mut(&player_id)
                .unwrap()
                .instrument_ids
                .push(instrument_id);
            instrument_ids.push(instrument_id);
        }

        (score, flow_id, instrument_ids)
    }

    fn make_flow_score_with_tracks_and_tempos(
        tracks: Vec<(Track, &'static str, bool, bool)>,
        tempos: &[(Tick, u16)],
    ) -> (Score, UniqueId, Vec<UniqueId>) {
        let (mut score, flow_id, instrument_ids) = make_flow_score_with_tracks(tracks);
        let master_id = score.flows.by_id.get(&flow_id).unwrap().master_id;
        let mut flow_length = score.flows.by_id.get(&flow_id).unwrap().length;

        {
            let master_track = score.tracks.by_id.get_mut(&master_id).unwrap();

            for (index, (tick, bpm)) in tempos.iter().enumerate() {
                master_track.insert_entry(Entry::Tempo(make_tempo(
                    300 + index as UniqueId,
                    *tick,
                    *bpm,
                )));
                flow_length = flow_length.max(*tick + 1);
            }
        }

        score.flows.by_id.get_mut(&flow_id).unwrap().length = flow_length;

        (score, flow_id, instrument_ids)
    }

    fn flatten_scheduled_starts(
        scheduled: &FxHashMap<Tick, Vec<ScheduledNote>>,
    ) -> Vec<PlaybackTick> {
        let mut ticks: Vec<_> = scheduled.keys().copied().collect();
        ticks.sort_unstable();

        ticks
            .into_iter()
            .flat_map(|tick| {
                scheduled
                    .get(&tick)
                    .into_iter()
                    .flat_map(|notes| notes.iter().map(|note| note.start.clone()))
            })
            .collect()
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
    fn transport_wait_duration_until_queue_window_caps_sleep_to_the_bounded_quantum() {
        assert_eq!(
            transport_wait_duration_until_queue_window(std::time::Duration::from_millis(20)),
            Some(TRANSPORT_WAIT_QUANTUM)
        );
        assert_eq!(
            transport_wait_duration_until_queue_window(TRANSPORT_LOOKAHEAD),
            None
        );
    }

    #[test]
    fn transport_event_sample_offset_frames_include_lookahead_lead_time() {
        assert_eq!(
            transport_event_sample_offset_frames(
                &PlaybackTick {
                    numerator: 0,
                    denominator: 1,
                },
                48_000,
                120.0,
                240,
            ),
            240
        );
    }

    #[test]
    fn tick_schedule_state_rebases_when_seek_jumps_to_a_new_tick() {
        let mut state = TickScheduleState::new();

        assert!(state.align_for_tick(96));
        state.advance(96, 120.0);

        let pre_seek_offset = state.next_tick_offset;
        assert!(pre_seek_offset > std::time::Duration::ZERO);
        assert_eq!(state.expected_tick, Some(97));

        assert!(state.align_for_tick(120));
        assert_eq!(state.next_tick_offset, std::time::Duration::ZERO);
        assert_eq!(state.expected_tick, Some(97));

        state.clear_expected_tick();
        assert!(state.align_for_tick(120));
        assert_eq!(state.next_tick_offset, std::time::Duration::ZERO);

        state.advance(120, 90.0);
        assert_eq!(state.expected_tick, Some(121));
        assert_eq!(
            state.next_tick_offset,
            std::time::Duration::from_secs_f64(seconds_per_tick(90.0))
        );
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
    fn build_supported_playback_snapshot_uses_transport_only_mode_for_supported_playback() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let app_ctx = make_app_context(score, Some(flow_id));

        let (snapshot, initial_tempo) =
            build_supported_playback_snapshot(&app_ctx, 0, 48_000).expect("expected snapshot");

        assert_eq!(initial_tempo, 120.0);
        assert_eq!(
            snapshot.execution_mode,
            PlaybackExecutionMode::TransportOnly
        );
        assert_eq!(snapshot.preparation.sample_rate, 48_000);
        assert_eq!(snapshot.scheduled_notes.get(&32).map(Vec::len), Some(1));
    }

    #[test]
    fn build_supported_playback_snapshot_returns_none_when_start_tick_is_past_flow_end() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let app_ctx = make_app_context(score, Some(flow_id));

        assert!(build_supported_playback_snapshot(&app_ctx, 160, 48_000).is_none());
    }

    #[test]
    fn build_supported_playback_snapshot_keeps_transport_running_when_routes_are_missing_from_runtime_activation(
    ) {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let mut app_ctx = make_app_context(score, Some(flow_id));
        app_ctx.playback_activation_runtime = PlaybackActivationRuntime::default();

        let (snapshot, initial_tempo) =
            build_supported_playback_snapshot(&app_ctx, 0, 48_000).expect("expected snapshot");

        assert_eq!(initial_tempo, 120.0);
        assert!(snapshot.preparation.supported_targets.is_empty());
        assert!(snapshot.scheduled_notes.is_empty());
        assert_eq!(
            snapshot.preparation.blockers,
            vec![SingleInstrumentTrustGateBlocker::RouteNotReady {
                instrument_id: 4,
                slot_id: 6,
                status: ResolvedPlaybackRouteStatus::NeedsAttention,
            }]
        );
    }

    #[test]
    fn build_supported_playback_snapshot_keeps_multiple_playable_routes_in_the_snapshot() {
        let mut first_track = Track::new(50);
        first_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        let mut second_track = Track::new(51);
        second_track
            .create_tone(81, 32, 16, 64, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id, _instrument_ids) = make_flow_score_with_tracks(vec![
            (first_track, "plugin-a", false, false),
            (second_track, "plugin-b", false, false),
        ]);
        let app_ctx = make_app_context(score, Some(flow_id));

        let (snapshot, initial_tempo) =
            build_supported_playback_snapshot(&app_ctx, 0, 48_000).expect("snapshot");
        let mut blockers = snapshot.preparation.blockers.clone();
        for blocker in &mut blockers {
            if let SingleInstrumentTrustGateBlocker::MultipleRoutedInstruments { instrument_ids } =
                blocker
            {
                instrument_ids.sort_unstable();
            }
        }

        assert_eq!(initial_tempo, 120.0);
        assert_eq!(
            snapshot
                .preparation
                .supported_targets
                .first()
                .map(|target| target.instrument_id),
            Some(20)
        );
        assert_eq!(
            snapshot
                .preparation
                .supported_targets
                .iter()
                .map(|target| target.instrument_id)
                .collect::<Vec<_>>(),
            vec![20, 21]
        );
        assert_eq!(
            blockers,
            vec![
                SingleInstrumentTrustGateBlocker::MultipleRoutedInstruments {
                    instrument_ids: vec![20, 21],
                }
            ]
        );
    }

    #[test]
    fn build_supported_playback_snapshot_allows_timing_only_when_no_route_is_assigned() {
        let mut score = Score::new();
        let flow_id = 10;
        let master_id = 11;
        let player_id = 12;
        let instrument_id = 13;
        let stave_id = 14;
        let track_id = 15;

        let mut flow = Flow::new(flow_id, master_id);
        flow.length = 160;
        flow.player_ids = FxHashSet::from_iter([player_id]);
        flow.staves.by_id.insert(
            stave_id,
            Stave {
                id: stave_id,
                lines: vec![1, 1, 1, 1, 1],
                master_id,
                track_ids: vec![track_id],
            },
        );

        let mut track = Track::new(track_id);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        score.flows.order.push(flow_id);
        score.flows.by_id.insert(flow_id, flow);
        score.tracks.by_id.insert(master_id, Track::new(master_id));
        score.tracks.by_id.insert(track_id, track);
        score.players.order.push(player_id);
        score
            .players
            .by_id
            .insert(player_id, Player::new(player_id, PlayerType::Solo));
        score
            .players
            .by_id
            .get_mut(&player_id)
            .unwrap()
            .instrument_ids = vec![instrument_id];
        score.instruments.by_id.insert(
            instrument_id,
            Instrument {
                id: instrument_id,
                definition_id: String::from("strings.violin"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Violin"),
                short_name: String::from("Vln."),
                stave_ids: vec![stave_id],
                count: None,
                playback_route: None,
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            },
        );

        let app_ctx = make_app_context(score, Some(flow_id));

        let (snapshot, initial_tempo) =
            build_supported_playback_snapshot(&app_ctx, 0, 48_000).expect("expected snapshot");

        assert_eq!(initial_tempo, 120.0);
        assert!(snapshot.scheduled_notes.is_empty());
        assert_eq!(snapshot.preparation.sample_rate, 48_000);
        assert_eq!(
            snapshot.preparation.blockers,
            vec![SingleInstrumentTrustGateBlocker::NoRoutedInstrument]
        );
    }

    #[test]
    fn transport_play_with_states_starts_transport_only_playback_from_supported_command_seam() {
        let _guard = transport_test_mutex().lock().unwrap();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));
        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert_eq!(ctx.shared.get_tempo(), 120.0);

        let mix = ctx.shared.mix_params.read().unwrap();
        assert_eq!(mix.len(), 1);
        let params = mix.values().next().unwrap();
        assert_eq!(params.volume, 100);
        assert_eq!(params.pan, 64);
        assert!(!params.mute);
        assert!(!params.solo);
        drop(mix);
        drop(ctx);

        stop_test_transport(&transport_state);
    }

    #[test]
    fn effective_mix_params_only_honor_controls_supported_by_the_ready_route() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, flow_id) = make_flow_score_with_track(track);
        let instrument = score.instruments.by_id.get_mut(&4).unwrap();
        instrument.volume = 24;
        instrument.pan = 11;
        instrument.mute = true;
        instrument.solo = true;

        let runtime = make_playback_runtime_for_score(&score);
        let params = effective_mix_params_for_instrument(&score, &runtime, 4)
            .expect("expected instrument mix params");

        assert_eq!(flow_id, 1);
        assert_eq!(params.volume, 24);
        assert_eq!(params.pan, 11);
        assert!(params.mute);
        assert!(params.solo);
    }

    #[test]
    fn transport_play_with_states_keeps_transport_stopped_when_tick_is_past_flow_end() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(160, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));
        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(!ctx.shared.playing.load(Ordering::Acquire));
        assert_eq!(ctx.shared.get_tempo(), 120.0);
        assert!(ctx.thread_handle.is_none());
    }

    #[test]
    fn transport_play_with_states_clears_stale_stop_signal_and_starts_even_without_ready_routes() {
        let _guard = transport_test_mutex().lock().unwrap();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.stop_signal.store(true, Ordering::Release);
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }

        let mut app_ctx = make_app_context(score, Some(flow_id));
        app_ctx.playback_activation_runtime = PlaybackActivationRuntime::default();
        let app_state = Mutex::new(app_ctx);

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert!(!ctx.shared.stop_signal.load(Ordering::Acquire));
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 32);
        assert_eq!(ctx.shared.get_tempo(), 120.0);
        assert!(ctx.thread_handle.is_some());
        drop(ctx);

        stop_test_transport(&transport_state);
    }

    #[test]
    fn transport_play_with_states_starts_when_one_route_is_ready_and_another_needs_attention() {
        let _guard = transport_test_mutex().lock().unwrap();

        let mut ready_track = Track::new(50);
        ready_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        let mut unresolved_track = Track::new(51);
        unresolved_track
            .create_tone(81, 32, 16, 67, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, flow_id, instrument_ids) = make_flow_score_with_tracks(vec![
            (ready_track, "plugin-a", false, false),
            (unresolved_track, "plugin-b", false, false),
        ]);
        let unresolved_instrument_id = instrument_ids[1];
        let unresolved_slot_id = score
            .instruments
            .by_id
            .get(&unresolved_instrument_id)
            .and_then(|instrument| instrument.playback_route.as_ref())
            .map(|route| route.slot_id)
            .expect("expected unresolved slot id");
        score
            .playback
            .rack
            .by_id
            .get_mut(&unresolved_slot_id)
            .unwrap()
            .target = None;

        let transport_state = make_transport_context_state();
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert_eq!(ctx.shared.get_tempo(), 120.0);
        assert!(ctx.thread_handle.is_some());
        drop(ctx);

        stop_test_transport(&transport_state);
    }

    #[test]
    fn transport_play_with_states_starts_when_multiple_routes_are_ready() {
        let _guard = transport_test_mutex().lock().unwrap();

        let mut first_track = Track::new(50);
        first_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        let mut second_track = Track::new(51);
        second_track
            .create_tone(81, 32, 16, 67, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id, _instrument_ids) = make_flow_score_with_tracks(vec![
            (first_track, "plugin-a", false, false),
            (second_track, "plugin-b", false, false),
        ]);
        let transport_state = make_transport_context_state();
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert_eq!(ctx.shared.get_tempo(), 120.0);
        assert!(ctx.thread_handle.is_some());
        drop(ctx);

        stop_test_transport(&transport_state);
    }

    #[test]
    fn scheduled_notes_for_tick_convert_to_engine_ready_transport_events() {
        let mut track = Track::new(50);
        track
            .create_tuplet(70, 96, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(80, 0, 16, 60, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(81, 16, 16, 62, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(82, 32, 16, 64, None, 96, Default::default(), Some(70))
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let app_ctx = make_app_context(score, Some(flow_id));
        let (snapshot, _) =
            build_supported_playback_snapshot(&app_ctx, 96, 48_000).expect("expected snapshot");

        let current_tick_events = transport_events_for_tick(&snapshot, 96, 48_000, 120.0, 0);
        assert_eq!(current_tick_events.len(), 1);
        let current_tick_events = &current_tick_events[0];
        assert_eq!(current_tick_events.slot_id, 6);
        assert_eq!(current_tick_events.events.len(), 1);
        assert_eq!(current_tick_events.events[0].pitch, 60);
        assert_eq!(current_tick_events.events[0].velocity, 96);
        assert_eq!(current_tick_events.events[0].sample_offset, 0);
        assert_eq!(current_tick_events.events[0].duration_frames, 8_000);

        let shared_tick_events = transport_events_for_tick(&snapshot, 106, 48_000, 120.0, 0);
        assert_eq!(shared_tick_events.len(), 1);
        let shared_tick_events = &shared_tick_events[0];
        assert_eq!(shared_tick_events.slot_id, 6);
        assert_eq!(shared_tick_events.events.len(), 1);
        assert_eq!(shared_tick_events.events[0].pitch, 62);
        assert_eq!(shared_tick_events.events[0].velocity, 96);
        assert_eq!(shared_tick_events.events[0].sample_offset, 500);
        assert_eq!(shared_tick_events.events[0].duration_frames, 8_000);
    }

    #[test]
    fn transport_events_for_tick_returns_an_empty_list_without_supported_targets() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, flow_id) = make_flow_score_with_track(track);
        score.instruments.by_id.get_mut(&4).unwrap().playback_route = None;
        let app_ctx = make_app_context(score, Some(flow_id));
        let (snapshot, _) =
            build_supported_playback_snapshot(&app_ctx, 32, 48_000).expect("expected snapshot");

        assert!(transport_events_for_tick(&snapshot, 32, 48_000, 120.0, 0).is_empty());
    }

    #[test]
    fn transport_events_for_tick_groups_multiple_playable_instruments_by_slot() {
        let mut first_track = Track::new(50);
        first_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        let mut second_track = Track::new(51);
        second_track
            .create_tone(81, 32, 16, 67, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id, _instrument_ids) = make_flow_score_with_tracks(vec![
            (first_track, "plugin-a", false, false),
            (second_track, "plugin-b", false, false),
        ]);
        let app_ctx = make_app_context(score, Some(flow_id));
        let (snapshot, _) =
            build_supported_playback_snapshot(&app_ctx, 32, 48_000).expect("expected snapshot");

        let tick_events = transport_events_for_tick(&snapshot, 32, 48_000, 120.0, 0);

        assert_eq!(tick_events.len(), 2);
        assert_eq!(tick_events[0].slot_id, 200);
        assert_eq!(tick_events[0].events.len(), 1);
        assert_eq!(tick_events[0].events[0].pitch, 60);
        assert_eq!(tick_events[1].slot_id, 201);
        assert_eq!(tick_events[1].events.len(), 1);
        assert_eq!(tick_events[1].events[0].pitch, 67);
    }

    #[test]
    fn transport_events_for_tick_emit_keyswitch_and_shortened_note_for_short_staccato_matches() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        track.apply_articulations_to_tick(32, &staccato_articulations());

        let (mut score, flow_id) = make_flow_score_with_track(track);
        score
            .instruments
            .by_id
            .get_mut(&4)
            .unwrap()
            .playback_route
            .as_mut()
            .unwrap()
            .expression_map_id = Some(String::from("staccato-proof"));

        let app_ctx = make_app_context(score, Some(flow_id));
        let expression_maps = proof_expression_map_library_state();
        let (snapshot, _) = build_supported_playback_snapshot_with_expression_maps(
            &app_ctx,
            32,
            &expression_maps,
            48_000,
        )
        .expect("expected snapshot");

        let tick_events = transport_events_for_tick(&snapshot, 32, 48_000, 120.0, 0);

        assert_eq!(tick_events.len(), 1);
        assert_eq!(tick_events[0].slot_id, 6);
        assert_eq!(tick_events[0].events.len(), 2);
        assert_eq!(tick_events[0].events[0].pitch, 24);
        assert_eq!(tick_events[0].events[0].velocity, 96);
        assert_eq!(tick_events[0].events[1].pitch, 60);
        assert_eq!(tick_events[0].events[1].duration_frames, 6_000);
    }

    #[test]
    fn transport_events_for_tick_preserve_plugin_default_path_when_no_expression_rule_matches() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 32, 60, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, flow_id) = make_flow_score_with_track(track);
        score
            .instruments
            .by_id
            .get_mut(&4)
            .unwrap()
            .playback_route
            .as_mut()
            .unwrap()
            .expression_map_id = Some(String::from("staccato-proof"));

        let app_ctx = make_app_context(score, Some(flow_id));
        let expression_maps = proof_expression_map_library_state();
        let (snapshot, _) = build_supported_playback_snapshot_with_expression_maps(
            &app_ctx,
            32,
            &expression_maps,
            48_000,
        )
        .expect("expected snapshot");

        let tick_events = transport_events_for_tick(&snapshot, 32, 48_000, 120.0, 0);

        assert_eq!(tick_events.len(), 1);
        assert_eq!(tick_events[0].events.len(), 1);
        assert_eq!(tick_events[0].events[0].pitch, 60);
        assert_eq!(tick_events[0].events[0].duration_frames, 24_000);
    }

    #[test]
    fn transport_events_for_tick_resolve_shared_expression_maps_per_instrument_without_leakage() {
        let mut short_track = Track::new(50);
        short_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        short_track.apply_articulations_to_tick(32, &staccato_articulations());

        let mut long_track = Track::new(51);
        long_track
            .create_tone(81, 32, 32, 67, None, 96, Default::default(), None)
            .unwrap();
        long_track.apply_articulations_to_tick(32, &staccato_articulations());

        let (mut score, flow_id, instrument_ids) = make_flow_score_with_tracks(vec![
            (short_track, "plugin-a", false, false),
            (long_track, "plugin-b", false, false),
        ]);
        for instrument_id in instrument_ids {
            score
                .instruments
                .by_id
                .get_mut(&instrument_id)
                .unwrap()
                .playback_route
                .as_mut()
                .unwrap()
                .expression_map_id = Some(String::from("staccato-proof"));
        }

        let app_ctx = make_app_context(score, Some(flow_id));
        let expression_maps = proof_expression_map_library_state();
        let (snapshot, _) = build_supported_playback_snapshot_with_expression_maps(
            &app_ctx,
            32,
            &expression_maps,
            48_000,
        )
        .expect("expected snapshot");

        let tick_events = transport_events_for_tick(&snapshot, 32, 48_000, 120.0, 0);

        assert_eq!(tick_events.len(), 2);
        assert_eq!(tick_events[0].slot_id, 200);
        assert_eq!(tick_events[0].events.len(), 2);
        assert_eq!(tick_events[0].events[0].pitch, 24);
        assert_eq!(tick_events[0].events[1].pitch, 60);
        assert_eq!(tick_events[0].events[1].duration_frames, 6_000);

        assert_eq!(tick_events[1].slot_id, 201);
        assert_eq!(tick_events[1].events.len(), 1);
        assert_eq!(tick_events[1].events[0].pitch, 67);
        assert_eq!(tick_events[1].events[0].duration_frames, 13_200);
    }

    #[test]
    fn transport_pause_with_states_republishes_tempo_at_the_paused_tick_and_joins_the_thread() {
        let _guard = transport_test_mutex().lock().unwrap();
        reset_continuous_render_engine_for_tests();
        let shared = Arc::new(make_transport_shared());
        shared.current_tick.store(80, Ordering::Relaxed);
        shared.playing.store(true, Ordering::Release);

        let joined = Arc::new(AtomicBool::new(false));
        let joined_flag = joined.clone();
        let thread_shared = shared.clone();
        let handle = std::thread::spawn(move || {
            while !thread_shared.stop_signal.load(Ordering::Acquire) {
                std::thread::yield_now();
            }

            joined_flag.store(true, Ordering::Release);
        });

        let transport_state = Mutex::new(TransportContext {
            shared: shared.clone(),
            thread_handle: Some(handle),
        });
        let (score, flow_id) = make_score_with_tempos(&[(0, 96), (64, 144)]);
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));
        let flush_count = Arc::new(AtomicU64::new(0));
        let flush_counter = flush_count.clone();

        transport_pause_with_states(&transport_state, &app_state, move || {
            flush_counter.fetch_add(1, Ordering::Relaxed);
        })
        .unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(!ctx.shared.playing.load(Ordering::Acquire));
        assert!(!ctx.shared.stop_signal.load(Ordering::Acquire));
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 80);
        assert_eq!(ctx.shared.get_tempo(), 144.0);
        assert!(ctx.thread_handle.is_none());
        drop(ctx);

        assert!(joined.load(Ordering::Acquire));
        assert_eq!(flush_count.load(Ordering::Relaxed), 1);

        reset_continuous_render_engine_for_tests();
    }

    #[test]
    fn transport_pause_with_states_clears_multi_target_engine_transport_work() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut first_track = Track::new(50);
        first_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        let mut second_track = Track::new(51);
        second_track
            .create_tone(81, 32, 16, 67, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id, _instrument_ids) = make_flow_score_with_tracks(vec![
            (first_track, "plugin-a", false, false),
            (second_track, "plugin-b", false, false),
        ]);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[200, 201]);

        transport_pause_with_states(&transport_state, &app_state, || {}).unwrap();

        assert!(transport_engine_work_is_cleared(200));
        assert!(transport_engine_work_is_cleared(201));

        let ctx = transport_state.lock().unwrap();
        assert!(!ctx.shared.playing.load(Ordering::Acquire));
        assert!(ctx.thread_handle.is_none());
    }

    #[test]
    fn transport_pause_with_states_rewinds_after_lookahead_queueing_clears_pending_work() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[6]);

        transport_pause_with_states(&transport_state, &app_state, || {}).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 32);
        assert_eq!(ctx.shared.last_queued_tick.load(Ordering::Relaxed), 32);
        drop(ctx);

        assert!(transport_engine_work_is_cleared(6));
    }

    #[test]
    fn transport_pause_with_states_rewinds_to_the_last_queued_tick_when_work_is_still_pending() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        clear_transport_playback();
        queue_transport_note(6, 0, 8, 60, 96);

        let shared = Arc::new(make_transport_shared());
        shared.current_tick.store(33, Ordering::Relaxed);
        shared.last_queued_tick.store(32, Ordering::Relaxed);
        shared.playing.store(true, Ordering::Release);

        let thread_shared = shared.clone();
        let handle = std::thread::spawn(move || {
            while !thread_shared.stop_signal.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
        });

        let transport_state = Mutex::new(TransportContext {
            shared: shared.clone(),
            thread_handle: Some(handle),
        });
        let (score, flow_id) = make_score_with_tempos(&[(0, 96), (64, 144)]);
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_pause_with_states(&transport_state, &app_state, || {}).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 32);
        assert_eq!(ctx.shared.last_queued_tick.load(Ordering::Relaxed), 32);
    }

    #[test]
    fn transport_play_with_states_resumes_from_the_current_tick_with_a_fresh_tempo_snapshot() {
        let _guard = transport_test_mutex().lock().unwrap();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track_and_tempos(track, &[(0, 96), (64, 144)]);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(80, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert_eq!(ctx.shared.get_tempo(), 144.0);
        drop(ctx);

        stop_test_transport(&transport_state);
    }

    #[test]
    fn transport_play_and_pause_with_states_remain_stable_across_repeated_cycles() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        for _ in 0..3 {
            transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
            {
                let ctx = transport_state.lock().unwrap();
                assert!(ctx.shared.playing.load(Ordering::Acquire));
            }

            transport_pause_with_states(&transport_state, &app_state, || {}).unwrap();

            let ctx = transport_state.lock().unwrap();
            assert!(!ctx.shared.playing.load(Ordering::Acquire));
            assert!(!ctx.shared.stop_signal.load(Ordering::Acquire));
            assert!(ctx.thread_handle.is_none());
            drop(ctx);

            assert!(transport_engine_work_is_cleared(6));
        }

        reset_continuous_render_engine_for_tests();
    }

    #[test]
    fn representative_transport_scenario_rebases_expression_map_playback_without_widening_route_truth(
    ) {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut staccato_track = Track::new(50);
        staccato_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        staccato_track.apply_articulations_to_tick(32, &staccato_articulations());

        let mut sustained_track = Track::new(51);
        sustained_track
            .create_tone(81, 96, 16, 67, None, 96, Default::default(), None)
            .unwrap();

        let mut unresolved_track = Track::new(52);
        unresolved_track
            .create_tone(82, 32, 16, 72, None, 96, Default::default(), None)
            .unwrap();
        unresolved_track
            .create_tone(83, 96, 16, 74, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, flow_id, instrument_ids) = make_flow_score_with_tracks_and_tempos(
            vec![
                (staccato_track, "plugin-a", false, false),
                (sustained_track, "plugin-b", false, false),
                (unresolved_track, "plugin-c", false, false),
            ],
            &[(0, 96), (64, 144)],
        );
        score
            .instruments
            .by_id
            .get_mut(&instrument_ids[0])
            .unwrap()
            .playback_route
            .as_mut()
            .unwrap()
            .expression_map_id = Some(String::from("staccato-proof"));
        let unresolved_slot_id = score
            .instruments
            .by_id
            .get(&instrument_ids[2])
            .and_then(|instrument| instrument.playback_route.as_ref())
            .map(|route| route.slot_id)
            .expect("expected unresolved slot id");
        score
            .playback
            .rack
            .by_id
            .get_mut(&unresolved_slot_id)
            .unwrap()
            .target = None;

        let expression_maps = proof_expression_map_library_state();
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        {
            let app_ctx = app_state.lock().unwrap();
            let (snapshot, initial_tempo) = build_supported_playback_snapshot_with_expression_maps(
                &app_ctx,
                32,
                &expression_maps,
                48_000,
            )
            .expect("expected snapshot");

            assert_eq!(initial_tempo, 96.0);
            assert_eq!(
                snapshot
                    .preparation
                    .supported_targets
                    .iter()
                    .map(|target| target.instrument_id)
                    .collect::<Vec<_>>(),
                vec![instrument_ids[0], instrument_ids[1]]
            );
            assert_eq!(
                snapshot
                    .preparation
                    .supported_target_for_instrument(instrument_ids[0])
                    .and_then(|target| target.expression_map.as_ref())
                    .map(|expression_map| expression_map.id.as_str()),
                Some("staccato-proof")
            );
            assert!(snapshot
                .preparation
                .supported_target_for_instrument(instrument_ids[2])
                .is_none());

            let opening_events = transport_events_for_tick(&snapshot, 32, 48_000, 96.0, 0);
            assert_eq!(opening_events.len(), 1);
            assert_eq!(opening_events[0].slot_id, 200);
            assert_eq!(opening_events[0].events.len(), 2);
            assert_eq!(opening_events[0].events[0].pitch, 24);
            assert_eq!(opening_events[0].events[1].pitch, 60);
        }

        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }

        transport_play_with_states_and_expression_maps(
            &transport_state,
            &app_state,
            &expression_maps,
            48_000,
        )
        .unwrap();
        wait_for_transport_engine_work(&[200]);

        assert!(transport_playback_debug_state(202).is_none());

        transport_seek_with_states_and_expression_maps(
            &transport_state,
            &app_state,
            &expression_maps,
            48_000,
            96,
        )
        .unwrap();
        wait_for_transport_engine_work(&[201]);

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 96);
        assert_eq!(ctx.shared.last_queued_tick.load(Ordering::Relaxed), 96);
        assert_eq!(ctx.shared.get_tempo(), 144.0);
        drop(ctx);

        assert!(transport_playback_debug_state(201).is_some());
        assert!(transport_playback_debug_state(202).is_none());

        transport_pause_with_states(&transport_state, &app_state, || {}).unwrap();

        assert!(transport_engine_work_is_cleared(200));
        assert!(transport_engine_work_is_cleared(201));
        assert!(transport_playback_debug_state(202).is_none());

        reset_continuous_render_engine_for_tests();
    }

    #[test]
    fn transport_seek_with_states_updates_stopped_transport_without_starting_playback() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track_and_tempos(track, &[(0, 96), (64, 144)]);
        let transport_state = make_transport_context_state();
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_seek_with_states(&transport_state, &app_state, 48_000, 80).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(!ctx.shared.playing.load(Ordering::Acquire));
        assert!(ctx.thread_handle.is_none());
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 80);
        assert_eq!(ctx.shared.last_queued_tick.load(Ordering::Relaxed), 80);
        assert_eq!(ctx.shared.get_tempo(), 144.0);
    }

    #[test]
    fn transport_seek_with_states_restarts_playback_from_a_fresh_snapshot_and_clears_old_work() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(81, 96, 16, 64, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track_and_tempos(track, &[(0, 96), (64, 144)]);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[6]);

        transport_seek_with_states(&transport_state, &app_state, 48_000, 80).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert!(ctx.thread_handle.is_some());
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 80);
        assert_eq!(ctx.shared.last_queued_tick.load(Ordering::Relaxed), 80);
        assert_eq!(ctx.shared.get_tempo(), 144.0);
        drop(ctx);

        let slot_state = transport_playback_debug_state(6).expect("expected slot state");
        assert!(slot_state.transport_active);
        assert_eq!(slot_state.queued_transport_event_count, 0);
        assert_eq!(slot_state.active_transport_note_count, 0);

        stop_test_transport(&transport_state);
    }

    #[test]
    fn transport_seek_with_states_joins_the_previous_transport_thread_before_restarting() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let shared = Arc::new(make_transport_shared());
        shared.current_tick.store(32, Ordering::Relaxed);
        shared.playing.store(true, Ordering::Release);

        let joined = Arc::new(AtomicBool::new(false));
        let joined_flag = joined.clone();
        let thread_shared = shared.clone();
        let handle = std::thread::spawn(move || {
            while !thread_shared.stop_signal.load(Ordering::Acquire) {
                std::thread::yield_now();
            }

            joined_flag.store(true, Ordering::Release);
        });

        let transport_state = Mutex::new(TransportContext {
            shared: shared.clone(),
            thread_handle: Some(handle),
        });
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_seek_with_states(&transport_state, &app_state, 48_000, 80).unwrap();

        assert!(joined.load(Ordering::Acquire));
        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert!(!ctx.shared.stop_signal.load(Ordering::Acquire));
        assert!(ctx.thread_handle.is_some());
        drop(ctx);

        stop_test_transport(&transport_state);
    }

    #[test]
    fn transport_seek_with_states_keeps_unresolved_routes_as_recoverable_no_ops() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut ready_track = Track::new(50);
        ready_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        ready_track
            .create_tone(82, 96, 16, 62, None, 96, Default::default(), None)
            .unwrap();
        let mut unresolved_track = Track::new(51);
        unresolved_track
            .create_tone(81, 32, 16, 67, None, 96, Default::default(), None)
            .unwrap();
        unresolved_track
            .create_tone(83, 96, 16, 69, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, flow_id, instrument_ids) = make_flow_score_with_tracks(vec![
            (ready_track, "plugin-a", false, false),
            (unresolved_track, "plugin-b", false, false),
        ]);
        let unresolved_instrument_id = instrument_ids[1];
        let unresolved_slot_id = score
            .instruments
            .by_id
            .get(&unresolved_instrument_id)
            .and_then(|instrument| instrument.playback_route.as_ref())
            .map(|route| route.slot_id)
            .expect("expected unresolved slot id");
        score
            .playback
            .rack
            .by_id
            .get_mut(&unresolved_slot_id)
            .unwrap()
            .target = None;

        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[200]);

        transport_seek_with_states(&transport_state, &app_state, 48_000, 96).unwrap();
        wait_for_transport_engine_work(&[200]);

        let ctx = transport_state.lock().unwrap();
        assert!(ctx.shared.playing.load(Ordering::Acquire));
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 96);
        assert_eq!(ctx.shared.last_queued_tick.load(Ordering::Relaxed), 96);
        drop(ctx);

        assert!(transport_playback_debug_state(200).is_some());
        assert!(transport_playback_debug_state(unresolved_slot_id).is_none());

        stop_test_transport(&transport_state);
    }

    #[test]
    fn transport_seek_with_states_remains_stable_across_repeated_rebases() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(81, 96, 16, 64, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track_and_tempos(track, &[(0, 96), (64, 144)]);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[6]);

        for seek_tick in [80, 32, 96, 0] {
            transport_seek_with_states(&transport_state, &app_state, 48_000, seek_tick).unwrap();

            let ctx = transport_state.lock().unwrap();
            assert!(ctx.shared.playing.load(Ordering::Acquire));
            assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), seek_tick);
            assert_eq!(
                ctx.shared.last_queued_tick.load(Ordering::Relaxed),
                seek_tick
            );
            assert!(!ctx.shared.stop_signal.load(Ordering::Acquire));
            assert!(ctx.thread_handle.is_some());
        }

        transport_pause_with_states(&transport_state, &app_state, || {}).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(!ctx.shared.playing.load(Ordering::Acquire));
        assert!(!ctx.shared.stop_signal.load(Ordering::Acquire));
        assert!(ctx.thread_handle.is_none());
        drop(ctx);

        assert!(transport_engine_work_is_cleared(6));

        reset_continuous_render_engine_for_tests();
    }

    #[test]
    fn transport_seek_with_states_updates_paused_transport_without_restarting_playback() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track_and_tempos(track, &[(0, 96), (64, 144)]);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[6]);
        transport_pause_with_states(&transport_state, &app_state, || {}).unwrap();

        transport_seek_with_states(&transport_state, &app_state, 48_000, 80).unwrap();

        let ctx = transport_state.lock().unwrap();
        assert!(!ctx.shared.playing.load(Ordering::Acquire));
        assert!(!ctx.shared.stop_signal.load(Ordering::Acquire));
        assert!(ctx.thread_handle.is_none());
        assert_eq!(ctx.shared.current_tick.load(Ordering::Relaxed), 80);
        assert_eq!(ctx.shared.last_queued_tick.load(Ordering::Relaxed), 80);
        assert_eq!(ctx.shared.get_tempo(), 144.0);
        drop(ctx);

        assert!(transport_engine_work_is_cleared(6));

        reset_continuous_render_engine_for_tests();
    }

    #[test]
    fn transport_thread_keeps_final_lookahead_work_alive_until_natural_end_drains() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 159, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(159, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[6]);

        let shared = {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.clone()
        };
        wait_for_transport_current_tick(&shared, 160);

        assert!(shared.playing.load(Ordering::Acquire));
        assert!(transport_playback_has_pending_work());

        stop_test_transport(&transport_state);
    }

    #[test]
    fn transport_rebuild_reset_requires_the_next_snapshot_to_use_the_new_sample_rate() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[6]);

        reset_transport_for_audio_rebuild_with_stop(
            &app_state,
            || {
                let (shared, handle) = {
                    let mut ctx = transport_state.lock().unwrap();
                    ctx.shared.stop_signal.store(true, Ordering::Release);
                    ctx.shared.playing.store(false, Ordering::Release);

                    (ctx.shared.clone(), ctx.thread_handle.take())
                };

                join_transport_thread(handle)?;
                Ok(shared)
            },
            |_| Ok(()),
        )
        .unwrap();

        let app_ctx = app_state.lock().unwrap();
        let current_tick = {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.load(Ordering::Relaxed)
        };
        let (snapshot, _) =
            build_supported_playback_snapshot(&app_ctx, current_tick, 96_000).expect("snapshot");
        drop(app_ctx);

        assert_eq!(snapshot.preparation.sample_rate, 96_000);
        let tick_events = transport_events_for_tick(&snapshot, 32, 96_000, 120.0, 0);
        assert_eq!(tick_events.len(), 1);
        assert_eq!(tick_events[0].events[0].duration_frames, 24_000);

        reset_continuous_render_engine_for_tests();
    }

    #[test]
    fn transport_play_with_states_keeps_current_pass_running_when_score_changes_are_deferred() {
        let _guard = transport_test_mutex().lock().unwrap();

        reset_continuous_render_engine_for_tests();

        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 32, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let transport_state = make_transport_context_state();
        {
            let ctx = transport_state.lock().unwrap();
            ctx.shared.current_tick.store(32, Ordering::Relaxed);
        }
        let app_state = Mutex::new(make_app_context(score, Some(flow_id)));

        transport_play_with_states(&transport_state, &app_state, 48_000).unwrap();
        wait_for_transport_engine_work(&[6]);

        {
            let mut app_ctx = app_state.lock().unwrap();
            app_ctx
                .score
                .tracks
                .by_id
                .get_mut(&50)
                .unwrap()
                .set_tone_pitch(80, 67, None, None);
            app_ctx.defer_playback_refresh(DeferredPlaybackRefreshKind::ScoreStructure);
        }

        {
            let ctx = transport_state.lock().unwrap();
            assert!(ctx.shared.playing.load(Ordering::Acquire));
            assert!(ctx.thread_handle.is_some());
        }
        let slot_state = transport_playback_debug_state(6).expect("expected slot state");
        assert!(
            slot_state.transport_active
                || slot_state.queued_transport_event_count > 0
                || slot_state.active_transport_note_count > 0
        );
        {
            let app_ctx = app_state.lock().unwrap();
            assert_eq!(
                app_ctx.pending_playback_refresh_kind(),
                Some(DeferredPlaybackRefreshKind::ScoreStructure)
            );
        }

        stop_test_transport(&transport_state);
    }

    #[test]
    fn playback_topology_refresh_priority_survives_later_score_structure_edits() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let mut app_ctx = make_app_context(score, Some(flow_id));

        app_ctx.defer_playback_refresh(DeferredPlaybackRefreshKind::PlaybackTopology);
        app_ctx.defer_playback_refresh(DeferredPlaybackRefreshKind::ScoreStructure);

        assert_eq!(
            app_ctx.pending_playback_refresh_kind(),
            Some(DeferredPlaybackRefreshKind::PlaybackTopology)
        );
    }

    #[test]
    fn build_supported_playback_snapshot_uses_updated_score_after_deferred_edit_boundary() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let mut app_ctx = make_app_context(score, Some(flow_id));

        let (before_snapshot, _) =
            build_supported_playback_snapshot(&app_ctx, 32, 48_000).expect("snapshot");
        let before_tick_events = transport_events_for_tick(&before_snapshot, 32, 48_000, 120.0, 0);
        assert_eq!(before_tick_events.len(), 1);
        assert_eq!(before_tick_events[0].events.len(), 1);
        assert_eq!(before_tick_events[0].events[0].pitch, 60);

        app_ctx
            .score
            .tracks
            .by_id
            .get_mut(&50)
            .unwrap()
            .set_tone_pitch(80, 67, None, None);
        app_ctx.defer_playback_refresh(DeferredPlaybackRefreshKind::ScoreStructure);

        let (after_snapshot, _) =
            build_supported_playback_snapshot(&app_ctx, 32, 48_000).expect("snapshot");
        let after_tick_events = transport_events_for_tick(&after_snapshot, 32, 48_000, 120.0, 0);
        assert_eq!(after_tick_events.len(), 1);
        assert_eq!(after_tick_events[0].events.len(), 1);
        assert_eq!(after_tick_events[0].events[0].pitch, 67);
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
    fn sync_transport_to_app_state_clamps_and_publishes_using_only_the_active_flow() {
        let shared = Arc::new(make_transport_shared());
        let mut score = Score::new();

        add_flow_with_tempos(&mut score, 1, 11, 128, &[(0, 90)]);
        add_flow_with_tempos(&mut score, 2, 22, 32, &[(0, 144)]);

        let app_ctx = make_app_context(score, Some(2));
        let bpm = sync_transport_to_app_state(&shared, &app_ctx, 96);

        assert_eq!(bpm, 144.0);
        assert_eq!(shared.current_tick.load(Ordering::Relaxed), 31);
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

    #[test]
    fn collect_scheduled_notes_only_schedules_the_selected_active_flow() {
        let mut active_track = Track::new(50);
        active_track
            .create_tuplet(70, 96, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        active_track
            .create_tone(80, 0, 16, 60, None, 96, Default::default(), Some(70))
            .unwrap();
        active_track
            .create_tone(81, 16, 16, 62, None, 96, Default::default(), Some(70))
            .unwrap();
        active_track
            .create_tone(82, 32, 16, 64, None, 96, Default::default(), Some(70))
            .unwrap();

        let (mut score, selected_flow_id) = make_flow_score_with_track(active_track);

        let sibling_flow_id = 101;
        let sibling_master_id = 102;
        let sibling_player_id = 103;
        let sibling_instrument_id = 104;
        let sibling_stave_id = 105;
        let sibling_track_id = 106;
        let mut sibling_flow = Flow::new(sibling_flow_id, sibling_master_id);
        sibling_flow.length = 160;
        sibling_flow.player_ids = FxHashSet::from_iter([sibling_player_id]);
        sibling_flow.staves.by_id.insert(
            sibling_stave_id,
            Stave {
                id: sibling_stave_id,
                lines: vec![1, 1, 1, 1, 1],
                master_id: sibling_master_id,
                track_ids: vec![sibling_track_id],
            },
        );

        let mut sibling_track = Track::new(sibling_track_id);
        sibling_track
            .create_tone(180, 32, 16, 72, None, 96, Default::default(), None)
            .unwrap();
        sibling_track
            .create_tuplet(170, 96, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        sibling_track
            .create_tone(181, 0, 16, 74, None, 96, Default::default(), Some(170))
            .unwrap();
        sibling_track
            .create_tone(182, 16, 16, 76, None, 96, Default::default(), Some(170))
            .unwrap();
        sibling_track
            .create_tone(183, 32, 16, 77, None, 96, Default::default(), Some(170))
            .unwrap();

        score.flows.order.push(sibling_flow_id);
        score.flows.by_id.insert(sibling_flow_id, sibling_flow);
        score
            .tracks
            .by_id
            .insert(sibling_master_id, Track::new(sibling_master_id));
        score.tracks.by_id.insert(sibling_track_id, sibling_track);
        score.players.order.push(sibling_player_id);
        score.players.by_id.insert(
            sibling_player_id,
            Player::new(sibling_player_id, PlayerType::Solo),
        );
        score
            .players
            .by_id
            .get_mut(&sibling_player_id)
            .unwrap()
            .instrument_ids = vec![sibling_instrument_id];
        score.instruments.by_id.insert(
            sibling_instrument_id,
            make_instrument(sibling_instrument_id, sibling_stave_id, "plugin-b"),
        );

        let app_ctx = make_app_context(score, Some(selected_flow_id));
        let preparation = make_playback_preparation_for_score(&app_ctx.score, 48_000);
        let scheduled = collect_scheduled_notes(
            &app_ctx.score,
            active_flow_id(&app_ctx).expect("expected active flow"),
            &preparation,
        );

        assert_eq!(
            flatten_scheduled_starts(&scheduled),
            vec![
                PlaybackTick::from_tick(96),
                PlaybackTick::new(320, 3),
                PlaybackTick::new(352, 3),
            ]
        );
        let mut pitches = scheduled
            .values()
            .flatten()
            .map(|note| note.pitch)
            .collect::<Vec<_>>();
        pitches.sort_unstable();
        assert_eq!(pitches, vec![60, 62, 64]);
    }

    #[test]
    fn collect_scheduled_notes_keeps_selected_host_only_flow_integer_timed_when_siblings_have_tuplets(
    ) {
        let mut active_track = Track::new(50);
        active_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        active_track
            .create_tone(81, 96, 16, 62, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, selected_flow_id) = make_flow_score_with_track(active_track);

        let sibling_flow_id = 101;
        let sibling_master_id = 102;
        let sibling_player_id = 103;
        let sibling_instrument_id = 104;
        let sibling_stave_id = 105;
        let sibling_track_id = 106;
        let mut sibling_flow = Flow::new(sibling_flow_id, sibling_master_id);
        sibling_flow.length = 160;
        sibling_flow.player_ids = FxHashSet::from_iter([sibling_player_id]);
        sibling_flow.staves.by_id.insert(
            sibling_stave_id,
            Stave {
                id: sibling_stave_id,
                lines: vec![1, 1, 1, 1, 1],
                master_id: sibling_master_id,
                track_ids: vec![sibling_track_id],
            },
        );

        let mut sibling_track = Track::new(sibling_track_id);
        sibling_track
            .create_tuplet(170, 96, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        sibling_track
            .create_tone(181, 0, 16, 74, None, 96, Default::default(), Some(170))
            .unwrap();
        sibling_track
            .create_tone(182, 16, 16, 76, None, 96, Default::default(), Some(170))
            .unwrap();
        sibling_track
            .create_tone(183, 32, 16, 77, None, 96, Default::default(), Some(170))
            .unwrap();

        score.flows.order.push(sibling_flow_id);
        score.flows.by_id.insert(sibling_flow_id, sibling_flow);
        score
            .tracks
            .by_id
            .insert(sibling_master_id, Track::new(sibling_master_id));
        score.tracks.by_id.insert(sibling_track_id, sibling_track);
        score.players.order.push(sibling_player_id);
        score.players.by_id.insert(
            sibling_player_id,
            Player::new(sibling_player_id, PlayerType::Solo),
        );
        score
            .players
            .by_id
            .get_mut(&sibling_player_id)
            .unwrap()
            .instrument_ids = vec![sibling_instrument_id];
        score.instruments.by_id.insert(
            sibling_instrument_id,
            make_instrument(sibling_instrument_id, sibling_stave_id, "plugin-b"),
        );

        let app_ctx = make_app_context(score, Some(selected_flow_id));
        let preparation = make_playback_preparation_for_score(&app_ctx.score, 48_000);
        let scheduled = collect_scheduled_notes(
            &app_ctx.score,
            active_flow_id(&app_ctx).expect("expected active flow"),
            &preparation,
        );

        assert_eq!(
            flatten_scheduled_starts(&scheduled),
            vec![PlaybackTick::from_tick(32), PlaybackTick::from_tick(96)]
        );
        assert!(scheduled
            .values()
            .flatten()
            .all(|note| note.start.denominator == 1 && note.duration.denominator == 1));
        let mut pitches = scheduled
            .values()
            .flatten()
            .map(|note| note.pitch)
            .collect::<Vec<_>>();
        pitches.sort_unstable();
        assert_eq!(pitches, vec![60, 62]);
    }

    #[test]
    fn collect_scheduled_notes_fallback_uses_only_the_first_ordered_flow() {
        let mut first_track = Track::new(50);
        first_track
            .create_tone(80, 32, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        first_track
            .create_tone(81, 96, 16, 62, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, first_flow_id) = make_flow_score_with_track(first_track);

        let second_flow_id = 101;
        let second_master_id = 102;
        let second_player_id = 103;
        let second_instrument_id = 104;
        let second_stave_id = 105;
        let second_track_id = 106;
        let mut second_flow = Flow::new(second_flow_id, second_master_id);
        second_flow.length = 160;
        second_flow.player_ids = FxHashSet::from_iter([second_player_id]);
        second_flow.staves.by_id.insert(
            second_stave_id,
            Stave {
                id: second_stave_id,
                lines: vec![1, 1, 1, 1, 1],
                master_id: second_master_id,
                track_ids: vec![second_track_id],
            },
        );

        let mut second_track = Track::new(second_track_id);
        second_track
            .create_tone(180, 48, 16, 72, None, 96, Default::default(), None)
            .unwrap();

        score.flows.order.push(second_flow_id);
        score.flows.by_id.insert(second_flow_id, second_flow);
        score
            .tracks
            .by_id
            .insert(second_master_id, Track::new(second_master_id));
        score.tracks.by_id.insert(second_track_id, second_track);
        score.players.order.push(second_player_id);
        score.players.by_id.insert(
            second_player_id,
            Player::new(second_player_id, PlayerType::Solo),
        );
        score
            .players
            .by_id
            .get_mut(&second_player_id)
            .unwrap()
            .instrument_ids = vec![second_instrument_id];
        score.instruments.by_id.insert(
            second_instrument_id,
            make_instrument(second_instrument_id, second_stave_id, "plugin-b"),
        );

        let app_ctx = make_app_context(score, None);
        let selected_flow_id = active_flow_id(&app_ctx).expect("expected fallback flow");
        let preparation = make_playback_preparation_for_score(&app_ctx.score, 48_000);
        let scheduled = collect_scheduled_notes(&app_ctx.score, selected_flow_id, &preparation);

        assert_eq!(selected_flow_id, first_flow_id);
        assert_eq!(
            flatten_scheduled_starts(&scheduled),
            vec![PlaybackTick::from_tick(32), PlaybackTick::from_tick(96)]
        );
        let mut pitches = scheduled
            .values()
            .flatten()
            .map(|note| note.pitch)
            .collect::<Vec<_>>();
        pitches.sort_unstable();
        assert_eq!(pitches, vec![60, 62]);
    }

    #[test]
    fn collect_scheduled_notes_groups_projected_tuplet_events_by_exact_host_tick() {
        let mut track = Track::new(50);
        track
            .create_tuplet(70, 96, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(80, 0, 16, 60, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(81, 16, 16, 62, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(82, 32, 16, 64, None, 96, Default::default(), Some(70))
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let scheduled = collect_scheduled_notes(&score, flow_id, &preparation);

        assert_eq!(scheduled.get(&96).map(Vec::len), Some(1));
        assert_eq!(scheduled.get(&106).map(Vec::len), Some(1));
        assert_eq!(scheduled.get(&117).map(Vec::len), Some(1));
    }

    #[test]
    fn collect_scheduled_notes_keeps_host_only_passages_in_tick_order() {
        let mut track = Track::new(50);
        track
            .create_tone(79, 80, 16, 55, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(80, 96, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(81, 111, 16, 58, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(82, 128, 16, 67, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let scheduled = collect_scheduled_notes(&score, flow_id, &preparation);

        assert_eq!(
            flatten_scheduled_starts(&scheduled),
            vec![
                PlaybackTick::from_tick(80),
                PlaybackTick::from_tick(96),
                PlaybackTick::from_tick(111),
                PlaybackTick::from_tick(128),
            ]
        );
    }

    #[test]
    fn collect_scheduled_notes_orders_host_and_tuplet_events_within_a_shared_tick_bucket() {
        let mut track = Track::new(50);
        track
            .create_tuplet(70, 96, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(80, 0, 16, 60, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(81, 16, 16, 62, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(82, 32, 16, 64, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(83, 111, 16, 58, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let scheduled = collect_scheduled_notes(&score, flow_id, &preparation);
        let bucket = scheduled
            .get(&106)
            .expect("expected mixed shared bucket at tick 106");

        assert_eq!(bucket.len(), 2);
        assert_eq!(bucket[0].start, PlaybackTick::from_tick(106));
        assert_eq!(bucket[0].pitch, 58);
        assert_eq!(bucket[1].start, PlaybackTick::new(320, 3));
        assert_eq!(bucket[1].pitch, 62);
    }

    #[test]
    fn collect_scheduled_notes_keeps_before_between_and_after_mixed_passages_in_host_order() {
        let mut track = Track::new(50);
        track
            .create_tone(79, 80, 16, 55, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tuplet(70, 96, 3, 2, NoteDuration::Eighth, vec![3])
            .unwrap();
        track
            .create_tone(80, 0, 16, 60, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(81, 16, 16, 62, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(82, 32, 16, 64, None, 96, Default::default(), Some(70))
            .unwrap();
        track
            .create_tone(83, 111, 16, 58, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(84, 128, 16, 67, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let scheduled = collect_scheduled_notes(&score, flow_id, &preparation);

        assert_eq!(
            flatten_scheduled_starts(&scheduled),
            vec![
                PlaybackTick::from_tick(80),
                PlaybackTick::from_tick(96),
                PlaybackTick::from_tick(106),
                PlaybackTick::new(320, 3),
                PlaybackTick::new(352, 3),
                PlaybackTick::from_tick(128),
            ]
        );
    }

    #[test]
    fn collect_scheduled_notes_keeps_host_only_passages_integer_timed() {
        let mut track = Track::new(50);
        track
            .create_tone(79, 32, 16, 55, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(80, 96, 24, 60, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(81, 128, 32, 64, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let scheduled = collect_scheduled_notes(&score, flow_id, &preparation);

        assert!(scheduled
            .values()
            .flatten()
            .all(|note| { note.start.denominator == 1 && note.duration.denominator == 1 }));
    }

    #[test]
    fn collect_scheduled_notes_keeps_same_start_host_only_notes_in_deterministic_order() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 96, 16, 60, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(81, 96, 16, 64, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track(track);
        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let scheduled = collect_scheduled_notes(&score, flow_id, &preparation);
        let bucket = scheduled
            .get(&96)
            .expect("expected host-only shared bucket");

        assert_eq!(bucket.len(), 2);
        assert!(bucket
            .iter()
            .all(|note| note.start == PlaybackTick::from_tick(96)));
        assert_eq!(
            bucket.iter().map(|note| note.pitch).collect::<Vec<_>>(),
            vec![60, 64]
        );
    }

    #[test]
    fn collect_scheduled_notes_preserves_host_only_tempo_resolution_at_note_onsets() {
        let mut track = Track::new(50);
        track
            .create_tone(79, 32, 16, 55, None, 96, Default::default(), None)
            .unwrap();
        track
            .create_tone(80, 96, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (score, flow_id) = make_flow_score_with_track_and_tempos(track, &[(0, 84), (64, 132)]);
        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let scheduled = collect_scheduled_notes(&score, flow_id, &preparation);
        let starts = flatten_scheduled_starts(&scheduled);

        assert_eq!(
            starts,
            vec![PlaybackTick::from_tick(32), PlaybackTick::from_tick(96)]
        );
        assert_eq!(
            starts
                .iter()
                .map(|start| resolve_score_tempo_bpm(&score, Some(flow_id), start.floor()))
                .collect::<Vec<_>>(),
            vec![84.0, 132.0]
        );
    }

    #[test]
    fn collect_scheduled_notes_from_prepared_events_no_longer_reads_master_tracks() {
        let mut track = Track::new(50);
        track
            .create_tone(80, 96, 16, 60, None, 96, Default::default(), None)
            .unwrap();

        let (mut score, flow_id) = make_flow_score_with_track(track);
        score
            .tracks
            .by_id
            .get_mut(&2)
            .unwrap()
            .insert_entry(Entry::PlayingTechnique(
                crate::score::entries::playing_technique::PlayingTechnique::new(
                    200,
                    96,
                    crate::score::entries::playing_technique::PlayingTechniqueKind::Pizz,
                ),
            ));

        let preparation = make_playback_preparation_for_score(&score, 48_000);
        let prepared_note_events =
            crate::playback::preparation::prepare_supported_note_events_for_flow(
                &score,
                flow_id,
                &preparation,
            );

        score.tracks.by_id.remove(&2);

        let scheduled =
            collect_scheduled_notes_from_prepared_events(&preparation, &prepared_note_events);

        assert_eq!(
            flatten_scheduled_starts(&scheduled),
            vec![PlaybackTick::from_tick(96)]
        );
        assert_eq!(
            scheduled
                .get(&96)
                .expect("expected scheduled note bucket")
                .iter()
                .map(|note| note.pitch)
                .collect::<Vec<_>>(),
            vec![60]
        );
    }
}
