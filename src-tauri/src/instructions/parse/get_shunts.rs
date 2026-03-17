use crate::score::components::duration::{Direction, Tick};
use crate::score::entries::tone::Tone;
use crate::utils::UniqueId;

use super::get_stem_directions::StemDirectionsByTrack;
use super::get_tone_offsets::ToneVerticalOffsets;
use super::get_written_durations::{Notation, NotationByTrack};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub type Clusters = Vec<Cluster>;
pub type Cluster = Vec<Tone>;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Shunt {
    Pre,
    None,
    Post,
}

pub struct Shunts {
    pub by_id: FxHashMap<(Tick, UniqueId), Shunt>,
    pub by_offset: FxHashMap<(Tick, i8), Shunt>,
}

pub type ShuntsByTrack = FxHashMap<UniqueId, Shunts>;

pub fn note_shunts_in_chord(
    tick: &Tick,
    entry: &Notation,
    tone_offsets: &ToneVerticalOffsets,
    stem_direction: &Direction,
    shunts: &mut Shunts,
) {
    let clusters = entry.get_clusters(tone_offsets);

    for cluster in clusters {
        let is_odd_length = cluster.len() % 2 != 0;
        let first_note_shunted = stem_direction == &Direction::Up || is_odd_length;

        for (i, tone) in cluster.iter().enumerate() {
            // alternate between shunted/not shunted based on the position
            // of the first tone in the cluster
            let shunted = match first_note_shunted {
                true => i % 2 > 0,
                false => i % 2 == 0,
            };

            let shunt = match shunted {
                true => match stem_direction {
                    Direction::Up => Shunt::Post,
                    Direction::Down => Shunt::Pre,
                },
                false => Shunt::None,
            };

            let offset = tone_offsets[&tone.id];

            shunts.by_id.insert((*tick, tone.id), shunt.clone());
            shunts.by_offset.insert((*tick, offset), shunt.clone());
        }
    }
}

pub fn get_note_shunts(
    notation_by_track: &NotationByTrack,
    tone_offsets: &ToneVerticalOffsets,
    stem_directions_by_track: &StemDirectionsByTrack,
) -> ShuntsByTrack {
    let results: Vec<(UniqueId, _)> = notation_by_track
        .par_iter()
        .map(|(track_id, notation)| {
            let mut shunts = Shunts {
                by_id: FxHashMap::default(),
                by_offset: FxHashMap::default(),
            };
            let stem_directions = &stem_directions_by_track[track_id];
            for (tick, entry) in &notation.track {
                if !entry.is_rest() {
                    let stem_direction = &stem_directions[tick];
                    note_shunts_in_chord(tick, entry, tone_offsets, stem_direction, &mut shunts);
                }
            }
            (*track_id, shunts)
        })
        .collect();

    let mut output: ShuntsByTrack = FxHashMap::default();
    for (track_id, shunts) in results {
        output.insert(track_id, shunts);
    }

    output
}

