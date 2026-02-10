/// Disk Streaming for Sample Playback
///
/// All sample audio is streamed from .scvi containers on disk.
/// A background thread decodes FLAC one packet at a time (~4096
/// frames per packet), resamples, pitch-shifts, applies envelope
/// and volume/pan, then pushes the processed chunk immediately.
/// The mixer receives a steady stream of audio with no gaps.
///
/// ```text
/// note-on
///   │
///   ▼
/// ┌─────────────────────────────────┐
/// │  Disk thread (per packet)       │
/// │                                 │
/// │  ┌─ decode 1 FLAC packet ──┐    │  push   ┌──────────────────┐
/// │  │  resample (~4096 frames)│────│────────►│  StreamBuffer     │
/// │  │  pitch / env / vol / pan│    │         │  (Mutex<VecDeque>)│
/// │  └──── loop ───────────────┘    │         └────────┬─────────┘
/// └─────────────────────────────────┘                  │ drain
///                                                      ▼
///                                             ┌──────────────────┐
///                                             │  Mixer voice      │
///                                             │  audio callback   │
///                                             └──────────────────┘
/// ```
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

// ── Stream buffer ──────────────────────────────────────────────────────

/// Shared buffer between the disk streaming thread and the mixer voice.
/// The streamer pushes decoded audio; the mixer drains it.
pub struct StreamBuffer {
    buffer: Mutex<VecDeque<f32>>,
    /// Set when the streamer has pushed all remaining audio
    finished: AtomicBool,
}

impl StreamBuffer {
    pub fn new() -> Self {
        // Pre-allocate ~0.5 seconds of stereo audio
        StreamBuffer {
            buffer: Mutex::new(VecDeque::with_capacity(48000 * 2)),
            finished: AtomicBool::new(false),
        }
    }

    /// Push decoded interleaved samples (called by streamer thread).
    pub fn push(&self, samples: &[f32]) {
        let mut buf = self.buffer.lock().unwrap();
        buf.extend(samples);
    }

    /// Drain samples into the output buffer, mixing additively.
    /// Called by the mixer's audio callback. Returns the number of
    /// interleaved samples actually mixed.
    pub fn drain_mix(&self, output: &mut [f32]) -> usize {
        let mut buf = self.buffer.lock().unwrap();
        let n = output.len().min(buf.len());
        for i in 0..n {
            output[i] += buf[i];
        }
        buf.drain(..n);
        n
    }

    /// Signal that the streamer has finished producing audio.
    pub fn mark_finished(&self) {
        self.finished.store(true, Ordering::Release);
    }

    /// True when the streamer is done AND the buffer is empty.
    pub fn is_exhausted(&self) -> bool {
        self.finished.load(Ordering::Acquire) && self.buffer.lock().unwrap().is_empty()
    }
}

// ── Bounded file reader ────────────────────────────────────────────────

/// A reader that exposes a sub-range of a file as if it were a standalone
/// stream. This lets symphonia decode FLAC data directly from the .scvi
/// container without reading the entire blob into memory.
struct BoundedReader {
    file: File,
    start: u64,
    length: u64,
    position: u64,
}

impl BoundedReader {
    fn new(mut file: File, start: u64, length: u64) -> std::io::Result<Self> {
        file.seek(SeekFrom::Start(start))?;
        Ok(BoundedReader {
            file,
            start,
            length,
            position: 0,
        })
    }
}

impl Read for BoundedReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = (self.length - self.position) as usize;
        if remaining == 0 {
            return Ok(0);
        }
        let to_read = buf.len().min(remaining);
        let n = self.file.read(&mut buf[..to_read])?;
        self.position += n as u64;
        Ok(n)
    }
}

impl Seek for BoundedReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::End(p) => self.length as i64 + p,
            SeekFrom::Current(p) => self.position as i64 + p,
        };
        if new_pos < 0 || new_pos > self.length as i64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek out of bounds",
            ));
        }
        self.position = new_pos as u64;
        self.file
            .seek(SeekFrom::Start(self.start + self.position))?;
        Ok(self.position)
    }
}

// Symphonia requires MediaSource which is Read + Seek + Send + Sync (via its extension trait).
// Our BoundedReader satisfies these. We need to implement the symphonia MediaSource trait:
impl symphonia::core::io::MediaSource for BoundedReader {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.length)
    }
}

