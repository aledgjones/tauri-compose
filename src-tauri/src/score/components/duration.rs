use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Tick = u64;
pub type Ticks = u64;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn to_modifier(&self) -> i8 {
        match self {
            Direction::Up => -1,
            Direction::Down => 1,
        }
    }
}

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
    /// convert ticks to NoteDuration. this may fail so wrap in option
    pub fn from_ticks(ticks: Ticks, subdivisions: Ticks) -> Option<NoteDuration> {
        if ticks == subdivisions * 4 {
            return Some(NoteDuration::Whole);
        }

        if ticks == subdivisions * 2 {
            return Some(NoteDuration::Half);
        }

        if ticks == subdivisions {
            return Some(NoteDuration::Quarter);
        }

        if ticks == subdivisions / 2 {
            return Some(NoteDuration::Eighth);
        }

        if ticks == subdivisions / 4 {
            return Some(NoteDuration::Sixteenth);
        }

        if ticks == subdivisions / 8 {
            return Some(NoteDuration::ThirtySecond);
        }

        if ticks == subdivisions / 16 {
            return Some(NoteDuration::SixtyFourth);
        }

        if ticks == subdivisions / 32 {
            return Some(NoteDuration::HudredTwentyEighth);
        }

        None
    }

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
