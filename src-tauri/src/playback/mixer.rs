use crate::playback::streamer::StreamBuffer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A single playing voice backed by a disk-streaming buffer.
struct Voice {
    stream: Arc<StreamBuffer>,
    channels: usize,
    finished: AtomicBool,
}

/// A handle to submit new voices into the mixer.
/// This is cheaply cloneable and can be sent to any thread.
#[derive(Clone)]
pub struct MixerHandle {
    voices: Arc<VoiceList>,
}

/// The mixer reads from the voice list in the audio callback.
/// This must only be used from the audio thread.
pub struct MixerReader {
    voices: Arc<VoiceList>,
}

/// Lock-free voice list using a fixed-size slot array.
/// Slots are claimed atomically and recycled when voices finish.
const MAX_VOICES: usize = 64;

struct VoiceList {
    slots: Box<[VoiceSlot; MAX_VOICES]>,
}

struct VoiceSlot {
    voice: std::sync::Mutex<Option<Arc<Voice>>>,
    active: AtomicBool,
}

impl VoiceList {
    fn new() -> Self {
        VoiceList {
            slots: Box::new(std::array::from_fn(|_| VoiceSlot {
                voice: std::sync::Mutex::new(None),
                active: AtomicBool::new(false),
            })),
        }
    }
}

/// Create a linked mixer handle and reader pair.
pub fn create_mixer() -> (MixerHandle, MixerReader) {
    let voices = Arc::new(VoiceList::new());
    (
        MixerHandle {
            voices: voices.clone(),
        },
        MixerReader { voices },
    )
}

impl MixerHandle {
    /// Submit a streaming voice. The disk thread pushes decoded audio
    /// into the stream buffer; the mixer drains it in the audio callback.
    pub fn play(&self, stream: Arc<StreamBuffer>, channels: usize) -> bool {
        let voice = Arc::new(Voice {
            stream,
            channels,
            finished: AtomicBool::new(false),
        });

        // Find a free slot
        for slot in self.voices.slots.iter() {
            if !slot.active.load(Ordering::Acquire)
                && slot
                    .active
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
            {
                let mut guard = slot.voice.lock().unwrap();
                *guard = Some(voice);
                return true;
            }
        }

        eprintln!(
            "Mixer: all {} voice slots are full, dropping note",
            MAX_VOICES
        );
        false
    }

    /// Drop all active voices immediately so no already-started notes continue sounding.
    pub fn stop_all(&self) {
        for slot in self.voices.slots.iter() {
            if !slot.active.swap(false, Ordering::AcqRel) {
                continue;
            }

            let mut guard = slot.voice.lock().unwrap();
            if let Some(voice) = guard.as_ref() {
                voice.stream.cancel();
                voice.finished.store(true, Ordering::Relaxed);
            }
            *guard = None;
        }
    }
}

impl MixerReader {
    /// Mix all active voices into the output buffer (additive mixing).
    /// Called from the audio callback — must be fast.
    pub fn fill_buffer(&self, output: &mut [f32], channels: usize) {
        // Zero the output first
        output.fill(0.0);

        for slot in self.voices.slots.iter() {
            if !slot.active.load(Ordering::Acquire) {
                continue;
            }

            // We briefly lock here but only to clone the Arc — very fast
            let voice = {
                let guard = slot.voice.lock().unwrap();
                match &*guard {
                    Some(v) => v.clone(),
                    None => {
                        slot.active.store(false, Ordering::Release);
                        continue;
                    }
                }
            };

            if voice.finished.load(Ordering::Relaxed) {
                let mut guard = slot.voice.lock().unwrap();
                *guard = None;
                slot.active.store(false, Ordering::Release);
                continue;
            }

            let voice_channels = voice.channels;

            // Drain decoded audio from the stream buffer and mix into output.
            // When voice and output channel counts match we can mix directly.
            if voice_channels == channels {
                voice.stream.drain_mix(output);
            } else {
                // Channel mismatch — drain into a temporary buffer then remap
                let needed = output.len() / channels * voice_channels;
                let mut tmp = vec![0.0_f32; needed];
                let drained = voice.stream.drain_mix(&mut tmp);
                let frames = drained / voice_channels;

                for frame in 0..frames {
                    for out_ch in 0..channels {
                        let voice_ch = if voice_channels == 1 {
                            0
                        } else {
                            out_ch.min(voice_channels - 1)
                        };
                        output[frame * channels + out_ch] += tmp[frame * voice_channels + voice_ch];
                    }
                }
            }

            // If the stream is finished and drained, recycle the slot
            if voice.stream.is_exhausted() {
                voice.finished.store(true, Ordering::Relaxed);
                let mut guard = slot.voice.lock().unwrap();
                *guard = None;
                slot.active.store(false, Ordering::Release);
            }
        }

        // Clamp output to prevent clipping from overlapping voices
        for sample in output.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_all_clears_active_voices_immediately() {
        let (handle, reader) = create_mixer();
        let stream = Arc::new(StreamBuffer::new());
        stream.push(&[0.5, 0.5, 0.25, 0.25]);

        assert!(handle.play(stream, 2));

        handle.stop_all();

        let mut output = [0.0_f32; 4];
        reader.fill_buffer(&mut output, 2);

        assert_eq!(output, [0.0, 0.0, 0.0, 0.0]);
    }
}
