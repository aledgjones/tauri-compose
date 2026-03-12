use crate::renderer::converter::Space;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::components::measurements::BoundingBox;
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;
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

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum TimeSignatureDrawType {
    Regular = 1,         // 4/4 etc
    CommonTime = 2,      // C
    SplitCommonTime = 3, // ¢
    Open = 4,            // X
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TimeSignature {
    pub id: UniqueId,
    pub tick: Tick,
    pub hidden: bool,
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
            TimeSignatureDrawType::Regular,
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
            hidden: false,
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
        } else if beats > 3 && beats.is_multiple_of(3) {
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

    pub fn groupings_to_ticks(&self, start: Tick) -> Vec<Tick> {
        let mut output = vec![start];

        let mut acc = start;
        for group in &self.groupings {
            acc += *group as Tick * self.ticks_per_beat();
            output.push(acc);
        }

        output
    }

    pub fn get_tick_at_beat(&self, start: Tick, beat: u8) -> Tick {
        start + (((beat - 1) as Tick) * self.beat_type.to_ticks())
    }

    // Returns the number of ticks per beat
    pub fn ticks_per_beat(&self) -> Ticks {
        self.beat_type.to_ticks()
    }

    // Returns the number of ticks per bar
    pub fn ticks_per_bar(&self) -> Ticks {
        self.beat_type.to_ticks() * (self.beats as Ticks)
    }

    // Returns true if the tick is on a beat
    pub fn is_on_beat(&self, tick: Tick) -> bool {
        self.is_on_beat_type(tick, &self.beat_type)
    }

    /// Return true if a tick is on an arbitrary beat type
    pub fn is_on_beat_type(&self, tick: Tick, beat_type: &NoteDuration) -> bool {
        let ticks_per_beat = beat_type.to_ticks();
        (tick - self.tick).is_multiple_of(ticks_per_beat)
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
        if self.hidden {
            return BoundingBox {
                width: 0.0,
                height: 4.0,
                right_padding: 0.0,
            };
        }

        let beats = self.beats_to_glyph();
        let beat_type = self.beat_type_to_glyph();

        BoundingBox {
            width: max(beats.chars().count(), beat_type.chars().count()) as Space * 1.7,
            height: 4.0,
            right_padding: 1.0,
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

fn time_signature_draw_type_is_valid(
    time_signature: &TimeSignature,
    draw_type: TimeSignatureDrawType,
) -> bool {
    match draw_type {
        TimeSignatureDrawType::CommonTime => {
            time_signature.beats == 4 && matches!(time_signature.beat_type, NoteDuration::Quarter)
        }
        TimeSignatureDrawType::SplitCommonTime => {
            time_signature.beats == 2 && matches!(time_signature.beat_type, NoteDuration::Half)
        }
        _ => true,
    }
}

fn clear_time_signature_at_tick(master: &mut Track, tick: Tick) -> bool {
    if tick == 0 {
        return false;
    }

    let Some(found) = master.get_time_signature_at_tick(&tick).cloned() else {
        return false;
    };

    master.remove_entry(found.id).is_some()
}

fn update_time_signature_draw_type(
    master: &mut Track,
    time_signature_id: UniqueId,
    draw_type: TimeSignatureDrawType,
    hidden: bool,
) -> Result<bool, String> {
    let Some(entry) = master.entries.by_id.get_mut(&time_signature_id) else {
        return Err(String::from("Time signature not found"));
    };

    let Entry::TimeSignature(time_signature) = entry else {
        return Err(String::from("Selected entry is not a time signature"));
    };

    if !time_signature_draw_type_is_valid(time_signature, draw_type) {
        return Err(String::from(
            "Requested draw type is invalid for this time signature",
        ));
    }

    time_signature.draw_type = draw_type;
    time_signature.hidden = hidden;

    Ok(true)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_time_signature_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    tick: Tick,
    beats: u8,
    beat_type: NoteDuration,
    draw_type: TimeSignatureDrawType,
    hidden: bool,
) -> Option<UniqueId> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let time_signature_id = state.generate_id();

    let (flow_master_id, flow_length) = {
        let flow = &state.score.flows.by_id[&flow_id];
        (flow.master_id, flow.length)
    };

    let ticks_per_bar = {
        let master = state.score.tracks.by_id.get_mut(&flow_master_id).unwrap();

        // remove old time signature if defined
        if let Some(old) = master.get_time_signature_at_tick(&tick) {
            master.remove_entry(old.id);
        };

        // insert the new time signature
        let mut new =
            TimeSignature::new(time_signature_id, tick, beats, beat_type, draw_type, None);
        new.hidden = hidden;
        let ticks_per_bar = new.ticks_per_bar();
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

    state.emit_affected_flow(&app, &mut renderer, flow_id);

    Some(time_signature_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_time_signature_clear(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    tick: Tick,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    clear_time_signature_at_tick(master, tick);
    state.ui.sequencer.selection = None;

    state.emit_affected_flow(&app, &mut renderer, flow_id);
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_time_signature_update(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    flow_id: UniqueId,
    time_signature_id: UniqueId,
    draw_type: TimeSignatureDrawType,
    hidden: bool,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    update_time_signature_draw_type(master, time_signature_id, draw_type, hidden)?;

    state.emit_affected_flow(&app, &mut renderer, flow_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_beats_and_beat_type() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beats, 4);
        assert_eq!(ts.id, 1);
        assert_eq!(ts.tick, 0);
    }

    #[test]
    fn new_with_explicit_groupings_uses_them() {
        let ts = TimeSignature::new(
            2,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            Some(vec![3, 3]),
        );
        assert_eq!(ts.groupings, vec![3, 3]);
    }

    #[test]
    fn default_creates_4_4_hidden() {
        let ts = TimeSignature::default();
        assert_eq!(ts.beats, 4);
    }

    // --- kind() ---

    #[test]
    fn kind_open_when_zero_beats() {
        let ts = TimeSignature::new(
            1,
            0,
            0,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Open,
            None,
        );
        assert!(matches!(ts.kind(), TimeSignatureType::Open));
    }

    #[test]
    fn kind_simple_for_four_four() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(matches!(ts.kind(), TimeSignatureType::Simple));
    }

    #[test]
    fn kind_compound_for_six_eight() {
        let ts = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(matches!(ts.kind(), TimeSignatureType::Compound));
    }

    #[test]
    fn kind_complex_for_five_four() {
        let ts = TimeSignature::new(
            1,
            0,
            5,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(matches!(ts.kind(), TimeSignatureType::Complex));
    }

    #[test]
    fn kind_complex_for_seven_eight() {
        let ts = TimeSignature::new(
            1,
            0,
            7,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(matches!(ts.kind(), TimeSignatureType::Complex));
    }

    #[test]
    fn kind_simple_for_two_four() {
        let ts = TimeSignature::new(
            1,
            0,
            2,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(matches!(ts.kind(), TimeSignatureType::Simple));
    }

    #[test]
    fn kind_simple_for_three_four() {
        let ts = TimeSignature::new(
            1,
            0,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(matches!(ts.kind(), TimeSignatureType::Simple));
    }

    // --- ticks_per_beat / ticks_per_bar ---

    #[test]
    fn ticks_per_beat_quarter_is_32() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.ticks_per_beat(), 32);
    }

    #[test]
    fn ticks_per_beat_eighth_is_16() {
        let ts = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.ticks_per_beat(), 16);
    }

    #[test]
    fn ticks_per_beat_half_is_64() {
        let ts = TimeSignature::new(
            1,
            0,
            2,
            NoteDuration::Half,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.ticks_per_beat(), 64);
    }

    #[test]
    fn ticks_per_bar_four_four_is_128() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.ticks_per_bar(), 128);
    }

    #[test]
    fn clear_time_signature_at_tick_keeps_opening_signature() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            10,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));

        assert!(!clear_time_signature_at_tick(&mut track, 0));
        assert!(track.get_time_signature_at_tick(&0).is_some());
    }

    #[test]
    fn clear_time_signature_at_tick_removes_non_zero_entry() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            10,
            64,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));

        assert!(clear_time_signature_at_tick(&mut track, 64));
        assert!(track.get_time_signature_at_tick(&64).is_none());
    }

    #[test]
    fn update_time_signature_draw_type_applies_valid_symbol_forms() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            10,
            64,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));

        let result = update_time_signature_draw_type(
            &mut track,
            10,
            TimeSignatureDrawType::CommonTime,
            false,
        );

        assert!(result.is_ok());
        assert_eq!(
            track.get_time_signature_at_tick(&64).unwrap().draw_type,
            TimeSignatureDrawType::CommonTime
        );
    }

    #[test]
    fn update_time_signature_draw_type_rejects_invalid_symbol_forms() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;

        let mut track = Track::new(1);
        track.insert_entry(Entry::TimeSignature(TimeSignature::new(
            10,
            64,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));

        let result = update_time_signature_draw_type(
            &mut track,
            10,
            TimeSignatureDrawType::CommonTime,
            false,
        );

        assert!(result.is_err());
        assert_eq!(
            track.get_time_signature_at_tick(&64).unwrap().draw_type,
            TimeSignatureDrawType::Regular
        );
    }

    #[test]
    fn ticks_per_bar_three_four_is_96() {
        let ts = TimeSignature::new(
            1,
            0,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.ticks_per_bar(), 96);
    }

    #[test]
    fn ticks_per_bar_six_eight_is_96() {
        let ts = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.ticks_per_bar(), 96);
    }

    // --- is_on_beat / is_on_beat_type ---

    #[test]
    fn is_on_beat_true_at_bar_start() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(ts.is_on_beat(0));
    }

    #[test]
    fn is_on_beat_true_at_second_beat() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(ts.is_on_beat(32));
    }

    #[test]
    fn is_on_beat_false_at_eighth() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(!ts.is_on_beat(16));
    }

    #[test]
    fn is_on_beat_type_eighth_at_8th_boundary() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert!(ts.is_on_beat_type(16, &NoteDuration::Eighth));
    }

    // --- get_tick_at_beat ---

    #[test]
    fn get_tick_at_beat_one_is_start() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.get_tick_at_beat(0, 1), 0);
    }

    #[test]
    fn get_tick_at_beat_two_is_one_quarter() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.get_tick_at_beat(0, 2), 32);
    }

    #[test]
    fn get_tick_at_beat_four_is_three_quarters() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.get_tick_at_beat(0, 4), 96);
    }

    // --- groupings_to_ticks ---

    #[test]
    fn groupings_to_ticks_4_4_default() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        let ticks = ts.groupings_to_ticks(0);
        // groups: [2,2] quarters each group 2*32=64 ticks
        assert_eq!(ticks[0], 0);
        assert!(ticks.len() > 1);
    }

    #[test]
    fn groupings_to_ticks_respects_start_offset() {
        let ts = TimeSignature::new(
            1,
            128,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        let ticks = ts.groupings_to_ticks(128);
        assert_eq!(ticks[0], 128);
    }

    // --- number_to_glyph ---

    #[test]
    fn number_to_glyph_single_digit() {
        assert_eq!(TimeSignature::number_to_glyph(4), "\u{E084}");
    }

    #[test]
    fn number_to_glyph_zero() {
        assert_eq!(TimeSignature::number_to_glyph(0), "\u{E080}");
    }

    #[test]
    fn number_to_glyph_double_digit() {
        // 12 = '1' + '2'
        assert_eq!(TimeSignature::number_to_glyph(12), "\u{E081}\u{E082}");
    }

    // --- beats_to_glyph / beat_type_to_glyph ---

    #[test]
    fn beats_to_glyph_four() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beats_to_glyph(), "\u{E084}");
    }

    #[test]
    fn beat_type_to_glyph_quarter_is_four() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E084}");
    }

    #[test]
    fn beat_type_to_glyph_half_is_two() {
        let ts = TimeSignature::new(
            1,
            0,
            2,
            NoteDuration::Half,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E082}");
    }

    #[test]
    fn beat_type_to_glyph_eighth_is_eight() {
        let ts = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E088}");
    }

    #[test]
    fn beat_type_to_glyph_whole_is_one() {
        let ts = TimeSignature::new(
            1,
            0,
            1,
            NoteDuration::Whole,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E081}");
    }

    #[test]
    fn beat_type_to_glyph_sixteenth_is_16() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Sixteenth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E081}\u{E086}");
    }

    #[test]
    fn beat_type_to_glyph_thirty_second_is_32() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::ThirtySecond,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E083}\u{E082}");
    }

    #[test]
    fn beat_type_to_glyph_sixty_fourth_is_64() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::SixtyFourth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E086}\u{E084}");
    }

    #[test]
    fn beat_type_to_glyph_hundred_twenty_eighth_is_128() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::HudredTwentyEighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.beat_type_to_glyph(), "\u{E081}\u{E082}\u{E088}");
    }

    // --- metrics ---

    #[test]
    fn metrics_4_4_has_positive_width() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        let bb = ts.metrics();
        assert!(bb.width > 0.0);
        assert!((bb.right_padding - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hidden_metrics_have_zero_width_and_padding() {
        let mut ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        ts.hidden = true;
        let bb = ts.metrics();
        assert!((bb.width - 0.0).abs() < f32::EPSILON);
        assert!((bb.right_padding - 0.0).abs() < f32::EPSILON);
    }

    // --- Track methods ---

    #[test]
    fn get_time_signature_at_tick_returns_none_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        assert!(track.get_time_signature_at_tick(&0).is_none());
    }

    #[test]
    fn get_time_signature_at_tick_returns_ts_when_present() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ts = TimeSignature::new(
            7,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        track.insert_entry(Entry::TimeSignature(ts));
        let found = track.get_time_signature_at_tick(&0).unwrap();
        assert_eq!(found.id, 7);
    }

    #[test]
    fn get_time_signature_at_tick_returns_none_for_wrong_tick() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ts = TimeSignature::new(
            7,
            64,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        track.insert_entry(Entry::TimeSignature(ts));
        assert!(track.get_time_signature_at_tick(&0).is_none());
    }

    #[test]
    fn get_time_signature_after_tick_returns_none_on_empty_track() {
        use crate::score::tracks::Track;
        let track = Track::new(1);
        assert!(track.get_time_signature_after_tick(0, 256).is_none());
    }

    #[test]
    fn get_time_signature_after_tick_finds_next_ts() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ts = TimeSignature::new(
            7,
            128,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        track.insert_entry(Entry::TimeSignature(ts));
        let found = track.get_time_signature_after_tick(0, 256).unwrap();
        assert_eq!(found.id, 7);
    }

    #[test]
    fn get_time_signature_after_tick_does_not_return_at_same_tick() {
        use crate::score::entries::Entry;
        use crate::score::tracks::Track;
        let mut track = Track::new(1);
        let ts = TimeSignature::new(
            7,
            64,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        track.insert_entry(Entry::TimeSignature(ts));
        // after_tick(64, ...) checks 65..length
        assert!(track.get_time_signature_after_tick(64, 256).is_none());
    }

    // --- default groupings ---

    #[test]
    fn default_groupings_one_beat_is_one_group() {
        let ts = TimeSignature::new(
            1,
            0,
            1,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.groupings, vec![1]);
    }

    #[test]
    fn default_groupings_three_beats_is_one_group() {
        let ts = TimeSignature::new(
            1,
            0,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.groupings, vec![3]);
    }

    #[test]
    fn default_groupings_four_beats_is_two_twos() {
        let ts = TimeSignature::new(
            1,
            0,
            4,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.groupings, vec![2, 2]);
    }

    #[test]
    fn default_groupings_six_beats_is_two_threes() {
        let ts = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        assert_eq!(ts.groupings, vec![3, 3]);
    }

    #[test]
    fn default_groupings_five_beats_complex() {
        let ts = TimeSignature::new(
            1,
            0,
            5,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        // 5 = complex: 3 + 2
        assert_eq!(ts.groupings.iter().sum::<u8>(), 5);
    }
}
