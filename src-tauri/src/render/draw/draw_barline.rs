use super::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_vertical_spacing::VerticalSpacing;
use crate::render::parse::get_vertical_spans::VerticalSpans;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::entries::barline::BarlineDrawType;
use skia_safe::Color;

pub fn draw_barline(
    origin: &Point,
    draw_type: &BarlineDrawType,
    vertical_spacing: &VerticalSpacing,
    vertical_spans: &VerticalSpans,
    renderer: &mut RendererContext,
) {
    let tweak_for_stave_line_width = STAVE_LINE_WIDTH / 2.0;

    for span in &vertical_spans.barlines {
        let start = &vertical_spacing.instruments[&span.start];
        let stop = &vertical_spacing.instruments[&span.stop];

        let top = origin.y + start.y - tweak_for_stave_line_width;
        let bottom = origin.y + stop.y + stop.height + tweak_for_stave_line_width;

        match draw_type {
            BarlineDrawType::Single => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Double => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 0.5,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.5,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Triple => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 0.5,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.5,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Dashed => {
                renderer.dashed_line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::Final => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x,
                            y: top,
                        },
                        Point {
                            x: origin.x,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 0.75,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.75,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::StartRepeat => {
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 0.25,
                            y: top,
                        },
                        Point {
                            x: origin.x + 0.25,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::EndRepeat => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 1.75,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.75,
                            y: bottom,
                        },
                    ],
                );
            }
            BarlineDrawType::EndStartRepeat => {
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 1.0,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.0,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH * 4.0,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 1.75,
                            y: top,
                        },
                        Point {
                            x: origin.x + 1.75,
                            y: bottom,
                        },
                    ],
                );
                renderer.line(
                    STAVE_LINE_WIDTH,
                    Color::BLACK,
                    vec![
                        Point {
                            x: origin.x + 2.5,
                            y: top,
                        },
                        Point {
                            x: origin.x + 2.5,
                            y: bottom,
                        },
                    ],
                );
            }
        }
    }

    match draw_type {
        BarlineDrawType::EndRepeat => {
            for position in vertical_spacing.staves.values() {
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y + 0.5,
                    },
                );
            }
        }
        BarlineDrawType::EndStartRepeat => {
            for position in vertical_spacing.staves.values() {
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 0.25,
                        y: origin.y + position.y + 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 3.25,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 3.25,
                        y: origin.y + position.y + 0.5,
                    },
                );
            }
        }
        BarlineDrawType::StartRepeat => {
            for position in vertical_spacing.staves.values() {
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 1.75,
                        y: origin.y + position.y - 0.5,
                    },
                );
                renderer.circle(
                    0.25,
                    Color::BLACK,
                    &Point {
                        x: origin.x + 1.75,
                        y: origin.y + position.y + 0.5,
                    },
                );
            }
        }
        _ => (),
    }
}
