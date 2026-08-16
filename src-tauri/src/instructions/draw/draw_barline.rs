use super::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::instructions::parse::get_vertical_spans::VerticalSpans;
use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::measurements::Point;
use crate::score::entries::barline::BarlineDrawType;
use crate::ui::sequencer::Selection;
use crate::utils::UniqueId;
use skia_safe::Color;

#[derive(Clone, Copy)]
pub struct BarlineInteraction {
    pub flow_id: UniqueId,
    pub tick: u64,
}

pub fn draw_barline(
    origin: &Point,
    draw_type: &BarlineDrawType,
    vertical_spacing: &VerticalSpacing,
    vertical_spans: &VerticalSpans,
    interaction: Option<BarlineInteraction>,
    selection: Option<&Selection>,
    renderer: &mut RendererContext,
) {
    let tweak_for_stave_line_width = STAVE_LINE_WIDTH / 2.0;
    let color = if matches!(
        (interaction, selection),
        (
            Some(BarlineInteraction { flow_id, tick }),
            Some(Selection::Barline {
                flow_id: selected_flow_id,
                tick: selected_tick,
            }),
        ) if flow_id == *selected_flow_id && tick == *selected_tick
    ) {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };

    for span in &vertical_spans.barlines {
        let start = &vertical_spacing.instruments[&span.start];
        let stop = &vertical_spacing.instruments[&span.stop];

        let top = origin.y + start.y - tweak_for_stave_line_width;
        let bottom = origin.y + stop.y + stop.height + tweak_for_stave_line_width;

        match draw_type {
            BarlineDrawType::Single => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Double => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x + 0.5,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.5,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Triple => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x + 0.5,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.5,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Dashed => {
                renderer.dashed_line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Final => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    color,
                    vec![
                        Point {
                            x: origin.x + 0.75,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.75,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::StartRepeat => {
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    color,
                    vec![
                        Point {
                            x: origin.x + 0.25,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.25,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::EndRepeat => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    color,
                    vec![
                        Point {
                            x: origin.x + 1.75,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.75,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::EndStartRepeat => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    color,
                    vec![
                        Point {
                            x: origin.x + 1.75,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.75,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    color,
                    vec![
                        Point {
                            x: origin.x + 2.5,
                            y: top,
                        },
                        Point {
                            x: origin.x + 2.5,
                            y: bottom,
                        },
                    ],
                );
            }
        }
    }

    match draw_type {
        BarlineDrawType::EndRepeat => {
            for position in vertical_spacing.staves.values() {
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y + 0.5,
                    },
                );
            }
        }
        BarlineDrawType::EndStartRepeat => {
            for position in vertical_spacing.staves.values() {
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y + 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 3.25,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 3.25,
                        y: origin.y + position.y + 0.5,
                    },
                );
            }
        }
        BarlineDrawType::StartRepeat => {
            for position in vertical_spacing.staves.values() {
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 1.75,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    color,
                    &Point {
                        x: origin.x + 1.75,
                        y: origin.y + position.y + 0.5,
                    },
                );
            }
        }
        _ => (),
    }

    if let (Some(interaction), Some(first_span), Some(last_span)) = (
        interaction,
        vertical_spans.barlines.first(),
        vertical_spans.barlines.last(),
    ) {
        let top = origin.y + vertical_spacing.instruments[&first_span.start].y
            - tweak_for_stave_line_width;
        let last = &vertical_spacing.instruments[&last_span.stop];
        let bottom = origin.y + last.y + last.height + tweak_for_stave_line_width;
        let metrics = draw_type.metrics();

        renderer.hit_targets.push(HitTarget {
            entry: Selection::Barline {
                flow_id: interaction.flow_id,
                tick: interaction.tick,
            },
            x: origin.x,
            y: top,
            width: metrics.width.max(1.2),
            height: bottom - top,
            popover_x: origin.x,
            popover_y: top - 3.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{draw_barline, BarlineInteraction};
    use crate::instructions::draw::test_helpers::{make_barline_spans, make_vertical_spacing};
    use crate::renderer::{test_helpers::make_test_renderer, DrawInstruction};
    use crate::score::components::measurements::Point;
    use crate::score::entries::barline::BarlineDrawType;

    const INST_ID: u64 = 1;
    const STAVE_ID: u64 = 2;
    const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    fn count(renderer: &crate::renderer::RendererContext) -> (usize, usize, usize) {
        let lines = renderer
            .instructions
            .iter()
            .filter(|i| matches!(i, DrawInstruction::Line { .. }))
            .count();
        let dashed = renderer
            .instructions
            .iter()
            .filter(|i| matches!(i, DrawInstruction::DashedLine { .. }))
            .count();
        let circles = renderer
            .instructions
            .iter()
            .filter(|i| matches!(i, DrawInstruction::Circle { .. }))
            .count();
        (lines, dashed, circles)
    }

    #[test]
    fn single_produces_one_line() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::Single,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        assert_eq!(count(&r), (1, 0, 0));
    }

    #[test]
    fn double_produces_two_lines() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::Double,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        assert_eq!(count(&r), (2, 0, 0));
    }

    #[test]
    fn triple_produces_three_lines() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::Triple,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        assert_eq!(count(&r), (3, 0, 0));
    }

    #[test]
    fn dashed_produces_one_dashed_line() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::Dashed,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        assert_eq!(count(&r), (0, 1, 0));
    }

    #[test]
    fn final_produces_two_lines() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::Final,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        assert_eq!(count(&r), (2, 0, 0));
    }

    #[test]
    fn start_repeat_produces_two_lines_and_two_circles_per_stave() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::StartRepeat,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        // 2 lines (thick + thin) + 2 circles from 1 stave
        assert_eq!(count(&r), (2, 0, 2));
    }

    #[test]
    fn end_repeat_produces_two_lines_and_two_circles_per_stave() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::EndRepeat,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        // 2 lines (thin + thick) + 2 circles from 1 stave
        assert_eq!(count(&r), (2, 0, 2));
    }

    #[test]
    fn end_start_repeat_produces_three_lines_and_four_circles_per_stave() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();
        draw_barline(
            &ORIGIN,
            &BarlineDrawType::EndStartRepeat,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );
        // 3 lines + 4 circles from 1 stave
        assert_eq!(count(&r), (3, 0, 4));
    }

    #[test]
    fn interaction_adds_hit_target_with_barline_tick() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();

        draw_barline(
            &ORIGIN,
            &BarlineDrawType::Single,
            &vs,
            &spans,
            Some(BarlineInteraction {
                flow_id: 10,
                tick: 128,
            }),
            None,
            &mut r,
        );

        assert_eq!(r.hit_targets.len(), 1);
        assert_eq!(
            r.hit_targets[0].entry,
            crate::ui::sequencer::Selection::Barline {
                flow_id: 10,
                tick: 128,
            }
        );
        assert!(r.hit_targets[0].width >= 1.2);
        assert!(r.hit_targets[0].height > 0.0);
    }

    #[test]
    fn no_interaction_adds_no_hit_target() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let mut r = make_test_renderer();

        draw_barline(
            &ORIGIN,
            &BarlineDrawType::Final,
            &vs,
            &spans,
            None,
            None,
            &mut r,
        );

        assert!(r.hit_targets.is_empty());
    }

    #[test]
    fn selected_split_repeat_highlights_every_component() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_barline_spans(INST_ID, INST_ID);
        let interaction = BarlineInteraction {
            flow_id: 10,
            tick: 128,
        };
        let selection = crate::ui::sequencer::Selection::Barline {
            flow_id: 10,
            tick: 128,
        };
        let mut r = make_test_renderer();

        draw_barline(
            &ORIGIN,
            &BarlineDrawType::EndStartRepeat,
            &vs,
            &spans,
            Some(interaction),
            Some(&selection),
            &mut r,
        );

        let selected_color = skia_safe::Color::from_rgb(14, 165, 233);
        assert!(r.instructions.iter().all(|instruction| match instruction {
            DrawInstruction::Line { color, .. } | DrawInstruction::Circle { color, .. } => {
                *color == selected_color
            }
            _ => true,
        }));
    }
}
