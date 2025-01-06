pub mod config;
pub mod meta;
pub mod players;

use config::Config;
use meta::Meta;
use players::Players;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Score {
    ticks_per_quarter: u8,
    config: Config,
    meta: Meta,
    players: Players,
}

impl Score {
    pub fn new() -> Self {
        Self {
            ticks_per_quarter: 128,
            config: Config::new(),
            meta: Meta::new(),
            players: Players::new(),
        }
    }
}
