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
    Stressed,
    Unstressed,
}

// Define mutually-exclusive articulation groups.
const STACCATO_GROUP: [Articulation; 4] = [
    Articulation::Staccatissimo,
    Articulation::StaccatoTenuto,
    Articulation::Staccato,
    Articulation::Tenuto,
];
const ACCENT_GROUP: [Articulation; 2] = [Articulation::Accent, Articulation::Marcato];
const STRESS_GROUP: [Articulation; 2] = [Articulation::Stressed, Articulation::Unstressed];

impl Articulation {
    /// Return the group this articulation belongs to (if any).
    pub fn exclusivity_group(&self) -> &'static [Articulation] {
        match self {
            Articulation::Staccato => &STACCATO_GROUP,
            Articulation::Tenuto => &STACCATO_GROUP,
            Articulation::Staccatissimo => &STACCATO_GROUP,
            Articulation::StaccatoTenuto => &STACCATO_GROUP,

            Articulation::Accent => &ACCENT_GROUP,
            Articulation::Marcato => &ACCENT_GROUP,

            Articulation::Stressed => &STRESS_GROUP,
            Articulation::Unstressed => &STRESS_GROUP,
        }
    }
}

/// Wrapper type around an FxHashSet of `Articulation` values. This provides
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_adds_when_absent() {
        let arts = Articulations::new();
        let result = arts.toggle(Articulation::Staccato);
        assert!(result.contains(&Articulation::Staccato));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn toggle_removes_when_present() {
        let arts = Articulations::new().toggle(Articulation::Staccato);
        let result = arts.toggle(Articulation::Staccato);
        assert!(!result.contains(&Articulation::Staccato));
        assert!(result.is_empty());
    }

    #[test]
    fn toggle_replaces_conflicting_staccato_group_member() {
        let arts = Articulations::new().toggle(Articulation::Staccato);
        let result = arts.toggle(Articulation::Tenuto);
        assert!(!result.contains(&Articulation::Staccato));
        assert!(result.contains(&Articulation::Tenuto));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn toggle_replaces_conflicting_accent_group_member() {
        let arts = Articulations::new().toggle(Articulation::Accent);
        let result = arts.toggle(Articulation::Marcato);
        assert!(!result.contains(&Articulation::Accent));
        assert!(result.contains(&Articulation::Marcato));
    }

    #[test]
    fn toggle_preserves_non_conflicting_groups() {
        let arts = Articulations::new()
            .toggle(Articulation::Staccato)
            .toggle(Articulation::Accent);
        let result = arts.toggle(Articulation::Stressed);
        assert!(result.contains(&Articulation::Staccato));
        assert!(result.contains(&Articulation::Accent));
        assert!(result.contains(&Articulation::Stressed));
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn toggle_stress_group_is_exclusive() {
        let arts = Articulations::new().toggle(Articulation::Stressed);
        let result = arts.toggle(Articulation::Unstressed);
        assert!(!result.contains(&Articulation::Stressed));
        assert!(result.contains(&Articulation::Unstressed));
    }

    #[test]
    fn is_empty_true_when_new() {
        assert!(Articulations::new().is_empty());
    }

    #[test]
    fn is_empty_false_after_toggle() {
        assert!(!Articulations::new().toggle(Articulation::Staccato).is_empty());
    }

    #[test]
    fn iter_yields_all_articulations() {
        let arts = Articulations::new().toggle(Articulation::Staccato).toggle(Articulation::Accent);
        let collected: Vec<_> = arts.iter().collect();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    fn new_creates_empty_articulations() {
        let arts = Articulations::new();
        assert_eq!(arts.len(), 0);
    }
}
