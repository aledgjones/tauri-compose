use serde_repr::{Deserialize_repr, Serialize_repr};

pub type Tick = u64;
pub type Ticks = u64;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn to_modifier(self) -> i8 {
        match self {
            Direction::Up => -1,
            Direction::Down => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NoteDuration {
    Whole = 1,
    Half = 2,
    Quarter = 4,
    Eighth = 8,
    Sixteenth = 16,
    ThirtySecond = 32,
    SixtyFourth = 64,
    HudredTwentyEighth = 128,
}

impl NoteDuration {
    /// convert ticks to NoteDuration. this may fail so wrap in option
    pub fn from_ticks(ticks: Ticks) -> Option<NoteDuration> {
        match ticks {
            128 => Some(NoteDuration::Whole),
            64 => Some(NoteDuration::Half),
            32 => Some(NoteDuration::Quarter),
            16 => Some(NoteDuration::Eighth),
            8 => Some(NoteDuration::Sixteenth),
            4 => Some(NoteDuration::ThirtySecond),
            2 => Some(NoteDuration::SixtyFourth),
            1 => Some(NoteDuration::HudredTwentyEighth),
            _ => None,
        }
    }

    pub fn to_ticks(&self) -> Ticks {
        match self {
            NoteDuration::Whole => 128,
            NoteDuration::Half => 64,
            NoteDuration::Quarter => 32,
            NoteDuration::Eighth => 16,
            NoteDuration::Sixteenth => 8,
            NoteDuration::ThirtySecond => 4,
            NoteDuration::SixtyFourth => 2,
            NoteDuration::HudredTwentyEighth => 1,
        }
    }

    pub fn half(&self) -> NoteDuration {
        match self {
            NoteDuration::Whole => NoteDuration::Half,
            NoteDuration::Half => NoteDuration::Quarter,
            NoteDuration::Quarter => NoteDuration::Eighth,
            NoteDuration::Eighth => NoteDuration::Sixteenth,
            NoteDuration::Sixteenth => NoteDuration::ThirtySecond,
            NoteDuration::ThirtySecond => NoteDuration::SixtyFourth,
            NoteDuration::SixtyFourth => NoteDuration::HudredTwentyEighth,
            NoteDuration::HudredTwentyEighth => panic!("Cannot halve HudredTwentyEighth duration"),
        }
    }

    pub fn spacing_ratio(&self, ratio: f32, is_dotted: bool) -> f32 {
        let base = match self {
            NoteDuration::Whole => ratio * 2.0,
            NoteDuration::Half => ratio,
            NoteDuration::Quarter => 1.0,
            NoteDuration::Eighth => 1.0 / ratio,
            NoteDuration::Sixteenth => 1.0 / (ratio * 2.0),
            NoteDuration::ThirtySecond => 1.0 / (ratio * 4.0),
            NoteDuration::SixtyFourth => 1.0 / (ratio * 8.0),
            NoteDuration::HudredTwentyEighth => 1.0 / (ratio * 16.0),
        };

        if is_dotted {
            base * 1.2
        } else {
            base
        }
    }
}

pub const NOTE_DURATIONS: [NoteDuration; 8] = [
    NoteDuration::Whole,
    NoteDuration::Half,
    NoteDuration::Quarter,
    NoteDuration::Eighth,
    NoteDuration::Sixteenth,
    NoteDuration::ThirtySecond,
    NoteDuration::SixtyFourth,
    NoteDuration::HudredTwentyEighth,
];

#[cfg(test)]
mod tests {
    use super::*;

    // --- from_ticks ---

    #[test]
    fn from_ticks_whole() {
        assert!(matches!(
            NoteDuration::from_ticks(128),
            Some(NoteDuration::Whole)
        ));
    }

    #[test]
    fn from_ticks_half() {
        assert!(matches!(
            NoteDuration::from_ticks(64),
            Some(NoteDuration::Half)
        ));
    }

    #[test]
    fn from_ticks_quarter() {
        assert!(matches!(
            NoteDuration::from_ticks(32),
            Some(NoteDuration::Quarter)
        ));
    }

    #[test]
    fn from_ticks_eighth() {
        assert!(matches!(
            NoteDuration::from_ticks(16),
            Some(NoteDuration::Eighth)
        ));
    }

    #[test]
    fn from_ticks_sixteenth() {
        assert!(matches!(
            NoteDuration::from_ticks(8),
            Some(NoteDuration::Sixteenth)
        ));
    }

    #[test]
    fn from_ticks_thirty_second() {
        assert!(matches!(
            NoteDuration::from_ticks(4),
            Some(NoteDuration::ThirtySecond)
        ));
    }

    #[test]
    fn from_ticks_sixty_fourth() {
        assert!(matches!(
            NoteDuration::from_ticks(2),
            Some(NoteDuration::SixtyFourth)
        ));
    }

    #[test]
    fn from_ticks_hudred_twenty_eighth() {
        assert!(matches!(
            NoteDuration::from_ticks(1),
            Some(NoteDuration::HudredTwentyEighth)
        ));
    }

    #[test]
    fn from_ticks_invalid_returns_none() {
        assert!(NoteDuration::from_ticks(99).is_none());
        assert!(NoteDuration::from_ticks(0).is_none());
        assert!(NoteDuration::from_ticks(3).is_none());
    }

    // --- to_ticks ---

    #[test]
    fn to_ticks_all_variants() {
        assert_eq!(NoteDuration::Whole.to_ticks(), 128);
        assert_eq!(NoteDuration::Half.to_ticks(), 64);
        assert_eq!(NoteDuration::Quarter.to_ticks(), 32);
        assert_eq!(NoteDuration::Eighth.to_ticks(), 16);
        assert_eq!(NoteDuration::Sixteenth.to_ticks(), 8);
        assert_eq!(NoteDuration::ThirtySecond.to_ticks(), 4);
        assert_eq!(NoteDuration::SixtyFourth.to_ticks(), 2);
        assert_eq!(NoteDuration::HudredTwentyEighth.to_ticks(), 1);
    }

    // --- round-trip ---

    #[test]
    fn round_trip_from_ticks_to_ticks() {
        for dur in &NOTE_DURATIONS {
            let ticks = dur.to_ticks();
            let result = NoteDuration::from_ticks(ticks);
            assert!(result.is_some(), "from_ticks({ticks}) returned None");
            assert_eq!(result.unwrap().to_ticks(), ticks);
        }
    }

    // --- half ---

    #[test]
    fn half_whole_to_half() {
        assert!(matches!(NoteDuration::Whole.half(), NoteDuration::Half));
    }

    #[test]
    fn half_half_to_quarter() {
        assert!(matches!(NoteDuration::Half.half(), NoteDuration::Quarter));
    }

    #[test]
    fn half_quarter_to_eighth() {
        assert!(matches!(NoteDuration::Quarter.half(), NoteDuration::Eighth));
    }

    #[test]
    fn half_eighth_to_sixteenth() {
        assert!(matches!(
            NoteDuration::Eighth.half(),
            NoteDuration::Sixteenth
        ));
    }

    #[test]
    fn half_sixteenth_to_thirty_second() {
        assert!(matches!(
            NoteDuration::Sixteenth.half(),
            NoteDuration::ThirtySecond
        ));
    }

    #[test]
    fn half_thirty_second_to_sixty_fourth() {
        assert!(matches!(
            NoteDuration::ThirtySecond.half(),
            NoteDuration::SixtyFourth
        ));
    }

    #[test]
    fn half_sixty_fourth_to_hudred_twenty_eighth() {
        assert!(matches!(
            NoteDuration::SixtyFourth.half(),
            NoteDuration::HudredTwentyEighth
        ));
    }

    #[test]
    #[should_panic]
    fn half_hudred_twenty_eighth_panics() {
        NoteDuration::HudredTwentyEighth.half();
    }

    // --- Direction::to_modifier ---

    #[test]
    fn direction_up_is_minus_one() {
        assert_eq!(Direction::Up.to_modifier(), -1);
    }

    #[test]
    fn direction_down_is_plus_one() {
        assert_eq!(Direction::Down.to_modifier(), 1);
    }

    // --- spacing_ratio ---

    #[test]
    fn spacing_ratio_quarter_undotted_is_one() {
        let ratio = 1.5f32;
        assert!(
            (NoteDuration::Quarter.spacing_ratio(ratio, false) - 1.0).abs() < f32::EPSILON,
            "Quarter undotted spacing ratio should be 1.0"
        );
    }

    #[test]
    fn spacing_ratio_whole_is_ratio_times_two() {
        let ratio = 1.5f32;
        let expected = ratio * 2.0;
        assert!((NoteDuration::Whole.spacing_ratio(ratio, false) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn spacing_ratio_dotted_is_1_2x_undotted() {
        let ratio = 1.5f32;
        let undotted = NoteDuration::Quarter.spacing_ratio(ratio, false);
        let dotted = NoteDuration::Quarter.spacing_ratio(ratio, true);
        assert!(
            (dotted / undotted - 1.2).abs() < 1e-6,
            "dotted should be exactly 1.2x undotted"
        );
    }

    #[test]
    fn spacing_ratio_eighth_is_1_over_ratio() {
        let ratio = 1.5f32;
        let expected = 1.0 / ratio;
        assert!((NoteDuration::Eighth.spacing_ratio(ratio, false) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn spacing_ratio_sixteenth_is_1_over_ratio_times_2() {
        let ratio = 1.5f32;
        let expected = 1.0 / (ratio * 2.0);
        assert!(
            (NoteDuration::Sixteenth.spacing_ratio(ratio, false) - expected).abs() < f32::EPSILON
        );
    }

    #[test]
    fn spacing_ratio_thirty_second_is_1_over_ratio_times_4() {
        let ratio = 1.5f32;
        let expected = 1.0 / (ratio * 4.0);
        assert!(
            (NoteDuration::ThirtySecond.spacing_ratio(ratio, false) - expected).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn spacing_ratio_sixty_fourth_is_1_over_ratio_times_8() {
        let ratio = 1.5f32;
        let expected = 1.0 / (ratio * 8.0);
        assert!(
            (NoteDuration::SixtyFourth.spacing_ratio(ratio, false) - expected).abs() < f32::EPSILON
        );
    }

    #[test]
    fn spacing_ratio_hundred_twenty_eighth_is_1_over_ratio_times_16() {
        let ratio = 1.5f32;
        let expected = 1.0 / (ratio * 16.0);
        assert!(
            (NoteDuration::HudredTwentyEighth.spacing_ratio(ratio, false) - expected).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn spacing_ratio_half_is_ratio() {
        let ratio = 1.5f32;
        assert!((NoteDuration::Half.spacing_ratio(ratio, false) - ratio).abs() < f32::EPSILON);
    }
}
