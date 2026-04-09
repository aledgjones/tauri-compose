use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::measurements::Point;
use crate::score::entries::playing_technique::PlayingTechnique;
use crate::ui::sequencer::Selection;
use crate::utils::UniqueId;
use skia_safe::Color;

pub const PLAYING_TECHNIQUE_STAVE_OFFSET: f32 = 4.0;

#[allow(clippy::too_many_arguments)]
pub fn draw_playing_technique(
    origin: &Point,
    playing_technique: &PlayingTechnique,
    text_measurer: &TextMeasurer<'_>,
    selection: Option<&Selection>,
    flow_id: UniqueId,
    stave_id: UniqueId,
    stave_top: f32,
    renderer: &mut RendererContext,
) {
    if playing_technique.hidden {
        return;
    }

    let color = if matches!(selection, Some(Selection::PlayingTechnique { flow_id: selected_flow_id, stave_id: selected_stave_id, playing_technique_id, .. }) if *selected_flow_id == flow_id && *selected_stave_id == stave_id && *playing_technique_id == playing_technique.id)
    {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };
    let metrics = playing_technique.metrics(text_measurer);

    if metrics.width <= 0.0 {
        return;
    }

    renderer.text(
        playing_technique.display_text(),
        &PlayingTechnique::text_font(),
        origin,
        color,
    );

    renderer.hit_targets.push(HitTarget {
        entry: Selection::PlayingTechnique {
            flow_id,
            stave_id,
            playing_technique_id: playing_technique.id,
            tick: playing_technique.tick,
        },
        x: origin.x,
        y: origin.y - metrics.height + (metrics.height * 0.2),
        width: metrics.width.max(1.2),
        height: metrics.height.max(1.0),
        popover_x: origin.x,
        popover_y: stave_top - 3.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{
        test_helpers::make_test_renderer, text_measurer::TextMeasurer, DrawInstruction,
    };
    use crate::score::entries::playing_technique::PlayingTechniqueKind;

    fn make_playing_technique(id: UniqueId, kind: PlayingTechniqueKind) -> PlayingTechnique {
        PlayingTechnique::new(id, 64, kind)
    }

    #[test]
    fn hidden_playing_technique_draws_nothing() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let mut playing_technique = make_playing_technique(10, PlayingTechniqueKind::Pizz);
        playing_technique.hidden = true;

        draw_playing_technique(
            &Point { x: 0.0, y: 0.0 },
            &playing_technique,
            &text_measurer,
            None,
            1,
            2,
            5.0,
            &mut renderer,
        );

        assert!(renderer.instructions.is_empty());
        assert!(renderer.hit_targets.is_empty());
    }

    #[test]
    fn visible_playing_technique_draws_text_and_hit_target() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let playing_technique = make_playing_technique(10, PlayingTechniqueKind::Pizz);

        draw_playing_technique(
            &Point { x: 1.0, y: 2.0 },
            &playing_technique,
            &text_measurer,
            None,
            1,
            2,
            5.0,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        match &renderer.instructions[0] {
            DrawInstruction::Text { text, .. } => assert_eq!(text, "pizz"),
            _ => panic!("expected text instruction"),
        }
        assert_eq!(renderer.hit_targets.len(), 1);
        assert_eq!(
            renderer.hit_targets[0].entry,
            Selection::PlayingTechnique {
                flow_id: 1,
                stave_id: 2,
                playing_technique_id: 10,
                tick: 64,
            }
        );
    }

    #[test]
    fn selected_playing_technique_uses_selection_color() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let playing_technique = make_playing_technique(10, PlayingTechniqueKind::Arco);

        draw_playing_technique(
            &Point { x: 0.0, y: 0.0 },
            &playing_technique,
            &text_measurer,
            Some(&Selection::PlayingTechnique {
                flow_id: 1,
                stave_id: 2,
                playing_technique_id: 10,
                tick: 64,
            }),
            1,
            2,
            5.0,
            &mut renderer,
        );

        match &renderer.instructions[0] {
            DrawInstruction::Text { color, .. } => {
                assert_eq!(*color, Color::from_rgb(14, 165, 233));
            }
            _ => panic!("expected text instruction"),
        }
    }
}
