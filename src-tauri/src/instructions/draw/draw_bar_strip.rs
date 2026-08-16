use crate::instructions::parse::get_barlines::Barlines;
use crate::instructions::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::instructions::parse::get_measures::Measures;
use crate::renderer::converter::Converter;
use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::duration::{Tick, Ticks};
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::barline::BarlineDrawType;
use crate::ui::sequencer::Selection;
use crate::utils::UniqueId;
use skia_safe::Color;

const BAR_SEPARATOR: f32 = 0.24;
const CONTROL_SIZE: f32 = 2.4;
const ICON_LINE_WIDTH: f32 = 0.16;
const BAR_VERTICAL_PADDING_PX: f32 = 1.0;
const DELETE_LEFT_PADDING_PX: f32 = 2.0;
const ADD_RIGHT_PADDING_PX: f32 = BAR_VERTICAL_PADDING_PX;
const NUMBER_LEFT_PADDING_PX: f32 = 6.0;
const MIN_STAVE_CLEARANCE: f32 = 4.0;
const MIN_NOTE_CLEARANCE: f32 = 2.0;
const NOTEHEAD_HALF_HEIGHT: f32 = 0.5;

pub fn bar_strip_height(converter: &Converter) -> f32 {
    CONTROL_SIZE + converter.px_to_spaces(BAR_VERTICAL_PADDING_PX * 2.0)
}

pub fn bar_strip_top_reserve(
    stave_center: f32,
    stave_height: f32,
    highest_tone_offset: Option<i8>,
    converter: &Converter,
) -> f32 {
    let top_stave_line = stave_center - (stave_height / 2.0);
    let stave_limit = top_stave_line - MIN_STAVE_CLEARANCE;
    let bottom_limit = highest_tone_offset
        .map(|offset| {
            stave_center + (f32::from(offset) / 2.0) - NOTEHEAD_HALF_HEIGHT - MIN_NOTE_CLEARANCE
        })
        .map_or(stave_limit, |note_limit| stave_limit.min(note_limit));

    (bar_strip_height(converter) - bottom_limit).max(0.0)
}

fn bar_boundary_x(
    tick: Tick,
    flow_length: Ticks,
    barlines: &Barlines,
    horizontal_spacing: &HorizontalSpacing,
) -> f32 {
    if tick == 0 {
        0.0
    } else if tick == flow_length {
        horizontal_spacing.width
    } else {
        let Some(def) = barlines.get(&tick) else {
            return horizontal_spacing.get(tick, Slot::Barline).x;
        };
        if let Some(draw_type) = def.draw_type {
            horizontal_spacing.get(tick, Slot::Barline).x + (draw_type.metrics().width / 2.0)
        } else if def.end_repeat {
            horizontal_spacing.get(tick, Slot::EndRepeat).x
                + (BarlineDrawType::EndRepeat.metrics().width / 2.0)
        } else if def.start_repeat {
            horizontal_spacing.get(tick, Slot::StartRepeat).x
                + (BarlineDrawType::StartRepeat.metrics().width / 2.0)
        } else {
            horizontal_spacing.get(tick, Slot::Barline).x
        }
    }
}

fn push_hit_target(
    renderer: &mut RendererContext,
    entry: Selection,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    renderer.hit_targets.push(HitTarget {
        entry,
        x,
        y,
        width,
        height,
        popover_x: x + (width / 2.0),
        popover_y: y,
    });
}

fn draw_add_icon(renderer: &mut RendererContext, x: f32, y: f32) {
    let center_x = x + (CONTROL_SIZE / 2.0);
    let center_y = y + (CONTROL_SIZE / 2.0);
    let radius = CONTROL_SIZE * 0.25;
    renderer.line(
        ICON_LINE_WIDTH,
        Color::WHITE,
        vec![
            Point {
                x: center_x - radius,
                y: center_y,
            },
            Point {
                x: center_x + radius,
                y: center_y,
            },
        ],
    );
    renderer.line(
        ICON_LINE_WIDTH,
        Color::WHITE,
        vec![
            Point {
                x: center_x,
                y: center_y - radius,
            },
            Point {
                x: center_x,
                y: center_y + radius,
            },
        ],
    );
}

