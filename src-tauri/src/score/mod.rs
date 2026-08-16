pub mod components;
pub mod config;
pub mod engraves;
pub mod entries;
pub mod flows;
pub mod instruments;
pub mod layouts;
pub mod meta;
pub mod playback;
pub mod players;
pub mod staves;
pub mod time_range;
pub mod tracks;

use config::Config;
use engraves::Engraves;
use flows::Flows;
use instruments::Instruments;
use layouts::Layouts;
use meta::Meta;
use playback::ScorePlayback;
use players::Players;
use serde::Deserialize;
use serde::Serialize;
use tracks::Tracks;

#[derive(Serialize, Deserialize, Clone)]
pub struct Score {
    pub config: Config,
    pub meta: Meta,
    pub playback: ScorePlayback,
    pub players: Players,
    pub instruments: Instruments,
    pub flows: Flows,
    pub tracks: Tracks,
    pub layouts: Layouts,
    pub engraves: Engraves,
}

impl Score {
    pub fn new() -> Self {
        Self {
            layouts: Layouts::new(),
            config: Config::new(),
            meta: Meta::new(),
            playback: ScorePlayback::new(),
            players: Players::new(),
            instruments: Instruments::new(),
            flows: Flows::new(),
            tracks: Tracks::new(),
            engraves: Engraves::new(),
        }
    }
}
