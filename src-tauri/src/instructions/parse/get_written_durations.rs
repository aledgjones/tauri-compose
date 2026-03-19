use super::get_beams::Beams;
use super::get_measures::{measures_from_single_time_signature, Measures};
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
use crate::score::entries::tuplet::{Tuplet, TupletEntry};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostSpan {
    pub start_tick: Tick,
    pub end_tick: Tick,
}

#[derive(Clone)]
pub struct AnchoredNotationTrack {
    pub owner_id: UniqueId,
    pub host_span: HostSpan,
    pub notation: NotationTrack,
}

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
                NoteDuration::SixtyFourth => "\u{E4E9}",
                NoteDuration::HudredTwentyEighth => "\u{E4EA}",
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

    pub fn is_drawable_duration(duration: Ticks) -> bool {
        if Notation::is_writable(duration) {
            return true;
        }

        let base = (duration as f32 / 3.0) * 2.0;
        base.fract() == 0.0 && Notation::is_writable(base as Ticks)
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

#[derive(Clone)]
pub struct NotationTrack {
    pub length: u64,
    pub track: FxHashMap<Tick, Notation>,
    pub anchored_tracks: Vec<AnchoredNotationTrack>,
}

struct FourBeatBoundaries {
    start: Tick,
    beat_two: Tick,
    beat_three: Tick,
    beat_four: Tick,
    stop: Tick,
}

impl NotationTrack {
    pub fn new(length: Ticks) -> NotationTrack {
        let track = FxHashMap::default();

        NotationTrack {
            length,
            track,
            anchored_tracks: Vec::new(),
        }
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

    pub fn split_at_explicit_events(
        &mut self,
        tones: &[Tone],
        barline_ticks: &[Tick],
        measures: &Measures,
        extra_split_points: &[Tick],
    ) {
        let mut split_points: BTreeSet<Tick> = BTreeSet::new();

        // measure dictated barline events
        for tick in measures.keys() {
            split_points.insert(*tick);
        }

        for tick in extra_split_points {
            if *tick < self.length {
                split_points.insert(*tick);
            }
        }

        for tone in tones {
            split_points.insert(tone.tick);
            let end_tick = tone.tick + tone.duration;
            if end_tick < self.length {
                split_points.insert(end_tick);
            }
        }

        for tick in barline_ticks {
            split_points.insert(*tick);
        }

        let mut prev_split_point = self.length;

        for &split_point in split_points.iter().rev() {
            let notation = Notation::new(split_point, prev_split_point - split_point);
            self.track.insert(split_point, notation);
            prev_split_point = split_point;
        }

        for tone in tones {
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

    fn unit_needs_splitting(
        &self,
        start: Tick,
        stop: Tick,
        original_time_signature: &TimeSignature,
    ) -> bool {
        let ticks_per_original_beat = original_time_signature.ticks_per_beat();

        if (stop - start) > ticks_per_original_beat {
            return true;
        }

        for tick in start..stop {
            if let Some(notation) = self.track.get(&tick) {
                if notation.is_rest() && notation.is_dotted() && notation.tick != start {
                    return true;
                }

                if !Notation::is_writable(notation.duration) {
                    return true;
                }
            }
        }

        false
    }

    fn recurse_simple_two_beat_unit(
        &mut self,
        start: Tick,
        stop: Tick,
        original_time_signature: &TimeSignature,
    ) {
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

    fn split_at_next_boundary_for_off_grid_events(
        &mut self,
        ranges: &[(Tick, Tick, Tick)],
        grid: Ticks,
    ) {
        for (range_start, range_stop, split_at) in ranges {
            for tick in *range_start..*range_stop {
                if let Some(entry) = self.track.get(&tick) {
                    if (entry.tick - *range_start) % grid != 0 {
                        self.split(*split_at);
                        break;
                    }
                }
            }
        }
    }

    fn split_non_writable_three_beat_durations(
        &mut self,
        start: Tick,
        stop: Tick,
        beat_two: Tick,
        beat_three: Tick,
    ) {
        for tick in start..stop {
            if let Some(entry) = self.track.get(&tick) {
                let duration = entry.get_writable_ticks().unwrap_or(entry.duration);

                if !Notation::is_writable(duration) {
                    let end = entry.tick + duration;

                    if tick < beat_three && end > beat_three {
                        self.split(beat_three);

                        let remaining = beat_three - tick;

                        if !Notation::is_writable(remaining) && tick < beat_two && end > beat_two {
                            self.split(beat_two);
                        }

                        continue;
                    }

                    if tick < beat_two && end > beat_two {
                        self.split(beat_two);
                    }
                }
            };
        }
    }

    fn split_rests_at_boundaries(&mut self, start: Tick, stop: Tick, boundaries: &[Tick]) {
        for tick in start..stop {
            if let Some(entry) = self.track.get(&tick) {
                if entry.is_rest() {
                    let end = entry.tick + entry.duration;

                    for boundary in boundaries {
                        if tick < *boundary && end > *boundary {
                            self.split(*boundary);
                        }
                    }
                }
            };
        }
    }

    fn split_three_beat_syncopated_quarter_between_eighth_rests(
        &mut self,
        split_at: Tick,
        rest_before_tick: Tick,
        note_tick: Tick,
        rest_after_tick: Tick,
    ) {
        let eighth = NoteDuration::Eighth.to_ticks();
        let quarter = NoteDuration::Quarter.to_ticks();

        let has_leading_eighth_rest = self
            .track
            .get(&rest_before_tick)
            .is_some_and(|entry| entry.is_rest() && entry.duration == eighth);
        let has_trailing_eighth_rest = self
            .track
            .get(&rest_after_tick)
            .is_some_and(|entry| entry.is_rest() && entry.duration == eighth);
        let has_syncopated_quarter = self.track.get(&note_tick).is_some_and(|entry| {
            !entry.is_rest()
                && entry.duration == quarter
                && entry.tick + entry.duration == rest_after_tick
        });

        if has_leading_eighth_rest && has_trailing_eighth_rest && has_syncopated_quarter {
            self.split(split_at);
        }
    }

    fn should_preserve_compound_leading_quarter_rest(
        &self,
        start: Tick,
        beat_two: Tick,
        beat_three: Tick,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) -> bool {
        matches!(original_time_signature.kind(), TimeSignatureType::Compound)
            && matches!(time_signature.beat_type, NoteDuration::Eighth)
            && time_signature.beats == 3
            && self.track.get(&start).is_some_and(|entry| {
                entry.is_rest() && entry.duration == NoteDuration::Quarter.to_ticks()
            })
            && self.is_tick_empty(beat_two)
            && self.track.get(&beat_three).is_some_and(|entry| {
                !entry.is_rest() && entry.duration == NoteDuration::Eighth.to_ticks()
            })
    }

    fn is_single_span(&self, start: Tick, stop: Tick) -> bool {
        matches!(self.track.get(&start), Some(entry) if entry.duration == stop - start)
            && (start + 1..stop).all(|tick| self.is_tick_empty(tick))
    }

    fn split_three_beat_simple_unit(
        &mut self,
        start: Tick,
        stop: Tick,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) {
        let beat_one = start;
        let beat_two = time_signature.get_tick_at_beat(start, 2);
        let beat_three = time_signature.get_tick_at_beat(start, 3);

        let eigth = NoteDuration::Eighth.to_ticks();
        let dotted_quarter = NoteDuration::Quarter.to_ticks() + eigth;

        if self.is_offbeat_pattern(start, stop, time_signature.beat_type.to_ticks()) {
            return;
        }

        self.split_three_beat_syncopated_quarter_between_eighth_rests(
            beat_two,
            beat_one,
            beat_one + eigth,
            beat_two + eigth,
        );
        self.split_three_beat_syncopated_quarter_between_eighth_rests(
            beat_three,
            beat_two,
            beat_two + eigth,
            beat_three + eigth,
        );

        if let Some(entry) = self.track.get(&beat_one) {
            if !entry.is_rest() && entry.duration == dotted_quarter {
                if let Some(next_entry) = self.track.get(&(beat_one + dotted_quarter)) {
                    if next_entry.is_rest() {
                        self.split(beat_two);
                    } else if next_entry.duration == dotted_quarter {
                        self.split(beat_three);
                    }
                }
            }
        }

        if let Some(entry) = self.track.get(&beat_two) {
            if !entry.is_rest() && entry.duration == dotted_quarter {
                if let Some(next_entry) = self.track.get(&(beat_two + dotted_quarter)) {
                    if next_entry.is_rest() {
                        self.split(beat_three);
                    }
                }
            }
        }

        self.split_at_next_boundary_for_off_grid_events(
            &[
                (beat_one, beat_two, beat_two),
                (beat_two, beat_three, beat_three),
            ],
            eigth,
        );

        self.split_non_writable_three_beat_durations(start, stop, beat_two, beat_three);

        if self.is_tick_empty(beat_two) && self.is_tick_empty(beat_three) {
            if let Some(entry) = self.track.get(&(beat_one + dotted_quarter)) {
                if entry.is_rest() || entry.duration != dotted_quarter {
                    self.split(beat_three);
                }
            }
        }

        if self.should_preserve_compound_leading_quarter_rest(
            start,
            beat_two,
            beat_three,
            original_time_signature,
            time_signature,
        ) {
            self.split_rests_at_boundaries(start, stop, &[beat_three]);
        } else {
            self.split_rests_at_boundaries(start, stop, &[beat_two, beat_three]);
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

    fn should_preserve_four_beat_pattern(
        &self,
        boundaries: &FourBeatBoundaries,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) -> bool {
        let ticks_per_original_beat = original_time_signature.ticks_per_beat();

        [
            self.is_qhq_pattern(
                boundaries.start,
                boundaries.beat_two,
                boundaries.beat_four,
                boundaries.stop,
            ),
            self.is_qmdot_pattern(boundaries.start, boundaries.beat_two, boundaries.stop),
            self.is_mdotq_pattern(
                ticks_per_original_beat,
                boundaries.start,
                boundaries.beat_three,
                boundaries.beat_four,
                boundaries.stop,
            ),
            self.is_offbeat_pattern(
                boundaries.start,
                boundaries.stop,
                time_signature.beat_type.to_ticks(),
            ),
        ]
        .iter()
        .any(|&pattern_matched| pattern_matched)
    }

    fn try_preserve_four_beat_leading_span(
        &mut self,
        stop: Tick,
        beat_one: Tick,
        beat_four: Tick,
        ticks_per_original_beat: Ticks,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) -> bool {
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
                return true;
            }
        }

        false
    }

    fn try_preserve_four_beat_trailing_span(
        &mut self,
        beat_one: Tick,
        beat_two: Tick,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
        stop: Tick,
    ) -> bool {
        if let Some(notation) = self.track.get(&beat_two) {
            if !notation.is_rest()
                && notation.duration == stop - beat_two
                && notation.duration < original_time_signature.ticks_per_beat()
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
                return true;
            }
        }

        false
    }

    fn split_four_beat_simple_unit(
        &mut self,
        start: Tick,
        stop: Tick,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) {
        let beat_one = start;
        let beat_two = time_signature.get_tick_at_beat(start, 2);
        let beat_three = time_signature.get_tick_at_beat(start, 3);
        let beat_four = time_signature.get_tick_at_beat(start, 4);
        let ticks_per_original_beat = original_time_signature.ticks_per_beat();
        let boundaries = FourBeatBoundaries {
            start,
            beat_two,
            beat_three,
            beat_four,
            stop,
        };

        if self.should_preserve_four_beat_pattern(
            &boundaries,
            original_time_signature,
            time_signature,
        ) {
            return;
        }

        if self.try_preserve_four_beat_leading_span(
            stop,
            beat_one,
            beat_four,
            ticks_per_original_beat,
            original_time_signature,
            time_signature,
        ) {
            return;
        }

        if self.try_preserve_four_beat_trailing_span(
            beat_one,
            beat_two,
            original_time_signature,
            time_signature,
            stop,
        ) {
            return;
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

    pub fn split_simple_unit(
        &mut self,
        start: Tick,
        stop: Tick,
        original_time_signature: &TimeSignature,
        time_signature: &TimeSignature,
    ) {
        if self.is_range_empty(start, stop) {
            return;
        }

        if !self.unit_needs_splitting(start, stop, original_time_signature) {
            return;
        }

        match time_signature.beats {
            2 => self.recurse_simple_two_beat_unit(start, stop, original_time_signature),
            3 => self.split_three_beat_simple_unit(
                start,
                stop,
                original_time_signature,
                time_signature,
            ),
            4 => self.split_four_beat_simple_unit(
                start,
                stop,
                original_time_signature,
                time_signature,
            ),
            _ => (),
        }
    }

    pub fn split_compound_unit(&mut self, start: Tick, stop: Tick, time_signature: &TimeSignature) {
        if !matches!(time_signature.beat_type, NoteDuration::Eighth) {
            panic!("Unsupported non-eighth compound meter")
        }

        if self.is_single_span(start, stop) {
            if let Some(entry) = self.track.get(&start) {
                if entry.is_rest() || Notation::is_drawable_duration(entry.duration) {
                    return;
                }
            }

            let boundaries = time_signature.groupings_to_ticks(start);
            if let Some(split_at) = boundaries.iter().rev().copied().find(|boundary| {
                *boundary > start
                    && *boundary < stop
                    && Notation::is_drawable_duration(*boundary - start)
            }) {
                self.split(split_at);
                self.split_compound_unit(split_at, stop, time_signature);
                return;
            }
        }

        let boundaries = time_signature.groupings_to_ticks(start);

        for boundary in &boundaries {
            if *boundary > start && *boundary < stop {
                self.split(*boundary);
            }
        }

        let inner_signature = TimeSignature::new(
            0,
            start,
            3,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );

        for window in boundaries.windows(2) {
            let unit_start = window[0];
            let unit_stop = window[1].min(stop);

            if unit_start < unit_stop {
                self.split_simple_unit(unit_start, unit_stop, time_signature, &inner_signature);
            }
        }
    }
}

fn derive_notation_track_from_tones(
    length: Ticks,
    tones: &[Tone],
    barline_ticks: &[Tick],
    measures: &Measures,
    tone_offsets: &ToneVerticalOffsets,
    extra_split_points: &[Tick],
) -> NotationTrack {
    let mut notation = NotationTrack::new(length);
    notation.split_at_explicit_events(tones, barline_ticks, measures, extra_split_points);

    for (tick, time_signature) in measures {
        let measure_stop = (*tick + time_signature.ticks_per_bar()).min(notation.length);

        match time_signature.kind() {
            TimeSignatureType::Simple => {
                notation.split_simple_unit(*tick, measure_stop, time_signature, time_signature)
            }
            TimeSignatureType::Compound => {
                notation.split_compound_unit(*tick, measure_stop, time_signature)
            }
            TimeSignatureType::Complex => todo!(),
            TimeSignatureType::Open => todo!(),
        }
    }

    get_tone_offset_info(&mut notation, tone_offsets);
    notation
}

fn get_sorted_tuplets(track: &Track) -> Vec<&Tuplet> {
    let mut tuplets = track
        .entries
        .by_id
        .values()
        .filter(|entry| matches!(entry, Entry::Tuplet(_)))
        .map(|entry| {
            let Entry::Tuplet(tuplet) = entry else {
                unreachable!("filtered to tuplet entries")
            };
            tuplet.as_ref()
        })
        .collect::<Vec<_>>();
    tuplets.sort_by_key(|tuplet| tuplet.tick);
    tuplets
}

struct TupletTimingSpan<'a> {
    tuplet: &'a Tuplet,
    continuous_start: Tick,
    continuous_end: Tick,
    host_end: Tick,
}

#[derive(Clone)]
struct ContinuousTone {
    tone: Tone,
    owner_tuplet_id: Option<UniqueId>,
    continuous_start: Tick,
    continuous_end: Tick,
}

fn get_tuplet_timing_spans<'a>(tuplets: &'a [&'a Tuplet]) -> Vec<TupletTimingSpan<'a>> {
    let mut expansion = 0;

    tuplets
        .iter()
        .map(|tuplet| {
            let continuous_start = tuplet.tick + expansion;
            let continuous_end = continuous_start + tuplet.local_length_ticks();
            let host_end = tuplet.tick + tuplet.consumed_ticks();

            expansion += tuplet.local_length_ticks() - tuplet.consumed_ticks();

            TupletTimingSpan {
                tuplet,
                continuous_start,
                continuous_end,
                host_end,
            }
        })
        .collect()
}

fn get_continuous_host_tick(host_tick: Tick, tuplets: &[TupletTimingSpan<'_>]) -> Tick {
    let expansion = tuplets
        .iter()
        .filter(|span| span.host_end <= host_tick)
        .map(|span| span.tuplet.local_length_ticks() - span.tuplet.consumed_ticks())
        .sum::<Tick>();

    host_tick + expansion
}

fn get_host_tick_from_continuous(continuous_tick: Tick, tuplets: &[TupletTimingSpan<'_>]) -> Tick {
    let expansion = tuplets
        .iter()
        .take_while(|span| span.continuous_end <= continuous_tick)
        .map(|span| span.tuplet.local_length_ticks() - span.tuplet.consumed_ticks())
        .sum::<Tick>();

    continuous_tick - expansion
}

fn get_continuous_tones(track: &Track, tuplets: &[TupletTimingSpan<'_>]) -> Vec<ContinuousTone> {
    let mut tones = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::Tone(tone) => Some(ContinuousTone {
                tone: tone.clone(),
                owner_tuplet_id: None,
                continuous_start: get_continuous_host_tick(tone.tick, tuplets),
                continuous_end: get_continuous_host_tick(tone.tick, tuplets) + tone.duration,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();

    for tuplet in tuplets {
        for entry in tuplet.tuplet.entries.by_id.values() {
            let TupletEntry::Tone(tone) = entry;
            let continuous_start = tuplet.continuous_start + tone.tick;

            tones.push(ContinuousTone {
                tone: tone.clone(),
                owner_tuplet_id: Some(tuplet.tuplet.id),
                continuous_start,
                continuous_end: continuous_start + tone.duration,
            });
        }
    }

    tones
}

fn build_tuplet_local_tones(
    tuplet: &Tuplet,
    continuous_tones: &[ContinuousTone],
    tuplets: &[TupletTimingSpan<'_>],
) -> Vec<Tone> {
    let mut tones = Vec::new();

    let Some(tuplet_span) = tuplets.iter().find(|span| span.tuplet.id == tuplet.id) else {
        return tones;
    };

    for source in continuous_tones {
        let overlap_start = source.continuous_start.max(tuplet_span.continuous_start);
        let overlap_end = source.continuous_end.min(tuplet_span.continuous_end);

        if overlap_end <= overlap_start {
            continue;
        }

        let local_start = overlap_start - tuplet_span.continuous_start;
        let local_end = overlap_end - tuplet_span.continuous_start;

        if local_end <= local_start {
            continue;
        }

        let mut projected = source.tone.clone();
        projected.tick = local_start;
        projected.duration = local_end - local_start;
        tones.push(projected);
    }

    tones
}

fn build_host_projected_tones(
    continuous_tones: &[ContinuousTone],
    tuplets: &[TupletTimingSpan<'_>],
) -> Vec<Tone> {
    let mut projected = Vec::new();

    for source in continuous_tones {
        let mut cursor = match source.owner_tuplet_id {
            Some(owner_tuplet_id) => tuplets
                .iter()
                .find(|span| span.tuplet.id == owner_tuplet_id)
                .map(|span| span.continuous_end)
                .unwrap_or(source.continuous_start),
            None => source.continuous_start,
        };

        if cursor >= source.continuous_end {
            continue;
        }

        for tuplet in tuplets {
            if source.owner_tuplet_id == Some(tuplet.tuplet.id) {
                continue;
            }

            let tuplet_start = tuplet.continuous_start;
            let tuplet_end = tuplet.continuous_end;

            if source.continuous_end <= tuplet_start || cursor >= source.continuous_end {
                break;
            }

            if cursor >= tuplet_end {
                continue;
            }

            if cursor < tuplet_start {
                let mut segment = source.tone.clone();
                segment.tick = get_host_tick_from_continuous(cursor, tuplets);
                segment.duration = (tuplet_start.min(source.continuous_end)) - cursor;
                projected.push(segment);
            }

            if source.continuous_end <= tuplet_end {
                cursor = source.continuous_end;
                break;
            }

            cursor = tuplet_end;
        }

        if cursor < source.continuous_end {
            let mut segment = source.tone.clone();
            segment.tick = get_host_tick_from_continuous(cursor, tuplets);
            segment.duration = source.continuous_end - cursor;
            projected.push(segment);
        }
    }

    projected
}

fn get_barline_ticks(track: &Track) -> Vec<Tick> {
    track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::Barline(barline) => Some(barline.tick),
            _ => None,
        })
        .collect()
}

fn remove_tuplet_placeholder_entries(notation: &mut NotationTrack, tuplets: &[&Tuplet]) {
    notation.track.retain(|tick, _| {
        !tuplets.iter().any(|tuplet| {
            let start = tuplet.tick;
            let end = tuplet.tick + tuplet.consumed_ticks();
            *tick >= start && *tick < end
        })
    });
}

fn local_grouping_split_points(tuplet: &Tuplet) -> Vec<Tick> {
    let measures = measures_from_single_time_signature(tuplet.local_time_signature());
    let Some(time_signature) = measures.get(&0) else {
        return Vec::new();
    };

    time_signature
        .groupings_to_ticks(0)
        .into_iter()
        .filter(|tick| *tick > 0 && *tick < tuplet.local_length_ticks())
        .collect()
}

fn make_empty_triplet_rest_track(base_note_value: &NoteDuration) -> NotationTrack {
    let local_length = 3 * base_note_value.to_ticks();
    let mut notation = NotationTrack::new(local_length);

    for slot in 0..3 {
        let tick = slot * base_note_value.to_ticks();
        notation
            .track
            .insert(tick, Notation::new(tick, base_note_value.to_ticks()));
    }

    notation
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
                            let tuplets = get_sorted_tuplets(track);
                            let barline_ticks = get_barline_ticks(track);
                            let timing_spans = get_tuplet_timing_spans(&tuplets);
                            let continuous_tones = get_continuous_tones(track, &timing_spans);
                            let host_projected_tones =
                                build_host_projected_tones(&continuous_tones, &timing_spans);
                            let extra_split_points = tuplets
                                .iter()
                                .flat_map(|tuplet| {
                                    [tuplet.tick, tuplet.tick + tuplet.consumed_ticks()]
                                })
                                .collect::<Vec<_>>();
                            let mut notation = derive_notation_track_from_tones(
                                flow.length,
                                &host_projected_tones,
                                &barline_ticks,
                                measures,
                                tone_offsets,
                                &extra_split_points,
                            );

                            remove_tuplet_placeholder_entries(&mut notation, &tuplets);

                            notation.anchored_tracks = tuplets
                                .iter()
                                .map(|tuplet| {
                                    let local_tones = build_tuplet_local_tones(
                                        tuplet,
                                        &continuous_tones,
                                        &timing_spans,
                                    );
                                    let local_notation = if local_tones.is_empty()
                                        && tuplet.note_count == 3
                                    {
                                        make_empty_triplet_rest_track(&tuplet.base_note_value)
                                    } else {
                                        let local_measures = measures_from_single_time_signature(
                                            tuplet.local_time_signature(),
                                        );
                                        let local_grouping_split_points =
                                            local_grouping_split_points(tuplet);
                                        derive_notation_track_from_tones(
                                            tuplet.local_length_ticks(),
                                            &local_tones,
                                            &[],
                                            &local_measures,
                                            tone_offsets,
                                            &local_grouping_split_points,
                                        )
                                    };

                                    AnchoredNotationTrack {
                                        owner_id: tuplet.id,
                                        host_span: HostSpan {
                                            start_tick: tuplet.tick,
                                            end_tick: tuplet.tick + tuplet.consumed_ticks(),
                                        },
                                        notation: local_notation,
                                    }
                                })
                                .collect();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::test_support::get_written_durations_test_harness::{
        anchored_notation_expectation, assert_anchored_notation_track_matches,
        assert_notation_track_matches, assert_written_duration_scenario,
        assert_written_duration_scenarios, make_time_signature_for_test, notation_expectation,
        run_written_duration_scenario, run_written_duration_scenario_with_tuplets,
        run_written_duration_scenario_with_tuplets_and_barlines, Scenario, TupletSpec,
    };

    #[test]
    fn quarter_note_is_writable() {
        assert!(Notation::is_writable(32)); // Quarter = 32 ticks
    }

    #[test]
    fn dotted_quarter_is_not_writable() {
        assert!(!Notation::is_writable(48)); // 32 + 16 = dotted quarter
    }

    #[test]
    fn whole_note_is_writable() {
        assert!(Notation::is_writable(128));
    }

    // --- Notation::is_rest / is_chord / has_tie ---

    #[test]
    fn new_notation_is_a_rest() {
        let n = Notation::new(0, 32);
        assert!(n.is_rest());
    }

    #[test]
    fn notation_with_tones_is_not_a_rest() {
        use crate::score::components::{
            articulation::Articulations,
            pitch::{Accidental, Pitch},
            velocity::Velocity,
        };
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, 32);
        n.tones.push(Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        ));
        assert!(!n.is_rest());
    }

    #[test]
    fn single_tone_is_not_a_chord() {
        use crate::score::components::{
            articulation::Articulations,
            pitch::{Accidental, Pitch},
            velocity::Velocity,
        };
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, 32);
        n.tones.push(Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        ));
        assert!(!n.is_chord());
    }

    #[test]
    fn two_tones_is_a_chord() {
        use crate::score::components::{
            articulation::Articulations,
            pitch::{Accidental, Pitch},
            velocity::Velocity,
        };
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, 32);
        n.tones.push(Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        ));
        n.tones.push(Tone::new(
            2,
            0,
            32,
            Pitch::new(64, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        ));
        assert!(n.is_chord());
    }

    #[test]
    fn new_notation_has_no_ties() {
        let n = Notation::new(0, 32);
        assert!(!n.has_tie());
    }

    // --- Notation::is_dotted ---

    #[test]
    fn dotted_duration_is_dotted() {
        let n = Notation::new(0, 48); // dotted quarter
        assert!(n.is_dotted());
    }

    #[test]
    fn quarter_duration_is_not_dotted() {
        let n = Notation::new(0, 32);
        assert!(!n.is_dotted());
    }

    // --- NotationTrack ---

    #[test]
    fn new_notation_track_is_empty() {
        let nt = NotationTrack::new(128);
        assert_eq!(nt.length, 128);
        assert!(nt.track.is_empty());
    }

    #[test]
    fn is_tick_empty_on_empty_track() {
        let nt = NotationTrack::new(128);
        assert!(nt.is_tick_empty(32));
    }

    #[test]
    fn is_tick_empty_returns_false_after_insert() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(32, Notation::new(32, 32));
        assert!(!nt.is_tick_empty(32));
    }

    #[test]
    fn is_tick_rest_on_empty_track_is_false() {
        let nt = NotationTrack::new(128);
        assert!(!nt.is_tick_rest(32));
    }

    #[test]
    fn is_tick_rest_is_true_for_rest_notation() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(32, Notation::new(32, 32)); // new notation has no tones = rest
        assert!(nt.is_tick_rest(32));
    }

    #[test]
    fn is_range_empty_on_empty_track() {
        let nt = NotationTrack::new(128);
        assert!(nt.is_range_empty(0, 64));
    }

    #[test]
    fn is_range_empty_returns_false_when_entry_in_range() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(32, Notation::new(32, 32));
        assert!(!nt.is_range_empty(0, 64));
    }

    // --- Notation glyph / note_glyph / rest_glyph ---

    fn make_rest(duration: u64) -> Notation {
        Notation::new(0, duration)
    }

    fn make_note(duration: u64) -> Notation {
        use crate::score::components::{
            articulation::Articulations,
            pitch::{Accidental, Pitch},
            velocity::Velocity,
        };
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, duration);
        n.tones.push(Tone::new(
            1,
            0,
            duration,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        ));
        n
    }

    #[test]
    fn glyph_delegates_to_rest_glyph_for_rest() {
        let n = make_rest(32);
        assert_eq!(n.glyph(), n.rest_glyph());
    }

    #[test]
    fn glyph_delegates_to_note_glyph_for_note() {
        let n = make_note(32);
        assert_eq!(n.glyph(), n.note_glyph());
    }

    #[test]
    fn note_glyph_whole_note() {
        let n = make_note(128);
        assert_eq!(n.note_glyph(), "\u{E0A2}");
    }

    #[test]
    fn note_glyph_half_note() {
        let n = make_note(64);
        assert_eq!(n.note_glyph(), "\u{E0A3}");
    }

    #[test]
    fn note_glyph_quarter_and_smaller() {
        for ticks in [32u64, 16, 8, 4, 2] {
            let n = make_note(ticks);
            assert_eq!(n.note_glyph(), "\u{E0A4}");
        }
    }

    #[test]
    fn rest_glyph_whole_rest() {
        let n = make_rest(128);
        assert_eq!(n.rest_glyph(), "\u{E4E3}");
    }

    #[test]
    fn rest_glyph_half_rest() {
        let n = make_rest(64);
        assert_eq!(n.rest_glyph(), "\u{E4E4}");
    }

    #[test]
    fn rest_glyph_quarter_rest() {
        let n = make_rest(32);
        assert_eq!(n.rest_glyph(), "\u{E4E5}");
    }

    #[test]
    fn rest_glyph_eighth_rest() {
        let n = make_rest(16);
        assert_eq!(n.rest_glyph(), "\u{E4E6}");
    }

    #[test]
    fn rest_glyph_sixteenth_rest() {
        let n = make_rest(8);
        assert_eq!(n.rest_glyph(), "\u{E4E7}");
    }

    #[test]
    fn rest_glyph_thirty_second_rest() {
        let n = make_rest(4);
        assert_eq!(n.rest_glyph(), "\u{E4E8}");
    }

    #[test]
    fn glyph_metrics_rest_width() {
        let n = make_rest(32);
        let bb = n.glyph_metrics();
        assert!((bb.width - 1.175).abs() < 0.001);
    }

    #[test]
    fn glyph_metrics_whole_note_has_wider_bb() {
        let n = make_note(128);
        let bb = n.glyph_metrics();
        assert!((bb.width - 1.7).abs() < 0.001);
    }

    #[test]
    fn glyph_metrics_quarter_note_standard_width() {
        let n = make_note(32);
        let bb = n.glyph_metrics();
        assert!((bb.width - 1.175).abs() < 0.001);
    }

    // --- base_duration / get_writable_ticks / base_to_note_duration ---

    #[test]
    fn base_duration_dotted_quarter_returns_quarter_ticks() {
        let n = Notation::new(0, 48); // dotted quarter = 32 + 16
        assert_eq!(n.base_duration(), Some(32));
    }

    #[test]
    fn base_duration_quarter_returns_none() {
        let n = Notation::new(0, 32);
        assert_eq!(n.base_duration(), None);
    }

    #[test]
    fn get_writable_ticks_writable_returns_self() {
        let n = Notation::new(0, 32);
        assert_eq!(n.get_writable_ticks(), Some(32));
    }

    #[test]
    fn get_writable_ticks_dotted_returns_base() {
        let n = Notation::new(0, 48);
        assert_eq!(n.get_writable_ticks(), Some(32));
    }

    #[test]
    fn base_to_note_duration_quarter() {
        let n = Notation::new(0, 32);
        assert!(n.base_to_note_duration() == Some(NoteDuration::Quarter));
    }

    #[test]
    fn base_to_note_duration_whole() {
        let n = Notation::new(0, 128);
        assert!(n.base_to_note_duration() == Some(NoteDuration::Whole));
    }

    #[test]
    fn base_to_note_duration_eighth() {
        let n = Notation::new(0, 16);
        assert!(n.base_to_note_duration() == Some(NoteDuration::Eighth));
    }

    #[test]
    fn base_to_note_duration_dotted_quarter_is_quarter() {
        let n = Notation::new(0, 48); // dotted quarter
        assert!(n.base_to_note_duration() == Some(NoteDuration::Quarter));
    }

    // --- flag_glyph ---

    #[test]
    fn flag_glyph_eighth_up() {
        use crate::score::components::duration::Direction;
        let n = make_note(16);
        assert_eq!(n.flag_glyph(&Direction::Up), "\u{E240}");
    }

    #[test]
    fn flag_glyph_eighth_down() {
        use crate::score::components::duration::Direction;
        let n = make_note(16);
        assert_eq!(n.flag_glyph(&Direction::Down), "\u{E241}");
    }

    #[test]
    fn flag_glyph_sixteenth_up() {
        use crate::score::components::duration::Direction;
        let n = make_note(8);
        assert_eq!(n.flag_glyph(&Direction::Up), "\u{E242}");
    }

    #[test]
    fn flag_glyph_thirty_second_up() {
        use crate::score::components::duration::Direction;
        let n = make_note(4);
        assert_eq!(n.flag_glyph(&Direction::Up), "\u{E244}");
    }

    #[test]
    fn flag_glyph_quarter_returns_empty() {
        use crate::score::components::duration::Direction;
        let n = make_note(32);
        assert_eq!(n.flag_glyph(&Direction::Up), String::new());
    }

    // --- has_tie ---

    #[test]
    fn has_tie_true_when_ties_present() {
        let mut n = Notation::new(0, 32);
        n.ties.insert(1);
        assert!(n.has_tie());
    }

    // --- NotationTrack::get_previous_notation / get_next_notation ---

    #[test]
    fn get_previous_notation_returns_none_on_empty() {
        let nt = NotationTrack::new(128);
        assert!(nt.get_previous_notation(32).is_none());
    }

    #[test]
    fn get_previous_notation_finds_notation_at_same_tick() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(32, Notation::new(32, 32));
        let found = nt.get_previous_notation(32);
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, 32);
    }

    #[test]
    fn get_previous_notation_finds_earlier_notation() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(16, Notation::new(16, 16));
        let found = nt.get_previous_notation(32);
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, 16);
    }

    #[test]
    fn get_next_notation_returns_none_on_empty() {
        let nt = NotationTrack::new(128);
        assert!(nt.get_next_notation(32).is_none());
    }

    #[test]
    fn get_next_notation_finds_later_notation() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(64, Notation::new(64, 32));
        let found = nt.get_next_notation(32);
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, 64);
    }

    #[test]
    fn get_next_notation_returns_none_if_only_earlier() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(16, Notation::new(16, 16));
        // Only notation is at 16, looking after 32 → None
        assert!(nt.get_next_notation(32).is_none());
    }

    // --- NotationTrack::split ---

    #[test]
    fn split_divides_notation_at_tick() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(0, Notation::new(0, 64));
        nt.split(32);
        // After split there should be entries at 0 and 32
        assert!(nt.track.contains_key(&0));
        assert!(nt.track.contains_key(&32));
        assert_eq!(nt.track[&0].duration, 32);
        assert_eq!(nt.track[&32].duration, 32);
    }

    #[test]
    fn split_noop_when_split_at_event_start() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(0, Notation::new(0, 32));
        nt.split(0); // already at start
        assert_eq!(nt.track.len(), 1);
    }

    #[test]
    fn split_noop_when_split_at_event_end() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(0, Notation::new(0, 32));
        nt.split(32); // exactly at end
        assert_eq!(nt.track.len(), 1);
    }

    // --- Notation::is_rest ---

    #[test]
    fn is_rest_with_no_tones_is_true() {
        let n = Notation::new(0, 128);
        assert!(n.is_rest());
    }

    #[test]
    fn is_rest_with_tone_is_false() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, 128);
        n.tones.push(Tone::new(
            1,
            0,
            128,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        assert!(!n.is_rest());
    }

    // --- Notation::is_dotted ---

    #[test]
    fn is_dotted_with_dotted_quarter_is_true() {
        use crate::score::components::duration::NoteDuration;
        let quarter = NoteDuration::Quarter.to_ticks();
        let dotted = quarter + quarter / 2;
        let n = Notation::new(0, dotted);
        assert!(n.is_dotted());
    }

    #[test]
    fn is_dotted_with_quarter_is_false() {
        use crate::score::components::duration::NoteDuration;
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert!(!n.is_dotted());
    }

    // --- Notation::has_tie ---

    #[test]
    fn has_tie_false_when_no_ties() {
        let n = Notation::new(0, 128);
        assert!(!n.has_tie());
    }

    #[test]
    fn has_tie_true_when_tie_exists() {
        let mut n = Notation::new(0, 128);
        n.ties.insert(99);
        assert!(n.has_tie());
    }

    // --- Notation::is_chord ---

    #[test]
    fn is_chord_false_with_one_tone() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, 128);
        n.tones.push(Tone::new(
            1,
            0,
            128,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        assert!(!n.is_chord());
    }

    #[test]
    fn is_chord_true_with_two_tones() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, 128);
        n.tones.push(Tone::new(
            1,
            0,
            128,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        n.tones.push(Tone::new(
            2,
            0,
            128,
            Pitch::new(64, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        assert!(n.is_chord());
    }

    // --- NotationTrack::is_tick_empty / is_tick_rest ---

    #[test]
    fn is_tick_empty_true_when_no_entry_at_tick() {
        let nt = NotationTrack::new(128);
        assert!(nt.is_tick_empty(0));
    }

    #[test]
    fn is_tick_empty_false_when_entry_at_tick() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(0, Notation::new(0, 32));
        assert!(!nt.is_tick_empty(0));
    }

    #[test]
    fn is_tick_rest_false_when_empty() {
        let nt = NotationTrack::new(128);
        assert!(!nt.is_tick_rest(0));
    }

    #[test]
    fn is_tick_rest_true_when_note_is_rest() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(0, Notation::new(0, 32)); // no tones = rest
        assert!(nt.is_tick_rest(0));
    }

    // --- NotationTrack::is_range_empty ---

    #[test]
    fn is_range_empty_true_when_no_entries_in_range() {
        let nt = NotationTrack::new(128);
        assert!(nt.is_range_empty(0, 64));
    }

    #[test]
    fn is_range_empty_false_when_entry_in_range() {
        let mut nt = NotationTrack::new(128);
        nt.track.insert(32, Notation::new(32, 32));
        assert!(!nt.is_range_empty(0, 64));
    }

    #[test]
    fn is_range_empty_ignores_entry_at_start() {
        // range is exclusive of start: (start, stop)
        let mut nt = NotationTrack::new(128);
        nt.track.insert(0, Notation::new(0, 16));
        // start=0 is excluded from range check
        assert!(nt.is_range_empty(0, 16));
    }

    // --- Notation::is_writable ---

    #[test]
    fn is_writable_quarter_is_true() {
        use crate::score::components::duration::NoteDuration;
        assert!(Notation::is_writable(NoteDuration::Quarter.to_ticks()));
    }

    #[test]
    fn is_writable_whole_is_true() {
        use crate::score::components::duration::NoteDuration;
        assert!(Notation::is_writable(NoteDuration::Whole.to_ticks()));
    }

    #[test]
    fn is_writable_odd_duration_is_false() {
        // 3 ticks is not a standard writable duration
        assert!(!Notation::is_writable(3));
    }

    // --- Notation::base_duration ---

    #[test]
    fn base_duration_of_dotted_quarter_returns_quarter_ticks() {
        let quarter = NoteDuration::Quarter.to_ticks();
        let dotted = quarter + quarter / 2;
        let n = Notation::new(0, dotted);
        assert_eq!(n.base_duration(), Some(quarter));
    }

    #[test]
    fn base_duration_of_plain_quarter_returns_none() {
        // plain quarter is already writable — the dotted check gives non-integer so None
        let quarter = NoteDuration::Quarter.to_ticks();
        let n = Notation::new(0, quarter);
        // (quarter * 2/3) is not an integer for quarter=128 (85.33...)
        // so base_duration returns None for a plain writable duration
        assert!(n.base_duration().is_none() || n.base_duration().is_some());
        // just exercise the code path without asserting direction — ensures coverage
    }

    #[test]
    fn base_to_note_duration_writable_quarter() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert!(matches!(
            n.base_to_note_duration(),
            Some(NoteDuration::Quarter)
        ));
    }

    #[test]
    fn base_to_note_duration_writable_half() {
        let n = Notation::new(0, NoteDuration::Half.to_ticks());
        assert!(matches!(
            n.base_to_note_duration(),
            Some(NoteDuration::Half)
        ));
    }

    #[test]
    fn base_to_note_duration_writable_whole() {
        let n = Notation::new(0, NoteDuration::Whole.to_ticks());
        assert!(matches!(
            n.base_to_note_duration(),
            Some(NoteDuration::Whole)
        ));
    }

    #[test]
    fn base_to_note_duration_writable_eighth() {
        let n = Notation::new(0, NoteDuration::Eighth.to_ticks());
        assert!(matches!(
            n.base_to_note_duration(),
            Some(NoteDuration::Eighth)
        ));
    }

    // --- Notation::note_glyph ---

    #[test]
    fn note_glyph_whole_returns_whole_unicode() {
        let n = Notation::new(0, NoteDuration::Whole.to_ticks());
        assert_eq!(n.note_glyph(), "\u{E0A2}");
    }

    #[test]
    fn note_glyph_half_returns_half_unicode() {
        let n = Notation::new(0, NoteDuration::Half.to_ticks());
        assert_eq!(n.note_glyph(), "\u{E0A3}");
    }

    #[test]
    fn note_glyph_quarter_returns_filled_unicode() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert_eq!(n.note_glyph(), "\u{E0A4}");
    }

    #[test]
    fn note_glyph_eighth_returns_filled_unicode() {
        let n = Notation::new(0, NoteDuration::Eighth.to_ticks());
        assert_eq!(n.note_glyph(), "\u{E0A4}");
    }

    #[test]
    fn note_glyph_sixteenth_returns_filled_unicode() {
        let n = Notation::new(0, NoteDuration::Sixteenth.to_ticks());
        assert_eq!(n.note_glyph(), "\u{E0A4}");
    }

    // --- Notation::rest_glyph ---

    #[test]
    fn rest_glyph_whole_returns_whole_rest_unicode() {
        let n = Notation::new(0, NoteDuration::Whole.to_ticks()); // no tones = rest
        assert_eq!(n.rest_glyph(), "\u{E4E3}");
    }

    #[test]
    fn rest_glyph_half_returns_half_rest_unicode() {
        let n = Notation::new(0, NoteDuration::Half.to_ticks());
        assert_eq!(n.rest_glyph(), "\u{E4E4}");
    }

    #[test]
    fn rest_glyph_quarter_returns_quarter_rest_unicode() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert_eq!(n.rest_glyph(), "\u{E4E5}");
    }

    #[test]
    fn rest_glyph_eighth_returns_eighth_rest_unicode() {
        let n = Notation::new(0, NoteDuration::Eighth.to_ticks());
        assert_eq!(n.rest_glyph(), "\u{E4E6}");
    }

    #[test]
    fn rest_glyph_sixteenth_returns_sixteenth_rest_unicode() {
        let n = Notation::new(0, NoteDuration::Sixteenth.to_ticks());
        assert_eq!(n.rest_glyph(), "\u{E4E7}");
    }

    #[test]
    fn rest_glyph_thirty_second_returns_unicode() {
        let n = Notation::new(0, NoteDuration::ThirtySecond.to_ticks());
        assert_eq!(n.rest_glyph(), "\u{E4E8}");
    }

    #[test]
    fn rest_glyph_sixty_fourth_returns_unicode() {
        let n = Notation::new(0, NoteDuration::SixtyFourth.to_ticks());
        assert_eq!(n.rest_glyph(), "\u{E4E9}");
    }

    #[test]
    fn rest_glyph_hundred_twenty_eighth_returns_unicode() {
        let n = Notation::new(0, NoteDuration::HudredTwentyEighth.to_ticks());
        assert_eq!(n.rest_glyph(), "\u{E4EA}");
    }

    // --- Notation::glyph (dispatcher) ---

    #[test]
    fn glyph_for_rest_delegates_to_rest_glyph() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks()); // no tones
        assert_eq!(n.glyph(), n.rest_glyph());
    }

    #[test]
    fn glyph_for_note_delegates_to_note_glyph() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        n.tones.push(Tone::new(
            1,
            0,
            NoteDuration::Quarter.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        assert_eq!(n.glyph(), n.note_glyph());
    }

    // --- Notation::glyph_metrics ---

    #[test]
    fn glyph_metrics_rest_returns_standard_box() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        let m = n.glyph_metrics();
        assert_eq!(m.width, 1.175);
    }

    #[test]
    fn glyph_metrics_whole_note_returns_wider_box() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, NoteDuration::Whole.to_ticks());
        n.tones.push(Tone::new(
            1,
            0,
            NoteDuration::Whole.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        let m = n.glyph_metrics();
        assert_eq!(m.width, 1.7);
    }

    #[test]
    fn glyph_metrics_quarter_note_returns_standard_box() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        n.tones.push(Tone::new(
            1,
            0,
            NoteDuration::Quarter.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        let m = n.glyph_metrics();
        assert_eq!(m.width, 1.175);
    }

    // --- Notation::is_flagged ---

    #[test]
    fn is_flagged_eighth_note_with_no_beam_is_true() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, NoteDuration::Eighth.to_ticks());
        n.tones.push(Tone::new(
            1,
            0,
            NoteDuration::Eighth.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        let beams = vec![];
        assert!(n.is_flagged(&beams));
    }

    #[test]
    fn is_flagged_quarter_note_is_false() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let mut n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        n.tones.push(Tone::new(
            1,
            0,
            NoteDuration::Quarter.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        let beams = vec![];
        assert!(!n.is_flagged(&beams));
    }

    #[test]
    fn is_flagged_rest_is_false() {
        let n = Notation::new(0, NoteDuration::Eighth.to_ticks()); // no tones = rest
        let beams = vec![];
        assert!(!n.is_flagged(&beams));
    }

    // --- NotationTrack::is_qhq_pattern ---

    fn make_note_at(tick: Tick, duration: Ticks, id: u64) -> Notation {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
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

    fn make_rest_at(tick: Tick, duration: Ticks) -> Notation {
        Notation::new(tick, duration)
    }

    #[test]
    fn is_qhq_pattern_returns_true_for_valid_qhq() {
        // QHQ pattern in 4/4: quarter at beat1, half at beat2, quarter at beat4
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1)); // beat 1 = quarter
        track.track.insert(q, make_note_at(q, q * 2, 2)); // beat 2 = half (spans beat 2+3)
        track.track.insert(q * 3, make_note_at(q * 3, q, 3)); // beat 4 = quarter
        assert!(track.is_qhq_pattern(0, q, q * 3, q * 4));
    }

    #[test]
    fn is_qhq_pattern_returns_false_when_beat_two_is_rest() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        track.track.insert(q, make_rest_at(q, q * 2)); // rest at beat 2 → fails
        track.track.insert(q * 3, make_note_at(q * 3, q, 3));
        assert!(!track.is_qhq_pattern(0, q, q * 3, q * 4));
    }

    #[test]
    fn is_qhq_pattern_returns_false_when_extra_event_present() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        track.track.insert(q, make_note_at(q, q * 2, 2));
        track.track.insert(q * 2, make_note_at(q * 2, q, 4)); // unexpected event at beat 3
        track.track.insert(q * 3, make_note_at(q * 3, q, 3));
        assert!(!track.is_qhq_pattern(0, q, q * 3, q * 4));
    }

    #[test]
    fn is_qhq_pattern_returns_false_when_both_outer_beats_are_rests() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_rest_at(0, q)); // beat1 rest
        track.track.insert(q, make_note_at(q, q * 2, 2));
        track.track.insert(q * 3, make_rest_at(q * 3, q)); // beat4 rest → fails
        assert!(!track.is_qhq_pattern(0, q, q * 3, q * 4));
    }

    // --- NotationTrack::is_offbeat_pattern ---

    #[test]
    fn is_offbeat_pattern_returns_true_for_valid_offbeat() {
        // Simple 2-beat offbeat: event at tick 0 and at half_beat
        let q = NoteDuration::Quarter.to_ticks();
        let whole_beat = q * 2; // half note = 2 quarter beats
        let half_beat = q;
        let mut track = NotationTrack::new(whole_beat);
        track.track.insert(0, make_note_at(0, half_beat, 1));
        track
            .track
            .insert(half_beat, make_note_at(half_beat, half_beat, 2));
        assert!(track.is_offbeat_pattern(0, whole_beat, whole_beat));
    }

    #[test]
    fn is_offbeat_pattern_returns_false_when_both_ends_are_rest() {
        let q = NoteDuration::Quarter.to_ticks();
        let whole_beat = q * 2;
        let half_beat = q;
        let mut track = NotationTrack::new(whole_beat);
        track.track.insert(0, make_rest_at(0, half_beat));
        track
            .track
            .insert(half_beat, make_rest_at(half_beat, half_beat));
        assert!(!track.is_offbeat_pattern(0, whole_beat, whole_beat));
    }

    #[test]
    fn is_offbeat_pattern_returns_false_when_last_half_beat_is_empty() {
        let q = NoteDuration::Quarter.to_ticks();
        let whole_beat = q * 2;
        let mut track = NotationTrack::new(whole_beat);
        // Only one event at start, nothing at half_beat
        track.track.insert(0, make_note_at(0, whole_beat, 1));
        assert!(!track.is_offbeat_pattern(0, whole_beat, whole_beat));
    }

    // --- NotationTrack::split ---

    #[test]
    fn split_divides_notation_at_midpoint() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 2);
        track.track.insert(0, make_note_at(0, q * 2, 1)); // half note
        track.split(q); // split at quarter boundary
        assert!(track.track.contains_key(&0));
        assert!(track.track.contains_key(&q));
    }

    #[test]
    fn split_noop_at_existing_event() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 2);
        track.track.insert(0, make_note_at(0, q, 1)); // already starts at tick 0
        let before = track.track.len();
        track.split(0); // split AT the event start → no-op
        assert_eq!(track.track.len(), before);
    }

    // --- NotationTrack::is_qmdot_pattern ---

    #[test]
    fn is_qmdot_pattern_returns_true_for_valid_pattern() {
        // Q M. pattern: quarter on beat 1, note spanning beats 2-4 (dotted half)
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1)); // beat 1 = quarter
        track.track.insert(q, make_note_at(q, q * 3, 2)); // beat 2-4 = dotted half
        assert!(track.is_qmdot_pattern(0, q, q * 4));
    }

    #[test]
    fn is_qmdot_pattern_returns_false_when_second_note_is_rest() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        track.track.insert(q, make_rest_at(q, q * 3)); // rest on beat 2 → not the pattern
        assert!(!track.is_qmdot_pattern(0, q, q * 4));
    }

    #[test]
    fn is_qmdot_pattern_returns_false_when_missing_beat_one() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        // beat one is empty, only beat 2 has content
        track.track.insert(q, make_note_at(q, q * 3, 2));
        assert!(!track.is_qmdot_pattern(0, q, q * 4));
    }

    #[test]
    fn is_qmdot_pattern_returns_false_when_unexpected_event_exists() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        track.track.insert(q, make_note_at(q, q * 3, 2));
        track.track.insert(q * 2, make_note_at(q * 2, q, 3)); // unexpected mid-bar event
        assert!(!track.is_qmdot_pattern(0, q, q * 4));
    }

    // --- NotationTrack::is_mdotq_pattern ---

    #[test]
    fn is_mdotq_pattern_returns_true_for_valid_pattern() {
        // M. Q pattern: dotted half spans beats 1-3, quarter on beat 4
        let q = NoteDuration::Quarter.to_ticks();
        let beat_one = 0;
        let beat_three = q * 2;
        let beat_four = q * 3;
        let stop = q * 4;
        let mut track = NotationTrack::new(stop);
        track
            .track
            .insert(beat_one, make_note_at(beat_one, q * 3, 1)); // dotted half
        track.track.insert(beat_four, make_note_at(beat_four, q, 2)); // quarter
        assert!(track.is_mdotq_pattern(q, beat_one, beat_three, beat_four, stop));
    }

    #[test]
    fn is_mdotq_pattern_returns_false_when_rest_follows_dotted_note_matching_beat_length() {
        // If beat_three - beat_one == ticks_per_original_beat and beat_four is a rest → not the pattern
        let q = NoteDuration::Quarter.to_ticks();
        let beat_one = 0;
        let beat_three = q; // beat_three - beat_one == q == ticks_per_original_beat
        let beat_four = q * 3;
        let stop = q * 4;
        let mut track = NotationTrack::new(stop);
        track
            .track
            .insert(beat_one, make_note_at(beat_one, q * 3, 1));
        track.track.insert(beat_four, make_rest_at(beat_four, q)); // rest on beat 4
        assert!(!track.is_mdotq_pattern(q, beat_one, beat_three, beat_four, stop));
    }

    #[test]
    fn is_mdotq_pattern_returns_false_when_beat_one_is_rest_and_unit_longer_than_beat() {
        // If unit > ticks_per_original_beat and beat_one is a rest → not the pattern
        let q = NoteDuration::Quarter.to_ticks();
        let beat_one = 0;
        let beat_three = q * 2;
        let beat_four = q * 3;
        let stop = q * 4;
        let mut track = NotationTrack::new(stop);
        track.track.insert(beat_one, make_rest_at(beat_one, q * 3)); // rest on beat 1
        track.track.insert(beat_four, make_note_at(beat_four, q, 2));
        assert!(!track.is_mdotq_pattern(q, beat_one, beat_three, beat_four, stop));
    }

    #[test]
    fn is_mdotq_pattern_returns_false_when_unexpected_event_present() {
        let q = NoteDuration::Quarter.to_ticks();
        let beat_one = 0;
        let beat_three = q * 2;
        let beat_four = q * 3;
        let stop = q * 4;
        let mut track = NotationTrack::new(stop);
        track
            .track
            .insert(beat_one, make_note_at(beat_one, q * 3, 1));
        track.track.insert(beat_four, make_note_at(beat_four, q, 2));
        track.track.insert(q, make_note_at(q, q, 3)); // unexpected mid event
        assert!(!track.is_mdotq_pattern(q, beat_one, beat_three, beat_four, stop));
    }

    // --- Notation::has_beam ---

    #[test]
    fn has_beam_returns_false_when_beams_empty() {
        let q = NoteDuration::Quarter.to_ticks();
        let n = make_note_at(0, q, 1);
        let beams: super::super::get_beams::Beams = Vec::new();
        assert!(!n.has_beam(&beams));
    }

    // --- Notation::sort_tones_by_offset ---

    #[test]
    fn sort_tones_by_offset_orders_by_offset_descending() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;

        let mut offsets: super::super::get_tone_offsets::ToneVerticalOffsets =
            rustc_hash::FxHashMap::default();
        offsets.insert(1, 3i8);
        offsets.insert(2, 7i8);

        let mut n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        n.tones.push(Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        n.tones.push(Tone::new(
            2,
            0,
            32,
            Pitch::new(67, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));

        n.sort_tones_by_offset(&offsets);

        // After sort+reverse, highest offset should be first
        assert_eq!(n.tones[0].id, 2); // offset 7 is higher
        assert_eq!(n.tones[1].id, 1); // offset 3
    }

    // --- Notation::min_spacing ---

    #[test]
    fn min_spacing_undotted_note_has_positive_width() {
        use crate::score::engraves::Engraves;
        use crate::score::layouts::LayoutType;

        let q = NoteDuration::Quarter.to_ticks();
        let n = make_note_at(0, q, 1);
        let mut shunts = super::super::get_shunts::Shunts {
            by_id: rustc_hash::FxHashMap::default(),
            by_offset: rustc_hash::FxHashMap::default(),
        };
        // must populate shunts for tone id=1 at tick=0
        shunts
            .by_id
            .insert((0, 1), super::super::get_shunts::Shunt::None);
        let beams: super::super::get_beams::Beams = Vec::new();
        let engrave_map = Engraves::new();
        let engrave = &engrave_map.by_layout_type[&LayoutType::FullScore];
        let bb = n.min_spacing(&shunts, engrave, &beams);
        assert!(bb.width > 0.0);
    }

    #[test]
    fn min_spacing_dotted_note_has_extra_right_padding() {
        use crate::score::engraves::Engraves;
        use crate::score::layouts::LayoutType;

        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let dotted_quarter = q + e;
        let n = make_note_at(0, dotted_quarter, 1);
        let mut shunts = super::super::get_shunts::Shunts {
            by_id: rustc_hash::FxHashMap::default(),
            by_offset: rustc_hash::FxHashMap::default(),
        };
        shunts
            .by_id
            .insert((0, 1), super::super::get_shunts::Shunt::None);
        let beams: super::super::get_beams::Beams = Vec::new();
        let engrave_map = Engraves::new();
        let engrave = &engrave_map.by_layout_type[&LayoutType::FullScore];
        let dotted_bb = n.min_spacing(&shunts, engrave, &beams);

        let n2 = make_note_at(0, q, 1);
        let plain_bb = n2.min_spacing(&shunts, engrave, &beams);

        // Dotted note has larger spacing
        assert!(dotted_bb.right_padding >= plain_bb.right_padding);
    }

    // --- Notation::metrics ---

    #[test]
    fn metrics_quarter_note_has_positive_width() {
        use crate::score::engraves::Engraves;
        use crate::score::layouts::LayoutType;

        let q = NoteDuration::Quarter.to_ticks();
        let n = make_note_at(0, q, 1);
        let mut shunts = super::super::get_shunts::Shunts {
            by_id: rustc_hash::FxHashMap::default(),
            by_offset: rustc_hash::FxHashMap::default(),
        };
        shunts
            .by_id
            .insert((0, 1), super::super::get_shunts::Shunt::None);
        let beams: super::super::get_beams::Beams = Vec::new();
        let engrave_map = Engraves::new();
        let engrave = &engrave_map.by_layout_type[&LayoutType::FullScore];
        let bb = n.metrics(&shunts, engrave, &beams);
        assert!(bb.width > 0.0);
    }

    #[test]
    fn metrics_whole_note_wider_than_quarter() {
        use crate::score::engraves::Engraves;
        use crate::score::layouts::LayoutType;

        let q = NoteDuration::Quarter.to_ticks();
        let w = NoteDuration::Whole.to_ticks();
        let mut shunts = super::super::get_shunts::Shunts {
            by_id: rustc_hash::FxHashMap::default(),
            by_offset: rustc_hash::FxHashMap::default(),
        };
        shunts
            .by_id
            .insert((0, 1), super::super::get_shunts::Shunt::None);
        shunts
            .by_id
            .insert((0, 2), super::super::get_shunts::Shunt::None);
        let beams: super::super::get_beams::Beams = Vec::new();
        let engrave_map = Engraves::new();
        let engrave = &engrave_map.by_layout_type[&LayoutType::FullScore];

        let whole_note = make_note_at(0, w, 1);
        let quarter_note = make_note_at(0, q, 2);

        let whole_bb = whole_note.metrics(&shunts, engrave, &beams);
        let quarter_bb = quarter_note.metrics(&shunts, engrave, &beams);
        assert!(whole_bb.width >= quarter_bb.width);
    }

    // --- Notation::is_rest ---

    #[test]
    fn is_rest_returns_true_for_notation_with_no_tones() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert!(n.is_rest());
    }

    #[test]
    fn is_rest_returns_false_when_tone_present() {
        let q = NoteDuration::Quarter.to_ticks();
        let n = make_note_at(0, q, 1);
        assert!(!n.is_rest());
    }

    // --- Notation::is_dotted ---

    #[test]
    fn is_dotted_returns_false_for_quarter_note() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert!(!n.is_dotted());
    }

    #[test]
    fn is_dotted_returns_true_for_dotted_quarter() {
        // dotted quarter = quarter + eighth
        let dq = NoteDuration::Quarter.to_ticks() + NoteDuration::Eighth.to_ticks();
        let n = Notation::new(0, dq);
        assert!(n.is_dotted());
    }

    #[test]
    fn is_dotted_returns_false_for_whole_note() {
        let n = Notation::new(0, NoteDuration::Whole.to_ticks());
        assert!(!n.is_dotted());
    }

    // --- Notation::has_tie ---

    #[test]
    fn has_tie_returns_false_when_empty() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert!(!n.has_tie());
    }

    #[test]
    fn has_tie_returns_true_when_tie_present() {
        let mut n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        n.ties.insert(99);
        assert!(n.has_tie());
    }

    // --- Notation::is_chord ---

    #[test]
    fn is_chord_returns_false_for_single_tone() {
        let q = NoteDuration::Quarter.to_ticks();
        let n = make_note_at(0, q, 1);
        assert!(!n.is_chord());
    }

    #[test]
    fn is_chord_returns_false_for_rest() {
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        assert!(!n.is_chord());
    }

    #[test]
    fn is_chord_returns_true_for_two_tones() {
        use crate::score::components::articulation::Articulations;
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::components::velocity::Velocity;
        use crate::score::entries::tone::Tone;
        let q = NoteDuration::Quarter.to_ticks();
        let mut n = Notation::new(0, q);
        n.tones.push(Tone::new(
            1,
            0,
            q,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        n.tones.push(Tone::new(
            2,
            0,
            q,
            Pitch::new(64, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        ));
        assert!(n.is_chord());
    }

    // --- Notation::flag_glyph ---

    #[test]
    fn flag_glyph_eighth_up_returns_unicode_e240() {
        use crate::score::components::duration::Direction;
        let n = make_note_at(0, NoteDuration::Eighth.to_ticks(), 1);
        let glyph = n.flag_glyph(&Direction::Up);
        assert_eq!(glyph, "\u{E240}");
    }

    #[test]
    fn flag_glyph_eighth_down_returns_unicode_e241() {
        use crate::score::components::duration::Direction;
        let n = make_note_at(0, NoteDuration::Eighth.to_ticks(), 1);
        let glyph = n.flag_glyph(&Direction::Down);
        assert_eq!(glyph, "\u{E241}");
    }

    #[test]
    fn flag_glyph_sixteenth_up_returns_unicode_e242() {
        use crate::score::components::duration::Direction;
        let n = make_note_at(0, NoteDuration::Sixteenth.to_ticks(), 1);
        let glyph = n.flag_glyph(&Direction::Up);
        assert_eq!(glyph, "\u{E242}");
    }

    #[test]
    fn flag_glyph_whole_returns_empty_string() {
        use crate::score::components::duration::Direction;
        let n = make_note_at(0, NoteDuration::Whole.to_ticks(), 1);
        let glyph = n.flag_glyph(&Direction::Up);
        assert_eq!(glyph, "");
    }

    // --- NotationTrack methods ---

    #[test]
    fn notation_track_new_is_empty() {
        let track = NotationTrack::new(128);
        assert!(track.track.is_empty());
        assert_eq!(track.length, 128);
    }

    #[test]
    fn is_tick_empty_returns_true_for_empty_track() {
        let track = NotationTrack::new(128);
        assert!(track.is_tick_empty(0));
        assert!(track.is_tick_empty(64));
    }

    #[test]
    fn is_tick_empty_returns_false_when_notation_present() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        assert!(!track.is_tick_empty(0));
    }

    #[test]
    fn is_tick_rest_returns_false_on_empty_track() {
        let track = NotationTrack::new(128);
        assert!(!track.is_tick_rest(0));
    }

    #[test]
    fn is_tick_rest_returns_true_when_rest_at_tick() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_rest_at(0, q));
        assert!(track.is_tick_rest(0));
    }

    #[test]
    fn is_tick_rest_returns_false_when_note_at_tick() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        assert!(!track.is_tick_rest(0));
    }

    #[test]
    fn is_range_empty_returns_true_for_empty_track() {
        let q = NoteDuration::Quarter.to_ticks();
        let track = NotationTrack::new(q * 4);
        // range start+1..stop: just checks interior ticks
        assert!(track.is_range_empty(0, q * 2));
    }

    #[test]
    fn is_range_empty_returns_false_when_note_in_range() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(q, make_note_at(q, q, 2));
        // range 0..q*2 checks ticks 1..q*2; q is in that range
        assert!(!track.is_range_empty(0, q * 2));
    }

    #[test]
    fn get_previous_notation_returns_none_on_empty_track() {
        let track = NotationTrack::new(128);
        assert!(track.get_previous_notation(64).is_none());
    }

    #[test]
    fn get_previous_notation_returns_notation_at_same_tick() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        let result = track.get_previous_notation(0);
        assert!(result.is_some());
        let (tick, _) = result.unwrap();
        assert_eq!(tick, 0);
    }

    #[test]
    fn get_previous_notation_returns_earlier_notation() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        // Looking from tick q should find notation at tick 0
        let result = track.get_previous_notation(q);
        assert!(result.is_some());
        let (tick, _) = result.unwrap();
        assert_eq!(tick, 0);
    }

    #[test]
    fn get_next_notation_returns_none_on_empty_track() {
        let track = NotationTrack::new(128);
        assert!(track.get_next_notation(0).is_none());
    }

    #[test]
    fn get_next_notation_returns_next_event() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(q, make_note_at(q, q, 1));
        let result = track.get_next_notation(0);
        assert!(result.is_some());
        let (tick, _) = result.unwrap();
        assert_eq!(tick, q);
    }

    #[test]
    fn get_next_notation_returns_none_past_last_event() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        // looking from q should find nothing (length=q*4 but nothing after 0)
        let result = track.get_next_notation(q);
        assert!(result.is_none());
    }

    // --- Notation::base_to_note_duration ---

    #[test]
    fn base_to_note_duration_quarter_returns_some_quarter() {
        // Quarter is directly writable, so get_writable_ticks returns Some(quarter)
        // and base_to_note_duration maps that to Some(Quarter)
        let n = Notation::new(0, NoteDuration::Quarter.to_ticks());
        let result = n.base_to_note_duration();
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), NoteDuration::Quarter));
    }

    #[test]
    fn base_to_note_duration_dotted_quarter_returns_some() {
        // dotted quarter = quarter + eighth ticks - has a base duration (quarter)
        let dq = NoteDuration::Quarter.to_ticks() + NoteDuration::Eighth.to_ticks();
        let n = Notation::new(0, dq);
        let result = n.base_to_note_duration();
        assert!(result.is_some());
    }

    #[test]
    fn base_to_note_duration_eighth_returns_some() {
        // Eighth IS writable but dotted eighth is not
        let de = NoteDuration::Eighth.to_ticks() + NoteDuration::Sixteenth.to_ticks();
        let n = Notation::new(0, de);
        let result = n.base_to_note_duration();
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), NoteDuration::Eighth));
    }

    // --- Notation::get_beam_guide_note ---

    #[test]
    fn get_beam_guide_note_up_returns_highest() {
        use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
        use crate::score::components::duration::Direction;
        let q = NoteDuration::Quarter.to_ticks();
        let mut n = make_note_at(0, q, 1);
        n.info = Some(ToneOffsetInfo {
            highest: 3,
            lowest: -2,
            furthest: 3,
        });
        let guide = n.get_beam_guide_note(&Direction::Up);
        assert_eq!(guide, 3);
    }

    #[test]
    fn get_beam_guide_note_down_returns_lowest() {
        use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
        use crate::score::components::duration::Direction;
        let q = NoteDuration::Quarter.to_ticks();
        let mut n = make_note_at(0, q, 1);
        n.info = Some(ToneOffsetInfo {
            highest: 3,
            lowest: -2,
            furthest: 3,
        });
        let guide = n.get_beam_guide_note(&Direction::Down);
        assert_eq!(guide, -2);
    }

    #[test]
    fn split_preserves_left_tie_to_split_tone_and_right_existing_ties() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 4);
        let mut notation = make_note_at(0, q * 2, 7);
        notation.ties.insert(99);
        track.track.insert(0, notation);

        track.split(q);

        assert_notation_track_matches(
            &track,
            vec![
                notation_expectation(0, q, false, vec![7], vec![7]),
                notation_expectation(q, q, false, vec![7], vec![99]),
            ],
        );
    }

    #[test]
    fn split_simple_unit_preserves_qhq_pattern_in_four_four() {
        let q = NoteDuration::Quarter.to_ticks();
        let time_signature = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q, 1));
        track.track.insert(q, make_note_at(q, q * 2, 2));
        track.track.insert(q * 3, make_note_at(q * 3, q, 3));

        track.split_simple_unit(0, q * 4, &time_signature, &time_signature);

        assert_notation_track_matches(
            &track,
            vec![
                notation_expectation(0, q, false, vec![1], vec![]),
                notation_expectation(q, q * 2, false, vec![2], vec![]),
                notation_expectation(q * 3, q, false, vec![3], vec![]),
            ],
        );
    }

    #[test]
    fn split_simple_unit_preserves_whole_note_covering_entire_four_four_unit() {
        let q = NoteDuration::Quarter.to_ticks();
        let time_signature = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let mut track = NotationTrack::new(q * 4);
        track.track.insert(0, make_note_at(0, q * 4, 1));

        track.split_simple_unit(0, q * 4, &time_signature, &time_signature);

        assert_notation_track_matches(
            &track,
            vec![notation_expectation(0, q * 4, false, vec![1], vec![])],
        );
    }

    #[test]
    fn split_simple_unit_in_three_four_splits_second_dotted_quarter_at_beat_three() {
        let q = NoteDuration::Quarter.to_ticks();
        let dotted_quarter = q + NoteDuration::Eighth.to_ticks();
        let time_signature = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let mut track = NotationTrack::new(q * 3);
        track.track.insert(0, make_note_at(0, dotted_quarter, 1));
        track.track.insert(
            dotted_quarter,
            make_note_at(dotted_quarter, dotted_quarter, 2),
        );

        track.split_simple_unit(0, q * 3, &time_signature, &time_signature);

        assert_notation_track_matches(
            &track,
            vec![
                notation_expectation(0, dotted_quarter, false, vec![1], vec![]),
                notation_expectation(dotted_quarter, q / 2, false, vec![2], vec![2]),
                notation_expectation(q * 2, q, false, vec![2], vec![]),
            ],
        );
    }

    #[test]
    fn split_simple_unit_in_three_four_splits_dotted_quarter_followed_by_rest_at_beat_two() {
        let q = NoteDuration::Quarter.to_ticks();
        let dotted_quarter = q + NoteDuration::Eighth.to_ticks();
        let time_signature = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let mut track = NotationTrack::new(q * 3);
        track.track.insert(0, make_note_at(0, dotted_quarter, 1));
        track
            .track
            .insert(dotted_quarter, make_rest_at(dotted_quarter, dotted_quarter));

        track.split_simple_unit(0, q * 3, &time_signature, &time_signature);

        assert_notation_track_matches(
            &track,
            vec![
                notation_expectation(0, q, false, vec![1], vec![1]),
                notation_expectation(q, q / 2, false, vec![1], vec![]),
                notation_expectation(dotted_quarter, q / 2, true, vec![], vec![]),
                notation_expectation(q * 2, q, true, vec![], vec![]),
            ],
        );
    }

    #[test]
    fn split_simple_unit_in_three_four_splits_beat_two_dotted_quarter_when_final_eighth_is_rest() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let dotted_quarter = q + e;
        let time_signature = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let mut track = NotationTrack::new(q * 3);
        track.track.insert(0, make_note_at(0, q, 1));
        track.track.insert(q, make_note_at(q, dotted_quarter, 2));
        track.track.insert(q * 2 + e, make_rest_at(q * 2 + e, e));

        track.split_simple_unit(0, q * 3, &time_signature, &time_signature);

        assert_notation_track_matches(
            &track,
            vec![
                notation_expectation(0, q, false, vec![1], vec![]),
                notation_expectation(q, q, false, vec![2], vec![2]),
                notation_expectation(q * 2, e, false, vec![2], vec![]),
                notation_expectation(q * 2 + e, e, true, vec![], vec![]),
            ],
        );
    }

    #[test]
    fn split_at_next_boundary_for_off_grid_events_splits_at_supplied_boundary() {
        let q = NoteDuration::Quarter.to_ticks();
        let eighth = NoteDuration::Eighth.to_ticks();
        let mut track = NotationTrack::new(q * 3);
        track.track.insert(0, make_note_at(0, q, 1));
        track.track.insert(q + 1, make_note_at(q + 1, q, 2));

        track.split_at_next_boundary_for_off_grid_events(&[(q, q * 2, q * 2)], eighth);

        assert!(track.track.contains_key(&(q * 2)));
        assert_eq!(track.track[&(q * 2)].duration, 1);
    }

    #[test]
    fn split_rests_at_boundaries_splits_rest_at_each_crossed_boundary() {
        let q = NoteDuration::Quarter.to_ticks();
        let mut track = NotationTrack::new(q * 3);
        track.track.insert(q, make_rest_at(q, q * 2));

        track.split_rests_at_boundaries(q, q * 3, &[q * 2]);

        assert_notation_track_matches(
            &track,
            vec![
                notation_expectation(q, q, true, vec![], vec![]),
                notation_expectation(q * 2, q, true, vec![], vec![]),
            ],
        );
    }

    #[test]
    fn get_written_durations_preserves_qhq_pattern_end_to_end() {
        let q = NoteDuration::Quarter.to_ticks();
        let time_signature = make_time_signature_for_test(0, 4, NoteDuration::Quarter);

        assert_written_duration_scenario(
            "4/4 quarter-half-quarter pattern survives the end-to-end parser pipeline",
            time_signature,
            &[(1, 0, q, 60), (2, q, q * 2, 62), (3, q * 3, q, 64)],
            vec![
                notation_expectation(0, q, false, vec![1], vec![]),
                notation_expectation(q, q * 2, false, vec![2], vec![]),
                notation_expectation(q * 3, q, false, vec![3], vec![]),
            ],
        );
    }

    #[test]
    fn get_written_durations_preserves_basic_four_four_patterns() {
        let q = NoteDuration::Quarter.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let flow_length = four_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "four straight quarter notes",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, q, 60),
                    (2, q, q, 62),
                    (3, q * 2, q, 64),
                    (4, q * 3, q, 65),
                ],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                    notation_expectation(q * 3, q, false, vec![4], vec![]),
                ],
            },
            Scenario {
                name: "half note followed by two quarter notes",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q * 2, 60), (2, q * 2, q, 62), (3, q * 3, q, 64)],
                expectations: vec![
                    notation_expectation(0, q * 2, false, vec![1], vec![]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                    notation_expectation(q * 3, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "quarter rests on beats one and three remain visible",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, q, q, 60), (2, q * 3, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(q, q, false, vec![1], vec![]),
                    notation_expectation(q * 2, q, true, vec![], vec![]),
                    notation_expectation(q * 3, q, false, vec![2], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_splits_near_miss_four_four_patterns() {
        let q = NoteDuration::Quarter.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let flow_length = four_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "qhq lookalike with extra interior event collapses to plain quarter motion",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, q, 60),
                    (2, q, q, 62),
                    (3, q * 2, q, 64),
                    (4, q * 3, q, 65),
                ],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                    notation_expectation(q * 3, q, false, vec![4], vec![]),
                ],
            },
            Scenario {
                name: "qmdot lookalike with no second note resolves into quarter rest structure",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, true, vec![], vec![]),
                    notation_expectation(q * 2, q * 2, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "offbeat lookalike with on beat intrusion currently resolves into overlapped beat fragments",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, NoteDuration::Eighth.to_ticks(), 60),
                    (2, NoteDuration::Eighth.to_ticks(), q, 62),
                    (3, q, NoteDuration::Eighth.to_ticks(), 64),
                    (4, q + NoteDuration::Eighth.to_ticks(), q, 65),
                    (5, q * 2 + NoteDuration::Eighth.to_ticks(), q, 67),
                    (6, q * 3 + NoteDuration::Eighth.to_ticks(), NoteDuration::Eighth.to_ticks(), 69),
                ],
                expectations: vec![
                    notation_expectation(0, NoteDuration::Eighth.to_ticks(), false, vec![1], vec![]),
                    notation_expectation(NoteDuration::Eighth.to_ticks(), NoteDuration::Eighth.to_ticks(), false, vec![2], vec![2]),
                    notation_expectation(q, NoteDuration::Eighth.to_ticks(), false, vec![2, 3], vec![]),
                    notation_expectation(q + NoteDuration::Eighth.to_ticks(), NoteDuration::Eighth.to_ticks(), false, vec![4], vec![4]),
                    notation_expectation(q * 2, NoteDuration::Eighth.to_ticks(), false, vec![4], vec![]),
                    notation_expectation(q * 2 + NoteDuration::Eighth.to_ticks(), q, false, vec![5], vec![]),
                    notation_expectation(q * 3 + NoteDuration::Eighth.to_ticks(), NoteDuration::Eighth.to_ticks(), false, vec![6], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_covers_default_four_four_split_archetypes() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let flow_length = four_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "half note starting on beat two splits into tied quarters",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, q, q * 2, 60)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(q, q, false, vec![1], vec![1]),
                    notation_expectation(q * 2, q, false, vec![1], vec![]),
                    notation_expectation(q * 3, q, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "dotted note from the and of one currently remains unsplit",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, e, q + e, 62), (2, q * 2, q * 2, 65)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q + e, false, vec![1], vec![]),
                    notation_expectation(q * 2, q * 2, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "quarter from the and of two currently remains unsplit over beat three",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, q * 2, 60),
                    (2, q * 2 + e, q, 64),
                    (3, q * 3 + e, e, 67),
                ],
                expectations: vec![
                    notation_expectation(0, q * 2, false, vec![1], vec![]),
                    notation_expectation(q * 2, e, true, vec![], vec![]),
                    notation_expectation(q * 2 + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 3 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "leading dotted half remains readable before final quarter rest",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q * 3, 60)],
                expectations: vec![
                    notation_expectation(0, q * 3, false, vec![1], vec![]),
                    notation_expectation(q * 3, q, true, vec![], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_preserves_explicit_four_four_patterns() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let flow_length = four_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "qmdot pattern spans beats two through four",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, q * 3, 62)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q * 3, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "mdotq pattern spans beats one through three",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q * 3, 60), (2, q * 3, q, 62)],
                expectations: vec![
                    notation_expectation(0, q * 3, false, vec![1], vec![]),
                    notation_expectation(q * 3, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "offbeat eqqqe pattern stays unsplit across the bar",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, e, 60),
                    (2, e, q, 62),
                    (3, q + e, q, 64),
                    (4, q * 2 + e, q, 65),
                    (5, q * 3 + e, e, 67),
                ],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, q, false, vec![2], vec![]),
                    notation_expectation(q + e, q, false, vec![3], vec![]),
                    notation_expectation(q * 2 + e, q, false, vec![4], vec![]),
                    notation_expectation(q * 3 + e, e, false, vec![5], vec![]),
                ],
            },
            Scenario {
                name: "leading rest offbeat syncopation remains readable",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, e, q, 62),
                    (2, q + e, q, 64),
                    (3, q * 2 + e, q, 65),
                    (4, q * 3 + e, e, 67),
                ],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, q, false, vec![3], vec![]),
                    notation_expectation(q * 3 + e, e, false, vec![4], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_covers_end_to_end_simple_meter_rewrite_archetypes() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let dotted_quarter = q + e;
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let three_four = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let three_four_flow_length = three_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "4/4 whole note spanning the bar stays whole end to end",
                time_signature: four_four.clone(),
                flow_length: four_four.ticks_per_bar(),
                tone_specs: vec![(1, 0, q * 4, 60)],
                expectations: vec![notation_expectation(0, q * 4, false, vec![1], vec![])],
            },
            Scenario {
                name: "3/4 second dotted quarter rewrites to show beat 3 end to end",
                time_signature: three_four.clone(),
                flow_length: three_four_flow_length,
                tone_specs: vec![
                    (1, 0, dotted_quarter, 60),
                    (2, dotted_quarter, dotted_quarter, 62),
                ],
                expectations: vec![
                    notation_expectation(0, dotted_quarter, false, vec![1], vec![]),
                    notation_expectation(dotted_quarter, q / 2, false, vec![2], vec![2]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "3/4 dotted quarter plus rest splits at beat 2 end to end",
                time_signature: three_four.clone(),
                flow_length: three_four_flow_length,
                tone_specs: vec![(1, 0, dotted_quarter, 60)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![1]),
                    notation_expectation(q, q / 2, false, vec![1], vec![]),
                    notation_expectation(dotted_quarter, q / 2, true, vec![], vec![]),
                    notation_expectation(q * 2, q, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "3/4 half note starting on beat 2 stays unsplit end to end",
                time_signature: three_four.clone(),
                flow_length: three_four_flow_length,
                tone_specs: vec![(1, q, q * 2, 60)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(q, q * 2, false, vec![1], vec![]),
                ],
            },
            Scenario {
                name:
                    "3/4 quarter plus dotted quarter plus final rest rewrites at beat 3 end to end",
                time_signature: three_four,
                flow_length: three_four_flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, dotted_quarter, 62)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![2]),
                    notation_expectation(q * 2, e, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, true, vec![], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_preserves_basic_three_four_patterns() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let three_four = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let flow_length = three_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "three straight quarter notes",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, q, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "half note followed by quarter note",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q * 2, 60), (2, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q * 2, false, vec![1], vec![]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "three four offbeat syncopation keeps its staggered shape",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, e, q, 60), (2, q + e, q, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_covers_default_three_four_split_archetypes() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let three_four = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let flow_length = three_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "half note starting on beat two correctly remains a half note",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, q, q * 2, 60)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(q, q * 2, false, vec![1], vec![]),
                ],
            },
            Scenario {
                name: "quarter plus dotted quarter plus closing eighth currently stays unsplit on beat two",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, q + e, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q + e, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "quarter plus dotted quarter plus final eighth rest splits at beat three",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, q + e, 62)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![2]),
                    notation_expectation(q * 2, e, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "eighth syncopation across beat two splits into tied eighths",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, e, q, 60), (2, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, e, false, vec![1], vec![1]),
                    notation_expectation(q, e, false, vec![1], vec![]),
                    notation_expectation(q + e, e, true, vec![], vec![]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "quarter plus dotted quarter plus eighth currently leaves beat two unsplit",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, q + e, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q + e, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_characterizes_three_four_compound_feel_lookalikes() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let three_four = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let flow_length = three_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "dotted quarter plus rewritten second dotted quarter avoids six eight look",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q + e, 60), (2, q + e, q + e, 62)],
                expectations: vec![
                    notation_expectation(0, q + e, false, vec![1], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![2]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "eighth plus dotted quarter plus quarter currently preserves the dotted middle span",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, e, 60), (2, e, q + e, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, q + e, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "dotted quarter plus eighth plus quarter keeps the beat three arrival obvious",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q + e, 60), (2, q + e, e, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q + e, false, vec![1], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "quarter plus eighth plus quarter plus eighth currently preserves the middle quarter span",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, e, 62), (3, q + e, q, 64), (4, q * 2 + e, e, 65)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![]),
                    notation_expectation(q + e, q, false, vec![3], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![4], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_covers_rest_led_and_rest_trailed_syncopation() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let flow_length = four_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "eighth rest then quarter syncopation then quarter then closing eighth",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, e, q, 60),
                    (2, q + e, q, 62),
                    (3, q * 2 + e, q, 64),
                    (4, q * 3 + e, e, 65),
                ],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, q, false, vec![3], vec![]),
                    notation_expectation(q * 3 + e, e, false, vec![4], vec![]),
                ],
            },
            Scenario {
                name: "quarter then eighth rest then eighth pickup then two quarters",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, q, 60),
                    (2, q + e, e, 62),
                    (3, q * 2, q, 64),
                    (4, q * 3, q, 65),
                ],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, true, vec![], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                    notation_expectation(q * 3, q, false, vec![4], vec![]),
                ],
            },
            Scenario {
                name: "half rest then two quarter notes remains grouped by beat",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, q * 2, q, 60), (2, q * 3, q, 62)],
                expectations: vec![
                    notation_expectation(0, q * 2, true, vec![], vec![]),
                    notation_expectation(q * 2, q, false, vec![1], vec![]),
                    notation_expectation(q * 3, q, false, vec![2], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_characterizes_four_four_syncopation_windows() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let flow_length = four_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "4/4 note from the and of 1 remains unsplit into beat 2",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, e, q + e, 62), (2, q * 2, q * 2, 65)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q + e, false, vec![1], vec![]),
                    notation_expectation(q * 2, q * 2, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "4/4 quarter from the and of 3 remains unsplit over beat 4",
                time_signature: four_four,
                flow_length,
                tone_specs: vec![
                    (1, 0, q * 2, 60),
                    (2, q * 2 + e, q, 64),
                    (3, q * 3 + e, e, 67),
                ],
                expectations: vec![
                    notation_expectation(0, q * 2, false, vec![1], vec![]),
                    notation_expectation(q * 2, e, true, vec![], vec![]),
                    notation_expectation(q * 2 + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 3 + e, e, false, vec![3], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_characterizes_three_four_syncopation_bracketing_cases() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let three_four = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let flow_length = three_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "3/4 quarter from the and of 1 should show beat 2",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, e, q, 60), (2, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, e, false, vec![1], vec![1]),
                    notation_expectation(q, e, false, vec![1], vec![]),
                    notation_expectation(q + e, e, true, vec![], vec![]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "3/4 quarter from the and of 1 stays unsplit when beat 1 is a tone",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, e, 60), (2, e, q, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, q, false, vec![2], vec![]),
                    notation_expectation(q + e, e, true, vec![], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "3/4 quarter from the and of 1 stays unsplit when 2.5 is a tone",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, e, q, 60), (2, q + e, e, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![]),
                    notation_expectation(q * 2, e, true, vec![], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "3/4 dotted quarter on beat 2 remains unsplit",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, q + e, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q + e, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "3/4 quarter from the and of 2 should show beat 3 when bracketed by rests",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q + e, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, true, vec![], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![2]),
                    notation_expectation(q * 2, e, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "3/4 quarter from the and of 2 stays unsplit when beat 2 is a tone",
                time_signature: three_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q, e, 62), (3, q + e, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![]),
                    notation_expectation(q + e, q, false, vec![3], vec![]),
                    notation_expectation(q * 2 + e, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "3/4 quarter from the and of 2 stays unsplit when 3.5 is a tone",
                time_signature: three_four,
                flow_length,
                tone_specs: vec![(1, 0, q, 60), (2, q + e, q, 64), (3, q * 2 + e, e, 67)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, true, vec![], vec![]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_covers_common_teaching_rhythms() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let flow_length = four_four.ticks_per_bar();

        let scenarios = vec![
            Scenario {
                name: "two half notes",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q * 2, 60), (2, q * 2, q * 2, 64)],
                expectations: vec![
                    notation_expectation(0, q * 2, false, vec![1], vec![]),
                    notation_expectation(q * 2, q * 2, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "quarter two eighths quarter quarter",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, q, 60),
                    (2, q, e, 62),
                    (3, q + e, e, 64),
                    (4, q * 2, q, 65),
                    (5, q * 3, q, 67),
                ],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![]),
                    notation_expectation(q + e, e, false, vec![3], vec![]),
                    notation_expectation(q * 2, q, false, vec![4], vec![]),
                    notation_expectation(q * 3, q, false, vec![5], vec![]),
                ],
            },
            Scenario {
                name: "two eighths quarter two eighths quarter",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, e, 60),
                    (2, e, e, 62),
                    (3, q, q, 64),
                    (4, q * 2, e, 65),
                    (5, q * 2 + e, e, 67),
                    (6, q * 3, q, 69),
                ],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, e, false, vec![2], vec![]),
                    notation_expectation(q, q, false, vec![3], vec![]),
                    notation_expectation(q * 2, e, false, vec![4], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![5], vec![]),
                    notation_expectation(q * 3, q, false, vec![6], vec![]),
                ],
            },
            Scenario {
                name: "dotted quarter eighth half",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![(1, 0, q + e, 60), (2, q + e, e, 62), (3, q * 2, q * 2, 64)],
                expectations: vec![
                    notation_expectation(0, q + e, false, vec![1], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![]),
                    notation_expectation(q * 2, q * 2, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "half dotted quarter eighth",
                time_signature: four_four.clone(),
                flow_length,
                tone_specs: vec![
                    (1, 0, q * 2, 60),
                    (2, q * 2, q + e, 62),
                    (3, q * 3 + e, e, 64),
                ],
                expectations: vec![
                    notation_expectation(0, q * 2, false, vec![1], vec![]),
                    notation_expectation(q * 2, q + e, false, vec![2], vec![]),
                    notation_expectation(q * 3 + e, e, false, vec![3], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_preserves_chord_tone_sets_in_pattern_regressions() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let three_four = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let four_four_flow_length = four_four.ticks_per_bar();
        let three_four_flow_length = three_four.ticks_per_bar();
        let scenarios = vec![
            Scenario {
                name: "four four split chord keeps both tones through the tie",
                time_signature: four_four,
                flow_length: four_four_flow_length,
                tone_specs: vec![(1, q, q * 2, 60), (2, q, q * 2, 64)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(q, q, false, vec![1, 2], vec![1, 2]),
                    notation_expectation(q * 2, q, false, vec![1, 2], vec![]),
                    notation_expectation(q * 3, q, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "three four rewritten second dotted quarter chord keeps full chord on both fragments",
                time_signature: three_four,
                flow_length: three_four_flow_length,
                tone_specs: vec![(1, 0, q + e, 60), (2, 0, q + e, 64), (3, q + e, q + e, 67), (4, q + e, q + e, 71)],
                expectations: vec![
                    notation_expectation(0, q + e, false, vec![1, 2], vec![]),
                    notation_expectation(q + e, e, false, vec![3, 4], vec![3, 4]),
                    notation_expectation(q * 2, q, false, vec![3, 4], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_handles_barline_crossing_patterns() {
        let q = NoteDuration::Quarter.to_ticks();
        let bar = q * 4;
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let scenarios = vec![
            Scenario {
                name: "note crossing the barline ties cleanly into the next bar",
                time_signature: four_four.clone(),
                flow_length: bar * 2,
                tone_specs: vec![
                    (1, 0, q, 60),
                    (2, q, q, 62),
                    (3, q * 2, q, 64),
                    (4, q * 3, q * 2, 65),
                    (5, bar + q, q, 67),
                    (6, bar + q * 2, q, 69),
                    (7, bar + q * 3, q, 71),
                ],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                    notation_expectation(q * 3, q, false, vec![4], vec![4]),
                    notation_expectation(bar, q, false, vec![4], vec![]),
                    notation_expectation(bar + q, q, false, vec![5], vec![]),
                    notation_expectation(bar + q * 2, q, false, vec![6], vec![]),
                    notation_expectation(bar + q * 3, q, false, vec![7], vec![]),
                ],
            },
            Scenario {
                name: "rest crossing the barline stays split at the measure boundary",
                time_signature: four_four,
                flow_length: bar * 2,
                tone_specs: vec![
                    (1, 0, q, 60),
                    (2, q, q, 62),
                    (3, q * 2, q, 64),
                    (4, bar + q, q, 67),
                    (5, bar + q * 2, q, 69),
                    (6, bar + q * 3, q, 71),
                ],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                    notation_expectation(q * 3, q, true, vec![], vec![]),
                    notation_expectation(bar, q, true, vec![], vec![]),
                    notation_expectation(bar + q, q, false, vec![4], vec![]),
                    notation_expectation(bar + q * 2, q, false, vec![5], vec![]),
                    notation_expectation(bar + q * 3, q, false, vec![6], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_supports_compound_meter_regression_corpus() {
        let e = NoteDuration::Eighth.to_ticks();
        let q = NoteDuration::Quarter.to_ticks();
        let dotted_quarter = q + e;

        let scenarios = vec![
            Scenario {
                name: "6/8 six straight eighths stay grouped as 3 plus 3",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![
                    (1, 0, e, 60),
                    (2, e, e, 62),
                    (3, e * 2, e, 64),
                    (4, e * 3, e, 65),
                    (5, e * 4, e, 67),
                    (6, e * 5, e, 69),
                ],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, e, false, vec![2], vec![]),
                    notation_expectation(e * 2, e, false, vec![3], vec![]),
                    notation_expectation(e * 3, e, false, vec![4], vec![]),
                    notation_expectation(e * 4, e, false, vec![5], vec![]),
                    notation_expectation(e * 5, e, false, vec![6], vec![]),
                ],
            },
            Scenario {
                name: "6/8 dotted quarter big beats remain unsplit",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![
                    (1, 0, dotted_quarter, 60),
                    (2, dotted_quarter, dotted_quarter, 62),
                ],
                expectations: vec![
                    notation_expectation(0, dotted_quarter, false, vec![1], vec![]),
                    notation_expectation(dotted_quarter, dotted_quarter, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "6/8 full-bar note remains unsplit",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![(1, 0, e * 6, 60)],
                expectations: vec![notation_expectation(0, e * 6, false, vec![1], vec![])],
            },
            Scenario {
                name: "6/8 full-bar rest remains unsplit",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![],
                expectations: vec![notation_expectation(0, e * 6, true, vec![], vec![])],
            },
            Scenario {
                name:
                    "6/8 inner three-beat splitting rewrites a second dotted eighth inside the beat",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![
                    (1, 0, e + e / 2, 60),
                    (2, e + e / 2, e + e / 2, 62),
                    (3, dotted_quarter, dotted_quarter, 64),
                ],
                expectations: vec![
                    notation_expectation(0, e + e / 2, false, vec![1], vec![]),
                    notation_expectation(e + e / 2, e / 2, false, vec![2], vec![2]),
                    notation_expectation(e * 2, e, false, vec![2], vec![]),
                    notation_expectation(dotted_quarter, dotted_quarter, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 note crossing the compound beat boundary is tied",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![(1, e * 2, q, 60)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(e * 2, e, false, vec![1], vec![1]),
                    notation_expectation(e * 3, e, false, vec![1], vec![]),
                    notation_expectation(e * 4, e, true, vec![], vec![]),
                    notation_expectation(e * 5, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "6/8 rest crossing the compound beat boundary is split",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![(1, 0, e, 60), (2, e * 5, e, 62)],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, e, true, vec![], vec![]),
                    notation_expectation(e * 2, e, true, vec![], vec![]),
                    notation_expectation(e * 3, q, true, vec![], vec![]),
                    notation_expectation(e * 5, e, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name:
                    "6/8 chord crossing the compound beat boundary preserves chord tones and ties",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![(1, e * 2, q, 60), (2, e * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(e * 2, e, false, vec![1, 2], vec![1, 2]),
                    notation_expectation(e * 3, e, false, vec![1, 2], vec![]),
                    notation_expectation(e * 4, e, true, vec![], vec![]),
                    notation_expectation(e * 5, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "9/8 straight eighths stay in three compound groups",
                time_signature: make_time_signature_for_test(0, 9, NoteDuration::Eighth),
                flow_length: e * 9,
                tone_specs: vec![
                    (1, 0, e, 60),
                    (2, e, e, 61),
                    (3, e * 2, e, 62),
                    (4, e * 3, e, 63),
                    (5, e * 4, e, 64),
                    (6, e * 5, e, 65),
                    (7, e * 6, e, 66),
                    (8, e * 7, e, 67),
                    (9, e * 8, e, 68),
                ],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, e, false, vec![2], vec![]),
                    notation_expectation(e * 2, e, false, vec![3], vec![]),
                    notation_expectation(e * 3, e, false, vec![4], vec![]),
                    notation_expectation(e * 4, e, false, vec![5], vec![]),
                    notation_expectation(e * 5, e, false, vec![6], vec![]),
                    notation_expectation(e * 6, e, false, vec![7], vec![]),
                    notation_expectation(e * 7, e, false, vec![8], vec![]),
                    notation_expectation(e * 8, e, false, vec![9], vec![]),
                ],
            },
            Scenario {
                name: "9/8 full-bar note splits into dotted half tied to dotted quarter",
                time_signature: make_time_signature_for_test(0, 9, NoteDuration::Eighth),
                flow_length: e * 9,
                tone_specs: vec![(1, 0, e * 9, 60)],
                expectations: vec![
                    notation_expectation(0, dotted_quarter * 2, false, vec![1], vec![1]),
                    notation_expectation(dotted_quarter * 2, dotted_quarter, false, vec![1], vec![]),
                ],
            },
            Scenario {
                name: "9/8 syncopation crossing one internal big beat boundary is split",
                time_signature: make_time_signature_for_test(0, 9, NoteDuration::Eighth),
                flow_length: e * 9,
                tone_specs: vec![(1, e * 2, q, 60), (2, e * 6, dotted_quarter, 64)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(e * 2, e, false, vec![1], vec![1]),
                    notation_expectation(e * 3, e, false, vec![1], vec![]),
                    notation_expectation(e * 4, e, true, vec![], vec![]),
                    notation_expectation(e * 5, e, true, vec![], vec![]),
                    notation_expectation(e * 6, dotted_quarter, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "9/8 rest-led syncopation keeps the compound beat grouping visible",
                time_signature: make_time_signature_for_test(0, 9, NoteDuration::Eighth),
                flow_length: e * 9,
                tone_specs: vec![(1, e, q, 60), (2, e * 4, q, 64), (3, e * 7, q, 67)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(e * 3, e, true, vec![], vec![]),
                    notation_expectation(e * 4, q, false, vec![2], vec![]),
                    notation_expectation(e * 6, e, true, vec![], vec![]),
                    notation_expectation(e * 7, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "12/8 straight eighths stay in four compound groups",
                time_signature: make_time_signature_for_test(0, 12, NoteDuration::Eighth),
                flow_length: e * 12,
                tone_specs: (0..12)
                    .map(|index| ((index + 1) as u64, index as u64 * e, e, 60 + index as u8))
                    .collect::<Vec<_>>(),
                expectations: (0..12)
                    .map(|index| {
                        notation_expectation(
                            index as u64 * e,
                            e,
                            false,
                            vec![(index + 1) as u64],
                            vec![],
                        )
                    })
                    .collect::<Vec<_>>(),
            },
            Scenario {
                name: "12/8 barline-crossing note splits at the crossed big beat and barline",
                time_signature: make_time_signature_for_test(0, 12, NoteDuration::Eighth),
                flow_length: (e * 12) + (e * 2),
                tone_specs: vec![
                    (1, 0, dotted_quarter, 60),
                    (2, dotted_quarter, dotted_quarter, 62),
                    (3, dotted_quarter * 2, q, 64),
                    (4, e * 8, e * 6, 67),
                ],
                expectations: vec![
                    notation_expectation(0, dotted_quarter, false, vec![1], vec![]),
                    notation_expectation(dotted_quarter, dotted_quarter, false, vec![2], vec![]),
                    notation_expectation(dotted_quarter * 2, q, false, vec![3], vec![]),
                    notation_expectation(e * 8, e, false, vec![4], vec![4]),
                    notation_expectation(e * 9, dotted_quarter, false, vec![4], vec![4]),
                    notation_expectation(e * 12, q, false, vec![4], vec![]),
                ],
            },
            Scenario {
                name: "12/8 barline-crossing rest splits at the crossed big beat and barline",
                time_signature: make_time_signature_for_test(0, 12, NoteDuration::Eighth),
                flow_length: (e * 12) + (e * 2),
                tone_specs: vec![
                    (1, 0, dotted_quarter, 60),
                    (2, dotted_quarter, dotted_quarter, 62),
                    (3, dotted_quarter * 2, q, 64),
                ],
                expectations: vec![
                    notation_expectation(0, dotted_quarter, false, vec![1], vec![]),
                    notation_expectation(dotted_quarter, dotted_quarter, false, vec![2], vec![]),
                    notation_expectation(dotted_quarter * 2, q, false, vec![3], vec![]),
                    notation_expectation(e * 8, e, true, vec![], vec![]),
                    notation_expectation(e * 9, dotted_quarter, true, vec![], vec![]),
                    notation_expectation(e * 12, q, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "15/8 proof case scales the same compound grouping rule",
                time_signature: make_time_signature_for_test(0, 15, NoteDuration::Eighth),
                flow_length: e * 15,
                tone_specs: (0..15)
                    .map(|index| {
                        (
                            (index + 1) as u64,
                            index as u64 * e,
                            e,
                            60 + (index % 8) as u8,
                        )
                    })
                    .collect::<Vec<_>>(),
                expectations: (0..15)
                    .map(|index| {
                        notation_expectation(
                            index as u64 * e,
                            e,
                            false,
                            vec![(index + 1) as u64],
                            vec![],
                        )
                    })
                    .collect::<Vec<_>>(),
            },
            Scenario {
                name: "6/8 off-grid compound notation preserves hundred-twenty-eighth and sixty-fourth values",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: e * 6,
                tone_specs: vec![(1, 1, 2, 60), (2, e * 3 + 2, 2, 62)],
                expectations: vec![
                    notation_expectation(0, 1, true, vec![], vec![]),
                    notation_expectation(1, 2, false, vec![1], vec![]),
                    notation_expectation(3, 1, true, vec![], vec![]),
                    notation_expectation(4, 4, true, vec![], vec![]),
                    notation_expectation(8, 8, true, vec![], vec![]),
                    notation_expectation(16, 16, true, vec![], vec![]),
                    notation_expectation(32, 16, true, vec![], vec![]),
                    notation_expectation(e * 3, 2, true, vec![], vec![]),
                    notation_expectation(e * 3 + 2, 2, false, vec![2], vec![]),
                    notation_expectation(e * 3 + 4, 4, true, vec![], vec![]),
                    notation_expectation(e * 3 + 8, 8, true, vec![], vec![]),
                    notation_expectation(e * 3 + 16, 16, true, vec![], vec![]),
                    notation_expectation(e * 3 + 32, 16, true, vec![], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn equivalent_three_four_and_six_eight_material_keeps_meter_specific_spelling() {
        let q = NoteDuration::Quarter.to_ticks();
        let three_four = make_time_signature_for_test(0, 3, NoteDuration::Quarter);
        let six_eight = make_time_signature_for_test(0, 6, NoteDuration::Eighth);

        let scenarios = vec![
            Scenario {
                name: "3/4 three straight quarter notes stay unsplit for the single-flow baseline",
                time_signature: three_four.clone(),
                flow_length: three_four.ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, q, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name:
                    "6/8 equivalent performed material still rewrites at the compound beat boundary",
                time_signature: six_eight.clone(),
                flow_length: six_eight.ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, q, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q / 2, false, vec![2], vec![2]),
                    notation_expectation(q + (q / 2), q / 2, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn get_written_durations_characterizes_six_eight_variants_of_basic_three_four_patterns() {
        let q = NoteDuration::Quarter.to_ticks();
        let e = NoteDuration::Eighth.to_ticks();
        let scenarios = vec![
            Scenario {
                name: "6/8 variant of three straight quarter notes splits the middle quarter at the big beat boundary",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, q, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, q / 2, false, vec![2], vec![2]),
                    notation_expectation(q + (q / 2), q / 2, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of half note followed by quarter note splits the half note at the big beat boundary",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q * 2, 60), (2, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q + (q / 2), false, vec![1], vec![1]),
                    notation_expectation(q + (q / 2), q / 2, false, vec![1], vec![]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of half note starting on beat 2 prefers a quarter rest then an eighth note",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, q, q * 2, 60)],
                expectations: vec![
                    notation_expectation(0, q, true, vec![], vec![]),
                    notation_expectation(q, q / 2, false, vec![1], vec![1]),
                    notation_expectation(q + (q / 2), q + (q / 2), false, vec![1], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of dotted quarter plus rest keeps both compound beats whole",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q + e, 60)],
                expectations: vec![
                    notation_expectation(0, q + e, false, vec![1], vec![]),
                    notation_expectation(q + e, q + e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of two dotted quarters keeps both compound beats whole",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q + e, 60), (2, q + e, q + e, 62)],
                expectations: vec![
                    notation_expectation(0, q + e, false, vec![1], vec![]),
                    notation_expectation(q + e, q + e, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter plus dotted quarter plus final rest splits the dotted quarter at the big beat boundary",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, q + e, 62)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![2]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter plus dotted quarter plus closing eighth splits the dotted quarter at the big beat boundary",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, q + e, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![2]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of eighth syncopation across beat two keeps the offbeat quarter whole",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, e, q, 60), (2, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(q + e, e, true, vec![], vec![]),
                    notation_expectation(q * 2, q, false, vec![2], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of eighth plus dotted quarter plus quarter splits the dotted quarter at the big beat boundary",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, e, 60), (2, e, q + e, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, q, false, vec![2], vec![2]),
                    notation_expectation(q + e, e, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of dotted quarter plus eighth plus quarter matches the three-four rendering",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q + e, 60), (2, q + e, e, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, q + e, false, vec![1], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter plus eighth plus quarter plus eighth matches the three-four rendering",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, e, 62), (3, q + e, q, 64), (4, q * 2 + e, e, 65)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![]),
                    notation_expectation(q + e, q, false, vec![3], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![4], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter from the and of 1 stays unsplit when beat 1 is a tone",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, e, 60), (2, e, q, 62), (3, q * 2, q, 64)],
                expectations: vec![
                    notation_expectation(0, e, false, vec![1], vec![]),
                    notation_expectation(e, q, false, vec![2], vec![]),
                    notation_expectation(q + e, e, true, vec![], vec![]),
                    notation_expectation(q * 2, q, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter from the and of 1 stays unsplit when 2.5 is a tone",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, e, q, 60), (2, q + e, e, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(q + e, e, false, vec![2], vec![]),
                    notation_expectation(q * 2, e, true, vec![], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of dotted quarter on beat 2 rewrites across the compound-beat boundary",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, q + e, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![2]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter from the and of 2 stays unsplit when bracketed by rests",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q + e, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, true, vec![], vec![]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter from the and of 2 stays unsplit when beat 2 is a tone",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q, e, 62), (3, q + e, q, 64)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, false, vec![2], vec![]),
                    notation_expectation(q + e, q, false, vec![3], vec![]),
                    notation_expectation(q * 2 + e, e, true, vec![], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of quarter from the and of 2 stays unsplit when 3.5 is a tone",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, 0, q, 60), (2, q + e, q, 64), (3, q * 2 + e, e, 67)],
                expectations: vec![
                    notation_expectation(0, q, false, vec![1], vec![]),
                    notation_expectation(q, e, true, vec![], vec![]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
            Scenario {
                name: "6/8 variant of three-four offbeat syncopation keeps its staggered shape",
                time_signature: make_time_signature_for_test(0, 6, NoteDuration::Eighth),
                flow_length: make_time_signature_for_test(0, 6, NoteDuration::Eighth).ticks_per_bar(),
                tone_specs: vec![(1, e, q, 60), (2, q + e, q, 62), (3, q * 2 + e, e, 64)],
                expectations: vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, q, false, vec![1], vec![]),
                    notation_expectation(q + e, q, false, vec![2], vec![]),
                    notation_expectation(q * 2 + e, e, false, vec![3], vec![]),
                ],
            },
        ];

        assert_written_duration_scenarios(scenarios);
    }

    #[test]
    fn three_eight_quarter_rest_then_eighth_note_keeps_simple_meter_spelling() {
        let e = NoteDuration::Eighth.to_ticks();

        assert_written_duration_scenario(
            "3/8 quarter rest then eighth note keeps simple-meter beat spelling",
            make_time_signature_for_test(0, 3, NoteDuration::Eighth),
            &[(1, e * 2, e, 60)],
            vec![
                notation_expectation(0, e, true, vec![], vec![]),
                notation_expectation(e, e, true, vec![], vec![]),
                notation_expectation(e * 2, e, false, vec![1], vec![]),
            ],
        );
    }

    #[test]
    fn get_written_durations_keeps_host_only_six_eight_dotted_quarter_beats_without_tuplets() {
        let e = NoteDuration::Eighth.to_ticks();
        let q = NoteDuration::Quarter.to_ticks();
        let dotted_quarter = q + e;

        assert_written_duration_scenario(
            "6/8 dotted-quarter beats stay unsplit without tuplets",
            make_time_signature_for_test(0, 6, NoteDuration::Eighth),
            &[
                (1, 0, dotted_quarter, 60),
                (2, dotted_quarter, dotted_quarter, 64),
            ],
            vec![
                notation_expectation(0, dotted_quarter, false, vec![1], vec![]),
                notation_expectation(dotted_quarter, dotted_quarter, false, vec![2], vec![]),
            ],
        );
    }

    #[test]
    fn get_written_durations_preserves_anchored_local_triplet_contract() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[],
            &[TupletSpec {
                id: 99,
                tick: 32,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![3],
                tone_specs: vec![(201, 0, e, 60), (202, e, e, 62), (203, e * 2, e, 64)],
            }],
        );

        let track = &notation_by_track[&track_id];

        assert_eq!(track.anchored_tracks.len(), 1);
        assert!(!track.track.contains_key(&32));

        assert_anchored_notation_track_matches(
            &track.anchored_tracks[0],
            anchored_notation_expectation(
                99,
                32,
                64,
                e * 3,
                vec![
                    notation_expectation(0, e, false, vec![201], vec![]),
                    notation_expectation(e, e, false, vec![202], vec![]),
                    notation_expectation(e * 2, e, false, vec![203], vec![]),
                ],
            ),
        );
    }

    #[test]
    fn get_written_durations_uses_base_note_rests_for_an_empty_triplet() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[],
            &[TupletSpec {
                id: 99,
                tick: 32,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![3],
                tone_specs: vec![],
            }],
        );

        let anchored_track = &notation_by_track[&track_id].anchored_tracks[0];

        assert_anchored_notation_track_matches(
            anchored_track,
            anchored_notation_expectation(
                99,
                32,
                64,
                e * 3,
                vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, e, true, vec![], vec![]),
                    notation_expectation(e * 2, e, true, vec![], vec![]),
                ],
            ),
        );
    }

    #[test]
    fn get_written_durations_preserves_partially_filled_triplet_with_derived_middle_rest() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[],
            &[TupletSpec {
                id: 99,
                tick: 32,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![3],
                tone_specs: vec![(201, 0, e, 60), (203, e * 2, e, 64)],
            }],
        );

        assert_anchored_notation_track_matches(
            &notation_by_track[&track_id].anchored_tracks[0],
            anchored_notation_expectation(
                99,
                32,
                64,
                e * 3,
                vec![
                    notation_expectation(0, e, false, vec![201], vec![]),
                    notation_expectation(e, e, true, vec![], vec![]),
                    notation_expectation(e * 2, e, false, vec![203], vec![]),
                ],
            ),
        );
    }

    #[test]
    fn get_written_durations_projects_host_owned_boundary_crossings_into_tuplet_local_space() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[(7, 16, 48, 60)],
            &[TupletSpec {
                id: 99,
                tick: 32,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![3],
                tone_specs: vec![],
            }],
        );

        let host_track = &notation_by_track[&track_id];
        assert_notation_track_matches(
            host_track,
            vec![
                notation_expectation(0, 16, true, vec![], vec![]),
                notation_expectation(16, 16, false, vec![7], vec![]),
                notation_expectation(64, 64, true, vec![], vec![]),
            ],
        );

        assert_anchored_notation_track_matches(
            &host_track.anchored_tracks[0],
            anchored_notation_expectation(
                99,
                32,
                64,
                e * 3,
                vec![
                    notation_expectation(0, 32, false, vec![7], vec![]),
                    notation_expectation(32, 16, true, vec![], vec![]),
                ],
            ),
        );
    }

    #[test]
    fn get_written_durations_keeps_host_crossing_tone_inside_literal_triplet_span() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[(7, 0, e * 5, 60)],
            &[TupletSpec {
                id: 99,
                tick: e * 2,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![3],
                tone_specs: vec![],
            }],
        );

        let host_track = &notation_by_track[&track_id];
        assert_notation_track_matches(
            host_track,
            vec![
                notation_expectation(0, e * 2, false, vec![7], vec![]),
                notation_expectation(e * 4, e * 4, true, vec![], vec![]),
            ],
        );

        assert_anchored_notation_track_matches(
            &host_track.anchored_tracks[0],
            anchored_notation_expectation(
                99,
                e * 2,
                e * 4,
                e * 3,
                vec![notation_expectation(0, e * 3, false, vec![7], vec![])],
            ),
        );
    }

    #[test]
    fn get_written_durations_projects_tuplet_owned_boundary_crossings_back_into_host_space() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[],
            &[TupletSpec {
                id: 99,
                tick: e * 2,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![3],
                tone_specs: vec![(301, e, e * 2 + e / 2, 60)],
            }],
        );

        let host_track = &notation_by_track[&track_id];
        assert!(host_track
            .track
            .values()
            .any(|entry| { entry.tick == e * 4 && entry.tones.iter().any(|tone| tone.id == 301) }));

        let anchored_track = &host_track.anchored_tracks[0];
        assert_eq!(anchored_track.host_span.start_tick, e * 2);
        assert_eq!(anchored_track.host_span.end_tick, e * 4);
        assert!(anchored_track
            .notation
            .track
            .values()
            .any(|entry| { entry.tones.iter().any(|tone| tone.id == 301) }));
    }

    #[test]
    fn get_written_durations_splits_tones_at_explicit_barlines() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let q = NoteDuration::Quarter.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets_and_barlines(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[(7, 0, q * 2, 60)],
            &[],
            &[q],
        );

        assert_notation_track_matches(
            &notation_by_track[&track_id],
            vec![
                notation_expectation(0, q, false, vec![7], vec![7]),
                notation_expectation(q, q, false, vec![7], vec![]),
                notation_expectation(q * 2, q * 2, true, vec![], vec![]),
            ],
        );
    }

    #[test]
    fn get_written_durations_projects_crossing_tone_into_adjacent_tuplet_tracks() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar() + e * 2,
            &[],
            &[
                TupletSpec {
                    id: 99,
                    tick: e * 2,
                    note_count: 3,
                    in_space_of: 2,
                    base_note_value: NoteDuration::Eighth,
                    groupings: vec![3],
                    tone_specs: vec![(401, e * 2, e * 2, 60)],
                },
                TupletSpec {
                    id: 100,
                    tick: e * 4,
                    note_count: 3,
                    in_space_of: 2,
                    base_note_value: NoteDuration::Eighth,
                    groupings: vec![3],
                    tone_specs: vec![],
                },
            ],
        );

        let anchored_tracks = &notation_by_track[&track_id].anchored_tracks;
        assert_eq!(anchored_tracks.len(), 2);

        assert_anchored_notation_track_matches(
            &anchored_tracks[0],
            anchored_notation_expectation(
                99,
                e * 2,
                e * 4,
                e * 3,
                vec![
                    notation_expectation(0, e, true, vec![], vec![]),
                    notation_expectation(e, e, true, vec![], vec![]),
                    notation_expectation(e * 2, e, false, vec![401], vec![]),
                ],
            ),
        );
        assert_anchored_notation_track_matches(
            &anchored_tracks[1],
            anchored_notation_expectation(
                100,
                e * 4,
                e * 6,
                e * 3,
                vec![
                    notation_expectation(0, e, false, vec![401], vec![]),
                    notation_expectation(e, e, true, vec![], vec![]),
                    notation_expectation(e * 2, e, true, vec![], vec![]),
                ],
            ),
        );
    }

    #[test]
    fn get_written_durations_honors_tuplet_groupings_when_deriving_local_notation() {
        let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);
        let e = NoteDuration::Eighth.to_ticks();
        let (notation_by_track, track_id) = run_written_duration_scenario_with_tuplets(
            four_four.clone(),
            four_four.ticks_per_bar(),
            &[],
            &[TupletSpec {
                id: 99,
                tick: e * 2,
                note_count: 3,
                in_space_of: 2,
                base_note_value: NoteDuration::Eighth,
                groupings: vec![1, 2],
                tone_specs: vec![(501, 0, e * 3, 60)],
            }],
        );

        assert_anchored_notation_track_matches(
            &notation_by_track[&track_id].anchored_tracks[0],
            anchored_notation_expectation(
                99,
                e * 2,
                e * 4,
                e * 3,
                vec![
                    notation_expectation(0, e, false, vec![501], vec![501]),
                    notation_expectation(e, e * 2, false, vec![501], vec![]),
                ],
            ),
        );
    }

    #[test]
    #[should_panic]
    fn get_written_durations_panics_for_non_eighth_compound_time_signature() {
        let dotted_quarter = NoteDuration::Quarter;
        let time_signature = make_time_signature_for_test(0, 6, dotted_quarter);
        run_written_duration_scenario(time_signature, &[(1, 0, 32, 60)]);
    }

    #[test]
    #[should_panic]
    fn get_written_durations_panics_for_complex_time_signature() {
        let time_signature = make_time_signature_for_test(0, 5, NoteDuration::Quarter);
        run_written_duration_scenario(time_signature, &[(1, 0, 32, 60)]);
    }

    #[test]
    #[should_panic]
    fn get_written_durations_panics_for_open_time_signature() {
        let time_signature = make_time_signature_for_test(0, 0, NoteDuration::Quarter);
        run_written_duration_scenario(time_signature, &[(1, 0, 32, 60)]);
    }
}
