use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SamplerInstrument {
    pub volume: u8, // 0-127
    pub solo: bool,
    pub mute: bool,
}

impl Default for SamplerInstrument {
    fn default() -> Self {
        Self {
            volume: 80,
            solo: false,
            mute: false,
        }
    }
}

pub type Sampler = FxHashMap<UniqueId, SamplerInstrument>;
