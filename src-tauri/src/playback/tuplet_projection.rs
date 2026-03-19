use crate::score::components::duration::{Tick, Ticks};
use crate::score::entries::tone::Tone;
use crate::score::entries::tuplet::{Tuplet, TupletEntry};
use crate::score::entries::Entry;
use crate::score::tracks::Track;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackTick {
    pub numerator: u64,
    pub denominator: u64,
}

impl PlaybackTick {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        assert!(denominator > 0, "PlaybackTick denominator must be non-zero");

        let divisor = gcd(numerator, denominator);

        Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        }
    }

    pub fn from_tick(tick: u64) -> Self {
        Self::new(tick, 1)
    }

    pub fn floor(&self) -> Tick {
        self.numerator / self.denominator
    }

    pub fn to_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    pub fn fractional_within_tick(&self) -> Self {
        Self::new(self.numerator % self.denominator, self.denominator)
    }

    fn add(&self, other: &Self) -> Self {
        let left = self.numerator as u128 * other.denominator as u128;
        let right = other.numerator as u128 * self.denominator as u128;
        let denominator = self.denominator as u128 * other.denominator as u128;

        Self::new((left + right) as u64, denominator as u64)
    }

    fn sub(&self, other: &Self) -> Self {
        let left = self.numerator as u128 * other.denominator as u128;
        let right = other.numerator as u128 * self.denominator as u128;
        let denominator = self.denominator as u128 * other.denominator as u128;

        assert!(left >= right, "PlaybackTick subtraction underflow");

        Self::new((left - right) as u64, denominator as u64)
    }
}

impl PartialOrd for PlaybackTick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlaybackTick {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.numerator as u128 * other.denominator as u128;
        let right = other.numerator as u128 * self.denominator as u128;
        left.cmp(&right)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackToneEvent {
    pub start: PlaybackTick,
    pub duration: PlaybackTick,
    pub pitch: u8,
    pub velocity: u8,
}

#[derive(Clone, Copy)]
struct TupletBoundary {
    host_start: Tick,
    host_end: Tick,
    expanded_start: Tick,
    expanded_end: Tick,
    local_length: Ticks,
    consumed_ticks: Ticks,
}

impl TupletBoundary {
    fn project_expanded_offset_to_host(&self, expanded_offset: Tick) -> PlaybackTick {
        let local_offset = expanded_offset - self.expanded_start;
        PlaybackTick::from_tick(self.host_start).add(&PlaybackTick::new(
            local_offset * self.consumed_ticks,
            self.local_length,
        ))
    }
}

pub fn collect_track_playback_events(track: &Track) -> Vec<PlaybackToneEvent> {
    let boundaries = collect_tuplet_boundaries(track);
    let mut events = Vec::new();

    for entry in track.entries.by_id.values() {
        match entry {
            Entry::Tone(tone) => {
                let expanded_start = project_host_owned_tone_start(tone.tick, &boundaries);
                push_tone_event(&mut events, tone, expanded_start, &boundaries);
            }
            Entry::Tuplet(tuplet) => {
                let Some(boundary) = boundaries.iter().find(|candidate| {
                    candidate.host_start == tuplet.tick
                        && candidate.local_length == tuplet.local_length_ticks()
                }) else {
                    continue;
                };

                for child_entry in tuplet.entries.by_id.values() {
                    let TupletEntry::Tone(tone) = child_entry;
                    let expanded_start = boundary.expanded_start + tone.tick;
                    push_tone_event(&mut events, tone, expanded_start, &boundaries);
                }
            }
            _ => {}
        }
    }

    events.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then(left.pitch.cmp(&right.pitch))
            .then(left.velocity.cmp(&right.velocity))
    });

    events
}

fn push_tone_event(
    events: &mut Vec<PlaybackToneEvent>,
    tone: &Tone,
    expanded_start: Tick,
    boundaries: &[TupletBoundary],
) {
    let expanded_end = expanded_start + tone.duration;
    let start = project_expanded_offset_to_host(expanded_start, boundaries);
    let end = project_expanded_offset_to_host(expanded_end, boundaries);
    let duration = end.sub(&start);

    if duration.numerator > 0 {
        events.push(PlaybackToneEvent {
            start,
            duration,
            pitch: tone.pitch.int,
            velocity: tone.velocity.int(),
        });
    }
}

