/// SCVI Binary Container Compiler
///
/// Compiles instrument source directories into .scvi binary container files.
///
/// ## Container Layout
///
/// ```text
/// ┌──────────────────────────┐
/// │  HEADER  (fixed 64B)     │  magic, version, chunk offsets/sizes
/// ├──────────────────────────┤
/// │  META    (MessagePack)   │  manifest data with sample IDs
/// ├──────────────────────────┤
/// │  SIDX    (fixed-size)    │  per-sample index: id, offset, length, metadata
/// ├──────────────────────────┤
/// │  SDAT    (raw FLAC)      │  concatenated FLAC-encoded sample data
/// ├──────────────────────────┤
/// │  COVER   (raw bytes)     │  cover image (optional)
/// └──────────────────────────┘
/// ```
use blake3;
use rmp_serde;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Constants ──────────────────────────────────────────────────────────

const MAGIC: &[u8; 4] = b"SCVI";
const CONTAINER_VERSION: u16 = 1;
const HEADER_SIZE: u64 = 64;

// ── Source manifest structures (read from instrument JSON) ─────────────

#[derive(Deserialize)]
struct SourceMeta {
    id: String,
    name: String,
    author: String,
    version: String,
}

#[derive(Deserialize)]
struct SourceSample {
    path: String,
}

#[derive(Deserialize)]
struct SourceVelocityLayer {
    range: [u8; 2],
    samples: Vec<SourceSample>,
}

#[derive(Deserialize)]
struct SourceZone {
    pitch: u8,
    #[serde(rename = "velocityLayers")]
    velocity_layers: Vec<SourceVelocityLayer>,
}

#[derive(Deserialize)]
struct SourceArticulation {
    id: String,
    name: String,
    range: [u8; 2],
    zones: Vec<SourceZone>,
}

#[derive(Deserialize)]
struct SourceManifest {
    meta: SourceMeta,
    articulations: Vec<SourceArticulation>,
}

// ── Container META structures (serialized to MessagePack) ──────────────

#[derive(Serialize, Deserialize)]
struct ContainerMeta {
    id: String,
    name: String,
    author: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct ContainerSampleRef {
    /// Index into the SIDX table
    sample_id: u32,
}

#[derive(Serialize, Deserialize)]
struct ContainerVelocityLayer {
    range: [u8; 2],
    samples: Vec<ContainerSampleRef>,
}

#[derive(Serialize, Deserialize)]
struct ContainerZone {
    pitch: u8,
    velocity_layers: Vec<ContainerVelocityLayer>,
}

#[derive(Serialize, Deserialize)]
struct ContainerArticulation {
    id: String,
    name: String,
    range: [u8; 2],
    zones: Vec<ContainerZone>,
}

#[derive(Serialize, Deserialize)]
struct ContainerManifest {
    meta: ContainerMeta,
    articulations: Vec<ContainerArticulation>,
}

// ── SIDX entry (fixed-size, written raw) ───────────────────────────────

/// 32 bytes per sample in the index
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct SampleIndexEntry {
    /// Unique sample ID (sequential)
    sample_id: u32,
    /// Byte offset within SDAT chunk
    offset: u64,
    /// Byte length in SDAT chunk
    length: u64,
    /// Original sample rate (e.g. 44100, 48000)
    sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    channels: u8,
    /// Reserved padding bytes
    _reserved: [u8; 7],
}

const SIDX_ENTRY_SIZE: usize = std::mem::size_of::<SampleIndexEntry>();

// ── HEADER layout ──────────────────────────────────────────────────────

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct ContainerHeader {
    magic: [u8; 4],         // "SCVI"
    version: u16,           // container format version
    _reserved_h: [u8; 2],   // alignment padding
    meta_offset: u64,       // byte offset of META chunk
    meta_size: u64,         // byte size of META chunk
    sidx_offset: u64,       // byte offset of SIDX chunk
    sidx_size: u64,         // byte size of SIDX chunk (entry_count * SIDX_ENTRY_SIZE)
    sdat_offset: u64,       // byte offset of SDAT chunk
    sdat_size: u64,         // byte size of SDAT chunk
}

const HEADER_STRUCT_SIZE: usize = std::mem::size_of::<ContainerHeader>();

// ── Helpers ────────────────────────────────────────────────────────────

/// Hash file content for deduplication
fn hash_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let hash = blake3::hash(&data);
    Ok(hash.to_hex().to_string())
}

