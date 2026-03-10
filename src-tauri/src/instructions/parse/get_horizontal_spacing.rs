use super::get_accidentals::AccidentalsByTrack;
use super::get_barlines::Barlines;
use super::get_beams::BeamsByTrack;
use super::get_measures::Measures;
use super::get_shunts::ShuntsByTrack;
use super::get_written_durations::NotationByTrack;
use crate::renderer::converter::Space;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::components::measurements::BoundingBox;
use crate::score::engraves::Engrave;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::entries::Entry;
use crate::score::entries::time_signature::TimeSignature;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;
use rayon::prelude::*;

const EMPTY_MEASURE_BASE_WIDTH_MULTIPLIER: f32 = 2.0;
const EMPTY_MEASURE_WIDTH_EXPONENT: f32 = 0.3;
const EMPTY_MEASURE_WIDTH_BLEND: f32 = 0.55;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Slot {
    PaddingStart = 0,
    EndRepeat,
    Clef,
    Barline,
    KeySignature,
    TimeSignature,
    StartRepeat,
    BarPadding,
    Accidentals, // only used if there is no previous space to cut into.
    PreNote,     // only used if there is no previous space to cut into.
    NoteSpacing,
}

impl Slot {
    pub fn count() -> usize {
        Slot::NoteSpacing as usize + 1
    }
}

#[derive(Clone)]
pub struct HorizontalSpacingSpan {
    pub x: Space,
    pub width: Space,
}

impl HorizontalSpacingSpan {
    pub fn new() -> Self {
        Self { width: 0.0, x: 0.0 }
    }
}

pub struct HorizontalSpacing {
    pub widths: Vec<HorizontalSpacingSpan>,
    pub width: Space,
}

impl HorizontalSpacing {
    pub fn new(length: Ticks) -> HorizontalSpacing {
        HorizontalSpacing {
            widths: vec![HorizontalSpacingSpan::new(); (length as usize + 1) * Slot::count()],
            width: 0.0,
        }
    }

    pub fn get(&self, tick: Tick, slot: Slot) -> HorizontalSpacingSpan {
        let index = (tick as usize * Slot::count()) + slot as usize;

        self.widths[index].clone()
    }

    pub fn set_width(&mut self, tick: Tick, slot: Slot, width: Space) {
        let index = (tick as usize * Slot::count()) + slot as usize;
        self.widths[index].width = width;
    }
}

fn empty_measure_width(time_signature: &TimeSignature, engrave: &Engrave) -> Space {
    let normalized_ticks =
        time_signature.ticks_per_bar() as f32 / NoteDuration::Whole.to_ticks() as f32;
    let compressed_width = normalized_ticks.powf(EMPTY_MEASURE_WIDTH_EXPONENT);
    let blended_width = 1.0 + ((compressed_width - 1.0) * EMPTY_MEASURE_WIDTH_BLEND);

    engrave.base_note_space
        * EMPTY_MEASURE_BASE_WIDTH_MULTIPLIER
        * blended_width
}

