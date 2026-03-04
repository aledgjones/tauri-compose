use super::get_measures::Measures;
use crate::score::components::duration::Tick;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::flows::Flow;
use crate::score::tracks::Tracks;
use rustc_hash::FxHashMap;

/// context can mean barlines need to be split over positions
/// esspecially repeat marks, this def allows info about when this happens
#[derive(Default)]
pub struct BarlineDrawDef {
    /// seperate end repeat
    pub end_repeat: bool,
    /// seperate start repeat
    pub start_repeat: bool,
    /// whether the split repeat has a time sig or key sig between the barlines
    pub split_by_signature: bool,
    /// other draw type if needed (including a combined endstart repeat)
    pub draw_type: Option<BarlineDrawType>,
}

impl BarlineDrawDef {
    pub fn new(
        end_repeat: bool,
        start_repeat: bool,
        split_by_signature: bool,
        draw_type: Option<BarlineDrawType>,
    ) -> Self {
        Self {
            end_repeat,
            start_repeat,
            split_by_signature,
            draw_type,
        }
    }
}

pub type Barlines = FxHashMap<Tick, BarlineDrawDef>;

pub fn get_barlines(flow: &Flow, tracks: &Tracks, measures: &Measures) -> Barlines {
    let mut output = Barlines::default();

    let master = &tracks.by_id[&flow.master_id];

    for tick in 0..flow.length {
        let key_signature = master.get_key_signature_at_tick(&tick);
        let barline = master.get_barline_at_tick(&tick);
        let time_signature = master.get_time_signature_at_tick(&tick);

        let mut def = BarlineDrawDef::default();

        match barline {
            Some(barline) => {
                // draw the user defined barline to veto default
                match barline.barline_draw_type {
                    BarlineDrawType::EndRepeat => {
                        def.end_repeat = true;
                    }
                    BarlineDrawType::EndStartRepeat => {
                        // repeat barlines are seperated if there is a clef, key or time sig :| K#3/4 |:
                        // else we can draw as one barline :||:
                        if key_signature.is_some()
                            || time_signature.is_some()
                            || flow.has_clef_at_tick(&tick, tracks)
                        {
                            def.start_repeat = true;
                            def.end_repeat = true;
                            def.split_by_signature =
                                key_signature.is_some() || time_signature.is_some();
                        } else {
                            def.draw_type = Some(BarlineDrawType::EndStartRepeat);
                        }
                    }
                    BarlineDrawType::StartRepeat => {
                        def.start_repeat = true;

                        if tick > 0 {
                            // time sig use an aditional single barline
                            if time_signature.is_some() {
                                def.draw_type = Some(BarlineDrawType::Single);
                            }

                            // key sig uses an aditional double barline
                            if key_signature.is_some() {
                                def.draw_type = Some(BarlineDrawType::Double);
                            }
                        }
                    }
                    _ => {
                        output.insert(
                            tick,
                            BarlineDrawDef::new(
                                false,
                                false,
                                false,
                                Some(barline.barline_draw_type),
                            ),
                        );
                    }
                }
            }
            None => {
                if tick > 0 {
                    if key_signature.is_some() {
                        // key signatures take double barlines
                        def.draw_type = Some(BarlineDrawType::Double);
                    } else if measures.contains_key(&tick) {
                        // this is just your standard barline
                        def.draw_type = Some(BarlineDrawType::Single);
                    }
                }
            }
        }

        if def.end_repeat || def.start_repeat || def.draw_type.is_some() {
            output.insert(tick, def);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::flows::Flow;
    use crate::score::tracks::{Track, Tracks};
    use rustc_hash::FxHashMap;

    #[test]
    fn barline_draw_def_new_stores_all_fields() {
        let def = BarlineDrawDef::new(true, false, true, Some(BarlineDrawType::Single));
        assert!(def.end_repeat);
        assert!(!def.start_repeat);
        assert!(def.split_by_signature);
        assert!(def.draw_type.is_some());
    }

    #[test]
    fn barline_draw_def_default_is_all_false() {
        let def = BarlineDrawDef::default();
        assert!(!def.end_repeat);
        assert!(!def.start_repeat);
        assert!(!def.split_by_signature);
        assert!(def.draw_type.is_none());
    }

    #[test]
    fn get_barlines_with_empty_flow_returns_empty() {
        let mut flow = Flow::new(1, 100);
        flow.length = 0;
        let mut tracks = Tracks::new();
        tracks.by_id.insert(100, Track::new(100));
        let measures: Measures = FxHashMap::default();
        let result = get_barlines(&flow, &tracks, &measures);
        assert!(result.is_empty());
    }

    #[test]
    fn get_barlines_with_no_entries_and_no_measures_returns_empty() {
        let mut flow = Flow::new(1, 100);
        flow.length = 128;
        let mut tracks = Tracks::new();
        tracks.by_id.insert(100, Track::new(100));
        // empty measures and no barline entries
        let measures: Measures = FxHashMap::default();
        let result = get_barlines(&flow, &tracks, &measures);
        assert!(result.is_empty());
    }
}
