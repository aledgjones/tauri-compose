use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::clef::Clef;
use crate::utils::UniqueId;
use skia_safe::Color;

pub struct ClefDrawContext {
    pub flow_id: UniqueId,
    pub stave_id: UniqueId,
    pub stave_top: f32,
    pub is_selected: bool,
}

pub fn draw_clef(
    origin: &Point,
    clef: &Clef,
    is_full_size: bool,
    context: &ClefDrawContext,
    renderer: &mut RendererContext,
) {
    if let Some(glyph) = clef.glyph() {
        let metrics = clef.metrics(is_full_size);
        let y = origin.y + (0.5 * clef.offset as f32);
        let color = if context.is_selected {
            Color::from_rgb(14, 165, 233)
        } else {
            Color::BLACK
        };

        renderer.text(
            &glyph,
            &Font {
                size: if is_full_size { 4.0 } else { 2.5 },
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Start,
            },
            &Point {
                x: origin.x,
                y,
            },
            color,
        );

        renderer.hit_targets.push(HitTarget {
            entry: crate::ui::sequencer::Selection::Clef {
                flow_id: context.flow_id,
                stave_id: context.stave_id,
                clef_id: clef.id,
                tick: clef.tick,
            },
            x: origin.x,
            y: y - metrics.height,
            width: metrics.width.max(1.0),
            height: metrics.height.max(1.0),
            popover_x: origin.x,
            popover_y: context.stave_top - 3.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::entries::clef::ClefDrawType;

    #[test]
    fn draw_clef_treble_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let clef = Clef {
            draw_as: ClefDrawType::G,
            ..Default::default()
        };
        let origin = Point { x: 2.0, y: 10.0 };

        draw_clef(
            &origin,
            &clef,
            true,
            &ClefDrawContext {
                flow_id: 1,
                stave_id: 2,
                stave_top: 10.0,
                is_selected: false,
            },
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
    }

    #[test]
    fn draw_clef_full_size_uses_font_size_4() {
        let mut renderer = make_test_renderer();
        let clef = Clef {
            draw_as: ClefDrawType::G,
            ..Default::default()
        };

        draw_clef(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            true,
            &ClefDrawContext {
                flow_id: 1,
                stave_id: 2,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer,
        );

        if let DrawInstruction::Text { font_size, .. } = &renderer.instructions[0] {
            // With Converter(1.0,1.0): 4.0 spaces == 4.0 px
            assert!((font_size - 4.0).abs() < 1e-5);
        } else {
            panic!("expected DrawInstruction::Text");
        }
    }

    #[test]
    fn draw_clef_small_size_uses_font_size_2_5() {
        let mut renderer = make_test_renderer();
        let clef = Clef {
            draw_as: ClefDrawType::G,
            ..Default::default()
        };

        draw_clef(
            &Point { x: 0.0, y: 0.0 },
            &clef,
            false,
            &ClefDrawContext {
                flow_id: 1,
                stave_id: 2,
                stave_top: 0.0,
                is_selected: false,
            },
            &mut renderer,
        );

        if let DrawInstruction::Text { font_size, .. } = &renderer.instructions[0] {
            assert!((font_size - 2.5).abs() < 1e-5);
        } else {
            panic!("expected DrawInstruction::Text");
        }
    }

    #[test]
    fn draw_clef_offset_shifts_y_position() {
        let mut renderer = make_test_renderer();
        let clef = Clef {
            draw_as: ClefDrawType::G,
            offset: 2,
            ..Default::default()
        }; // 0.5 * 2 = 1.0 space shift
        let origin = Point { x: 0.0, y: 5.0 };

        draw_clef(
            &origin,
            &clef,
            true,
            &ClefDrawContext {
                flow_id: 1,
                stave_id: 2,
                stave_top: 5.0,
                is_selected: false,
            },
            &mut renderer,
        );

        if let DrawInstruction::Text { point, .. } = &renderer.instructions[0] {
            // y = origin.y + (0.5 * offset) = 5.0 + 1.0 = 6.0
            assert!((point.1 - 6.0).abs() < 1e-5);
        } else {
            panic!("expected DrawInstruction::Text");
        }
    }

    #[test]
    fn selected_clef_uses_highlight_and_emits_hit_target() {
        let mut renderer = make_test_renderer();
        let clef = Clef {
            id: 9,
            tick: 64,
            draw_as: ClefDrawType::G,
            ..Default::default()
        };

        draw_clef(
            &Point { x: 1.0, y: 5.0 },
            &clef,
            true,
            &ClefDrawContext {
                flow_id: 4,
                stave_id: 7,
                stave_top: 5.0,
                is_selected: true,
            },
            &mut renderer,
        );

        assert!(!renderer.hit_targets.is_empty());
        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { color, .. } if *color == Color::from_rgb(14, 165, 233))
        }));
    }
}
