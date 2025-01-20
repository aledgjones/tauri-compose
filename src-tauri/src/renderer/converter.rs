pub type Px = f32;
pub type Mm = f32;
pub type Space = f32;

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

    pub fn mm_to_spaces(&self, mm: Mm) -> Space {
        mm / self.mm_per_space
    }

    pub fn spaces_to_px(&self, spaces: Space) -> Px {
        spaces * self.mm_per_space * self.px_per_mm
    }

    pub fn px_to_spaces(&self, px: Px) -> Space {
        px / self.px_per_mm / self.mm_per_space
    }
}
