use crate::renderer::converter::{Converter, Mm, Space};
use serde::{Deserialize, Serialize};
use skia_safe::Point as SkiaPoint;

#[derive(Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub width: Space,
    pub height: Space,
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
