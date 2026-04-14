use crate::playback::plugin_host::{resolve_plugin_target, same_bundle_path};
use crate::ManagedPluginInventoryContextState;
use rack::plugin_info::{PluginInfo, PluginType};
use rack::vst3::{Vst3Plugin, Vst3Scanner};
use rack::{MidiEvent, PluginInstance, PluginScanner};
use serde::Serialize;
use tauri::{AppHandle, State};

#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::sync::mpsc;

const PROBE_SAMPLE_RATE: f64 = 48_000.0;
const PROBE_BLOCK_SIZE: usize = 512;
const EFFECT_BLOCK_COUNT: usize = 4;
const INSTRUMENT_BLOCK_COUNT: usize = 16;
const NOTE_OFF_BLOCK_INDEX: usize = 8;
const PROBE_NOTE: u8 = 60;
const PROBE_VELOCITY: u8 = 100;
const PROBE_CHANNEL: u8 = 0;
const AUDIBLE_OUTPUT_EPSILON: f64 = 1.0e-5;
const TRANSFORMED_INPUT_EPSILON: f64 = 1.0e-4;

#[derive(Clone, Copy)]
struct ChannelLayout {
    input_channels: usize,
    output_channels: usize,
}

#[derive(Default)]
struct ProbeMetrics {
    input_peak: f64,
    output_peak: f64,
    input_energy_sum: f64,
    output_energy_sum: f64,
    difference_energy_sum: f64,
    input_sample_count: usize,
    output_sample_count: usize,
    difference_sample_count: usize,
}

impl ProbeMetrics {
    fn input_rms(&self) -> f64 {
        rms(self.input_energy_sum, self.input_sample_count)
    }

    fn output_rms(&self) -> f64 {
        rms(self.output_energy_sum, self.output_sample_count)
    }

    fn difference_rms(&self) -> Option<f64> {
        if self.difference_sample_count == 0 {
            None
        } else {
            Some(rms(
                self.difference_energy_sum,
                self.difference_sample_count,
            ))
        }
    }
}

#[derive(Serialize)]
pub struct PluginPlaybackPocProbeResult {
    pub plugin_path: String,
    pub plugin_name: String,
    pub manufacturer: String,
    pub plugin_type: String,
    pub probe_kind: String,
    pub sample_rate: f64,
    pub block_size: usize,
    pub blocks_processed: usize,
    pub input_channels: usize,
    pub output_channels: usize,
    pub midi_events_sent: usize,
    pub input_peak: f64,
    pub output_peak: f64,
    pub input_rms: f64,
    pub output_rms: f64,
    pub difference_rms: Option<f64>,
    pub produced_audible_output: bool,
    pub transformed_input: bool,
    pub summary: String,
}

#[cfg(target_os = "macos")]
fn scan_and_load_vst3_with_rack(plugin_path: &Path) -> Result<(PluginInfo, Vst3Plugin), String> {
    let search_root = plugin_path.parent().ok_or_else(|| {
        format!(
            "Cannot scan '{}' with rack because it has no parent directory.",
            plugin_path.display()
        )
    })?;

    let scanner = Vst3Scanner::new()
        .map_err(|error| format!("Failed to create rack VST3 scanner: {error}"))?;

    let plugin_info = scanner
        .scan_path(search_root)
        .map_err(|error| {
            format!(
                "Rack failed to scan '{}' for VST3 plugins: {error}",
                search_root.display()
            )
        })?
        .into_iter()
        .find(|info| same_bundle_path(&info.path, plugin_path))
        .ok_or_else(|| {
            format!(
                "Rack scanned '{}' but did not surface class metadata for '{}'.",
                search_root.display(),
                plugin_path.display()
            )
        })?;

    let mut plugin = scanner.load(&plugin_info).map_err(|error| {
        format!(
            "Rack failed to load '{}' from '{}': {error}",
            plugin_info.name,
            plugin_path.display()
        )
    })?;

    plugin
        .initialize(PROBE_SAMPLE_RATE, PROBE_BLOCK_SIZE)
        .map_err(|error| {
            format!(
                "Rack loaded '{}' but failed to initialize it at {} Hz / {} frames: {error}",
                plugin_info.name, PROBE_SAMPLE_RATE, PROBE_BLOCK_SIZE
            )
        })?;

    Ok((plugin_info, plugin))
}

