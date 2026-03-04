use serde::{Deserialize, Serialize};

pub type Px = f32;
pub type Mm = f32;
pub type Space = f32;

#[derive(Serialize, Deserialize, Clone)]
pub struct Converter {
    pub px_per_mm: f32,
    pub mm_per_space: f32,
}

impl Converter {
    pub fn new(px_per_mm: f32, mm_per_space: f32) -> Self {
        Converter {
            px_per_mm,
            mm_per_space,
        }
    }

    pub fn spaces_to_px(&self, spaces: Space) -> Px {
        spaces * self.mm_per_space * self.px_per_mm
    }

    pub fn px_to_spaces(&self, px: Px) -> Space {
        px / self.px_per_mm / self.mm_per_space
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_converter() -> Converter {
        // 96 DPI = 3.7795… px/mm, 1.75mm per space
        Converter::new(3.779_527_7, 1.75)
    }

    #[test]
    fn spaces_to_px_matches_formula() {
        let c = test_converter();
        let spaces = 5.0f32;
        let expected = spaces * 1.75 * 3.779_527_7_f32;
        assert!(
            (c.spaces_to_px(spaces) - expected).abs() < 0.001,
            "spaces_to_px({spaces}) expected {expected}"
        );
    }

    #[test]
    fn px_to_spaces_matches_formula() {
        let c = test_converter();
        let px = 100.0f32;
        let expected = px / 3.779_527_7_f32 / 1.75;
        assert!(
            (c.px_to_spaces(px) - expected).abs() < 0.001,
            "px_to_spaces({px}) expected {expected}"
        );
    }

    #[test]
    fn round_trip_spaces_to_px_and_back() {
        let c = test_converter();
        let spaces = 5.0f32;
        let result = c.px_to_spaces(c.spaces_to_px(spaces));
        assert!(
            (result - spaces).abs() < 1e-5,
            "round-trip: expected {spaces}, got {result}"
        );
    }

    #[test]
    fn zero_spaces_is_zero_px() {
        let c = test_converter();
        assert_eq!(c.spaces_to_px(0.0), 0.0);
    }

    #[test]
    fn zero_px_is_zero_spaces() {
        let c = test_converter();
        assert_eq!(c.px_to_spaces(0.0), 0.0);
    }
}
