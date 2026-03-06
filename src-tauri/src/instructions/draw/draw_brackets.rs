use super::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::instructions::parse::get_vertical_spans::VerticalSpans;
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

#[cfg(test)]
mod tests {
    use super::draw_brackets;
    use crate::instructions::draw::test_helpers::{make_bracket_spans, make_vertical_spacing};
    use crate::renderer::{DrawInstruction, test_helpers::make_test_renderer};
    use crate::score::components::measurements::{PaddingMm, Point};
    use crate::score::components::text::{Align, Font, FontFamily, Justify};
    use crate::score::engraves::{BracketingApproach, BracketStyle, Engrave};

    const INST_ID: u64 = 1;
    const STAVE_ID: u64 = 2;
    const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    fn make_engrave(bracket_style: BracketStyle) -> Engrave {
        Engrave {
            space: 1.5,
            page_padding: PaddingMm::new(0.0, 0.0, 0.0, 0.0),
            frame_padding: PaddingMm::new(0.0, 0.0, 0.0, 0.0),
            instrument_spacing: 8.0,
            stave_spacing: 6.0,
            system_start_padding: 0.0,
            instrument_name_font: Font {
                size: 1.0,
                family: FontFamily::LibreBaskerville,
                justify: Justify::Start,
                align: Align::Middle,
            },
            instrument_name_spacing: 1.0,
            systemic_barline_single_instrument_system: false,
            bracketing_approach: BracketingApproach::None,
            bracket_style,
            bracket_single_staves: true,
            base_note_space: 4.0,
            minimum_note_space: 0.9,
            minimum_tie_space: 1.6,
            note_space_ratio: 1.41,
            maximum_beam_slant: 0.5,
        }
    }

    #[test]
    fn bracket_style_none_produces_one_line() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_bracket_spans(INST_ID, INST_ID);
        let engrave = make_engrave(BracketStyle::None);
        let mut r = make_test_renderer();
        draw_brackets(&ORIGIN, &spans, &vs, &engrave, &mut r);
        let lines = r.instructions.iter()
            .filter(|i| matches!(i, DrawInstruction::Line { .. }))
            .count();
        assert_eq!(lines, 1);
    }

    #[test]
    fn bracket_style_line_produces_three_lines() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_bracket_spans(INST_ID, INST_ID);
        let engrave = make_engrave(BracketStyle::Line);
        let mut r = make_test_renderer();
        draw_brackets(&ORIGIN, &spans, &vs, &engrave, &mut r);
        let lines = r.instructions.iter()
            .filter(|i| matches!(i, DrawInstruction::Line { .. }))
            .count();
        assert_eq!(lines, 3);
    }

    #[test]
    fn bracket_style_wing_produces_one_line_and_two_text_glyphs() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_bracket_spans(INST_ID, INST_ID);
        let engrave = make_engrave(BracketStyle::Wing);
        let mut r = make_test_renderer();
        draw_brackets(&ORIGIN, &spans, &vs, &engrave, &mut r);
        let lines = r.instructions.iter()
            .filter(|i| matches!(i, DrawInstruction::Line { .. }))
            .count();
        let texts = r.instructions.iter()
            .filter(|i| matches!(i, DrawInstruction::Text { .. }))
            .count();
        assert_eq!(lines, 1);
        assert_eq!(texts, 2);
    }

    #[test]
    fn bracket_single_stave_false_skips_same_start_stop() {
        let vs = make_vertical_spacing(INST_ID, STAVE_ID);
        let spans = make_bracket_spans(INST_ID, INST_ID);
        let mut engrave = make_engrave(BracketStyle::Line);
        engrave.bracket_single_staves = false;
        let mut r = make_test_renderer();
        draw_brackets(&ORIGIN, &spans, &vs, &engrave, &mut r);
        // span.start == span.stop and bracket_single_staves=false → skipped
        assert_eq!(r.instructions.len(), 0);
    }
}
