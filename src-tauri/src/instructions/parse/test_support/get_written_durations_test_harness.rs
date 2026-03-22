use crate::instructions::parse::get_measures::get_measures;
use crate::instructions::parse::get_tone_offsets::get_tone_offsets;
use crate::instructions::parse::get_written_durations::get_written_durations;
use crate::instructions::parse::get_written_durations::AnchoredNotationTrack;
use crate::instructions::parse::get_written_durations::NotationTrack;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::entries::barline::{Barline, BarlineDrawType};
use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
use crate::score::entries::tuplet::{Tuplet, TupletEntry};
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::defs::InstrumentType;
use crate::score::instruments::{Instrument, PlaybackAssignmentIntent};
use crate::score::staves::{Stave, Staves};
use crate::score::tracks::{Track, Tracks};
use rustc_hash::{FxHashMap, FxHashSet};

pub(crate) type ToneSpec = (u64, Tick, Ticks, u8);
pub(crate) type TupletToneSpec = (u64, Tick, Ticks, u8);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NotationExpectation {
    tick: Tick,
    duration: Ticks,
    is_rest: bool,
    tone_ids: Vec<u64>,
    tie_ids: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AnchoredNotationExpectation {
    owner_id: u64,
    host_start_tick: Tick,
    host_end_tick: Tick,
    length: Ticks,
    entries: Vec<NotationExpectation>,
}

pub(crate) struct Scenario {
    pub(crate) name: &'static str,
    pub(crate) time_signature: TimeSignature,
    pub(crate) flow_length: Ticks,
    pub(crate) tone_specs: Vec<ToneSpec>,
    pub(crate) expectations: Vec<NotationExpectation>,
}

pub(crate) struct TupletSpec {
    pub(crate) id: u64,
    pub(crate) tick: Tick,
    pub(crate) note_count: u8,
    pub(crate) in_space_of: u8,
    pub(crate) base_note_value: NoteDuration,
    pub(crate) groupings: Vec<u8>,
    pub(crate) tone_specs: Vec<TupletToneSpec>,
}

struct ParserFixture {
    pub(crate) flow: Flow,
    pub(crate) instrument: Instrument,
    pub(crate) tracks: Tracks,
    pub(crate) track_id: u64,
}

pub(crate) fn notation_expectation(
    tick: Tick,
    duration: Ticks,
    is_rest: bool,
    tone_ids: Vec<u64>,
    tie_ids: Vec<u64>,
) -> NotationExpectation {
    NotationExpectation {
        tick,
        duration,
        is_rest,
        tone_ids,
        tie_ids,
    }
}

pub(crate) fn anchored_notation_expectation(
    owner_id: u64,
    host_start_tick: Tick,
    host_end_tick: Tick,
    length: Ticks,
    entries: Vec<NotationExpectation>,
) -> AnchoredNotationExpectation {
    AnchoredNotationExpectation {
        owner_id,
        host_start_tick,
        host_end_tick,
        length,
        entries,
    }
}

pub(crate) fn assert_notation_track_matches(
    notation_track: &NotationTrack,
    expectations: Vec<NotationExpectation>,
) {
    assert_eq!(normalize_notation_track(notation_track), expectations);
}

pub(crate) fn normalize_notation_track(notation_track: &NotationTrack) -> Vec<NotationExpectation> {
    let mut actual = notation_track.track.values().collect::<Vec<_>>();
    actual.sort_by_key(|entry| entry.tick);

    actual
        .into_iter()
        .map(|entry| {
            let mut tone_ids = entry.tones.iter().map(|tone| tone.id).collect::<Vec<_>>();
            let mut tie_ids = entry.ties.iter().copied().collect::<Vec<_>>();
            tone_ids.sort_unstable();
            tie_ids.sort_unstable();

            NotationExpectation {
                tick: entry.tick,
                duration: entry.duration,
                is_rest: entry.is_rest(),
                tone_ids,
                tie_ids,
            }
        })
        .collect::<Vec<_>>()
}

pub(crate) fn normalize_anchored_notation_track(
    anchored_track: &AnchoredNotationTrack,
) -> AnchoredNotationExpectation {
    AnchoredNotationExpectation {
        owner_id: anchored_track.owner_id,
        host_start_tick: anchored_track.host_span.start_tick,
        host_end_tick: anchored_track.host_span.end_tick,
        length: anchored_track.notation.length,
        entries: normalize_notation_track(&anchored_track.notation),
    }
}

pub(crate) fn assert_anchored_notation_track_matches(
    anchored_track: &AnchoredNotationTrack,
    expectation: AnchoredNotationExpectation,
) {
    assert_eq!(
        normalize_anchored_notation_track(anchored_track),
        expectation
    );
}

pub(crate) fn make_time_signature_for_test(
    tick: Tick,
    beats: u8,
    beat_type: NoteDuration,
) -> TimeSignature {
    TimeSignature::new(
        1,
        tick,
        beats,
        beat_type,
        TimeSignatureDrawType::Regular,
        None,
    )
}

fn make_tone_entry(id: u64, tick: Tick, duration: Ticks, pitch_int: u8) -> Entry {
    use crate::score::components::articulation::Articulations;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tone::Tone;

    Entry::Tone(Tone::new(
        id,
        tick,
        duration,
        Pitch::new(pitch_int, Accidental::Natural),
        Velocity::new(100),
        Articulations::new(),
    ))
}

fn make_parser_fixture(
    time_signature: TimeSignature,
    tone_specs: &[(u64, Tick, Ticks, u8)],
) -> ParserFixture {
    make_parser_fixture_with_length(
        time_signature.clone(),
        time_signature.ticks_per_bar(),
        tone_specs,
    )
}

fn make_parser_fixture_with_length(
    time_signature: TimeSignature,
    flow_length: Ticks,
    tone_specs: &[(u64, Tick, Ticks, u8)],
) -> ParserFixture {
    make_parser_fixture_with_length_and_tuplets_and_barlines(
        time_signature,
        flow_length,
        tone_specs,
        &[],
        &[],
    )
}

fn make_parser_fixture_with_length_and_tuplets(
    time_signature: TimeSignature,
    flow_length: Ticks,
    tone_specs: &[(u64, Tick, Ticks, u8)],
    tuplet_specs: &[TupletSpec],
) -> ParserFixture {
    make_parser_fixture_with_length_and_tuplets_and_barlines(
        time_signature,
        flow_length,
        tone_specs,
        tuplet_specs,
        &[],
    )
}

fn make_parser_fixture_with_length_and_tuplets_and_barlines(
    time_signature: TimeSignature,
    flow_length: Ticks,
    tone_specs: &[(u64, Tick, Ticks, u8)],
    tuplet_specs: &[TupletSpec],
    barline_ticks: &[Tick],
) -> ParserFixture {
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::entries::clef::{Clef, ClefDrawType};

    let master_id = 10;
    let track_id = 11;
    let stave_id = 12;
    let instrument_id = 13;
    let flow_id = 14;

    let mut master = Track::new(master_id);
    master.insert_entry(Entry::TimeSignature(time_signature.clone()));
    master.insert_entry(Entry::Clef(Clef::new(
        2,
        0,
        Pitch::new(60, Accidental::Natural),
        0,
        ClefDrawType::C,
    )));

    let mut track = Track::new(track_id);
    for (id, tick, duration, pitch_int) in tone_specs {
        track.insert_entry(make_tone_entry(*id, *tick, *duration, *pitch_int));
    }

    for (index, tick) in barline_ticks.iter().enumerate() {
        track.insert_entry(Entry::Barline(Barline::new(
            10_000 + index as u64,
            *tick,
            BarlineDrawType::Single,
        )));
    }

    for tuplet_spec in tuplet_specs {
        let mut tuplet = Tuplet::new(
            tuplet_spec.id,
            tuplet_spec.tick,
            tuplet_spec.note_count,
            tuplet_spec.in_space_of,
            tuplet_spec.base_note_value.clone(),
            tuplet_spec.groupings.clone(),
        );

        for (id, tick, duration, pitch_int) in &tuplet_spec.tone_specs {
            let Entry::Tone(tone) = make_tone_entry(*id, *tick, *duration, *pitch_int) else {
                unreachable!("make_tone_entry always returns Entry::Tone")
            };
            tuplet.entries.insert_entry(TupletEntry::Tone(tone));
        }

        track.insert_entry(Entry::Tuplet(Box::new(tuplet)));
    }

    let mut by_id = FxHashMap::default();
    by_id.insert(master_id, master);
    by_id.insert(track_id, track);

    let mut staves = Staves::new();
    staves.by_id.insert(
        stave_id,
        Stave {
            id: stave_id,
            lines: vec![1, 1, 1, 1, 1],
            master_id,
            track_ids: vec![track_id],
        },
    );

    let flow = Flow {
        id: flow_id,
        title: String::new(),
        player_ids: FxHashSet::default(),
        length: flow_length,
        master_id,
        staves,
    };

    let instrument = Instrument {
        id: instrument_id,
        definition_id: String::from("test.instrument"),
        instrument_type: InstrumentType::Melodic,
        long_name: String::from("Test Instrument"),
        short_name: String::from("Test"),
        stave_ids: vec![stave_id],
        count: None,
        assignment_intent: PlaybackAssignmentIntent::default(),
        volume: 100,
        pan: 64,
        mute: false,
        solo: false,
    };

    ParserFixture {
        flow,
        instrument,
        tracks: Tracks { by_id },
        track_id,
    }
}

pub(crate) fn assert_written_duration_scenario(
    name: &str,
    time_signature: TimeSignature,
    tone_specs: &[ToneSpec],
    expectations: Vec<NotationExpectation>,
) {
    assert_written_duration_scenario_with_length(
        name,
        time_signature.clone(),
        time_signature.ticks_per_bar(),
        tone_specs,
        expectations,
    );
}

pub(crate) fn run_written_duration_scenario(
    time_signature: TimeSignature,
    tone_specs: &[ToneSpec],
) {
    let ParserFixture {
        flow,
        instrument,
        tracks,
        track_id: _,
    } = make_parser_fixture(time_signature, tone_specs);
    let instruments = vec![&instrument];
    let measures = get_measures(&flow, &tracks);
    let tone_offsets = get_tone_offsets(&flow, &instruments, &tracks);

    let _ = get_written_durations(&flow, &instruments, &tracks, &measures, &tone_offsets);
}

pub(crate) fn assert_written_duration_scenario_with_length(
    name: &str,
    time_signature: TimeSignature,
    flow_length: Ticks,
    tone_specs: &[ToneSpec],
    expectations: Vec<NotationExpectation>,
) {
    let ParserFixture {
        flow,
        instrument,
        tracks,
        track_id,
    } = make_parser_fixture_with_length(time_signature, flow_length, tone_specs);
    let instruments = vec![&instrument];
    let measures = get_measures(&flow, &tracks);
    let tone_offsets = get_tone_offsets(&flow, &instruments, &tracks);
    let notation = get_written_durations(&flow, &instruments, &tracks, &measures, &tone_offsets);

    assert_eq!(
        normalize_notation_track(&notation[&track_id]),
        expectations,
        "scenario: {name}"
    );
}

pub(crate) fn run_written_duration_scenario_with_tuplets(
    time_signature: TimeSignature,
    flow_length: Ticks,
    tone_specs: &[ToneSpec],
    tuplet_specs: &[TupletSpec],
) -> (
    crate::instructions::parse::get_written_durations::NotationByTrack,
    u64,
) {
    let ParserFixture {
        flow,
        instrument,
        tracks,
        track_id,
    } = make_parser_fixture_with_length_and_tuplets(
        time_signature,
        flow_length,
        tone_specs,
        tuplet_specs,
    );
    let instruments = vec![&instrument];
    let measures = get_measures(&flow, &tracks);
    let tone_offsets = get_tone_offsets(&flow, &instruments, &tracks);
    let notation = get_written_durations(&flow, &instruments, &tracks, &measures, &tone_offsets);

    (notation, track_id)
}

pub(crate) fn run_written_duration_scenario_with_tuplets_and_barlines(
    time_signature: TimeSignature,
    flow_length: Ticks,
    tone_specs: &[ToneSpec],
    tuplet_specs: &[TupletSpec],
    barline_ticks: &[Tick],
) -> (
    crate::instructions::parse::get_written_durations::NotationByTrack,
    u64,
) {
    let ParserFixture {
        flow,
        instrument,
        tracks,
        track_id,
    } = make_parser_fixture_with_length_and_tuplets_and_barlines(
        time_signature,
        flow_length,
        tone_specs,
        tuplet_specs,
        barline_ticks,
    );
    let instruments = vec![&instrument];
    let measures = get_measures(&flow, &tracks);
    let tone_offsets = get_tone_offsets(&flow, &instruments, &tracks);
    let notation = get_written_durations(&flow, &instruments, &tracks, &measures, &tone_offsets);

    (notation, track_id)
}

pub(crate) fn assert_written_duration_scenarios(scenarios: Vec<Scenario>) {
    for scenario in scenarios {
        assert_written_duration_scenario_with_length(
            scenario.name,
            scenario.time_signature,
            scenario.flow_length,
            &scenario.tone_specs,
            scenario.expectations,
        );
    }
}
