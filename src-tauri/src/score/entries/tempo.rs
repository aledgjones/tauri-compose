use super::Entry;
use crate::playback::transport::{sync_transport_to_app_state, TransportSharedState};
use crate::renderer::converter::Space;
use crate::renderer::text_measurer::TextMeasurer;
use crate::score::components::duration::{NoteDuration, Tick};
use crate::score::components::measurements::BoundingBox;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::tracks::Track;
use crate::utils::UniqueId;
use crate::AppContextState;
use crate::RendererContextState;
use serde::{Deserialize, Serialize};
use tauri::State;

// Horizontal spacing used by both tempo measurement and tempo drawing.
//
// These values describe the internal layout of a single rendered tempo mark.
// They do NOT affect score flow spacing or push other notation horizontally.
// They only control the relative spacing between the visible tempo parts.
//
// Layout model:
//
//   text         gap      (   note   dots   gap   =120)
//   Allegro      ^           ^                  ^
//                |           |                  |
//                |           |                  +-- TEMPO_METRONOME_EQUALS_GAP
//                |           +--------------------- note glyph is the only separately-spaced item
//                +--------------------------------- TEMPO_TEXT_GAP
//
// Dot placement:
//
//   note   dot1  dot2
//          <->   <->
//        TEMPO_DOT_GAP
//
// All values are in staff-space units.
pub const TEMPO_TEXT_GAP: Space = 0.3;
pub const TEMPO_METRONOME_EQUALS_GAP: Space = 0.1;
pub const TEMPO_DOT_GAP: Space = 0.2;
pub const TEMPO_STAVE_OFFSET: Space = 5.0;
pub const DEFAULT_TEMPO_BPM: u16 = 120;

#[derive(Clone)]
pub struct EffectiveTempoResolver {
    fallback: Tempo,
    changes: Vec<Tempo>,
}

