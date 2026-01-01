use crate::renderer::converter::Space;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::components::measurements::BoundingBox;
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::AppState;
use crate::RenderState;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::max;
use tauri::State;

use super::Entry;

pub enum TimeSignatureType {
    Simple,
    Compound,
    Complex,
    Open,
}

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub enum TimeSignatureDrawType {
    Hidden,          // always hidden
    Regular,         // 4/4 etc
    CommonTime,      // C
    SplitCommonTime, // ¢
    Open,            // X
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TimeSignature {
    pub id: UniqueId,
    pub tick: Tick,
    pub beats: u8,
    pub beat_type: NoteDuration,
    pub draw_type: TimeSignatureDrawType,
    pub groupings: Vec<u8>,
}

impl Default for TimeSignature {
    fn default() -> Self {
        TimeSignature::new(
            0,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Hidden,
            None,
        )
    }
}

impl TimeSignature {
    pub fn new(
        id: UniqueId,
        tick: Tick,
        beats: u8,
        beat_type: NoteDuration,
        draw_type: TimeSignatureDrawType,
        groupings: Option<Vec<u8>>,
    ) -> Self {
        Self {
            id,
            tick,
            beats,
            beat_type,
            groupings: match groupings {
                Some(groupings) => groupings,
                None => TimeSignature::default_groupings(beats),
            },
            draw_type,
        }
    }

    /// Return the time signature type Open, Compound, Simple or Complex.
    pub fn kind(&self) -> TimeSignatureType {
        TimeSignature::kind_from_beats(self.beats)
    }

    /// Return the time signature type Open, Compound, Simple or Complex.
    fn kind_from_beats(beats: u8) -> TimeSignatureType {
        if beats == 0 {
            TimeSignatureType::Open
        } else if beats > 3 && beats % 3 == 0 {
            TimeSignatureType::Compound
        } else if beats == 1 || beats == 2 || beats == 3 || beats == 4 {
            TimeSignatureType::Simple
        } else {
            TimeSignatureType::Complex
        }
    }

    fn default_groupings(beats: u8) -> Vec<u8> {
        if beats > 0 && beats <= 3 {
            vec![beats]
        } else {
            match TimeSignature::kind_from_beats(beats) {
                TimeSignatureType::Simple => vec![2; (beats as usize) / 2],
                TimeSignatureType::Compound => vec![3; (beats as usize) / 3],
                TimeSignatureType::Complex => {
                    let mut out: Vec<u8> = Vec::new();
                    let mut remaining = beats;
                    while remaining > 4 {
                        out.push(3);
                        remaining -= 3;
                    }
                    out.push(remaining);
                    out
                }
                TimeSignatureType::Open => vec![2, 2],
            }
        }
    }

    pub fn groupings_to_ticks(&self, start: Tick, subdivisions: Ticks) -> Vec<Tick> {
        let mut output = vec![start];

        let mut acc = start;
        for group in &self.groupings {
            acc += *group as Tick * self.ticks_per_beat(subdivisions);
            output.push(acc);
        }

        output
    }

    pub fn get_tick_at_beat(&self, start: Tick, beat: u8, subdivisions: Ticks) -> Tick {
        start + (((beat - 1) as Tick) * self.beat_type.to_ticks(subdivisions))
    }

    // Returns the number of ticks per beat
    pub fn ticks_per_beat(&self, subdivisions: Ticks) -> Ticks {
        self.beat_type.to_ticks(subdivisions)
    }

    // Returns the number of ticks per bar
    pub fn ticks_per_bar(&self, subdivisions: Ticks) -> Ticks {
        self.beat_type.to_ticks(subdivisions) * (self.beats as Ticks)
    }

    /// Returns how far away the tick is from the nearest barline
    pub fn distance_from_barline(&self, tick: Tick, subdivisions: Ticks) -> Ticks {
        match self.kind() {
            TimeSignatureType::Open => tick - self.tick,
            _ => (tick - self.tick) % self.ticks_per_bar(subdivisions),
        }
    }

    pub fn is_on_first_beat(&self, tick: Tick, subdivisions: Ticks) -> bool {
        self.distance_from_barline(tick, subdivisions) == 0
    }

    // Returns true if the tick is on a beat
    pub fn is_on_beat(&self, tick: Tick, subdivisions: Ticks) -> bool {
        self.is_on_beat_type(tick, &self.beat_type, subdivisions)
    }

    /// Return true if a tick is on an arbitrary beat type
    pub fn is_on_beat_type(
        &self,
        tick: Tick,
        beat_type: &NoteDuration,
        subdivisions: Ticks,
    ) -> bool {
        let ticks_per_beat = beat_type.to_ticks(subdivisions);
        ((tick - self.tick) % ticks_per_beat) == 0
    }

    fn digit_to_glyph(input: char) -> String {
        match input {
            '0' => String::from("\u{E080}"),
            '1' => String::from("\u{E081}"),
            '2' => String::from("\u{E082}"),
            '3' => String::from("\u{E083}"),
            '4' => String::from("\u{E084}"),
            '5' => String::from("\u{E085}"),
            '6' => String::from("\u{E086}"),
            '7' => String::from("\u{E087}"),
            '8' => String::from("\u{E088}"),
            '9' => String::from("\u{E089}"),
            _ => String::new(),
        }
    }

    pub fn number_to_glyph(input: u8) -> String {
        let mut output: Vec<String> = Vec::new();
        let str = input.to_string();
        for digit in str.chars() {
            output.push(TimeSignature::digit_to_glyph(digit));
        }
        output.concat()
    }

    pub fn beats_to_glyph(&self) -> String {
        TimeSignature::number_to_glyph(self.beats)
    }

    pub fn beat_type_to_glyph(&self) -> String {
        let number = match &self.beat_type {
            NoteDuration::Whole => 1,
            NoteDuration::Half => 2,
            NoteDuration::Quarter => 4,
            NoteDuration::Eighth => 8,
            NoteDuration::Sixteenth => 16,
            NoteDuration::ThirtySecond => 32,
            NoteDuration::SixtyFourth => 64,
            NoteDuration::HudredTwentyEighth => 128,
        };
        TimeSignature::number_to_glyph(number)
    }

    pub fn metrics(&self) -> BoundingBox {
        let beats = self.beats_to_glyph();
        let beat_type = self.beat_type_to_glyph();

        BoundingBox {
            width: (max(beats.chars().count(), beat_type.chars().count()) as Space * 1.7) + 1.0,
            height: 4.0,
        }
    }
}

impl Track {
    /// Returns the time signature entry at a given tick if it exists
    pub fn get_time_signature_at_tick(&self, at: &Tick) -> Option<&TimeSignature> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::TimeSignature(time_signature)) = self.entries.by_id.get(key) {
                return Some(time_signature);
            }
        }

        None
    }

    /// Returns the next time signature entry *after* a given tick if it exists
    pub fn get_time_signature_after_tick(
        &self,
        tick: Tick,
        length: Ticks,
    ) -> Option<&TimeSignature> {
        for i in tick + 1..length {
            if let Some(time_signature) = self.get_time_signature_at_tick(&i) {
                return Some(time_signature);
            };
        }

        None
    }
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_time_signature_create(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    tick: Tick,
    beats: u8,
    beat_type: NoteDuration,
    draw_type: TimeSignatureDrawType,
) -> Option<UniqueId> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let time_signature_id = state.generate_id();

    let (flow_master_id, flow_length, flow_subdivisions) = {
        let flow = &state.score.flows.by_id[&flow_id];
        (flow.master_id, flow.length, flow.subdivisions)
    };

    let ticks_per_bar = {
        let master = state.score.tracks.by_id.get_mut(&flow_master_id).unwrap();

        // remove old time signature if defined
        if let Some(old) = master.get_time_signature_at_tick(&tick) {
            master.remove_entry(old.id);
        };

        // insert the new time signature
        let new = TimeSignature::new(time_signature_id, tick, beats, beat_type, draw_type, None);
        let ticks_per_bar = new.ticks_per_bar(flow_subdivisions);
        master.insert_entry(Entry::TimeSignature(new));

        ticks_per_bar
    };

    // calculate difference in flow length witht the new time signature in place
    let next_time_signature_tick = {
        let master = &state.score.tracks.by_id[&flow_master_id];
        match master.get_time_signature_after_tick(tick, flow_length) {
            Some(entry) => entry.tick,
            None => flow_length,
        }
    };

    let overflow = (next_time_signature_tick - tick) % ticks_per_bar;

    if overflow > 0 {
        let fill = ticks_per_bar - overflow;

        let new_flow_length = {
            let flow = state.score.flows.by_id.get_mut(&flow_id).unwrap();
            flow.length += fill;
            flow.length
        };

        let master = state.score.tracks.by_id.get_mut(&flow_master_id).unwrap();
        for tick in (tick + 1..new_flow_length).rev() {
            if let Some(time_signature) = master.get_time_signature_at_tick(&tick) {
                let id = time_signature.id;
                let tick = time_signature.tick + fill;
                master.shift_entry(id, tick);
            };
        }
    }

    state.emit(&app, &mut renderer, true);

    Some(time_signature_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_time_signature_clear(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    renderer_state: State<'_, RenderState>,
    flow_id: UniqueId,
    tick: Tick,
) {
    if tick == 0 {
        return;
    }

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    if let Some(old) = master.get_time_signature_at_tick(&tick) {
        master.remove_entry(old.id);
    };

    state.emit(&app, &mut renderer, true);
}
