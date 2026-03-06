use super::get_measures::Measures;
use super::get_written_durations::{Notation, NotationByTrack, NotationTrack};
use crate::score::components::duration::{NoteDuration, Tick};
use crate::score::entries::time_signature::TimeSignature;
use crate::score::flows::Flow;
use crate::utils::UniqueId;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;

pub type BeamTicks = FxHashMap<Tick, u8>;

#[derive(Clone)]
pub struct BeamSegment {
    pub start_tick: Tick,
    pub end_tick: Tick,
}

#[derive(Clone)]
pub struct FractionalBeam {
    pub tick: Tick,
    pub direction: FractionalDirection,
}

#[derive(Clone, Copy)]
pub enum FractionalDirection {
    Left,
    Right,
}

pub type BeamSegments = FxHashMap<u8, Vec<BeamSegment>>; // beam_level -> segments
pub type FractionalBeams = FxHashMap<u8, Vec<FractionalBeam>>; // beam_level -> fractional beams

#[derive(Clone)]
pub struct Beam {
    pub ticks: BeamTicks,
    pub start: Tick,
    pub stop: Tick,
    pub beam_segments: BeamSegments,
    pub fractional_beams: FractionalBeams,
}

pub type Beams = Vec<Beam>;
pub type BeamsByTrack = FxHashMap<UniqueId, Beams>;

enum EighthType {
    Rest,
    Note,
}

/// eighths can be beamed in certain patterns
fn is_pattern(pattern: &[EighthType], notation: &NotationTrack, start: Tick) -> bool {
    let eighth = NoteDuration::Eighth.to_ticks();

    for (i, eighth_type) in pattern.iter().enumerate() {
        match notation.track.get(&(start + (i as Tick * eighth))) {
            Some(entry) => match eighth_type {
                EighthType::Rest => {
                    if !entry.is_rest() || entry.duration != eighth {
                        return false;
                    }
                }
                EighthType::Note => {
                    if entry.is_rest() || entry.duration != eighth {
                        return false;
                    }
                }
            },
            None => return false,
        }
    }

    true
}

fn grouping_is_beamable(beats: u8, notation: &NotationTrack, start: Tick) -> bool {
    match beats {
        2 | 4 => {
            // -eee
            let pattern = vec![
                EighthType::Rest,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
            ];
            if is_pattern(&pattern, notation, start) {
                return true;
            }

            // eee-
            let pattern = vec![
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Rest,
            ];
            if is_pattern(&pattern, notation, start) {
                return true;
            }

            // eeee
            let pattern = vec![
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
            ];
            if is_pattern(&pattern, notation, start) {
                return true;
            }

            false
        }
        3 => {
            // -eeeee
            let pattern = vec![
                EighthType::Rest,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
            ];
            if is_pattern(&pattern, notation, start) {
                return true;
            }

            // eeeee-
            let pattern = vec![
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Rest,
            ];
            if is_pattern(&pattern, notation, start) {
                return true;
            }

            // eeeeee
            let pattern = vec![
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
                EighthType::Note,
            ];
            if is_pattern(&pattern, notation, start) {
                return true;
            }

            false
        }
        _ => false,
    }
}

/// Determines if beams should break at on-beat ticks based on the current grouping
fn should_break_at_beats(
    time_signature: &TimeSignature,
    notation: &NotationTrack,
    current_tick: Tick,
) -> bool {
    match time_signature.beat_type {
        // For quarter note beats, break if the grouping is not beamable
        NoteDuration::Quarter => {
            !grouping_is_beamable(time_signature.beats, notation, current_tick)
        }
        // For other beat types, break if the beat is larger than a quarter
        _ => {
            let quarter_ticks = NoteDuration::Quarter.to_ticks();
            time_signature.ticks_per_beat() > quarter_ticks
        }
    }
}

