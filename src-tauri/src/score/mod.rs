pub mod config;
pub mod meta;

use config::Config;
use meta::Meta;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Score {
    config: Config,
    meta: Meta,
}

impl Score {
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            meta: Meta::new(),
        }
    }
}
