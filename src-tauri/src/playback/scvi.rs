/// SCVI Binary Container Reader
///
/// Reads .scvi binary container files produced by scvi_compress.
/// Provides zero-copy access to the META, SIDX, and SDAT chunks.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

// ── Constants (must match scvi_compress.rs) ────────────────────────────

const MAGIC: &[u8; 4] = b"SCVI";

// ── HEADER ─────────────────────────────────────────────────────────────

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct RawHeader {
    magic: [u8; 4],
    version: u16,
    _reserved_h: [u8; 2],
    meta_offset: u64,
    meta_size: u64,
    sidx_offset: u64,
    sidx_size: u64,
    sdat_offset: u64,
    sdat_size: u64,
}

const RAW_HEADER_SIZE: usize = std::mem::size_of::<RawHeader>();

// ── SIDX entry (must match scvi_compress.rs) ───────────────────────────

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct RawSampleIndexEntry {
    sample_id: u32,
    offset: u64,
    length: u64,
    sample_rate: u32,
    channels: u8,
    _reserved: [u8; 7],
}

const SIDX_ENTRY_SIZE: usize = std::mem::size_of::<RawSampleIndexEntry>();

// ── META structures (deserialized from MessagePack) ────────────────────

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InstrumentType {
    Pitched,
    Percussion,
}

#[derive(Deserialize)]
pub struct ContainerMeta {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub r#type: InstrumentType,
}

#[derive(Deserialize)]
pub struct ContainerSampleRef {
    pub sample_id: u32,
}

#[derive(Deserialize)]
pub struct ContainerVelocityLayer {
    pub range: [u8; 2],
    pub samples: Vec<ContainerSampleRef>,
}

#[derive(Deserialize)]
pub struct ContainerZone {
    pub pitch: u8,
    pub velocity_layers: Vec<ContainerVelocityLayer>,
}

#[derive(Deserialize)]
pub struct ContainerArticulation {
    pub id: String,
    pub name: String,
    pub range: Vec<[u8; 2]>,
    pub zones: Vec<ContainerZone>,
}

#[derive(Deserialize)]
pub struct ContainerManifest {
    pub meta: ContainerMeta,
    pub articulations: Vec<ContainerArticulation>,
}

// ── Parsed sample index entry ──────────────────────────────────────────

#[derive(Clone)]
pub struct SampleIndex {
    pub sample_id: u32,
    /// Absolute byte offset within the file (sdat_offset + entry.offset)
    pub file_offset: u64,
    /// Byte length of the Vorbis data
    pub length: u64,
    pub sample_rate: u32,
    pub channels: u8,
}

// ── Container handle ───────────────────────────────────────────────────

pub struct ScviContainer {
    pub manifest: ContainerManifest,
    pub samples: Vec<SampleIndex>,
    /// Absolute byte offset of SDAT chunk start within the file
    sdat_offset: u64,
    /// Path to the .scvi file (needed to re-open for sample data reads)
    path: std::path::PathBuf,
    /// Offset and size of cover image, if present
    cover_region: Option<(u64, u64)>,
}

impl ScviContainer {
    /// Open and parse an SCVI container.
    /// Reads HEADER, META, and SIDX. Does NOT read sample data (SDAT).
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;

        // ── Read HEADER ────────────────────────────────────────────────
        let mut header_buf = [0u8; RAW_HEADER_SIZE];
        file.read_exact(&mut header_buf)?;

        // Safety: RawHeader is repr(C, packed) with known layout
        let header: RawHeader =
            unsafe { std::ptr::read_unaligned(header_buf.as_ptr() as *const RawHeader) };

        if &header.magic != MAGIC {
            return Err(format!(
                "Invalid SCVI magic: expected {:?}, got {:?}",
                MAGIC, &header.magic
            )
            .into());
        }

        let version = header.version;
        if version != 1 {
            return Err(format!("Unsupported SCVI version: {}", version).into());
        }

        // ── Read META chunk ────────────────────────────────────────────
        file.seek(SeekFrom::Start(header.meta_offset))?;
        let mut meta_buf = vec![0u8; header.meta_size as usize];
        file.read_exact(&mut meta_buf)?;

        let manifest: ContainerManifest = rmp_serde::from_slice(&meta_buf)?;

        // ── Read SIDX chunk ────────────────────────────────────────────
        file.seek(SeekFrom::Start(header.sidx_offset))?;
        let entry_count = header.sidx_size as usize / SIDX_ENTRY_SIZE;
        let mut sidx_buf = vec![0u8; header.sidx_size as usize];
        file.read_exact(&mut sidx_buf)?;

        let mut samples = Vec::with_capacity(entry_count);
        for i in 0..entry_count {
            let offset = i * SIDX_ENTRY_SIZE;
            let entry: RawSampleIndexEntry = unsafe {
                std::ptr::read_unaligned(sidx_buf[offset..].as_ptr() as *const RawSampleIndexEntry)
            };

            samples.push(SampleIndex {
                sample_id: entry.sample_id,
                file_offset: header.sdat_offset + entry.offset,
                length: entry.length,
                sample_rate: entry.sample_rate,
                channels: entry.channels,
            });
        }

        // ── Detect COVER region ────────────────────────────────────────
        let sdat_end = header.sdat_offset + header.sdat_size;
        let file_size = file.seek(SeekFrom::End(0))?;

        let cover_region = if sdat_end < file_size {
            Some((sdat_end, file_size - sdat_end))
        } else {
            None
        };

        Ok(ScviContainer {
            manifest,
            samples,
            sdat_offset: header.sdat_offset,
            path: path.to_path_buf(),
            cover_region,
        })
    }

    /// Get the sample index entry for a given sample ID
    pub fn get_sample_index(&self, sample_id: u32) -> Option<&SampleIndex> {
        self.samples.iter().find(|s| s.sample_id == sample_id)
    }

    /// Read the cover image bytes, if present
    pub fn read_cover(&self) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        match self.cover_region {
            Some((offset, size)) => {
                let mut file = File::open(&self.path)?;
                file.seek(SeekFrom::Start(offset))?;
                let mut buf = vec![0u8; size as usize];
                file.read_exact(&mut buf)?;
                Ok(Some(buf))
            }
            None => Ok(None),
        }
    }
}