#[allow(clippy::too_many_arguments)]
pub fn get_horizontal_spacing(
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
    measures: &Measures,
    notations_by_track: &NotationByTrack,
    barlines: &Barlines,
    shunts_by_track: &ShuntsByTrack,
    beams_by_track: &BeamsByTrack,
    accidentals_by_track: &AccidentalsByTrack,
    engrave: &Engrave,
) -> HorizontalSpacing {
    let mut spacings = HorizontalSpacing::new(flow.length);
    let flow_master = &tracks.by_id[&flow.master_id];

    spacings.set_width(0, Slot::PaddingStart, 0.75);
    spacings.set_width(0, Slot::BarPadding, 0.5);

    for (tick, time_signature) in measures {
        let measure_stop = (*tick + time_signature.ticks_per_bar()).min(flow.length);
        if measure_stop <= *tick {
            continue;
        }

        let width_per_tick = empty_measure_width(time_signature, engrave) / (measure_stop - *tick) as f32;

        for measure_tick in *tick..measure_stop {
            let current = spacings.get(measure_tick, Slot::NoteSpacing).width;
            if width_per_tick > current {
                spacings.set_width(measure_tick, Slot::NoteSpacing, width_per_tick);
            }
        }
    }

    for entry in flow_master.entries.by_id.values() {
        if let Entry::TimeSignature(time_signature) = entry {
            let metrics = time_signature.metrics();
            spacings.set_width(
                time_signature.tick,
                Slot::TimeSignature,
                metrics.total_width(),
            );
        }
    }

    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_master = &tracks.by_id[&stave.master_id];

            for entry in flow_master.entries.by_id.values() {
                if let Entry::KeySignature(key_signature) = entry {
                    let render_key_signature = if key_signature.offset == 0 {
                        flow_master.get_key_signature_before_tick(key_signature.tick)
                    } else {
                        Some(key_signature)
                    };

                    let width = match stave_master
                        .get_clef_before_tick_inclusive(key_signature.tick)
                    {
                        Some(clef)
                            if clef.can_render_key_signature()
                                && render_key_signature
                                    .is_some_and(|signature| signature.pattern(clef).is_some()) =>
                        {
                            let metrics = if key_signature.offset == 0 {
                                match flow_master.get_key_signature_before_tick(key_signature.tick)
                                {
                                    Some(previous) => previous.metrics(),
                                    None => BoundingBox {
                                        width: 0.0,
                                        height: 0.0,
                                        right_padding: 0.0,
                                    },
                                }
                            } else {
                                key_signature.metrics()
                            };

                            metrics.total_width()
                        }
                        _ => 0.0,
                    };
                    let current = spacings.get(key_signature.tick, Slot::KeySignature).width;

                    if width > current {
                        spacings.set_width(key_signature.tick, Slot::KeySignature, width);
                    }
                }
            }

            for entry in stave_master.entries.by_id.values() {
                if let Entry::Clef(clef) = entry {
                    let metrics = clef.metrics(clef.tick == 0);
                    spacings.set_width(clef.tick, Slot::Clef, metrics.total_width());
                }
            }
        }
    }

    for (tick, def) in barlines {
        spacings.set_width(*tick, Slot::BarPadding, 0.5);

        if def.end_repeat {
            let metrics = BarlineDrawType::EndRepeat.metrics();
            spacings.set_width(*tick, Slot::EndRepeat, metrics.total_width());
        }

        if let Some(draw_type) = &def.draw_type {
            let metrics = draw_type.metrics();

            let has_time_signature = flow_master.get_time_signature_at_tick(tick).is_some();
            let has_key_signature = spacings.get(*tick, Slot::KeySignature).width > 0.0;

            if has_key_signature {
                spacings.set_width(*tick, Slot::Barline, metrics.width + 0.5);
            } else if has_time_signature {
                spacings.set_width(*tick, Slot::Barline, metrics.width + 0.25);
            } else {
                spacings.set_width(*tick, Slot::Barline, metrics.total_width());
            }
        }

        if def.start_repeat {
            let metrics = BarlineDrawType::StartRepeat.metrics();
            spacings.set_width(*tick, Slot::StartRepeat, metrics.total_width());
        }
    }

    let spacing_updates: Vec<(Tick, Slot, Space)> = notations_by_track
        .par_iter()
        .flat_map(|(track_key, notation)| {
            let shunts = &shunts_by_track[track_key];
            let accidentals = &accidentals_by_track[track_key];
            let beams = &beams_by_track[track_key];
            let mut updates: Vec<(Tick, Slot, Space)> = Vec::new();

            for (tick, entry) in &notation.track {
                let mut spacing = entry.metrics(shunts, engrave, beams).width;

                // if the tick is at the start of a barline or the start of
                // the piece we add explicit space for accidentals and pre-shunts
                if *tick == 0 || barlines.contains_key(tick) {
                    let metrics = entry.glyph_metrics();
                    let glyph_width = metrics.total_width();
                    let has_pre_shunt = entry.has_pre_shunt(shunts);

                    if has_pre_shunt {
                        updates.push((*tick, Slot::PreNote, glyph_width));
                    }

                    // start of bars has no previous spacing to extend so we use the accidentals slot
                    if let Some(slots) = accidentals.slots_by_tick.get(tick) {
                        let width = if has_pre_shunt {
                            ((*slots as f32 - 1.0) * 1.1) + 0.25
                        } else {
                            (*slots as f32 * 1.1) + 0.25
                        };
                        updates.push((*tick, Slot::Accidentals, width));
                    };
                }

                // extend spacing to accomodate accidentals + pre shunts that are part of the next note (if needed)
                if let Some((next_tick, next_entry)) = notation.get_next_notation(*tick) {
                    if !barlines.contains_key(&next_tick) {
                        let min = entry.min_spacing(shunts, engrave, beams);
                        let next_metrics = next_entry.glyph_metrics();
                        let next_glyph_width = next_metrics.total_width();
                        let next_has_pre_shunt = next_entry.has_pre_shunt(shunts);

                        let accidentals_width = match accidentals.slots_by_tick.get(&next_tick) {
                            Some(slots) if *slots > 0 => {
                                if next_has_pre_shunt {
                                    ((*slots as f32 - 1.0) * 1.1) + 0.25
                                } else {
                                    (*slots as f32 * 1.1) + 0.25
                                }
                            }
                            _ => 0.0,
                        };

                        let pre_shunt_width = if next_has_pre_shunt {
                            next_glyph_width
                        } else {
                            0.0
                        };

                        let ledger_line_width = match &entry.info {
                            Some(info) if info.highest < -5 || info.lowest > 5 => 0.4,
                            _ => 0.0,
                        };

                        let avoid_collisions_spacing = accidentals_width
                            + pre_shunt_width
                            + min.width
                            + ledger_line_width
                            + 0.2;

                        let min_natural_spacing = min.total_width() + pre_shunt_width;
                        if min_natural_spacing > spacing {
                            spacing = min_natural_spacing;
                        }

                        if avoid_collisions_spacing > spacing {
                            spacing = avoid_collisions_spacing;
                        }
                    }
                };

                let note_spacing_per_tick = spacing / entry.duration as f32;
                let end = tick + entry.duration;
                for i in *tick..end {
                    updates.push((i, Slot::NoteSpacing, note_spacing_per_tick));
                }
            }

            updates
        })
        .collect();

    // Apply all updates sequentially
    for (tick, slot, width) in spacing_updates {
        let current = spacings.get(tick, slot);
        if width > current.width {
            spacings.set_width(tick, slot, width);
        }
    }

    let mut x: Space = 0.0;
    for span in &mut spacings.widths {
        span.x = x;
        x += span.width;
    }

    let metrics = BarlineDrawType::Final.metrics();
    spacings.width = x + metrics.total_width();

    spacings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_measures::get_measures;

    #[test]
    fn slot_count_reflects_all_variants() {
        // Slot::NoteSpacing = 10, so count = 11
        assert_eq!(Slot::count(), 11);
    }

    #[test]
    fn horizontal_spacing_span_new_has_zero_values() {
        let span = HorizontalSpacingSpan::new();
        assert_eq!(span.x, 0.0);
        assert_eq!(span.width, 0.0);
    }

    #[test]
    fn horizontal_spacing_new_creates_correct_size() {
        // length=1 → (1+1)*11 = 22 spans
        let hs = HorizontalSpacing::new(1);
        assert_eq!(hs.widths.len(), 22);
    }

    #[test]
    fn horizontal_spacing_get_returns_default_span() {
        let hs = HorizontalSpacing::new(4);
        let span = hs.get(0, Slot::NoteSpacing);
        assert_eq!(span.width, 0.0);
    }

    #[test]
    fn horizontal_spacing_set_width_updates_correct_slot() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(2, Slot::Barline, 3.5);
        let span = hs.get(2, Slot::Barline);
        assert!((span.width - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_spacing_set_does_not_affect_other_slots() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(1, Slot::Clef, 2.0);
        let unaffected = hs.get(1, Slot::Barline);
        assert_eq!(unaffected.width, 0.0);
    }

    #[test]
    fn horizontal_spacing_initial_width_is_zero() {
        let hs = HorizontalSpacing::new(4);
        assert_eq!(hs.width, 0.0);
    }

    #[test]
    fn slot_padding_start_is_zero() {
        assert_eq!(Slot::PaddingStart as usize, 0);
    }

    #[test]
    fn slot_note_spacing_is_last() {
        assert_eq!(Slot::NoteSpacing as usize, Slot::count() - 1);
    }

    #[test]
    fn horizontal_spacing_new_length_zero_still_allocates() {
        // length=0 → (0+1)*11 = 11 spans
        let hs = HorizontalSpacing::new(0);
        assert_eq!(hs.widths.len(), 11);
    }

    #[test]
    fn horizontal_spacing_set_width_at_tick_zero() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(0, Slot::PaddingStart, 5.0);
        let span = hs.get(0, Slot::PaddingStart);
        assert!((span.width - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_spacing_multiple_slots_at_same_tick() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(1, Slot::Clef, 1.5);
        hs.set_width(1, Slot::TimeSignature, 2.5);
        assert!((hs.get(1, Slot::Clef).width - 1.5).abs() < f32::EPSILON);
        assert!((hs.get(1, Slot::TimeSignature).width - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_spacing_span_new_x_and_width_are_floats() {
        let span = HorizontalSpacingSpan::new();
        let _ = span.x + span.width; // just ensure they are f32 compatible
        assert_eq!(span.x, 0.0_f32);
    }

    #[test]
    fn horizontal_spacing_set_overwrite_updates_value() {
        let mut hs = HorizontalSpacing::new(4);
        hs.set_width(0, Slot::KeySignature, 1.0);
        hs.set_width(0, Slot::KeySignature, 4.0);
        let span = hs.get(0, Slot::KeySignature);
        assert!((span.width - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn percussion_clef_yields_zero_key_signature_width_in_flow_pass() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::engraves::Engraves;
        use crate::score::entries::clef::{Clef, ClefDrawType};
        use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::entries::Entry;
        use crate::score::instruments::defs::InstrumentType;
        use crate::score::instruments::Instrument;
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;
        use crate::score::tracks::{Track, Tracks};
        use rustc_hash::{FxHashMap, FxHashSet};

        let mut flow = Flow::new(1, 100);
        flow.player_ids = FxHashSet::default();
        flow.length = 128;
        flow.staves.by_id.insert(
            10,
            Stave {
                id: 10,
                lines: vec![1, 1, 1, 1, 1],
                master_id: 200,
                track_ids: vec![],
            },
        );

        let instrument = Instrument {
            id: 20,
            definition_id: String::from("unpitched-percussion.snare"),
            instrument_type: InstrumentType::Percussive,
            long_name: String::from("Snare"),
            short_name: String::from("Sn."),
            stave_ids: vec![10],
            count: None,
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        };

        let mut flow_master = Track::new(100);
        flow_master.insert_entry(Entry::TimeSignature(TimeSignature::new(
            1,
            0,
            4,
            crate::score::components::duration::NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));
        flow_master.insert_entry(Entry::KeySignature(KeySignature::new(
            400,
            64,
            KeySignatureMode::Major,
            -5,
        )));

        let mut stave_master = Track::new(200);
        stave_master.insert_entry(Entry::Clef(Clef::new(
            300,
            0,
            Pitch::new(60, Accidental::Natural),
            0,
            ClefDrawType::Percussion,
        )));

        let mut tracks = Tracks::new();
        tracks.by_id = FxHashMap::default();
        tracks.by_id.insert(100, flow_master);
        tracks.by_id.insert(200, stave_master);

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();

        let spacing = get_horizontal_spacing(
            &flow,
            &vec![&instrument],
            &tracks,
            &get_measures(&flow, &tracks),
            &NotationByTrack::default(),
            &Barlines::default(),
            &ShuntsByTrack::default(),
            &BeamsByTrack::default(),
            &AccidentalsByTrack::default(),
            engrave,
        );

        assert_eq!(spacing.get(64, Slot::KeySignature).width, 0.0);
    }

    #[test]
    fn cancelling_key_signature_reserves_width_from_previous_signature() {
        use crate::score::components::pitch::{Accidental, Pitch};
        use crate::score::engraves::Engraves;
        use crate::score::entries::clef::{Clef, ClefDrawType};
        use crate::score::entries::key_signature::{KeySignature, KeySignatureMode};
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::entries::Entry;
        use crate::score::instruments::defs::InstrumentType;
        use crate::score::instruments::Instrument;
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;
        use crate::score::tracks::{Track, Tracks};
        use rustc_hash::{FxHashMap, FxHashSet};

        let mut flow = Flow::new(1, 100);
        flow.player_ids = FxHashSet::default();
        flow.length = 128;
        flow.staves.by_id.insert(
            10,
            Stave {
                id: 10,
                lines: vec![1, 1, 1, 1, 1],
                master_id: 200,
                track_ids: vec![],
            },
        );

        let instrument = Instrument {
            id: 20,
            definition_id: String::from("strings.violin"),
            instrument_type: InstrumentType::Melodic,
            long_name: String::from("Violin"),
            short_name: String::from("Vln."),
            stave_ids: vec![10],
            count: None,
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        };

        let mut flow_master = Track::new(100);
        flow_master.insert_entry(Entry::TimeSignature(TimeSignature::new(
            1,
            0,
            4,
            crate::score::components::duration::NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        )));
        flow_master.insert_entry(Entry::KeySignature(KeySignature::new(
            401,
            32,
            KeySignatureMode::Major,
            3,
        )));
        flow_master.insert_entry(Entry::KeySignature(KeySignature::new(
            402,
            64,
            KeySignatureMode::Major,
            0,
        )));

        let mut stave_master = Track::new(200);
        stave_master.insert_entry(Entry::Clef(Clef::new(
            300,
            0,
            Pitch::new(67, Accidental::Natural),
            2,
            ClefDrawType::G,
        )));

        let mut tracks = Tracks::new();
        tracks.by_id = FxHashMap::default();
        tracks.by_id.insert(100, flow_master);
        tracks.by_id.insert(200, stave_master);

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();

        let spacing = get_horizontal_spacing(
            &flow,
            &vec![&instrument],
            &tracks,
            &get_measures(&flow, &tracks),
            &NotationByTrack::default(),
            &Barlines::default(),
            &ShuntsByTrack::default(),
            &BeamsByTrack::default(),
            &AccidentalsByTrack::default(),
            engrave,
        );

        assert!(spacing.get(64, Slot::KeySignature).width > 0.0);
    }

    #[test]
    fn empty_measure_width_matches_between_three_four_and_six_eight() {
        use crate::score::components::duration::NoteDuration;
        use crate::score::engraves::Engraves;
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::layouts::LayoutType;

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
        let three_four = TimeSignature::new(
            1,
            0,
            3,
            NoteDuration::Quarter,
            TimeSignatureDrawType::Regular,
            None,
        );
        let six_eight = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );

        assert!((empty_measure_width(&three_four, engrave) - empty_measure_width(&six_eight, engrave)).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_measure_width_nine_eight_exceeds_six_eight() {
        use crate::score::components::duration::NoteDuration;
        use crate::score::engraves::Engraves;
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::layouts::LayoutType;

        let engraves = Engraves::new();
        let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
        let six_eight = TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );
        let nine_eight = TimeSignature::new(
            1,
            0,
            9,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        );

        assert!(empty_measure_width(&nine_eight, engrave) > empty_measure_width(&six_eight, engrave));
    }

    #[test]
    fn horizontal_spacing_empty_nine_eight_bar_is_wider_than_empty_six_eight_bar() {
        use crate::score::components::duration::NoteDuration;
        use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
        use crate::score::entries::Entry;
        use crate::score::flows::Flow;
        use crate::score::instruments::defs::InstrumentType;
        use crate::score::instruments::Instrument;
        use crate::score::layouts::LayoutType;
        use crate::score::staves::Stave;
        use crate::score::tracks::{Track, Tracks};
        use rustc_hash::{FxHashMap, FxHashSet};

        fn make_empty_spacing(time_signature: TimeSignature) -> HorizontalSpacing {
            let mut flow = Flow::new(1, 100);
            flow.player_ids = FxHashSet::default();
            flow.length = time_signature.ticks_per_bar();
            flow.staves.by_id.insert(
                10,
                Stave {
                    id: 10,
                    lines: vec![1, 1, 1, 1, 1],
                    master_id: 200,
                    track_ids: vec![],
                },
            );

            let instrument = Instrument {
                id: 20,
                definition_id: String::from("strings.violin"),
                instrument_type: InstrumentType::Melodic,
                long_name: String::from("Violin"),
                short_name: String::from("Vln."),
                stave_ids: vec![10],
                count: None,
                assigned_plugin_id: None,
                volume: 100,
                pan: 64,
                mute: false,
                solo: false,
            };

            let mut flow_master = Track::new(100);
            flow_master.insert_entry(Entry::TimeSignature(time_signature));

            let mut tracks = Tracks::new();
            tracks.by_id = FxHashMap::default();
            tracks.by_id.insert(100, flow_master);
            tracks.by_id.insert(200, Track::new(200));

            let engraves = crate::score::engraves::Engraves::new();
            let engrave = engraves.by_layout_type.get(&LayoutType::FullScore).unwrap();
            let measures = get_measures(&flow, &tracks);

            get_horizontal_spacing(
                &flow,
                &vec![&instrument],
                &tracks,
                &measures,
                &NotationByTrack::default(),
                &Barlines::default(),
                &ShuntsByTrack::default(),
                &BeamsByTrack::default(),
                &AccidentalsByTrack::default(),
                engrave,
            )
        }

        let six_eight_spacing = make_empty_spacing(TimeSignature::new(
            1,
            0,
            6,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        ));
        let nine_eight_spacing = make_empty_spacing(TimeSignature::new(
            1,
            0,
            9,
            NoteDuration::Eighth,
            TimeSignatureDrawType::Regular,
            None,
        ));

        assert!(nine_eight_spacing.width > six_eight_spacing.width);
    }
}
