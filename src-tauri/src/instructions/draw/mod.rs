pub mod draw_accidentals;
pub mod draw_articulations;
pub mod draw_barline;
pub mod draw_beam;
pub mod draw_braces;
pub mod draw_brackets;
pub mod draw_clef;
pub mod draw_dot;
pub mod draw_flag;
pub mod draw_key_signature;
pub mod draw_ledger_lines;
pub mod draw_names;
pub mod draw_notehead;
pub mod draw_rest;
pub mod draw_staves;
pub mod draw_stem;
pub mod draw_sub_brackets;
pub mod draw_systemic_barline;
pub mod draw_tempo;
pub mod draw_ties;
pub mod draw_time_signature;
#[cfg(test)]
pub mod test_helpers {
    use crate::instructions::parse::get_beams::Beam;
    use crate::instructions::parse::get_stem_directions::StemDirections;
    use crate::instructions::parse::get_stem_lengths::{StemDef, StemLengths};
    use crate::instructions::parse::get_vertical_spacing::{VerticalPosition, VerticalSpacing};
    use crate::instructions::parse::get_vertical_spans::{VerticalSpan, VerticalSpans};
    use crate::score::components::duration::{Direction, Tick};
    use crate::score::components::measurements::Point;
    use crate::utils::UniqueId;
    use rustc_hash::FxHashMap;

    /// Build a VerticalSpacing with one instrument (y=0, height=4) and one stave (y=2, height=4).
    pub fn make_vertical_spacing(instrument_id: UniqueId, stave_id: UniqueId) -> VerticalSpacing {
        let mut instruments = FxHashMap::default();
        instruments.insert(
            instrument_id,
            VerticalPosition {
                y: 0.0,
                height: 4.0,
            },
        );
        let mut staves = FxHashMap::default();
        staves.insert(
            stave_id,
            VerticalPosition {
                y: 2.0,
                height: 4.0,
            },
        );
        VerticalSpacing {
            height: 4.0,
            instruments,
            staves,
        }
    }

    /// Build a VerticalSpans with one barline span (start → stop).
    pub fn make_barline_spans(start: UniqueId, stop: UniqueId) -> VerticalSpans {
        VerticalSpans {
            brackets: Vec::new(),
            sub_brackets: Vec::new(),
            braces: Vec::new(),
            barlines: vec![VerticalSpan { start, stop }],
        }
    }

    /// Build a VerticalSpans with one bracket span (start → stop).
    pub fn make_bracket_spans(start: UniqueId, stop: UniqueId) -> VerticalSpans {
        VerticalSpans {
            brackets: vec![VerticalSpan { start, stop }],
            sub_brackets: Vec::new(),
            braces: Vec::new(),
            barlines: Vec::new(),
        }
    }

    /// Build a StemDef with the tail at (tail_x, tail_y) and head above it.
    pub fn make_stem_def(tail_x: f32, tail_y: f32) -> StemDef {
        StemDef {
            head: Point {
                x: tail_x,
                y: tail_y + 3.5,
            },
            tail: Point {
                x: tail_x,
                y: tail_y,
            },
        }
    }

    /// Build a simple 2-note beam with tails at x=0 and x=4, plus the
    /// matching StemLengths and StemDirections (both up).
    pub fn make_two_note_beam() -> (Beam, StemLengths, StemDirections) {
        let start_tick: Tick = 0;
        let stop_tick: Tick = 8;
        let mut ticks = FxHashMap::default();
        ticks.insert(start_tick, 1u8);
        ticks.insert(stop_tick, 1u8);
        let beam = Beam {
            ticks,
            start: start_tick,
            stop: stop_tick,
            beam_segments: FxHashMap::default(),
            fractional_beams: FxHashMap::default(),
        };
        let mut stem_lengths: StemLengths = FxHashMap::default();
        stem_lengths.insert(start_tick, make_stem_def(0.0, -3.5));
        stem_lengths.insert(stop_tick, make_stem_def(4.0, -3.5));
        let mut stem_directions: StemDirections = FxHashMap::default();
        stem_directions.insert(start_tick, Direction::Up);
        stem_directions.insert(stop_tick, Direction::Up);
        (beam, stem_lengths, stem_directions)
    }
}
