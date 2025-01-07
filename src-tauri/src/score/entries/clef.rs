use serde::{Deserialize, Serialize};

use crate::score::components::{
    duration::Tick,
    pitch::{Accidental, Pitch},
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClefDrawType {
    Hidden,
    G,
    F,
    C,
    Percussion,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Clef {
    pub tick: Tick,
    pub draw_as: ClefDrawType,
    pub pitch: Pitch, // the pitch that the clef sits on
    pub offset: i8,   // visual offset from middle stave line
}

impl Clef {
    pub fn new(tick: Tick, pitch: u8, offset: i8, draw_as: ClefDrawType) -> Self {
        Self {
            tick,
            draw_as,
            pitch: Pitch::new(pitch, Accidental::Natural),
            offset,
        }
    }
}
