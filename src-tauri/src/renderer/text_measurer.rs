use super::converter::Converter;
use super::fonts::FONTS;
use crate::score::components::measurements::BoundingBox;
use crate::score::components::text::{Font, FontFamily};
use skia_safe::Font as SkiaFont;

pub struct TextMeasurer<'a> {
    converter: &'a Converter,
}

impl<'a> TextMeasurer<'a> {
    pub fn new(converter: &'a Converter) -> TextMeasurer<'a> {
        TextMeasurer { converter }
    }

    pub fn measure(&self, text: &str, font: &Font) -> BoundingBox {
        let typeface = match font.family {
            FontFamily::Bravura => &FONTS.bravura,
            FontFamily::LibreBaskerville => &FONTS.libre_baskerville,
        };

        let skia_font = SkiaFont::new(typeface.clone(), self.converter.spaces_to_px(font.size));
        let (_, rect) = skia_font.measure_str(text, None);

        BoundingBox {
            width: self.converter.px_to_spaces(rect.width()),
            height: self.converter.px_to_spaces(rect.height()),
        }
    }
}
