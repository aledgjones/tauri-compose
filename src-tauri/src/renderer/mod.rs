pub mod converter;
pub mod fonts;
pub mod text_measurer;

use crate::score::components::measurements::Point;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use converter::{Converter, Space};
use fonts::Fonts;
use skia_safe::utils::text_utils::Align as SkiaAlign;
use skia_safe::{
    surfaces, Color, EncodedImageFormat, Font as SkiaFont, Paint, Path, Point as SkiaPoint, Surface,
};

pub struct Renderer {
    surface: Surface,
    converter: Converter,
    fonts: Fonts,
}

impl Renderer {
    pub fn new(width: Space, height: Space, fonts: Fonts, converter: Converter) -> Renderer {
        let width = converter.spaces_to_px(width) as i32;
        let height = converter.spaces_to_px(height) as i32;

        let mut surface = surfaces::raster_n32_premul((width, height)).expect("surface");
        surface.canvas().clear(Color::WHITE);

        Renderer {
            surface,
            fonts,
            converter,
        }
    }

    pub fn line(&mut self, width: Space, color: Color, points: Vec<Point>) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_stroke(true);
        paint.set_stroke_width(self.converter.spaces_to_px(width));
        paint.set_color(color);

        let mut path = Path::new();
        let first = points[0].to_skia(&self.converter);

        path.move_to(first);
        for point in &points[1..] {
            path.line_to(point.to_skia(&self.converter));
        }

        self.surface.canvas().draw_path(&path, &paint);
    }

    pub fn text(&mut self, text: &str, font: &Font, point: &Point) {
        let origin = point.to_skia(&self.converter);

        let typeface = match font.family {
            FontFamily::Bravura => &self.fonts.bravura,
            FontFamily::LibreBaskerville => &self.fonts.libre_baskerville,
        };

        let skia_font = SkiaFont::new(typeface.clone(), self.converter.spaces_to_px(font.size));

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::BLACK);

        let (_, metrics) = skia_font.metrics();

        let justify = match font.justify {
            Justify::Start => SkiaAlign::Left,
            Justify::Middle => SkiaAlign::Center,
            Justify::End => SkiaAlign::Right,
        };

        let offset_y = match font.align {
            Align::Top => metrics.x_height,
            Align::Middle => (metrics.x_height) / 2.0,
            Align::Bottom => 0.0,
        };

        self.surface.canvas().draw_text_align(
            text,
            SkiaPoint {
                x: origin.x,
                y: origin.y + offset_y,
            },
            &skia_font,
            &paint,
            justify,
        );
    }

    pub fn data(&mut self) -> Vec<u8> {
        let image = self.surface.image_snapshot();
        let mut context = self.surface.direct_context();
        match image.encode(context.as_mut(), EncodedImageFormat::WEBP, None) {
            Some(data) => data.as_bytes().to_vec(),
            None => {
                println!("None");
                vec![]
            }
        }
    }
}
