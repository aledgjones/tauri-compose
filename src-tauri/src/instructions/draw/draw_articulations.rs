use crate::instructions::parse::get_stem_lengths::StemDef;
use crate::instructions::parse::get_written_durations::Notation;
use crate::renderer::RendererContext;
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::Direction;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use rustc_hash::FxHashSet;
use skia_safe::Color;

const TUPLET_ARTICULATION_CLEARANCE_STACCATO: f32 = 0.25;
const TUPLET_ARTICULATION_CLEARANCE_TENUTO: f32 = 0.5;
const TUPLET_ARTICULATION_CLEARANCE_STACCATISSIMO: f32 = 1.25;
const TUPLET_ARTICULATION_CLEARANCE_STACKED_POINT: f32 = 0.25;
const TUPLET_ARTICULATION_CLEARANCE_STACCATO_TENUTO: f32 = 1.0;
const TUPLET_ARTICULATION_CLEARANCE_ACCENT: f32 = 1.25;
const TUPLET_ARTICULATION_CLEARANCE_MARCATO: f32 = 1.0;

pub(crate) fn tuplet_bracket_clearance(articulations: &FxHashSet<Articulation>) -> f32 {
    if articulations.is_empty() {
        return 0.0;
    }

    let stacked_point_clearance = if articulations.len() > 1 {
        articulations
            .iter()
            .filter(|articulation| {
                matches!(
                    articulation,
                    Articulation::Staccato | Articulation::Staccatissimo
                )
            })
            .count() as f32
            * TUPLET_ARTICULATION_CLEARANCE_STACKED_POINT
    } else {
        0.0
    };

    articulations
        .iter()
        .map(|articulation| match articulation {
            Articulation::Staccato => TUPLET_ARTICULATION_CLEARANCE_STACCATO,
            Articulation::Tenuto => TUPLET_ARTICULATION_CLEARANCE_TENUTO,
            Articulation::Staccatissimo => TUPLET_ARTICULATION_CLEARANCE_STACCATISSIMO,
            Articulation::StaccatoTenuto => TUPLET_ARTICULATION_CLEARANCE_STACCATO_TENUTO,
            Articulation::Accent | Articulation::Stressed | Articulation::Unstressed => {
                TUPLET_ARTICULATION_CLEARANCE_ACCENT
            }
            Articulation::Marcato => TUPLET_ARTICULATION_CLEARANCE_MARCATO,
        })
        .sum::<f32>()
        + stacked_point_clearance
}

