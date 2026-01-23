use crate::render::parse::get_shunts::{Shunt, Shunts};
use crate::renderer::HitTarget;
use crate::renderer::Renderer;
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
    renderer: &mut Renderer,
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
