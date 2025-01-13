use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Tick = u64;
pub type Ticks = u64;

#[derive(Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NoteDuration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    HudredTwentyEighth,
}