impl EffectiveTempoResolver {
    pub fn from_track(track: &Track) -> Self {
        let mut changes = track
            .entries
            .by_id
            .values()
            .filter_map(|entry| match entry {
                Entry::Tempo(tempo) => Some(tempo.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        changes.sort_unstable_by_key(|tempo| tempo.tick);
        validate_tempo_entries(&changes);

        Self {
            fallback: Tempo::transport_fallback(),
            changes,
        }
    }

    pub fn resolve(&self, tick: Tick) -> &Tempo {
        let index = self.changes.partition_point(|tempo| tempo.tick <= tick);

        if index == 0 {
            &self.fallback
        } else {
            &self.changes[index - 1]
        }
    }
}

fn validate_tempo_entries(changes: &[Tempo]) {
    let mut previous_tick = None;

    for tempo in changes {
        assert!(
            (20..=400).contains(&tempo.bpm),
            "Tempo BPM {} out of range at tick {}",
            tempo.bpm,
            tempo.tick
        );

        if let Some(last_tick) = previous_tick {
            assert_ne!(
                last_tick, tempo.tick,
                "Multiple tempo entries found at tick {}",
                tempo.tick
            );
        }

        previous_tick = Some(tempo.tick);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tempo {
    pub id: UniqueId,
    pub tick: Tick,
    pub text: String,
    pub beat_type: NoteDuration,
    pub dots: u8,
    pub bpm: u16,
    pub hidden: bool,
    pub show_text: bool,
    pub show_metronome: bool,
    pub parenthesized: bool,
}

impl Tempo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: UniqueId,
        tick: Tick,
        text: String,
        beat_type: NoteDuration,
        dots: u8,
        bpm: u16,
        hidden: bool,
        show_text: bool,
        show_metronome: bool,
        parenthesized: bool,
    ) -> Self {
        Self {
            id,
            tick,
            text,
            beat_type,
            dots,
            bpm,
            hidden,
            show_text,
            show_metronome,
            parenthesized,
        }
    }

    pub fn text_font() -> Font {
        Font {
            size: 1.7,
            family: FontFamily::LibreBaskerville,
            justify: Justify::Start,
            align: Align::Bottom,
        }
    }

    pub fn transport_fallback() -> Self {
        Self::new(
            0,
            0,
            String::new(),
            NoteDuration::Quarter,
            0,
            DEFAULT_TEMPO_BPM,
            true,
            false,
            false,
            false,
        )
    }

    pub fn bpm_f64(&self) -> f64 {
        f64::from(self.bpm)
    }

    pub fn metronome_font() -> Font {
        Font {
            size: 1.75,
            family: FontFamily::Bravura,
            justify: Justify::Start,
            align: Align::Bottom,
        }
    }

    pub fn note_glyph(&self) -> &'static str {
        match self.beat_type {
            NoteDuration::Whole => "\u{E1D2}",
            NoteDuration::Half => "\u{E1D3}",
            NoteDuration::Quarter => "\u{E1D5}",
            NoteDuration::Eighth => "\u{E1D7}",
            NoteDuration::Sixteenth => "\u{E1D9}",
            NoteDuration::ThirtySecond => "\u{E1DB}",
            NoteDuration::SixtyFourth | NoteDuration::HudredTwentyEighth => "\u{E1DB}",
        }
    }

    pub fn display_text(&self) -> Option<&str> {
        if self.show_text && !self.text.is_empty() {
            Some(self.text.as_str())
        } else {
            None
        }
    }

    pub fn metronome_prefix_text(&self) -> &'static str {
        if self.parenthesized {
            "("
        } else {
            ""
        }
    }

    pub fn metronome_suffix_text(&self) -> String {
        if self.parenthesized {
            format!("= {})", self.bpm)
        } else {
            format!("= {}", self.bpm)
        }
    }

    fn text_width(&self, text_measurer: &TextMeasurer<'_>) -> Space {
        match self.display_text() {
            Some(text) => text_measurer.measure(text, &Self::text_font()).width,
            None => 0.0,
        }
    }

    fn text_height(&self, text_measurer: &TextMeasurer<'_>) -> Space {
        match self.display_text() {
            Some(text) => text_measurer.measure(text, &Self::text_font()).height,
            None => 0.0,
        }
    }

    fn metronome_width(&self, text_measurer: &TextMeasurer<'_>) -> Space {
        if self.hidden || !self.show_metronome {
            return 0.0;
        }

        let note_width = text_measurer
            .measure(self.note_glyph(), &Self::metronome_font())
            .width;
        let prefix_width = text_measurer
            .measure(self.metronome_prefix_text(), &Self::text_font())
            .width;
        let suffix_width = text_measurer
            .measure(&self.metronome_suffix_text(), &Self::text_font())
            .width;
        let dot_width = if self.dots == 0 {
            0.0
        } else {
            (self.dots as f32 * TEMPO_DOT_GAP) + 0.15
        };

        prefix_width + note_width + dot_width + TEMPO_METRONOME_EQUALS_GAP + suffix_width
    }

    fn metronome_text_height(&self, text_measurer: &TextMeasurer<'_>) -> Space {
        if self.hidden || !self.show_metronome {
            return 0.0;
        }

        let prefix_height = if self.parenthesized {
            text_measurer
                .measure(self.metronome_prefix_text(), &Self::text_font())
                .height
        } else {
            0.0
        };
        let suffix_height = text_measurer
            .measure(&self.metronome_suffix_text(), &Self::text_font())
            .height;

        prefix_height.max(suffix_height)
    }

    pub fn metrics(&self, text_measurer: &TextMeasurer<'_>) -> BoundingBox {
        if self.hidden {
            return BoundingBox {
                width: 0.0,
                height: 0.0,
                right_padding: 0.0,
            };
        }

        let text_width = self.text_width(text_measurer);
        let text_height = self.text_height(text_measurer);
        let metronome_width = self.metronome_width(text_measurer);
        let metronome_text_height = self.metronome_text_height(text_measurer);
        let gap = if text_width > 0.0 && metronome_width > 0.0 {
            TEMPO_TEXT_GAP
        } else {
            0.0
        };
        let width = text_width + gap + metronome_width;
        let height = text_height.max(metronome_text_height);

        BoundingBox {
            width,
            height,
            right_padding: 0.0,
        }
    }
}

impl Track {
    pub fn effective_tempo_resolver(&self) -> EffectiveTempoResolver {
        EffectiveTempoResolver::from_track(self)
    }

    pub fn get_tempo_at_tick(&self, at: &Tick) -> Option<&Tempo> {
        let entry_keys = self.entries.by_tick.get(at)?;

        for key in entry_keys {
            if let Some(Entry::Tempo(tempo)) = self.entries.by_id.get(key) {
                return Some(tempo);
            }
        }

        None
    }