#[cfg(target_os = "macos")]
fn candidate_layouts_for_plugin_type(
    plugin_type: PluginType,
) -> (&'static str, &'static [ChannelLayout]) {
    const EFFECT_LAYOUTS: &[ChannelLayout] = &[
        ChannelLayout {
            input_channels: 2,
            output_channels: 2,
        },
        ChannelLayout {
            input_channels: 1,
            output_channels: 1,
        },
        ChannelLayout {
            input_channels: 2,
            output_channels: 1,
        },
        ChannelLayout {
            input_channels: 1,
            output_channels: 2,
        },
    ];
    const INSTRUMENT_LAYOUTS: &[ChannelLayout] = &[
        ChannelLayout {
            input_channels: 0,
            output_channels: 2,
        },
        ChannelLayout {
            input_channels: 0,
            output_channels: 1,
        },
    ];

    match plugin_type {
        PluginType::Instrument => ("instrument", INSTRUMENT_LAYOUTS),
        _ => ("effect", EFFECT_LAYOUTS),
    }
}

#[cfg(target_os = "macos")]
fn probe_plugin_playback(
    plugin_info: &PluginInfo,
    plugin: &mut Vst3Plugin,
    plugin_path: &Path,
) -> Result<PluginPlaybackPocProbeResult, String> {
    let (probe_kind, layouts) = candidate_layouts_for_plugin_type(plugin_info.plugin_type);
    let mut attempts = Vec::new();

    for layout in layouts {
        let attempt_result = match plugin_info.plugin_type {
            PluginType::Instrument => run_instrument_probe(plugin, *layout),
            _ => run_effect_probe(plugin, *layout),
        };

        match attempt_result {
            Ok((metrics, blocks_processed, midi_events_sent)) => {
                let difference_rms = metrics.difference_rms();
                let output_rms = metrics.output_rms();
                let produced_audible_output = metrics.output_peak > AUDIBLE_OUTPUT_EPSILON
                    || output_rms > AUDIBLE_OUTPUT_EPSILON;
                let transformed_input = difference_rms
                    .map(|value| value > TRANSFORMED_INPUT_EPSILON)
                    .unwrap_or(false);

                let summary = match probe_kind {
                    "instrument" => format!(
                        "Rack initialized '{}' and rendered {} offline blocks with a {}->{} layout after {} MIDI events; output_peak={:.6}, output_rms={:.6}.",
                        plugin_info.name,
                        blocks_processed,
                        layout.input_channels,
                        layout.output_channels,
                        midi_events_sent,
                        metrics.output_peak,
                        output_rms
                    ),
                    _ => format!(
                        "Rack initialized '{}' and processed {} offline blocks with a {}->{} layout; input_peak={:.6}, output_peak={:.6}, difference_rms={:.6}.",
                        plugin_info.name,
                        blocks_processed,
                        layout.input_channels,
                        layout.output_channels,
                        metrics.input_peak,
                        metrics.output_peak,
                        difference_rms.unwrap_or(0.0)
                    ),
                };

                return Ok(PluginPlaybackPocProbeResult {
                    plugin_path: plugin_path.display().to_string(),
                    plugin_name: plugin_info.name.clone(),
                    manufacturer: plugin_info.manufacturer.clone(),
                    plugin_type: format!("{:?}", plugin_info.plugin_type),
                    probe_kind: probe_kind.to_string(),
                    sample_rate: PROBE_SAMPLE_RATE,
                    block_size: PROBE_BLOCK_SIZE,
                    blocks_processed,
                    input_channels: layout.input_channels,
                    output_channels: layout.output_channels,
                    midi_events_sent,
                    input_peak: metrics.input_peak,
                    output_peak: metrics.output_peak,
                    input_rms: metrics.input_rms(),
                    output_rms,
                    difference_rms,
                    produced_audible_output,
                    transformed_input,
                    summary,
                });
            }
            Err(error) => {
                attempts.push(format!(
                    "{}->{}: {}",
                    layout.input_channels, layout.output_channels, error
                ));
                let _ = plugin.reset();
            }
        }
    }

    Err(format!(
        "Rack initialized '{}' but could not complete the {} playback proof for '{}'. Tried layouts: {}",
        plugin_info.name,
        probe_kind,
        plugin_path.display(),
        attempts.join(" | ")
    ))
}

