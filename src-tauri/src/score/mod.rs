pub mod components;
pub mod config;
pub mod entries;
pub mod flows;
pub mod instruments;
pub mod meta;
pub mod players;
pub mod staves;
pub mod tracks;

use config::Config;
use flows::Flows;
use instruments::Instruments;
use meta::Meta;
use players::Players;
use serde::Serialize;
use tracks::Tracks;

#[derive(Serialize, Clone)]
pub struct Score {
    config: Config,
    meta: Meta,
    players: Players,
    instruments: Instruments,
    flows: Flows,
    tracks: Tracks,
}

impl Score {
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            meta: Meta::new(),
            players: Players::new(),
            instruments: Instruments::new(),
            flows: Flows::new(),
            tracks: Tracks::new(),
        }
    }
}
