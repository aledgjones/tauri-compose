use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::renderer::converter::Space;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use skia_safe::Color;

pub const STAVE_LINE_WIDTH: Space = 0.125;

pub fn draw_staves(
    origin: &Point,
    width: Space,
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    vertical_spacing: &VerticalSpacing,
    renderer: &mut RendererContext,
) {
    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave = &flow.staves.by_id[stave_id];
            let stave_spacing = &vertical_spacing.staves[stave_id];

            for (i, line) in stave.lines.iter().enumerate() {
                if line == &1 {
                    let top =
                        origin.y + stave_spacing.y - (stave_spacing.height / 2.0) + i as Space;

                    renderer.line(
                        STAVE_LINE_WIDTH,
                        Color::BLACK,
                        vec![
                            Point {
                                x: origin.x,
                                y: top,
                            },
                            Point {
                                x: origin.x + width,
                                y: top,
                            },
                        ],
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_vertical_spacing::{VerticalPosition, VerticalSpacing};
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::instruments::defs::InstrumentType;
    use crate::score::instruments::PlaybackAssignmentIntent;
    use crate::score::staves::Stave;
    use crate::utils::UniqueId;
    use rustc_hash::FxHashMap;

    fn make_instrument_with_stave(instrument_id: UniqueId, stave_id: UniqueId) -> Instrument {
        Instrument {
            id: instrument_id,
            definition_id: String::from("strings.violin"),
            instrument_type: InstrumentType::Melodic,
            long_name: String::from("Violin"),
            short_name: String::from("Vln."),
            stave_ids: vec![stave_id],
            count: None,
            assignment_intent: PlaybackAssignmentIntent::default(),
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    fn make_flow_with_stave(stave_id: UniqueId) -> Flow {
        let mut flow = Flow::new(1, 0);
        flow.staves.by_id.insert(
            stave_id,
            Stave {
                id: stave_id,
                lines: vec![1, 1, 1, 1, 1], // standard 5-line stave
                master_id: 0,
                track_ids: vec![],
            },
        );
        flow
    }

    fn make_spacing(stave_id: UniqueId) -> VerticalSpacing {
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
            instruments: FxHashMap::default(),
            staves,
        }
    }

    #[test]
    fn five_line_stave_produces_five_line_instructions() {
        let mut renderer = make_test_renderer();
        let stave_id = 10u64;
        let instr = make_instrument_with_stave(1, stave_id);
        let flow = make_flow_with_stave(stave_id);
        let spacing = make_spacing(stave_id);

        draw_staves(
            &Point { x: 0.0, y: 0.0 },
            20.0,
            &flow,
            &vec![&instr],
            &spacing,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 5);
        for instr in &renderer.instructions {
            assert!(matches!(instr, DrawInstruction::Line { .. }));
        }
    }

    #[test]
    fn hidden_lines_produce_no_instructions() {
        let mut renderer = make_test_renderer();
        let stave_id = 11u64;
        let instr = make_instrument_with_stave(1, stave_id);
        let mut flow = Flow::new(1, 0);
        // lines with value 0 = hidden
        flow.staves.by_id.insert(
            stave_id,
            Stave {
                id: stave_id,
                lines: vec![0, 1, 0, 1, 0],
                master_id: 0,
                track_ids: vec![],
            },
        );
        let spacing = make_spacing(stave_id);

        draw_staves(
            &Point { x: 0.0, y: 0.0 },
            20.0,
            &flow,
            &vec![&instr],
            &spacing,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 2);
    }

    #[test]
    fn no_instruments_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let flow = Flow::new(1, 0);
        let spacing = VerticalSpacing {
            height: 0.0,
            instruments: FxHashMap::default(),
            staves: FxHashMap::default(),
        };

        draw_staves(
            &Point { x: 0.0, y: 0.0 },
            20.0,
            &flow,
            &vec![],
            &spacing,
            &mut renderer,
        );

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn stave_lines_span_full_width() {
        let mut renderer = make_test_renderer();
        let stave_id = 12u64;
        let instr = make_instrument_with_stave(1, stave_id);
        let flow = make_flow_with_stave(stave_id);
        let spacing = make_spacing(stave_id);
        let width = 30.0;

        draw_staves(
            &Point { x: 5.0, y: 0.0 },
            width,
            &flow,
            &vec![&instr],
            &spacing,
            &mut renderer,
        );

        if let DrawInstruction::Line { points, .. } = &renderer.instructions[0] {
            assert!((points[0].0 - 5.0).abs() < 1e-5); // starts at origin.x
            assert!((points[1].0 - (5.0 + width)).abs() < 1e-5); // ends at origin.x + width
        } else {
            panic!("expected Line instruction");
        }
    }
}
