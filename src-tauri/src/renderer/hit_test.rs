use crate::ui::sequencer::Selection;
use crate::RendererContextState;
use tauri::State;

fn hit_test_selection(
    renderer: &crate::renderer::RendererContext,
    x: f32,
    y: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> Option<Selection> {
    let zoom_scale = zoom / 100.0;

    let render_x = (x - pan_x) / zoom_scale;
    let render_y = (y - pan_y) / zoom_scale;

    let render_x_spaces = renderer.converter.px_to_spaces(render_x);
    let render_y_spaces = renderer.converter.px_to_spaces(render_y);

    for target in &renderer.hit_targets {
        if render_x_spaces >= target.x
            && render_x_spaces <= target.x + target.width
            && render_y_spaces >= target.y
            && render_y_spaces <= target.y + target.height
        {
            return Some(target.entry.clone());
        }
    }

    None
}

#[tauri::command(rename_all = "snake_case")]
pub fn renderer_hit_test(
    renderer_state: State<'_, RendererContextState>,
    x: f32,
    y: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> Option<Selection> {
    let renderer = renderer_state.lock().unwrap();

    hit_test_selection(&renderer, x, y, pan_x, pan_y, zoom)
}

#[cfg(test)]
mod tests {
    use super::hit_test_selection;
    use crate::renderer::{test_helpers::make_test_renderer, HitTarget};
    use crate::ui::sequencer::Selection;

    #[test]
    fn returns_tempo_selection_when_click_is_inside_tempo_hit_target() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tempo {
                flow_id: 4,
                tempo_id: 8,
                tick: 64,
            },
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 2.0,
            popover_x: 1.0,
            popover_y: 1.0,
        });

        let hit = hit_test_selection(&renderer, 2.5, 3.0, 0.0, 0.0, 100.0);

        assert_eq!(
            hit,
            Some(Selection::Tempo {
                flow_id: 4,
                tempo_id: 8,
                tick: 64,
            })
        );
    }

    #[test]
    fn returns_none_when_click_is_outside_hit_targets() {
        let renderer = make_test_renderer();

        let hit = hit_test_selection(&renderer, 10.0, 15.0, 0.0, 0.0, 100.0);

        assert!(hit.is_none());
    }

    #[test]
    fn returns_new_directive_selection_variants_when_click_is_inside_hit_target() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::TimeSignature {
                flow_id: 4,
                time_signature_id: 8,
                tick: 64,
            },
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 2.0,
            popover_x: 1.0,
            popover_y: 1.0,
        });
        renderer.hit_targets.push(HitTarget {
            entry: Selection::KeySignature {
                flow_id: 4,
                key_signature_id: 9,
                tick: 96,
            },
            x: 5.0,
            y: 2.0,
            width: 3.0,
            height: 2.0,
            popover_x: 5.0,
            popover_y: 1.0,
        });
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Clef {
                flow_id: 4,
                stave_id: 3,
                clef_id: 10,
                tick: 128,
            },
            x: 9.0,
            y: 2.0,
            width: 3.0,
            height: 2.0,
            popover_x: 9.0,
            popover_y: 1.0,
        });

        assert_eq!(
            hit_test_selection(&renderer, 2.5, 3.0, 0.0, 0.0, 100.0),
            Some(Selection::TimeSignature {
                flow_id: 4,
                time_signature_id: 8,
                tick: 64,
            })
        );
        assert_eq!(
            hit_test_selection(&renderer, 6.5, 3.0, 0.0, 0.0, 100.0),
            Some(Selection::KeySignature {
                flow_id: 4,
                key_signature_id: 9,
                tick: 96,
            })
        );
        assert_eq!(
            hit_test_selection(&renderer, 10.5, 3.0, 0.0, 0.0, 100.0),
            Some(Selection::Clef {
                flow_id: 4,
                stave_id: 3,
                clef_id: 10,
                tick: 128,
            })
        );
    }
}
