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

    if args.len() > 1 {
        // Process specific instrument
        let instrument_name = &args[1];
        create_scvi_archive(instrument_name)?;
    } else {
        // Process all instruments
        process_all_instruments()?;
    }

    Ok(())
}

fn create_scvi_archive(instrument_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = format!("instruments/src/{}", instrument_name);
    let temp_path = format!("instruments/temp/{}", instrument_name);
    let output_dir = "/Library/Audio/Plug-Ins/SCVI";
    let output_path = format!("{}/{}.scvi", output_dir, instrument_name);

    // Ensure we're running from the correct directory (project root)
    let current_dir = env::current_dir()?;
    if !current_dir.join("package.json").exists() {
        eprintln!("Error: This binary must be run from the project root directory");
        std::process::exit(1);
    }

    println!("🔧 Processing instrument: {}", instrument_name);
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
                // Convert WAV to WavPack with maximum compression
                let wv_path = temp_file.with_extension("wv");
                if let Some(parent) = wv_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                println!("🎵 Converting {} to WavPack...", path.display());

                let wv_result = Command::new("wavpack")
                    .args([
                        "-hh", // High compression mode
                        "-y",  // Yes to overwrite
                        &path.to_string_lossy(),
                        "-o",
                        &wv_path.to_string_lossy(),
                    ])
                    .status();

                match wv_result {
                    Ok(status) if status.success() => {
                        // Check if WavPack is actually smaller
                        let wv_size = fs::metadata(&wv_path)?.len();
                        if wv_size < original_size {
                            println!(
                                "✅ Compressed: {} bytes → {} bytes ({:.1}% reduction)",
                                original_size,
                                wv_size,
                                (1.0 - wv_size as f64 / original_size as f64) * 100.0
                            );
                        } else {
                            // WavPack is not smaller, keep original
                            fs::copy(path, &temp_file)?;
                            fs::remove_file(&wv_path)?;
                            println!(
                                "⚠️  Kept original: WavPack was larger ({} bytes)",
                                original_size
                            );
                        }
                    }
                    _ => {
                        // WavPack conversion failed, copy original
                        fs::copy(path, &temp_file)?;
                        println!("❌ WavPack failed, kept original: {} bytes", original_size);
                    }
                }
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
        .status();

    if !archive_result?.success() {
        return Err("7z archive creation failed".into());
    }

    // Get final file size
    let final_size = fs::metadata(&output_path)?.len();

    println!("\n🎉 Success!");
    println!("📦 Created: {}.scvi", instrument_name);
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

    // Process each instrument
    for instrument in instruments {
        println!("\n🎵 Processing instrument: {}", instrument);
        if let Err(e) = create_scvi_archive(&instrument) {
            eprintln!("❌ Failed to process {}: {}", instrument, e);
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
