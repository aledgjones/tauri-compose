use serde::{Deserialize, Serialize};

use super::units::{Converter, Mm, Space};

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

    pub fn to_spaces(&self, converter: &Converter) -> PaddingSpaces {
        PaddingSpaces::new(
            converter.mm_to_spaces(self.top),
            converter.mm_to_spaces(self.right),
            converter.mm_to_spaces(self.bottom),
            converter.mm_to_spaces(self.left),
        )
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: Space,
    pub y: Space,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: Space,
    pub y: Space,
    pub thickness: Space,
}
