use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Velocity {
    int: u8,
}

impl Velocity {
    pub fn new(int: u8) -> Self {
        Self { int }
    }

    pub fn int(&self) -> u8 {
        self.int
    }
}
