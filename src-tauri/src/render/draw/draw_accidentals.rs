use crate::render::parse::get_accidentals::AccidentalNotation;
use crate::render::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::render::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::renderer::Renderer;
use crate::score::components::duration::Tick;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::tone::Tone;

pub fn draw_accidental(
    origin: &Point,
    tick: Tick,
    accidental: &AccidentalNotation,
    tone: &Tone,
    tone_offsets: &ToneVerticalOffsets,
    horizontal_spacing: &HorizontalSpacing,
    renderer: &mut Renderer,
) {
    let glyph = tone.pitch.accidental.to_glyph();
    let offset = tone_offsets[&tone.id];
    let top = origin.y + (offset as f32 / 2.0);

    let x = horizontal_spacing.get(tick, Slot::NoteSpacing).x;
    let left = origin.x + x - ((accidental.slot as f32 - 1.0) * 1.1) - 0.25;

    renderer.text(
        &glyph,
        &Font {
            size: 4.0,
            family: FontFamily::Bravura,
            align: Align::Bottom,
            justify: Justify::End,
        },
        &Point { x: left, y: top },
    );
}
