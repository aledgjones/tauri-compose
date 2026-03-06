use crate::instructions::parse::get_shunts::{Shunt, Shunts};
use crate::renderer::HitTarget;
use crate::renderer::RendererContext;
use crate::score::components::duration::Tick;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::tone::Tone;
use crate::ui::sequencer::Selection;
use crate::utils::UniqueId;
use skia_safe::Color;

#[allow(clippy::too_many_arguments)]
pub fn draw_notehead(
    origin: &Point,
    tick: Tick,
    tone: &Tone,
    glyph: &str,
    glyph_width: f32,
    shunts: &Shunts,
    selection: Option<&Selection>,
    track_id: UniqueId,
    stave_id: UniqueId,
    flow_id: UniqueId,
    instrument_id: UniqueId,
    stave_top: f32,
    renderer: &mut RendererContext,
) {
    let left = match &shunts.by_id[&(tick, tone.id)] {
        Shunt::Pre => origin.x - glyph_width,
        Shunt::None => origin.x,
        Shunt::Post => origin.x + glyph_width,
    };

    let color = if matches!(selection, Some(Selection::Tone { tone_id, track_id: selected_track_id, .. }) if *tone_id == tone.id && *selected_track_id == track_id)
    {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };

    renderer.text(
        glyph,
        &Font {
            family: FontFamily::Bravura,
            size: 4.0,
            justify: Justify::Start,
            align: Align::Bottom,
        },
        &Point {
            x: left,
            y: origin.y,
        },
        color,
    );

    renderer.hit_targets.push(HitTarget {
        entry: Selection::Tone {
            tone_id: tone.id,
            track_id,
            stave_id,
            flow_id,
            instrument_id,
        },
        x: left,
        y: origin.y - 0.5,
        width: glyph_width,
        height: 1.0,
        popover_x: left,
        popover_y: f32::min(stave_top - 3.0, origin.y - 1.5),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_shunts::{Shunt, Shunts};
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::components::articulation::Articulations;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tone::Tone;
    use rustc_hash::FxHashMap;

    fn make_tone(id: u64) -> Tone {
        Tone::new(
            id,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            Articulations::new(),
        )
    }

    fn make_shunts(tick: u64, tone_id: u64, shunt: Shunt) -> Shunts {
        let mut by_id = FxHashMap::default();
        by_id.insert((tick, tone_id), shunt);
        Shunts {
            by_id,
            by_offset: FxHashMap::default(),
        }
    }

    #[test]
    fn no_shunt_produces_one_text_and_one_hit_target() {
        let mut renderer = make_test_renderer();
        let tone = make_tone(1);
        let shunts = make_shunts(0, 1, Shunt::None);
        draw_notehead(
            &Point { x: 3.0, y: 5.0 },
            0,
            &tone,
            "\u{E0A4}",
            1.175,
            &shunts,
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
    fn no_shunt_x_equals_origin_x() {
        let mut renderer = make_test_renderer();
        let tone = make_tone(1);
        let shunts = make_shunts(0, 1, Shunt::None);
        let origin = Point { x: 3.0, y: 5.0 };
        draw_notehead(
            &origin,
            0,
            &tone,
            "\u{E0A4}",
            1.175,
            &shunts,
            None,
            1,
            1,
            1,
            1,
            5.0,
            &mut renderer,
        );
        if let DrawInstruction::Text { point, .. } = &renderer.instructions[0] {
            assert!((point.0 - origin.x).abs() < 1e-5);
        } else {
            panic!("expected Text");
        }
    }

    #[test]
    fn pre_shunt_shifts_x_left_by_glyph_width() {
        let mut renderer = make_test_renderer();
        let tone = make_tone(1);
        let shunts = make_shunts(0, 1, Shunt::Pre);
        let origin = Point { x: 3.0, y: 5.0 };
        let glyph_width = 1.175;
        draw_notehead(
            &origin,
            0,
            &tone,
            "\u{E0A4}",
            glyph_width,
            &shunts,
            None,
            1,
            1,
            1,
            1,
            5.0,
            &mut renderer,
        );
        if let DrawInstruction::Text { point, .. } = &renderer.instructions[0] {
            assert!((point.0 - (origin.x - glyph_width)).abs() < 1e-5);
        } else {
            panic!("expected Text");
        }
    }

    #[test]
    fn post_shunt_shifts_x_right_by_glyph_width() {
        let mut renderer = make_test_renderer();
        let tone = make_tone(1);
        let shunts = make_shunts(0, 1, Shunt::Post);
        let origin = Point { x: 3.0, y: 5.0 };
        let glyph_width = 1.175;
        draw_notehead(
            &origin,
            0,
            &tone,
            "\u{E0A4}",
            glyph_width,
            &shunts,
            None,
            1,
            1,
            1,
            1,
            5.0,
            &mut renderer,
        );
        if let DrawInstruction::Text { point, .. } = &renderer.instructions[0] {
            assert!((point.0 - (origin.x + glyph_width)).abs() < 1e-5);
        } else {
            panic!("expected Text");
        }
    }

    #[test]
    fn selected_tone_uses_blue_color() {
        let mut renderer = make_test_renderer();
        let tone = make_tone(1);
        let shunts = make_shunts(0, 1, Shunt::None);
        let selection = Selection::Tone {
            tone_id: 1,
            track_id: 42,
            stave_id: 1,
            flow_id: 1,
            instrument_id: 1,
        };
        draw_notehead(
            &Point { x: 0.0, y: 0.0 },
            0,
            &tone,
            "\u{E0A4}",
            1.175,
            &shunts,
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
    fn unselected_tone_uses_black_color() {
        let mut renderer = make_test_renderer();
        let tone = make_tone(1);
        let shunts = make_shunts(0, 1, Shunt::None);
        draw_notehead(
            &Point { x: 0.0, y: 0.0 },
            0,
            &tone,
            "\u{E0A4}",
            1.175,
            &shunts,
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
}