#[cfg(target_os = "macos")]
fn probe_plugin_playback_on_main_thread(
    app: &AppHandle,
    plugin_path: PathBuf,
) -> Result<PluginPlaybackPocProbeResult, String> {
    let (sender, receiver) = mpsc::sync_channel(1);

    app.run_on_main_thread(move || {
        let result =
            scan_and_load_vst3_with_rack(&plugin_path).and_then(|(plugin_info, mut plugin)| {
                probe_plugin_playback(&plugin_info, &mut plugin, &plugin_path)
            });

        let _ = sender.send(result);
    })
    .map_err(|error| {
        format!("Failed to schedule plugin playback proof on the main thread: {error}")
    })?;

    receiver.recv().map_err(|_| {
        String::from("Failed to receive plugin playback proof result from the main thread")
    })?
}

#[cfg(target_os = "macos")]
fn run_effect_probe(
    plugin: &mut Vst3Plugin,
    layout: ChannelLayout,
) -> Result<(ProbeMetrics, usize, usize), String> {
    let mut metrics = ProbeMetrics::default();

    for block_index in 0..EFFECT_BLOCK_COUNT {
        let inputs = build_effect_input_block(layout.input_channels, block_index);
        let mut outputs = build_empty_output_block(layout.output_channels);
        let input_refs = inputs.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut output_refs = outputs
            .iter_mut()
            .map(Vec::as_mut_slice)
            .collect::<Vec<_>>();

        plugin
            .process(&input_refs, &mut output_refs, PROBE_BLOCK_SIZE)
            .map_err(|error| error.to_string())?;

        drop(output_refs);
        accumulate_metrics(&mut metrics, &inputs, &outputs);
    }

    Ok((metrics, EFFECT_BLOCK_COUNT, 0))
}

#[cfg(target_os = "macos")]
fn run_instrument_probe(
    plugin: &mut Vst3Plugin,
    layout: ChannelLayout,
) -> Result<(ProbeMetrics, usize, usize), String> {
    let mut metrics = ProbeMetrics::default();

    plugin.reset().map_err(|error| error.to_string())?;
    plugin
        .send_midi(&[MidiEvent::note_on(
            PROBE_NOTE,
            PROBE_VELOCITY,
            PROBE_CHANNEL,
            0,
        )])
        .map_err(|error| error.to_string())?;

    let mut midi_events_sent = 1;

    for block_index in 0..INSTRUMENT_BLOCK_COUNT {
        if block_index == NOTE_OFF_BLOCK_INDEX {
            plugin
                .send_midi(&[MidiEvent::note_off(PROBE_NOTE, 0, PROBE_CHANNEL, 0)])
                .map_err(|error| error.to_string())?;
            midi_events_sent += 1;
        }

        let inputs = build_empty_input_block(layout.input_channels);
        let mut outputs = build_empty_output_block(layout.output_channels);
        let input_refs = inputs.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut output_refs = outputs
            .iter_mut()
            .map(Vec::as_mut_slice)
            .collect::<Vec<_>>();

        plugin
            .process(&input_refs, &mut output_refs, PROBE_BLOCK_SIZE)
            .map_err(|error| error.to_string())?;

        drop(output_refs);
        accumulate_metrics(&mut metrics, &inputs, &outputs);
    }

    plugin.reset().map_err(|error| error.to_string())?;

    Ok((metrics, INSTRUMENT_BLOCK_COUNT, midi_events_sent))
}

#[cfg(target_os = "macos")]
fn build_empty_input_block(channel_count: usize) -> Vec<Vec<f32>> {
    (0..channel_count)
        .map(|_| vec![0.0_f32; PROBE_BLOCK_SIZE])
        .collect()
}

#[cfg(target_os = "macos")]
fn build_empty_output_block(channel_count: usize) -> Vec<Vec<f32>> {
    (0..channel_count)
        .map(|_| vec![0.0_f32; PROBE_BLOCK_SIZE])
        .collect()
}

