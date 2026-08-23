use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

const MAX_STREAM_BUFFER_SAMPLES: usize = 48_000 * 2 * 2;
const MAX_AUDIO_CALLBACK_FRAMES: usize = 16_384;
const MASTER_LIMIT_THRESHOLD: f32 = 0.95;
static MASTER_PEAK_BITS: AtomicU32 = AtomicU32::new(0);
static CURRENT_MASTER_PEAK_BITS: AtomicU32 = AtomicU32::new(0);
static MASTER_OVERLOAD_COUNT: AtomicU64 = AtomicU64::new(0);

/// Shared buffer between a producer thread and the audio mixer.
#[cfg_attr(not(test), allow(dead_code))]
pub struct StreamBuffer {
    buffer: ArrayQueue<f32>,
    finished: AtomicBool,
    canceled: AtomicBool,
    dropped_samples: AtomicU64,
}

impl StreamBuffer {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new() -> Self {
        Self {
            buffer: ArrayQueue::new(MAX_STREAM_BUFFER_SAMPLES),
            finished: AtomicBool::new(false),
            canceled: AtomicBool::new(false),
            dropped_samples: AtomicU64::new(0),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn push(&self, samples: &[f32]) {
        if self.canceled.load(Ordering::Acquire) {
            return;
        }

        let mut accepted = 0;
        for sample in samples {
            if self.buffer.push(*sample).is_err() {
                break;
            }
            accepted += 1;
        }
        self.dropped_samples
            .fetch_add((samples.len() - accepted) as u64, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn finish(&self) {
        self.finished.store(true, Ordering::Release);
    }

    pub fn drain_mix(&self, output: &mut [f32]) -> usize {
        let mut sample_count = 0;
        for destination in output {
            let Some(sample) = self.buffer.pop() else {
                break;
            };
            *destination += sample;
            sample_count += 1;
        }
        sample_count
    }

    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::Release);
        self.finished.store(true, Ordering::Release);
        while self.buffer.pop().is_some() {}
    }

    pub fn is_exhausted(&self) -> bool {
        self.finished.load(Ordering::Acquire) && self.buffer.is_empty()
    }
}

/// A single playing voice backed by a disk-streaming buffer.
struct Voice {
    stream: Arc<StreamBuffer>,
    channels: usize,
    generation: u64,
    remap_scratch: Vec<f32>,
}

/// A handle to submit new voices into the mixer.
/// This is cheaply cloneable and can be sent to any thread.
#[derive(Clone)]
pub struct MixerHandle {
    commands: Arc<ArrayQueue<Box<Voice>>>,
    generation: Arc<AtomicU64>,
    claimed_voices: Arc<AtomicUsize>,
}

/// The mixer reads from the voice list in the audio callback.
/// This must only be used from the audio thread.
pub struct MixerReader {
    commands: Arc<ArrayQueue<Box<Voice>>>,
    generation: Arc<AtomicU64>,
    claimed_voices: Arc<AtomicUsize>,
    voices: [Option<Box<Voice>>; MAX_VOICES],
}

/// Lock-free voice list using a fixed-size slot array.
/// Slots are claimed atomically and recycled when voices finish.
const MAX_VOICES: usize = 64;

/// Create a linked mixer handle and reader pair.
pub fn create_mixer() -> (MixerHandle, MixerReader) {
    let commands = Arc::new(ArrayQueue::new(MAX_VOICES));
    let generation = Arc::new(AtomicU64::new(0));
    let claimed_voices = Arc::new(AtomicUsize::new(0));
    (
        MixerHandle {
            commands: commands.clone(),
            generation: generation.clone(),
            claimed_voices: claimed_voices.clone(),
        },
        MixerReader {
            commands,
            generation,
            claimed_voices,
            voices: std::array::from_fn(|_| None),
        },
    )
}

impl MixerHandle {
    /// Submit a streaming voice. The disk thread pushes decoded audio
    /// into the stream buffer; the mixer drains it in the audio callback.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn play(&self, stream: Arc<StreamBuffer>, channels: usize) -> bool {
        if channels == 0 {
            return false;
        }
        if self
            .claimed_voices
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < MAX_VOICES).then_some(count + 1)
            })
            .is_err()
        {
            return false;
        }
        let generation = self.generation.load(Ordering::Acquire);
        let voice = Box::new(Voice {
            stream,
            channels,
            generation,
            remap_scratch: vec![0.0; MAX_AUDIO_CALLBACK_FRAMES.saturating_mul(channels)],
        });
        if self.commands.push(voice).is_err() {
            self.claimed_voices.fetch_sub(1, Ordering::AcqRel);
            return false;
        }
        true
    }

    /// Drop all active voices immediately so no already-started notes continue sounding.
    pub fn stop_all(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        while let Some(voice) = self.commands.pop() {
            voice.stream.cancel();
        }
        self.claimed_voices.store(0, Ordering::Release);
    }
}

