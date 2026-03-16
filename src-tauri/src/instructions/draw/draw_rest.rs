use crate::instructions::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::instructions::parse::get_written_durations::Notation;
use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::duration::{NoteDuration, Tick};
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::ui::sequencer::Selection;
use crate::utils::UniqueId;
use skia_safe::Color;

#[allow(clippy::too_many_arguments)]
pub fn draw_rest(
    origin: &Point,
    tick: Tick,
    glyph: &str,
    entry: &Notation,
    is_full_bar: bool,
    horizontal_spacing: &HorizontalSpacing,
    selection: Option<&Selection>,
    track_id: UniqueId,
    stave_id: UniqueId,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_top: f32,
    renderer: &mut RendererContext,
) {
    let color = if matches!(selection, Some(Selection::Rest { tick: selected_tick, track_id: selected_track_id, .. }) if *selected_tick == tick && *selected_track_id == track_id)
    {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };

    let start = horizontal_spacing.get(tick, Slot::NoteSpacing);

    if is_full_bar {
        let end = horizontal_spacing.get(tick + entry.duration - 1, Slot::NoteSpacing);
        let left = (origin.x + start.x + ((end.x - start.x) / 2.0)) - 1.0;
        let top = origin.y - 1.0;

        renderer.text(
            "\u{E4E3}",
            &Font {
                family: FontFamily::Bravura,
                size: 4.0,
                justify: Justify::Start,
                align: Align::Bottom,
            },
            &Point { x: left, y: top },
            color,
        );

        renderer.hit_targets.push(HitTarget {
            entry: Selection::Rest {
                flow_id,
                instrument_id,
                stave_id,
                track_id,
                tick,
                tuplet_id: None,
            },
            x: left,
            y: top - 0.25,
            width: 1.1,
            height: 1.0,
            popover_x: origin.x + start.x,
            popover_y: stave_top - 3.0,
        });
    } else {
        let left = origin.x + start.x;
        let base = entry.base_to_note_duration();

        let top = match base {
            Some(NoteDuration::Whole) => origin.y - 1.0,
            _ => origin.y,
        };

        renderer.text(
            glyph,
            &Font {
                family: FontFamily::Bravura,
                size: 4.0,
                justify: Justify::Start,
                align: Align::Bottom,
            },
            &Point { x: left, y: top },
            color,
        );

        let hit_height = match base {
            Some(NoteDuration::Whole) => 1.0,
            Some(NoteDuration::Half) => 1.0,
            Some(NoteDuration::Quarter) => 3.0,
            Some(NoteDuration::Eighth) => 2.0,
            Some(NoteDuration::Sixteenth) => 3.0,
            Some(NoteDuration::ThirtySecond) => 4.0,
            Some(NoteDuration::SixtyFourth) => 5.0,
            Some(NoteDuration::HudredTwentyEighth) => 6.0,
            _ => 1.0,
        };

        let hit_width = match base {
            Some(NoteDuration::Whole) => 1.1,
            Some(NoteDuration::Half) => 1.1,
            Some(NoteDuration::Quarter) => 1.0,
            Some(NoteDuration::Eighth) => 1.0,
            Some(NoteDuration::Sixteenth) => 1.2,
            Some(NoteDuration::ThirtySecond) => 1.5,
            Some(NoteDuration::SixtyFourth) => 1.7,
            Some(NoteDuration::HudredTwentyEighth) => 1.9,
            _ => 1.0,
        };

        let hit_top_offset = match base {
            Some(NoteDuration::Whole) => 0.25,
            Some(NoteDuration::Half) => 0.75,
            Some(NoteDuration::Quarter) => 1.5,
            Some(NoteDuration::Eighth) => 1.0,
            Some(NoteDuration::Sixteenth) => 1.0,
            Some(NoteDuration::ThirtySecond) => 2.0,
            Some(NoteDuration::SixtyFourth) => 2.5,
            Some(NoteDuration::HudredTwentyEighth) => 3.0,
            _ => 1.0,
        };

        renderer.hit_targets.push(HitTarget {
            entry: Selection::Rest {
                flow_id,
                instrument_id,
                stave_id,
                track_id,
                tick,
                tuplet_id: None,
            },
            x: left,
            y: top - hit_top_offset,
            width: hit_width,
            height: hit_height,
            popover_x: left,
            popover_y: stave_top - 3.0,
        });

        if entry.is_dotted() {
            let metrics = entry.glyph_metrics();
            let glyph_width = metrics.total_width();

            renderer.circle(
                0.2,
                color,
                &Point {
                    x: left + glyph_width + 0.5,
                    y: top - 0.5,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_horizontal_spacing::HorizontalSpacing;
    use crate::instructions::parse::get_written_durations::Notation;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::ui::sequencer::Selection;

    fn make_notation(duration: u64) -> Notation {
        Notation::new(0, duration)
    }

    fn make_spacing(duration: u64) -> HorizontalSpacing {
        // Must be large enough for get(tick + duration - 1, ...) to not panic
        HorizontalSpacing::new(duration)
    }

    #[test]
    fn non_full_bar_rest_produces_text_and_hit_target() {
        let mut renderer = make_test_renderer();
        let entry = make_notation(32);
        let spacing = make_spacing(32);
        draw_rest(
            &Point { x: 2.0, y: 5.0 },
            0,
            "\u{E4E5}", // quarter rest glyph
            &entry,
            false,
            &spacing,
            None,
            1,
            1,
            1,
            1,
            5.0,
            &mut renderer,
        );
        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
        assert_eq!(renderer.hit_targets.len(), 1);
    }

    #[test]
    fn full_bar_rest_produces_text_and_hit_target() {
        let mut renderer = make_test_renderer();
        let entry = make_notation(32);
        let spacing = make_spacing(32);
        draw_rest(
            &Point { x: 2.0, y: 5.0 },
            0,
            "\u{E4E3}", // whole-bar rest glyph
            &entry,
            true,
            &spacing,
            None,
            1,
            1,
            1,
            1,
            5.0,
            &mut renderer,
        );
        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
        assert_eq!(renderer.hit_targets.len(), 1);
    }

    #[test]
    fn selected_rest_uses_blue_color() {
        let mut renderer = make_test_renderer();
        let entry = make_notation(32);
        let spacing = make_spacing(32);
        let selection = Selection::Rest {
            tick: 0,
            track_id: 42,
            stave_id: 1,
            flow_id: 1,
            instrument_id: 1,
            tuplet_id: None,
        };
        draw_rest(
            &Point { x: 0.0, y: 0.0 },
            0,
            "\u{E4E5}",
            &entry,
            false,
            &spacing,
            Some(&selection),
            42,
            1,
            1,
            1,
            0.0,
            &mut renderer,
        );
        if let DrawInstruction::Text { color, .. } = &renderer.instructions[0] {
            assert_eq!(*color, skia_safe::Color::from_rgb(14, 165, 233));
        } else {
            panic!("expected Text");
        }
    }

    #[test]
    fn unselected_rest_uses_black_color() {
        let mut renderer = make_test_renderer();
        let entry = make_notation(32);
        let spacing = make_spacing(32);
        draw_rest(
            &Point { x: 0.0, y: 0.0 },
            0,
            "\u{E4E5}",
            &entry,
            false,
            &spacing,
            None,
            1,
            1,
            1,
            1,
            0.0,
            &mut renderer,
        );
        if let DrawInstruction::Text { color, .. } = &renderer.instructions[0] {
            assert_eq!(*color, skia_safe::Color::BLACK);
        } else {
            panic!("expected Text");
        }
    }

    #[test]
    fn sixty_fourth_rest_produces_expanded_hit_target() {
        let mut renderer = make_test_renderer();
        let entry = make_notation(NoteDuration::SixtyFourth.to_ticks());
        let spacing = make_spacing(NoteDuration::SixtyFourth.to_ticks());

        draw_rest(
            &Point { x: 0.0, y: 0.0 },
            0,
            "\u{E4E9}",
            &entry,
            false,
            &spacing,
            None,
            1,
            1,
            1,
            1,
            0.0,
            &mut renderer,
        );

        assert_eq!(renderer.hit_targets.len(), 1);
        assert!(renderer.hit_targets[0].height >= 5.0);
        assert!(renderer.hit_targets[0].width >= 1.7);
    }

    #[test]
    fn hundred_twenty_eighth_rest_produces_expanded_hit_target() {
        let mut renderer = make_test_renderer();
        let entry = make_notation(NoteDuration::HudredTwentyEighth.to_ticks());
        let spacing = make_spacing(NoteDuration::HudredTwentyEighth.to_ticks());

        draw_rest(
            &Point { x: 0.0, y: 0.0 },
            0,
            "\u{E4EA}",
            &entry,
            false,
            &spacing,
            None,
            1,
            1,
            1,
            1,
            0.0,
            &mut renderer,
        );

        assert_eq!(renderer.hit_targets.len(), 1);
        assert!(renderer.hit_targets[0].height >= 6.0);
        assert!(renderer.hit_targets[0].width >= 1.9);
    }
}