/// Write a struct as raw bytes
unsafe fn struct_as_bytes<T: Sized>(s: &T) -> &[u8] {
    std::slice::from_raw_parts(s as *const T as *const u8, std::mem::size_of::<T>())
}

/// Encode WAV to FLAC using ffmpeg, returns path to the FLAC file
fn encode_to_flac(
    wav_path: &Path,
    temp_dir: &Path,
    filename_stem: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let flac_path = temp_dir.join(format!("{}.flac", filename_stem));

    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            wav_path.to_str().unwrap(),
            "-c:a",
            "flac",
            "-compression_level",
            "8",
            flac_path.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg FLAC encode failed for {}: {}", wav_path.display(), stderr).into());
    }

    Ok(flac_path)
}

/// Read sample rate and channel count from a WAV/FLAC file using ffprobe
fn probe_audio(path: &Path) -> Result<(u32, u8), Box<dyn std::error::Error>> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-show_streams",
            "-select_streams",
            "a:0",
            "-print_format",
            "json",
            path.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!("ffprobe failed for {}", path.display()).into());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let stream = json["streams"]
        .as_array()
        .and_then(|s| s.first())
        .ok_or("No audio stream found")?;

    let sample_rate: u32 = stream["sample_rate"]
        .as_str()
        .ok_or("Missing sample_rate")?
        .parse()?;

    let channels: u8 = stream["channels"]
        .as_u64()
        .ok_or("Missing channels")? as u8;

    Ok((sample_rate, channels))
}

// ── Integrity check ────────────────────────────────────────────────────