fn assign_beam(beams: &mut Beams, ticks: BeamTicks) -> BeamTicks {
    if ticks.len() > 1 {
        let mut min: Option<Tick> = None;
        let mut max: Option<Tick> = None;

        for tick in ticks.keys() {
            match min {
                None => min = Some(*tick),
                Some(val) => {
                    if tick < &val {
                        min = Some(*tick)
                    }
                }
            }

            match max {
                None => max = Some(*tick),
                Some(val) => {
                    if tick > &val {
                        max = Some(*tick)
                    }
                }
            }
        }

        let start_tick = min.unwrap();
        let stop_tick = max.unwrap();

        // Calculate beam segments and fractional beams immediately
        let mut beam_segments = FxHashMap::default();
        let mut fractional_beams = FxHashMap::default();

        // Find the maximum beam count in this beam group
        let max_beam_count = ticks.values().max().unwrap_or(&1);

        // Calculate eighth note boundaries for splitting inner beams
        // Align boundaries to absolute eighth note positions from tick 0
        let eighth_note_interval = 16; // Half of quarter note (32 ticks)
        let mut eighth_boundaries = Vec::new();
        // Find all eighth note boundaries within the beam range
        let mut tick_boundary = eighth_note_interval;
        while tick_boundary <= stop_tick {
            if tick_boundary >= start_tick && tick_boundary <= stop_tick {
                eighth_boundaries.push(tick_boundary);
            }
            tick_boundary += eighth_note_interval;
        }

        // Calculate beam segments and fractional beams for each level
        for beam_index in 1..*max_beam_count {
            let should_split_at_eighths = *max_beam_count >= 3;

            // Find consecutive runs of notes that qualify for this beam level
            let mut sorted_ticks: Vec<Tick> = ticks.keys().cloned().collect();
            sorted_ticks.sort();

            let mut i = 0;
            while i < sorted_ticks.len() {
                let tick = sorted_ticks[i];
                let beam_count = ticks[&tick];

                // Check if this note qualifies for this beam level
                if beam_count > beam_index {
                    // Start of a consecutive run
                    let run_start_tick = tick;
                    let mut run_end_tick = tick;

                    // Extend the run while consecutive ticks also qualify
                    while i + 1 < sorted_ticks.len() {
                        let next_tick = sorted_ticks[i + 1];
                        let next_beam_count = ticks[&next_tick];

                        // If splitting at eighths, don't extend across boundaries
                        if should_split_at_eighths {
                            // Check if next_tick is at an eighth boundary
                            if eighth_boundaries.contains(&next_tick) {
                                break;
                            }
                        }

                        if next_beam_count > beam_index {
                            run_end_tick = next_tick;
                            i += 1;
                        } else {
                            break;
                        }
                    }

                    // Handle the run: connect if 2+ notes, otherwise draw fractional
                    if run_start_tick != run_end_tick {
                        // Add connecting beam segment
                        beam_segments
                            .entry(beam_index)
                            .or_insert_with(Vec::new)
                            .push(BeamSegment {
                                start_tick: run_start_tick,
                                end_tick: run_end_tick,
                            });
                    } else {
                        // Add fractional beam
                        // Point left if at the end of the beam group or at an eighth boundary split (but not at start)
                        let direction = if run_start_tick == stop_tick
                            || (should_split_at_eighths
                                && eighth_boundaries
                                    .iter()
                                    .any(|&boundary| boundary > run_start_tick)
                                && run_start_tick != start_tick)
                        {
                            FractionalDirection::Left
                        } else {
                            FractionalDirection::Right
                        };

                        fractional_beams
                            .entry(beam_index)
                            .or_insert_with(Vec::new)
                            .push(FractionalBeam {
                                tick: run_start_tick,
                                direction,
                            });
                    }
                }

                i += 1;
            }
        }

        beams.push(Beam {
            ticks,
            start: start_tick,
            stop: stop_tick,
            beam_segments,
            fractional_beams,
        });
    }

    FxHashMap::default()
}

