use crate::playback::audition::AuditionNote;
use crate::playback::plugin_editor_poc::{
    render_slot_runtime_block, stop_slot_runtime_render, SlotRuntimeRenderOutcome,
};
use crate::playback::preparation::EffectivePlaybackMix;
use crate::utils::UniqueId;
use rustc_hash::{FxHashMap, FxHashSet};
use std::f32;
use std::sync::{Mutex, OnceLock};

const AUDITION_RELEASE_TAIL_MS: u32 = 1_000;

fn audition_release_tail_frames(sample_rate: u32) -> u64 {
    (u64::from(sample_rate) * u64::from(AUDITION_RELEASE_TAIL_MS) / 1_000).max(1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveRenderEventKind {
    NoteOn,
    NoteOff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveRenderEvent {
    pub sample_offset: usize,
    pub kind: LiveRenderEventKind,
    pub pitch: u8,
    pub velocity: u8,
    pub note_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderActiveReason {
    Editor,
    Audition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderEventOwner {
    Audition,
    Transport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct QueuedRenderEvent {
    owner: RenderEventOwner,
    frames_until_dispatch: u64,
    kind: LiveRenderEventKind,
    pitch: u8,
    velocity: u8,
    note_id: i32,
}

#[derive(Debug, Default)]
struct SlotRenderState {
    editor_active: bool,
    audition_active: bool,
    transport_active: bool,
    tail_frames_remaining: u64,
    next_note_id: i32,
    active_transport_note_ids: FxHashSet<i32>,
    queued_events: Vec<QueuedRenderEvent>,
}

impl SlotRenderState {
    fn allocate_note_id(&mut self) -> i32 {
        let note_id = self.next_note_id;
        self.next_note_id = if self.next_note_id == i32::MAX {
            0
        } else {
            self.next_note_id + 1
        };
        note_id
    }

    fn set_reason(&mut self, reason: RenderActiveReason, active: bool) {
        match reason {
            RenderActiveReason::Editor => self.editor_active = active,
            RenderActiveReason::Audition => self.audition_active = active,
        }
    }

    fn enqueue_event(&mut self, event: QueuedRenderEvent) {
        self.queued_events.push(event);
        self.queued_events
            .sort_by_key(|queued| queued.frames_until_dispatch);
    }

    fn requeue_due_events(&mut self, events: Vec<QueuedRenderEvent>) {
        for event in events {
            self.queued_events.push(event);
        }

        self.queued_events
            .sort_by_key(|queued| queued.frames_until_dispatch);
    }

    fn has_queued_transport_events(&self) -> bool {
        self.queued_events
            .iter()
            .any(|event| event.owner == RenderEventOwner::Transport)
    }

    fn clear_transport_state(&mut self) -> bool {
        let had_transport_state = self.transport_active
            || !self.active_transport_note_ids.is_empty()
            || self.has_queued_transport_events();

        self.transport_active = false;
        self.active_transport_note_ids.clear();
        self.queued_events
            .retain(|event| event.owner != RenderEventOwner::Transport);

        had_transport_state
    }

    fn is_render_active(&self) -> bool {
        self.editor_active
            || self.transport_active
            || self.audition_active
            || self.tail_frames_remaining > 0
            || !self.queued_events.is_empty()
    }

    fn collect_due_events(&mut self, frames_per_block: usize) -> Vec<QueuedRenderEvent> {
        let frames_per_block = frames_per_block as u64;
        let mut due_events = Vec::new();
        let mut remaining_events = Vec::with_capacity(self.queued_events.len());

        for mut queued in self.queued_events.drain(..) {
            if queued.frames_until_dispatch < frames_per_block {
                due_events.push(queued);
            } else {
                queued.frames_until_dispatch -= frames_per_block;
                remaining_events.push(queued);
            }
        }

        self.queued_events = remaining_events;
        due_events.sort_by_key(|event| event.frames_until_dispatch);
        due_events
    }

    fn apply_rendered_transport_events(&mut self, events: &[QueuedRenderEvent]) {
        for event in events {
            if event.owner != RenderEventOwner::Transport {
                continue;
            }

            match event.kind {
                LiveRenderEventKind::NoteOn => {
                    self.active_transport_note_ids.insert(event.note_id);
                }
                LiveRenderEventKind::NoteOff => {
                    self.active_transport_note_ids.remove(&event.note_id);
                }
            }
        }
    }

    fn finish_block(&mut self, sample_rate: u32, frames_per_block: usize) {
        if self.audition_active && self.queued_events.is_empty() {
            self.audition_active = false;
            if !self.editor_active && !self.transport_active {
                self.tail_frames_remaining = self
                    .tail_frames_remaining
                    .max(audition_release_tail_frames(sample_rate));
            }
        }

        if !self.editor_active
            && !self.transport_active
            && !self.audition_active
            && self.tail_frames_remaining > 0
        {
            self.tail_frames_remaining = self
                .tail_frames_remaining
                .saturating_sub(frames_per_block as u64);
        }
    }
}

#[derive(Debug, Default)]
struct ContinuousRenderEngineState {
    slot_states: FxHashMap<UniqueId, SlotRenderState>,
    slot_mix_by_slot_id: FxHashMap<UniqueId, EffectivePlaybackMix>,
    mix_scratch_buffer: Vec<f32>,
}

impl ContinuousRenderEngineState {
    fn pan_balance_gains(mix: EffectivePlaybackMix) -> (f32, f32) {
        let gain = mix.gain_scalar();
        let pan = if mix.pan >= 64 {
            (f32::from(mix.pan) - 64.0) / 63.0
        } else {
            (f32::from(mix.pan) - 64.0) / 64.0
        };

        (gain * (1.0 - pan.max(0.0)), gain * (1.0 + pan.min(0.0)))
    }

    fn sync_live_slots(&mut self, live_slot_ids: &FxHashSet<UniqueId>) -> Vec<UniqueId> {
        let mut removed_slot_ids = Vec::new();
        self.slot_states.retain(|slot_id, _| {
            let keep = live_slot_ids.contains(slot_id);
            if !keep {
                removed_slot_ids.push(*slot_id);
            }
            keep
        });

        for slot_id in live_slot_ids {
            self.slot_states.entry(*slot_id).or_default();
        }

        self.slot_mix_by_slot_id
            .retain(|slot_id, _| live_slot_ids.contains(slot_id));

        removed_slot_ids
    }

    fn sync_slot_mix_states(
        &mut self,
        slot_mix_by_slot_id: FxHashMap<UniqueId, EffectivePlaybackMix>,
    ) {
        self.slot_mix_by_slot_id = slot_mix_by_slot_id;
    }

    fn set_slot_mix(&mut self, slot_id: UniqueId, mix: EffectivePlaybackMix) {
        self.slot_mix_by_slot_id.insert(slot_id, mix);
        self.slot_states.entry(slot_id).or_default();
    }

    fn set_slot_reason(
        &mut self,
        slot_id: UniqueId,
        reason: RenderActiveReason,
        active: bool,
    ) -> bool {
        let state = self.slot_states.entry(slot_id).or_default();
        state.set_reason(reason, active);
        !state.is_render_active()
    }

    fn set_transport_targets(&mut self, slot_ids: &FxHashSet<UniqueId>) -> Vec<UniqueId> {
        let mut inactive_slots = Vec::new();
        for (existing_slot_id, state) in &mut self.slot_states {
            let was_active = state.is_render_active();
            state.transport_active = slot_ids.contains(existing_slot_id);
            if was_active && !state.is_render_active() {
                inactive_slots.push(*existing_slot_id);
            }
        }

        for slot_id in slot_ids {
            self.slot_states
                .entry(*slot_id)
                .or_default()
                .transport_active = true;
        }

        inactive_slots
    }

    fn queue_audition_note(&mut self, slot_id: UniqueId, sample_rate: u32, note: AuditionNote) {
        let state = self.slot_states.entry(slot_id).or_default();
        state.set_reason(RenderActiveReason::Audition, true);
        state.tail_frames_remaining = audition_release_tail_frames(sample_rate);
        let note_id = state.allocate_note_id();
        state.enqueue_event(QueuedRenderEvent {
            owner: RenderEventOwner::Audition,
            frames_until_dispatch: 0,
            kind: LiveRenderEventKind::NoteOn,
            pitch: note.pitch,
            velocity: note.velocity,
            note_id,
        });

        let duration_frames = (u64::from(sample_rate) * u64::from(note.duration_ms) / 1_000).max(1);
        state.enqueue_event(QueuedRenderEvent {
            owner: RenderEventOwner::Audition,
            frames_until_dispatch: duration_frames,
            kind: LiveRenderEventKind::NoteOff,
            pitch: note.pitch,
            velocity: 0,
            note_id,
        });
    }

    fn queue_transport_note(
        &mut self,
        slot_id: UniqueId,
        sample_offset: usize,
        duration_frames: u64,
        pitch: u8,
        velocity: u8,
    ) {
        let state = self.slot_states.entry(slot_id).or_default();
        state.transport_active = true;
        let note_id = state.allocate_note_id();
        state.enqueue_event(QueuedRenderEvent {
            owner: RenderEventOwner::Transport,
            frames_until_dispatch: sample_offset as u64,
            kind: LiveRenderEventKind::NoteOn,
            pitch,
            velocity,
            note_id,
        });
        state.enqueue_event(QueuedRenderEvent {
            owner: RenderEventOwner::Transport,
            frames_until_dispatch: sample_offset as u64 + duration_frames.max(1),
            kind: LiveRenderEventKind::NoteOff,
            pitch,
            velocity: 0,
            note_id,
        });
    }

    fn clear_transport_playback(&mut self) -> Vec<UniqueId> {
        let mut slots_to_stop = Vec::new();

        for (slot_id, state) in &mut self.slot_states {
            let should_stop = state.clear_transport_state();
            if should_stop {
                slots_to_stop.push(*slot_id);
            }
        }

        slots_to_stop
    }

    fn has_pending_transport_work(&self) -> bool {
        self.slot_states.values().any(|state| {
            state.has_queued_transport_events() || !state.active_transport_note_ids.is_empty()
        })
    }

    fn process_block<B: RenderBackend>(
        &mut self,
        backend: &B,
        output: &mut [f32],
        output_channels: usize,
        sample_rate: u32,
    ) {
        let frames_per_block = output.len() / output_channels.max(1);
        let mut slots_to_stop = Vec::new();
        let slot_ids = self.slot_states.keys().copied().collect::<Vec<_>>();

        for slot_id in slot_ids {
            let (mix, due_events) = {
                let Some(slot_state) = self.slot_states.get_mut(&slot_id) else {
                    continue;
                };

                if !slot_state.is_render_active() {
                    continue;
                }

                (
                    self.slot_mix_by_slot_id
                        .get(&slot_id)
                        .copied()
                        .unwrap_or_default(),
                    slot_state.collect_due_events(frames_per_block),
                )
            };

            let events = due_events
                .iter()
                .map(|event| LiveRenderEvent {
                    sample_offset: event.frames_until_dispatch as usize,
                    kind: event.kind,
                    pitch: event.pitch,
                    velocity: event.velocity,
                    note_id: event.note_id,
                })
                .collect::<Vec<_>>();

            self.mix_scratch_buffer.clear();
            self.mix_scratch_buffer.resize(output.len(), 0.0);
            let outcome = backend.render_slot(
                slot_id,
                sample_rate,
                frames_per_block,
                &events,
                self.mix_scratch_buffer.as_mut_slice(),
                output_channels,
            );

            match outcome {
                SlotRuntimeRenderOutcome::Rendered | SlotRuntimeRenderOutcome::SilenceOnly => {
                    let Some(slot_state) = self.slot_states.get_mut(&slot_id) else {
                        continue;
                    };
                    slot_state.apply_rendered_transport_events(&due_events);
                }
                SlotRuntimeRenderOutcome::Busy | SlotRuntimeRenderOutcome::MissingSession => {
                    let Some(slot_state) = self.slot_states.get_mut(&slot_id) else {
                        continue;
                    };
                    slot_state.requeue_due_events(due_events);
                    continue;
                }
                SlotRuntimeRenderOutcome::Unsupported => {
                    slots_to_stop.push(slot_id);
                }
                SlotRuntimeRenderOutcome::Error(error) => {
                    eprintln!(
                        "continuous_render_engine: slot {} render block failed: {}",
                        slot_id, error
                    );
                    slots_to_stop.push(slot_id);
                }
            }

            if !mix.muted {
                let gain = mix.gain_scalar();
                if output_channels <= 1 {
                    for (destination, sample) in
                        output.iter_mut().zip(self.mix_scratch_buffer.iter())
                    {
                        *destination += *sample * gain;
                    }
                } else {
                    let (left_gain, right_gain) = Self::pan_balance_gains(mix);
                    for (destination_frame, source_frame) in output
                        .chunks_exact_mut(output_channels)
                        .zip(self.mix_scratch_buffer.chunks_exact(output_channels))
                    {
                        destination_frame[0] += source_frame[0] * left_gain;
                        destination_frame[1] += source_frame[1] * right_gain;

                        for channel_index in 2..output_channels {
                            destination_frame[channel_index] += source_frame[channel_index] * gain;
                        }
                    }
                }
            }

            let Some(slot_state) = self.slot_states.get_mut(&slot_id) else {
                continue;
            };
            slot_state.finish_block(sample_rate, frames_per_block);
            if !slot_state.is_render_active() {
                slots_to_stop.push(slot_id);
            }
        }

        slots_to_stop.sort_unstable();
        slots_to_stop.dedup();
        for slot_id in slots_to_stop {
            backend.stop_slot(slot_id);
        }
    }
}

trait RenderBackend {
    fn render_slot(
        &self,
        slot_id: UniqueId,
        sample_rate: u32,
        frames_per_block: usize,
        events: &[LiveRenderEvent],
        output: &mut [f32],
        output_channels: usize,
    ) -> SlotRuntimeRenderOutcome;

    fn stop_slot(&self, slot_id: UniqueId);
}

struct SlotRuntimeRenderBackend;

impl RenderBackend for SlotRuntimeRenderBackend {
    fn render_slot(
        &self,
        slot_id: UniqueId,
        sample_rate: u32,
        frames_per_block: usize,
        events: &[LiveRenderEvent],
        output: &mut [f32],
        output_channels: usize,
    ) -> SlotRuntimeRenderOutcome {
        render_slot_runtime_block(
            slot_id,
            sample_rate,
            frames_per_block,
            events,
            output,
            output_channels,
        )
    }

    fn stop_slot(&self, slot_id: UniqueId) {
        stop_slot_runtime_render(slot_id);
    }
}

fn continuous_render_engine() -> &'static Mutex<ContinuousRenderEngineState> {
    static CONTINUOUS_RENDER_ENGINE: OnceLock<Mutex<ContinuousRenderEngineState>> = OnceLock::new();
    CONTINUOUS_RENDER_ENGINE.get_or_init(|| Mutex::new(ContinuousRenderEngineState::default()))
}

fn stop_slots(slot_ids: Vec<UniqueId>) {
    for slot_id in slot_ids {
        stop_slot_runtime_render(slot_id);
    }
}

pub(crate) fn sync_live_slot_targets(slot_ids: impl IntoIterator<Item = UniqueId>) {
    let live_slot_ids = FxHashSet::from_iter(slot_ids);
    let removed_slot_ids = continuous_render_engine()
        .lock()
        .unwrap()
        .sync_live_slots(&live_slot_ids);
    stop_slots(removed_slot_ids);
}

pub(crate) fn sync_slot_mix_states(slot_mix_by_slot_id: FxHashMap<UniqueId, EffectivePlaybackMix>) {
    continuous_render_engine()
        .lock()
        .unwrap()
        .sync_slot_mix_states(slot_mix_by_slot_id);
}

pub(crate) fn set_slot_mix(slot_id: UniqueId, mix: EffectivePlaybackMix) {
    continuous_render_engine()
        .lock()
        .unwrap()
        .set_slot_mix(slot_id, mix);
}

pub(crate) fn set_slot_editor_render_active(slot_id: UniqueId, active: bool) {
    let should_stop = continuous_render_engine().lock().unwrap().set_slot_reason(
        slot_id,
        RenderActiveReason::Editor,
        active,
    );
    if should_stop {
        stop_slot_runtime_render(slot_id);
    }
}

pub(crate) fn set_transport_render_targets(slot_ids: impl IntoIterator<Item = UniqueId>) {
    let active_slot_ids = FxHashSet::from_iter(slot_ids);
    let slots_to_stop = continuous_render_engine()
        .lock()
        .unwrap()
        .set_transport_targets(&active_slot_ids);
    stop_slots(slots_to_stop);
}

pub(crate) fn clear_transport_playback() {
    let slots_to_stop = continuous_render_engine()
        .lock()
        .unwrap()
        .clear_transport_playback();
    stop_slots(slots_to_stop);
}

pub(crate) fn transport_playback_has_pending_work() -> bool {
    continuous_render_engine()
        .lock()
        .unwrap()
        .has_pending_transport_work()
}

pub(crate) fn queue_audition_note(slot_id: UniqueId, sample_rate: u32, note: AuditionNote) {
    continuous_render_engine()
        .lock()
        .unwrap()
        .queue_audition_note(slot_id, sample_rate, note);
}

pub(crate) fn queue_transport_note(
    slot_id: UniqueId,
    sample_offset: usize,
    duration_frames: u64,
    pitch: u8,
    velocity: u8,
) {
    continuous_render_engine()
        .lock()
        .unwrap()
        .queue_transport_note(slot_id, sample_offset, duration_frames, pitch, velocity);
}

pub(crate) fn process_audio_block(output: &mut [f32], output_channels: usize, sample_rate: u32) {
    let Ok(mut engine) = continuous_render_engine().try_lock() else {
        return;
    };

    engine.process_block(
        &SlotRuntimeRenderBackend,
        output,
        output_channels,
        sample_rate,
    );
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TransportPlaybackDebugState {
    pub transport_active: bool,
    pub queued_transport_event_count: usize,
    pub active_transport_note_count: usize,
}

#[cfg(test)]
pub(crate) fn transport_playback_debug_state(
    slot_id: UniqueId,
) -> Option<TransportPlaybackDebugState> {
    let engine = continuous_render_engine().lock().unwrap();
    let slot_state = engine.slot_states.get(&slot_id)?;

    Some(TransportPlaybackDebugState {
        transport_active: slot_state.transport_active,
        queued_transport_event_count: slot_state
            .queued_events
            .iter()
            .filter(|event| event.owner == RenderEventOwner::Transport)
            .count(),
        active_transport_note_count: slot_state.active_transport_note_ids.len(),
    })
}

#[cfg(test)]
pub(crate) fn reset_continuous_render_engine_for_tests() {
    let mut engine = continuous_render_engine().lock().unwrap();
    let slot_ids = engine.slot_states.keys().copied().collect::<Vec<_>>();
    *engine = ContinuousRenderEngineState::default();
    drop(engine);
    stop_slots(slot_ids);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RenderCall {
        slot_id: UniqueId,
        sample_rate: u32,
        frames_per_block: usize,
        events: Vec<LiveRenderEvent>,
    }

    #[derive(Default)]
    struct FakeRenderBackend {
        render_calls: RefCell<Vec<RenderCall>>,
        stop_calls: RefCell<Vec<UniqueId>>,
        output_fill: f32,
    }

    impl RenderBackend for FakeRenderBackend {
        fn render_slot(
            &self,
            slot_id: UniqueId,
            sample_rate: u32,
            frames_per_block: usize,
            events: &[LiveRenderEvent],
            output: &mut [f32],
            _output_channels: usize,
        ) -> SlotRuntimeRenderOutcome {
            output.fill(self.output_fill);
            self.render_calls.borrow_mut().push(RenderCall {
                slot_id,
                sample_rate,
                frames_per_block,
                events: events.to_vec(),
            });
            SlotRuntimeRenderOutcome::SilenceOnly
        }

        fn stop_slot(&self, slot_id: UniqueId) {
            self.stop_calls.borrow_mut().push(slot_id);
        }
    }

    fn make_audition_note(duration_ms: u32) -> AuditionNote {
        AuditionNote {
            pitch: 64,
            duration_ms,
            velocity: 96,
            articulation: crate::playback::audition::AuditionArticulation::Legato,
        }
    }

    #[test]
    fn queued_audition_runs_through_note_off_then_tail_release() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([7]));
        engine.queue_audition_note(7, 1_000, make_audition_note(4));

        for _ in 0..300 {
            engine.process_block(&backend, &mut output, 2, 1_000);
        }

        let calls = backend.render_calls.borrow();
        assert!(!calls.is_empty());
        assert_eq!(calls[0].events[0].kind, LiveRenderEventKind::NoteOn);
        assert_eq!(calls[0].events[0].note_id, 0);
        assert!(calls.iter().any(|call| {
            call.events
                .iter()
                .any(|event| event.kind == LiveRenderEventKind::NoteOff && event.note_id == 0)
        }));
        assert_eq!(backend.stop_calls.borrow().as_slice(), &[7]);
    }

    #[test]
    fn queueing_new_audition_preserves_existing_tail_and_appends_events() {
        let mut engine = ContinuousRenderEngineState::default();

        engine.sync_live_slots(&FxHashSet::from_iter([11]));
        engine.queue_audition_note(11, 1_000, make_audition_note(4));

        let slot_state = engine.slot_states.get_mut(&11).unwrap();
        slot_state.audition_active = false;
        slot_state.tail_frames_remaining = audition_release_tail_frames(1_000);
        slot_state.enqueue_event(QueuedRenderEvent {
            owner: RenderEventOwner::Audition,
            frames_until_dispatch: 100,
            kind: LiveRenderEventKind::NoteOff,
            pitch: 64,
            velocity: 0,
            note_id: 0,
        });

        engine.queue_audition_note(11, 1_000, make_audition_note(4));

        let slot_state = engine.slot_states.get(&11).unwrap();
        assert!(slot_state.audition_active);
        assert_eq!(
            slot_state.tail_frames_remaining,
            audition_release_tail_frames(1_000)
        );
        assert_eq!(slot_state.queued_events.len(), 5);
        let note_on_count = slot_state
            .queued_events
            .iter()
            .filter(|event| event.kind == LiveRenderEventKind::NoteOn)
            .count();
        let note_off_count = slot_state
            .queued_events
            .iter()
            .filter(|event| event.kind == LiveRenderEventKind::NoteOff)
            .count();
        assert_eq!(note_on_count, 2);
        assert_eq!(note_off_count, 3);
        assert!(slot_state
            .queued_events
            .iter()
            .any(|event| event.kind == LiveRenderEventKind::NoteOn && event.note_id == 0));
        assert!(slot_state
            .queued_events
            .iter()
            .any(|event| event.kind == LiveRenderEventKind::NoteOn && event.note_id == 1));
        assert_eq!(
            slot_state
                .queued_events
                .iter()
                .filter(|event| event.kind == LiveRenderEventKind::NoteOff && event.note_id == 0)
                .count(),
            2
        );
        assert_eq!(
            slot_state
                .queued_events
                .iter()
                .filter(|event| event.kind == LiveRenderEventKind::NoteOff && event.note_id == 1)
                .count(),
            1
        );
    }

    #[test]
    fn editor_activity_keeps_slot_rendering_without_transport() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([9]));
        assert!(!engine.set_slot_reason(9, RenderActiveReason::Editor, true));
        engine.process_block(&backend, &mut output, 2, 48_000);

        let calls = backend.render_calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].slot_id, 9);
        assert!(calls[0].events.is_empty());
    }

    #[test]
    fn transport_targets_switch_without_reassigning_stream_ownership() {
        let mut engine = ContinuousRenderEngineState::default();

        engine.sync_live_slots(&FxHashSet::from_iter([1, 2]));
        assert!(engine
            .set_transport_targets(&FxHashSet::from_iter([1]))
            .is_empty());

        let first = engine.slot_states.get(&1).unwrap();
        let second = engine.slot_states.get(&2).unwrap();
        assert!(first.transport_active);
        assert!(!second.transport_active);

        let inactive = engine.set_transport_targets(&FxHashSet::from_iter([2]));
        assert_eq!(inactive, vec![1]);
        assert!(!engine.slot_states.get(&1).unwrap().transport_active);
        assert!(engine.slot_states.get(&2).unwrap().transport_active);
    }

    #[test]
    fn transport_targets_keep_multiple_slots_render_active() {
        let mut engine = ContinuousRenderEngineState::default();

        engine.sync_live_slots(&FxHashSet::from_iter([1, 2, 3]));
        assert!(engine
            .set_transport_targets(&FxHashSet::from_iter([1, 3]))
            .is_empty());

        assert!(engine.slot_states.get(&1).unwrap().transport_active);
        assert!(!engine.slot_states.get(&2).unwrap().transport_active);
        assert!(engine.slot_states.get(&3).unwrap().transport_active);
    }

    #[test]
    fn syncing_live_slots_drops_removed_slot_state() {
        let mut engine = ContinuousRenderEngineState::default();

        engine.sync_live_slots(&FxHashSet::from_iter([4, 5]));
        engine.set_slot_reason(4, RenderActiveReason::Editor, true);

        let removed = engine.sync_live_slots(&FxHashSet::from_iter([5]));
        assert_eq!(removed, vec![4]);
        assert!(!engine.slot_states.contains_key(&4));
        assert!(engine.slot_states.contains_key(&5));
    }

    #[test]
    fn busy_render_requeues_due_events() {
        struct BusyBackend;

        impl RenderBackend for BusyBackend {
            fn render_slot(
                &self,
                _slot_id: UniqueId,
                _sample_rate: u32,
                _frames_per_block: usize,
                _events: &[LiveRenderEvent],
                _output: &mut [f32],
                _output_channels: usize,
            ) -> SlotRuntimeRenderOutcome {
                SlotRuntimeRenderOutcome::Busy
            }

            fn stop_slot(&self, _slot_id: UniqueId) {}
        }

        let mut engine = ContinuousRenderEngineState::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([12]));
        engine.queue_audition_note(12, 1_000, make_audition_note(4));
        engine.process_block(&BusyBackend, &mut output, 2, 1_000);

        let slot_state = engine.slot_states.get(&12).unwrap();
        assert!(slot_state.audition_active);
        assert_eq!(slot_state.queued_events.len(), 2);
        assert!(slot_state
            .queued_events
            .iter()
            .all(|event| event.frames_until_dispatch == 0));
        assert!(slot_state
            .queued_events
            .iter()
            .any(|event| event.kind == LiveRenderEventKind::NoteOn));
        assert!(slot_state
            .queued_events
            .iter()
            .any(|event| event.kind == LiveRenderEventKind::NoteOff));
    }

    #[test]
    fn busy_render_does_not_mutate_transport_note_tracking_before_success() {
        struct BusyBackend;

        impl RenderBackend for BusyBackend {
            fn render_slot(
                &self,
                _slot_id: UniqueId,
                _sample_rate: u32,
                _frames_per_block: usize,
                _events: &[LiveRenderEvent],
                _output: &mut [f32],
                _output_channels: usize,
            ) -> SlotRuntimeRenderOutcome {
                SlotRuntimeRenderOutcome::Busy
            }

            fn stop_slot(&self, _slot_id: UniqueId) {}
        }

        let mut engine = ContinuousRenderEngineState::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([18]));
        engine.set_transport_targets(&FxHashSet::from_iter([18]));
        engine.queue_transport_note(18, 0, 1, 64, 100);

        engine.process_block(&BusyBackend, &mut output, 2, 1_000);

        let slot_state = engine.slot_states.get(&18).unwrap();
        assert!(slot_state.active_transport_note_ids.is_empty());
        assert_eq!(
            slot_state
                .queued_events
                .iter()
                .filter(|event| event.owner == RenderEventOwner::Transport)
                .count(),
            2
        );
    }

    #[test]
    fn equal_offset_transport_note_ons_preserve_enqueue_order() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([25]));
        engine.set_transport_targets(&FxHashSet::from_iter([25]));
        engine.queue_transport_note(25, 0, 4, 24, 96);
        engine.queue_transport_note(25, 0, 4, 60, 100);

        engine.process_block(&backend, &mut output, 2, 1_000);

        let calls = backend.render_calls.borrow();
        assert_eq!(calls.len(), 1);
        let note_on_pitches = calls[0]
            .events
            .iter()
            .filter(|event| event.kind == LiveRenderEventKind::NoteOn)
            .map(|event| event.pitch)
            .collect::<Vec<_>>();
        assert_eq!(note_on_pitches, vec![24, 60]);
    }

    #[test]
    fn supported_slot_mix_scales_rendered_output_gain() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend {
            output_fill: 1.0,
            ..Default::default()
        };
        let mut output = [0.0_f32; 4];

        engine.sync_live_slots(&FxHashSet::from_iter([3]));
        engine.set_slot_reason(3, RenderActiveReason::Editor, true);
        engine.set_slot_mix(
            3,
            EffectivePlaybackMix {
                volume: 64,
                pan: 64,
                muted: false,
            },
        );

        engine.process_block(&backend, &mut output, 2, 48_000);

        let expected = 64.0 / 127.0;
        assert!(output
            .iter()
            .all(|sample| (*sample - expected).abs() < 0.000_1));
    }

    #[test]
    fn supported_slot_mix_can_mute_rendered_output_without_stopping_session() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend {
            output_fill: 1.0,
            ..Default::default()
        };
        let mut output = [0.0_f32; 4];

        engine.sync_live_slots(&FxHashSet::from_iter([6]));
        engine.set_slot_reason(6, RenderActiveReason::Editor, true);
        engine.set_slot_mix(
            6,
            EffectivePlaybackMix {
                volume: 127,
                pan: 64,
                muted: true,
            },
        );

        engine.process_block(&backend, &mut output, 2, 48_000);

        assert!(output.iter().all(|sample| sample.abs() < 0.000_1));
        assert!(backend.stop_calls.borrow().is_empty());
    }

    #[test]
    fn supported_slot_mix_applies_pan_balance_to_stereo_output() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend {
            output_fill: 1.0,
            ..Default::default()
        };
        let mut output = [0.0_f32; 4];

        engine.sync_live_slots(&FxHashSet::from_iter([8]));
        engine.set_slot_reason(8, RenderActiveReason::Editor, true);
        engine.set_slot_mix(
            8,
            EffectivePlaybackMix {
                volume: 127,
                pan: 0,
                muted: false,
            },
        );

        engine.process_block(&backend, &mut output, 2, 48_000);

        assert!((output[0] - 1.0).abs() < 0.000_1);
        assert!(output[1].abs() < 0.000_1);
        assert!((output[2] - 1.0).abs() < 0.000_1);
        assert!(output[3].abs() < 0.000_1);
    }

    #[test]
    fn queued_transport_note_dispatches_note_on_and_note_off_across_blocks() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([14]));
        engine.set_transport_targets(&FxHashSet::from_iter([14]));
        engine.queue_transport_note(14, 2, 6, 67, 101);

        engine.process_block(&backend, &mut output, 2, 1_000);
        engine.process_block(&backend, &mut output, 2, 1_000);
        engine.process_block(&backend, &mut output, 2, 1_000);

        let calls = backend.render_calls.borrow();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0].events.len(), 1);
        assert_eq!(calls[0].events[0].kind, LiveRenderEventKind::NoteOn);
        assert_eq!(calls[0].events[0].sample_offset, 2);
        assert_eq!(calls[0].events[0].pitch, 67);
        assert_eq!(calls[0].events[0].velocity, 101);
        assert!(calls[1].events.is_empty());
        assert_eq!(calls[2].events.len(), 1);
        assert_eq!(calls[2].events[0].kind, LiveRenderEventKind::NoteOff);
        assert_eq!(calls[2].events[0].sample_offset, 0);
        assert_eq!(calls[2].events[0].pitch, 67);
        assert_eq!(calls[2].events[0].velocity, 0);
    }

    #[test]
    fn pending_transport_work_tracks_multiple_slots_until_transport_cleanup_runs() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([19, 20]));
        engine.set_transport_targets(&FxHashSet::from_iter([19, 20]));
        engine.queue_transport_note(19, 0, 8, 60, 100);
        engine.queue_transport_note(20, 64, 8, 67, 100);

        assert!(engine.has_pending_transport_work());

        engine.process_block(&backend, &mut output, 2, 1_000);

        assert!(engine.has_pending_transport_work());

        let slots_to_stop = engine.clear_transport_playback();

        assert_eq!(slots_to_stop, vec![19, 20]);
        assert!(!engine.has_pending_transport_work());
    }

    #[test]
    fn clearing_transport_playback_drops_transport_queue_and_stops_active_transport_slots() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend::default();
        let mut output = [0.0_f32; 8];

        engine.sync_live_slots(&FxHashSet::from_iter([21]));
        engine.set_transport_targets(&FxHashSet::from_iter([21]));
        engine.queue_transport_note(21, 0, 8, 64, 100);
        engine.process_block(&backend, &mut output, 2, 1_000);

        let slot_state = engine.slot_states.get(&21).unwrap();
        assert!(slot_state.transport_active);
        assert_eq!(slot_state.active_transport_note_ids.len(), 1);
        assert!(slot_state.has_queued_transport_events());

        let slots_to_stop = engine.clear_transport_playback();

        assert_eq!(slots_to_stop, vec![21]);
        let slot_state = engine.slot_states.get(&21).unwrap();
        assert!(!slot_state.transport_active);
        assert!(slot_state.active_transport_note_ids.is_empty());
        assert!(!slot_state.has_queued_transport_events());
    }

    #[test]
    fn clearing_transport_playback_stops_slots_with_queued_transport_work_before_note_on() {
        let mut engine = ContinuousRenderEngineState::default();

        engine.sync_live_slots(&FxHashSet::from_iter([23]));
        engine.set_transport_targets(&FxHashSet::from_iter([23]));
        engine.queue_transport_note(23, 64, 8, 64, 100);

        let slots_to_stop = engine.clear_transport_playback();

        assert_eq!(slots_to_stop, vec![23]);
        let slot_state = engine.slot_states.get(&23).unwrap();
        assert!(!slot_state.transport_active);
        assert!(slot_state.active_transport_note_ids.is_empty());
        assert!(!slot_state.has_queued_transport_events());
    }

    #[test]
    fn clearing_transport_playback_preserves_non_transport_render_activity() {
        let mut engine = ContinuousRenderEngineState::default();

        engine.sync_live_slots(&FxHashSet::from_iter([24]));
        engine.set_transport_targets(&FxHashSet::from_iter([24]));
        engine.set_slot_reason(24, RenderActiveReason::Editor, true);
        engine.queue_transport_note(24, 0, 8, 64, 100);

        let slots_to_stop = engine.clear_transport_playback();

        assert_eq!(slots_to_stop, vec![24]);
        let slot_state = engine.slot_states.get(&24).unwrap();
        assert!(slot_state.editor_active);
        assert!(!slot_state.transport_active);
        assert!(slot_state.active_transport_note_ids.is_empty());
        assert!(!slot_state.has_queued_transport_events());
        assert!(slot_state.is_render_active());
    }

    #[test]
    fn supported_slot_mix_updates_apply_to_in_flight_render_without_restart() {
        let mut engine = ContinuousRenderEngineState::default();
        let backend = FakeRenderBackend {
            output_fill: 1.0,
            ..Default::default()
        };
        let mut output = [0.0_f32; 4];

        engine.sync_live_slots(&FxHashSet::from_iter([22]));
        engine.set_slot_reason(22, RenderActiveReason::Editor, true);
        engine.set_slot_mix(
            22,
            EffectivePlaybackMix {
                volume: 127,
                pan: 64,
                muted: false,
            },
        );
        engine.process_block(&backend, &mut output, 2, 48_000);
        assert!(output.iter().all(|sample| (*sample - 1.0).abs() < 0.000_1));

        output.fill(0.0);
        engine.set_slot_mix(
            22,
            EffectivePlaybackMix {
                volume: 32,
                pan: 127,
                muted: false,
            },
        );
        engine.process_block(&backend, &mut output, 2, 48_000);

        assert!(output[0].abs() < 0.000_1);
        assert!((output[1] - (32.0 / 127.0)).abs() < 0.000_1);
        assert!(output[2].abs() < 0.000_1);
        assert!((output[3] - (32.0 / 127.0)).abs() < 0.000_1);
    }
}
