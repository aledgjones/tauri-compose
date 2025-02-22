use crate::renderer::Renderer;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::time_signature::{TimeSignature, TimeSignatureDrawType};

pub fn draw_time_signature(
    origin: &Point,
    time_signature: &TimeSignature,
    renderer: &mut Renderer,
) {
    let offset = (time_signature.metrics().width - 1.0) * 0.5;

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
            );
        }
    };
}
