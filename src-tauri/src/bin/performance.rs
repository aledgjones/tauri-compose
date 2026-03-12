#![allow(dead_code, unused_imports, clippy::items_after_test_module)]

use std::sync::Mutex;

#[path = "../renderer/mod.rs"]
mod renderer;
#[path = "../score/mod.rs"]
mod score;
#[path = "../ui/mod.rs"]
mod ui;
#[path = "../utils.rs"]
mod utils;

mod preferences {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Preferences {
        pub audition: bool,
    }
}

mod playback {
    pub mod sampler {
        use tauri::AppHandle;

        use crate::score::Score;
        use crate::{AudioContextState, SamplerContextState};

        pub fn sync_sampler_with_score(
            _app: &AppHandle,
            _score: &Score,
            _sampler_state: &SamplerContextState,
            _audio_state: &AudioContextState,
            _error_context: &str,
        ) {
        }
    }

    pub mod transport {
        use rustc_hash::FxHashMap;
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::sync::{Arc, RwLock};

        use crate::score::components::duration::Tick;
        use crate::utils::UniqueId;
        use crate::AppContext;

        #[derive(Clone)]
        pub struct InstrumentMixParams {
            pub volume: u8,
            pub pan: u8,
            pub mute: bool,
            pub solo: bool,
        }

        pub struct TransportShared {
            pub playing: AtomicBool,
            pub current_tick: AtomicU64,
            pub stop_signal: AtomicBool,
            pub tempo_bits: AtomicU64,
            pub mix_params: RwLock<FxHashMap<UniqueId, InstrumentMixParams>>,
        }

        impl Default for TransportShared {
            fn default() -> Self {
                Self {
                    playing: AtomicBool::new(false),
                    current_tick: AtomicU64::new(0),
                    stop_signal: AtomicBool::new(false),
                    tempo_bits: AtomicU64::new(120.0_f64.to_bits()),
                    mix_params: RwLock::new(FxHashMap::default()),
                }
            }
        }

        impl TransportShared {
            pub fn set_tempo(&self, bpm: f64) {
                self.tempo_bits.store(bpm.to_bits(), Ordering::Relaxed);
            }
        }

        pub type TransportSharedState = Arc<TransportShared>;

        pub(crate) fn sync_transport_to_app_state(
            shared: &TransportSharedState,
            _app_ctx: &AppContext,
            tick: Tick,
        ) -> f64 {
            shared.current_tick.store(tick, Ordering::Relaxed);
            let bpm = 120.0;
            shared.set_tempo(bpm);
            bpm
        }
    }
}

struct AudioContext;
struct SamplerContext;

#[derive(Clone)]
struct AppContext {
    ui: ui::Ui,
    score: score::Score,
    preferences: preferences::Preferences,
    next_id: utils::UniqueId,
    current_file_path: Option<String>,
}

type AppContextState = Mutex<AppContext>;
type RendererContextState = Mutex<renderer::RendererContext>;
type AudioContextState = Mutex<AudioContext>;
type SamplerContextState = Mutex<SamplerContext>;

impl AppContext {
    fn flow_id_for_track(&self, track_id: utils::UniqueId) -> Option<utils::UniqueId> {
        self.score.flows.by_id.values().find_map(|flow| {
            if flow.master_id == track_id {
                return Some(flow.id);
            }

            flow.staves.by_id.values().find_map(|stave| {
                if stave.master_id == track_id || stave.track_ids.contains(&track_id) {
                    Some(flow.id)
                } else {
                    None
                }
            })
        })
    }

    fn render_only(&mut self, _app: &tauri::AppHandle, _renderer: &mut renderer::RendererContext) {}

    fn emit_general_state(
        &mut self,
        _app: &tauri::AppHandle,
        _renderer: &mut renderer::RendererContext,
        _recalculation_request: Option<instructions::FlowRenderRecalculationRequest>,
    ) {
    }

    fn emit_affected_flows(
        &mut self,
        _app: &tauri::AppHandle,
        _renderer: &mut renderer::RendererContext,
        _flow_ids: &[utils::UniqueId],
    ) {
    }