fn collect_tuplet_boundaries(track: &Track) -> Vec<TupletBoundary> {
    let mut tuplets: Vec<&Tuplet> = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::Tuplet(tuplet) => Some(tuplet.as_ref()),
            _ => None,
        })
        .collect();

    tuplets.sort_by_key(|tuplet| tuplet.tick);

    let mut expansion = 0;
    let mut boundaries = Vec::with_capacity(tuplets.len());

    for tuplet in tuplets {
        let local_length = tuplet.local_length_ticks();
        let consumed_ticks = tuplet.consumed_ticks();
        let expanded_start = tuplet.tick + expansion;
        let expanded_end = expanded_start + local_length;

        boundaries.push(TupletBoundary {
            host_start: tuplet.tick,
            host_end: tuplet.tick + consumed_ticks,
            expanded_start,
            expanded_end,
            local_length,
            consumed_ticks,
        });

        expansion += local_length - consumed_ticks;
    }

    boundaries
}

fn project_host_owned_tone_start(host_tick: Tick, boundaries: &[TupletBoundary]) -> Tick {
    let expansion = boundaries
        .iter()
        .filter(|boundary| boundary.host_end <= host_tick)
        .map(|boundary| boundary.local_length - boundary.consumed_ticks)
        .sum::<Tick>();

    host_tick + expansion
}

