use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Ensure we're running from the project root directory
    let current_dir = env::current_dir()?;
    if !current_dir.join("package.json").exists() {
        eprintln!("Error: This binary must be run from the project root directory (where package.json is located)");
        std::process::exit(1);
    }

    // Parse arguments - only instrument name now
    let instrument_name = if args.len() > 1 {
        Some(args[1].to_string())
    } else {
        None
    };

    match instrument_name {
        Some(name) => {
            // Process specific instrument - create both versions
            create_scvi_archive(&name, &name, "wavpack")?;
            create_scvi_archive(&name, &format!("{} (Lite)", name), "mp3")?;
        }
        None => {
            // Process all instruments - create both versions for each
            process_all_instruments()?;
        }
    }

    Ok(())
}

fn create_scvi_archive(
    src_instrument_name: &str,
    output_instrument_name: &str,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = format!("instruments/src/{}", src_instrument_name);
    let temp_path = format!("instruments/temp/{}", output_instrument_name);
    let output_dir = "/Library/Audio/Plug-Ins/SCVI";
    let output_path = format!("{}/{}.scvi", output_dir, output_instrument_name);

    // Ensure we're running from the correct directory (project root)
    let current_dir = env::current_dir()?;
    if !current_dir.join("package.json").exists() {
        eprintln!("Error: This binary must be run from the project root directory");
        std::process::exit(1);
    }

    println!(
        "🔧 Processing instrument: {} (format: {})",
        output_instrument_name, format
    );
    println!("📁 Source: {}", src_path);
    println!("📦 Output: {}", output_path);

    // Create output directory (may require admin permissions)
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory {}: {}", output_dir, e))?;

    // Create temp directory
    fs::create_dir_all(&temp_path)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Use a scope guard to ensure temp cleanup even on errors
    struct TempDirGuard {
        path: String,
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            if let Err(e) = fs::remove_dir_all(&self.path) {
                eprintln!(
                    "Warning: Failed to cleanup temp directory {}: {}",
                    self.path, e
                );
            }
        }
    }

    let _temp_guard = TempDirGuard {
        path: temp_path.clone(),
    };

    let mut total_original_size = 0u64;
    let mut files_processed = 0;

    // Process all files
    for entry in WalkDir::new(&src_path) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(&src_path)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        let temp_file = Path::new(&temp_path).join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&temp_file).map_err(|e| {
                format!("Failed to create directory {}: {}", temp_file.display(), e)
            })?;
        } else {
            let original_size = fs::metadata(path)?.len();
            total_original_size += original_size;

            if path.extension().and_then(|s| s.to_str()) == Some("wav") {
                match format {
                    "wavpack" => compress_wavpack(path, &temp_file, original_size)?,
                    "mp3" => compress_mp3(path, &temp_file, original_size)?,
                    _ => {
                        eprintln!("Unknown format: {}", format);
                        fs::copy(path, &temp_file)?;
                    }
                }
            } else if path.file_name().and_then(|s| s.to_str()) == Some("meta.json") {
                // Special handling for meta.json: add format field and minify
                process_meta_json(path, &temp_file, format)?;
            } else {
                // Copy other files as-is
                fs::copy(path, &temp_file)?;
                println!("📄 Copied: {} ({} bytes)", path.display(), original_size);
            }

            files_processed += 1;
        }
    }

    println!("\n📊 Pre-compression summary:");
    println!("   Files processed: {}", files_processed);
    println!(
        "   Total size: {} bytes ({:.2} MB)",
        total_original_size,
        total_original_size as f64 / 1_000_000.0
    );

    // Create 7z archive with maximum compression
    println!("\n🗜️  Creating 7z archive with maximum compression...");

    // Remove existing archive if it exists
    if Path::new(&output_path).exists() {
        fs::remove_file(&output_path)
            .map_err(|e| format!("Failed to remove existing archive: {}", e))?;
    }

    // Change to temp directory and archive contents at root level
    let archive_result = Command::new("7zz")
        .current_dir(&temp_path)
        .args([
            "a",         // Add to archive
            "-t7z",      // 7z format
            "-mx=9",     // Maximum compression
            "-m0=LZMA2", // LZMA2 algorithm
            "-ms=8m",    // 8MB dictionary size for better compression
            "-mmt=on",   // Multi-threading
            "-mfb=273",  // Fast bytes for LZMA
            "-md=256m",  // Match finder cycles
            &output_path,
            "*", // Archive all files/folders in current directory (temp_path)
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if !archive_result?.success() {
        return Err("7z archive creation failed".into());
    }

    // Get final file size
    let final_size = fs::metadata(&output_path)?.len();

    println!("\n🎉 Success!");
    println!("📦 Created: {}.scvi", output_instrument_name);
    println!(
        "📊 Final size: {} bytes ({:.2} MB)",
        final_size,
        final_size as f64 / 1_000_000.0
    );
    println!(
        "🗜️  Compression ratio: {:.2}% (from {:.2} MB to {:.2} MB)",
        (1.0 - final_size as f64 / total_original_size as f64) * 100.0,
        total_original_size as f64 / 1_000_000.0,
        final_size as f64 / 1_000_000.0
    );

    Ok(())
}

fn process_meta_json(
    src_path: &Path,
    dest_path: &Path,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the original meta.json
    let content = fs::read_to_string(src_path)?;
    let mut json_value: Value = serde_json::from_str(&content)?;

    // Add or update the format field
    if let Some(meta) = json_value.get_mut("meta") {
        if let Some(meta_obj) = meta.as_object_mut() {
            meta_obj.insert("format".to_string(), json!(format));

            // For lite version, append (Lite) to the instrument name
            if format == "mp3" {
                if let Some(name) = meta_obj.get_mut("name") {
                    if let Some(name_str) = name.as_str() {
                        *name = json!(format!("{} (Lite)", name_str));
                    }
                }
            }
        }
    } else {
        // If no meta object exists, create one
        json_value["meta"] = json!({
            "format": format
        });
    }

    // Minify the JSON (remove whitespace)
    let minified = serde_json::to_string(&json_value)?;
    let minified_len = minified.len();

    // Write the minified version
    fs::write(dest_path, minified)?;

    println!(
        "📄 Processed meta.json: added format='{}'{}minified from {} to {} bytes",
        format,
        if format == "mp3" {
            ", updated name with '(Lite)'"
        } else {
            ""
        },
        content.len(),
        minified_len
    );

    Ok(())
}

fn compress_wavpack(
    path: &Path,
    temp_file: &Path,
    original_size: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert WAV to WavPack with maximum compression
    let wv_path = temp_file.with_extension("wv");
    if let Some(parent) = wv_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let wv_result = Command::new("wavpack")
        .args([
            "-hh", // High compression mode
            "-y",  // Yes to overwrite
            &path.to_string_lossy(),
            "-o",
            &wv_path.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match wv_result {
        Ok(status) if status.success() => {
            // Check if WavPack is actually smaller
            let wv_size = fs::metadata(&wv_path)?.len();
            if wv_size < original_size {
                println!(
                    "🎵 {} → WavPack: {} → {} bytes ({:.1}% reduction)",
                    path.display(),
                    original_size,
                    wv_size,
                    (1.0 - wv_size as f64 / original_size as f64) * 100.0
                );
            } else {
                // WavPack is not smaller, keep original
                fs::copy(path, temp_file)?;
                fs::remove_file(&wv_path)?;
                println!(
                    "⚠️  Kept original: WavPack was larger ({} bytes)",
                    original_size
                );
            }
        }
        _ => {
            // WavPack conversion failed, copy original
            fs::copy(path, temp_file)?;
            println!("❌ WavPack failed, kept original: {} bytes", original_size);
        }
    }

    Ok(())
}

fn compress_mp3(
    path: &Path,
    temp_file: &Path,
    original_size: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert WAV to MP3 with high quality
    let mp3_path = temp_file.with_extension("mp3");
    if let Some(parent) = mp3_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mp3_result = Command::new("ffmpeg")
        .args([
            "-i",
            &path.to_string_lossy(),
            "-codec:a",
            "libmp3lame",
            "-q:a",
            "0",  // Highest quality VBR (~245kbps)
            "-y", // Overwrite output
            &mp3_path.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match mp3_result {
        Ok(status) if status.success() => {
            // Check if MP3 is actually smaller
            let mp3_size = fs::metadata(&mp3_path)?.len();
            if mp3_size < original_size {
                println!(
                    "🎵 {} → MP3: {} → {} bytes ({:.1}% reduction)",
                    path.display(),
                    original_size,
                    mp3_size,
                    (1.0 - mp3_size as f64 / original_size as f64) * 100.0
                );
            } else {
                // MP3 is not smaller, keep original
                fs::copy(path, temp_file)?;
                fs::remove_file(&mp3_path)?;
                println!(
                    "⚠️  Kept original: MP3 was larger ({} bytes)",
                    original_size
                );
            }
        }
        _ => {
            // MP3 conversion failed, copy original
            fs::copy(path, temp_file)?;
            println!("❌ MP3 failed, kept original: {} bytes", original_size);
        }
    }

    Ok(())
}

fn process_all_instruments() -> Result<(), Box<dyn std::error::Error>> {
    let instruments_dir = Path::new("instruments/src");

    if !instruments_dir.exists() {
        eprintln!("Error: instruments/src directory not found");
        std::process::exit(1);
    }

    // Get all instrument folders
    let instruments: Vec<String> = fs::read_dir(instruments_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
        .collect();

    if instruments.is_empty() {
        println!("No instruments found in instruments/src/");
        return Ok(());
    }

    println!(
        "🔧 Found {} instruments: {}",
        instruments.len(),
        instruments.join(", ")
    );

    // Process each instrument - create both versions
    for instrument in instruments {
        println!("\n🎵 Processing instrument: {}", instrument);
        if let Err(e) = create_scvi_archive(&instrument, &instrument, "wavpack") {
            eprintln!("❌ Failed to process {}: {}", instrument, e);
            // Continue with other instruments
        }
        if let Err(e) = create_scvi_archive(&instrument, &format!("{} (Lite)", instrument), "mp3") {
            eprintln!("❌ Failed to process {} (Lite): {}", instrument, e);
            // Continue with other instruments
        }
    }

    // Clean up temp directory if it's empty
    let temp_dir = Path::new("instruments/temp");
    if temp_dir.exists() {
        if let Ok(entries) = fs::read_dir(temp_dir) {
            if entries.count() == 0 {
                if let Err(e) = fs::remove_dir(temp_dir) {
                    eprintln!("Warning: Failed to remove empty temp directory: {}", e);
                }
            }
        }
    }

    println!("\n🎉 All instruments processed!");
    Ok(())
}