impl Notation {
    pub fn get_clusters(&self, tone_offsets: &ToneVerticalOffsets) -> Clusters {
        let mut clusters: Clusters = Vec::new();
        let mut cluster: Cluster = Vec::new();
        let mut previous_tone = self.tones.first().unwrap();

        for current_tone in &self.tones[1..] {
            let current_offset = tone_offsets[&current_tone.id];
            let previous_offset = tone_offsets[&previous_tone.id];

            cluster.push(previous_tone.clone());

            // if not a cluster
            if previous_offset - current_offset > 1 {
                clusters.push(cluster);
                cluster = Vec::new();
            }

            previous_tone = current_tone;
        }

        if !self.tones.is_empty() {
            cluster.push(previous_tone.clone());
            clusters.push(cluster);
        }

        clusters
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_written_durations::Notation;
    use crate::score::components::{
        articulation::Articulations,
        pitch::{Accidental, Pitch},
        velocity::Velocity,
    };
    use crate::score::entries::tone::Tone;
    use crate::utils::UniqueId;
    use rustc_hash::FxHashMap;

    fn make_tone(id: UniqueId, pitch_int: u8) -> Tone {
        Tone::new(
            id,
            0,
            32,
            Pitch::new(pitch_int, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        )
    }

    #[test]
    fn shunt_ordering_pre_less_than_none() {
        assert!(Shunt::Pre < Shunt::None);
    }

    #[test]
    fn shunt_ordering_none_less_than_post() {
        assert!(Shunt::None < Shunt::Post);
    }

    #[test]
    fn get_clusters_single_tone_returns_one_cluster() {
        let mut n = Notation::new(0, 32);
        n.tones.push(make_tone(1, 60));
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 0);
        let clusters = n.get_clusters(&offsets);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].len(), 1);
    }

    #[test]
    fn get_clusters_non_adjacent_tones_return_two_clusters() {
        let mut n = Notation::new(0, 32);
        n.tones.push(make_tone(1, 60)); // offset 0
        n.tones.push(make_tone(2, 55)); // offset -2
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 0);
        offsets.insert(2, -2); // gap of 2 > 1
        let clusters = n.get_clusters(&offsets);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn get_clusters_adjacent_tones_return_one_cluster() {
        let mut n = Notation::new(0, 32);
        n.tones.push(make_tone(1, 60)); // offset 0
        n.tones.push(make_tone(2, 59)); // offset -1
        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 0);
        offsets.insert(2, -1); // gap of 1, not > 1
        let clusters = n.get_clusters(&offsets);
        assert_eq!(clusters.len(), 1);
    }

    // --- note_shunts_in_chord ---

    #[test]
    fn note_shunts_single_tone_gets_none_shunt_when_stem_down() {
        use super::*;
        use crate::score::components::duration::Direction;

        let tone = make_tone(1, 60);
        let mut n = Notation::new(0, 32);
        n.tones.push(tone.clone());

        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 0);

        let mut shunts = Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        };

        note_shunts_in_chord(&0, &n, &offsets, &Direction::Down, &mut shunts);

        // odd-length cluster (1) => first_note_shunted=true; i=0 shunted=false => None
        assert!(shunts.by_id[&(0, 1)] == Shunt::None);
    }

    #[test]
    fn note_shunts_two_adjacent_tones_stem_up_gets_post_and_none() {
        use super::*;
        use crate::score::components::duration::Direction;

        let tone1 = make_tone(1, 60); // offset 0
        let tone2 = make_tone(2, 59); // offset -1 (adjacent)
        let mut n = Notation::new(0, 32);
        n.tones.push(tone1.clone());
        n.tones.push(tone2.clone());

        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 0);
        offsets.insert(2, -1);

        let mut shunts = Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        };

        // adjacent 2-tone cluster; stem Up; even length => first_note_shunted=true from stem rule
        note_shunts_in_chord(&0, &n, &offsets, &Direction::Up, &mut shunts);

        // i=0: shunted = first_note_shunted && i%2==0 => shunted (Post for Up)
        // i=1: shunted = !( i%2 > 0 ) = false => None
        let s1 = &shunts.by_id[&(0, 1)];
        let s2 = &shunts.by_id[&(0, 2)];
        // One should be Post and one None, but order is determined by stem
        assert!(*s1 == Shunt::Post || *s1 == Shunt::None);
        assert!(*s2 == Shunt::Post || *s2 == Shunt::None);
        assert!(*s1 != *s2);
    }

    #[test]
    fn note_shunts_two_adjacent_tones_stem_down_gets_pre_and_none() {
        use super::*;
        use crate::score::components::duration::Direction;

        let tone1 = make_tone(1, 60);
        let tone2 = make_tone(2, 59);
        let mut n = Notation::new(0, 32);
        n.tones.push(tone1.clone());
        n.tones.push(tone2.clone());

        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 0);
        offsets.insert(2, -1);

        let mut shunts = Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        };

        note_shunts_in_chord(&0, &n, &offsets, &Direction::Down, &mut shunts);

        let s1 = &shunts.by_id[&(0, 1)];
        let s2 = &shunts.by_id[&(0, 2)];
        assert!(*s1 == Shunt::Pre || *s1 == Shunt::None);
        assert!(*s2 == Shunt::Pre || *s2 == Shunt::None);
        assert!(*s1 != *s2);
    }

    #[test]
    fn note_shunts_three_tones_odd_cluster_stem_up() {
        use super::*;
        use crate::score::components::duration::Direction;

        // 3 non-adjacent tones → three clusters of 1 each
        let tone1 = make_tone(1, 60);
        let tone2 = make_tone(2, 55);
        let tone3 = make_tone(3, 50);
        let mut n = Notation::new(0, 32);
        n.tones.push(tone1.clone());
        n.tones.push(tone2.clone());
        n.tones.push(tone3.clone());

        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 4);
        offsets.insert(2, 0);
        offsets.insert(3, -4);

        let mut shunts = Shunts {
            by_id: FxHashMap::default(),
            by_offset: FxHashMap::default(),
        };

        note_shunts_in_chord(&0, &n, &offsets, &Direction::Up, &mut shunts);
        // 3 entries should be in shunts
        assert_eq!(shunts.by_id.len(), 3);
    }

    #[test]
    fn get_note_shunts_assigns_shunts_for_each_split_chord_fragment() {
        use crate::score::components::duration::Direction;

        let mut entry_one = Notation::new(0, 32);
        entry_one.tones.push(make_tone(1, 60));
        entry_one.tones.push(make_tone(2, 59));

        let mut entry_two = Notation::new(32, 16);
        entry_two.tones.push(make_tone(1, 60));
        entry_two.tones.push(make_tone(2, 59));

        let mut notation = NotationByTrack::default();
        let mut notation_track = FxHashMap::default();
        notation_track.insert(0_u64, entry_one);
        notation_track.insert(32_u64, entry_two);
        notation.insert(
            99,
            crate::instructions::parse::get_written_durations::NotationTrack {
                length: 96,
                track: notation_track,
                anchored_tracks: Vec::new(),
            },
        );

        let mut offsets: ToneVerticalOffsets = FxHashMap::default();
        offsets.insert(1, 0);
        offsets.insert(2, -1);

        let mut stem_directions = StemDirectionsByTrack::default();
        let mut by_tick = FxHashMap::default();
        by_tick.insert(0_u64, Direction::Up);
        by_tick.insert(32_u64, Direction::Up);
        stem_directions.insert(99, by_tick);

        let result = get_note_shunts(&notation, &offsets, &stem_directions);
        let shunts = &result[&99];

        assert!(shunts.by_id.contains_key(&(0, 1)));
        assert!(shunts.by_id.contains_key(&(0, 2)));
        assert!(shunts.by_id.contains_key(&(32, 1)));
        assert!(shunts.by_id.contains_key(&(32, 2)));
    }
}