    pub fn get_tempo_before_tick(&self, tick: Tick) -> Option<&Tempo> {
        for current in (0..tick).rev() {
            if let Some(tempo) = self.get_tempo_at_tick(&current) {
                return Some(tempo);
            }
        }

        None
    }
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tempo_create(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    tick: Tick,
    text: String,
    beat_type: NoteDuration,
    dots: u8,
    bpm: u16,
    hidden: bool,
    show_text: bool,
    show_metronome: bool,
    parenthesized: bool,
) -> UniqueId {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let tempo_id = state.generate_id();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();
    let _previous_tempo = master.get_tempo_before_tick(tick);

    if let Some(old) = master.get_tempo_at_tick(&tick) {
        master.remove_entry(old.id);
    }

    master.insert_entry(Entry::Tempo(Tempo::new(
        tempo_id,
        tick,
        text,
        beat_type,
        dots,
        bpm,
        hidden,
        show_text,
        show_metronome,
        parenthesized,
    )));

    let current_tick = transport_shared
        .current_tick
        .load(std::sync::atomic::Ordering::Relaxed);
    sync_transport_to_app_state(transport_shared.inner(), &state, current_tick);

    state.emit(&app, &mut renderer, true);

    tempo_id
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tempo_update(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    tempo_id: UniqueId,
    text: String,
    beat_type: NoteDuration,
    dots: u8,
    bpm: u16,
    hidden: bool,
    show_text: bool,
    show_metronome: bool,
    parenthesized: bool,
) -> Result<(), String> {
    if !(20..=400).contains(&bpm) {
        return Err(format!("Tempo BPM {} out of range", bpm));
    }

    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let master_id = state
        .score
        .flows
        .by_id
        .get(&flow_id)
        .map(|flow| flow.master_id)
        .ok_or_else(|| format!("Flow {} not found", flow_id))?;
    let master = state
        .score
        .tracks
        .by_id
        .get_mut(&master_id)
        .ok_or_else(|| format!("Master track {} not found", master_id))?;

    let tempo = master
        .entries
        .by_id
        .get_mut(&tempo_id)
        .ok_or_else(|| format!("Tempo {} not found", tempo_id))?;

    let Entry::Tempo(tempo) = tempo else {
        return Err(format!("Entry {} is not a tempo", tempo_id));
    };

    tempo.text = text;
    tempo.beat_type = beat_type;
    tempo.dots = dots;
    tempo.bpm = bpm;
    tempo.hidden = hidden;
    tempo.show_text = show_text;
    tempo.show_metronome = show_metronome;
    tempo.parenthesized = parenthesized;

    let current_tick = transport_shared
        .current_tick
        .load(std::sync::atomic::Ordering::Relaxed);
    sync_transport_to_app_state(transport_shared.inner(), &state, current_tick);

    state.emit(&app, &mut renderer, true);

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_entries_tempo_clear(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    transport_shared: State<'_, TransportSharedState>,
    flow_id: UniqueId,
    tick: Tick,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let flow = &state.score.flows.by_id[&flow_id];
    let master_id = flow.master_id;
    let master = state.score.tracks.by_id.get_mut(&master_id).unwrap();

    if let Some(old) = master.get_tempo_at_tick(&tick) {
        master.remove_entry(old.id);
    }

    let current_tick = transport_shared
        .current_tick
        .load(std::sync::atomic::Ordering::Relaxed);
    sync_transport_to_app_state(transport_shared.inner(), &state, current_tick);

    state.emit(&app, &mut renderer, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{test_helpers::make_test_renderer, text_measurer::TextMeasurer};
    use crate::score::tracks::Track;

    fn make_tempo(id: UniqueId, tick: Tick) -> Tempo {
        Tempo::new(
            id,
            tick,
            String::from("Allegro"),
            NoteDuration::Quarter,
            0,
            120,
            false,
            true,
            true,
            false,
        )
    }

    #[test]
    fn tempo_new_stores_fields() {
        let tempo = make_tempo(10, 64);
        assert_eq!(tempo.id, 10);
        assert_eq!(tempo.tick, 64);
        assert_eq!(tempo.text, "Allegro");
        assert_eq!(tempo.bpm, 120);
    }

    #[test]
    fn hidden_tempo_has_zero_metrics() {
        let renderer = make_test_renderer();
        let text_measurer = TextMeasurer::new(&renderer.converter);
        let tempo = Tempo::new(
            1,
            0,
            String::from("Allegro"),
            NoteDuration::Quarter,
            0,
            120,
            true,
            true,
            true,
            false,
        );

        let metrics = tempo.metrics(&text_measurer);
        assert_eq!(metrics.width, 0.0);
        assert_eq!(metrics.height, 0.0);
        assert_eq!(metrics.right_padding, 0.0);
    }

    #[test]
    fn metrics_are_positive_for_visible_tempo() {
        let renderer = make_test_renderer();
        let text_measurer = TextMeasurer::new(&renderer.converter);
        let metrics = make_tempo(1, 0).metrics(&text_measurer);
        assert!(metrics.width > 0.0);
        assert!(metrics.height > 0.0);
        assert_eq!(metrics.right_padding, 0.0);
    }

    #[test]
    fn metrics_height_uses_libre_text_only() {
        let renderer = make_test_renderer();
        let text_measurer = TextMeasurer::new(&renderer.converter);
        let tempo = Tempo::new(
            1,
            0,
            String::from("Allegro"),
            NoteDuration::Quarter,
            0,
            120,
            false,
            true,
            true,
            false,
        );

        let metrics = tempo.metrics(&text_measurer);
        let expected_height = text_measurer
            .measure("Allegro", &Tempo::text_font())
            .height
            .max(text_measurer.measure("= 120", &Tempo::text_font()).height);

        assert!((metrics.height - expected_height).abs() < f32::EPSILON);
    }

    #[test]
    fn get_tempo_at_tick_returns_matching_entry() {
        let mut track = Track::new(1);
        let tempo = make_tempo(7, 32);
        track.insert_entry(Entry::Tempo(tempo));

        let found = track.get_tempo_at_tick(&32).unwrap();
        assert_eq!(found.id, 7);
    }

    #[test]
    fn get_tempo_before_tick_returns_previous_tempo() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tempo(make_tempo(7, 32)));
        track.insert_entry(Entry::Tempo(make_tempo(8, 96)));

        let found = track.get_tempo_before_tick(96).unwrap();
        assert_eq!(found.id, 7);
    }

    #[test]
    fn replacing_tempo_at_tick_keeps_only_latest_entry() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tempo(make_tempo(7, 32)));
        if let Some(old) = track.get_tempo_at_tick(&32) {
            track.remove_entry(old.id);
        }
        track.insert_entry(Entry::Tempo(make_tempo(8, 32)));

        let ids = track.entries.by_tick.get(&32).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 8);
    }

