use crate::score::components::duration::Tick;
use crate::score::components::duration::Ticks;
use crate::score::components::duration::NOTE_DURATIONS;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::entries::time_signature::TimeSignatureDrawType;
use crate::score::entries::tone::Tone;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Track;
use crate::score::tracks::Tracks;
use crate::utils::UniqueId;

use super::get_bars::Bars;
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
    fn is_writable(&self, subdivisions: Ticks) -> bool {
        for option in NOTE_DURATIONS {
            if self.duration == option.to_ticks(subdivisions) {
                return true;
            }
        }

        let base_duration = (self.duration as f32 / 3.0) * 2.0;

        if base_duration.fract() != 0.0 {
            return false;
        }

        for option in NOTE_DURATIONS {
            if base_duration as Ticks == option.to_ticks(subdivisions) {
                return true;
            }
        }

        false
    }

    pub fn is_rest(&self) -> bool {
        self.tones.is_empty()
    }

    pub fn longest_written_duration(&self, subdivisions: Ticks) -> Ticks {
        for option in NOTE_DURATIONS {
            let ticks = option.to_ticks(subdivisions);
            if ticks < self.duration {
                return ticks;
            }
        }

        0
    }
}

pub struct NotationTrack {
    pub length: u64,
    pub track: FxHashMap<Tick, Notation>,
}

