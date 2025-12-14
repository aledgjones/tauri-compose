use super::get_beams::Beams;
use super::get_measures::Measures;
use super::get_shunts::Shunt;
use super::get_shunts::Shunts;
use super::get_tone_offset_info::get_tone_offset_info;
use super::get_tone_offset_info::ToneOffsetInfo;
use super::get_tone_offsets::ToneVerticalOffsets;
use crate::renderer::converter::Space;
use crate::score::components::duration::Direction;
use crate::score::components::duration::NoteDuration;
use crate::score::components::duration::Tick;
use crate::score::components::duration::Ticks;
use crate::score::components::duration::NOTE_DURATIONS;
use crate::score::components::measurements::BoundingBox;
use crate::score::engraves::Engrave;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::entries::time_signature::TimeSignatureDrawType;
use crate::score::entries::time_signature::TimeSignatureType;
use crate::score::entries::tone::Tone;
use crate::score::entries::Entry;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Track;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use std::collections::BTreeSet;

pub type NotationByTrack = FxHashMap<UniqueId, NotationTrack>;

#[derive(Clone)]
pub struct Notation {
    pub tick: Tick,
    pub tones: Vec<Tone>,
    pub duration: Ticks,
    pub ties: FxHashSet<UniqueId>,
    pub info: Option<ToneOffsetInfo>,
}

impl Notation {
    pub fn new(tick: Tick, duration: Ticks) -> Notation {
        Notation {
            tick,
            tones: Vec::new(),
            duration,
            ties: FxHashSet::default(),
            info: None,
        }
    }

    pub fn glyph(&self, subdivisions: Ticks) -> &str {
        if self.is_rest() {
            match self.base_to_note_duration(subdivisions) {
                Some(base) => match base {
                    NoteDuration::Whole => "\u{E4E3}",
                    NoteDuration::Half => "\u{E4E4}",
                    NoteDuration::Quarter => "\u{E4E5}",
                    NoteDuration::Eighth => "\u{E4E6}",
                    NoteDuration::Sixteenth => "\u{E4E7}",
                    NoteDuration::ThirtySecond => "\u{E4E8}",
                    NoteDuration::SixtyFourth => todo!(),
                    NoteDuration::HudredTwentyEighth => todo!(),
                },
                None => "\u{E4E5}",
            }
        } else {
            match self.base_to_note_duration(subdivisions) {
                Some(base) => match base {
                    NoteDuration::Whole => "\u{E0A4}",
                    NoteDuration::Half => "\u{E0A3}",
                    NoteDuration::Quarter => "\u{E0A4}",
                    NoteDuration::Eighth => "\u{E0A4}",
                    NoteDuration::Sixteenth => "\u{E0A4}",
                    NoteDuration::ThirtySecond => "\u{E0A4}",
                    NoteDuration::SixtyFourth => "\u{E0A4}",
                    NoteDuration::HudredTwentyEighth => "\u{E0A4}",
                },
                None => "\u{E0A4}",
            }
        }
    }

    // TODO: make this more accurate for whole notes
    pub fn glyph_metrics(&self) -> BoundingBox {
        if self.is_rest() {
            BoundingBox {
                width: 1.175,
                height: 4.0,
            }
        } else {
            BoundingBox {
                width: 1.175,
                height: 4.0,
            }
        }
    }

    pub fn base_duration(&self, subdivisions: Ticks) -> Option<Ticks> {
        let base = (self.duration as f32 / 3.0) * 2.0;
        if base.fract() == 0.0 && Notation::is_writable(base as Ticks, subdivisions) {
            Some(base as Ticks)
        } else {
            None
        }
    }

    pub fn is_writable(duration: Ticks, subdivisions: Ticks) -> bool {
        for option in NOTE_DURATIONS {
            if duration == option.to_ticks(subdivisions) {
                return true;
            }
        }
        false
    }

