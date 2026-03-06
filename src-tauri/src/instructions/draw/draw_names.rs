use crate::instructions::parse::get_vertical_spacing::VerticalSpacing;
use crate::renderer::converter::Space;
use crate::renderer::RendererContext;
use crate::score::components::measurements::Point;
use crate::score::components::text::Font;
use crate::score::engraves::Engrave;
use crate::score::instruments::Instrument;
use skia_safe::Color;

pub fn draw_names(
    origin: &Point,
    instruments: &Vec<&Instrument>,
    vertical_spacing: &VerticalSpacing,
    engrave: &Engrave,
    renderer: &mut RendererContext,
) {
    for instrument in instruments {
        let spacing = &vertical_spacing.instruments[&instrument.id];
        let top: Space = origin.y + spacing.y + (spacing.height / 2.0);

        renderer.text(
            &instrument.name(),
            &Font {
                family: engrave.instrument_name_font.family,
                align: engrave.instrument_name_font.align,
                justify: engrave.instrument_name_font.justify,
                size: engrave.instrument_name_font.size,
            },
            &Point {
                x: origin.x,
                y: top,
            },
            Color::BLACK,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_vertical_spacing::{VerticalPosition, VerticalSpacing};
    use crate::renderer::test_helpers::make_test_renderer;
    use crate::renderer::DrawInstruction;
    use crate::score::engraves::Engraves;
    use crate::score::layouts::LayoutType;
    use crate::score::instruments::defs::InstrumentType;
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
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    fn make_engrave() -> crate::score::engraves::Engrave {
        Engraves::new().by_layout_type.remove(&LayoutType::FullScore).unwrap()
    }

    fn make_spacing(instrument_id: UniqueId) -> VerticalSpacing {
        let mut instruments = FxHashMap::default();
        instruments.insert(instrument_id, VerticalPosition { y: 2.0, height: 4.0 });
        VerticalSpacing { height: 4.0, staves: FxHashMap::default(), instruments }
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
            &mut renderer,
        );

        assert_eq!(renderer.instructions.len(), 1);
        assert!(matches!(renderer.instructions[0], DrawInstruction::Text { .. }));
    }

    #[test]
    fn draw_names_with_no_instruments_produces_no_instructions() {
        let mut renderer = make_test_renderer();
        let spacing = make_spacing(1);
        let engrave = make_engrave();

        draw_names(&Point { x: 0.0, y: 0.0 }, &vec![], &spacing, &engrave, &mut renderer);

        assert!(renderer.instructions.is_empty());
    }

    #[test]
    fn draw_names_with_two_instruments_produces_two_instructions() {
        let mut renderer = make_test_renderer();
        let i1 = make_instrument(1, "Violin");
        let i2 = make_instrument(2, "Viola");
        let mut instruments = FxHashMap::default();
        instruments.insert(1u64, VerticalPosition { y: 0.0, height: 4.0 });
        instruments.insert(2u64, VerticalPosition { y: 8.0, height: 4.0 });
        let spacing = VerticalSpacing { height: 12.0, staves: FxHashMap::default(), instruments };
        let engrave = make_engrave();

        draw_names(
            &Point { x: 0.0, y: 0.0 },
            &vec![&i1, &i2],
            &spacing,
            &engrave,
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
            &mut renderer,
        );

        if let DrawInstruction::Text { text, .. } = &renderer.instructions[0] {
            assert_eq!(text, "Flute");
        } else {
            panic!("expected Text instruction");
        }
    }
}
