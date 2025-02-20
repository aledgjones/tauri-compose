use crate::score::components::duration::Tick;
use crate::score::components::duration::Ticks;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::entries::time_signature::TimeSignatureDrawType;
use crate::score::entries::tone::Tone;
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Track;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;
use std::collections::BTreeSet;

use super::get_measures::Measures;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

pub type NotationByTrack = FxHashMap<UniqueId, NotationTrack>;

pub struct Notation {
    pub tick: Tick,
    pub tones: Vec<Tone>,
    pub duration: Ticks,
    pub ties: FxHashSet<UniqueId>,
}

impl Notation {
    pub fn new(tick: Tick, duration: Ticks) -> Notation {
        Notation {
            tick,
            tones: Vec::new(),
            duration,
            ties: FxHashSet::default(),
        }
    }

    pub fn is_rest(&self) -> bool {
        self.tones.is_empty()
    }

    pub fn is_tied(&self) -> bool {
        !self.ties.is_empty()
    }
}

pub struct NotationTrack {
    pub length: u64,
    pub track: FxHashMap<Tick, Notation>,
}

impl NotationTrack {
    pub fn new(length: Ticks) -> NotationTrack {
        let track = FxHashMap::default();

        NotationTrack { length, track }
    }

    pub fn is_tick_empty(&self, tick: Tick) -> bool {
        !self.track.contains_key(&tick)
    }

    pub fn is_tick_rest(&self, tick: Tick) -> bool {
        match self.track.get(&tick) {
            Some(entry) => entry.is_rest(),
            None => false,
        }
    }

    pub fn is_tick_tied(&self, tick: Tick) -> bool {
        match self.track.get(&tick) {
            Some(entry) => entry.is_tied(),
            None => false,
        }
    }

    pub fn is_range_empty(&self, start: Tick, stop: Tick) -> bool {
        for tick in start + 1..stop {
            match self.track.contains_key(&tick) {
                true => return false,
                false => continue,
            }
        }

        true
    }

    pub fn get_previous_notation(&self, at: Tick) -> Option<(Tick, &Notation)> {
        for tick in (0..at + 1).rev() {
            match self.track.get(&tick) {
                Some(notation) => return Some((tick, notation)),
                None => continue,
            }
        }

        None
    }

    pub fn split(&mut self, split_at: Tick) {
        if let Some((event_at, notation)) = self.get_previous_notation(split_at) {
            // only split if:
            // 1. split index not already the start of an event.
            // 2. split index is not the end of an event (ie; end of flow);
            if event_at != split_at && split_at != event_at + notation.duration {
                let tones = notation.tones.clone();
                let ties = notation.ties.clone();
                let duration = notation.duration;

                self.track.insert(
                    event_at,
                    Notation {
                        tick: event_at,
                        tones: tones.clone(),
                        duration: split_at - event_at,
                        ties: tones.iter().map(|tone| tone.id).collect(),
                    },
                );

                self.track.insert(
                    split_at,
                    Notation {
                        tick: split_at,
                        tones,
                        duration: event_at + duration - split_at,
                        ties,
                    },
                );
            }
        }
    }

    // [qhq] patterns dont't split middle
    pub fn is_qhq_pattern(
        &self,
        beat_one: Tick,
        beat_two: Tick,
        beat_four: Tick,
        stop: Tick,
    ) -> bool {
        // make sure that there are only events at the correct points in the bar for this pattern
        for tick in beat_one..stop {
            let is_event_expected = tick == beat_one || tick == beat_two || tick == beat_four;
            let is_empty = self.is_tick_empty(tick);

            if is_empty && is_event_expected {
                return false;
            }

            if !is_empty && !is_event_expected {
                return false;
            }
        }

        // if the first or fourth beats are both rests this is not the pattern
        if self.is_tick_rest(beat_one) && self.is_tick_rest(beat_four) {
            return false;
        }

        // if the event that spans the middle of the bar is a rest it is not the pattern
        if self.is_tick_rest(beat_two) {
            return false;
        }

        true
    }

    /// [qm.] patterns don't split middle
    pub fn is_qmdot_pattern(
        &self,
        beat_one: Tick,
        beat_two: Tick,
        stop: Tick,
        is_complete_measure: bool,
    ) -> bool {
        // make sure that there are only events at the correct points in the bar for this pattern
        for tick in beat_one..stop {
            let is_event_expected = tick == beat_one || tick == beat_two;
            let is_empty = self.is_tick_empty(tick);

            if is_empty && is_event_expected {
                return false;
            }

            if !is_empty && !is_event_expected {
                return false;
            }
        }

        // if the second beat is a rest this is not the pattern
        if self.is_tick_rest(beat_two) {
            return false;
        }

        // if the second beat is tied this is not the pattern
        if !is_complete_measure && self.is_tick_tied(beat_two) {
            return false;
        }

        true
    }

    /// [m.q] patterns don't split middle
    pub fn is_mdotq_pattern(
        &self,
        beat_one: Tick,
        beat_four: Tick,
        stop: Tick,
        is_complete_measure: bool,
    ) -> bool {
        // make sure that there are only events at the correct points in the bar for this pattern
        for tick in beat_one..stop {
            let is_event_expected = tick == beat_one || tick == beat_four;
            let is_empty = self.is_tick_empty(tick);

            if is_empty && is_event_expected {
                return false;
            }

            if !is_empty && !is_event_expected {
                return false;
            }
        }

        if !is_complete_measure && self.is_tick_rest(beat_four) {
            return false;
        }

        true
    }

