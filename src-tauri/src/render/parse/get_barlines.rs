use super::get_measures::Measures;
use crate::score::components::duration::Tick;
use crate::score::entries::barline::BarlineDrawType;
use crate::score::flows::Flow;
use crate::score::tracks::Tracks;
use rustc_hash::FxHashMap;

/// context can mean barlines need to be split over positions
/// esspecially repeat marks, this def allows info about when this happens
pub struct BarlineDrawDef {
    /// seperate end repeat
    pub end_repeat: bool,
    /// seperate start repeat
    pub start_repeat: bool,
    /// other draw type if needed (including a combined endstart repeat)
    pub draw_type: Option<BarlineDrawType>,
}

impl BarlineDrawDef {
    pub fn new(end_repeat: bool, start_repeat: bool, draw_type: Option<BarlineDrawType>) -> Self {
        Self {
            end_repeat,
            start_repeat,
            draw_type,
        }
    }
}

pub type Barlines = FxHashMap<Tick, BarlineDrawDef>;

pub fn get_barlines(flow: &Flow, tracks: &Tracks, measures: &Measures) -> Barlines {
    let mut output: Barlines = FxHashMap::default();

    let master = &tracks.by_id[&flow.master_id];

    for tick in 0..flow.length {
        let key_signature = master.get_key_signature_at_tick(&tick);
        let barline = master.get_barline_at_tick(&tick);
        let time_signature = master.get_time_signature_at_tick(&tick);

        let mut def = BarlineDrawDef::new(false, false, None);

        match barline {
            Some(barline) => {
                // draw the user defined barline to veto default
                match barline.barline_draw_type {
                    BarlineDrawType::EndRepeat => {
                        def.end_repeat = true;
                    }
                    BarlineDrawType::EndStartRepeat => {
                        // repeat barlines are seperated if there is a key or time sig :|#3/4|:
                        // else we can draw as one barline :||:
                        if key_signature.is_some() || time_signature.is_some() {
                            def.start_repeat = true;
                            def.end_repeat = true;
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
                            BarlineDrawDef::new(false, false, Some(barline.barline_draw_type)),
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
