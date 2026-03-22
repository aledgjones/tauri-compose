use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::renderer::converter::Space;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::{
    measure_tokenized_text, text_span_width, tokenize_text_with_symbols, Align, Font, FontFamily,
    Justify,
};
use crate::score::engraves::Engrave;
use crate::score::instruments::Instrument;
use skia_safe::Color;

pub fn draw_names(
    origin: &Point,
    instruments: &Vec<&Instrument>,
    vertical_spacing: &VerticalSpacing,
    engrave: &Engrave,
    debug: bool,
    renderer: &mut RendererContext,
) {
    for instrument in instruments {
        let spacing = &vertical_spacing.instruments[&instrument.id];
        let top: Space = origin.y + spacing.y + (spacing.height / 2.0);
        let name = instrument.name();
        let spans = tokenize_text_with_symbols(&name, &engrave.instrument_name_font);
        let total_width = measure_tokenized_text(
            &name,
            &engrave.instrument_name_font,
            &renderer_text_measurer(renderer),
        )
        .width;
        let mut x = match engrave.instrument_name_font.justify {
            Justify::Start => origin.x,
            Justify::Middle => origin.x - (total_width / 2.0),
            Justify::End => origin.x - total_width,
        };

        for span in spans {
            let bounds = renderer_text_measurer(renderer).measure(&span.text, &span.font);
            let advance_width = text_span_width(&span, &renderer_text_measurer(renderer));
            let span_origin = Point {
                x: x + span.x_offset,
                y: top + span.y_offset,
            };

            if debug {
                renderer.rectangle(
                    span_origin.x,
                    debug_box_y(&span_origin, bounds.height, span.font.align),
                    advance_width,
                    bounds.height,
                    debug_box_color(span.font.family),
                );
            }
            renderer.text(
                &span.text,
                &Font {
                    family: span.font.family,
                    align: span.font.align,
                    justify: Justify::Start,
                    size: span.font.size,
                },
                &span_origin,
                Color::BLACK,
            );
            x += span.x_offset + advance_width;
        }
    }
}

fn debug_box_y(origin: &Point, height: Space, align: Align) -> Space {
    match align {
        Align::Top => origin.y,
        Align::Middle => origin.y - (height / 2.0),
        Align::Bottom => origin.y - height,
    }
}

fn debug_box_color(font_family: FontFamily) -> Color {
    match font_family {
        FontFamily::LibreBaskerville => Color::from_argb(64, 249, 115, 22),
        FontFamily::Bravura => Color::from_argb(64, 14, 165, 233),
    }
}

