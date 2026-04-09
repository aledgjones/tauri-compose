use crate::ui::sequencer::Selection;
use crate::RendererContextState;
use std::cmp::Ordering;
use tauri::State;

fn contains_point(target: &crate::renderer::HitTarget, x: f32, y: f32) -> bool {
    x >= target.x && x <= target.x + target.width && y >= target.y && y <= target.y + target.height
}

fn compare_hit_candidates(
    point_x: f32,
    point_y: f32,
    candidate_index: usize,
    candidate: &crate::renderer::HitTarget,
    best_index: usize,
    best: &crate::renderer::HitTarget,
) -> Ordering {
    let candidate_center_x = candidate.x + (candidate.width / 2.0);
    let candidate_center_y = candidate.y + (candidate.height / 2.0);
    let best_center_x = best.x + (best.width / 2.0);
    let best_center_y = best.y + (best.height / 2.0);

    let candidate_distance =
        (point_x - candidate_center_x).powi(2) + (point_y - candidate_center_y).powi(2);
    let best_distance = (point_x - best_center_x).powi(2) + (point_y - best_center_y).powi(2);

    candidate_distance
        .total_cmp(&best_distance)
        .then_with(|| {
            let candidate_area = candidate.width * candidate.height;
            let best_area = best.width * best.height;
            candidate_area.total_cmp(&best_area)
        })
        .then_with(|| candidate_index.cmp(&best_index))
}

