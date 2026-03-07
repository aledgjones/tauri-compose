/// Fixture Builder for .soloc Visual Regression Files
///
/// Recursively reads JSON project payloads from a source directory tree and
/// wraps each one into a valid .soloc binary container, mirroring the
/// directory structure into the output directory.
///
/// Usage: build-fixtures <source-dir> <output-dir>
///
/// Layout:
///   source/time-signatures/time-signatures.json
///   built/time-signatures/time-signatures.soloc
///
/// Each .json file must contain a valid project payload with
/// `score.meta.app_version` set. That value is used as both the container
/// header version and the payload version.
///
/// To create a fixture JSON:
///   1. Save a project in the app (produces a .soloc file)
///   2. Extract the JSON: zcat <file.soloc> | python3 -m json.tool > fixture.json
///   3. Edit as needed and commit to tests/fixtures/source/<concern>/
///   4. Run: npm run build:fixtures  — regenerates tests/fixtures/built/
#[path = "../soloc.rs"]
mod soloc;

use soloc::{build_soloc_container, compress_project_payload};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_json_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.path());
    for entry in sorted {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files);
        } else if path.extension().map(|e| e == "json").unwrap_or(false) {
            files.push(path);
        }
    }
}

fn build_fixture(json_path: &Path, source_root: &Path, out_root: &Path) -> Result<(), String> {
    let json_bytes = fs::read(json_path)
        .map_err(|e| format!("Failed to read {}: {}", json_path.display(), e))?;

    let json_value: serde_json::Value = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("Failed to parse {}: {}", json_path.display(), e))?;

    let app_version = json_value
        .get("score")
        .and_then(|s| s.get("meta"))
        .and_then(|m| m.get("app_version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing score.meta.app_version in {}", json_path.display()))?;

    let compressed = compress_project_payload(&json_bytes)?;
    let container = build_soloc_container(app_version, &compressed)?;

    // Mirror the relative path from source root into output root
    let rel = json_path
        .strip_prefix(source_root)
        .map_err(|_| "Path not under source root".to_string())?;
    let out_path = out_root.join(rel).with_extension("soloc");

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output dir {}: {}", parent.display(), e))?;
    }

    fs::write(&out_path, &container)
        .map_err(|e| format!("Failed to write {}: {}", out_path.display(), e))?;

    let rel_display = rel.display();
    let out_rel = out_path
        .strip_prefix(out_root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| out_path.display().to_string());

    println!("  ✓  {}  →  {}", rel_display, out_rel);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: build-fixtures <source-dir> <output-dir>");
        std::process::exit(1);
    }

    let source_dir = Path::new(&args[1]);
    let out_dir = Path::new(&args[2]);

    if !source_dir.is_dir() {
        eprintln!(
            "Error: source directory does not exist: {}",
            source_dir.display()
        );
        std::process::exit(1);
    }

    fs::create_dir_all(out_dir).unwrap_or_else(|e| {
        eprintln!("Error creating output directory: {}", e);
        std::process::exit(1);
    });

    let mut files: Vec<PathBuf> = Vec::new();
    collect_json_files(source_dir, &mut files);

    if files.is_empty() {
        println!("No .json files found under {}", source_dir.display());
        return;
    }

    println!("Building {} fixture(s)...", files.len());
    let mut errors = 0;

    for path in &files {
        if let Err(e) = build_fixture(path, source_dir, out_dir) {
            eprintln!("  ✗  {}", e);
            errors += 1;
        }
    }

    if errors > 0 {
        eprintln!("\n{} fixture(s) failed.", errors);
        std::process::exit(1);
    } else {
        println!(
            "\nDone — {} fixture(s) written to {}",
            files.len(),
            out_dir.display()
        );
    }
}
