pub mod components;
pub mod config;
pub mod entries;
pub mod instruments;
pub mod meta;
pub mod players;

use config::Config;
use instruments::Instruments;
use meta::Meta;
use players::Players;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Score {
    ticks_per_quarter: u8,
    config: Config,
    meta: Meta,
    players: Players,
    instruments: Instruments,
}

impl Score {
    pub fn new() -> Self {
        Self {
            ticks_per_quarter: 128,
            config: Config::new(),
            meta: Meta::new(),
            players: Players::new(),
            instruments: Instruments::new(),
        }
    }
}