fn hit_test_selection(
    renderer: &crate::renderer::RendererContext,
    x: f32,
    y: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> Option<Selection> {
    let zoom_scale = zoom / 100.0;

    if zoom_scale <= 0.0 {
        return None;
    }

    let render_x = (x - pan_x) / zoom_scale;
    let render_y = (y - pan_y) / zoom_scale;

    let render_x_spaces = renderer.converter.px_to_spaces(render_x);
    let render_y_spaces = renderer.converter.px_to_spaces(render_y);

    renderer
        .hit_targets
        .iter()
        .enumerate()
        .filter(|(_, target)| contains_point(target, render_x_spaces, render_y_spaces))
        .min_by(|(candidate_index, candidate), (best_index, best)| {
            compare_hit_candidates(
                render_x_spaces,
                render_y_spaces,
                *candidate_index,
                candidate,
                *best_index,
                best,
            )
        })
        .map(|(_, target)| target.entry.clone())
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

    #[test]
    fn returns_playing_technique_selection_when_click_is_inside_hit_target() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::PlayingTechnique {
                flow_id: 4,
                stave_id: 3,
                playing_technique_id: 10,
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
            hit_test_selection(&renderer, 10.5, 3.0, 0.0, 0.0, 100.0),
            Some(Selection::PlayingTechnique {
                flow_id: 4,
                stave_id: 3,
                playing_technique_id: 10,
                tick: 128,
            })
        );
    }

    #[test]
    fn returns_tuplet_selection_when_click_is_inside_tuplet_hit_target() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tuplet {
                flow_id: 4,
                instrument_id: 2,
                stave_id: 3,
                track_id: 5,
                tuplet_id: 11,
                tick: 64,
            },
            x: 6.0,
            y: 1.0,
            width: 4.0,
            height: 2.0,
            popover_x: 8.0,
            popover_y: 0.0,
        });

        assert_eq!(
            hit_test_selection(&renderer, 8.0, 2.0, 0.0, 0.0, 100.0),
            Some(Selection::Tuplet {
                flow_id: 4,
                instrument_id: 2,
                stave_id: 3,
                track_id: 5,
                tuplet_id: 11,
                tick: 64,
            })
        );
    }

    #[test]
    fn resolves_offset_second_flow_hit_targets_in_shared_renderer_space() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tempo {
                flow_id: 20,
                tempo_id: 8,
                tick: 64,
            },
            x: 25.0,
            y: 2.0,
            width: 3.0,
            height: 2.0,
            popover_x: 25.0,
            popover_y: 1.0,
        });

        let hit = hit_test_selection(&renderer, 26.5, 3.0, 0.0, 0.0, 100.0);

        assert_eq!(
            hit,
            Some(Selection::Tempo {
                flow_id: 20,
                tempo_id: 8,
                tick: 64,
            })
        );
    }

    #[test]
    fn returns_none_when_click_lands_in_the_gap_between_visible_flows() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tempo {
                flow_id: 10,
                tempo_id: 1,
                tick: 0,
            },
            x: 23.0,
            y: 1.0,
            width: 1.5,
            height: 2.0,
            popover_x: 23.0,
            popover_y: 1.0,
        });
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tempo {
                flow_id: 20,
                tempo_id: 2,
                tick: 32,
            },
            x: 25.25,
            y: 1.0,
            width: 1.5,
            height: 2.0,
            popover_x: 25.25,
            popover_y: 1.0,
        });

        let hit = hit_test_selection(&renderer, 24.875, 2.0, 0.0, 0.0, 100.0);

        assert_eq!(hit, None);
    }

    #[test]
    fn resolves_overlapping_boundary_hits_to_the_visually_closest_flow() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tempo {
                flow_id: 10,
                tempo_id: 1,
                tick: 0,
            },
            x: 23.0,
            y: 1.0,
            width: 3.0,
            height: 2.0,
            popover_x: 23.0,
            popover_y: 1.0,
        });
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tempo {
                flow_id: 20,
                tempo_id: 2,
                tick: 32,
            },
            x: 24.5,
            y: 1.0,
            width: 3.0,
            height: 2.0,
            popover_x: 24.5,
            popover_y: 1.0,
        });

        let hit = hit_test_selection(&renderer, 25.75, 2.0, 0.0, 0.0, 100.0);

        assert_eq!(
            hit,
            Some(Selection::Tempo {
                flow_id: 20,
                tempo_id: 2,
                tick: 32,
            })
        );
    }

    #[test]
    fn repeated_hits_at_the_same_location_return_the_same_flow_aware_target() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Clef {
                flow_id: 20,
                stave_id: 6,
                clef_id: 10,
                tick: 96,
            },
            x: 48.0,
            y: 2.0,
            width: 3.0,
            height: 2.5,
            popover_x: 48.0,
            popover_y: 1.0,
        });
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Clef {
                flow_id: 30,
                stave_id: 7,
                clef_id: 11,
                tick: 96,
            },
            x: 49.25,
            y: 2.0,
            width: 2.0,
            height: 2.5,
            popover_x: 49.25,
            popover_y: 1.0,
        });

        for _ in 0..5 {
            assert_eq!(
                hit_test_selection(&renderer, 50.0, 3.0, 0.0, 0.0, 100.0),
                Some(Selection::Clef {
                    flow_id: 30,
                    stave_id: 7,
                    clef_id: 11,
                    tick: 96,
                })
            );
        }
    }

    #[test]
    fn applies_pan_and_zoom_before_resolving_shared_surface_targets() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::KeySignature {
                flow_id: 30,
                key_signature_id: 14,
                tick: 128,
            },
            x: 49.25,
            y: 1.5,
            width: 1.75,
            height: 2.0,
            popover_x: 49.25,
            popover_y: 0.75,
        });

        let hit = hit_test_selection(&renderer, 110.0, 1.0, 10.0, -4.0, 200.0);

        assert_eq!(
            hit,
            Some(Selection::KeySignature {
                flow_id: 30,
                key_signature_id: 14,
                tick: 128,
            })
        );
    }

    #[test]
    fn resolves_hits_for_later_visible_flows_after_shared_surface_offsets() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::KeySignature {
                flow_id: 30,
                key_signature_id: 14,
                tick: 128,
            },
            x: 49.25,
            y: 1.5,
            width: 1.75,
            height: 2.0,
            popover_x: 49.25,
            popover_y: 0.75,
        });

        let hit = hit_test_selection(&renderer, 50.0, 2.5, 0.0, 0.0, 100.0);

        assert_eq!(
            hit,
            Some(Selection::KeySignature {
                flow_id: 30,
                key_signature_id: 14,
                tick: 128,
            })
        );
    }

    #[test]
    fn preserves_existing_single_flow_hit_testing_behavior_through_the_shared_path() {
        let mut renderer = make_test_renderer();
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tone {
                flow_id: 4,
                track_id: 9,
                stave_id: 2,
                instrument_id: 3,
                tone_id: 15,
                tuplet_id: None,
            },
            x: 8.0,
            y: 4.0,
            width: 1.5,
            height: 1.5,
            popover_x: 8.0,
            popover_y: 4.0,
        });

        let hit = hit_test_selection(&renderer, 8.75, 4.75, 0.0, 0.0, 100.0);

        assert_eq!(
            hit,
            Some(Selection::Tone {
                flow_id: 4,
                track_id: 9,
                stave_id: 2,
                instrument_id: 3,
                tone_id: 15,
                tuplet_id: None,
            })
        );
    }
}
