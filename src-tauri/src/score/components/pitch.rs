use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Accidental {
    DoubleSharp = 0,
    Sharp,
    Natural,
    Flat,
    DoubleFlat,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Pitch {
    pub int: u8, // the midi number
    pub accidental: Accidental,
}

impl Pitch {
    pub fn new(int: u8, accidental: Accidental) -> Self {
        Pitch { int, accidental }
    }
}
