use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// A single playing voice with its pre-rendered audio buffer.
struct Voice {
    data: Vec<f32>,
    channels: usize,
    position: AtomicUsize,
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
    /// Submit a new voice with pre-rendered interleaved audio data.
    /// `channels` must match the output stream channel count.
    /// Returns false if all voice slots are full (voice is dropped).
    pub fn play(&self, data: Vec<f32>, channels: usize) -> bool {
        let voice = Arc::new(Voice {
            data,
            channels,
            position: AtomicUsize::new(0),
            finished: AtomicBool::new(false),
        });

        // Find a free slot
        for slot in self.voices.slots.iter() {
            if !slot.active.load(Ordering::Acquire) {
                // Try to claim this slot
                if slot
                    .active
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    let mut guard = slot.voice.lock().unwrap();
                    *guard = Some(voice);
                    return true;
                }
            }
        }

        eprintln!(
            "Mixer: all {} voice slots are full, dropping note",
            MAX_VOICES
        );
        false
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
                // Recycle slot
                let mut guard = slot.voice.lock().unwrap();
                *guard = None;
                slot.active.store(false, Ordering::Release);
                continue;
            }

            let pos = voice.position.load(Ordering::Relaxed);
            let remaining = voice.data.len().saturating_sub(pos);

            if remaining == 0 {
                voice.finished.store(true, Ordering::Relaxed);
                let mut guard = slot.voice.lock().unwrap();
                *guard = None;
                slot.active.store(false, Ordering::Release);
                continue;
            }

            // Mix this voice into the output buffer
            // Handle channel mismatch: voice may be mono, output may be stereo
            let voice_channels = voice.channels;
            let output_frames = output.len() / channels;
            let voice_frames_available = remaining / voice_channels;
            let frames_to_mix = output_frames.min(voice_frames_available);

            for frame in 0..frames_to_mix {
                for out_ch in 0..channels {
                    let voice_ch = if voice_channels == 1 {
                        0 // Mono: use the same sample for all output channels
                    } else {
                        out_ch.min(voice_channels - 1)
                    };
                    let voice_idx = pos + frame * voice_channels + voice_ch;
                    let out_idx = frame * channels + out_ch;
                    output[out_idx] += voice.data[voice_idx];
                }
            }

            voice
                .position
                .store(pos + frames_to_mix * voice_channels, Ordering::Relaxed);

            // If we've consumed all the data, mark as finished
            if frames_to_mix == voice_frames_available {
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