// ── Stream request ─────────────────────────────────────────────────────

/// Everything the streaming thread needs to decode and deliver
/// a sample from disk.
pub struct StreamRequest {
    /// Path to the .scvi container file
    pub container_path: PathBuf,
    /// Absolute byte offset of the Vorbis data within the container
    pub ogg_offset: u64,
    /// Byte length of the Vorbis data
    pub ogg_length: u64,
    /// Source sample rate of the Vorbis data
    pub source_sample_rate: u32,
    /// Target output sample rate
    pub target_sample_rate: u32,
    /// Number of audio channels
    pub channels: usize,
    /// Pitch offset in semitones (applied via interpolation)
    pub pitch_offset: i32,
    /// Total maximum output frames to produce (duration + release)
    pub max_output_frames: usize,
    /// Output frame at which fade-out begins
    pub fade_start_frame: usize,
    /// Volume gain (0.0–1.0) applied to every output sample
    pub volume_gain: f32,
    /// Left channel gain for stereo panning
    pub pan_left: f32,
    /// Right channel gain for stereo panning
    pub pan_right: f32,
    /// Destination buffer, shared with the mixer voice
    pub stream_buffer: Arc<StreamBuffer>,
}

/// Spawn a background thread that decodes the sustain portion of a sample
/// from disk and pushes it into the stream buffer.
pub fn start_stream(request: StreamRequest) {
    std::thread::Builder::new()
        .name("disk-stream".into())
        .spawn(move || {
            if let Err(e) = run_stream(request) {
                eprintln!("Disk stream error: {}", e);
            }
        })
        .expect("Failed to spawn disk streaming thread");
}

