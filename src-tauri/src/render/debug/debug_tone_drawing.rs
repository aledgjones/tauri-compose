use crate::render::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_horizontal_spacing::{HorizontalSpacing, Slot};
use crate::render::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::render::parse::get_written_durations::NotationByTrack;
use crate::renderer::Renderer;
use crate::score::components::duration::NoteDuration;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;
use skia_safe::Color;

pub fn debug_tone_drawing(
    origin: &Point,
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    notation_tracks: &NotationByTrack,
    vertical_spacing: &VerticalSpacing,
    horizontal_spacing: &HorizontalSpacing,
    tone_offsets: &ToneVerticalOffsets,
    renderer: &mut Renderer,
) {
    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave_spacing = &vertical_spacing.staves[stave_id];
            let top = origin.y + stave_spacing.y;

            let stave = &flow.staves.by_id[stave_id];
            for track_id in &stave.track_ids {
                let notation_track = &notation_tracks[track_id];

                for notation in notation_track.track.values() {
                    let start = horizontal_spacing.get(notation.tick, Slot::NoteSpacing);
                    let end = horizontal_spacing
                        .get(notation.tick + notation.duration, Slot::NoteSpacing);

                    // DEBUG RESTS
                    if notation.is_rest() {
                        renderer.line(
                            STAVE_LINE_WIDTH * 2.0,
                            Color::BLUE,
                            vec![
                                Point {
                                    x: origin.x + start.x,
                                    y: top - 1.0,
                                },
                                Point {
                                    x: origin.x + start.x,
                                    y: top,
                                },
                                Point {
                                    x: origin.x + end.x,
                                    y: top,
                                },
                            ],
                        );
                    };

                    // DEBUG TONES
                    for tone in &notation.tones {
                        let tone_offset = tone_offsets.by_tone_id[&tone.id] as f32 / 2.0;

                        // DEBUG TIES
                        if notation.ties.contains(&tone.id) {
                            renderer.line(
                                STAVE_LINE_WIDTH * 2.0,
                                Color::RED,
                                vec![
                                    Point {
                                        x: origin.x + start.x,
                                        y: top + tone_offset,
                                    },
                                    Point {
                                        x: origin.x + start.x,
                                        y: top + tone_offset + 1.0,
                                    },
                                    Point {
                                        x: origin.x + end.x,
                                        y: top + tone_offset + 1.0,
                                    },
                                    Point {
                                        x: origin.x + end.x,
                                        y: top + tone_offset,
                                    },
                                ],
                            );
                        }

                        // DEBUG NOTE HEADS
                        renderer.text(
                            NoteDuration::Quarter.to_notehead_glyph(),
                            &Font {
                                family: FontFamily::Bravura,
                                justify: Justify::Start,
                                align: Align::Bottom,
                                size: 4.0,
                            },
                            &Point {
                                x: origin.x + start.x,
                                y: top + tone_offset,
                            },
                        );
                    }
                }
            }
        }
    }
}
