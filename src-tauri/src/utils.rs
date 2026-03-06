use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub type UniqueId = u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_returns_positive_timestamp() {
        assert!(now() > 0);
    }

    #[test]
    fn now_returns_recent_timestamp() {
        // Should be way past year 2000 (timestamp ~946684800000 ms)
        assert!(now() > 946_684_800_000);
    }

    #[test]
    fn now_monotonically_increases() {
        let t1 = now();
        let t2 = now();
        assert!(t2 >= t1);
    }
}
