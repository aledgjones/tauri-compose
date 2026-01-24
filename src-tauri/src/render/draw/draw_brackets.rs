use super::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::render::parse::get_vertical_spans::VerticalSpans;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::engraves::{BracketStyle, Engrave};
use skia_safe::Color;

pub fn draw_brackets(
    origin: &Point,
    vertical_spans: &VerticalSpans,
    vertical_spacing: &VerticalSpacing,
    engrave: &Engrave,
    renderer: &mut RendererContext,
) {
    for bracket in &vertical_spans.brackets {
        // short circuit if its a single stave and we don't want single staves bracketed.
        if !engrave.bracket_single_staves && bracket.start == bracket.stop {
            continue;
        }

        let start = &vertical_spacing.instruments[&bracket.start];
        let stop = &vertical_spacing.instruments[&bracket.stop];
        let top = origin.y + start.y;
        let bottom = origin.y + stop.y + stop.height;

        match engrave.bracket_style {
            BracketStyle::None => {
                // think line
                renderer.line(
                    0.5,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x - 0.75,
                            y: top - (STAVE_LINE_WIDTH / 2.0),
                        },
                        Point {
                            x: origin.x - 0.75,
                            y: bottom + (STAVE_LINE_WIDTH / 2.0),
                        },
                    ],
                );
            }
            BracketStyle::Line => {
                // think line
                renderer.line(
                    0.5,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x - 0.75,
                            y: top,
                        },
                        Point {
                            x: origin.x - 0.75,
                            y: bottom,
                        },
                    ],
                );
                // thin lines
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x - 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: top,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x - 1.0,
                            y: bottom,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
            }
            BracketStyle::Wing => {
                // think line
                renderer.line(
                    0.5,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x - 0.75,
                            y: top - 0.5,
                        },
                        Point {
                            x: origin.x - 0.75,
                            y: bottom + 0.5,
                        },
                    ],
                );
                // wings
                renderer.text(
                    "\u{E003}",
                    &Font {
                        family: FontFamily::Bravura,
                        size: 4.0,
                        justify: Justify::Start,
                        align: Align::Middle,
                    },
                    &Point {
                        x: origin.x - 1.0,
                        y: top - 1.0,
                    },
                    Color::BLACK,
                );
                renderer.text(
                    "\u{E004}",
                    &Font {
                        family: FontFamily::Bravura,
                        size: 4.0,
                        justify: Justify::Start,
                        align: Align::Middle,
                    },
                    &Point {
                        x: origin.x - 1.0,
                        y: bottom,
                    },
                    Color::BLACK,
                );
            }
        }
    }
}
