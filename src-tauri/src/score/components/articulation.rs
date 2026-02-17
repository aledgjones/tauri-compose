use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Articulation {
    Staccato,
    Tenuto,
}
