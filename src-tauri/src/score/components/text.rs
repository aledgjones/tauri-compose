use crate::renderer::converter::Space;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Justify {
    Start,
    Middle,
    End,
}

#[derive(Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Align {
    Top,
    Middle,
    Bottom,
}

#[derive(Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum FontFamily {
    Bravura,
    LibreBaskerville,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Font {
    pub size: Space,
    pub family: FontFamily,
    pub justify: Justify,
    pub align: Align,
}
