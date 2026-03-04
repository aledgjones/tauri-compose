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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_new_stores_value() {
        let v = Velocity::new(100);
        assert_eq!(v.int(), 100);
    }

    #[test]
    fn velocity_zero() {
        let v = Velocity::new(0);
        assert_eq!(v.int(), 0);
    }

    #[test]
    fn velocity_max() {
        let v = Velocity::new(127);
        assert_eq!(v.int(), 127);
    }
}
