pub mod components;
pub mod config;
pub mod entries;
pub mod flows;
pub mod instruments;
pub mod layouts;
pub mod meta;
pub mod players;
pub mod staves;
pub mod tracks;

use config::Config;
use flows::Flows;
use instruments::Instruments;
use layouts::Layouts;
use meta::Meta;
use players::Players;
use serde::Serialize;
use tracks::Tracks;

#[derive(Serialize, Clone)]
pub struct Score {
    pub config: Config,
    pub meta: Meta,
    pub players: Players,
    pub instruments: Instruments,
    pub flows: Flows,
    pub tracks: Tracks,
    pub layouts: Layouts,
}

impl Score {
    pub fn new() -> Self {
        Self {
            layouts: Layouts::new(),
            config: Config::new(),
            meta: Meta::new(),
            players: Players::new(),
            instruments: Instruments::new(),
            flows: Flows::new(),
            tracks: Tracks::new(),
        }
    }
}
