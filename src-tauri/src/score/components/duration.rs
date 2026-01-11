use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Tick = u64;
pub type Ticks = u64;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn to_modifier(self) -> i8 {
        match self {
            Direction::Up => -1,
            Direction::Down => 1,
        }
    }
}

#[derive(Clone, PartialEq, Serialize_repr, Deserialize_repr)]
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
    pub fn from_ticks(ticks: Ticks) -> Option<NoteDuration> {
        match ticks {
            128 => Some(NoteDuration::Whole),
            64 => Some(NoteDuration::Half),
            32 => Some(NoteDuration::Quarter),
            16 => Some(NoteDuration::Eighth),
            8 => Some(NoteDuration::Sixteenth),
            4 => Some(NoteDuration::ThirtySecond),
            2 => Some(NoteDuration::SixtyFourth),
            1 => Some(NoteDuration::HudredTwentyEighth),
            _ => None,
        }
    }

    pub fn to_ticks(&self) -> Ticks {
        match self {
            NoteDuration::Whole => 128,
            NoteDuration::Half => 64,
            NoteDuration::Quarter => 32,
            NoteDuration::Eighth => 16,
            NoteDuration::Sixteenth => 8,
            NoteDuration::ThirtySecond => 4,
            NoteDuration::SixtyFourth => 2,
            NoteDuration::HudredTwentyEighth => 1,
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
            NoteDuration::HudredTwentyEighth => panic!("Cannot halve HudredTwentyEighth duration"),
        }
    }

    pub fn spacing_ratio(&self, ratio: f32, is_dotted: bool) -> f32 {
        let base = match self {
            NoteDuration::Whole => ratio * 2.0,
            NoteDuration::Half => ratio,
            NoteDuration::Quarter => 1.0,
            NoteDuration::Eighth => 1.0 / ratio,
            NoteDuration::Sixteenth => 1.0 / (ratio * 2.0),
            NoteDuration::ThirtySecond => 1.0 / (ratio * 4.0),
            NoteDuration::SixtyFourth => 1.0 / (ratio * 8.0),
            NoteDuration::HudredTwentyEighth => 1.0 / (ratio * 16.0),
        };

        if is_dotted {
            base * 1.2
        } else {
            base
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
