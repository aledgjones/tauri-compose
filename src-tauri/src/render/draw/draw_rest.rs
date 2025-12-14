use crate::render::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::render::parse::get_written_durations::Notation;
use crate::renderer::Renderer;
use crate::score::components::duration::{NoteDuration, Tick, Ticks};
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use skia_safe::Color;

#[allow(clippy::too_many_arguments)]
pub fn draw_rest(
    origin: &Point,
    tick: Tick,
    glyph: &str,
    entry: &Notation,
    subdivisions: Ticks,
    is_full_bar: bool,
    horizontal_spacing: &HorizontalSpacing,
    renderer: &mut Renderer,
) {
    let start = horizontal_spacing.get(tick, Slot::NoteSpacing);

    if is_full_bar {
        let end = horizontal_spacing.get(tick + entry.duration - 1, Slot::NoteSpacing);
        let left = (origin.x + start.x + ((end.x - start.x) / 2.0)) - 1.0;

        renderer.text(
            "\u{E4E3}",
            &Font {
                family: FontFamily::Bravura,
                size: 4.0,
                justify: Justify::Start,
                align: Align::Bottom,
            },
            &Point {
                x: left,
                y: origin.y - 1.0,
            },
        );
    } else {
        let left = origin.x + start.x;
        let base = entry.base_to_note_duration(subdivisions);

        let top = match base {
            Some(NoteDuration::Whole) => origin.y - 1.0,
            _ => origin.y,
        };

        renderer.text(
            glyph,
            &Font {
                family: FontFamily::Bravura,
                size: 4.0,
                justify: Justify::Start,
                align: Align::Bottom,
            },
            &Point { x: left, y: top },
        );

        if entry.is_dotted(subdivisions) {
            renderer.circle(
                0.2,
                Color::BLACK,
                &Point {
                    x: left + entry.glyph_metrics().width + 0.5,
                    y: top - 0.5,
                },
            );
        }
    }
}