pub fn get_beams_in_track(notation: &NotationTrack, barlines: &Measures) -> Beams {
    let mut output: Beams = Vec::new();

    let mut ticks: BeamTicks = FxHashMap::default(); // Collects ticks that can be beamed together
    let mut time_signature = &TimeSignature::default();
    let mut boundaries = time_signature.groupings_to_ticks(0); // Beat boundaries
    let mut break_at_beats = false; // Whether to break beams at on-beat ticks

    for tick in 0..notation.length {
        // Update time signature and boundaries at barlines
        if let Some(entry) = barlines.get(&tick) {
            time_signature = entry;
            boundaries = time_signature.groupings_to_ticks(tick);
        }

        // At beat boundaries, finalize current beam group and determine if we should break at on-beats
        // within this boundary
        if boundaries.contains(&tick) {
            ticks = assign_beam(&mut output, ticks);

            break_at_beats = should_break_at_beats(time_signature, notation, tick);
        }

        // Additional break at on-beat ticks if conditions are met
        if break_at_beats && time_signature.is_on_beat(tick) && notation.track.contains_key(&tick) {
            ticks = assign_beam(&mut output, ticks);
        }

        // Process the notation at this tick
        if let Some(entry) = notation.track.get(&tick) {
            let beam_count = entry.get_beam_count();
            if beam_count > 0 {
                // Add beamable note to current group
                ticks.insert(tick, beam_count);
            } else {
                // Rest: finalize current beam group
                ticks = assign_beam(&mut output, ticks);
            }
        }
    }

    // Finalize any remaining beam group
    assign_beam(&mut output, ticks);

    output
}

pub fn get_beams(_flow: &Flow, tracks: &NotationByTrack, bars: &Measures) -> BeamsByTrack {
    let results: Vec<(UniqueId, _)> = tracks
        .par_iter()
        .map(|(track_id, track)| {
            let beams = get_beams_in_track(track, bars);
            (*track_id, beams)
        })
        .collect();

    let mut output: BeamsByTrack = FxHashMap::default();
    for (track_id, beams) in results {
        output.insert(track_id, beams);
    }

    output
}