fn integrity_check(
    instrument_dir: &Path,
    manifest: &SourceManifest,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for articulation in &manifest.articulations {
        // Check for duplicate pitches within an articulation
        let mut seen_pitches = std::collections::HashSet::new();
        for zone in &articulation.zones {
            if !seen_pitches.insert(zone.pitch) {
                errors.push(format!(
                    "Duplicate pitch {} in articulation '{}'",
                    zone.pitch, articulation.id
                ));
            }

            // Check for overlapping velocity ranges
            for (i, vl_a) in zone.velocity_layers.iter().enumerate() {
                for vl_b in zone.velocity_layers.iter().skip(i + 1) {
                    if vl_a.range[0] <= vl_b.range[1] && vl_b.range[0] <= vl_a.range[1] {
                        errors.push(format!(
                            "Overlapping velocity ranges [{}-{}] and [{}-{}] at pitch {} in '{}'",
                            vl_a.range[0], vl_a.range[1],
                            vl_b.range[0], vl_b.range[1],
                            zone.pitch, articulation.id
                        ));
                    }
                }
            }

            // Check sample files exist
            for velocity_layer in &zone.velocity_layers {
                for sample in &velocity_layer.samples {
                    let sample_path = instrument_dir.join(&sample.path);
                    if !sample_path.exists() {
                        errors.push(format!(
                            "Missing sample: {} (pitch {}, articulation '{}')",
                            sample.path, zone.pitch, articulation.id
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// ── Compile a single instrument ────────────────────────────────────────

fn compile_instrument(
    instrument_dir: &Path,
    output_dir: &Path,
    temp_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse source manifest
    let manifest_path = instrument_dir.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: SourceManifest = serde_json::from_str(&manifest_content)?;

    println!("  Compiling: {} ({})", manifest.meta.name, manifest.meta.id);

    // Integrity check
    if let Err(errors) = integrity_check(instrument_dir, &manifest) {
        for error in &errors {
            eprintln!("    ✗ {}", error);
        }
        return Err(format!(
            "Integrity check failed with {} error(s)",
            errors.len()
        )
        .into());
    }
    println!("    ✓ Integrity check passed");

    // ── Phase 1: Collect all unique samples ────────────────────────────

    // Maps source path → (blake3 hash)
    // Maps blake3 hash → (sample_id, flac_path, sample_rate, channels)
    let mut path_to_hash: BTreeMap<String, String> = BTreeMap::new();
    let mut hash_to_sample: BTreeMap<String, (u32, PathBuf, u32, u8)> = BTreeMap::new();
    let mut next_sample_id: u32 = 0;

    // Collect samples in deterministic order: articulation → zone (by pitch) → velocity layer
    let mut sorted_articulations: Vec<&SourceArticulation> = manifest.articulations.iter().collect();
    sorted_articulations.sort_by(|a, b| a.id.cmp(&b.id));

    for articulation in &sorted_articulations {
        let mut sorted_zones: Vec<&SourceZone> = articulation.zones.iter().collect();
        sorted_zones.sort_by_key(|z| z.pitch);

        for zone in sorted_zones {
            for velocity_layer in &zone.velocity_layers {
                for sample in &velocity_layer.samples {
                    let abs_path = instrument_dir.join(&sample.path);

                    // Hash for deduplication
                    let hash = hash_file(&abs_path)?;

                    if path_to_hash.contains_key(&sample.path) {
                        continue;
                    }
                    path_to_hash.insert(sample.path.clone(), hash.clone());

                    if hash_to_sample.contains_key(&hash) {
                        // Duplicate content — will share the same sample_id
                        continue;
                    }

                    // Probe audio metadata
                    let (sample_rate, channels) = probe_audio(&abs_path)?;

                    // Encode to FLAC
                    let stem = format!("{}_{}", manifest.meta.id, next_sample_id);
                    let flac_path = encode_to_flac(&abs_path, temp_dir, &stem)?;

                    hash_to_sample.insert(
                        hash.clone(),
                        (next_sample_id, flac_path, sample_rate, channels),
                    );
                    next_sample_id += 1;
                }
            }
        }
    }

    let total_samples = hash_to_sample.len();
    let total_refs = path_to_hash.len();
    if total_refs > total_samples {
        println!(
            "    ✓ Deduplicated: {} unique samples from {} references",
            total_samples, total_refs
        );
    } else {
        println!("    ✓ {} samples collected", total_samples);
    }

    // Build path → sample_id lookup
    let path_to_id: BTreeMap<String, u32> = path_to_hash
        .iter()
        .map(|(path, hash)| {
            let (id, _, _, _) = &hash_to_sample[hash];
            (path.clone(), *id)
        })
        .collect();

    // ── Phase 2: Build META chunk ──────────────────────────────────────

    let container_manifest = ContainerManifest {
        meta: ContainerMeta {
            id: manifest.meta.id.clone(),
            name: manifest.meta.name.clone(),
            author: manifest.meta.author.clone(),
            version: manifest.meta.version.clone(),
        },
        articulations: sorted_articulations
            .iter()
            .map(|art| {
                let mut sorted_zones: Vec<&SourceZone> = art.zones.iter().collect();
                sorted_zones.sort_by_key(|z| z.pitch);

                ContainerArticulation {
                    id: art.id.clone(),
                    name: art.name.clone(),
                    range: art.range,
                    zones: sorted_zones
                        .iter()
                        .map(|zone| ContainerZone {
                            pitch: zone.pitch,
                            velocity_layers: zone
                                .velocity_layers
                                .iter()
                                .map(|vl| ContainerVelocityLayer {
                                    range: vl.range,
                                    samples: vl
                                        .samples
                                        .iter()
                                        .map(|s| ContainerSampleRef {
                                            sample_id: path_to_id[&s.path],
                                        })
                                        .collect(),
                                })
                                .collect(),
                        })
                        .collect(),
                }
            })
            .collect(),
    };

    let meta_bytes = rmp_serde::to_vec(&container_manifest)?;
    println!("    ✓ META chunk: {} bytes", meta_bytes.len());

    // ── Phase 3: Build SIDX and SDAT ───────────────────────────────────

    // Sort samples by ID for deterministic output
    let mut sorted_samples: Vec<(u32, &PathBuf, u32, u8)> = hash_to_sample
        .values()
        .map(|(id, path, sr, ch)| (*id, path, *sr, *ch))
        .collect();
    sorted_samples.sort_by_key(|(id, _, _, _)| *id);

    // Read all FLAC data and build index entries
    let mut sdat_buffer: Vec<u8> = Vec::new();
    let mut sidx_entries: Vec<SampleIndexEntry> = Vec::new();

    for (sample_id, flac_path, sample_rate, channels) in &sorted_samples {
        let flac_data = fs::read(flac_path)?;
        let offset = sdat_buffer.len() as u64;
        let length = flac_data.len() as u64;

        sdat_buffer.extend_from_slice(&flac_data);

        sidx_entries.push(SampleIndexEntry {
            sample_id: *sample_id,
            offset,
            length,
            sample_rate: *sample_rate,
            channels: *channels,
            _reserved: [0u8; 7],
        });
    }

    println!(
        "    ✓ SDAT chunk: {} bytes ({} samples)",
        sdat_buffer.len(),
        sidx_entries.len()
    );

    // ── Phase 4: Read optional cover image ─────────────────────────────

    let cover_path = instrument_dir.join("cover.webp");
    let cover_data = if cover_path.exists() {
        let data = fs::read(&cover_path)?;
        println!("    ✓ COVER chunk: {} bytes", data.len());
        Some(data)
    } else {
        println!("    ⚠ No cover.webp found");
        None
    };

    // ── Phase 5: Assemble container ────────────────────────────────────

    let output_path = output_dir.join(format!("{}.scvi", manifest.meta.id));
    let mut out = File::create(&output_path)?;

    // Calculate chunk offsets
    let meta_offset = HEADER_SIZE;
    let meta_size = meta_bytes.len() as u64;

    let sidx_offset = meta_offset + meta_size;
    let sidx_size = (sidx_entries.len() * SIDX_ENTRY_SIZE) as u64;

    let sdat_offset = sidx_offset + sidx_size;
    let sdat_size = sdat_buffer.len() as u64;

    // Write HEADER (padded to HEADER_SIZE)
    let header = ContainerHeader {
        magic: *MAGIC,
        version: CONTAINER_VERSION,
        _reserved_h: [0u8; 2],
        meta_offset,
        meta_size,
        sidx_offset,
        sidx_size,
        sdat_offset,
        sdat_size,
    };

    // Safety: ContainerHeader is repr(C, packed) with no padding ambiguity
    let header_bytes = unsafe { struct_as_bytes(&header) };
    assert!(
        HEADER_STRUCT_SIZE <= HEADER_SIZE as usize,
        "Header struct exceeds reserved header space"
    );
    out.write_all(header_bytes)?;

    // Pad rest of header to HEADER_SIZE
    let header_padding = HEADER_SIZE as usize - HEADER_STRUCT_SIZE;
    if header_padding > 0 {
        out.write_all(&vec![0u8; header_padding])?;
    }

    // Write META chunk
    let mut current_offset = HEADER_SIZE;
    out.write_all(&meta_bytes)?;
    current_offset += meta_size;

    // Write SIDX chunk
    for entry in &sidx_entries {
        let entry_bytes = unsafe { struct_as_bytes(entry) };
        out.write_all(entry_bytes)?;
    }
    current_offset += sidx_size;

    // Write SDAT chunk
    out.write_all(&sdat_buffer)?;
    current_offset += sdat_size;

    // Write COVER chunk (appended directly after SDAT)
    if let Some(cover) = &cover_data {
        out.write_all(cover)?;
        current_offset += cover.len() as u64;
    }

    out.flush()?;

    let total_size = current_offset;
    println!(
        "    ✓ Written: {} ({:.2} MB)",
        output_path.display(),
        total_size as f64 / (1024.0 * 1024.0)
    );

    Ok(())
}

// ── Main entry point ───────────────────────────────────────────────────

fn main() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let instruments_dir = project_root.join("instruments");
    let output_dir = PathBuf::from("/Library/Audio/Plug-Ins/SCVI");
    let temp_dir = project_root.join("temp");

    // Ensure output and temp directories exist
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    println!("SCVI Binary Container Compiler v{}", CONTAINER_VERSION);
    println!("────────────────────────────────────────");
    println!("Source:  {}", instruments_dir.display());
    println!("Output:  {}", output_dir.display());
    println!();

    // Discover instruments
    let mut instruments: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&instruments_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("manifest.json").exists() {
                instruments.push(path);
            }
        }
    }

    instruments.sort();

    if instruments.is_empty() {
        eprintln!("No instruments found in {}", instruments_dir.display());
        std::process::exit(1);
    }

    println!("Found {} instrument(s)", instruments.len());
    println!();

    let mut success_count = 0;
    let mut fail_count = 0;

    for instrument_dir in &instruments {
        match compile_instrument(instrument_dir, &output_dir, &temp_dir) {
            Ok(()) => success_count += 1,
            Err(e) => {
                eprintln!(
                    "  ✗ Failed to compile {}: {}",
                    instrument_dir.display(),
                    e
                );
                fail_count += 1;
            }
        }
        println!();
    }

    // Clean up temp directory
    if let Err(e) = fs::remove_dir_all(&temp_dir) {
        eprintln!("Warning: Failed to clean temp directory: {}", e);
    }

    println!("────────────────────────────────────────");
    println!(
        "Done: {} succeeded, {} failed",
        success_count, fail_count
    );

    if fail_count > 0 {
        std::process::exit(1);
    }
}
