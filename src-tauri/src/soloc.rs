// These items are all used by the lib; bins that include this file via #[path]
// only use a subset, so suppress dead_code warnings for standalone compilation.
#![allow(dead_code)]

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::io::{Cursor, Read, Write};

pub const SOLOC_MAGIC: &[u8; 5] = b"SOLOC";
pub const SOLOC_CONTAINER_VERSION: u8 = 1;
pub const SOLOC_PROJECT_JSON_CHUNK_TYPE: u32 = u32::from_le_bytes(*b"JSON");
pub const SOLOC_CHUNK_FLAG_GZIP: u16 = 1 << 0;
pub const SOLOC_SUPPORTED_CHUNK_FLAGS: u16 = SOLOC_CHUNK_FLAG_GZIP;
pub const SOLOC_RESERVED_CHUNK_FLAGS_MASK: u16 = !SOLOC_SUPPORTED_CHUNK_FLAGS;
pub const SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE: usize = 4 + 2 + 8 + 8;

pub fn compress_project_payload(json_data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(json_data)
        .map_err(|e| format!("Failed to compress project payload: {}", e))?;

    let compressed_data = encoder
        .finish()
        .map_err(|e| format!("Failed to finish compression: {}", e))?;

    Ok(compressed_data)
}

pub fn decompress_project_payload(payload: &[u8], flags: u16) -> Result<Vec<u8>, String> {
    if flags & SOLOC_CHUNK_FLAG_GZIP == 0 {
        return Err(
            "Failed to decompress project payload: PROJECT_JSON chunk is not gzip-compressed"
                .to_string(),
        );
    }

    let mut decoder = GzDecoder::new(Cursor::new(payload));
    let mut json_data = Vec::new();
    decoder
        .read_to_end(&mut json_data)
        .map_err(|e| format!("Failed to decompress project payload: {}", e))?;

    Ok(json_data)
}

pub fn build_soloc_container(app_version: &str, project_payload: &[u8]) -> Result<Vec<u8>, String> {
    let app_version_bytes = app_version.as_bytes();
    let app_version_len = u8::try_from(app_version_bytes.len())
        .map_err(|_| "App version is too long for .soloc header".to_string())?;
    let chunk_count = 1_u16;
    let header_len = SOLOC_MAGIC.len() + 1 + 1 + app_version_bytes.len() + 2;
    let directory_len = SOLOC_CHUNK_DIRECTORY_ENTRY_SIZE;
    let payload_offset = header_len
        .checked_add(directory_len)
        .ok_or("Failed to build .soloc container: payload offset overflow".to_string())?;
    let payload_offset = u64::try_from(payload_offset)
        .map_err(|_| "Failed to build .soloc container: payload offset overflow".to_string())?;
    let payload_length = u64::try_from(project_payload.len())
        .map_err(|_| "Failed to build .soloc container: payload length overflow".to_string())?;
    let total_len = usize::try_from(payload_offset)
        .ok()
        .and_then(|offset| offset.checked_add(project_payload.len()))
        .ok_or("Failed to build .soloc container: file size overflow".to_string())?;

    let mut container = Vec::with_capacity(total_len);
    container.extend_from_slice(SOLOC_MAGIC);
    container.push(SOLOC_CONTAINER_VERSION);
    container.push(app_version_len);
    container.extend_from_slice(app_version_bytes);
    container.extend_from_slice(&chunk_count.to_le_bytes());
    container.extend_from_slice(&SOLOC_PROJECT_JSON_CHUNK_TYPE.to_le_bytes());
    container.extend_from_slice(&SOLOC_CHUNK_FLAG_GZIP.to_le_bytes());
    container.extend_from_slice(&payload_offset.to_le_bytes());
    container.extend_from_slice(&payload_length.to_le_bytes());
    container.extend_from_slice(project_payload);

    Ok(container)
}
