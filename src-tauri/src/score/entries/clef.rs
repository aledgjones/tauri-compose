use crate::score::components::pitch::{Accidental, Pitch};
use crate::score::components::{duration::Tick, measurements::BoundingBox};
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::Entry;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ClefDrawType {
    Hidden,
    G,
    F,
    C,
    Percussion,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Clef {
    pub id: UniqueId,
    pub tick: Tick,
    pub draw_as: ClefDrawType,
    pub pitch: Pitch, // the pitch that the clef represents (treble => G, Alto => C etc.)
    pub offset: i8,   // visual offset from middle stave line
}

impl Clef {
    pub fn new(id: UniqueId, tick: Tick, pitch: Pitch, offset: i8, draw_as: ClefDrawType) -> Self {
        Self {
            id,
            tick,
            draw_as,
            pitch,
            offset,
        }
    }

    pub fn metrics(&self) -> BoundingBox {
        match self.draw_as {
            ClefDrawType::Hidden => BoundingBox {
                width: 0.0,
                height: 4.0,
            },
            _ => BoundingBox {
                width: 2.8,
                height: 4.0,
            },
        }
    }

    pub fn glyph(&self) -> Option<String> {
        match self.draw_as {
            ClefDrawType::Hidden => None,
            ClefDrawType::G => Some(String::from("\u{E050}")),
            ClefDrawType::F => Some(String::from("\u{E062}")),
            ClefDrawType::C => Some(String::from("\u{E05C}")),
            ClefDrawType::Percussion => Some(String::from("\u{E069}")),
        }
    }
}

impl Default for Clef {
    fn default() -> Self {
        Clef::new(
            0,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::C,
        )
    }
}

impl Track {
    /// Returns the time signature entry at a given tick if it exists
    pub fn get_clef_at_tick(&self, at: &Tick) -> Option<&Clef> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::Clef(time_signature)) = self.entries.by_id.get(key) {
                return Some(time_signature);
            }
        }

        None
    }
}
