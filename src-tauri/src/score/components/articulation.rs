use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Articulation {
    None,
    Staccato,
    Tenuto,
    Accent,
    Marcato,
}