fn project_expanded_offset_to_host(
    expanded_offset: Tick,
    boundaries: &[TupletBoundary],
) -> PlaybackTick {
    let mut expansion = 0;

    for boundary in boundaries {
        if expanded_offset < boundary.expanded_start {
            break;
        }

        if expanded_offset < boundary.expanded_end {
            return boundary.project_expanded_offset_to_host(expanded_offset);
        }

        expansion += boundary.local_length - boundary.consumed_ticks;
    }

    PlaybackTick::from_tick(expanded_offset - expansion)
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    if left == 0 {
        return right.max(1);
    }

    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }

    left.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::components::articulation::Articulations;
    use crate::score::components::duration::NoteDuration;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tone::Tone;
    use crate::score::entries::tuplet::{Tuplet, TupletEntries, TupletEntry};
    use crate::score::entries::Entry;
    use crate::score::tracks::Track;

    fn make_tone(id: u64, tick: u64, duration: u64, pitch: u8) -> Tone {
        Tone::new(
            id,
            tick,
            duration,
            Pitch::new(pitch, Accidental::Natural),
            Velocity::new(96),
            Articulations::new(),
        )
    }

    fn make_triplet(track_tick: u64, entries: &[(u64, u64, u64, u8)]) -> Entry {
        let mut tuplet_entries = TupletEntries::new();

        for (id, tick, duration, pitch) in entries {
            tuplet_entries
                .insert_entry(TupletEntry::Tone(make_tone(*id, *tick, *duration, *pitch)));
        }

        Entry::Tuplet(Box::new(Tuplet {
            id: 900,
            tick: track_tick,
            note_count: 3,
            in_space_of: 2,
            base_note_value: NoteDuration::Eighth,
            groupings: vec![3],
            entries: tuplet_entries,
        }))
    }

    #[test]
    fn projects_triplet_child_onsets_and_durations_exactly_into_host_time() {
        let mut track = Track::new(1);
        track.insert_entry(make_triplet(
            96,
            &[(10, 0, 16, 60), (11, 16, 16, 62), (12, 32, 16, 64)],
        ));

        let events = collect_track_playback_events(&track);

        assert_eq!(
            events,
            vec![
                PlaybackToneEvent {
                    start: PlaybackTick::from_tick(96),
                    duration: PlaybackTick::new(32, 3),
                    pitch: 60,
                    velocity: 96,
                },
                PlaybackToneEvent {
                    start: PlaybackTick::new(320, 3),
                    duration: PlaybackTick::new(32, 3),
                    pitch: 62,
                    velocity: 96,
                },
                PlaybackToneEvent {
                    start: PlaybackTick::new(352, 3),
                    duration: PlaybackTick::new(32, 3),
                    pitch: 64,
                    velocity: 96,
                },
            ]
        );
    }

    #[test]
    fn projects_partially_filled_triplet_without_inventing_middle_events() {
        let mut track = Track::new(1);
        track.insert_entry(make_triplet(96, &[(10, 0, 16, 60), (12, 32, 16, 64)]));

        let events = collect_track_playback_events(&track);

        assert_eq!(
            events,
            vec![
                PlaybackToneEvent {
                    start: PlaybackTick::from_tick(96),
                    duration: PlaybackTick::new(32, 3),
                    pitch: 60,
                    velocity: 96,
                },
                PlaybackToneEvent {
                    start: PlaybackTick::new(352, 3),
                    duration: PlaybackTick::new(32, 3),
                    pitch: 64,
                    velocity: 96,
                },
            ]
        );
    }

    #[test]
    fn keeps_host_owned_tone_continuous_when_it_enters_a_tuplet() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 80, 64, 60)));
        track.insert_entry(make_triplet(96, &[]));

        let events = collect_track_playback_events(&track);

        assert_eq!(
            events,
            vec![PlaybackToneEvent {
                start: PlaybackTick::from_tick(80),
                duration: PlaybackTick::from_tick(48),
                pitch: 60,
                velocity: 96,
            },]
        );
    }

    #[test]
    fn keeps_tuplet_owned_tone_continuous_when_it_exits_into_host_time() {
        let mut track = Track::new(1);
        track.insert_entry(make_triplet(96, &[(10, 32, 32, 60)]));

        let events = collect_track_playback_events(&track);

        assert_eq!(
            events,
            vec![PlaybackToneEvent {
                start: PlaybackTick::new(352, 3),
                duration: PlaybackTick::new(80, 3),
                pitch: 60,
                velocity: 96,
            },]
        );
    }

    #[test]
    fn mixed_host_and_tuplet_material_stays_exact_without_drift() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 80, 16, 55)));
        track.insert_entry(make_triplet(
            96,
            &[(10, 0, 16, 60), (11, 16, 16, 62), (12, 32, 16, 64)],
        ));
        track.insert_entry(Entry::Tone(make_tone(2, 128, 16, 67)));

        let starts: Vec<PlaybackTick> = collect_track_playback_events(&track)
            .into_iter()
            .map(|event| event.start)
            .collect();

        assert_eq!(
            starts,
            vec![
                PlaybackTick::from_tick(80),
                PlaybackTick::from_tick(96),
                PlaybackTick::new(320, 3),
                PlaybackTick::new(352, 3),
                PlaybackTick::from_tick(128),
            ]
        );
    }

    #[test]
    fn flat_tracks_remain_integer_timed() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 32, 16, 60)));
        track.insert_entry(Entry::Tone(make_tone(2, 48, 32, 64)));

        let events = collect_track_playback_events(&track);

        assert_eq!(
            events,
            vec![
                PlaybackToneEvent {
                    start: PlaybackTick::from_tick(32),
                    duration: PlaybackTick::from_tick(16),
                    pitch: 60,
                    velocity: 96,
                },
                PlaybackToneEvent {
                    start: PlaybackTick::from_tick(48),
                    duration: PlaybackTick::from_tick(32),
                    pitch: 64,
                    velocity: 96,
                },
            ]
        );
    }

    #[test]
    fn collect_track_playback_events_keeps_host_only_passages_integer_timed() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 32, 16, 60)));
        track.insert_entry(Entry::Tone(make_tone(2, 80, 24, 64)));
        track.insert_entry(Entry::Tone(make_tone(3, 111, 16, 67)));

        let events = collect_track_playback_events(&track);

        assert_eq!(
            events,
            vec![
                PlaybackToneEvent {
                    start: PlaybackTick::from_tick(32),
                    duration: PlaybackTick::from_tick(16),
                    pitch: 60,
                    velocity: 96,
                },
                PlaybackToneEvent {
                    start: PlaybackTick::from_tick(80),
                    duration: PlaybackTick::from_tick(24),
                    pitch: 64,
                    velocity: 96,
                },
                PlaybackToneEvent {
                    start: PlaybackTick::from_tick(111),
                    duration: PlaybackTick::from_tick(16),
                    pitch: 67,
                    velocity: 96,
                },
            ]
        );

        assert!(events
            .iter()
            .all(|event| { event.start.denominator == 1 && event.duration.denominator == 1 }));
    }
}
