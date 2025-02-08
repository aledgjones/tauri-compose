use skia_safe::Color;

use crate::render::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_bars::Bars;
use crate::render::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::render::parse::get_written_durations::NotationByTrack;
use crate::renderer::Renderer;
use crate::score::components::duration::NoteDuration;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::flows::Flow;
use crate::score::instruments::Instrument;

pub fn debug_tone_drawing(
    origin: &Point,
    flow: &Flow,
    instruments: &Vec<&Instrument>,
    bars: &Bars,
    notation_tracks: &NotationByTrack,
    vertical_spacing: &VerticalSpacing,
    tone_offsets: &ToneVerticalOffsets,
    renderer: &mut Renderer,
) {
    for instrument in instruments {
        for stave_id in &instrument.stave_ids {
            let stave_spacing = &vertical_spacing.staves[stave_id];
            let top = origin.y + stave_spacing.y;

            for tick in bars.keys() {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + (*tick as f32 * 0.02),
                            y: top - 2.0,
                        },
                        Point {
                            x: origin.x + (*tick as f32 * 0.02),
                            y: top + 2.0,
                        },
                    ],
                );
            }

            let stave = &flow.staves.by_id[stave_id];
            for track_id in &stave.track_ids {
                let notation_track = &notation_tracks[track_id];

                for notation in notation_track.track.values() {
                    // DEBUG RESTS
                    if notation.is_rest() {
                        renderer.line(
                            STAVE_LINE_WIDTH * 2.0,
                            Color::BLUE,
                            vec![
                                Point {
                                    x: origin.x + (notation.tick as f32 * 0.02),
                                    y: top - 1.0,
                                },
                                Point {
                                    x: origin.x + (notation.tick as f32 * 0.02),
                                    y: top,
                                },
                                Point {
                                    x: origin.x
                                        + (notation.tick as f32 * 0.02)
                                        + (notation.duration as f32 * 0.02),
                                    y: top,
                                },
                            ],
                        );
                    };

                    // DEBUG TONES
                    for tone in &notation.tones {
                        let tone_offset = tone_offsets[&tone.id] as f32 / 2.0;

                        // DEBUG TIES
                        if notation.ties.contains(&tone.id) {
                            renderer.line(
                                STAVE_LINE_WIDTH * 2.0,
                                Color::RED,
                                vec![
                                    Point {
                                        x: origin.x + (notation.tick as f32 * 0.02),
                                        y: top + tone_offset,
                                    },
                                    Point {
                                        x: origin.x + (notation.tick as f32 * 0.02),
                                        y: top + tone_offset + 1.0,
                                    },
                                    Point {
                                        x: origin.x
                                            + (notation.tick as f32 * 0.02)
                                            + (notation.duration as f32 * 0.02),
                                        y: top + tone_offset + 1.0,
                                    },
                                    Point {
                                        x: origin.x
                                            + (notation.tick as f32 * 0.02)
                                            + (notation.duration as f32 * 0.02),
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
                                x: origin.x + (notation.tick as f32 * 0.02),
                                y: top + tone_offset,
                            },
                        );
                    }
                }
            }
        }
    }
}