fn renderer_text_measurer(
    renderer: &RendererContext,
) -> crate::renderer::text_measurer::TextMeasurer<'_> {
    crate::renderer::text_measurer::TextMeasurer::new(&renderer.converter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_vertical_spacing::{VerticalPosition, VerticalSpacing};
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::engraves::Engrave;
    use crate::score::engraves::Engraves;
    use crate::score::instruments::defs::InstrumentType;
    use crate::score::instruments::PlaybackAssignmentIntent;
    use crate::score::layouts::LayoutType;
    use crate::utils::UniqueId;
    use rustc_hash::FxHashMap;

    fn make_instrument(id: UniqueId, name: &str) -> Instrument {
        Instrument {
            id,
            definition_id: String::from("strings.violin"),
            instrument_type: InstrumentType::Melodic,
            long_name: String::from(name),
            short_name: String::from(name),
            stave_ids: vec![],
            count: None,
            assignment_intent: PlaybackAssignmentIntent::default(),
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    fn make_engrave() -> Engrave {
        Engraves::new()
            .by_layout_type
            .remove(&LayoutType::FullScore)
            .unwrap()
    }

    fn make_spacing(instrument_id: UniqueId) -> VerticalSpacing {
        let mut instruments = FxHashMap::default();
        instruments.insert(
            instrument_id,
            VerticalPosition {
                y: 2.0,
                height: 4.0,
            },
        );
        VerticalSpacing {
            height: 4.0,
            staves: FxHashMap::default(),
            instruments,
        }
    }

    #[test]
    fn draw_names_with_one_instrument_produces_one_text_instruction() {
        let mut renderer = make_test_renderer();
        let instrument = make_instrument(1, "Violin");
        let spacing = make_spacing(1);
        let engrave = make_engrave();

        draw_names(
            &Point { x: 0.0, y: 0.0 },
            &vec![&instrument],
            &spacing,
            &engrave,
            false,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        assert!(!renderer
            .instructions
            .iter()
            .any(|instruction| { matches!(instruction, DrawInstruction::Rectangle { .. }) }));
        assert!(renderer
            .instructions
            .iter()
            .any(|instruction| { matches!(instruction, DrawInstruction::Text { .. }) }));
    }

    #[test]
    fn draw_names_with_no_instruments_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let spacing = make_spacing(1);
        let engrave = make_engrave();

        draw_names(
            &Point { x: 0.0, y: 0.0 },
            &vec![],
            &spacing,
            &engrave,
            false,
            &mut renderer,
        );

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn draw_names_with_two_instruments_produces_two_instructions() {
        let mut renderer = make_test_renderer();
        let i1 = make_instrument(1, "Violin");
        let i2 = make_instrument(2, "Viola");
        let mut instruments = FxHashMap::default();
        instruments.insert(
            1u64,
            VerticalPosition {
                y: 0.0,
                height: 4.0,
            },
        );
        instruments.insert(
            2u64,
            VerticalPosition {
                y: 8.0,
                height: 4.0,
            },
        );
        let spacing = VerticalSpacing {
            height: 12.0,
            staves: FxHashMap::default(),
            instruments,
        };
        let engrave = make_engrave();

        draw_names(
            &Point { x: 0.0, y: 0.0 },
            &vec![&i1, &i2],
            &spacing,
            &engrave,
            false,
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 2);
    }

    #[test]
    fn draw_names_text_contains_instrument_name() {
        let mut renderer = make_test_renderer();
        let instrument = make_instrument(1, "Flute");
        let spacing = make_spacing(1);
        let engrave = make_engrave();

        draw_names(
            &Point { x: 0.0, y: 0.0 },
            &vec![&instrument],
            &spacing,
            &engrave,
            false,
            &mut renderer,
        );

        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, .. } if text == "Flute")
        }));
    }

    #[test]
    fn draw_names_renders_symbol_placeholders_inline() {
        let mut renderer = make_test_renderer();
        let instrument = make_instrument(1, "Trumpet in B${flat}");
        let spacing = make_spacing(1);
        let engrave = make_engrave();

        draw_names(
            &Point { x: 10.0, y: 0.0 },
            &vec![&instrument],
            &spacing,
            &engrave,
            false,
            &mut renderer,
        );

        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, font_family, .. } if text == "Trumpet in B" && *font_family == crate::score::components::text::FontFamily::LibreBaskerville)
        }));
        assert!(renderer.instructions.iter().any(|instruction| {
            matches!(instruction, DrawInstruction::Text { text, font_family, point, .. } if text == "\u{E260}" && *font_family == crate::score::components::text::FontFamily::Bravura && (point.1 - 3.7).abs() < 1e-6)
        }));
        assert_eq!(renderer.instructions.len(), 2);
    }

    #[test]
    fn draw_names_debug_mode_renders_span_boxes() {
        let mut renderer = make_test_renderer();
        let instrument = make_instrument(1, "Trumpet in B${flat}");
        let spacing = make_spacing(1);
        let engrave = make_engrave();

        draw_names(
            &Point { x: 10.0, y: 0.0 },
            &vec![&instrument],
            &spacing,
            &engrave,
            true,
            &mut renderer,
        );

        assert_eq!(
            renderer
                .instructions
                .iter()
                .filter(|instruction| matches!(instruction, DrawInstruction::Rectangle { .. }))
                .count(),
            2
        );
    }
}
