pub mod config;
pub mod meta;
pub mod players;

use config::Config;
use meta::Meta;
use players::Players;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Score {
    config: Config,
    meta: Meta,
    players: Players,
}

impl Score {
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            meta: Meta::new(),
            players: Players::new(),
        }
    }
}
