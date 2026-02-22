use super::get_beams::Beams;
use super::get_measures::Measures;
use super::get_shunts::Shunt;
use super::get_shunts::Shunts;
use super::get_tone_offset_info::get_tone_offset_info;
use super::get_tone_offset_info::ToneOffsetInfo;
use super::get_tone_offsets::ToneVerticalOffsets;
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

    pub fn glyph(&self) -> &str {
        if self.is_rest() {
            self.rest_glyph()
        } else {
            self.note_glyph()
        }
    }

    pub fn note_glyph(&self) -> &str {
        match self.base_to_note_duration() {
            Some(base) => match base {
                NoteDuration::Whole => "\u{E0A2}",
                NoteDuration::Half => "\u{E0A3}",
                NoteDuration::Quarter => "\u{E0A4}",
                NoteDuration::Eighth => "\u{E0A4}",
                NoteDuration::Sixteenth => "\u{E0A4}",
                NoteDuration::ThirtySecond => "\u{E0A4}",
                NoteDuration::SixtyFourth => "\u{E0A4}",
                NoteDuration::HudredTwentyEighth => "\u{E0A4}",
            },
            None => "\u{E0A4}", // Default quarter note
        }
    }

    pub fn rest_glyph(&self) -> &str {
        match self.base_to_note_duration() {
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
            None => "\u{E4E5}", // Default quarter rest
        }
    }

    pub fn glyph_metrics(&self) -> BoundingBox {
        let is_whole_note = self.base_to_note_duration() == Some(NoteDuration::Whole);

        if self.is_rest() {
            BoundingBox {
                width: 1.175,
                height: 4.0,
                right_padding: 0.0,
            }
        } else if is_whole_note {
            BoundingBox {
                width: 1.7,
                height: 4.0,
                right_padding: 0.0,
            }
        } else {
            BoundingBox {
                width: 1.175,
                height: 4.0,
                right_padding: 0.0,
            }
        }
    }

    pub fn base_duration(&self) -> Option<Ticks> {
        let base = (self.duration as f32 / 3.0) * 2.0;
        if base.fract() == 0.0 && Notation::is_writable(base as Ticks) {
            Some(base as Ticks)
        } else {
            None
        }
    }

    pub fn is_writable(duration: Ticks) -> bool {
        for option in NOTE_DURATIONS {
            if duration == option.to_ticks() {
                return true;
            }
        }

        false
    }

    // gets base duration from a possibly dotted duration
    pub fn get_writable_ticks(&self) -> Option<Ticks> {
        if Notation::is_writable(self.duration) {
            Some(self.duration)
        } else {
            self.base_duration()
        }
    }

    pub fn base_to_note_duration(&self) -> Option<NoteDuration> {
        match self.get_writable_ticks() {
            Some(base) => NoteDuration::from_ticks(base),
            None => None,
        }
    }

    pub fn is_rest(&self) -> bool {
        self.tones.is_empty()
    }

    pub fn is_dotted(&self) -> bool {
        !Notation::is_writable(self.duration) && self.base_duration().is_some()
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

    pub fn is_flagged(&self, beams: &Beams) -> bool {
        !self.is_rest() && !self.has_beam(beams) && self.duration < NoteDuration::Quarter.to_ticks()
    }

    pub fn flag_glyph(&self, stem_direction: &Direction) -> String {
        let base = self.base_to_note_duration();
        match base {
            Some(duration) => match stem_direction {
                Direction::Up => match duration {
                    NoteDuration::Eighth => "\u{E240}".to_string(),
                    NoteDuration::Sixteenth => "\u{E242}".to_string(),
                    NoteDuration::ThirtySecond => "\u{E244}".to_string(),
                    NoteDuration::SixtyFourth => "\u{E246}".to_string(),
                    NoteDuration::HudredTwentyEighth => "\u{E248}".to_string(),
                    _ => String::new(),
                },
                Direction::Down => match duration {
                    NoteDuration::Eighth => "\u{E241}".to_string(),
                    NoteDuration::Sixteenth => "\u{E243}".to_string(),
                    NoteDuration::ThirtySecond => "\u{E245}".to_string(),
                    NoteDuration::SixtyFourth => "\u{E247}".to_string(),
                    NoteDuration::HudredTwentyEighth => "\u{E249}".to_string(),
                    _ => String::new(),
                },
            },
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

    pub fn min_spacing(&self, shunts: &Shunts, engrave: &Engrave, beams: &Beams) -> BoundingBox {
        let metrics = self.glyph_metrics();
        let glyph_width = metrics.total_width();

        let mut avoid_collisions_spacing = glyph_width;

        if self.has_post_shunt(shunts) {
            avoid_collisions_spacing += glyph_width;
        } else if self.is_flagged(beams) {
            avoid_collisions_spacing += 0.5;
        }

        // only apply min tie space if internal ties (chords)
        let mut spacing = if self.has_tie() && self.is_chord() {
            engrave.minimum_tie_space
        } else {
            engrave.minimum_note_space
        };

        if self.is_dotted() {
            // avoid the dot
            avoid_collisions_spacing += 0.2;
            // extra spacing which can be eaten into.
            spacing += 0.4;
        }

        BoundingBox {
            width: avoid_collisions_spacing,
            height: 1.0,
            right_padding: spacing,
        }
    }

    pub fn metrics(&self, shunts: &Shunts, engrave: &Engrave, beams: &Beams) -> BoundingBox {
        let min = self.min_spacing(shunts, engrave, beams);
        let spacing = match self.base_to_note_duration() {
            Some(base) => {
                let space = engrave.base_note_space
                    * base.spacing_ratio(engrave.note_space_ratio, self.is_dotted());
                if space > min.total_width() {
                    space
                } else {
                    min.total_width()
                }
            }
            None => min.total_width(),
        };

        BoundingBox {
            width: spacing,
            height: 1.0,
            right_padding: 0.0,
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
        ticks_per_original_beat: Ticks,
        beat_one: Tick,
        beat_three: Tick,
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

        // If the pattern starts with a dotted note of the original time signature's beat length, it's only
        // the pattern if it is followed by a note.
        if beat_three - beat_one == ticks_per_original_beat && self.is_tick_rest(beat_four) {
            return false;
        }

        // If the unit is longer than a beat, and the first beat is a rest, it's not the pattern
        if self.is_tick_rest(beat_one) && stop - beat_one > ticks_per_original_beat {
            return false;
        };

        true
    }

    // [e.g eqqqe] offbeats don't split if the pattern spans the whole unit
    pub fn is_offbeat_pattern(&self, start: Tick, stop: Tick, whole_beat: Ticks) -> bool {
        let half_beat = whole_beat / 2;

        // first and last positions cannot both be rests
        if self.is_tick_rest(start) && self.is_tick_rest(stop - half_beat) {
            return false;
        }

        // check that there are only events at the expected positions
        for tick in start..stop {
            // we ignore the first tick, it doesn't give us any useful information
            if tick == start {
                continue;
            };

            // there must be notation on the last half beat (note or rest)
            if tick == stop - half_beat {
                if self.is_tick_empty(tick) {
                    return false;
                } else {
                    continue;
                }
            }

            let is_offbeat = (tick - start).is_multiple_of(half_beat);
            let is_onbeat = (tick - start).is_multiple_of(whole_beat);

            // if we are on an offbeat whole note there must be a note
            if is_offbeat && !is_onbeat {
                if self.is_tick_empty(tick) || self.is_tick_rest(tick) {
                    return false;
                } else {
                    continue;
                }
            };

            // other ticks must be empty
            if !self.is_tick_empty(tick) {
                return false;
            }
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
                    let end_tick = tone.tick + tone.duration;
                    if end_tick < self.length {
                        split_points.insert(end_tick);
                    }
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
        start: Tick,
        stop: Tick,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) {
        // we stop once there are no events in the range
        if self.is_range_empty(start, stop) {
            return;
        }

        let ticks_per_original_beat = original_time_signature.ticks_per_beat();

        let mut needs_splitting = (stop - start) > ticks_per_original_beat;
        if !needs_splitting {
            for tick in start..stop {
                if let Some(notation) = self.track.get(&tick) {
                    // Dotted rests can only appear at the beginning of a unit
                    if notation.is_rest() && notation.is_dotted() && notation.tick != start {
                        needs_splitting = true;
                        break;
                    }

                    // Everything else needs to be writable
                    if !Notation::is_writable(notation.duration) {
                        needs_splitting = true;
                        break;
                    }
                }
            }
        }

        if !needs_splitting {
            return;
        }

        match time_signature.beats {
            2 => {
                self.split_simple_unit(
                    start,
                    stop,
                    original_time_signature,
                    &TimeSignature::new(
                        0,
                        original_time_signature.tick,
                        4,
                        original_time_signature.beat_type.half(),
                        TimeSignatureDrawType::Regular,
                        None,
                    ),
                );
            }
            3 => {
                let beat_one = start;
                let beat_two = time_signature.get_tick_at_beat(start, 2);
                let beat_three = time_signature.get_tick_at_beat(start, 3);

                let eigth = NoteDuration::Eighth.to_ticks();
                let dotted_quarter = NoteDuration::Quarter.to_ticks() + eigth;

                let patterns =
                    [self.is_offbeat_pattern(start, stop, time_signature.beat_type.to_ticks())];

                if patterns.iter().any(|&pattern_matched| pattern_matched) {
                    // These patterns span the entire unit so we can early return as there is no
                    // higher fidelity that is needed.
                    return;
                }

                // Patterns that start with a dotted quarter on beat 1
                if let Some(entry) = self.track.get(&beat_one) {
                    if !entry.is_rest() && entry.duration == dotted_quarter {
                        if let Some(next_entry) = self.track.get(&(beat_one + dotted_quarter)) {
                            // if the first beat is a dotted 1/4 and it's followed by a rest, we split at beat 2
                            if next_entry.is_rest() {
                                self.split(beat_two);
                            }
                            // split 2 dotted quarters at beat 3 so it doesn't look like 6/8
                            else if next_entry.duration == dotted_quarter {
                                self.split(beat_three);
                            }
                        }
                    }
                }

                // Split at the next beat if any note starts off the eighth-note grid. Esures valid note lengths are not
                // erroniously drawn across beat boundries. No need to check the last beat since it ends at the stop point.
                let ranges = [
                    (beat_one, beat_two, beat_two),
                    (beat_two, beat_three, beat_three),
                ];

                for (start, end, split_at) in ranges {
                    for tick in start..end {
                        if let Some(entry) = self.track.get(&tick) {
                            if (entry.tick - start) % eigth != 0 {
                                self.split(split_at);
                                break;
                            }
                        }
                    }
                }

                // Split all non-writable durations at beat boundaries
                for tick in start..stop {
                    if let Some(entry) = self.track.get(&tick) {
                        let duration = entry.get_writable_ticks().unwrap_or(entry.duration);

                        if !Notation::is_writable(duration) {
                            let end = entry.tick + duration;

                            // Split at the last beat if non-writable
                            if tick < beat_three && end > beat_three {
                                self.split(beat_three);

                                let remaining = beat_three - tick;

                                // If the remaining is still not writable and if it goes across beat 2, we split it
                                // otherwise it's managed by the next recursion
                                if !Notation::is_writable(remaining)
                                    && tick < beat_two
                                    && end > beat_two
                                {
                                    self.split(beat_two);
                                }

                                continue;
                            }

                            // It does not cross beat 3 but crosses beat 2
                            if tick < beat_two && end > beat_two {
                                self.split(beat_two);
                            }
                        }
                    };
                }

                // if we haven't made any splits we split at the third beat
                if self.is_tick_empty(beat_two) && self.is_tick_empty(beat_three) {
                    // the exception is if there is a dotted quarter at the end of the bar
                    if let Some(entry) = self.track.get(&(beat_one + dotted_quarter)) {
                        if entry.is_rest() || entry.duration != dotted_quarter {
                            self.split(beat_three);
                        }
                    }
                }

                // Split all rests at beat boundaries. Do this right at the end as rests don't count
                // towards the "if we haven't made any splits we split at the third beat" rule
                for tick in start..stop {
                    if let Some(entry) = self.track.get(&tick) {
                        if entry.is_rest() {
                            let end = entry.tick + entry.duration;
                            if tick < beat_two && end > beat_two {
                                self.split(beat_two);
                            }

                            if tick < beat_three && end > beat_three {
                                self.split(beat_three);
                            }
                        }
                    };
                }

                let next = TimeSignature::new(
                    0,
                    time_signature.tick,
                    4,
                    time_signature.beat_type.half().half(),
                    TimeSignatureDrawType::Regular,
                    None,
                );

                self.split_simple_unit(beat_one, beat_two, original_time_signature, &next);
                self.split_simple_unit(beat_two, beat_three, original_time_signature, &next);
                self.split_simple_unit(beat_three, stop, original_time_signature, &next);
            }
            4 => {
                let beat_one = start;
                let beat_two = time_signature.get_tick_at_beat(start, 2);
                let beat_three = time_signature.get_tick_at_beat(start, 3);
                let beat_four = time_signature.get_tick_at_beat(start, 4);

                let patterns = [
                    self.is_qhq_pattern(beat_one, beat_two, beat_four, stop),
                    self.is_qmdot_pattern(beat_one, beat_two, stop),
                    self.is_mdotq_pattern(
                        ticks_per_original_beat,
                        beat_one,
                        beat_three,
                        beat_four,
                        stop,
                    ),
                    self.is_offbeat_pattern(start, stop, time_signature.beat_type.to_ticks()),
                ];

                if patterns.iter().any(|&pattern_matched| pattern_matched) {
                    // These patterns span the entire unit so we can early return as there is no
                    // higher fidelity that is needed.
                    return;
                }

                // Allow spanning the middle point for notes with dotted rhythms at the beginning of the unit,
                // the exception being a dotted note with a base of the tiem signature that is followed by a rest.
                if let Some(notation) = self.track.get(&beat_one) {
                    if !notation.is_rest()
                        && notation.duration == beat_four - beat_one
                        && (notation.duration != (ticks_per_original_beat / 2) * 3
                            || !self.is_tick_rest(beat_four))
                    {
                        let next_unit_time_signature = TimeSignature::new(
                            0,
                            time_signature.tick,
                            4,
                            time_signature.beat_type.half(),
                            TimeSignatureDrawType::Regular,
                            None,
                        );
                        self.split_simple_unit(
                            beat_four,
                            stop,
                            original_time_signature,
                            &next_unit_time_signature,
                        );
                        return;
                    }
                }

                // Allow spanning the middle point for notes with dotted rhythms at the end of the unit less than a beat length
                if let Some(notation) = self.track.get(&beat_two) {
                    if !notation.is_rest()
                        && notation.duration == stop - beat_two
                        && notation.duration < ticks_per_original_beat
                    {
                        let next_unit_time_signature = TimeSignature::new(
                            0,
                            time_signature.tick,
                            4,
                            time_signature.beat_type.half(),
                            TimeSignatureDrawType::Regular,
                            None,
                        );
                        self.split_simple_unit(
                            beat_one,
                            beat_two,
                            original_time_signature,
                            &next_unit_time_signature,
                        );
                        return;
                    }
                }

                self.split(beat_three);

                let next_unit_time_signature = TimeSignature::new(
                    0,
                    time_signature.tick,
                    4,
                    time_signature.beat_type.half(),
                    TimeSignatureDrawType::Regular,
                    None,
                );

                self.split_simple_unit(
                    start,
                    beat_three,
                    original_time_signature,
                    &next_unit_time_signature,
                );
                self.split_simple_unit(
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

    let results: Vec<_> = instruments
        .par_iter()
        .flat_map(|instrument| {
            instrument
                .stave_ids
                .par_iter()
                .flat_map(|stave_id| {
                    let stave = &flow.staves.by_id[stave_id];
                    stave
                        .track_ids
                        .par_iter()
                        .map(|track_id| {
                            let track = &tracks.by_id[track_id];
                            let mut notation = NotationTrack::new(flow.length);
                            notation.split_at_events(track, measures);

                            // split as measures
                            for (tick, time_signature) in measures {
                                match time_signature.kind() {
                                    TimeSignatureType::Simple => notation.split_simple_unit(
                                        *tick,
                                        tick + time_signature.ticks_per_bar(),
                                        time_signature,
                                        time_signature,
                                    ),
                                    TimeSignatureType::Compound => todo!(),
                                    TimeSignatureType::Complex => todo!(),
                                    TimeSignatureType::Open => todo!(),
                                }
                            }

                            // Assign tone offset info here so we don't have to pass mutable
                            // notation tracks around
                            get_tone_offset_info(&mut notation, tone_offsets);

                            (track.id, notation)
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for (track_id, notation) in results {
        entries.insert(track_id, notation);
    }

    entries
}