    // [eqqqe] patterns dont't split middle
    pub fn is_eqqqe_pattern(
        &self,
        beat_one: Tick,
        beat_one_half: Tick,
        beat_two_half: Tick,
        beat_three_half: Tick,
        beat_four_half: Tick,
        stop: Tick,
    ) -> bool {
        // make sure that there are only events at the correct points in the bar for this pattern
        for tick in beat_one..stop {
            let is_event_expected = tick == beat_one
                || tick == beat_one_half
                || tick == beat_two_half
                || tick == beat_three_half
                || tick == beat_four_half;
            let is_empty = self.is_tick_empty(tick);

            if is_empty && is_event_expected {
                return false;
            }

            if !is_empty && !is_event_expected {
                return false;
            }
        }

        // inner rhythems must be continuously off-beat
        let patterns = [
            self.is_tick_rest(beat_one_half),
            self.is_tick_rest(beat_two_half),
            self.is_tick_rest(beat_three_half),
        ];

        if patterns.iter().any(|&is_rest| is_rest) {
            return false;
        }

        // first or last half beat can be rest
        if self.is_tick_rest(beat_one) && self.is_tick_rest(beat_four_half) {
            return false;
        }

        true
    }

    pub fn split_at_events(&mut self, track: &Track, measures: &Measures) {
        let mut split_points: BTreeSet<Tick> = BTreeSet::new();

        // measure dictated barline events
        for tick in measures.keys() {
            split_points.insert(*tick);
        }

        // tone / explicit barline events
        for entry in track.entries.by_id.values() {
            match entry {
                Entry::Tone(tone) => {
                    split_points.insert(tone.tick);
                    split_points.insert(tone.tick + tone.duration);
                }
                Entry::Barline(barline) => {
                    split_points.insert(barline.tick);
                }
                _ => {}
            }
        }

        let mut prev_split_point = self.length;

        for &split_point in split_points.iter().rev() {
            let notation = Notation::new(split_point, prev_split_point - split_point);
            self.track.insert(split_point, notation);
            prev_split_point = split_point;
        }

        for entry in track.entries.by_id.values() {
            if let Entry::Tone(tone) = entry {
                let mut is_split = false;
                let start_tick = tone.tick;
                let end_tick = tone.tick + tone.duration;

                for tick in (start_tick..end_tick).rev() {
                    if let Some(notation) = self.track.get_mut(&tick) {
                        notation.tones.push(tone.clone());
                        if is_split {
                            notation.ties.insert(tone.id);
                        }
                        is_split = true
                    }
                }
            }
        }
    }

    pub fn split_unit(
        &mut self,
        flow: &Flow,
        start: Tick,
        stop: Tick,
        time_signature: &TimeSignature,
        is_complete_measure: bool,
    ) {
        // we stop once there are no events in the range
        if self.is_range_empty(start, stop) {
            return;
        }

        // convert 2 beats to 4 at higher fidelity as it follows exactly
        // the same patterns at that level
        if time_signature.beats == 2 {
            return self.split_unit(
                flow,
                start,
                stop,
                &TimeSignature::new(
                    0,
                    time_signature.tick,
                    4,
                    time_signature.beat_type.half(),
                    TimeSignatureDrawType::Regular,
                    None,
                ),
                false,
            );
        }

        match time_signature.beats {
            3 => todo!(),
            4 => {
                let beat_one = start;
                let beat_two = time_signature.get_tick_at_beat(start, 2, flow.subdivisions);
                let beat_three = time_signature.get_tick_at_beat(start, 3, flow.subdivisions);
                let beat_four = time_signature.get_tick_at_beat(start, 4, flow.subdivisions);

                let half_beat = time_signature.beat_type.half().to_ticks(flow.subdivisions);

                let patterns = [
                    self.is_qhq_pattern(beat_one, beat_two, beat_four, stop),
                    self.is_qmdot_pattern(beat_one, beat_two, stop, is_complete_measure),
                    self.is_mdotq_pattern(beat_one, beat_four, stop, is_complete_measure),
                    self.is_eqqqe_pattern(
                        beat_one,
                        beat_one + half_beat,
                        beat_two + half_beat,
                        beat_three + half_beat,
                        beat_four + half_beat,
                        stop,
                    ),
                ];

                if patterns.iter().any(|&pattern_matched| pattern_matched) {
                    println!("{} -- PATTERN MATCH {:?}", start, patterns);
                    // these patterns span the entire unit so we can early return as there is no
                    // higher fidelity that is needed.
                    return;
                } else {
                    // other rhythms split at the middle and we must split the unit further
                    self.split(beat_three);
                }

                let next_unit_time_signature = TimeSignature::new(
                    0,
                    time_signature.tick,
                    4,
                    time_signature.beat_type.half(),
                    TimeSignatureDrawType::Regular,
                    None,
                );

                self.split_unit(flow, start, beat_three, &next_unit_time_signature, false);
                self.split_unit(flow, beat_three, stop, &next_unit_time_signature, false);
            }
            _ => (),
        }
    }
}

pub fn get_written_durations(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
    measures: &Measures,
) -> NotationByTrack {
    let mut entries = NotationByTrack::default();

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];

            for track_id in &stave.track_ids {
                let track = &tracks.by_id[track_id];
                let mut notation = NotationTrack::new(flow.length);
                notation.split_at_events(track, measures);

                // split as measures
                for (tick, time_signature) in measures {
                    notation.split_unit(
                        flow,
                        *tick,
                        tick + time_signature.ticks_per_bar(flow.subdivisions),
                        time_signature,
                        true,
                    );
                }

                entries.insert(track.id, notation);
            }
        }
    }

    entries
}
