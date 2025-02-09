use crate::score::components::duration::Tick;
use crate::score::components::measurements::BoundingBox;
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;

use super::Entry;

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

    pub fn metrics(&self) -> BoundingBox {
        match self.barline_type {
            BarlineDrawType::Single => BoundingBox {
                width: 0.0,
                height: 4.0,
            },
            BarlineDrawType::Double => BoundingBox {
                width: 0.5,
                height: 4.0,
            },
            BarlineDrawType::EndRepeat => BoundingBox {
                width: 2.0,
                height: 4.0,
            },
            BarlineDrawType::EndStartRepeat => BoundingBox {
                width: 3.5,
                height: 4.0,
            },
            BarlineDrawType::StartRepeat => BoundingBox {
                width: 2.0,
                height: 4.0,
            },
            BarlineDrawType::Final => BoundingBox {
                width: 1.0,
                height: 4.0,
            },
        }
    }
}

impl Track {
    /// Returns the barline entry at a given tick if it exists
    pub fn get_barline_at_tick(&self, at: &Tick) -> Option<&Barline> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::Barline(barline)) = self.entries.by_id.get(key) {
                return Some(barline);
            }
        }

        None
    }
}
