/// SCVI Container Inspector
///
/// Usage: scvi-inspect <file.scvi>
///
/// Dumps the full structure of an SCVI binary container: header layout,
/// sample index, and articulation mappings.

#[path = "../playback/scvi.rs"]
mod scvi;

use scvi::ScviContainer;
use std::path::Path;

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn inspect(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let container = ScviContainer::open(path)?;
    let m = &container.manifest.meta;
    let file_size = std::fs::metadata(path)?.len();

    // ── Header ─────────────────────────────────────────────────────────

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  SCVI Container Inspection                                  ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  File:   {:<51} ║", path.display());
    println!("║  ID:     {:<51} ║", m.id);
    println!("║  Name:   {:<51} ║", m.name);
    println!("║  Author: {:<51} ║", m.author);
    println!("║  Ver:    {:<51} ║", m.version);
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ── File layout (re-read raw header for offsets) ───────────────────

    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut hdr = [0u8; 64];
    f.read_exact(&mut hdr)?;

    let meta_off = u64::from_le_bytes(hdr[8..16].try_into()?);
    let meta_sz = u64::from_le_bytes(hdr[16..24].try_into()?);
    let sidx_off = u64::from_le_bytes(hdr[24..32].try_into()?);
    let sidx_sz = u64::from_le_bytes(hdr[32..40].try_into()?);
    let sdat_off = u64::from_le_bytes(hdr[40..48].try_into()?);
    let sdat_sz = u64::from_le_bytes(hdr[48..56].try_into()?);
    let sdat_end = sdat_off + sdat_sz;

    println!("── File Layout ({}) ──", format_bytes(file_size));
    println!("  HEADER:  offset=0            size=64 B");
    println!(
        "  META:    offset={:<12} size={}",
        meta_off,
        format_bytes(meta_sz)
    );
    println!(
        "  SIDX:    offset={:<12} size={} ({} entries)",
        sidx_off,
        format_bytes(sidx_sz),
        sidx_sz / 32
    );
    println!(
        "  SDAT:    offset={:<12} size={}",
        sdat_off,
        format_bytes(sdat_sz)
    );
    if sdat_end < file_size {
        println!(
            "  COVER:   offset={:<12} size={}",
            sdat_end,
            format_bytes(file_size - sdat_end)
        );
    } else {
        println!("  COVER:   (none)");
    }
    println!();

    // ── Sample index ───────────────────────────────────────────────────

    let mut unique_ids: Vec<u32> = container.samples.iter().map(|s| s.sample_id).collect();
    unique_ids.sort();
    unique_ids.dedup();

    println!(
        "── Sample Index ({} entries, {} unique) ──",
        container.samples.len(),
        unique_ids.len()
    );
    println!(
        "  {:>6}  {:>12}  {:>10}  {:>6}  {:>4}",
        "ID", "Offset", "Size", "Rate", "Ch"
    );
    println!(
        "  {:─>6}  {:─>12}  {:─>10}  {:─>6}  {:─>4}",
        "", "", "", "", ""
    );
    for s in &container.samples {
        println!(
            "  {:>6}  {:>12}  {:>10}  {:>6}  {:>4}",
            s.sample_id,
            s.file_offset,
            format_bytes(s.length),
            s.sample_rate,
            s.channels
        );
    }
    println!();

    // ── Articulations ──────────────────────────────────────────────────

    println!(
        "── Articulations ({}) ──",
        container.manifest.articulations.len()
    );

    for art in &container.manifest.articulations {
        let total_samples: usize = art
            .zones
            .iter()
            .flat_map(|z| &z.velocity_layers)
            .map(|vl| vl.samples.len())
            .sum();

        println!();
        println!("  {} (\"{}\")", art.name, art.id);
        let range_str = art
            .range
            .iter()
            .map(|&[low, high]| format!("{}-{}", low, high))
            .collect::<Vec<_>>()
            .join(", ");
        println!("  Range: MIDI {}", range_str);
        println!("  Zones: {}  Samples: {}", art.zones.len(), total_samples);
        println!();
        println!("    {:>5}  {:>12}  {:>8}", "Pitch", "Vel Range", "Samples");
        println!("    {:─>5}  {:─>12}  {:─>8}", "", "", "");

        for zone in &art.zones {
            for vl in &zone.velocity_layers {
                let ids: Vec<String> = vl.samples.iter().map(|s| s.sample_id.to_string()).collect();
                println!(
                    "    {:>5}  {:>5}–{:<5}  {}",
                    zone.pitch,
                    vl.range[0],
                    vl.range[1],
                    ids.join(", ")
                );
            }
        }
    }

    println!();
    println!("✓ Container is valid.");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: scvi-inspect <file.scvi>");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    if let Err(e) = inspect(path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
