use crate::instructions::parse::get_written_durations::Notation;
use crate::renderer::RendererContext;
use crate::score::components::articulation::Articulation;
use crate::score::components::duration::Direction;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use skia_safe::Color;

pub fn draw_articulations(
    origin: &Point,
    articulation: &Articulation,
    notation: &Notation,
    glyph_width: f32,
    stem_direction: Option<&Direction>,
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
        _ => (),
    }
}
