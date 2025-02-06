use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Tick = u64;
pub type Ticks = u64;

#[derive(Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NoteDuration {
    Whole = 1,
    Half = 2,
    Quarter = 4,
    Eighth = 8,
    Sixteenth = 16,
    ThirtySecond = 32,
    SixtyFourth = 64,
    HudredTwentyEighth = 128,
}

impl NoteDuration {
    pub fn to_ticks(&self, subdivisions: Ticks) -> Ticks {
        match self {
            NoteDuration::Whole => subdivisions * 4,
            NoteDuration::Half => subdivisions * 2,
            NoteDuration::Quarter => subdivisions,
            NoteDuration::Eighth => subdivisions / 2,
            NoteDuration::Sixteenth => subdivisions / 4,
            NoteDuration::ThirtySecond => subdivisions / 8,
            NoteDuration::SixtyFourth => subdivisions / 16,
            NoteDuration::HudredTwentyEighth => subdivisions / 32,
        }
    }

    pub fn to_glyph(&self) -> &str {
        match self {
            NoteDuration::Whole => "\u{1D15D}",
            NoteDuration::Half => "\u{1D15E}",
            NoteDuration::Quarter => "\u{1D15F}",
            NoteDuration::Eighth => "\u{1D160}",
            NoteDuration::Sixteenth => "\u{1D161}",
            NoteDuration::ThirtySecond => "\u{1D162}",
            NoteDuration::SixtyFourth => "\u{1D162}",
            NoteDuration::HudredTwentyEighth => "\u{1D162}",
        }
    }

    pub fn to_notehead_glyph(&self) -> &str {
        match self {
            NoteDuration::Whole => "\u{E0A4}",
            NoteDuration::Half => "\u{E0A3}",
            NoteDuration::Quarter => "\u{E0A4}",
            NoteDuration::Eighth => "\u{E0A4}",
            NoteDuration::Sixteenth => "\u{E0A4}",
            NoteDuration::ThirtySecond => "\u{E0A4}",
            NoteDuration::SixtyFourth => "\u{E0A4}",
            NoteDuration::HudredTwentyEighth => "\u{E0A4}",
        }
    }

    pub fn half(&self) -> NoteDuration {
        match self {
            NoteDuration::Whole => NoteDuration::Half,
            NoteDuration::Half => NoteDuration::Quarter,
            NoteDuration::Quarter => NoteDuration::Eighth,
            NoteDuration::Eighth => NoteDuration::Sixteenth,
            NoteDuration::Sixteenth => NoteDuration::ThirtySecond,
            NoteDuration::ThirtySecond => NoteDuration::SixtyFourth,
            NoteDuration::SixtyFourth => NoteDuration::HudredTwentyEighth,
            NoteDuration::HudredTwentyEighth => NoteDuration::HudredTwentyEighth,
        }
    }
}

pub const NOTE_DURATIONS: [NoteDuration; 8] = [
    NoteDuration::Whole,
    NoteDuration::Half,
    NoteDuration::Quarter,
    NoteDuration::Eighth,
    NoteDuration::Sixteenth,
    NoteDuration::ThirtySecond,
    NoteDuration::SixtyFourth,
    NoteDuration::HudredTwentyEighth,
];