    #[test]
    fn note_glyphs_match_supported_durations() {
        let mut tempo = make_tempo(1, 0);
        tempo.beat_type = NoteDuration::Quarter;
        assert_eq!(tempo.note_glyph(), "\u{E1D5}");

        tempo.beat_type = NoteDuration::Eighth;
        assert_eq!(tempo.note_glyph(), "\u{E1D7}");
    }

    #[test]
    fn effective_tempo_resolver_uses_fallback_when_no_tempo_exists() {
        let track = Track::new(1);

        let resolver = track.effective_tempo_resolver();

        assert_eq!(resolver.resolve(0).bpm, DEFAULT_TEMPO_BPM);
        assert_eq!(resolver.resolve(64).tick, 0);
    }

    #[test]
    fn effective_tempo_resolver_prefers_tick_zero_tempo() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tempo(make_tempo(7, 0)));

        let resolver = track.effective_tempo_resolver();

        assert_eq!(resolver.resolve(0).id, 7);
        assert_eq!(resolver.resolve(0).bpm, 120);
    }

    #[test]
    fn effective_tempo_resolver_inherits_latest_tempo_at_or_before_tick() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tempo(make_tempo(7, 32)));
        track.insert_entry(Entry::Tempo(Tempo {
            bpm: 84,
            ..make_tempo(8, 96)
        }));

        let resolver = track.effective_tempo_resolver();

        assert_eq!(resolver.resolve(95).id, 7);
        assert_eq!(resolver.resolve(96).id, 8);
        assert_eq!(resolver.resolve(128).bpm, 84);
    }

    #[test]
    #[should_panic(expected = "Multiple tempo entries found at tick 32")]
    fn effective_tempo_resolver_panics_on_duplicate_ticks() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tempo(make_tempo(7, 32)));
        track.insert_entry(Entry::Tempo(make_tempo(8, 32)));

        let _ = track.effective_tempo_resolver();
    }

    #[test]
    #[should_panic(expected = "Tempo BPM 12 out of range at tick 64")]
    fn effective_tempo_resolver_panics_on_out_of_range_bpm() {
        let mut track = Track::new(1);
        track.insert_entry(Entry::Tempo(Tempo {
            bpm: 12,
            ..make_tempo(7, 64)
        }));

        let _ = track.effective_tempo_resolver();
    }
}
