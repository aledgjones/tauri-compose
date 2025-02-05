use crate::render::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::renderer::Renderer;
use crate::score::components::duration::NoteDuration;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use crate::score::tracks::Tracks;

pub fn debug_tone_drawing(
    origin: &Point,
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    tracks: &Tracks,
    vertical_spacing: &VerticalSpacing,
    tone_offsets: &ToneVerticalOffsets,
    renderer: &mut Renderer,
) {
    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave_spacing = &vertical_spacing.staves[stave_id];
            let top = origin.y + stave_spacing.y;

            let stave = &flow.staves.by_id[stave_id];
            for track_id in &stave.track_ids {
                let track = &tracks.by_id[track_id];

                for (tone_id, tone) in &track.entries.by_id {
                    let tone_offset = tone_offsets[tone_id] as f32 / 2.0;

                    renderer.text(
                        NoteDuration::Quarter.to_notehead_glyph(),
                        &Font {
                            family: FontFamily::Bravura,
                            justify: Justify::Start,
                            align: Align::Bottom,
                            size: 4.0,
                        },
                        &Point {
                            x: origin.x + (tone.tick() as f32 * 0.02),
                            y: top + tone_offset,
                        },
                    );
                }
            }
        }
    }
}
