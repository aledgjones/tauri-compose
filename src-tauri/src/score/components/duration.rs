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