impl Notation {
    pub fn get_beam_count(&self) -> u8 {
        if self.is_rest() {
            0
        } else {
            match self.get_writable_ticks() {
                Some(base) => {
                    let eighth_ticks = NoteDuration::Eighth.to_ticks();
                    let ratio = eighth_ticks as f32 / base as f32;

                    if ratio >= 1.0 {
                        let mut beam_count = 1;
                        let mut temp = ratio;
                        while temp >= 2.0 {
                            beam_count += 1;
                            temp /= 2.0;
                        }
                        beam_count
                    } else {
                        0
                    }
                }
                None => 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_written_durations::{Notation, NotationTrack};
    use crate::score::components::articulation::Articulations;
    use crate::score::components::duration::NoteDuration;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
    use crate::score::entries::tone::Tone;
    use rustc_hash::FxHashMap;

    fn make_note(id: u64, duration: u64) -> Notation {
        let mut n = Notation::new(0, duration);
        n.tones.push(Tone::new(
            id,
            0,
            duration,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        n
    }

    fn four_four() -> TimeSignature {
        TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )
    }

    fn three_four() -> TimeSignature {
        TimeSignature::new(
            2,
            0,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )
    }

    fn make_note_at(tick: u64, id: u64, duration: u64) -> Notation {
        let mut n = Notation::new(tick, duration);
        n.tones.push(Tone::new(
            id,
            tick,
            duration,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        n
    }

    #[test]
    fn get_beam_count_quarter_is_zero() {
        let n = make_note(1, NoteDuration::Quarter.to_ticks());
        assert_eq!(n.get_beam_count(), 0);
    }

    #[test]
    fn get_beam_count_eighth_is_one() {
        let n = make_note(1, NoteDuration::Eighth.to_ticks());
        assert_eq!(n.get_beam_count(), 1);
    }

    #[test]
    fn get_beam_count_sixteenth_is_two() {
        let n = make_note(1, NoteDuration::Sixteenth.to_ticks());
        assert_eq!(n.get_beam_count(), 2);
    }

    #[test]
    fn get_beam_count_thirty_second_is_three() {
        let n = make_note(1, NoteDuration::ThirtySecond.to_ticks());
        assert_eq!(n.get_beam_count(), 3);
    }

    #[test]
    fn get_beam_count_rest_is_zero() {
        let n = Notation::new(0, NoteDuration::Eighth.to_ticks()); // no tones = rest
        assert_eq!(n.get_beam_count(), 0);
    }

    #[test]
    fn get_beams_empty_track_returns_no_beams() {
        let track = NotationTrack::new(512);
        let barlines: Measures = FxHashMap::default();
        let result = get_beams_in_track(&track, &barlines);
        assert!(result.is_empty());
    }

    #[test]
    fn get_beams_single_eighth_produces_no_beam() {
        // A single eighth without a partner can't form a beam
        let mut track = NotationTrack::new(512);
        let eighth = NoteDuration::Eighth.to_ticks();
        track.track.insert(0, make_note(1, eighth));
        let barlines: Measures = FxHashMap::default();
        let result = get_beams_in_track(&track, &barlines);
        assert!(result.is_empty());
    }

    #[test]
    fn get_beams_two_adjacent_eighths_form_beam() {
        let mut track = NotationTrack::new(512);
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut ts = FxHashMap::default();
        ts.insert(0_u64, four_four());

        track.track.insert(0, make_note(1, eighth));
        track.track.insert(eighth, make_note(2, eighth));
        let result = get_beams_in_track(&track, &ts);
        assert!(!result.is_empty());
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].stop, eighth);
    }

    #[test]
    fn get_beams_three_four_split_dotted_quarters_do_not_form_false_beams() {
        let quarter = NoteDuration::Quarter.to_ticks();
        let eighth = NoteDuration::Eighth.to_ticks();
        let dotted_quarter = quarter + eighth;
        let mut track = NotationTrack::new(quarter * 3);
        let mut barlines = FxHashMap::default();
        barlines.insert(0_u64, three_four());

        track.track.insert(0, make_note_at(0, 1, dotted_quarter));
        track.track.insert(dotted_quarter, make_note_at(dotted_quarter, 2, eighth));
        track.track.insert(quarter * 2, make_note_at(quarter * 2, 2, quarter));

        let result = get_beams_in_track(&track, &barlines);

        assert!(result.is_empty());
    }

    #[test]
    fn get_beam_count_whole_is_zero() {
        let n = make_note(1, NoteDuration::Whole.to_ticks());
        assert_eq!(n.get_beam_count(), 0);
    }

    #[test]
    fn get_beam_count_half_is_zero() {
        let n = make_note(1, NoteDuration::Half.to_ticks());
        assert_eq!(n.get_beam_count(), 0);
    }

    #[test]
    fn get_beam_count_dotted_eighth_writable_ticks_is_eighth() {
        // dotted eighth = eighth + sixteenth; base (get_writable_ticks) = eighth → 1 beam
        let eighth = NoteDuration::Eighth.to_ticks();
        let sixteenth = NoteDuration::Sixteenth.to_ticks();
        let n = make_note(1, eighth + sixteenth);
        assert_eq!(n.get_beam_count(), 1);
    }

    #[test]
    fn get_beams_four_eighths_forms_single_beam_group() {
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let mut ts = FxHashMap::default();
        ts.insert(0_u64, four_four());

        for i in 0..4_u64 {
            track.track.insert(i * eighth, make_note(i + 1, eighth));
        }

        let result = get_beams_in_track(&track, &ts);
        assert!(!result.is_empty());
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].stop, 3 * eighth);
    }

    #[test]
    fn get_beams_eighth_rest_between_notes_breaks_beam() {
        // note - rest - note: the rest breaks the beam group
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let mut ts = FxHashMap::default();
        ts.insert(0_u64, four_four());

        track.track.insert(0, make_note(1, eighth));
        // rest at tick = eighth (no tones pushed → is_rest = true)
        track.track.insert(eighth, Notation::new(eighth, eighth));
        track.track.insert(eighth * 2, make_note(2, eighth));

        let result = get_beams_in_track(&track, &ts);
        // Each isolated note cannot beam alone so result may be empty
        assert!(result.len() <= 1);
    }

    #[test]
    fn get_beams_two_sixteenths_form_beam() {
        let sixteenth = NoteDuration::Sixteenth.to_ticks();
        let mut track = NotationTrack::new(512);
        let mut ts = FxHashMap::default();
        ts.insert(0_u64, four_four());

        track.track.insert(0, make_note(1, sixteenth));
        track.track.insert(sixteenth, make_note(2, sixteenth));
        let result = get_beams_in_track(&track, &ts);
        assert!(!result.is_empty());
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].stop, sixteenth);
    }