pub fn draw_articulations(
    origin: &Point,
    articulations: &FxHashSet<Articulation>,
    notation: &Notation,
    glyph_width: f32,
    stem_direction: Option<&Direction>,
    stem_length: Option<&StemDef>,
    renderer: &mut RendererContext,
) {
    if notation.tones.is_empty() {
        return;
    }

    // require precomputed ToneOffsetInfo (parsing guarantees this). If missing, bail out.
    let info = match &notation.info {
        Some(i) => i,
        None => return,
    };

    let x = origin.x + (glyph_width / 2.0);

    for articulation in articulations.iter() {
        match articulation {
            Articulation::Staccato => {
                let glyph = match stem_direction {
                    Some(Direction::Down) | None => "\u{E4A2}",
                    Some(Direction::Up) => "\u{E4A3}",
                };
                let y = match stem_direction {
                    Some(Direction::Down) | None => {
                        let offset = origin.y + (info.highest as f32 / 2.0) - 0.85;

                        if info.highest > 5 {
                            origin.y + 1.65
                        } else if info.highest > -3 && info.highest % 2 == 0 {
                            offset - 0.5
                        } else {
                            offset
                        }
                    }
                    Some(Direction::Up) => {
                        let offset = origin.y + (info.lowest as f32 / 2.0) + 0.85;

                        if info.lowest < -5 {
                            origin.y - 1.65
                        } else if info.lowest < 3 && info.lowest % 2 == 0 {
                            offset + 0.5
                        } else {
                            offset
                        }
                    }
                };
                renderer.text(
                    glyph,
                    &Font {
                        size: 4.0,
                        family: FontFamily::Bravura,
                        align: Align::Bottom,
                        justify: Justify::Middle,
                    },
                    &Point { x, y },
                    Color::BLACK,
                );
            }
            Articulation::Tenuto => {
                let glyph = match stem_direction {
                    Some(Direction::Down) | None => "\u{E4A4}",
                    Some(Direction::Up) => "\u{E4A5}",
                };
                let y = match stem_direction {
                    Some(Direction::Down) | None => {
                        let offset = origin.y + (info.highest as f32 / 2.0) - 0.9;

                        if info.highest > 5 {
                            origin.y + 1.6
                        } else if info.highest > -3 && info.highest % 2 == 0 {
                            offset - 0.5
                        } else {
                            offset
                        }
                    }
                    Some(Direction::Up) => {
                        let offset = origin.y + (info.lowest as f32 / 2.0) + 0.9;

                        if info.lowest < -5 {
                            origin.y - 1.6
                        } else if info.lowest < 3 && info.lowest % 2 == 0 {
                            offset + 0.5
                        } else {
                            offset
                        }
                    }
                };
                renderer.text(
                    glyph,
                    &Font {
                        size: 4.0,
                        family: FontFamily::Bravura,
                        align: Align::Bottom,
                        justify: Justify::Middle,
                    },
                    &Point { x, y },
                    Color::BLACK,
                );
            }
            Articulation::Staccatissimo => {
                // cannot be rendered inside the stave, clamp to the nearest possible position instead.
                let glyph = match stem_direction {
                    Some(Direction::Down) | None => "\u{E4A6}",
                    Some(Direction::Up) => "\u{E4A7}",
                };
                let y = match stem_direction {
                    Some(Direction::Down) | None => {
                        let highest_clamped = info.highest.min(-3);
                        origin.y + (highest_clamped as f32 / 2.0) - 0.95
                    }
                    Some(Direction::Up) => {
                        let lowest_clamped = info.lowest.max(3);
                        origin.y + (lowest_clamped as f32 / 2.0) + 0.95
                    }
                };
                renderer.text(
                    glyph,
                    &Font {
                        size: 4.0,
                        family: FontFamily::Bravura,
                        align: Align::Bottom,
                        justify: Justify::Middle,
                    },
                    &Point { x, y },
                    Color::BLACK,
                );
            }
            Articulation::StaccatoTenuto => {
                let staccato_glyph = match stem_direction {
                    Some(Direction::Down) | None => "\u{E4A2}",
                    Some(Direction::Up) => "\u{E4A3}",
                };

                let staccato_y = match stem_direction {
                    Some(Direction::Down) | None => {
                        let offset = origin.y + (info.highest as f32 / 2.0) - 0.85;

                        if info.highest > 5 {
                            origin.y + 1.65
                        } else if info.highest > -3 && info.highest % 2 == 0 {
                            offset - 0.5
                        } else {
                            offset
                        }
                    }
                    Some(Direction::Up) => {
                        let offset = origin.y + (info.lowest as f32 / 2.0) + 0.85;

                        if info.lowest < -5 {
                            origin.y - 1.65
                        } else if info.lowest < 3 && info.lowest % 2 == 0 {
                            offset + 0.5
                        } else {
                            offset
                        }
                    }
                };

                renderer.text(
                    staccato_glyph,
                    &Font {
                        size: 4.0,
                        family: FontFamily::Bravura,
                        align: Align::Bottom,
                        justify: Justify::Middle,
                    },
                    &Point { x, y: staccato_y },
                    Color::BLACK,
                );

                let tenuto_glyph = match stem_direction {
                    Some(Direction::Down) | None => "\u{E4A4}",
                    Some(Direction::Up) => "\u{E4A5}",
                };

                let tenuto_y = match stem_direction {
                    Some(Direction::Down) | None => {
                        if info.highest > -2 {
                            staccato_y - 1.05
                        } else {
                            staccato_y - 0.55
                        }
                    }
                    Some(Direction::Up) => {
                        if info.lowest < 2 {
                            staccato_y + 1.05
                        } else {
                            staccato_y + 0.55
                        }
                    }
                };

                renderer.text(
                    tenuto_glyph,
                    &Font {
                        size: 4.0,
                        family: FontFamily::Bravura,
                        align: Align::Bottom,
                        justify: Justify::Middle,
                    },
                    &Point { x, y: tenuto_y },
                    Color::BLACK,
                );
            }
            _ => (),
        }
    }

    if articulations.contains(&Articulation::Accent) {
        let glyph = match stem_direction {
            Some(Direction::Down) | None => "\u{E4A0}",
            Some(Direction::Up) => "\u{E4A1}",
        };

        let y = match stem_direction {
            Some(Direction::Down) | None => {
                let offset = origin.y + (info.highest as f32 / 2.0);

                let has_single_space_articulation = articulations.contains(&Articulation::Staccato)
                    || articulations.contains(&Articulation::Tenuto);

                if has_single_space_articulation {
                    if info.highest > 5 {
                        origin.y + 1.0
                    } else if info.highest > 0 && info.highest % 2 != 0 {
                        offset - 1.5
                    } else if info.highest > 0 && info.highest % 2 == 0 {
                        offset - 2.0
                    } else if info.highest <= 0 && info.highest > -2 {
                        origin.y - 2.5
                    } else if info.highest == -2 {
                        origin.y - 3.0
                    } else {
                        offset - 1.5
                    }
                } else if articulations.contains(&Articulation::StaccatoTenuto) {
                    if info.highest > 4 {
                        origin.y
                    } else if info.highest == 4 {
                        offset - 3.0
                    } else if info.highest == 3 {
                        offset - 2.5
                    } else if info.highest == 2 {
                        offset - 3.5
                    } else if info.highest == 1 || info.highest == 0 {
                        offset - 3.0
                    } else if info.highest == -1 || info.highest == -2 {
                        offset - 2.5
                    } else {
                        offset - 2.0
                    }
                } else if articulations.contains(&Articulation::Staccatissimo) {
                    (origin.y - 4.0).min(offset - 2.5)
                } else if info.highest > 5 {
                    origin.y + 2.0
                } else if info.highest > 0 && info.highest % 2 != 0 {
                    offset - 1.5
                } else if info.highest < 0 && info.highest > -4 {
                    origin.y - 2.5
                } else {
                    offset - 1.0
                }
            }
            Some(Direction::Up) => {
                let offset = origin.y + (info.lowest as f32 / 2.0);

                let has_single_space_articulation = articulations.contains(&Articulation::Staccato)
                    || articulations.contains(&Articulation::Tenuto);

                if has_single_space_articulation {
                    if info.lowest < -5 {
                        origin.y - 1.0
                    } else if info.lowest < 0 && info.lowest % 2 != 0 {
                        offset + 1.5
                    } else if info.lowest < 0 && info.lowest % 2 == 0 {
                        offset + 2.0
                    } else if info.lowest >= 0 && info.lowest < 2 {
                        origin.y + 2.5
                    } else if info.lowest == 2 {
                        origin.y + 3.0
                    } else {
                        offset + 1.5
                    }
                } else if articulations.contains(&Articulation::StaccatoTenuto) {
                    if info.lowest < -4 {
                        origin.y
                    } else if info.lowest == -4 {
                        offset + 3.0
                    } else if info.lowest == -3 {
                        offset + 2.5
                    } else if info.lowest == -2 {
                        offset + 3.5
                    } else if info.lowest == -1 || info.lowest == 0 {
                        offset + 3.0
                    } else if info.lowest == 1 || info.lowest == 2 {
                        offset + 2.5
                    } else {
                        offset + 2.0
                    }
                } else if articulations.contains(&Articulation::Staccatissimo) {
                    (origin.y + 4.0).max(offset + 2.5)
                } else if info.lowest < -5 {
                    origin.y - 2.0
                } else if info.lowest < 0 && info.lowest % 2 != 0 {
                    offset + 1.5
                } else if info.lowest > 0 && info.lowest < 4 {
                    origin.y + 2.5
                } else {
                    offset + 1.0
                }
            }
        };

        renderer.text(
            glyph,
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point { x, y },
            Color::BLACK,
        );
    }

    if articulations.contains(&Articulation::Marcato) {
        let has_single_space_articulation = articulations.contains(&Articulation::Staccato)
            || articulations.contains(&Articulation::Tenuto);

        let glyph = "\u{E4AC}";

        let y = match stem_direction {
            Some(Direction::Down) | None => {
                let offset = origin.y + (info.highest as f32 / 2.0);

                if has_single_space_articulation {
                    if info.highest == 0 {
                        offset - 2.5
                    } else if info.highest == -1 || info.highest == -2 {
                        offset - 2.0
                    } else {
                        offset - 1.5
                    }
                } else if articulations.contains(&Articulation::StaccatoTenuto) {
                    if info.highest == 0 {
                        offset - 3.0
                    } else if info.highest == -1 || info.highest == -2 {
                        offset - 2.5
                    } else {
                        offset - 2.0
                    }
                } else if articulations.contains(&Articulation::Staccatissimo) {
                    (origin.y - 4.0).min(offset - 2.5)
                } else {
                    (origin.y - 2.5).min(offset - 1.0)
                }
            }
            Some(Direction::Up) => {
                if let Some(stem_len) = stem_length {
                    (origin.y - stem_len.tail.y.abs() - 0.5).min(origin.y - 2.5)
                } else {
                    origin.y - 2.5
                }
            }
        };

        renderer.text(
            glyph,
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point { x, y },
            Color::BLACK,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
    use crate::instructions::parse::get_written_durations::Notation;
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::components::articulation::Articulation;
    use crate::score::components::duration::Direction;
    use crate::score::components::pitch::{Accidental, Pitch};
    use crate::score::components::velocity::Velocity;
    use crate::score::entries::tone::Tone;
    use rustc_hash::FxHashSet;

    fn make_notation_with_tone() -> Notation {
        let mut n = Notation::new(0, 32);
        n.tones.push(Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            crate::score::components::articulation::Articulations::new(),
        ));
        n.info = Some(ToneOffsetInfo {
            highest: 0,
            lowest: 0,
            furthest: 0,
        });
        n
    }

    #[test]
    fn empty_tones_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let notation = Notation::new(0, 32); // tones empty
        let arts: FxHashSet<Articulation> = [Articulation::Staccato].into_iter().collect();
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &notation,
            1.0,
            Some(&Direction::Down),
            None,
            &mut renderer,
        );
        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn no_offset_info_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let mut notation = Notation::new(0, 32);
        notation.tones.push(Tone::new(
            1,
            0,
            32,
            Pitch::new(60, Accidental::Natural),
            Velocity::new(80),
            crate::score::components::articulation::Articulations::new(),
        ));
        // info remains None
        let arts: FxHashSet<Articulation> = [Articulation::Staccato].into_iter().collect();
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &notation,
            1.0,
            Some(&Direction::Down),
            None,
            &mut renderer,
        );
        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn staccato_stem_down_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let notation = make_notation_with_tone();
        let arts: FxHashSet<Articulation> = [Articulation::Staccato].into_iter().collect();
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &notation,
            1.0,
            Some(&Direction::Down),
            None,
            &mut renderer,
        );
        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
    }

    #[test]
    fn staccato_stem_up_uses_different_glyph_than_stem_down() {
        let mut renderer_down = make_test_renderer();
        let mut renderer_up = make_test_renderer();
        let notation = make_notation_with_tone();
        let arts: FxHashSet<Articulation> = [Articulation::Staccato].into_iter().collect();
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &notation,
            1.0,
            Some(&Direction::Down),
            None,
            &mut renderer_down,
        );
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &notation,
            1.0,
            Some(&Direction::Up),
            None,
            &mut renderer_up,
        );
        if let (
            DrawInstruction::Text { text: t_down, .. },
            DrawInstruction::Text { text: t_up, .. },
        ) = (&renderer_down.instructions[0], &renderer_up.instructions[0])
        {
            assert_ne!(t_down, t_up);
        } else {
            panic!("expected Text instructions");
        }
    }

    #[test]
    fn tenuto_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let notation = make_notation_with_tone();
        let arts: FxHashSet<Articulation> = [Articulation::Tenuto].into_iter().collect();
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &notation,
            1.0,
            Some(&Direction::Down),
            None,
            &mut renderer,
        );
        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(
            renderer.instructions[0],
            DrawInstruction::Text { .. }
        ));
    }

    fn accent_y(renderer: &crate::renderer::RendererContext, glyph: &str) -> f32 {
        renderer
            .instructions
            .iter()
            .find_map(|instruction| match instruction {
                DrawInstruction::Text { text, point, .. } if text == glyph => Some(point.1),
                _ => None,
            })
            .expect("accent glyph should be drawn")
    }

    #[test]
    fn accent_stem_down_position_ignores_lower_note_extent() {
        let mut high_only_renderer = make_test_renderer();
        let mut wide_chord_renderer = make_test_renderer();
        let notation = make_notation_with_tone();
        let arts: FxHashSet<Articulation> = [Articulation::Accent].into_iter().collect();
        let accent_down = "\u{E4A0}";

        let mut high_only = notation.clone();
        high_only.info = Some(ToneOffsetInfo {
            highest: -2,
            lowest: 2,
            furthest: 2,
        });

        let mut wide_chord = notation;
        wide_chord.info = Some(ToneOffsetInfo {
            highest: -2,
            lowest: -3,
            furthest: -3,
        });

        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &high_only,
            1.0,
            Some(&Direction::Down),
            None,
            &mut high_only_renderer,
        );
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &wide_chord,
            1.0,
            Some(&Direction::Down),
            None,
            &mut wide_chord_renderer,
        );

        assert_eq!(
            accent_y(&high_only_renderer, accent_down),
            accent_y(&wide_chord_renderer, accent_down)
        );
    }

    #[test]
    fn accent_staccato_tenuto_stem_down_position_ignores_lower_note_extent() {
        let mut shallow_renderer = make_test_renderer();
        let mut deep_renderer = make_test_renderer();
        let notation = make_notation_with_tone();
        let arts: FxHashSet<Articulation> = [Articulation::Accent, Articulation::StaccatoTenuto]
            .into_iter()
            .collect();
        let accent_down = "\u{E4A0}";

        let mut shallow_chord = notation.clone();
        shallow_chord.info = Some(ToneOffsetInfo {
            highest: -1,
            lowest: 4,
            furthest: 4,
        });

        let mut deep_chord = notation;
        deep_chord.info = Some(ToneOffsetInfo {
            highest: -1,
            lowest: -6,
            furthest: -6,
        });

        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &shallow_chord,
            1.0,
            Some(&Direction::Down),
            None,
            &mut shallow_renderer,
        );
        draw_articulations(
            &Point { x: 0.0, y: 0.0 },
            &arts,
            &deep_chord,
            1.0,
            Some(&Direction::Down),
            None,
            &mut deep_renderer,
        );

        assert_eq!(
            accent_y(&shallow_renderer, accent_down),
            accent_y(&deep_renderer, accent_down)
        );
    }
}
