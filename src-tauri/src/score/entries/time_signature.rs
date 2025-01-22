use crate::score::components::duration::{NoteDuration, Tick};
use crate::utils::UniqueId;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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
}
