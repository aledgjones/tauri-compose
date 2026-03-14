use super::Entries;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::utils::UniqueId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Tuplet {
    pub id: UniqueId,
    pub tick: Tick,
    pub note_count: u8,
    pub in_space_of: u8,
    pub base_note_value: NoteDuration,
    pub groupings: Vec<u8>,
    pub entries: Entries,
}

#[allow(dead_code)]
impl Tuplet {
    pub fn consumed_ticks(&self) -> Ticks {
        u64::from(self.in_space_of) * self.base_note_value.to_ticks()
    }

    pub fn local_length_ticks(&self) -> Ticks {
        u64::from(self.note_count) * self.base_note_value.to_ticks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triplet_ratio_derives_expected_tick_spans() {
        let tuplet = Tuplet {
            id: 1,
            tick: 64,
            note_count: 3,
            in_space_of: 2,
            base_note_value: NoteDuration::Eighth,
            groupings: vec![3],
            entries: Entries::new(),
        };

        assert_eq!(tuplet.local_length_ticks(), 48);
        assert_eq!(tuplet.consumed_ticks(), 32);
    }
}
