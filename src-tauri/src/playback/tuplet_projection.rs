use crate::playback::expression_maps::ExpressionMapPlaybackContext;
use crate::score::components::duration::{Tick, Ticks};
use crate::score::entries::playing_technique::{
    EffectivePlayingTechniqueContext, EffectivePlayingTechniqueResolver,
};
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
    pub expression_map_context: ExpressionMapPlaybackContext,
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

pub fn collect_track_playback_events(
    track: &Track,
    technique_resolver: Option<&EffectivePlayingTechniqueResolver>,
) -> Vec<PlaybackToneEvent> {
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

    apply_effective_playing_techniques(&mut events, technique_resolver);

    events
}

fn apply_effective_playing_techniques(
    events: &mut [PlaybackToneEvent],
    technique_resolver: Option<&EffectivePlayingTechniqueResolver>,
) {
    let Some(technique_resolver) = technique_resolver else {
        return;
    };

    let mut context = EffectivePlayingTechniqueContext::new();
    let mut next_change_index = 0;

    for event in events {
        while let Some(change) = technique_resolver.changes().get(next_change_index) {
            if change.tick > event.start.floor() {
                break;
            }

            context.set_active(change.kind);
            next_change_index += 1;
        }

        event.expression_map_context.effective_playing_techniques = context.clone();
    }
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
            expression_map_context: ExpressionMapPlaybackContext {
                articulations: tone.articulations.clone(),
                written_duration_ticks: tone.duration,
                effective_playing_techniques: EffectivePlayingTechniqueContext::new(),
            },
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
    use crate::score::entries::playing_technique::{
        PlayingTechnique, PlayingTechniqueExclusivityGroup, PlayingTechniqueKind,
    };
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

    fn expected_event(
        start: PlaybackTick,
        duration: PlaybackTick,
        pitch: u8,
        written_duration_ticks: Ticks,
    ) -> PlaybackToneEvent {
        expected_event_with_context(
            start,
            duration,
            pitch,
            written_duration_ticks,
            EffectivePlayingTechniqueContext::new(),
        )
    }

    fn expected_event_with_context(
        start: PlaybackTick,
        duration: PlaybackTick,
        pitch: u8,
        written_duration_ticks: Ticks,
        effective_playing_techniques: EffectivePlayingTechniqueContext,
    ) -> PlaybackToneEvent {
        PlaybackToneEvent {
            start,
            duration,
            pitch,
            velocity: 96,
            expression_map_context: ExpressionMapPlaybackContext {
                articulations: Articulations::new(),
                written_duration_ticks,
                effective_playing_techniques,
            },
        }
    }

    fn bowed_context(kind: PlayingTechniqueKind) -> EffectivePlayingTechniqueContext {
        let mut context = EffectivePlayingTechniqueContext::new();
        context.set_active(kind);
        context
    }

    fn make_playing_technique_entry(id: u64, tick: u64, kind: PlayingTechniqueKind) -> Entry {
        Entry::PlayingTechnique(PlayingTechnique::new(id, tick, kind))
    }

    #[test]
    fn projects_triplet_child_onsets_and_durations_exactly_into_host_time() {
        let mut track = Track::new(1);
        track.insert_entry(make_triplet(
            96,
            &[(10, 0, 16, 60), (11, 16, 16, 62), (12, 32, 16, 64)],
        ));

        let events = collect_track_playback_events(&track, None);

        assert_eq!(
            events,
            vec![
                expected_event(
                    PlaybackTick::from_tick(96),
                    PlaybackTick::new(32, 3),
                    60,
                    16
                ),
                expected_event(PlaybackTick::new(320, 3), PlaybackTick::new(32, 3), 62, 16),
                expected_event(PlaybackTick::new(352, 3), PlaybackTick::new(32, 3), 64, 16),
            ]
        );
    }

    #[test]
    fn projects_partially_filled_triplet_without_inventing_middle_events() {
        let mut track = Track::new(1);
        track.insert_entry(make_triplet(96, &[(10, 0, 16, 60), (12, 32, 16, 64)]));

        let events = collect_track_playback_events(&track, None);

        assert_eq!(
            events,
            vec![
                expected_event(
                    PlaybackTick::from_tick(96),
                    PlaybackTick::new(32, 3),
                    60,
                    16
                ),
                expected_event(PlaybackTick::new(352, 3), PlaybackTick::new(32, 3), 64, 16),
            ]
        );
    }

    #[test]
    fn keeps_host_owned_tone_continuous_when_it_enters_a_tuplet() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 80, 64, 60)));
        track.insert_entry(make_triplet(96, &[]));

        let events = collect_track_playback_events(&track, None);

        assert_eq!(
            events,
            vec![expected_event(
                PlaybackTick::from_tick(80),
                PlaybackTick::from_tick(48),
                60,
                64,
            ),]
        );
    }

    #[test]
    fn keeps_tuplet_owned_tone_continuous_when_it_exits_into_host_time() {
        let mut track = Track::new(1);
        track.insert_entry(make_triplet(96, &[(10, 32, 32, 60)]));

        let events = collect_track_playback_events(&track, None);

        assert_eq!(
            events,
            vec![expected_event(
                PlaybackTick::new(352, 3),
                PlaybackTick::new(80, 3),
                60,
                32,
            ),]
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

        let starts: Vec<PlaybackTick> = collect_track_playback_events(&track, None)
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

        let events = collect_track_playback_events(&track, None);

        assert_eq!(
            events,
            vec![
                expected_event(
                    PlaybackTick::from_tick(32),
                    PlaybackTick::from_tick(16),
                    60,
                    16
                ),
                expected_event(
                    PlaybackTick::from_tick(48),
                    PlaybackTick::from_tick(32),
                    64,
                    32
                ),
            ]
        );
    }

    #[test]
    fn collect_track_playback_events_keeps_host_only_passages_integer_timed() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 32, 16, 60)));
        track.insert_entry(Entry::Tone(make_tone(2, 80, 24, 64)));
        track.insert_entry(Entry::Tone(make_tone(3, 111, 16, 67)));

        let events = collect_track_playback_events(&track, None);

        assert_eq!(
            events,
            vec![
                expected_event(
                    PlaybackTick::from_tick(32),
                    PlaybackTick::from_tick(16),
                    60,
                    16
                ),
                expected_event(
                    PlaybackTick::from_tick(80),
                    PlaybackTick::from_tick(24),
                    64,
                    24
                ),
                expected_event(
                    PlaybackTick::from_tick(111),
                    PlaybackTick::from_tick(16),
                    67,
                    16
                ),
            ]
        );

        assert!(events
            .iter()
            .all(|event| { event.start.denominator == 1 && event.duration.denominator == 1 }));
    }

    #[test]
    fn collect_track_playback_events_applies_master_track_techniques_to_host_and_tuplet_tones() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 96, 16, 60)));
        track.insert_entry(make_triplet(128, &[(10, 0, 16, 62), (11, 16, 16, 64)]));
        track.insert_entry(Entry::Tone(make_tone(2, 160, 16, 67)));

        let mut master_track = Track::new(2);
        master_track.insert_entry(make_playing_technique_entry(
            100,
            96,
            PlayingTechniqueKind::Pizz,
        ));
        master_track.insert_entry(make_playing_technique_entry(
            101,
            128,
            PlayingTechniqueKind::Arco,
        ));

        let resolver = master_track.effective_playing_technique_resolver();
        let events = collect_track_playback_events(&track, Some(&resolver));

        assert_eq!(
            events,
            vec![
                expected_event_with_context(
                    PlaybackTick::from_tick(96),
                    PlaybackTick::from_tick(16),
                    60,
                    16,
                    bowed_context(PlayingTechniqueKind::Pizz),
                ),
                expected_event_with_context(
                    PlaybackTick::from_tick(128),
                    PlaybackTick::new(32, 3),
                    62,
                    16,
                    bowed_context(PlayingTechniqueKind::Arco),
                ),
                expected_event_with_context(
                    PlaybackTick::new(416, 3),
                    PlaybackTick::new(32, 3),
                    64,
                    16,
                    bowed_context(PlayingTechniqueKind::Arco),
                ),
                expected_event_with_context(
                    PlaybackTick::from_tick(160),
                    PlaybackTick::from_tick(16),
                    67,
                    16,
                    bowed_context(PlayingTechniqueKind::Arco),
                ),
            ]
        );
    }

    #[test]
    fn collect_track_playback_events_keeps_same_tick_conflicts_deterministic_for_prepared_context()
    {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tone(make_tone(1, 96, 16, 60)));

        let mut master_track = Track::new(2);
        master_track.insert_entry(make_playing_technique_entry(
            200,
            96,
            PlayingTechniqueKind::Arco,
        ));
        master_track.insert_entry(make_playing_technique_entry(
            150,
            96,
            PlayingTechniqueKind::Pizz,
        ));

        let resolver = master_track.effective_playing_technique_resolver();
        let events = collect_track_playback_events(&track, Some(&resolver));

        assert_eq!(
            events[0]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Arco)
        );
    }

    #[test]
    fn collect_track_playback_events_reuses_same_master_context_across_same_stave_tracks() {
        let mut upper_track = Track::new(1);
        upper_track.insert_entry(Entry::Tone(make_tone(1, 96, 16, 60)));
        let mut lower_track = Track::new(2);
        lower_track.insert_entry(Entry::Tone(make_tone(2, 128, 16, 55)));

        let mut master_track = Track::new(3);
        master_track.insert_entry(make_playing_technique_entry(
            100,
            96,
            PlayingTechniqueKind::Pizz,
        ));

        let resolver = master_track.effective_playing_technique_resolver();
        let upper_events = collect_track_playback_events(&upper_track, Some(&resolver));
        let lower_events = collect_track_playback_events(&lower_track, Some(&resolver));

        assert_eq!(
            upper_events[0]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Pizz)
        );
        assert_eq!(
            lower_events[0]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Pizz)
        );
    }

    #[test]
    fn collect_track_playback_events_keep_multi_stave_contexts_independent() {
        let mut upper_track = Track::new(1);
        upper_track.insert_entry(Entry::Tone(make_tone(1, 96, 16, 60)));
        let mut lower_track = Track::new(2);
        lower_track.insert_entry(Entry::Tone(make_tone(2, 96, 16, 55)));

        let mut upper_master_track = Track::new(3);
        upper_master_track.insert_entry(make_playing_technique_entry(
            100,
            96,
            PlayingTechniqueKind::Pizz,
        ));
        let mut lower_master_track = Track::new(4);
        lower_master_track.insert_entry(make_playing_technique_entry(
            101,
            96,
            PlayingTechniqueKind::Arco,
        ));

        let upper_events = collect_track_playback_events(
            &upper_track,
            Some(&upper_master_track.effective_playing_technique_resolver()),
        );
        let lower_events = collect_track_playback_events(
            &lower_track,
            Some(&lower_master_track.effective_playing_technique_resolver()),
        );

        assert_eq!(
            upper_events[0]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Pizz)
        );
        assert_eq!(
            lower_events[0]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Arco)
        );
    }

    #[test]
    fn collect_track_playback_events_do_not_leak_between_flow_scoped_master_contexts() {
        let mut active_flow_track = Track::new(1);
        active_flow_track.insert_entry(Entry::Tone(make_tone(1, 96, 16, 60)));
        let mut sibling_flow_track = Track::new(2);
        sibling_flow_track.insert_entry(Entry::Tone(make_tone(2, 96, 16, 55)));

        let mut active_flow_master = Track::new(3);
        active_flow_master.insert_entry(make_playing_technique_entry(
            100,
            96,
            PlayingTechniqueKind::Pizz,
        ));
        let sibling_flow_master = Track::new(4);

        let active_events = collect_track_playback_events(
            &active_flow_track,
            Some(&active_flow_master.effective_playing_technique_resolver()),
        );
        let sibling_events = collect_track_playback_events(
            &sibling_flow_track,
            Some(&sibling_flow_master.effective_playing_technique_resolver()),
        );

        assert_eq!(
            active_events[0]
                .expression_map_context
                .effective_playing_techniques
                .active_kind(PlayingTechniqueExclusivityGroup::Bowed),
            Some(PlayingTechniqueKind::Pizz)
        );
        assert!(sibling_events[0]
            .expression_map_context
            .effective_playing_techniques
            .is_empty());
    }
}