fn draw_delete_icon(renderer: &mut RendererContext, x: f32, y: f32) {
    let color = Color::from_rgb(14, 165, 233);
    renderer.rectangle(
        x + (CONTROL_SIZE * 0.32),
        y + (CONTROL_SIZE * 0.36),
        CONTROL_SIZE * 0.36,
        CONTROL_SIZE * 0.38,
        color,
    );
    renderer.rectangle(
        x + (CONTROL_SIZE * 0.25),
        y + (CONTROL_SIZE * 0.27),
        CONTROL_SIZE * 0.5,
        CONTROL_SIZE * 0.09,
        color,
    );
    renderer.rectangle(
        x + (CONTROL_SIZE * 0.42),
        y + (CONTROL_SIZE * 0.2),
        CONTROL_SIZE * 0.16,
        CONTROL_SIZE * 0.07,
        color,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn draw_bar_strip(
    origin: &Point,
    flow_id: UniqueId,
    flow_length: Ticks,
    measures: &Measures,
    barlines: &Barlines,
    horizontal_spacing: &HorizontalSpacing,
    selection: Option<&Selection>,
    renderer: &mut RendererContext,
) {
    let mut starts = measures.keys().copied().collect::<Vec<_>>();
    starts.sort_unstable();
    let font = Font {
        size: 1.35,
        family: FontFamily::SystemSans,
        justify: Justify::Start,
        align: Align::Middle,
    };
    let strip_height = bar_strip_height(&renderer.converter);
    let number_left_padding = renderer.converter.px_to_spaces(NUMBER_LEFT_PADDING_PX);
    let delete_left_padding = renderer.converter.px_to_spaces(DELETE_LEFT_PADDING_PX);
    let add_right_padding = renderer.converter.px_to_spaces(ADD_RIGHT_PADDING_PX);

    for (index, start) in starts.iter().copied().enumerate() {
        let end = starts.get(index + 1).copied().unwrap_or(flow_length);
        let left = origin.x + bar_boundary_x(start, flow_length, barlines, horizontal_spacing);
        let right = origin.x + bar_boundary_x(end, flow_length, barlines, horizontal_spacing);
        let width = (right - left).max(0.1);
        let selected = matches!(
            selection,
            Some(Selection::Bar { flow_id: selected_flow_id, tick })
                if *selected_flow_id == flow_id && *tick == start
        );
        let background = if selected {
            Color::from_rgb(186, 230, 253)
        } else {
            Color::from_rgb(241, 245, 249)
        };
        let box_left = left + (BAR_SEPARATOR / 2.0);
        let box_right = if end == flow_length {
            right
        } else {
            right - (BAR_SEPARATOR / 2.0)
        };
        let box_width = (box_right - box_left).max(0.1);
        renderer.rectangle(box_left, origin.y, box_width, strip_height, background);
        if !selected {
            renderer.text(
                &(index + 1).to_string(),
                &font,
                &Point {
                    x: box_left + number_left_padding,
                    y: origin.y + (strip_height / 2.0),
                },
                Color::from_rgb(150, 150, 150),
            );
        }
        push_hit_target(
            renderer,
            Selection::Bar {
                flow_id,
                tick: start,
            },
            left,
            origin.y,
            width,
            strip_height,
        );

        if selected {
            let controls_top = origin.y + ((strip_height - CONTROL_SIZE) / 2.0);
            let delete_x = box_left + delete_left_padding;
            let add_x = box_left + box_width - add_right_padding - CONTROL_SIZE;

            draw_delete_icon(renderer, delete_x, controls_top);
            push_hit_target(
                renderer,
                Selection::DeleteBar {
                    flow_id,
                    tick: start,
                },
                delete_x,
                controls_top,
                CONTROL_SIZE,
                CONTROL_SIZE,
            );

            renderer.rectangle(
                add_x,
                controls_top,
                CONTROL_SIZE,
                CONTROL_SIZE,
                Color::from_rgb(14, 165, 233),
            );
            draw_add_icon(renderer, add_x, controls_top);
            push_hit_target(
                renderer,
                Selection::InsertBarAfter {
                    flow_id,
                    tick: start,
                },
                add_x,
                controls_top,
                CONTROL_SIZE,
                CONTROL_SIZE,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_barlines::BarlineDrawDef;
    use crate::instructions::parse::get_horizontal_spacing::HorizontalSpacingSpan;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::entries::time_signature::TimeSignature;

    fn spacing() -> HorizontalSpacing {
        let mut spacing = HorizontalSpacing::new(256);
        spacing.width = 30.0;
        let index = (128 * Slot::count()) + Slot::Barline as usize;
        spacing.widths[index] = HorizontalSpacingSpan {
            x: 12.0,
            width: 0.2,
        };
        spacing
    }

    fn measures() -> Measures {
        let mut measures = Measures::default();
        measures.insert(0, TimeSignature::default());
        measures.insert(128, TimeSignature::default());
        measures
    }

    fn barlines() -> Barlines {
        let mut barlines = Barlines::default();
        barlines.insert(
            128,
            BarlineDrawDef::new(false, false, false, Some(BarlineDrawType::Single)),
        );
        barlines
    }

    #[test]
    fn internal_boundary_uses_the_visual_middle_of_a_wide_barline() {
        let mut spacing = spacing();
        let mut barlines = barlines();
        barlines.insert(
            128,
            BarlineDrawDef::new(false, false, false, Some(BarlineDrawType::Double)),
        );
        let index = (128 * Slot::count()) + Slot::Barline as usize;
        spacing.widths[index] = HorizontalSpacingSpan {
            x: 12.0,
            width: 1.0,
        };

        assert_eq!(bar_boundary_x(128, 256, &barlines, &spacing), 12.25);
    }

    #[test]
    fn top_reserve_keeps_four_spaces_above_an_empty_stave() {
        let renderer = make_test_renderer();
        let height = bar_strip_height(&renderer.converter);

        assert_eq!(
            bar_strip_top_reserve(2.0, 4.0, None, &renderer.converter),
            height + 4.0
        );
    }

    #[test]
    fn top_reserve_moves_above_a_note_higher_than_stave_clearance() {
        let renderer = make_test_renderer();
        let height = bar_strip_height(&renderer.converter);

        assert_eq!(
            bar_strip_top_reserve(2.0, 4.0, Some(-16), &renderer.converter),
            height + 8.5
        );
    }

    #[test]
    fn top_reserve_keeps_stave_clearance_when_notes_are_lower() {
        let renderer = make_test_renderer();
        let height = bar_strip_height(&renderer.converter);

        assert_eq!(
            bar_strip_top_reserve(2.0, 4.0, Some(2), &renderer.converter),
            height + 4.0
        );
    }

    #[test]
    fn draws_one_hit_target_for_each_bar_at_spacing_boundaries() {
        let mut renderer = make_test_renderer();
        draw_bar_strip(
            &Point { x: 5.0, y: 2.0 },
            7,
            256,
            &measures(),
            &barlines(),
            &spacing(),
            None,
            &mut renderer,
        );

        let bars = renderer
            .hit_targets
            .iter()
            .filter(|target| matches!(target.entry, Selection::Bar { .. }))
            .collect::<Vec<_>>();
        assert_eq!(bars.len(), 2);
        assert_eq!((bars[0].x, bars[0].width), (5.0, 12.0));
        assert_eq!(bars[1].x, 17.0);

        let rectangles = renderer
            .instructions
            .iter()
            .filter_map(|instruction| match instruction {
                DrawInstruction::Rectangle {
                    x,
                    width,
                    height,
                    color,
                    ..
                } => Some((*x, *width, *height, *color)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(rectangles.len(), 2);
        assert!(rectangles
            .iter()
            .all(|(_, _, _, color)| *color == Color::from_rgb(241, 245, 249)));
        assert!(rectangles[0].0 > renderer.converter.spaces_to_px(5.0));
        assert!(rectangles[0].1 < renderer.converter.spaces_to_px(12.0));
        assert_eq!(
            rectangles[0].2,
            renderer.converter.spaces_to_px(CONTROL_SIZE) + 2.0
        );
        assert!(
            (rectangles[1].0 + rectangles[1].1 - renderer.converter.spaces_to_px(35.0)).abs()
                < 0.001
        );

        assert!(renderer.instructions.iter().any(|instruction| matches!(
            instruction,
            DrawInstruction::Text {
                text,
                font_family: FontFamily::SystemSans,
                justify: Justify::Start,
                point,
                ..
            } if text == "1" && point.0 < renderer.converter.spaces_to_px(11.0)
        )));
    }

    #[test]
    fn selected_bar_adds_insert_and_delete_controls_only_for_that_bar() {
        let mut renderer = make_test_renderer();
        draw_bar_strip(
            &Point { x: 5.0, y: 2.0 },
            7,
            256,
            &measures(),
            &barlines(),
            &spacing(),
            Some(&Selection::Bar {
                flow_id: 7,
                tick: 128,
            }),
            &mut renderer,
        );

        assert_eq!(
            renderer
                .hit_targets
                .iter()
                .filter(|target| matches!(target.entry, Selection::InsertBarAfter { .. }))
                .count(),
            1
        );
        assert_eq!(
            renderer
                .hit_targets
                .iter()
                .filter(|target| matches!(target.entry, Selection::DeleteBar { .. }))
                .count(),
            1
        );
        let controls = renderer
            .hit_targets
            .iter()
            .filter(|target| {
                matches!(
                    target.entry,
                    Selection::InsertBarAfter { .. } | Selection::DeleteBar { .. }
                )
            })
            .collect::<Vec<_>>();
        assert!(controls
            .iter()
            .all(|target| (target.width - target.height).abs() < f32::EPSILON));
        let selected_bar = renderer
            .hit_targets
            .iter()
            .find(|target| {
                matches!(
                    target.entry,
                    Selection::Bar {
                        flow_id: 7,
                        tick: 128
                    }
                )
            })
            .unwrap();
        assert_eq!(
            renderer
                .converter
                .spaces_to_px(selected_bar.height - controls[0].height),
            2.0
        );
        let delete = controls
            .iter()
            .find(|target| matches!(target.entry, Selection::DeleteBar { .. }))
            .unwrap();
        let add = controls
            .iter()
            .find(|target| matches!(target.entry, Selection::InsertBarAfter { .. }))
            .unwrap();
        let selected_box_left = selected_bar.x + (BAR_SEPARATOR / 2.0);
        let selected_box_right = selected_bar.x + selected_bar.width;
        assert!(
            (renderer
                .converter
                .spaces_to_px(delete.x - selected_box_left)
                - DELETE_LEFT_PADDING_PX)
                .abs()
                < 0.001
        );
        assert!(
            (renderer
                .converter
                .spaces_to_px(selected_box_right - add.x - add.width)
                - ADD_RIGHT_PADDING_PX)
                .abs()
                < 0.001
        );
        assert!(!renderer.instructions.iter().any(|instruction| matches!(
            instruction,
            DrawInstruction::Text { text, .. } if text == "2"
        )));
        assert!(!renderer.instructions.iter().any(|instruction| matches!(
            instruction,
            DrawInstruction::Text { text, .. } if text == "+" || text == "x"
        )));
        assert_eq!(
            renderer
                .instructions
                .iter()
                .filter(|instruction| matches!(instruction, DrawInstruction::Line { .. }))
                .count(),
            2
        );
        assert_eq!(
            renderer
                .instructions
                .iter()
                .filter(|instruction| matches!(
                    instruction,
                    DrawInstruction::Rectangle {
                        width,
                        height,
                        color,
                        ..
                    } if (*width - renderer.converter.spaces_to_px(CONTROL_SIZE)).abs() < f32::EPSILON
                        && (*height - renderer.converter.spaces_to_px(CONTROL_SIZE)).abs() < f32::EPSILON
                        && *color == Color::from_rgb(14, 165, 233)
                ))
                .count(),
            1
        );
    }
}
