use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Articulation {
    Staccato,
    Tenuto,
    Staccatissimo,
    StaccatoTenuto,
    Accent,
    Marcato,
}

// Define mutually-exclusive articulation groups. Order denotes priority when
// sanitizing a set that contains multiple members from the same group.
const STACCATO_GROUP: [Articulation; 4] = [
    Articulation::Staccatissimo,
    Articulation::StaccatoTenuto,
    Articulation::Staccato,
    Articulation::Tenuto,
];

const ACCENT_GROUP: [Articulation; 2] = [Articulation::Accent, Articulation::Marcato];

impl Articulation {
    /// Return the group this articulation belongs to (if any).
    pub fn exclusivity_group(&self) -> &'static [Articulation] {
        match self {
            Articulation::Staccato
            | Articulation::Tenuto
            | Articulation::Staccatissimo
            | Articulation::StaccatoTenuto => &STACCATO_GROUP,
            Articulation::Accent | Articulation::Marcato => &ACCENT_GROUP,
        }
    }
}

/// Newtype wrapper around an FxHashSet of `Articulation` values. This provides
/// encapsulated behavior such as `toggle` that enforces group invariants.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Articulations(pub FxHashSet<Articulation>);

impl From<FxHashSet<Articulation>> for Articulations {
    fn from(set: FxHashSet<Articulation>) -> Self {
        Articulations(set)
    }
}

impl From<Articulations> for FxHashSet<Articulation> {
    fn from(a: Articulations) -> Self {
        a.0
    }
}

impl Articulations {
    pub fn new() -> Self {
        Articulations(FxHashSet::default())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, a: &Articulation) -> bool {
        self.0.contains(a)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Articulation> {
        self.0.iter()
    }

    /// Return a new `Articulations` with `articulation` toggled.
    /// - if present => removed in the returned set
    /// - if absent  => added in the returned set and any conflicting group
    ///   members are removed
    pub fn toggle(&self, articulation: Articulation) -> Articulations {
        let mut out = self.clone();
        if out.0.contains(&articulation) {
            out.0.remove(&articulation);
        } else {
            for &member in articulation.exclusivity_group().iter() {
                if member != articulation {
                    out.0.remove(&member);
                }
            }
            out.0.insert(articulation);
        }
        out
    }
}