    // gets base duration from a possibly dotted duration
    pub fn base_to_ticks(&self, subdivisions: Ticks) -> Option<Ticks> {
        if Notation::is_writable(self.duration, subdivisions) {
            Some(self.duration)
        } else {
            self.base_duration(subdivisions)
        }
    }

    pub fn base_to_note_duration(&self, subdivisions: Ticks) -> Option<NoteDuration> {
        match self.base_to_ticks(subdivisions) {
            Some(base) => NoteDuration::from_ticks(base, subdivisions),
            None => None,
        }
    }

    pub fn is_rest(&self) -> bool {
        self.tones.is_empty()
    }

    pub fn is_dotted(&self, subdivisions: Ticks) -> bool {
        !Notation::is_writable(self.duration, subdivisions)
            && self.base_duration(subdivisions).is_some()
    }

    pub fn is_tied(&self) -> bool {
        !self.ties.is_empty()
    }

    pub fn has_pre_shunt(&self, shunts: &Shunts) -> bool {
        for tone in &self.tones {
            if let Shunt::Pre = shunts.by_id[&(self.tick, tone.id)] {
                return true;
            }
        }

        false
    }

    pub fn has_post_shunt(&self, shunts: &Shunts) -> bool {
        for tone in &self.tones {
            if let Shunt::Post = shunts.by_id[&(self.tick, tone.id)] {
                return true;
            }
        }

        false
    }

    pub fn sort_tones_by_offset(&mut self, tone_offsets: &ToneVerticalOffsets) {
        self.tones.sort_by_key(|tone| tone_offsets[&tone.id]);
        self.tones.reverse();
    }

    pub fn get_beam_guide_note(&self, stem_direction: &Direction) -> i8 {
        let info = self.info.clone().unwrap();

        match stem_direction {
            Direction::Up => info.highest,
            Direction::Down => info.lowest,
        }
    }

    pub fn is_flagged(&self, beams: &Beams, subdivisions: Ticks) -> bool {
        !self.is_rest()
            && !self.has_beam(beams)
            && self.duration < NoteDuration::Quarter.to_ticks(subdivisions)
    }

    pub fn flag_glyph(&self, stem_direction: &Direction, subdivisions: Ticks) -> String {
        let base = self.base_to_note_duration(subdivisions);
        match base {
            Some(duration) => duration.to_flag_glyph(stem_direction).to_string(),
            None => String::new(),
        }
    }

    pub fn has_tie(&self) -> bool {
        !self.ties.is_empty()
    }

    pub fn is_chord(&self) -> bool {
        self.tones.len() > 1
    }

    pub fn has_beam(&self, beams: &Beams) -> bool {
        for beam in beams {
            if beam.ticks.contains_key(&self.tick) {
                return true;
            }
        }

        false
    }

    pub fn min_spacing(
        &self,
        shunts: &Shunts,
        subdivisions: Ticks,
        engrave: &Engrave,
        beams: &Beams,
    ) -> Space {
        let mut min_space = self.glyph_metrics().width;
        if self.has_post_shunt(shunts) {
            min_space *= 2.0;
        }

        if self.is_dotted(subdivisions) || self.is_flagged(beams, subdivisions) {
            min_space += 1.0
        }

        // only apply min tie space if internal ties (chords)
        if self.has_tie() && self.is_chord() {
            min_space += engrave.minimum_tie_space;
        } else {
            min_space += engrave.minimum_note_space;
        };

        min_space
    }

