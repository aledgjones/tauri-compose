use crate::renderer::converter::{Converter, Mm, Space};
use serde::{Deserialize, Serialize};
use skia_safe::Point as SkiaPoint;

#[derive(Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub width: Space,
    pub height: Space,
    pub right_padding: Space,
}

impl BoundingBox {
    /// Total horizontal space this element occupies (visual width + padding)
    pub fn total_width(&self) -> Space {
        self.width + self.right_padding
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PaddingSpaces {
    pub top: Space,
    pub right: Space,
    pub bottom: Space,
    pub left: Space,
}

impl PaddingSpaces {
    pub fn new(top: Space, right: Space, bottom: Space, left: Space) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PaddingMm {
    pub top: Mm,
    pub right: Mm,
    pub bottom: Mm,
    pub left: Mm,
}

impl PaddingMm {
    pub fn new(top: Mm, right: Mm, bottom: Mm, left: Mm) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn to_spaces(&self, mm_per_space: &f32) -> PaddingSpaces {
        PaddingSpaces::new(
            self.top / mm_per_space,
            self.right / mm_per_space,
            self.bottom / mm_per_space,
            self.left / mm_per_space,
        )
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: Space,
    pub y: Space,
}

impl Point {
    pub fn to_skia(self, converter: &Converter) -> SkiaPoint {
        SkiaPoint {
            x: converter.spaces_to_px(self.x),
            y: converter.spaces_to_px(self.y),
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: Space,
    pub y: Space,
    pub thickness: Space,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounding_box_total_width_sums_width_and_padding() {
        let bb = BoundingBox {
            width: 3.0,
            height: 4.0,
            right_padding: 1.5,
        };
        assert!((bb.total_width() - 4.5).abs() < f32::EPSILON);
    }

    #[test]
    fn bounding_box_total_width_zero_padding() {
        let bb = BoundingBox {
            width: 2.0,
            height: 4.0,
            right_padding: 0.0,
        };
        assert!((bb.total_width() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn padding_mm_to_spaces_divides_each_side() {
        let padding = PaddingMm::new(4.0, 2.0, 6.0, 1.0);
        let spaces = padding.to_spaces(&2.0);
        assert!((spaces.top - 2.0).abs() < f32::EPSILON);
        assert!((spaces.right - 1.0).abs() < f32::EPSILON);
        assert!((spaces.bottom - 3.0).abs() < f32::EPSILON);
        assert!((spaces.left - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn padding_mm_new_stores_fields() {
        let p = PaddingMm::new(1.0, 2.0, 3.0, 4.0);
        assert!((p.top - 1.0).abs() < f32::EPSILON);
        assert!((p.right - 2.0).abs() < f32::EPSILON);
        assert!((p.bottom - 3.0).abs() < f32::EPSILON);
        assert!((p.left - 4.0).abs() < f32::EPSILON);
    }
}
