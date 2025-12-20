use super::get_measures::Measures;
use super::get_written_durations::{Notation, NotationByTrack, NotationTrack};
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
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
fn is_pattern(
    pattern: &[EighthType],
    notation: &NotationTrack,
    start: Tick,
    subdivisions: Ticks,
) -> bool {
    let eighth = NoteDuration::Eighth.to_ticks(subdivisions);

    for (i, eighth_type) in pattern.iter().enumerate() {
        match notation.track.get(&(start + (i as Tick * eighth))) {
            Some(entry) => match eighth_type {
                EighthType::Rest => {
                    if !(entry.is_rest() && entry.duration == eighth) {
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
        };
    }

    true
}

fn grouping_is_beamable(
    beats: u8,
    notation: &NotationTrack,
    start: Tick,
    stop: Tick,
    subdivisions: Ticks,
) -> bool {
    // beat specific patterns
    if let 4 = beats {
        // -eee
        let pattern = vec![
            EighthType::Rest,
            EighthType::Note,
            EighthType::Note,
            EighthType::Note,
        ];
        if is_pattern(&pattern, notation, start, subdivisions) {
            return true;
        }

        // eee-
        let pattern = vec![
            EighthType::Note,
            EighthType::Note,
            EighthType::Note,
            EighthType::Rest,
        ];
        if is_pattern(&pattern, notation, start, subdivisions) {
            return true;
        }
    }

    let sixteenth = NoteDuration::Sixteenth.to_ticks(subdivisions);
    for tick in start..stop {
        if let Some(entry) = notation.track.get(&tick) {
            // TODO: make this more sophisticated, -eee | eee- can be beamed -ee- cannot, for example
            // for now just bail out if there are rests
            if entry.is_rest() || entry.duration <= sixteenth {
                return false;
            }
        }
    }

    true
}

fn assign_beam(beams: &mut Beams, ticks: BeamTicks, subdivisions: Ticks) -> BeamTicks {
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
        let eighth_note_interval = subdivisions / 2;
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
                                && eighth_boundaries.contains(&run_start_tick)
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

pub fn get_beams_in_track(
    notation: &NotationTrack,
    barlines: &Measures,
    subdivisions: Ticks,
) -> Beams {
    let mut output: Beams = Vec::new();

    let mut ticks: BeamTicks = FxHashMap::default();
    let mut time_signature = &TimeSignature::default();
    let mut boundries = time_signature.groupings_to_ticks(0, subdivisions);
    let mut break_at_beats = false;

    for tick in 0..notation.length {
        if let Some(entry) = barlines.get(&tick) {
            time_signature = entry;
            boundries = time_signature.groupings_to_ticks(tick, subdivisions);
        }

        if boundries.contains(&tick) {
            ticks = assign_beam(&mut output, ticks, subdivisions);

            break_at_beats = match time_signature.beat_type {
                // quarters break if rhythm has durations < sixteenth
                NoteDuration::Quarter => {
                    let i = boundries.iter().position(|entry| entry == &tick).unwrap();
                    let stop = boundries[i + 1];
                    !grouping_is_beamable(time_signature.beats, notation, tick, stop, subdivisions)
                }
                // larger beats break, smaller don't
                _ => {
                    let quarter = NoteDuration::Quarter.to_ticks(subdivisions);
                    time_signature.ticks_per_beat(subdivisions) > quarter
                }
            }
        }

        if break_at_beats && time_signature.is_on_beat(tick, subdivisions) {
            ticks = assign_beam(&mut output, ticks, subdivisions);
        }

        if let Some(entry) = notation.track.get(&tick) {
            let count = entry.get_beam_count(subdivisions);
            if count > 0 {
                ticks.insert(tick, count);
            } else {
                ticks = assign_beam(&mut output, ticks, subdivisions);
            }
        }
    }

    assign_beam(&mut output, ticks, subdivisions);

    output
}

pub fn get_beams(flow: &Flow, tracks: &NotationByTrack, bars: &Measures) -> BeamsByTrack {
    let results: Vec<(UniqueId, _)> = tracks
        .par_iter()
        .map(|(track_id, track)| {
            let beams = get_beams_in_track(track, bars, flow.subdivisions);
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
    pub fn get_beam_count(&self, subdivisions: Ticks) -> u8 {
        if self.is_rest() {
            0
        } else {
            match self.base_to_ticks(subdivisions) {
                Some(base) => {
                    let eighth_ticks = NoteDuration::Eighth.to_ticks(subdivisions);
                    let ratio = eighth_ticks / base;

                    let mut beam_count = 1;
                    let mut temp = ratio;
                    while temp > 1 {
                        beam_count += 1;
                        temp /= 2;
                    }
                    beam_count
                }
                None => 0,
            }
        }
    }
}
