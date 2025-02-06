use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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

    /// Returns the time signature entry *at* or *before* a given tick if it exists
    pub fn get_time_signature_before_tick_inclusive(&self, tick: Tick) -> Option<&TimeSignature> {
        for i in (0..tick + 1).rev() {
            if let Some(time_signature) = self.get_time_signature_at_tick(&i) {
                return Some(time_signature);
            };
        }

        None
    }
}