fn run_stream(req: StreamRequest) -> Result<(), Box<dyn std::error::Error>> {
    // Open a bounded reader directly on the Vorbis data within the container
    let file = File::open(&req.container_path)?;
    let reader = BoundedReader::new(file, req.ogg_offset, req.ogg_length)?;
    let mss = MediaSourceStream::new(Box::new(reader), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("ogg");

    let probed = get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No audio track found")?;

    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;
    let track_id = track.id;
    let channels = req.channels;

    let pitch_ratio = if req.pitch_offset != 0 {
        2.0_f64.powf(req.pitch_offset as f64 / 12.0)
    } else {
        1.0
    };
    let needs_resample = req.source_sample_rate != req.target_sample_rate;
    let resample_ratio = req.target_sample_rate as f64 / req.source_sample_rate as f64;

    let max_output = req.max_output_frames;

    // ── Incremental state ─────────────────────────────────────────────
    //
    // We carry a small tail buffer of source samples across packets to
    // handle interpolation at packet boundaries (resample and pitch-shift
    // both need to peek one frame ahead).

    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    // Source samples carried over from the previous packet (at most 1
    // frame of look-ahead needed for linear interpolation).
    let mut tail: Vec<f32> = Vec::new();

    // Fractional position within the resampler output, carried across
    // packets so there are no discontinuities.
    let mut resample_frac: f64 = 0.0;

    // Output state
    let mut src_pos = 0.0_f64; // fractional position in resampled data
    let mut output_frame: usize = 0;

    let chunk_size: usize = 1024;
    let mut chunk_buf: Vec<f32> = Vec::with_capacity(chunk_size * channels);

    // ── Packet loop ───────────────────────────────────────────────────
    //
    // Each iteration: decode one FLAC packet (~4096 frames), resample
    // just those frames, pitch-shift + envelope + vol/pan, and push
    // immediately. The mixer gets audio within <1ms of thread start.

    loop {
        if output_frame >= max_output {
            break;
        }

        // — Decode one packet —
        let packet_samples = match format.next_packet() {
            Ok(packet) => {
                if packet.track_id() != track_id {
                    continue;
                }
                match decoder.decode(&packet) {
                    Ok(decoded) => {
                        if sample_buf.is_none() {
                            sample_buf = Some(SampleBuffer::<f32>::new(
                                decoded.capacity() as u64,
                                *decoded.spec(),
                            ));
                        }
                        let buf = sample_buf.as_mut().unwrap();
                        buf.copy_interleaved_ref(decoded);
                        buf.samples().to_vec()
                    }
                    Err(symphonia::core::errors::Error::IoError(_)) => break,
                    Err(e) => return Err(e.into()),
                }
            }
            Err(symphonia::core::errors::Error::IoError(_)) => break,
            Err(symphonia::core::errors::Error::ResetRequired) => continue,
            Err(e) => return Err(e.into()),
        };

        // — Build source chunk: tail from previous packet + new packet —
        let mut source = std::mem::take(&mut tail);
        source.extend_from_slice(&packet_samples);
        let source_frames = source.len() / channels;

        if source_frames == 0 {
            continue;
        }

        // — Resample this chunk —
        let resampled = if needs_resample {
            // How many output frames we can produce, accounting for the
            // fractional position carried from the previous chunk.
            let usable_src = source_frames.saturating_sub(1); // hold back 1 for interp
            let out_frames =
                ((usable_src as f64 - resample_frac) * resample_ratio).floor() as usize;

            let mut output = Vec::with_capacity(out_frames * channels);

            for i in 0..out_frames {
                let src_f = resample_frac + i as f64 / resample_ratio;
                let idx = src_f.floor() as usize;
                let frac = (src_f - idx as f64) as f32;

                for ch in 0..channels {
                    let s0 = source[idx * channels + ch];
                    let s1 = if idx + 1 < source_frames {
                        source[(idx + 1) * channels + ch]
                    } else {
                        s0
                    };
                    output.push(s0 + (s1 - s0) * frac);
                }
            }

            // Advance fractional position: we consumed `out_frames` output
            // frames worth of source.
            let consumed_src = resample_frac + out_frames as f64 / resample_ratio;
            let consumed_whole = consumed_src.floor() as usize;
            resample_frac = consumed_src - consumed_whole as f64;

            // Keep unconsumed source frames as tail for next iteration
            let keep_from = consumed_whole * channels;
            if keep_from < source.len() {
                tail = source[keep_from..].to_vec();
            }

            output
        } else {
            // No resampling — keep last frame as tail for pitch interp
            if source_frames > 1 {
                tail = source[(source_frames - 1) * channels..].to_vec();
            }
            // Use all but the last frame (tail), unless this is the only frame
            let use_frames = if source_frames > 1 {
                source_frames - 1
            } else {
                source_frames
            };
            source[..use_frames * channels].to_vec()
        };

        let resampled_frames = resampled.len() / channels;
        if resampled_frames == 0 {
            continue;
        }

        // — Pitch-shift, envelope, volume/pan, push —
        let available = resampled_frames;

        loop {
            if output_frame >= max_output {
                break;
            }

            chunk_buf.clear();
            let frames_this_chunk = chunk_size.min(max_output - output_frame);
            let mut need_more = false;

            for _ in 0..frames_this_chunk {
                let read_pos = src_pos * pitch_ratio;
                let idx = read_pos.floor() as usize;
                let frac = (read_pos - idx as f64) as f32;

                if idx + 1 >= available {
                    // Need more data — save fractional position for next packet
                    need_more = true;
                    break;
                }

                // Envelope gain
                let gain = if output_frame >= req.fade_start_frame {
                    let fade_len = max_output - req.fade_start_frame;
                    if fade_len > 0 {
                        1.0 - (output_frame - req.fade_start_frame) as f32 / fade_len as f32
                    } else {
                        0.0
                    }
                } else {
                    1.0
                };

                for ch in 0..channels {
                    let s0 = resampled[idx * channels + ch];
                    let s1 = resampled[(idx + 1) * channels + ch];
                    let sample = (s0 + (s1 - s0) * frac) * gain;
                    let sample = if channels >= 2 {
                        let pan_gain = if ch == 0 {
                            req.pan_left
                        } else if ch == 1 {
                            req.pan_right
                        } else {
                            1.0
                        };
                        sample * req.volume_gain * pan_gain
                    } else {
                        sample * req.volume_gain
                    };
                    chunk_buf.push(sample);
                }

                src_pos += 1.0;
                output_frame += 1;
            }

            if !chunk_buf.is_empty() {
                req.stream_buffer.push(&chunk_buf);
            }

            if need_more || output_frame >= max_output {
                // Adjust src_pos so it's relative to the start of the
                // next resampled chunk. The consumed frames won't exist
                // in the next iteration's resampled buffer.
                let consumed = src_pos.floor() as usize;
                src_pos -= consumed as f64;
                break;
            }
        }
    }

    req.stream_buffer.mark_finished();
    Ok(())
}