    fn emit_affected_track(
        &mut self,
        _app: &tauri::AppHandle,
        _renderer: &mut renderer::RendererContext,
        _track_id: utils::UniqueId,
    ) {
    }

    fn generate_id(&mut self) -> utils::UniqueId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

mod instructions {
    include!("../instructions/mod.rs");

    const BASELINE_REGRESSION_WARNING_US: u128 = 2_000;

    use crate::preferences::Preferences;
    use crate::score::components::articulation::Articulations;
    use crate::score::components::duration::NoteDuration;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tempo::Tempo;
    use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
    use crate::score::entries::tone::Tone;
    use crate::score::entries::Entry;
    use crate::score::flows::Flow;
    use crate::score::instruments::defs::InstrumentType;
    use crate::score::instruments::Instrument;
    use crate::score::players::{Player, PlayerType};
    use crate::score::tracks::Track;
    use crate::score::Score;
    use crate::ui::Ui;

    struct CliOptions {
        output_mode: OutputMode,
        sample_count: usize,
    }

    enum OutputMode {
        Table,
        Json,
    }

    impl CliOptions {
        fn from_args() -> Result<Self, String> {
            let mut output_mode = OutputMode::Table;
            let mut sample_count = 5_usize;
            let mut args = std::env::args().skip(1).peekable();

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--json" => output_mode = OutputMode::Json,
                    "--samples" => {
                        let value = args
                            .next()
                            .ok_or_else(|| String::from("missing value for --samples"))?;
                        sample_count = parse_sample_count(&value)?;
                    }
                    _ if arg.starts_with("--samples=") => {
                        let value = arg.trim_start_matches("--samples=");
                        sample_count = parse_sample_count(value)?;
                    }
                    _ => return Err(format!("unknown argument: {}", arg)),
                }
            }

