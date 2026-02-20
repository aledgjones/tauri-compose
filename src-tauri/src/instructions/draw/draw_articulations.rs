use crate::instructions::parse::get_stem_lengths::StemDef;
use crate::instructions::parse::get_written_durations::Notation;
use crate::renderer::RendererContext;
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::Direction;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use rustc_hash::FxHashSet;
use skia_safe::Color;

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
                        if info.highest == -2 || info.highest == 0 {
                            offset - 0.5
                        } else {
                            offset
                        }
                    }
                    Some(Direction::Up) => {
                        let offset = origin.y + (info.lowest as f32 / 2.0) + 0.85;
                        if info.lowest == 2 || info.lowest == 0 {
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
                        if info.highest == -2 || info.highest == 0 {
                            offset - 0.5
                        } else {
                            offset
                        }
                    }
                    Some(Direction::Up) => {
                        let offset = origin.y + (info.lowest as f32 / 2.0) + 0.9;
                        if info.lowest == 2 || info.lowest == 0 {
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
                        if info.highest == -2 || info.highest == 0 {
                            offset - 0.5
                        } else {
                            offset
                        }
                    }
                    Some(Direction::Up) => {
                        let offset = origin.y + (info.lowest as f32 / 2.0) + 0.85;
                        if info.lowest == 2 || info.lowest == 0 {
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
                        if info.highest == -1 || info.highest == 0 {
                            staccato_y - 1.05
                        } else {
                            staccato_y - 0.55
                        }
                    }
                    Some(Direction::Up) => {
                        if info.lowest == 1 || info.lowest == 0 {
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
                } else if info.highest == -1 {
                    offset - 2.0
                } else if info.highest == -2 {
                    offset - 1.5
                } else {
                    offset - 1.0
                }
            }
            Some(Direction::Up) => {
                let offset = origin.y + (info.lowest as f32 / 2.0);

                let has_single_space_articulation = articulations.contains(&Articulation::Staccato)
                    || articulations.contains(&Articulation::Tenuto);

                if has_single_space_articulation {
                    if info.lowest == 0 {
                        offset + 2.5
                    } else if info.lowest == 1 || info.lowest == 2 {
                        offset + 2.0
                    } else {
                        offset + 1.5
                    }
                } else if articulations.contains(&Articulation::StaccatoTenuto) {
                    if info.lowest == 0 {
                        offset + 3.0
                    } else if info.lowest == 1 || info.lowest == 2 {
                        offset + 2.5
                    } else {
                        offset + 2.0
                    }
                } else if articulations.contains(&Articulation::Staccatissimo) {
                    (origin.y + 4.0).max(offset + 2.5)
                } else if info.lowest == 1 {
                    offset + 2.0
                } else if info.lowest == 2 {
                    offset + 1.5
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
