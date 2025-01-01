pub mod meta;

use meta::Meta;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Score {
    meta: Meta,
}

impl Score {
    pub fn new() -> Self {
        Self { meta: Meta::new() }
    }
}