            Ok(Self {
                output_mode,
                sample_count,
            })
        }
    }

    fn parse_sample_count(value: &str) -> Result<usize, String> {
        let parsed = value
            .parse::<usize>()
            .map_err(|_| format!("invalid sample count: {}", value))?;

        if parsed == 0 {
            return Err(String::from("sample count must be at least 1"));
        }

        Ok(parsed)
    }

    #[derive(Clone, Copy)]
    struct RenderMeasurement {
        prepared_us: u128,
        segment_us: u128,
        compose_us: u128,
        composed_flows: usize,
        recalculated_flows: usize,
    }

    impl RenderMeasurement {
        fn total_us(&self) -> u128 {
            self.prepared_us + self.segment_us + self.compose_us
        }
    }

    #[derive(serde::Serialize)]
    struct TimingSummary {
        min_us: u128,
        median_us: u128,
        max_us: u128,
    }

    impl TimingSummary {
        fn from_samples(samples: &[u128]) -> Self {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();

            let median_us = if sorted.len().is_multiple_of(2) {
                let right = sorted.len() / 2;
                let left = right - 1;
                (sorted[left] + sorted[right]) / 2
            } else {
                sorted[sorted.len() / 2]
            };

            Self {
                min_us: *sorted.first().expect("timing samples should not be empty"),
                median_us,
                max_us: *sorted.last().expect("timing samples should not be empty"),
            }
        }
    }

    #[derive(serde::Serialize)]
    struct ScenarioReport {
        scenario: &'static str,
        label: &'static str,
        description: &'static str,
        request: String,
        instruments: u64,
        bars_per_flow: u64,
        visible_flows: usize,
        sample_count: usize,
        prepared_us: TimingSummary,
        segment_us: TimingSummary,
        compose_us: TimingSummary,
        total_us: TimingSummary,
        composed_flows: usize,
        recalculated_flows: usize,
        warning_threshold_us: Option<u128>,
        regression_warning: bool,
    }

    struct ScenarioSpec {
        scenario: &'static str,
        label: &'static str,
        description: &'static str,
        request: &'static str,
        visible_flows: usize,
        warning_threshold_us: Option<u128>,
    }

    impl ScenarioReport {
        fn from_measurements(
            spec: ScenarioSpec,
            bars_per_flow: u64,
            measurements: &[RenderMeasurement],
        ) -> Self {
            let prepared_samples = measurements
                .iter()
                .map(|measurement| measurement.prepared_us)
                .collect::<Vec<_>>();
            let segment_samples = measurements
                .iter()
                .map(|measurement| measurement.segment_us)
                .collect::<Vec<_>>();
            let compose_samples = measurements
                .iter()
                .map(|measurement| measurement.compose_us)
                .collect::<Vec<_>>();
            let total_samples = measurements
                .iter()
                .map(RenderMeasurement::total_us)
                .collect::<Vec<_>>();
            let last = measurements
                .last()
                .expect("scenario measurements should not be empty");

            Self {
                scenario: spec.scenario,
                label: spec.label,
                description: spec.description,
                request: String::from(spec.request),
                instruments: 3,
                bars_per_flow,
                visible_flows: spec.visible_flows,
                sample_count: measurements.len(),
                prepared_us: TimingSummary::from_samples(&prepared_samples),
                segment_us: TimingSummary::from_samples(&segment_samples),
                compose_us: TimingSummary::from_samples(&compose_samples),
                total_us: TimingSummary::from_samples(&total_samples),
                composed_flows: last.composed_flows,
                recalculated_flows: last.recalculated_flows,
                warning_threshold_us: spec.warning_threshold_us,
                regression_warning: spec.warning_threshold_us.is_some_and(|threshold| {
                    TimingSummary::from_samples(&total_samples).median_us >= threshold
                }),
            }
        }
    }

    fn print_characterization_table(reports: &[ScenarioReport]) {
        println!("render characterization");
        println!("  build: release standalone binary");
        println!("  fixture: 3 instruments per flow, 30 bars per flow");
        if let Some(report) = reports.first() {
            println!("  samples: {} measured (+1 warm-up)", report.sample_count);
        }
        println!();
        println!(
            "{:<18} {:<14} {:>5} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
            "scenario",
            "request",
            "flows",
            "recalc",
            "tot-min",
            "tot-med",
            "tot-max",
            "prep-med",
            "seg-med",
            "comp-med",
        );
        println!(
            "{:-<18} {:-<14} {:-<5} {:-<8} {:-<8} {:-<8} {:-<8} {:-<8} {:-<8} {:-<8}",
            "", "", "", "", "", "", "", "", "", ""
        );

        for report in reports {
            println!(
                "{:<18} {:<14} {:>5} {:>8} {:>7} {:>7} {:>7} {:>8} {:>7} {:>8}",
                report.label,
                report.request,
                report.visible_flows,
                format!("{}/{}", report.recalculated_flows, report.composed_flows),
                format!("{}us", report.total_us.min_us),
                format!("{}us", report.total_us.median_us),
                format!("{}us", report.total_us.max_us),
                format!("{}us", report.prepared_us.median_us),
                format!("{}us", report.segment_us.median_us),
                format!("{}us", report.compose_us.median_us),
            );
        }

        println!();
        println!("notes");
        for report in reports {
            println!(
                "  {:<18} {}",
                format!("{}:", report.label),
                report.description
            );
        }

        if let Some(report) = reports
            .iter()
            .find(|report| report.warning_threshold_us.is_some())
        {
            let threshold = report.warning_threshold_us.unwrap();
            println!();
            println!("baseline warning");
            println!(
                "  threshold: total median >= {}us for the representative single-flow pass",
                threshold
            );
            println!(
                "  status: {} (current total median: {}us)",
                if report.regression_warning {
                    "warning triggered"
                } else {
                    "within threshold"
                },
                report.total_us.median_us
            );
        }
    }

    fn print_characterization_json(reports: &[ScenarioReport]) {
        println!(
            "{}",
            serde_json::to_string_pretty(reports)
                .expect("scenario reports should serialize to JSON")
        );
    }

    fn make_characterization_app_context(flow_ids: &[u64], bar_count: u64) -> crate::AppContext {
        let mut app = crate::AppContext {
            ui: Ui::new(),
            score: Score::new(),
            preferences: Preferences { audition: true },
            next_id: 1000,
            current_file_path: None,
        };

        for instrument_index in 0..3_u64 {
            let instrument_id = app.generate_id();
            let player_id = app.generate_id();
            let stave_id = app.generate_id();

            app.score.players.order.push(player_id);
            app.score.players.by_id.insert(
                player_id,
                Player {
                    id: player_id,
                    player_type: PlayerType::Solo,
                    instrument_ids: vec![instrument_id],
                },
            );
            app.score.instruments.by_id.insert(
                instrument_id,
                Instrument {
                    id: instrument_id,
                    definition_id: String::from("strings.violin"),
                    instrument_type: InstrumentType::Melodic,
                    long_name: format!("Validation Violin {}", instrument_index + 1),
                    short_name: format!("V{}", instrument_index + 1),
                    stave_ids: vec![stave_id],
                    count: None,
                    assigned_plugin_id: None,
                    volume: 100,
                    pan: 64,
                    mute: false,
                    solo: false,
                },
            );

            app.assign_player_to_layout(0, player_id);
        }

        for (flow_index, flow_id) in flow_ids.iter().copied().enumerate() {
            let master_id = app.generate_id();
            let time_signature_id = app.generate_id();
            let tempo_id = app.generate_id();

            let mut master = Track::new(master_id);
            master.insert_entry(Entry::TimeSignature(TimeSignature::new(
                time_signature_id,
                0,
                4,
                NoteDuration::Quarter,
                TimeSignatureDrawType::Regular,
                None,
            )));
            master.insert_entry(Entry::Tempo(Tempo::new(
                tempo_id,
                0,
                format!("Validation {}", flow_index + 1),
                NoteDuration::Quarter,
                0,
                96,
                false,
                true,
                true,
                false,
            )));
            app.score.tracks.by_id.insert(master_id, master);
            app.score.flows.order.push(flow_id);
            app.score
                .flows
                .by_id
                .insert(flow_id, Flow::new(flow_id, master_id));
            app.score.flows.by_id.get_mut(&flow_id).unwrap().length = bar_count * 128;
            app.assign_flow_to_layout(0, flow_id);

            for player_id in app.score.players.order.clone() {
                app.assign_player_to_flow(flow_id, player_id);
            }

            let track_ids = app.score.flows.by_id[&flow_id]
                .staves
                .by_id
                .values()
                .flat_map(|stave| stave.track_ids.iter().copied())
                .collect::<Vec<_>>();

            for (track_index, track_id) in track_ids.into_iter().enumerate() {
                let mut tone_entries = Vec::new();
                for bar in 0..bar_count {
                    let tone_id = app.generate_id();
                    tone_entries.push(Entry::Tone(Tone::new(
                        tone_id,
                        bar * 128,
                        NoteDuration::Quarter.to_ticks(),
                        Pitch::new(60 + track_index as u8 * 2, Accidental::Natural),
                        Velocity::new(90),
                        Articulations::new(),
                    )));
                }
                let track = app.score.tracks.by_id.get_mut(&track_id).unwrap();
                for tone_entry in tone_entries {
                    track.insert_entry(tone_entry);
                }
            }
        }

        app.ui.layout_id = Some(0);
        app.ui.flow_id = flow_ids.first().copied();
        app
    }

    fn make_characterization_renderer() -> RendererContext {
        RendererContext {
            backend: None,
            converter: crate::renderer::converter::Converter::new(1.0, 1.75),
            px_per_mm: 1.0,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
            flow_segment_cache: None,
        }
    }

    fn measure_render_pass_once(
        app: &crate::AppContext,
        renderer: &mut RendererContext,
        request: FlowRenderRecalculationRequest,
    ) -> RenderMeasurement {
        let converter = build_render_converter(app, renderer, 0);
        let preparation_start = std::time::Instant::now();
        let plan =
            prepare_flow_render_plan(&app.score, 0, app.ui.flow_id.unwrap(), request).unwrap();
        let resolved_plan = resolve_render_plan_recalculation(app, renderer, &plan);
        let preparation_us = preparation_start.elapsed().as_micros();

        let segment_start = std::time::Instant::now();
        let segments = collect_flow_render_segments(app, renderer, &resolved_plan, &converter);
        let segment_us = segment_start.elapsed().as_micros();

        let compose_start = std::time::Instant::now();
        let _composed = compose_render_segments(
            segments
                .iter()
                .map(|(_, segment)| segment.clone())
                .collect(),
            &converter,
        );
        let compose_us = compose_start.elapsed().as_micros();

        RenderMeasurement {
            prepared_us: preparation_us,
            segment_us,
            compose_us,
            composed_flows: resolved_plan.composed_flow_count(),
            recalculated_flows: resolved_plan.recalculated_flow_count(),
        }
    }

    fn sample_render_pass(
        app: &crate::AppContext,
        renderer: &mut RendererContext,
        request: FlowRenderRecalculationRequest,
        sample_count: usize,
    ) -> Vec<RenderMeasurement> {
        let _ = measure_render_pass_once(app, renderer, request.clone());

        (0..sample_count)
            .map(|_| measure_render_pass_once(app, renderer, request.clone()))
            .collect()
    }

    pub fn run_characterization_report() -> Result<(), String> {
        let bars_per_flow = 30;
        let cli = CliOptions::from_args()?;

        let baseline_app = make_characterization_app_context(&[10], 30);
        let mut baseline_renderer = make_characterization_renderer();
        let baseline_measurements = sample_render_pass(
            &baseline_app,
            &mut baseline_renderer,
            FlowRenderRecalculationRequest::full_visible_flows(),
            cli.sample_count,
        );

        let multi_flow_app = make_characterization_app_context(&[10, 20], 30);
        let mut multi_flow_renderer = make_characterization_renderer();
        let multi_measurements = sample_render_pass(
            &multi_flow_app,
            &mut multi_flow_renderer,
            FlowRenderRecalculationRequest::full_visible_flows(),
            cli.sample_count,
        );

        let local_edit_app = make_characterization_app_context(&[10, 20, 30], 30);
        let mut local_edit_renderer = make_characterization_renderer();
        let cache_converter = build_render_converter(&local_edit_app, &local_edit_renderer, 0);
        let cached_segments = rustc_hash::FxHashMap::from_iter([
            (
                10,
                build_flow_render_segment(&local_edit_app, 0, 10, &cache_converter),
            ),
            (
                20,
                build_flow_render_segment(&local_edit_app, 0, 20, &cache_converter),
            ),
            (
                30,
                build_flow_render_segment(&local_edit_app, 0, 30, &cache_converter),
            ),
        ]);
        local_edit_renderer.store_flow_segments(0, &cache_converter, false, cached_segments);
        let local_measurements = sample_render_pass(
            &local_edit_app,
            &mut local_edit_renderer,
            FlowRenderRecalculationRequest::Flows(vec![20]),
            cli.sample_count,
        );

        let reports = vec![
            ScenarioReport::from_measurements(
                ScenarioSpec {
                    scenario: "baseline_single_flow",
                    label: "baseline",
                    description: "single-flow baseline",
                    request: "full",
                    visible_flows: 1,
                    warning_threshold_us: Some(BASELINE_REGRESSION_WARNING_US),
                },
                bars_per_flow,
                &baseline_measurements,
            ),
            ScenarioReport::from_measurements(
                ScenarioSpec {
                    scenario: "shared_two_flow",
                    label: "shared-two-flow",
                    description: "full shared-surface rerender across two visible flows",
                    request: "full",
                    visible_flows: 2,
                    warning_threshold_us: None,
                },
                bars_per_flow,
                &multi_measurements,
            ),
            ScenarioReport::from_measurements(
                ScenarioSpec {
                    scenario: "local_edit_three_flow",
                    label: "local-edit",
                    description: "targeted edit with cache reuse across three visible flows",
                    request: "flow 20",
                    visible_flows: 3,
                    warning_threshold_us: None,
                },
                bars_per_flow,
                &local_measurements,
            ),
        ];

        match cli.output_mode {
            OutputMode::Table => print_characterization_table(&reports),
            OutputMode::Json => print_characterization_json(&reports),
        }

        Ok(())
    }
}

fn main() {
    if let Err(error) = instructions::run_characterization_report() {
        eprintln!("performance: {}", error);
        std::process::exit(2);
    }
}
