use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::process;

// ── Manifest structures ────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Debug)]
struct Meta {
    id: String,
    name: String,
    author: String,
    version: String,
    r#type: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Sample {
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct VelocityLayer {
    range: [u8; 2],
    samples: Vec<Sample>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Zone {
    pitch: u8,
    #[serde(rename = "velocityLayers")]
    velocity_layers: Vec<VelocityLayer>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Articulation {
    id: String,
    name: String,
    range: Vec<Vec<u8>>,
    zones: Vec<Zone>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Manifest {
    meta: Meta,
    articulations: Vec<Articulation>,
}

// ── Sample parsing structures ──────────────────────────────────────────

#[derive(Debug)]
struct ParsedSample {
    articulation: String,
    dynamic: String,
    pitch: String,
    midi_note: u8,
    round_robin: Option<u8>,
    filename: String,
}

// ── MIDI note mapping ──────────────────────────────────────────────────

fn pitch_to_midi(pitch_str: &str) -> Option<u8> {
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let re = regex::Regex::new(r"^([A-G]#?)(\d+)$").unwrap();
    if let Some(caps) = re.captures(pitch_str) {
        let note = caps.get(1)?.as_str();
        let octave: i8 = caps.get(2)?.as_str().parse().ok()?;
        let note_index = note_names.iter().position(|&n| n == note)?;
        Some(((octave + 1) as u8) * 12 + (note_index as u8))
    } else {
        None
    }
}

// ── Articulation name formatting ───────────────────────────────────────

fn format_articulation_id(articulation: &str) -> String {
    articulation
        .to_lowercase()
        .replace(" ", "-")
        .replace("_", "-")
}

fn format_articulation_name(articulation: &str) -> String {
    articulation
        .split(&[' ', '_'][..])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// ── Main logic ─────────────────────────────────────────────────────────

fn parse_sample_filename(filename: &str, _instrument: &str) -> Option<ParsedSample> {
    // Remove .opus extension
    let filename_no_ext = filename.strip_suffix(".opus")?;

    // Pattern: [identifier]_[articulation]_[dynamic]_[pitch]_[optional RR number]
    // We ignore the identifier (first part) since we're already in the correct folder
    let parts: Vec<&str> = filename_no_ext.split('_').collect();
    if parts.len() < 4 {
        return None;
    }

    let last_part = *parts.last()?;
    let round_robin: Option<u8>;

    // Extract round robin if present
    let pitch_part;
    if last_part.starts_with("RR") {
        let rr_str = last_part.strip_prefix("RR")?;
        round_robin = Some(rr_str.parse().ok()?);
        pitch_part = *parts.get(parts.len() - 2)?;
    } else {
        round_robin = None;
        pitch_part = last_part;
    };

    // Extract dynamic (second to last, or third to last if RR present)
    let dynamic_idx = if round_robin.is_some() {
        parts.len() - 3
    } else {
        parts.len() - 2
    };
    let dynamic = *parts.get(dynamic_idx)?;

    // Extract articulation (everything from start up to dynamic)
    let articulation = parts[1..dynamic_idx].join("_");

    // Convert pitch to MIDI
    let midi_note = pitch_to_midi(pitch_part)?;

    Some(ParsedSample {
        articulation,
        dynamic: dynamic.to_string(),
        pitch: pitch_part.to_string(),
        midi_note,
        round_robin,
        filename: filename.to_string(),
    })
}

fn generate_velocity_layers(samples: &[&ParsedSample]) -> Vec<VelocityLayer> {
    // Group by dynamic
    let mut dynamics: BTreeMap<String, Vec<&ParsedSample>> = BTreeMap::new();
    for sample in samples {
        dynamics
            .entry(sample.dynamic.clone())
            .or_default()
            .push(sample);
    }

    // Sort dynamics by some logical order (you might want to customize this)
    let dynamic_order = ["PP", "P", "MP", "MF", "F", "FF"];
    let mut sorted_dynamics: Vec<String> = dynamics.keys().cloned().collect();
    sorted_dynamics.sort_by_key(|d| dynamic_order.iter().position(|&x| x == d).unwrap_or(999));

    // Create velocity ranges that span 1-127
    let num_dynamics = sorted_dynamics.len();
    let mut layers = Vec::new();

    for (i, dynamic) in sorted_dynamics.into_iter().enumerate() {
        let samples_for_dynamic = &dynamics[&dynamic];

        // Calculate velocity range
        let start_vel = if i == 0 {
            1
        } else {
            (i * 127 / num_dynamics) as u8 + 1
        };
        let end_vel = if i == num_dynamics - 1 {
            127
        } else {
            ((i + 1) * 127 / num_dynamics) as u8
        };

        // Group samples by round robin
        let mut rr_groups: BTreeMap<Option<u8>, Vec<&ParsedSample>> = BTreeMap::new();
        for sample in samples_for_dynamic {
            rr_groups
                .entry(sample.round_robin)
                .or_default()
                .push(sample);
        }

        let sample_refs: Vec<Sample> = rr_groups
            .into_iter()
            .flat_map(|(_, group)| {
                group.into_iter().map(|s| Sample {
                    path: format!("samples/{}", s.filename),
                })
            })
            .collect();

        layers.push(VelocityLayer {
            range: [start_vel, end_vel],
            samples: sample_refs,
        });
    }

    layers
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <instrument>", args[0]);
        process::exit(1);
    }

    let cwd = env::current_dir().unwrap();

    let instrument = &args[1];
    let instrument_path = cwd.join("instruments").join(instrument);
    let manifest_path = instrument_path.join("manifest.json");
    let samples_path = instrument_path.join("samples");

    // Read existing manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|_| panic!("Failed to read manifest.json for {}", instrument));

    let mut manifest: Manifest = serde_json::from_str(&manifest_content)
        .unwrap_or_else(|_| panic!("Failed to parse manifest.json for {}", instrument));

    // Clear articulations
    manifest.articulations.clear();

    // Read samples
    let samples_dir = fs::read_dir(&samples_path)
        .unwrap_or_else(|_| panic!("Failed to read samples directory for {}", instrument));

    let mut parsed_samples = Vec::new();
    for entry in samples_dir {
        let entry = entry.unwrap();
        let filename = entry.file_name().to_string_lossy().to_string();
        if let Some(parsed) = parse_sample_filename(&filename, instrument) {
            parsed_samples.push(parsed);
        }
    }

    // Group by articulation
    let mut articulations: BTreeMap<String, Vec<ParsedSample>> = BTreeMap::new();
    for sample in parsed_samples {
        articulations
            .entry(sample.articulation.clone())
            .or_default()
            .push(sample);
    }

    // Define articulation order (easy to update)
    let articulation_order = vec![
        "sustain",
        "espressivo",
        "nonvib",
        "marcato",
        "staccato",
        "staccatissimo",
        "tremolo",
        "trill_half",
        "trill_whole",
        "sul_ponticello",
        "sul_tasto",
        "harmonic",
        "pizzicato",
        "bartok",
        "bartok_pizz",
        "col_legno",
        "Fall",
        "fall",
        "Hit",
        "Roll",
        "Hard_Hit",
        "Hard_Roll",
    ];

    // Convert articulations to a vector for custom sorting
    let mut articulation_entries: Vec<(String, Vec<ParsedSample>)> =
        articulations.into_iter().collect();

    // Sort articulations by custom order, with unknown articulations at the end
    articulation_entries.sort_by_key(|(key, _)| {
        articulation_order
            .iter()
            .position(|&order_key| order_key == key)
            .unwrap_or(articulation_order.len())
    });

    // Log warnings for unknown articulations
    for (key, _) in &articulation_entries {
        if !articulation_order.contains(&key.as_str()) {
            eprintln!("Warning: Unknown articulation '{}' - adding at end", key);
        }
    }

    // Generate articulations in the sorted order
    for (articulation_key, samples) in articulation_entries {
        // Group by pitch
        let mut pitches: BTreeMap<u8, Vec<&ParsedSample>> = BTreeMap::new();
        for sample in &samples {
            pitches.entry(sample.midi_note).or_default().push(sample);
        }

        let mut zones = Vec::new();
        for (midi_note, pitch_samples) in &pitches {
            let velocity_layers = generate_velocity_layers(pitch_samples);
            zones.push(Zone {
                pitch: *midi_note,
                velocity_layers,
            });
        }

        // Find min/max pitch for range
        let min_pitch = *pitches.keys().min().unwrap_or(&0);
        let max_pitch = *pitches.keys().max().unwrap_or(&127);

        manifest.articulations.push(Articulation {
            id: format_articulation_id(&articulation_key),
            name: format_articulation_name(&articulation_key),
            range: vec![vec![min_pitch, max_pitch]],
            zones,
        });
    }

    // Write back manifest
    let updated_content = serde_json::to_string_pretty(&manifest).unwrap();
    fs::write(&manifest_path, updated_content)
        .unwrap_or_else(|_| panic!("Failed to write manifest.json for {}", instrument));

    println!(
        "Generated manifest for {} with {} articulations",
        instrument,
        manifest.articulations.len()
    );
}
