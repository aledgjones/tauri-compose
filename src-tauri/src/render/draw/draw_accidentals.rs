use crate::render::parse::get_accidentals::AccidentalNotation;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::entries::tone::Tone;
use crate::ui::sequencer::Selection;
use skia_safe::Color;

fn get_accidental_slot_offset(slot: u8, glyph_width: f32, has_pre_shunt: bool) -> f32 {
    let mut offset = 0.0;

    for i in 1..slot {
        if i == 1 && has_pre_shunt {
            offset += glyph_width
        } else {
            offset += 1.1
        }
    }

    offset + 0.25
}

pub fn draw_accidental(
    origin: &Point,
    accidental: &AccidentalNotation,
    tone: &Tone,
    glyph_width: f32,
    has_pre_shunt: bool,
    selection: Option<&Selection>,
    renderer: &mut RendererContext,
) {
    let glyph = tone.pitch.accidental.to_glyph();
    let slot_offset = get_accidental_slot_offset(accidental.slot, glyph_width, has_pre_shunt);
    let left = origin.x - slot_offset;

    let color = if matches!(selection, Some(Selection::Tone { tone_id, .. }) if *tone_id == tone.id)
    {
        Color::from_rgb(14, 165, 233)
    } else {
        Color::BLACK
    };

    renderer.text(
        &glyph,
        &Font {
            size: 4.0,
            family: FontFamily::Bravura,
            align: Align::Bottom,
            justify: Justify::End,
        },
        &Point {
            x: left,
            y: origin.y,
        },
        color,
    );
}
