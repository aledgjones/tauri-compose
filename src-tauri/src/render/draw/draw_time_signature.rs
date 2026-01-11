use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};
use skia_safe::Color;

pub fn draw_time_signature(
    origin: &Point,
    time_signature: &TimeSignature,
    renderer: &mut Renderer,
) {
    let offset = (time_signature.metrics().width - 2.0) * 0.5;

    match time_signature.draw_type {
        TimeSignatureDrawType::Hidden => (),
        TimeSignatureDrawType::CommonTime => renderer.text(
            "\u{E08A}",
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point {
                y: origin.y,
                x: origin.x + offset,
            },
            Color::BLACK,
        ),
        TimeSignatureDrawType::SplitCommonTime => renderer.text(
            "\u{E08B}",
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point {
                y: origin.y,
                x: origin.x + offset,
            },
            Color::BLACK,
        ),
        TimeSignatureDrawType::Open => renderer.text(
            "\u{E09C}",
            &Font {
                size: 4.0,
                family: FontFamily::Bravura,
                align: Align::Bottom,
                justify: Justify::Middle,
            },
            &Point {
                y: origin.y,
                x: origin.x + offset,
            },
            Color::BLACK,
        ),
        TimeSignatureDrawType::Regular => {
            renderer.text(
                &time_signature.beats_to_glyph(),
                &Font {
                    size: 4.0,
                    family: FontFamily::Bravura,
                    align: Align::Bottom,
                    justify: Justify::Middle,
                },
                &Point {
                    y: origin.y - 1.0,
                    x: origin.x + offset,
                },
                Color::BLACK,
            );
            renderer.text(
                &time_signature.beat_type_to_glyph(),
                &Font {
                    size: 4.0,
                    family: FontFamily::Bravura,
                    align: Align::Bottom,
                    justify: Justify::Middle,
                },
                &Point {
                    y: origin.y + 1.0,
                    x: origin.x + offset,
                },
                Color::BLACK,
            );
        }
    };
}