    pub fn metrics(
        &self,
        shunts: &Shunts,
        subdivisions: Ticks,
        engrave: &Engrave,
        beams: &Beams,
    ) -> BoundingBox {
        let min = self.min_spacing(shunts, subdivisions, engrave, beams);
        let spacing = match self.base_to_note_duration(subdivisions) {
            Some(base) => {
                let space = engrave.base_note_space
                    * base.spacing_ratio(engrave.note_space_ratio, self.is_dotted(subdivisions));
                if space > min {
                    space
                } else {
                    min
                }
            }
            None => min,
        };

        BoundingBox {
            width: spacing,
            height: 1.0,
        }
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

    pub fn get_next_notation(&self, at: Tick) -> Option<(Tick, &Notation)> {
        for tick in at + 1..self.length {
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
                        info: None,
                    },
                );

                self.track.insert(
                    split_at,
                    Notation {
                        tick: split_at,
                        tones,
                        duration: event_at + duration - split_at,
                        ties,
                        info: None,
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
    pub fn is_qmdot_pattern(&self, beat_one: Tick, beat_two: Tick, stop: Tick) -> bool {
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

        true
    }

    /// [m.q] patterns don't split middle
    pub fn is_mdotq_pattern(
        &self,
        ticks_per_time_signature_beat: Ticks,
        beat_one: Tick,
        beat_four: Tick,
        stop: Tick,
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

        // Rests must be split if they extend over the time signature's beat.
        let is_longer_than_beat = beat_four - beat_one > ticks_per_time_signature_beat;
        if is_longer_than_beat && self.is_tick_rest(beat_one) {
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

    pub fn split_simple_unit(
        &mut self,
        flow: &Flow,
        start: Tick,
        stop: Tick,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) {
        // we stop once there are no events in the range
        if self.is_range_empty(start, stop) {
            return;
        }

        match time_signature.beats {
            2 => todo!(),
            3 => todo!(),
            4 => {
                let beat_one = start;
                let beat_two = time_signature.get_tick_at_beat(start, 2, flow.subdivisions);
                let beat_three = time_signature.get_tick_at_beat(start, 3, flow.subdivisions);
                let beat_four = time_signature.get_tick_at_beat(start, 4, flow.subdivisions);

                let half_beat = time_signature.beat_type.half().to_ticks(flow.subdivisions);

                let patterns = [
                    self.is_qhq_pattern(beat_one, beat_two, beat_four, stop),
                    self.is_qmdot_pattern(beat_one, beat_two, stop),
                    self.is_mdotq_pattern(
                        original_time_signature.ticks_per_beat(flow.subdivisions),
                        beat_one,
                        beat_four,
                        stop,
                    ),
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

                self.split_simple_unit(
                    flow,
                    start,
                    beat_three,
                    original_time_signature,
                    &next_unit_time_signature,
                );
                self.split_simple_unit(
                    flow,
                    beat_three,
                    stop,
                    original_time_signature,
                    &next_unit_time_signature,
                );
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
    tone_offsets: &ToneVerticalOffsets,
) -> NotationByTrack {
    let mut entries = NotationByTrack::default();

    let results: Vec<_> = instruments.par_iter().flat_map(|instrument| {
        instrument.stave_ids.par_iter().flat_map(|stave_id| {
            let stave = &flow.staves.by_id[stave_id];
            stave.track_ids.par_iter().map(|track_id| {
                let track = &tracks.by_id[track_id];
                let mut notation = NotationTrack::new(flow.length);
                notation.split_at_events(track, measures);

                // split as measures
                for (tick, time_signature) in measures {
                    match time_signature.kind() {
                        TimeSignatureType::Simple => notation.split_simple_unit(
                            flow,
                            *tick,
                            tick + time_signature.ticks_per_bar(flow.subdivisions),
                            time_signature,
                            time_signature,
                        ),
                        TimeSignatureType::Compound => todo!(),
                        TimeSignatureType::Complex => todo!(),
                        TimeSignatureType::Open => todo!(),
                    }
                }

                // assign tone offset info here so we don't have to pass mutable
                // notation tracks around
                get_tone_offset_info(&mut notation, tone_offsets);

                (track.id, notation)
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>()
    }).collect();

    for (track_id, notation) in results {
        entries.insert(track_id, notation);
    }

    entries
}
