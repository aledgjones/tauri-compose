use crate::render::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_shunts::{Shunt, Shunts};
use crate::render::parse::get_written_durations::Notation;
use crate::renderer::converter::Space;
use crate::renderer::Renderer;
use crate::score::components::duration::Tick;
use crate::score::components::measurements::Point;
use skia_safe::Color;

#[allow(clippy::too_many_arguments)]
pub fn draw_lines<T: Iterator<Item = i8>>(
    origin: &Point,
    range: T,
    tick: &Tick,
    glyph_width: Space,
    shunts: &Shunts,
    renderer: &mut Renderer,
) {
    let notehead = glyph_width;

    let mut start_x = origin.x;
    let mut stop_x = origin.x;

    for offset in range {
        // get the furthest start position
        if let Some(shunt) = shunts.by_offset.get(&(*tick, offset)) {
            match shunt {
                Shunt::Pre => {
                    if origin.x - notehead < start_x {
                        start_x = origin.x - notehead;
                    }
                }
                Shunt::None => {
                    if origin.x + notehead > stop_x {
                        stop_x = origin.x + notehead;
                    }
                }
                Shunt::Post => {
                    if origin.x + (notehead * 2.0) > stop_x {
                        stop_x = origin.x + (notehead * 2.0);
                    }
                }
            }
        };

        if offset % 2 == 0 {
            let y = origin.y + (offset as f32 / 2.0);

            let start = start_x - 0.4;
            let stop = stop_x + 0.4;

            renderer.line(
                STAVE_LINE_WIDTH * 1.5,
                Color::BLACK,
                vec![Point { x: start, y }, Point { x: stop, y }],
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ledger_line(
    origin: &Point,
    tick: &Tick,
    entry: &Notation,
    glyph_width: Space,
    shunts: &Shunts,
    renderer: &mut Renderer,
) {
    let info = entry.info.as_ref().unwrap();
    let highest = info.highest;
    let lowest = info.lowest;

    let from = if highest > -5 { -5 } else { highest };
    let to = if lowest < 5 { 5 } else { lowest + 1 };

    // high ledger lines
    draw_lines(origin, from..-5, tick, glyph_width, shunts, renderer);

    // low ledger lines
    draw_lines(origin, (5..to).rev(), tick, glyph_width, shunts, renderer);
}
