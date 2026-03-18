use super::get_written_durations::{NotationByTrack, NotationTrack};
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::Tick;
use crate::utils::UniqueId;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

pub type ArticulationSet = FxHashSet<Articulation>;
pub type Articulations = FxHashMap<Tick, ArticulationSet>;
pub type ArticulationsByTrack = FxHashMap<UniqueId, Articulations>;

pub(crate) fn get_articulations_in_track(notation: &NotationTrack) -> Articulations {
    let mut output: Articulations = FxHashMap::default();

    for (tick, entry) in &notation.track {
        if entry.is_rest() {
            continue;
        }

        let mut tick_set: ArticulationSet = FxHashSet::default();
        for tone in &entry.tones {
            for a in tone.articulations.iter() {
                tick_set.insert(*a);
            }
        }

        if !tick_set.is_empty() {
            output.insert(*tick, tick_set);
        }
    }

    output
}

pub fn get_articulations(notation_by_track: &NotationByTrack) -> ArticulationsByTrack {
    let results: Vec<(UniqueId, _)> = notation_by_track
        .par_iter()
        .map(|(track_id, notation)| {
            let articulations: Articulations = get_articulations_in_track(notation);
            (*track_id, articulations)
        })
        .collect();

    let mut output: ArticulationsByTrack = FxHashMap::default();
    for (track_id, articulations) in results {
        output.insert(track_id, articulations);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_written_durations::{Notation, NotationTrack};
    use crate::score::components::articulation::{Articulation, Articulations};
    use crate::score::components::duration::NoteDuration;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::entries::tone::Tone;

    fn make_tone_with_articulation(id: u64, art: Articulation) -> Tone {
        use crate::score::components::velocity::Velocity;
        Tone::new(
            id,
            0,
            NoteDuration::Quarter.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new().toggle(art),
        )
    }

    fn make_tone_no_articulation(id: u64) -> Tone {
        use crate::score::components::velocity::Velocity;
        Tone::new(
            id,
            0,
            NoteDuration::Quarter.to_ticks(),
            Pitch::new(60, Accidental::Natural),
            Velocity::new(100),
            Articulations::new(),
        )
    }

    #[test]
    fn empty_notation_track_produces_empty_articulations() {
        let track = NotationTrack::new(128);
        let result = get_articulations_in_track(&track);
        assert!(result.is_empty());
    }

    #[test]
    fn rest_entry_produces_no_articulations() {
        let mut track = NotationTrack::new(128);
        // Notation with no tones = rest
        let entry = Notation::new(0, NoteDuration::Quarter.to_ticks());
        track.track.insert(0, entry);
        let result = get_articulations_in_track(&track);
        assert!(result.is_empty());
    }

    #[test]
    fn tone_with_articulation_is_recorded() {
        let mut track = NotationTrack::new(128);
        let mut entry = Notation::new(0, NoteDuration::Quarter.to_ticks());
        entry
            .tones
            .push(make_tone_with_articulation(1, Articulation::Staccato));
        track.track.insert(0, entry);
        let result = get_articulations_in_track(&track);
        assert!(result.contains_key(&0));
        assert!(result[&0].contains(&Articulation::Staccato));
    }

    #[test]
    fn tone_without_articulation_not_recorded() {
        let mut track = NotationTrack::new(128);
        let mut entry = Notation::new(0, NoteDuration::Quarter.to_ticks());
        entry.tones.push(make_tone_no_articulation(1));
        track.track.insert(0, entry);
        let result = get_articulations_in_track(&track);
        assert!(result.is_empty());
    }

    #[test]
    fn multiple_tones_same_tick_merge_articulations() {
        let mut track = NotationTrack::new(128);
        let mut entry = Notation::new(0, NoteDuration::Quarter.to_ticks());
        entry
            .tones
            .push(make_tone_with_articulation(1, Articulation::Staccato));
        entry
            .tones
            .push(make_tone_with_articulation(2, Articulation::Accent));
        track.track.insert(0, entry);
        let result = get_articulations_in_track(&track);
        assert_eq!(result.len(), 1);
        let set = &result[&0];
        assert!(set.contains(&Articulation::Staccato));
        assert!(set.contains(&Articulation::Accent));
    }

    #[test]
    fn articulations_at_multiple_ticks() {
        let mut track = NotationTrack::new(256);
        let mut e1 = Notation::new(0, NoteDuration::Quarter.to_ticks());
        e1.tones
            .push(make_tone_with_articulation(1, Articulation::Tenuto));
        let mut e2 = Notation::new(128, NoteDuration::Quarter.to_ticks());
        e2.tones
            .push(make_tone_with_articulation(2, Articulation::Marcato));
        track.track.insert(0, e1);
        track.track.insert(128, e2);
        let result = get_articulations_in_track(&track);
        assert_eq!(result.len(), 2);
        assert!(result[&0].contains(&Articulation::Tenuto));
        assert!(result[&128].contains(&Articulation::Marcato));
    }

    #[test]
    fn split_note_fragments_keep_articulations_at_each_written_tick() {
        let mut track = NotationTrack::new(96);
        let mut first = Notation::new(0, 32);
        first
            .tones
            .push(make_tone_with_articulation(1, Articulation::Staccato));
        let mut second = Notation::new(32, 16);
        second
            .tones
            .push(make_tone_with_articulation(1, Articulation::Staccato));

        track.track.insert(0, first);
        track.track.insert(32, second);

        let result = get_articulations_in_track(&track);

        assert!(result[&0].contains(&Articulation::Staccato));
        assert!(result[&32].contains(&Articulation::Staccato));
    }
}
