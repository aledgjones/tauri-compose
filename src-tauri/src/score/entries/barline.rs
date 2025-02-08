use crate::score::components::duration::Tick;
use crate::utils::UniqueId;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BarlineDrawType {
    Single,
    Double,
    EndRepeat,
    EndStartRepeat,
    StartRepeat,
    Final,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Barline {
    pub id: UniqueId,
    pub tick: Tick,
    pub barline_type: BarlineDrawType,
}

impl Barline {
    pub fn new(id: UniqueId, tick: Tick, barline_type: BarlineDrawType) -> Self {
        Self {
            id,
            tick,
            barline_type,
        }
    }
}