#[cfg(target_os = "macos")]
fn build_effect_input_block(channel_count: usize, block_index: usize) -> Vec<Vec<f32>> {
    (0..channel_count)
        .map(|channel_index| {
            (0..PROBE_BLOCK_SIZE)
                .map(|sample_index| {
                    let absolute_index = block_index * PROBE_BLOCK_SIZE + sample_index;
                    let impulse = if absolute_index == 0 {
                        1.0_f32
                    } else {
                        0.0_f32
                    };
                    let phase = absolute_index as f32 * 0.071 + channel_index as f32 * 0.37;
                    let tone = phase.sin() * 0.2_f32;
                    impulse + tone
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn accumulate_metrics(metrics: &mut ProbeMetrics, inputs: &[Vec<f32>], outputs: &[Vec<f32>]) {
    for channel in inputs {
        for sample in channel {
            let value = f64::from(*sample);
            metrics.input_peak = metrics.input_peak.max(value.abs());
            metrics.input_energy_sum += value * value;
            metrics.input_sample_count += 1;
        }
    }

    for channel in outputs {
        for sample in channel {
            let value = f64::from(*sample);
            metrics.output_peak = metrics.output_peak.max(value.abs());
            metrics.output_energy_sum += value * value;
            metrics.output_sample_count += 1;
        }
    }

    let comparable_channels = inputs.len().min(outputs.len());
    for channel_index in 0..comparable_channels {
        for sample_index in 0..PROBE_BLOCK_SIZE {
            let difference = f64::from(outputs[channel_index][sample_index])
                - f64::from(inputs[channel_index][sample_index]);
            metrics.difference_energy_sum += difference * difference;
            metrics.difference_sample_count += 1;
        }
    }
}

fn rms(energy_sum: f64, sample_count: usize) -> f64 {
    if sample_count == 0 {
        0.0
    } else {
        (energy_sum / sample_count as f64).sqrt()
    }
}

#[cfg(target_os = "macos")]
#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_playback_poc_probe(
    app: AppHandle,
    plugin_inventory_state: State<'_, ManagedPluginInventoryContextState>,
    plugin_id: Option<String>,
    plugin_path: Option<String>,
) -> Result<PluginPlaybackPocProbeResult, String> {
    let plugin_path = resolve_plugin_target(
        plugin_id.as_deref(),
        plugin_path.as_deref(),
        plugin_inventory_state.inner(),
    )?;

    probe_plugin_playback_on_main_thread(&app, plugin_path)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command(rename_all = "snake_case")]
pub async fn plugin_playback_poc_probe(
    _plugin_inventory_state: State<'_, ManagedPluginInventoryContextState>,
    _plugin_id: Option<String>,
    _plugin_path: Option<String>,
) -> Result<PluginPlaybackPocProbeResult, String> {
    Err(String::from("plugin_playback_poc_probe is macOS-only"))
}

#[cfg(test)]
mod tests {
    use super::{accumulate_metrics, rms, ProbeMetrics, PROBE_BLOCK_SIZE};

    #[test]
    fn rms_returns_zero_for_empty_input() {
        assert_eq!(rms(0.0, 0), 0.0);
    }

    #[test]
    fn accumulate_metrics_tracks_input_output_and_difference_energy() {
        let inputs = vec![vec![1.0_f32; PROBE_BLOCK_SIZE]];
        let outputs = vec![vec![0.5_f32; PROBE_BLOCK_SIZE]];
        let mut metrics = ProbeMetrics::default();

        accumulate_metrics(&mut metrics, &inputs, &outputs);

        assert_eq!(metrics.input_peak, 1.0);
        assert_eq!(metrics.output_peak, 0.5);
        assert!(metrics.input_rms() > metrics.output_rms());
        assert_eq!(metrics.difference_sample_count, PROBE_BLOCK_SIZE);
        assert!(metrics.difference_rms().unwrap() > 0.0);
    }

    #[test]
    fn accumulate_metrics_skips_difference_when_channel_counts_do_not_overlap() {
        let inputs: Vec<Vec<f32>> = Vec::new();
        let outputs = vec![vec![0.25_f32; PROBE_BLOCK_SIZE]];
        let mut metrics = ProbeMetrics::default();

        accumulate_metrics(&mut metrics, &inputs, &outputs);

        assert_eq!(metrics.output_peak, 0.25);
        assert_eq!(metrics.output_sample_count, PROBE_BLOCK_SIZE);
        assert_eq!(metrics.difference_rms(), None);
    }
}