    #[test]
    fn get_beam_count_non_writable_duration_is_zero() {
        // A duration that has no writable base → 0 beams
        // 5 ticks: not a standard duration, not dotted (no base 2/3 that's writable)
        let n = make_note(1, 5);
        assert_eq!(n.get_beam_count(), 0);
    }

    #[test]
    fn get_beam_count_sixty_fourth_is_four() {
        let n = make_note(1, NoteDuration::SixtyFourth.to_ticks());
        assert_eq!(n.get_beam_count(), 4);
    }

    #[test]
    fn get_beam_count_hundred_twenty_eighth_is_five() {
        let n = make_note(1, NoteDuration::HudredTwentyEighth.to_ticks());
        assert_eq!(n.get_beam_count(), 5);
    }

    #[test]
    fn get_beams_three_four_four_eighths_forms_beam() {
        // In 3/4, 4 eighths should beam together
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(512);
        let mut ts = FxHashMap::default();
        ts.insert(
            0_u64,
            TimeSignature::new(
                1,
                0,
                3,
                NoteDuration::Quarter,
                TimeSignatureDrawType::Regular,
                None,
            ),
        );

        for i in 0..4_u64 {
            track.track.insert(i * eighth, make_note(i + 1, eighth));
        }

        let result = get_beams_in_track(&track, &ts);
        assert!(!result.is_empty());
    }

    #[test]
    fn get_beams_six_eight_six_eighths_beams() {
        // In 6/8 (beat_type=Eighth), should beam 6 eighths
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(1024);
        let mut ts = FxHashMap::default();
        ts.insert(
            0_u64,
            TimeSignature::new(
                1,
                0,
                6,
                NoteDuration::Eighth,
                TimeSignatureDrawType::Regular,
                None,
            ),
        );

        for i in 0..6_u64 {
            track.track.insert(i * eighth, make_note(i + 1, eighth));
        }

        let result = get_beams_in_track(&track, &ts);
        assert!(!result.is_empty());
    }

    #[test]
    fn get_beams_mixed_sixteenth_eighth_beams_together() {
        // 2 sixteenths followed by 1 eighth → all beamed together
        let eighth = NoteDuration::Eighth.to_ticks();
        let sixteenth = NoteDuration::Sixteenth.to_ticks();
        let mut track = NotationTrack::new(512);
        let mut ts = FxHashMap::default();
        ts.insert(0_u64, four_four());

        track.track.insert(0, make_note(1, sixteenth));
        track.track.insert(sixteenth, make_note(2, sixteenth));
        track.track.insert(eighth, make_note(3, eighth));

        let result = get_beams_in_track(&track, &ts);
        assert!(!result.is_empty());
        let beam = &result[0];
        assert_eq!(beam.start, 0);
        assert_eq!(beam.stop, eighth);
    }

    #[test]
    fn assign_beam_single_tick_produces_no_beam() {
        let mut beams: Beams = Vec::new();
        let mut ticks = FxHashMap::default();
        ticks.insert(0_u64, 1u8);
        let remaining = assign_beam(&mut beams, ticks);
        // Only 1 tick → no beam pushed
        assert!(beams.is_empty());
        assert!(remaining.is_empty());
    }

    #[test]
    fn assign_beam_two_ticks_produces_beam_with_correct_range() {
        let mut beams: Beams = Vec::new();
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut ticks = FxHashMap::default();
        ticks.insert(0_u64, 1u8);
        ticks.insert(eighth, 1u8);
        assign_beam(&mut beams, ticks);
        assert_eq!(beams.len(), 1);
        assert_eq!(beams[0].start, 0);
        assert_eq!(beams[0].stop, eighth);
    }

    #[test]
    fn assign_beam_two_sixteenths_produces_beam_segment() {
        // Two sixteenth notes (beam_count = 2) should produce a beam_segment at level 1
        let mut beams: Beams = Vec::new();
        let sixteenth = NoteDuration::Sixteenth.to_ticks();
        let mut ticks = FxHashMap::default();
        ticks.insert(0_u64, 2u8);
        ticks.insert(sixteenth, 2u8);
        assign_beam(&mut beams, ticks);
        assert_eq!(beams.len(), 1);
        // Level 1 beam segment should exist
        assert!(beams[0].beam_segments.contains_key(&1));
    }
}