impl MixerReader {
    pub fn mix_into_buffer(&mut self, output: &mut [f32], channels: usize) {
        let generation = self.generation.load(Ordering::Acquire);
        for slot in &mut self.voices {
            if slot
                .as_ref()
                .is_some_and(|voice| voice.generation != generation)
            {
                if let Some(voice) = slot.take() {
                    voice.stream.cancel();
                }
            }
        }
        while let Some(voice) = self.commands.pop() {
            if voice.generation != generation {
                voice.stream.cancel();
                continue;
            }
            let Some(slot) = self.voices.iter_mut().find(|slot| slot.is_none()) else {
                voice.stream.cancel();
                self.claimed_voices.fetch_sub(1, Ordering::AcqRel);
                continue;
            };
            *slot = Some(voice);
        }

        for slot in &mut self.voices {
            let Some(voice) = slot.as_mut() else {
                continue;
            };

            let voice_channels = voice.channels;

            // Drain decoded audio from the stream buffer and mix into output.
            // When voice and output channel counts match we can mix directly.
            if voice_channels == channels {
                voice.stream.drain_mix(output);
            } else {
                let needed = output.len() / channels * voice_channels;
                let usable = needed.min(voice.remap_scratch.len());
                voice.remap_scratch[..usable].fill(0.0);
                let drained = voice.stream.drain_mix(&mut voice.remap_scratch[..usable]);
                let frames = drained / voice_channels;

                for frame in 0..frames {
                    for out_ch in 0..channels {
                        let voice_ch = if voice_channels == 1 {
                            0
                        } else {
                            out_ch.min(voice_channels - 1)
                        };
                        output[frame * channels + out_ch] +=
                            voice.remap_scratch[frame * voice_channels + voice_ch];
                    }
                }
            }

            // If the stream is finished and drained, recycle the slot
            if voice.stream.is_exhausted() {
                *slot = None;
                self.claimed_voices.fetch_sub(1, Ordering::AcqRel);
            }
        }
    }

    pub fn limit_buffer(output: &mut [f32]) {
        let mut peak = 0.0_f32;
        let mut overloads = 0_u64;
        for sample in output.iter_mut() {
            let magnitude = sample.abs();
            peak = peak.max(magnitude);
            overloads += u64::from(magnitude > 1.0);
            if magnitude > MASTER_LIMIT_THRESHOLD {
                let normalized_excess =
                    (magnitude - MASTER_LIMIT_THRESHOLD) / (1.0 - MASTER_LIMIT_THRESHOLD);
                let limited = MASTER_LIMIT_THRESHOLD
                    + (1.0 - MASTER_LIMIT_THRESHOLD) * normalized_excess.tanh();
                *sample = sample.signum() * limited;
            }
        }
        MASTER_OVERLOAD_COUNT.fetch_add(overloads, Ordering::Relaxed);
        CURRENT_MASTER_PEAK_BITS.store(peak.to_bits(), Ordering::Relaxed);
        let mut recorded = f32::from_bits(MASTER_PEAK_BITS.load(Ordering::Relaxed));
        while peak > recorded {
            match MASTER_PEAK_BITS.compare_exchange_weak(
                recorded.to_bits(),
                peak.to_bits(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(bits) => recorded = f32::from_bits(bits),
            }
        }
    }

    /// Mix all active voices into the output buffer (additive mixing).
    /// Called from the audio callback — must be fast.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn fill_buffer(&mut self, output: &mut [f32], channels: usize) {
        // Zero the output first
        output.fill(0.0);

        self.mix_into_buffer(output, channels);
        Self::limit_buffer(output);
    }
}

pub(crate) fn master_overload_count() -> u64 {
    MASTER_OVERLOAD_COUNT.load(Ordering::Relaxed)
}

pub(crate) fn master_peak() -> f32 {
    f32::from_bits(MASTER_PEAK_BITS.load(Ordering::Relaxed))
}

pub(crate) fn current_master_peak() -> f32 {
    f32::from_bits(CURRENT_MASTER_PEAK_BITS.load(Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_all_clears_active_voices_immediately() {
        let (handle, mut reader) = create_mixer();
        let stream = Arc::new(StreamBuffer::new());
        stream.push(&[0.5, 0.5, 0.25, 0.25]);

        assert!(handle.play(stream, 2));

        handle.stop_all();

        let mut output = [0.0_f32; 4];
        reader.fill_buffer(&mut output, 2);

        assert_eq!(output, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn master_limiter_preserves_normal_levels_and_bounds_overloads() {
        let mut output = [-0.5, 0.5, -2.0, 2.0];

        MixerReader::limit_buffer(&mut output);

        assert_eq!(output[0], -0.5);
        assert_eq!(output[1], 0.5);
        assert!(output[2] >= -1.0 && output[2] < -MASTER_LIMIT_THRESHOLD);
        assert!(output[3] <= 1.0 && output[3] > MASTER_LIMIT_THRESHOLD);
    }

    #[test]
    fn stop_all_allows_immediate_new_voice() {
        let (handle, mut reader) = create_mixer();
        let old_stream = Arc::new(StreamBuffer::new());
        old_stream.push(&[0.75, 0.75]);
        assert!(handle.play(old_stream, 2));
        handle.stop_all();

        let new_stream = Arc::new(StreamBuffer::new());
        new_stream.push(&[0.25, 0.25]);
        assert!(handle.play(new_stream, 2));

        let mut output = [0.0; 2];
        reader.mix_into_buffer(&mut output, 2);
        assert_eq!(output, [0.25, 0.25]);
    }
}
