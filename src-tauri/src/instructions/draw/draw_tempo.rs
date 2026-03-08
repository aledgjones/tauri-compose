use crate::renderer::text_measurer::TextMeasurer;
use crate::renderer::{HitTarget, RendererContext};
use crate::score::components::measurements::Point;
use crate::score::entries::tempo::{
    Tempo, TEMPO_DOT_GAP, TEMPO_METRONOME_EQUALS_GAP, TEMPO_TEXT_GAP,
};
use crate::ui::sequencer::Selection;
use crate::utils::UniqueId;
use skia_safe::Color;

pub fn draw_tempo(
    origin: &Point,
    tempo: &Tempo,
    text_measurer: &TextMeasurer<'_>,
    selection: Option<&Selection>,
    flow_id: UniqueId,
    stave_top: f32,
    renderer: &mut RendererContext,
) {
    if tempo.hidden {
        return;
    }

    let mut x = origin.x;
    let color = if matches!(selection, Some(Selection::Tempo { tempo_id, flow_id: selected_flow_id, .. }) if *tempo_id == tempo.id && *selected_flow_id == flow_id)
    {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };
    let metrics = tempo.metrics(text_measurer);

    if let Some(text) = tempo.display_text() {
        renderer.text(text, &Tempo::text_font(), &Point { x, y: origin.y }, color);
        x += text_measurer.measure(text, &Tempo::text_font()).width;

        if tempo.show_metronome {
            x += TEMPO_TEXT_GAP;
        }
    }

    if tempo.show_metronome {
        let prefix_text = tempo.metronome_prefix_text();
        if !prefix_text.is_empty() {
            renderer.text(
                prefix_text,
                &Tempo::bpm_font(),
                &Point { x, y: origin.y },
                color,
            );
            x += text_measurer.measure(prefix_text, &Tempo::bpm_font()).width;
        }

        let note_glyph = tempo.note_glyph();
        let note_width = text_measurer
            .measure(note_glyph, &Tempo::metronome_font())
            .width;
        renderer.text(
            note_glyph,
            &Tempo::metronome_font(),
            &Point { x, y: origin.y },
            color,
        );

        let mut metronome_tail = x + note_width;

        if tempo.dots > 0 {
            for index in 0..tempo.dots {
                renderer.circle(
                    0.12,
                    color,
                    &Point {
                        x: x + note_width + 0.3 + (index as f32 * TEMPO_DOT_GAP),
                        y: origin.y - 0.85,
                    },
                );
            }
            metronome_tail += (tempo.dots as f32 * TEMPO_DOT_GAP) + 0.15;
        }

        metronome_tail += TEMPO_METRONOME_EQUALS_GAP;
        let bpm_text = tempo.metronome_suffix_text();
        renderer.text(
            &bpm_text,
            &Tempo::bpm_font(),
            &Point {
                x: metronome_tail,
                y: origin.y,
            },
            color,
        );
    }

    if metrics.width > 0.0 {
        renderer.hit_targets.push(HitTarget {
            entry: Selection::Tempo {
                flow_id,
                tempo_id: tempo.id,
                tick: tempo.tick,
            },
            x: origin.x,
            y: origin.y - (metrics.height - 1.0),
            width: metrics.width.max(1.2),
            height: metrics.height.max(1.0),
            popover_x: origin.x,
            popover_y: stave_top - 3.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{
        test_helpers::make_test_renderer, text_measurer::TextMeasurer, DrawInstruction,
    };
    use crate::score::components::duration::NoteDuration;
    use crate::ui::sequencer::Selection;

    fn make_tempo(text: &str, show_text: bool, show_metronome: bool, parenthesized: bool) -> Tempo {
        Tempo::new(
            1,
            0,
            String::from(text),
            NoteDuration::Quarter,
            0,
            120,
            false,
            show_text,
            show_metronome,
            parenthesized,
        )
    }

    #[test]
    fn hidden_tempo_draws_nothing() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let tempo = Tempo::new(
            1,
            0,
            String::from("Allegro"),
            NoteDuration::Quarter,
            0,
            120,
            true,
            true,
            true,
            false,
        );

        draw_tempo(
            &Point { x: 0.0, y: 0.0 },
            &tempo,
            &text_measurer,
            None,
            1,
            0.0,
            &mut renderer,
        );

        assert!(renderer.instructions.is_empty());
        assert!(renderer.hit_targets.is_empty());
    }

    #[test]
    fn tempo_with_no_visible_content_emits_no_hit_target() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let tempo = make_tempo("", false, false, false);

        draw_tempo(
            &Point { x: 0.0, y: 0.0 },
            &tempo,
            &text_measurer,
            None,
            1,
            0.0,
            &mut renderer,
        );

        assert!(renderer.instructions.is_empty());
        assert!(renderer.hit_targets.is_empty());
    }

    #[test]
    fn text_only_tempo_draws_single_libre_text_instruction() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let tempo = make_tempo("Allegro", true, false, false);

        draw_tempo(
            &Point { x: 0.0, y: 0.0 },
            &tempo,
            &text_measurer,
            None,
            1,
            0.0,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        match &renderer.instructions[0] {
            DrawInstruction::Text {
                text, font_family, ..
            } => {
                assert_eq!(text, "Allegro");
                assert_eq!(
                    *font_family,
                    crate::score::components::text::FontFamily::LibreBaskerville
                );
            }
            _ => panic!("expected text instruction"),
        }
    }

    #[test]
    fn metronome_only_tempo_draws_bravura_and_bpm_instructions() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let tempo = make_tempo("", false, true, false);

        draw_tempo(
            &Point { x: 0.0, y: 0.0 },
            &tempo,
            &text_measurer,
            None,
            1,
            0.0,
            &mut renderer,
        );

        assert!(renderer.instructions.len() >= 2);
        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(
                instruction,
                DrawInstruction::Text {
                    font_family: crate::score::components::text::FontFamily::Bravura,
                    ..
                }
            )
        }));
        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text == "= 120")
        }));
        assert!(!renderer
            .instructions
            .iter()
            .any(|instruction| { matches!(instruction, DrawInstruction::Line { .. }) }));
    }

    #[test]
    fn parenthesized_tempo_draws_parentheses() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let tempo = make_tempo("Allegro", true, true, true);

        draw_tempo(
            &Point { x: 0.0, y: 0.0 },
            &tempo,
            &text_measurer,
            None,
            1,
            0.0,
            &mut renderer,
        );

        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text.contains('('))
        }));
        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text == "= 120)")
        }));
    }

    #[test]
    fn selected_tempo_uses_blue_highlight_and_emits_hit_target() {
        let mut renderer = make_test_renderer();
        let converter = renderer.converter.clone();
        let text_measurer = TextMeasurer::new(&converter);
        let tempo = make_tempo("Allegro", true, true, false);
        let selection = Selection::Tempo {
            flow_id: 8,
            tempo_id: 1,
            tick: 0,
        };

        draw_tempo(
            &Point { x: 2.0, y: 4.0 },
            &tempo,
            &text_measurer,
            Some(&selection),
            8,
            6.0,
            &mut renderer,
        );

        assert!(!renderer.hit_targets.is_empty());
        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { color, .. } if *color == skia_safe::Color::from_rgb(14, 165, 233))
        }));
    }
}