impl NotationTrack {
    pub fn new(length: Ticks) -> Self {
        let mut track = FxHashMap::default();
        track.insert(
            0,
            Notation {
                tick: 0,
                tones: Vec::new(),
                duration: length,
                ties: FxHashSet::default(),
            },
        );
        Self { length, track }
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

    pub fn insert(&mut self, tick: Tick, notation: Notation) {
        self.track.insert(tick, notation);
    }

    pub fn add_tone(&mut self, tick: &Tick, tone: &Tone) {
        let entry = self.track.get_mut(tick).unwrap();
        entry.tones.push(tone.clone());
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

                self.insert(
                    event_at,
                    Notation {
                        tick: event_at,
                        tones: tones.clone(),
                        duration: split_at - event_at,
                        ties: tones.iter().map(|tone| tone.id).collect(),
                    },
                );

                self.insert(
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

    pub fn is_range_empty(&self, start: Tick, stop: Tick) -> bool {
        for tick in start + 1..stop {
            match self.track.contains_key(&tick) {
                true => return false,
                false => continue,
            }
        }

        true
    }

    pub fn is_tick_empty(&self, tick: Tick) -> bool {
        !self.track.contains_key(&tick)
    }

    pub fn is_tick_rest(&self, tick: &Tick) -> bool {
        match self.track.get(tick) {
            Some(entry) => entry.is_rest(),
            None => false,
        }
    }

    // [qhq] patterns dont't split middle
    pub fn is_qhq_pattern(
        &self,
        start: Tick,
        time_signature: &TimeSignature,
        subdivisions: Ticks,
    ) -> bool {
        let beat_one = start;
        let beat_two = time_signature.get_tick_at_beat(start, 2, subdivisions);
        let beat_four = time_signature.get_tick_at_beat(start, 4, subdivisions);

        !self.is_tick_empty(beat_one)
            && !self.is_tick_empty(beat_two)
            && !self.is_tick_empty(beat_four)
            && (!self.is_tick_rest(&beat_one) || !self.is_tick_rest(&beat_four))
            && !self.is_tick_rest(&beat_two)
            && self.is_range_empty(beat_one, beat_two)
            && self.is_range_empty(beat_two, beat_four)
    }

    /// [qm.] patterns don't split middle
    pub fn is_qmdot_pattern(
        &self,
        start: Tick,
        time_signature: &TimeSignature,
        subdivisions: Ticks,
    ) -> bool {
        let beat_one = start;
        let beat_two = time_signature.get_tick_at_beat(start, 2, subdivisions);
        let end = start + time_signature.ticks_per_bar(subdivisions);

        !self.is_tick_empty(beat_one)
            && !self.is_tick_empty(beat_two)
            && !self.is_tick_rest(&beat_two)
            && self.is_range_empty(beat_one, beat_two)
            && self.is_range_empty(beat_two, end)
    }

    /// [m.q] patterns don't split middle
    pub fn is_mdotq_pattern(
        &self,
        start: Tick,
        time_signature: &TimeSignature,
        original_time_signature: &TimeSignature,
        subdivisions: Ticks,
    ) -> bool {
        let beat_one = start;
        let beat_four = time_signature.get_tick_at_beat(start, 4, subdivisions);

        let middle = start + (time_signature.ticks_per_bar(subdivisions) / 2);

        let is_pattern = !self.is_tick_empty(beat_one)
            && !self.is_tick_empty(beat_four)
            && self.is_range_empty(beat_one, beat_four);

        let are_both_notes = !self.is_tick_rest(&beat_one) && !self.is_tick_rest(&beat_four);

        let is_allowed_with_rest = !self.is_tick_rest(&beat_one)
            && self.is_tick_rest(&beat_four)
            && match self.track.get(&beat_one) {
                Some(entry) => {
                    entry.duration
                        != (original_time_signature.ticks_per_beat(subdivisions) as f32 * 1.5)
                            as Ticks
                }
                None => false,
            };

        let intersect_beat = original_time_signature.is_on_beat(middle, subdivisions);

        is_pattern && (are_both_notes || is_allowed_with_rest || !intersect_beat)
    }

    // time_signature represents the unit as if it were a "measure"
    pub fn split_unit(
        &mut self,
        start: Tick,
        time_signature: &TimeSignature,
        original_time_signature: &TimeSignature,
        subdivisions: Ticks,
    ) {
        let stop = start + time_signature.ticks_per_bar(subdivisions);

        // we stop once there are no events in the range
        if self.is_range_empty(start, stop) {
            return;
        }

        // convert 2 beats to 4 at hgher fidelity
        if time_signature.beats == 2 {
            return self.split_unit(
                start,
                &TimeSignature::new(
                    0,
                    time_signature.tick,
                    4,
                    time_signature.beat_type.half(),
                    TimeSignatureDrawType::Regular,
                    None,
                ),
                original_time_signature,
                subdivisions,
            );
        }

        match time_signature.beats {
            3 => {
                let beat_one = start;
                let beat_two = time_signature.get_tick_at_beat(start, 2, subdivisions);
                let beat_three = time_signature.get_tick_at_beat(start, 3, subdivisions);

                // split all rests at beats
                for beat in [beat_one, beat_two, beat_three] {
                    if let Some((_, entry)) = self.get_previous_notation(beat) {
                        if entry.is_rest() {
                            self.split(beat);
                        }
                    }
                }

                // make sure it doesn't look compound! (c. at end of bar)
                let middle = start + (time_signature.ticks_per_bar(subdivisions) / 2);
                if !self.is_tick_empty(middle) && self.is_range_empty(middle, stop) {
                    self.split(beat_three);
                };

                // allow sustaining two beats into one
                if !self.is_range_empty(beat_one, beat_two) {
                    self.split(beat_two);
                }

                // if we haven't made any splits we split at the third beat
                if !self.is_tick_empty(beat_two) && !self.is_tick_empty(beat_three) {
                    self.split(beat_three);
                }

                let next = TimeSignature::new(
                    0,
                    time_signature.tick,
                    4,
                    time_signature.beat_type.half().half(),
                    TimeSignatureDrawType::Regular,
                    None,
                );

                self.split_unit(beat_one, &next, original_time_signature, subdivisions);
                self.split_unit(beat_two, &next, original_time_signature, subdivisions);
                self.split_unit(beat_three, &next, original_time_signature, subdivisions);
            }
            4 => {
                let middle = start + (time_signature.ticks_per_bar(subdivisions) / 2);

                if !self.is_qhq_pattern(start, time_signature, subdivisions)
                    && !self.is_qmdot_pattern(start, time_signature, subdivisions)
                    && !self.is_mdotq_pattern(
                        start,
                        time_signature,
                        original_time_signature,
                        subdivisions,
                    )
                {
                    self.split(middle);
                }

                let next = TimeSignature::new(
                    0,
                    time_signature.tick,
                    4,
                    time_signature.beat_type.half(),
                    TimeSignatureDrawType::Regular,
                    None,
                );

                self.split_unit(start, &next, original_time_signature, subdivisions);
                self.split_unit(middle, &next, original_time_signature, subdivisions);
            }
            _ => (),
        }
    }

    pub fn split_as_per_meter(&mut self, barlines: &Bars, subdivisions: Ticks) {
        for (tick, time_signature) in barlines {
            self.split_unit(*tick, time_signature, time_signature, subdivisions);
        }
    }

    pub fn split_at_barlines(&mut self, barlines: &Bars) {
        for tick in barlines.keys() {
            self.split(*tick);
        }
    }

    pub fn split_at_tone_events(&mut self, track: &Track) {
        for tick in 0..self.length {
            let tones = track.get_tones_at_tick(&tick);
            if !tones.is_empty() {
                self.split(tick);
                for tone in tones {
                    self.split(tick + tone.duration);
                    for inner_tick in tick..tick + tone.duration {
                        if self.track.contains_key(&inner_tick) {
                            self.add_tone(&inner_tick, tone);
                        }
                    }
                }
            }
        }
    }

    pub fn split_unwritable(&mut self, barlines: &Bars, subdivisions: Ticks) {
        for (i, time) in barlines {
            for tick in *i..*i + time.ticks_per_bar(subdivisions) {
                if let Some(entry) = self.track.get(&tick) {
                    if !entry.is_rest() && !entry.is_writable(subdivisions) {
                        let longest = entry.longest_written_duration(subdivisions);
                        self.split(tick + longest);
                    }
                };
            }
        }
    }
}

pub fn get_written_durations(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
    bars: &Bars,
) -> NotationByTrack {
    let mut entries = NotationByTrack::default();

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];

            for track_id in &stave.track_ids {
                let track = &tracks.by_id[track_id];
                let notation = track.to_notation_track(&flow.length, bars, flow.subdivisions);
                entries.insert(track.id, notation);
            }
        }
    }

    entries
}

impl Track {
    pub fn to_notation_track(
        &self,
        flow_length: &Ticks,
        barlines: &Bars,
        subdivisions: Ticks,
    ) -> NotationTrack {
        let mut notation = NotationTrack::new(*flow_length);
        notation.split_at_tone_events(self);
        notation.split_at_barlines(barlines);
        notation.split_as_per_meter(barlines, subdivisions);
        notation.split_unwritable(barlines, subdivisions);
        notation
    }
}
